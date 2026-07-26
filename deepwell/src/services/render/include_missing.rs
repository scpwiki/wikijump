//! Live-compatible output for missing Wikidot include targets.

use super::literal_regions::LiteralRegionIndex;
use regex::Regex;
use std::sync::LazyLock;

static EMPTY_INCLUDE_TARGET_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\[\[include[ \t]*\]\]").unwrap());
static SITE_ONLY_INCLUDE_TARGET_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[\[include[ \t]+:(?P<site>[^:\]\s]+):[ \t]*\]\]").unwrap()
});
static EMPTY_SITE_INCLUDE_TARGET_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[\[include[ \t]+::(?P<page>[^:\]\s]+)[ \t]*\]\]").unwrap()
});

pub(super) fn missing_include_source(page: &str, site: Option<&str>) -> String {
    let edit_url = match site {
        Some(site) => format!("http://{site}.wikidot.com/{page}/edit/true"),
        None => format!("/{page}/edit/true"),
    };
    format!(
        "[[div class=\"error-block\"]]\nIncluded page \"{page}\" does not exist ([[a href=\"{edit_url}\"]]create it now[[/a]])\n[[/div]]"
    )
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
}
