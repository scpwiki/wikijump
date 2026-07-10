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

use std::ops::Range;

const OPAQUE_ELEMENTS: &[&str] = &[
    "code", "iframe", "math", "noembed", "noframes", "pre", "script", "style", "svg",
    "textarea", "title", "xmp",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct HtmlDataSegment {
    pub range: Range<usize>,
    pub continues_from_previous: bool,
}

pub(super) fn html_data_segments(html: &str) -> Vec<HtmlDataSegment> {
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

        if let TagKind::Element { closing: false } = kind
            && let Some(name) = element_name(&html[tag_start..tag_end])
            && OPAQUE_ELEMENTS.contains(&name.as_str())
            && !is_self_closing(&html[tag_start..tag_end])
        {
            let Some(close_end) = opaque_element_end(html, tag_end, &name) else {
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

#[derive(Clone, Copy, Debug)]
enum TagKind {
    Comment,
    Cdata,
    Declaration,
    Element { closing: bool },
}

fn tag_kind(input: &str) -> Option<TagKind> {
    let bytes = input.as_bytes();
    debug_assert_eq!(bytes.first(), Some(&b'<'));
    match bytes.get(1).copied()? {
        b'!' if input.starts_with("<!--") => Some(TagKind::Comment),
        b'!' if input.starts_with("<![CDATA[") => Some(TagKind::Cdata),
        b'!' | b'?' => Some(TagKind::Declaration),
        b'/' if bytes.get(2).is_some_and(|byte| byte.is_ascii_alphabetic()) => {
            Some(TagKind::Element { closing: true })
        }
        byte if byte.is_ascii_alphabetic() => Some(TagKind::Element { closing: false }),
        _ => None,
    }
}

fn protected_construct_end(html: &str, start: usize, kind: TagKind) -> Option<usize> {
    match kind {
        TagKind::Comment => html[start + 4..]
            .find("-->")
            .map(|offset| start + 4 + offset + 3),
        TagKind::Cdata => html[start + 9..]
            .find("]]>")
            .map(|offset| start + 9 + offset + 3),
        TagKind::Declaration | TagKind::Element { .. } => {
            quote_aware_tag_end(html, start)
        }
    }
}

fn quote_aware_tag_end(html: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    for (offset, byte) in html.as_bytes()[start + 1..].iter().copied().enumerate() {
        match (quote, byte) {
            (Some(active), current) if current == active => quote = None,
            (None, b'\'' | b'"') => quote = Some(byte),
            (None, b'>') => return Some(start + 1 + offset + 1),
            _ => {}
        }
    }
    None
}

fn opaque_element_end(html: &str, mut cursor: usize, name: &str) -> Option<usize> {
    let raw_text = matches!(
        name,
        "iframe"
            | "noembed"
            | "noframes"
            | "script"
            | "style"
            | "textarea"
            | "title"
            | "xmp"
    );
    if raw_text {
        while let Some(relative) = html[cursor..].find('<') {
            let start = cursor + relative;
            if matches!(
                tag_kind(&html[start..]),
                Some(TagKind::Element { closing: true })
            ) {
                let end = quote_aware_tag_end(html, start)?;
                if element_name(&html[start..end]).as_deref() == Some(name) {
                    return Some(end);
                }
            }
            cursor = start + 1;
        }
        return None;
    }

    let mut depth = 1usize;

    while let Some(relative) = html[cursor..].find('<') {
        let start = cursor + relative;
        let Some(kind) = tag_kind(&html[start..]) else {
            cursor = start + 1;
            continue;
        };
        let end = protected_construct_end(html, start, kind)?;
        if let TagKind::Element { closing } = kind
            && element_name(&html[start..end]).as_deref() == Some(name)
        {
            if closing {
                depth -= 1;
                if depth == 0 {
                    return Some(end);
                }
            } else if !is_self_closing(&html[start..end]) {
                depth += 1;
            }
        }
        cursor = end;
    }
    None
}

fn element_name(tag: &str) -> Option<String> {
    let bytes = tag.as_bytes();
    let mut start = 1;
    if bytes.get(start) == Some(&b'/') {
        start += 1;
    }
    let end = bytes[start..]
        .iter()
        .position(|byte| !byte.is_ascii_alphanumeric() && !matches!(*byte, b'-' | b':'))
        .map_or(bytes.len(), |offset| start + offset);
    (end > start).then(|| tag[start..end].to_ascii_lowercase())
}

fn is_self_closing(tag: &str) -> bool {
    tag[..tag.len() - 1].trim_end().ends_with('/')
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
    fn excludes_comments_cdata_and_opaque_element_bodies() {
        let html = concat!(
            "a<!-- hidden > -->b<![CDATA[hidden <tag>]]>c",
            "<ScRiPt>hidden </not-script></sCrIpT>d",
            "<pre>hidden <b>too</b></pre>e",
            "<svg><text>hidden</text></svg>f",
        );
        assert_eq!(data(html), "a|b|c|d|e|f");
        assert_eq!(
            continuity(html),
            vec![true, false, false, false, false, false],
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
