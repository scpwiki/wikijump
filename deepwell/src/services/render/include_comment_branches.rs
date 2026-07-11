/*
 * services/render/include_comment_branches.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

const UNRESOLVED_BRANCH_OPEN_PREFIX: &str = "[!-- {$";
const COMMENT_BOUNDARY_MARKER: &str = "[!----]";
const SELECTED_BRANCH_MARKER: &str = "[!-- --]";

/// Removes Wikidot include-variable comment branches that remain hidden after
/// variable substitution.
///
/// Wikidot components use both compact comment boundaries (`[!----]`) and
/// empty-comment boundaries (`[!-- --]`). A branch without either boundary
/// is preserved literally so malformed input fails closed during FTML parsing.
pub(super) fn remove_unresolved_include_comment_branches(wikitext: &mut String) {
    let mut output = String::with_capacity(wikitext.len());
    let mut unresolved_branch = None::<UnresolvedBranch>;

    for line in wikitext.split_inclusive('\n') {
        let marker = line.trim();

        if let Some(branch) = unresolved_branch.as_mut() {
            branch.source.push_str(line);
            if is_unresolved_branch_open(marker) {
                branch.nested_opener = true;
            } else if !branch.nested_opener && is_boundary(marker) {
                unresolved_branch = None;
            }
            continue;
        }

        if is_unresolved_branch_open(marker) {
            unresolved_branch = Some(UnresolvedBranch {
                source: line.to_owned(),
                nested_opener: false,
            });
        } else if !is_boundary(marker) {
            output.push_str(line);
        }
    }

    if let Some(branch) = unresolved_branch {
        output.push_str(&branch.source);
    }

    *wikitext = output;
}

struct UnresolvedBranch {
    source: String,
    nested_opener: bool,
}

fn is_unresolved_branch_open(line: &str) -> bool {
    line.starts_with(UNRESOLVED_BRANCH_OPEN_PREFIX)
}

fn is_boundary(line: &str) -> bool {
    matches!(line, COMMENT_BOUNDARY_MARKER | SELECTED_BRANCH_MARKER)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ftml::data::{PageInfo, ScoreValue};
    use ftml::layout::Layout;
    use ftml::render::Render;
    use ftml::render::html::HtmlRender;
    use ftml::settings::{WikitextMode, WikitextSettings};
    use std::borrow::Cow;

    #[test]
    fn removes_compact_unselected_include_comment_branches() {
        let mut wikitext = concat!(
            "Before\n",
            "[!----]\n",
            "[!-- {$inc-hidden}\n",
            "Hidden branch %%title%%\n",
            "[!----]\n",
            "[!-- --]\n",
            "Selected branch body\n",
            "[!----]\n",
            "[!-- {$inc-other}\n",
            "Other hidden branch\n",
            "[!----]\n",
            "After\n",
        )
        .to_owned();

        remove_unresolved_include_comment_branches(&mut wikitext);

        assert_eq!(wikitext, "Before\nSelected branch body\nAfter\n");
    }

    #[test]
    fn empty_comment_boundaries_preserve_split_author_label_divs() {
        let mut wikitext = concat!(
            "[!-- --]\n",
            "[[div_ class=\"authorlink-wrapper\"]]\n",
            "[# Ecronak]\n",
            "[[div class=\"authorbox\"]]\n",
            "[[div class=\"authorcontent\"]]\n",
            "[!-- {$end}]\n",
            "[[/div]]\n",
            "[[/div]]\n",
            "[[/div]]\n",
            "[!-- --]\n",
            "Enjoyed the skip? Give some of my other works a look!\n",
            "[!-- {$start}]\n",
            "[[div_ class=\"authorlink-wrapper\"]]\n",
            "[# {$name}]\n",
            "[[div class=\"authorbox\"]]\n",
            "[[div class=\"authorcontent\"]]\n",
            "[!-- --]\n",
            "[[/div]]\n",
            "[[/div]]\n",
            "[[/div]]\n",
            "[!-- --]\n",
        )
        .to_owned();

        remove_unresolved_include_comment_branches(&mut wikitext);

        assert!(!wikitext.contains("[!-- {$start}]"), "{wikitext}");
        assert!(!wikitext.contains("[!-- {$end}]"), "{wikitext}");
        assert!(!wikitext.contains("[!-- --]"), "{wikitext}");
        assert!(!wikitext.contains("[!----]"), "{wikitext}");

        let page_info = PageInfo {
            page: Cow::Borrowed("scp-6670"),
            category: None,
            site: Cow::Borrowed("scp-wiki"),
            title: Cow::Borrowed("SCP-6670"),
            alt_title: None,
            score: ScoreValue::Integer(0),
            tags: Vec::new(),
            language: Cow::Borrowed("en"),
        };
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        ftml::preprocess(&mut wikitext);
        let tokens = ftml::tokenize(&wikitext);
        let result = ftml::parse(&tokens, &page_info, &settings);
        let (tree, errors) = result.into();
        assert!(errors.is_empty(), "{errors:?}\n{wikitext}");
        let html = HtmlRender.render(&tree, &page_info, &settings).body;

        assert!(
            html.contains(
                r#"<div class="authorlink-wrapper"><a href="javascript:;">Ecronak</a><div class="authorbox"><div class="authorcontent">"#,
            ),
            "{html}",
        );
        assert!(html.contains("Enjoyed the skip?"), "{html}");
        assert!(!html.contains("[[div"), "{html}");
        assert!(!html.contains("[[/div]]"), "{html}");
    }

    #[test]
    fn preserves_unresolved_branch_without_a_boundary() {
        let original = concat!("Before\n", "[!-- {$end}]\n", "[[/div]]\n", "After\n",);
        let mut wikitext = original.to_owned();

        remove_unresolved_include_comment_branches(&mut wikitext);

        assert_eq!(wikitext, original);
    }

    #[test]
    fn preserves_inline_marker_like_text() {
        let original =
            concat!("Before [!-- {$end}] inline\n", "After [!-- --] inline\n",);
        let mut wikitext = original.to_owned();

        remove_unresolved_include_comment_branches(&mut wikitext);

        assert_eq!(wikitext, original);
    }

    #[test]
    fn inline_boundary_text_does_not_close_an_unresolved_branch() {
        let original = concat!(
            "Before\n",
            "[!-- {$end}]\n",
            "This mentions [!-- --] inline.\n",
            "After\n",
        );
        let mut wikitext = original.to_owned();

        remove_unresolved_include_comment_branches(&mut wikitext);

        assert_eq!(wikitext, original);
    }

    #[test]
    fn removes_consecutive_unresolved_branches_independently() {
        let mut wikitext = concat!(
            "Before\n",
            "[!-- {$first}]\n",
            "First hidden branch\n",
            "[!-- --]\n",
            "[!-- {$second}]\n",
            "Second hidden branch\n",
            "[!----]\n",
            "After\n",
        )
        .to_owned();

        remove_unresolved_include_comment_branches(&mut wikitext);

        assert_eq!(wikitext, "Before\nAfter\n");
    }

    #[test]
    fn preserves_nested_unresolved_branches_as_malformed_input() {
        let original = concat!(
            "Before\n",
            "[!-- {$outer}]\n",
            "Outer body\n",
            "[!-- {$inner}]\n",
            "Inner body\n",
            "[!-- --]\n",
            "After first boundary\n",
            "[!----]\n",
        );
        let mut wikitext = original.to_owned();

        remove_unresolved_include_comment_branches(&mut wikitext);

        assert_eq!(wikitext, original);
    }
}
