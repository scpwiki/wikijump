//! Live-compatible output for missing Wikidot include targets.

use super::compat::text_fragments::CompatTextFragments;
use super::literal_regions::LiteralRegionIndex;
use crate::error::prelude::{Error, ErrorType, ExnError, Result};
use ftml::data::PageRef;
use ftml::includes::{FetchedPage, IncludeRef};
use ftml::prelude::Includer;
use regex::{Regex, RegexBuilder};
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::sync::LazyLock;

static EMPTY_INCLUDE_TARGET_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\[\[include[ \t]+\]\]").unwrap());
static SITE_ONLY_INCLUDE_TARGET_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[\[include[ \t]+:(?P<site>[^:\]\s]+):[ \t]*\]\]").unwrap()
});
static EMPTY_SITE_INCLUDE_TARGET_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[\[include[ \t]+::(?P<page>[^:\]\s]+)[ \t]*\]\]").unwrap()
});
static INCLUDE_TARGET_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"^(?:[ \t]*>)*\[\[\s*include\s+(?P<target>[^\s|\]]+)")
        .case_insensitive(true)
        .multi_line(true)
        .build()
        .unwrap()
});

pub(super) fn missing_include_source(page: &str, site: Option<&str>) -> String {
    let message = missing_include_message(page, site);
    format!("[[div class=\"error-block\"]]\n{message}\n[[/div]]")
}

fn spaced_empty_separator_missing_include_source(
    page: &str,
    site: Option<&str>,
    compat_text: &mut CompatTextFragments,
) -> String {
    let opener = compat_text.push_escaped_html_text(r#"[[div class="error-block"]]"#);
    let message = missing_include_message(page, site);
    format!("{opener}\n{message}")
}

fn missing_include_message(page: &str, site: Option<&str>) -> String {
    let edit_url = match site {
        Some(site) => format!("http://{site}.wikidot.com/{page}/edit/true"),
        None => format!("/{page}/edit/true"),
    };
    format!(
        "Included page \"{page}\" does not exist ([[a href=\"{edit_url}\"]]create it now[[/a]])"
    )
}

pub(super) fn collect_include_display_pages(
    wikitext: &str,
) -> HashMap<PageRef, VecDeque<String>> {
    let literal_regions = LiteralRegionIndex::new_wikidot_syntax(wikitext);
    let mut pages = HashMap::<PageRef, VecDeque<String>>::new();
    for captures in INCLUDE_TARGET_REGEX.captures_iter(wikitext) {
        let matched = captures.get(0).expect("include capture has a full match");
        if literal_regions.contains(matched.start()) {
            continue;
        }
        let target = &captures["target"];
        let Ok(page_ref) = PageRef::parse(target) else {
            continue;
        };
        let raw_page = match target
            .strip_prefix(':')
            .and_then(|value| value.split_once(':'))
        {
            Some((_site, page)) => page,
            None => target,
        };
        let raw_page = raw_page
            .find(['#', '/'])
            .map_or(raw_page, |index| &raw_page[..index]);
        pages
            .entry(page_ref)
            .or_default()
            .push_back(raw_page.to_ascii_lowercase());
    }
    pages
}

pub(super) fn collect_missing_include_replacements(
    includes: &[IncludeRef<'_>],
    fetched_pages: &[Option<String>],
    include_display_pages: &mut HashMap<PageRef, VecDeque<String>>,
    compat_text: &mut CompatTextFragments,
) -> VecDeque<String> {
    includes
        .iter()
        .zip(fetched_pages)
        .rev()
        .filter_map(|(include, fetched_page)| {
            fetched_page.is_none().then(|| {
                let display_page = include_display_pages
                    .get_mut(include.page_ref())
                    .and_then(VecDeque::pop_back)
                    .unwrap_or_else(|| include.page_ref().page().to_owned());
                if is_optional_no_visible_wikidot_include(include.page_ref()) {
                    String::new()
                } else if include.has_spaced_empty_separator() {
                    spaced_empty_separator_missing_include_source(
                        &display_page,
                        include.page_ref().site(),
                        compat_text,
                    )
                } else {
                    missing_include_source(&display_page, include.page_ref().site())
                }
            })
        })
        .collect()
}

#[derive(Debug)]
pub(super) struct PreparedIncluder {
    pub(super) pages: Vec<Option<String>>,
    pub(super) missing_replacements: VecDeque<String>,
}

impl<'t> Includer<'t> for PreparedIncluder {
    type Error = ExnError;

    fn include_pages(
        &mut self,
        includes: &[IncludeRef<'t>],
    ) -> Result<Vec<FetchedPage<'t>>> {
        if includes.len() != self.pages.len() {
            return Err(Error::new(
                "include expansion returned mismatched page references",
                ErrorType::Render,
            )
            .into());
        }

        Ok(includes
            .iter()
            .zip(std::mem::take(&mut self.pages))
            .map(|(include, content)| FetchedPage {
                page_ref: include.page_ref().clone(),
                content: content.map(Cow::Owned),
            })
            .collect())
    }

    fn no_such_include(&mut self, page_ref: &PageRef) -> Result<Cow<'t, str>> {
        let Some(replacement) = self.missing_replacements.pop_front() else {
            return Ok(wikidot_no_such_include_replacement(page_ref));
        };
        Ok(Cow::Owned(replacement))
    }
}

pub(super) fn wikidot_no_such_include_replacement(
    page_ref: &PageRef,
) -> Cow<'static, str> {
    if is_optional_no_visible_wikidot_include(page_ref) {
        Cow::Borrowed("")
    } else {
        Cow::Owned(missing_include_source(page_ref.page(), page_ref.site()))
    }
}

