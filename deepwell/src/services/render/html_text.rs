/*
 * services/render/html_text.rs
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

//! Conservative, non-reserializing discovery of HTML data-state ranges.

mod scanner;

pub(super) use scanner::{
    TagKind, is_foreign_self_closing, opaque_element_end, protected_construct_end,
    tag_kind,
};
use scanner::{element_name, is_global_tree_builder_barrier};

use std::ops::Range;

pub(in crate::services::render) const OPAQUE_ELEMENTS: &[&str] = &[
    "code",
    "iframe",
    "math",
    "noembed",
    "noframes",
    "noscript",
    "plaintext",
    "pre",
    "script",
    "style",
    "svg",
    "textarea",
    "title",
    "xmp",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct HtmlDataSegment {
    pub range: Range<usize>,
    pub continues_from_previous: bool,
}

pub(super) fn html_data_segments(html: &str) -> Vec<HtmlDataSegment> {
    html_data_segments_with_options(html, false)
}

/// Discovers data ranges while allowing canonical generated inline monospace contents.
/// Every other code element and opaque element remains protected.
pub(super) fn html_data_segments_with_inline_code(html: &str) -> Vec<HtmlDataSegment> {
    html_data_segments_with_options(html, true)
}

fn html_data_segments_with_options(
    html: &str,
    inline_code_is_data: bool,
) -> Vec<HtmlDataSegment> {
    let mut segments = Vec::new();
    let mut data_start = 0;
    let mut cursor = 0;
    let mut break_before_next_data = false;

    while let Some(relative) = html[cursor..].find('<') {
        let tag_start = cursor + relative;
        let Some(kind) = tag_kind(&html[tag_start..]) else {
            cursor = tag_start + 1;
            continue;
        };

        push_nonempty_segment(
            &mut segments,
            data_start..tag_start,
            &mut break_before_next_data,
        );
        let Some(tag_end) = protected_construct_end(html, tag_start, kind) else {
            return segments;
        };

        let tag = &html[tag_start..tag_end];
        let name = matches!(kind, TagKind::Element { .. })
            .then(|| element_name(tag))
            .flatten();
        if name.as_deref().is_some_and(is_global_tree_builder_barrier) {
            return segments;
        }
        if let TagKind::Element { closing: false } = kind
            && let Some(name) = name.as_deref()
            && OPAQUE_ELEMENTS.contains(&name)
            && !(inline_code_is_data && is_generated_inline_code(name, tag))
            && !is_foreign_self_closing(name, tag)
        {
            let Some(close_end) = opaque_element_end(html, tag_end, name) else {
                return segments;
            };
            cursor = close_end;
            data_start = close_end;
            break_before_next_data = true;
            continue;
        }

        if !matches!(kind, TagKind::Element { .. }) {
            break_before_next_data = true;
        }

        cursor = tag_end;
        data_start = tag_end;
    }

    push_nonempty_segment(
        &mut segments,
        data_start..html.len(),
        &mut break_before_next_data,
    );
    segments
}

fn is_generated_inline_code(name: &str, tag: &str) -> bool {
    name == "code" && tag == r#"<code class="wj-monospace">"#
}

fn push_nonempty_segment(
    segments: &mut Vec<HtmlDataSegment>,
    range: Range<usize>,
    break_before_next_data: &mut bool,
) {
    if !range.is_empty() {
        segments.push(HtmlDataSegment {
            range,
            continues_from_previous: !*break_before_next_data,
        });
        *break_before_next_data = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(html: &str) -> String {
        html_data_segments(html)
            .into_iter()
            .map(|segment| &html[segment.range])
            .collect::<Vec<_>>()
            .join("|")
    }

    fn continuity(html: &str) -> Vec<bool> {
        html_data_segments(html)
            .into_iter()
            .map(|segment| segment.continues_from_previous)
            .collect()
    }

    #[test]
    fn excludes_tags_and_quoted_greater_than_attributes() {
        assert_eq!(
            data(r#"before<img alt='safe > hidden'>after"#),
            "before|after",
        );
    }

    #[test]
    fn treats_quotes_in_malformed_attribute_names_as_name_characters() {
        let html = r#"before<span foo"= "x > MARKER">after"#;
        assert_eq!(data(html), "before|after");
    }

    #[test]
    fn excludes_comments_and_opaque_element_bodies() {
        let html = concat!(
            "a<!-- hidden > -->b",
            "<ScRiPt>hidden </not-script></sCrIpT>c",
            "<pre>hidden <b>too</b></pre>d",
            "<svg><text>hidden</text></svg>e",
        );
        assert_eq!(data(html), "a|b|c|d|e");
        assert_eq!(continuity(html), vec![true, false, false, false, false],);
    }

    #[test]
    fn follows_html_comment_abrupt_and_alternative_endings() {
        assert_eq!(data("a<!-->MARKER"), "a|MARKER");
        assert_eq!(data("a<!--->MARKER"), "a|MARKER");
        for html in [
            r#"a<!--><div title="<-->MARKER">b"#,
            "a<!--><script><-->MARKER</script>b",
            r#"a<!---><span title="MARKER">b"#,
            r#"a<!-- hidden --!><span title="MARKER">b"#,
        ] {
            assert_eq!(data(html), "a|b", "html: {html}");
        }
    }

    #[test]
    fn protects_bogus_comments_from_invalid_end_tag_opens() {
        for html in [
            "a</ bogus MARKER <img src=x>>b",
            "a</\tMARKER <img src=x>>b",
            "a</?MARKER <img src=x>>b",
            "a</\0MARKER <img src=x>>b",
            "a</éMARKER <img src=x>>b",
        ] {
            assert_eq!(data(html), "a|>b", "html: {html:?}");
        }

        for html in ["a</ bogus MARKER", "a</1MARKER <img"] {
            assert_eq!(data(html), "a", "html: {html:?}");
        }

        assert_eq!(data("a</ bogus hidden>MARKER"), "a|MARKER");
        assert_eq!(data("a</>MARKER"), "a</>MARKER");
        assert_eq!(data("a</"), "a</");
    }

    #[test]
    fn fails_closed_at_cdata_declarations_and_processing_instructions() {
        for html in [
            "a<![CDATA[hidden]]>MARKER",
            r#"a<!bogus "><script>MARKER</script>">"#,
            "a<!DOCTYPE html>MARKER",
            r#"a<?pi "><script>MARKER</script>">"#,
        ] {
            assert_eq!(data(html), "a", "html: {html}");
        }
    }

    #[test]
    fn ignores_opaque_parent_end_tags_inside_nested_script_data() {
        let html = "a<pre><script></pre>MARKER</script></pre>b";
        assert_eq!(data(html), "a|b");
    }

    #[test]
    fn fails_closed_at_unmodeled_tree_builder_boundaries() {
        for html in [
            "a<select><pre></select><script></pre>MARKER</script>",
            "a<template>MARKER</template>b",
            "a<object>MARKER</object>b",
            "a<applet>MARKER</applet>b",
            "a<marquee>MARKER</marquee>b",
            "a<noscript><script></noscript>MARKER</script></noscript>b",
            "a<foreignObject>MARKER</foreignObject>b",
            "a<annotation-xml>MARKER</annotation-xml>b",
        ] {
            assert_eq!(data(html), "a", "html: {html}");
        }
    }

    #[test]
    fn preserves_canonical_tables_but_not_tables_inside_opaque_matching() {
        let canonical = "<table><tbody><tr><td>cell</td></tr></tbody></table>MARKER";
        assert_eq!(data(canonical), "cell|MARKER");

        let unsafe_nesting = "a<pre><table><tr><td></pre>MARKER</td></tr></table></pre>b";
        assert_eq!(data(unsafe_nesting), "a");
        assert_eq!(data("a<pre><select></pre></select>MARKER</pre>b"), "a",);
    }

    #[test]
    fn fails_closed_at_foreign_content_integration_points() {
        let svg = "a<svg><foreignObject><textarea></svg>MARKER</textarea></foreignObject></svg>b";
        let math = "a<math><mtext><textarea></math>MARKER</textarea></mtext></math>b";
        assert_eq!(data(svg), "a");
        assert_eq!(data(math), "a");
    }

    #[test]
    fn fails_closed_instead_of_lexically_counting_nested_opaque_names() {
        assert_eq!(data("a<pre><pre></pre>MARKER</pre>b"), "a");
        assert_eq!(data("a<code><code></code>MARKER</code>b"), "a");
        assert_eq!(data("a<svg><svg></svg>MARKER</svg>b"), "a");
        assert_eq!(data("a<math><math></math>MARKER</math>b"), "a");
    }

    #[test]
    fn ignores_self_closing_slashes_on_html_opaque_elements() {
        let html = "a<script/>hidden</script>b<pre/>hidden</pre>c<code/>hidden</code>d";
        assert_eq!(data(html), "a|b|c|d");
    }

    #[test]
    fn honors_self_closing_slashes_only_on_foreign_opaque_elements() {
        assert_eq!(data("a<svg/>b<math />c"), "a|b|c");
        assert_eq!(data(r#"a<svg viewBox="0 0 1 1"/>b"#), "a|b");
        assert_eq!(data("a<math display=block />b"), "a|b");
        assert_eq!(data("a<svg / >hidden</svg>b"), "a|b");
        assert_eq!(data("a<svg data=x/>hidden</svg>b"), "a|b");
        assert_eq!(data("a<math data=/>hidden</math>b"), "a|b");
    }

    #[test]
    fn requires_a_tag_name_delimiter_for_raw_text_end_tags() {
        let html = "a<script>hidden</script.foo>still hidden</script>b<style>hidden</style:foo>still hidden</style>c";
        assert_eq!(data(html), "a|b|c");
    }

    #[test]
    fn follows_script_double_escaped_end_transitions() {
        let html = "a<script><!--<script>hidden</script>still hidden</script>b";
        assert_eq!(data(html), "a|b");
    }

    #[test]
    fn treats_plaintext_as_opaque_to_eof() {
        assert_eq!(data("a<plaintext>hidden</plaintext>b"), "a");
    }

    #[test]
    fn allows_only_canonical_generated_inline_code_data() {
        let inline = r#"a<code class="wj-monospace">visible</code>b"#;
        let authored = r#"a<code class="wj-monospace" >hidden</code>b"#;
        let uppercase = r#"a<CODE class="wj-monospace">hidden</CODE>b"#;

        assert_eq!(
            html_data_segments_with_inline_code(inline)
                .into_iter()
                .map(|segment| &inline[segment.range])
                .collect::<Vec<_>>()
                .join("|"),
            "a|visible|b",
        );
        assert_eq!(
            html_data_segments_with_inline_code(authored)
                .into_iter()
                .map(|segment| &authored[segment.range])
                .collect::<Vec<_>>()
                .join("|"),
            "a|b",
        );
        assert_eq!(
            html_data_segments_with_inline_code(uppercase)
                .into_iter()
                .map(|segment| &uppercase[segment.range])
                .collect::<Vec<_>>()
                .join("|"),
            "a|b",
        );
    }

    #[test]
    fn preserves_continuity_across_ordinary_elements_only() {
        assert_eq!(continuity("a<strong>b</strong>c"), vec![true, true, true]);
        assert_eq!(continuity("a<style>x</style>b"), vec![true, false]);
    }

    #[test]
    fn fails_closed_after_unterminated_tags_or_opaque_elements() {
        assert_eq!(data("visible <img alt='unterminated > hidden"), "visible ");
        assert_eq!(data("visible <style>hidden"), "visible ");
    }

    #[test]
    fn treats_plain_less_than_as_data() {
        assert_eq!(data("one < two and three"), "one < two and three");
    }
}
