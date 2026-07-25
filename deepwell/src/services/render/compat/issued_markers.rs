/*
 * services/render/issued_markers.rs
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

//! One-pass restoration for unguessable markers issued by a render pass.

use super::super::html_text::html_data_segments_with_inline_code;
use std::collections::HashMap;

const WIKIDOT_COMPAT_LINK_MARKER_PREFIX: &str = "WIKIJUMPWIKIDOTCOMPATLINK";

/// Restores issued markers in HTML text while leaving tag attributes unchanged.
///
/// This scanner only accepts the Wikidot compatibility link marker prefix. The fixed, non-overlapping, non-hex-leading prefix lets a rejected hexadecimal candidate be skipped without rescanning each byte of a forged suffix. Markers consist of that prefix, one or more ASCII hexadecimal digits, and a trailing `X`. Only exact marker strings present in `replacements` are restored.
pub(in crate::services::render) fn restore_issued_html_text_markers<'a>(
    html: String,
    prefix: &str,
    replacements: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> String {
    assert_eq!(
        prefix, WIKIDOT_COMPAT_LINK_MARKER_PREFIX,
        "issued marker restoration only supports the Wikidot compatibility link prefix",
    );

    if !html.contains(prefix) {
        return html;
    }

    let replacements = replacements.into_iter().collect::<HashMap<_, _>>();
    if replacements.is_empty() {
        return html;
    }

    let mut output = String::with_capacity(html.len());
    let mut last_copied = 0usize;

    for segment in html_data_segments_with_inline_code(&html) {
        let mut cursor = segment.range.start;
        while let Some(relative_start) = html[cursor..segment.range.end].find(prefix) {
            let marker_start = cursor + relative_start;
            let digits_start = marker_start + prefix.len();
            let mut marker_end = digits_start;
            while marker_end < segment.range.end
                && html.as_bytes()[marker_end].is_ascii_hexdigit()
            {
                marker_end += 1;
            }

            if marker_end > digits_start
                && marker_end < segment.range.end
                && html.as_bytes().get(marker_end) == Some(&b'X')
                && let Some(replacement) =
                    replacements.get(&html[marker_start..=marker_end])
            {
                output.push_str(&html[last_copied..marker_start]);
                output.push_str(replacement);
                cursor = marker_end + 1;
                last_copied = cursor;
            } else {
                cursor = digits_start;
            }
        }
    }

    if last_copied == 0 {
        return html;
    }

    output.push_str(&html[last_copied..]);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    const PREFIX: &str = WIKIDOT_COMPAT_LINK_MARKER_PREFIX;

    #[test]
    fn restores_only_issued_text_markers_outside_quoted_attributes() {
        let marker = format!("{PREFIX}00000000000000000000000000000001X");
        let forged = format!("{PREFIX}00000000000000000000000000000002X");
        let html = format!(
            r#"<span data-double=">{marker}" data-single='>{marker}'>{marker}</span>{forged}"#,
        );

        let restored = restore_issued_html_text_markers(
            html,
            PREFIX,
            [(marker.as_str(), "<a href=\"javascript:;\">link</a>")],
        );

        assert!(restored.contains(&format!(
            r#"<span data-double=">{marker}" data-single='>{marker}'>"#,
        )));
        assert!(restored.ends_with(&forged));
        assert_eq!(restored.matches("<a href=").count(), 1);
    }

    #[test]
    fn comment_quotes_cannot_move_issued_markers_out_of_attributes() {
        let marker = format!("{PREFIX}00000000000000000000000000000001X");
        let html = format!(r#"<!-- " --><span title="> {marker}">{marker}</span>"#,);

        let restored = restore_issued_html_text_markers(
            html,
            PREFIX,
            [(marker.as_str(), "<a>link</a>")],
        );

        assert!(restored.contains(&format!(r#"title="> {marker}""#)));
        assert_eq!(restored.matches("<a>link</a>").count(), 1);
    }

    #[test]
    fn restores_only_canonical_inline_monospace_and_ordinary_div_text() {
        let marker = format!("{PREFIX}00000000000000000000000000000001X");
        let html = format!(
            r#"<code class="wj-monospace">{marker}</code><code class="wj-monospace" >{marker}</code><code>{marker}</code><div class="code">{marker}</div><div class="code"><pre><code>{marker}</code></pre></div><script>{marker}</script>"#,
        );

        let restored = restore_issued_html_text_markers(
            html,
            PREFIX,
            [(marker.as_str(), "<a>link</a>")],
        );

        assert_eq!(restored.matches("<a>link</a>").count(), 2);
        assert_eq!(restored.matches(&marker).count(), 4);
    }

    #[test]
    fn keeps_markers_in_raw_and_opaque_browser_states() {
        let marker = format!("{PREFIX}00000000000000000000000000000001X");
        let html = format!(
            concat!(
                "<script/>{0}</script>",
                "<pre/>{0}</pre>",
                "<script>{0}</script.foo>{0}</script>",
                "<script><!--<script>{0}</script>{0}</script>",
                "<style>{0}</style><textarea>{0}</textarea>",
                "<title>{0}</title><iframe>{0}</iframe>",
                "<svg data=x/>{0}</svg><math data=/>{0}</math>",
                "<noscript>{0}</noscript><plaintext>{0}</plaintext>{0}",
            ),
            marker,
        );

        let restored = restore_issued_html_text_markers(
            html,
            PREFIX,
            [(marker.as_str(), "<a>link</a>")],
        );

        assert!(!restored.contains("<a>link</a>"));
        assert_eq!(restored.matches(&marker).count(), 15);
    }

    #[test]
    fn keeps_markers_in_attributes_comments_and_pre_wrapped_blocks() {
        let marker = format!("{PREFIX}00000000000000000000000000000001X");
        let html = format!(
            r#"<!-- {marker} --><span title="{marker}">{marker}</span><div class="code"><pre>{marker}</pre></div>"#,
        );

        let restored = restore_issued_html_text_markers(
            html,
            PREFIX,
            [(marker.as_str(), "<a>link</a>")],
        );

        assert_eq!(restored.matches("<a>link</a>").count(), 1);
        assert_eq!(restored.matches(&marker).count(), 3);
    }

    #[test]
    fn restores_dense_markers_in_one_pass() {
        const MARKER_COUNT: usize = 10_000;

        let markers = (0..MARKER_COUNT)
            .map(|index| format!("{PREFIX}{index:032x}X"))
            .collect::<Vec<_>>();
        let replacements = (0..MARKER_COUNT)
            .map(|index| format!("<a>{index}</a>"))
            .collect::<Vec<_>>();
        let html = markers.join("\n");

        let restored = restore_issued_html_text_markers(
            html,
            PREFIX,
            markers
                .iter()
                .zip(&replacements)
                .map(|(marker, replacement)| (marker.as_str(), replacement.as_str())),
        );

        assert_eq!(restored.matches("<a>").count(), MARKER_COUNT);
        assert!(!restored.contains(PREFIX));
    }

    #[test]
    fn restores_marker_after_partial_prefix() {
        let marker = format!("{PREFIX}00000000000000000000000000000001X");
        let partial_prefix = &PREFIX[..PREFIX.len() - 1];
        let html = format!("{partial_prefix}{marker}");

        let restored = restore_issued_html_text_markers(
            html,
            PREFIX,
            [(marker.as_str(), "<a>link</a>")],
        );

        assert_eq!(restored, format!("{partial_prefix}<a>link</a>"));
    }

    #[test]
    fn restores_marker_after_long_forged_hex_candidate() {
        let marker = format!("{PREFIX}00000000000000000000000000000001X");
        let forged = format!("{PREFIX}{}X", "a".repeat(100_000));
        let html = format!("{forged}{marker}");

        let restored = restore_issued_html_text_markers(
            html,
            PREFIX,
            [(marker.as_str(), "<a>link</a>")],
        );

        assert_eq!(restored, format!("{forged}<a>link</a>"));
    }

    #[test]
    fn restores_markers_surrounded_by_unicode() {
        let marker = format!("{PREFIX}00000000000000000000000000000001X");
        let html = format!("前🍣{marker}後🦀");

        let restored = restore_issued_html_text_markers(
            html,
            PREFIX,
            [(marker.as_str(), "<a>リンク</a>")],
        );

        assert_eq!(restored, "前🍣<a>リンク</a>後🦀");
    }

    #[test]
    fn does_not_restore_marker_inserted_by_a_replacement() {
        let first = format!("{PREFIX}00000000000000000000000000000001X");
        let second = format!("{PREFIX}00000000000000000000000000000002X");

        let restored = restore_issued_html_text_markers(
            first,
            PREFIX,
            [
                (
                    format!("{PREFIX}00000000000000000000000000000001X"),
                    second.clone(),
                ),
                (second.clone(), "<a>second</a>".to_owned()),
            ]
            .iter()
            .map(|(marker, replacement)| (marker.as_str(), replacement.as_str())),
        );

        assert_eq!(restored, second);
    }

    #[test]
    #[should_panic(
        expected = "issued marker restoration only supports the Wikidot compatibility link prefix"
    )]
    fn rejects_overlapping_generic_prefix() {
        let marker = "AAA1X";

        let _ = restore_issued_html_text_markers(
            format!("A{marker}"),
            "AAA",
            [(marker, "<a>link</a>")],
        );
    }
}