fn is_optional_no_visible_wikidot_include(page_ref: &PageRef) -> bool {
    let Some(site) = page_ref.site() else {
        return false;
    };
    let page = page_ref.page();
    (site.eq_ignore_ascii_case("drizzles") && page.eq_ignore_ascii_case("raven"))
        || (site.eq_ignore_ascii_case("crom") && page.eq_ignore_ascii_case("pixel"))
}

pub(super) fn expand_malformed_include_targets(wikitext: &mut String) {
    if !EMPTY_INCLUDE_TARGET_REGEX.is_match(wikitext)
        && !SITE_ONLY_INCLUDE_TARGET_REGEX.is_match(wikitext)
        && !EMPTY_SITE_INCLUDE_TARGET_REGEX.is_match(wikitext)
    {
        return;
    }
    let literal_regions = LiteralRegionIndex::new_wikidot_syntax(wikitext);
    let mut replacements = EMPTY_INCLUDE_TARGET_REGEX
        .find_iter(wikitext)
        .filter(|matched| !literal_regions.contains(matched.start()))
        .map(|matched| (matched.range(), missing_include_source("", None)))
        .collect::<Vec<_>>();
    for captures in SITE_ONLY_INCLUDE_TARGET_REGEX.captures_iter(wikitext) {
        let matched = captures.get(0).expect("include capture has a full match");
        if !literal_regions.contains(matched.start()) {
            replacements.push((
                matched.range(),
                missing_include_source(&captures["site"], None),
            ));
        }
    }
    for captures in EMPTY_SITE_INCLUDE_TARGET_REGEX.captures_iter(wikitext) {
        let matched = captures.get(0).expect("include capture has a full match");
        if !literal_regions.contains(matched.start()) {
            replacements.push((
                matched.range(),
                missing_include_source(&captures["page"], None),
            ));
        }
    }
    replacements.sort_by_key(|(range, _)| range.start);
    for (range, replacement) in replacements.into_iter().rev() {
        wikitext.replace_range(range, &replacement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_include_targets_become_the_live_missing_page_source() {
        let mut source = "[[include    ]]\n[[include\t]]".to_owned();
        expand_malformed_include_targets(&mut source);
        assert_eq!(
            source.matches("Included page \"\" does not exist").count(),
            2
        );
        assert_eq!(source.matches("href=\"//edit/true\"").count(), 2);
    }

    #[test]
    fn argumentless_include_without_spacing_remains_literal() {
        let mut source = "[[include]] [[INCLUDE]]".to_owned();
        let original = source.clone();
        expand_malformed_include_targets(&mut source);
        assert_eq!(source, original);
    }

    #[test]
    fn empty_include_targets_inside_literal_blocks_remain_literal() {
        let mut source = "[[code]]\n[[include ]]\n[[/code]]".to_owned();
        let original = source.clone();
        expand_malformed_include_targets(&mut source);
        assert_eq!(source, original);
    }

    #[test]
    fn incomplete_cross_site_targets_fall_back_to_local_page_names() {
        let mut source = "[[include :scp-wiki:]]\n[[include ::page]]".to_owned();
        expand_malformed_include_targets(&mut source);
        assert!(source.contains("Included page \"scp-wiki\" does not exist"));
        assert!(source.contains("href=\"/scp-wiki/edit/true\""));
        assert!(source.contains("Included page \"page\" does not exist"));
        assert!(source.contains("href=\"/page/edit/true\""));
    }

    #[test]
    fn include_display_pages_preserve_raw_colons_separately_from_lookup_keys() {
        let pages = collect_include_display_pages(
            "[[include :scp-wiki:deleted:protected:component:magic]]",
        );
        let canonical =
            PageRef::page_and_site("scp-wiki", "deleted:protected:component:magic");
        assert_eq!(
            pages
                .get(&canonical)
                .and_then(|values| values.front())
                .map(String::as_str),
            Some("deleted:protected:component:magic"),
        );
        assert_eq!(canonical.page(), "deleted-protected-component:magic");
    }

    #[test]
    fn include_display_pages_are_lowercase_and_follow_reverse_substitution_order() {
        let source = "A\n[[include B]]\nC\n[[include D]]\nE\n[[include F]]\nG";
        let mut display_pages = collect_include_display_pages(source);
        let includes = ["B", "D", "F"]
            .into_iter()
            .map(|page| IncludeRef::page_only(PageRef::page_only(page)))
            .collect::<Vec<_>>();
        let fetched_pages = vec![None, None, None];

        let mut compat_text = CompatTextFragments::new(source);
        let replacements = collect_missing_include_replacements(
            &includes,
            &fetched_pages,
            &mut display_pages,
            &mut compat_text,
        );
        assert!(replacements[0].contains("Included page \"f\""));
        assert!(replacements[1].contains("Included page \"d\""));
        assert!(replacements[2].contains("Included page \"b\""));
    }

    #[test]
    fn spaced_empty_separator_uses_protected_literal_opener() {
        let source = "[[include PAGE | ]]";
        let mut display_pages = collect_include_display_pages(source);
        let includes = vec![
            IncludeRef::page_only(PageRef::page_only("PAGE"))
                .with_spaced_empty_separator(true),
        ];
        let mut compat_text = CompatTextFragments::new(source);

        let replacements = collect_missing_include_replacements(
            &includes,
            &[None],
            &mut display_pages,
            &mut compat_text,
        );
        let replacement = replacements.front().expect("replacement exists");

        assert!(!replacement.contains("[[div class=\"error-block\"]]"));
        assert!(replacement.contains("Included page \"page\""));
        assert!(!replacement.contains("[[/div]]"));
        assert!(
            compat_text
                .restore(replacement)
                .starts_with("[[div class=&quot;error-block&quot;]]\nIncluded page")
        );
    }
}
