/*
 * services/render/wikidot_inline_markers.rs
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::render) enum WikidotCompatInlineMarkerKind {
    Color,
    Italic,
    Underline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::render) struct WikidotCompatInlineMarker {
    pub(in crate::services::render) start: usize,
    pub(in crate::services::render) end: usize,
    pub(in crate::services::render) kind: WikidotCompatInlineMarkerKind,
}

/// Return the earliest complete compatibility marker in `value`.
///
/// Candidate discovery advances monotonically through the string. Invalid
/// candidates never restart the search from the beginning, which keeps a call
/// linear in the supplied text instead of rescanning it once per marker kind.
pub(in crate::services::render) fn next_wikidot_compat_inline_marker(
    value: &str,
) -> Option<WikidotCompatInlineMarker> {
    let mut offset = 0;

    while let Some((start, kind)) = find_next_inline_delimiter(value, offset) {
        let (marker, next_offset) = match kind {
            WikidotCompatInlineMarkerKind::Color => {
                (match_color_marker(value, start), start + 2)
            }
            WikidotCompatInlineMarkerKind::Italic => match_delimited_marker(
                value,
                start,
                "//",
                WikidotCompatInlineMarkerKind::Italic,
            ),
            WikidotCompatInlineMarkerKind::Underline => match_delimited_marker(
                value,
                start,
                "__",
                WikidotCompatInlineMarkerKind::Underline,
            ),
        };

        if marker.is_some() {
            return marker;
        }

        offset = next_offset;
    }

    None
}

fn find_next_inline_delimiter(
    value: &str,
    offset: usize,
) -> Option<(usize, WikidotCompatInlineMarkerKind)> {
    value[offset..].char_indices().find_map(|(relative, _)| {
        let start = offset + relative;
        let rest = &value[start..];
        if rest.starts_with("##") {
            Some((start, WikidotCompatInlineMarkerKind::Color))
        } else if rest.starts_with("//") {
            Some((start, WikidotCompatInlineMarkerKind::Italic))
        } else if rest.starts_with("__") {
            Some((start, WikidotCompatInlineMarkerKind::Underline))
        } else {
            None
        }
    })
}

fn match_color_marker(value: &str, start: usize) -> Option<WikidotCompatInlineMarker> {
    let marker_start = &value[start + 2..];
    let pipe_relative = find_color_pipe(marker_start)?;
    let color = marker_start[..pipe_relative].trim();
    if !valid_color_value(color) {
        return None;
    }
    let content_start = start + 2 + pipe_relative + 1;
    let end_relative = value[content_start..].find("##")?;

    Some(WikidotCompatInlineMarker {
        start,
        end: content_start + end_relative + 2,
        kind: WikidotCompatInlineMarkerKind::Color,
    })
}

fn match_delimited_marker(
    value: &str,
    start: usize,
    delimiter: &str,
    kind: WikidotCompatInlineMarkerKind,
) -> (Option<WikidotCompatInlineMarker>, usize) {
    if delimiter == "//" && value[..start].ends_with(':') {
        return (None, start + delimiter.len());
    }
    let content_start = start + delimiter.len();
    let Some(end_relative) = value[content_start..].find(delimiter) else {
        return (None, start + delimiter.len());
    };
    if end_relative == 0 {
        return (None, content_start + delimiter.len());
    }

    (
        Some(WikidotCompatInlineMarker {
            start,
            end: content_start + end_relative + delimiter.len(),
            kind,
        }),
        start + delimiter.len(),
    )
}

fn find_color_pipe(value: &str) -> Option<usize> {
    value.char_indices().find_map(|(offset, character)| {
        if character == '|' {
            Some(Some(offset))
        } else if value[offset..].starts_with("##") {
            Some(None)
        } else {
            None
        }
    })?
}

fn valid_color_value(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '#')
}

#[cfg(test)]
mod tests {
    use super::{WikidotCompatInlineMarkerKind, next_wikidot_compat_inline_marker};

    #[test]
    fn finds_the_earliest_complete_marker_after_invalid_candidates() {
        let value = "http://example.test ##bad## __underlined__ //italic//";
        let marker = next_wikidot_compat_inline_marker(value).expect("marker exists");

        assert_eq!(marker.kind, WikidotCompatInlineMarkerKind::Underline);
        assert_eq!(&value[marker.start..marker.end], "__underlined__");
    }

    #[test]
    fn preserves_byte_offsets_after_unicode() {
        let value = "雪 //italic//";
        let marker = next_wikidot_compat_inline_marker(value).expect("marker exists");

        assert_eq!(marker.kind, WikidotCompatInlineMarkerKind::Italic);
        assert_eq!(&value[marker.start..marker.end], "//italic//");
    }

    #[test]
    fn rejects_incomplete_and_empty_markers() {
        for value in [
            "##red|missing close",
            "////",
            "____",
            "////#//#",
            "____x__x",
        ] {
            assert_eq!(next_wikidot_compat_inline_marker(value), None, "{value:?}");
        }
    }

    #[test]
    fn does_not_span_a_color_prefix_across_another_color_delimiter() {
        let value = "##bad## ##red|ok##";
        let marker = next_wikidot_compat_inline_marker(value).expect("marker exists");

        assert_eq!(marker.kind, WikidotCompatInlineMarkerKind::Color);
        assert_eq!(&value[marker.start..marker.end], "##red|ok##");
    }
}
