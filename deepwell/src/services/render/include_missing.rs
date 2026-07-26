//! Live-compatible output for missing Wikidot include targets.

use super::literal_regions::LiteralRegionIndex;
use regex::Regex;
use std::sync::LazyLock;

static EMPTY_INCLUDE_TARGET_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\[\[include[ \t]*\]\]").unwrap());

pub(super) fn missing_include_source(page: &str, site: Option<&str>) -> String {
    let edit_url = match site {
        Some(site) => format!("http://{site}.wikidot.com/{page}/edit/true"),
        None => format!("/{page}/edit/true"),
    };
    format!(
        "[[div class=\"error-block\"]]\nIncluded page \"{page}\" does not exist ([[a href=\"{edit_url}\"]]create it now[[/a]])\n[[/div]]"
    )
}

pub(super) fn expand_empty_include_targets(wikitext: &mut String) {
    if !EMPTY_INCLUDE_TARGET_REGEX.is_match(wikitext) {
        return;
    }
    let literal_regions = LiteralRegionIndex::new_wikidot_syntax(wikitext);
    let ranges = EMPTY_INCLUDE_TARGET_REGEX
        .find_iter(wikitext)
        .filter(|matched| !literal_regions.contains(matched.start()))
        .map(|matched| matched.range())
        .collect::<Vec<_>>();
    let replacement = missing_include_source("", None);
    for range in ranges.into_iter().rev() {
        wikitext.replace_range(range, &replacement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_include_targets_become_the_live_missing_page_source() {
        let mut source = "[[include    ]]\n[[include\t]]".to_owned();
        expand_empty_include_targets(&mut source);
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
        expand_empty_include_targets(&mut source);
        assert_eq!(source, original);
    }
}
