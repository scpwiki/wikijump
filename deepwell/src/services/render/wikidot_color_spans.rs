/*
 * services/render/wikidot_color_spans.rs
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

use regex::Regex;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::LazyLock;
use uuid::Uuid;

// BND-08 correctness hardening for the existing color-span shim. This module
// deliberately does not add or broaden Wikidot color syntax.
pub(super) const SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTCOLORSPAN";

const SENTINEL_LENGTH: usize = SENTINEL_PREFIX.len() + 32 + 1;

static COLOR_SPAN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?P<hashes>#{2,})(?P<color>[A-Za-z0-9_-]+)\s*\|(?P<body>.*?)##")
        .expect("Wikidot color span regex should compile")
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProtectedWikidotColorSpan {
    pub(super) marker: String,
    pub(super) html: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WikidotBlockKind {
    Code,
    Html,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InlineOpaqueKind {
    Comment,
    Escape,
    WikidotTag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpaqueHtmlElement {
    Code,
    Pre,
    Script,
    Style,
    Textarea,
}

pub(super) fn protect(
    wikitext: &mut String,
    mut render_html: impl FnMut(&str, &str) -> String,
) -> Vec<ProtectedWikidotColorSpan> {
    if !wikitext.contains("##") {
        return Vec::new();
    }

    let opaque_ranges = opaque_source_ranges(wikitext);
    let mut opaque_index = 0;
    let mut spans = Vec::new();
    let mut output = String::with_capacity(wikitext.len());
    let mut last = 0;
    let mut matched = false;

    for captures in COLOR_SPAN_REGEX.captures_iter(wikitext) {
        let Some(whole_match) = captures.get(0) else {
            continue;
        };
        matched = true;
        output.push_str(&wikitext[last..whole_match.start()]);
        last = whole_match.end();

        while opaque_ranges
            .get(opaque_index)
            .is_some_and(|range| range.end <= whole_match.start())
        {
            opaque_index += 1;
        }
        let is_opaque = opaque_ranges.get(opaque_index).is_some_and(|range| {
            range.start <= whole_match.start() && whole_match.start() < range.end
        });
        if is_opaque {
            output.push_str(whole_match.as_str());
            continue;
        }

        let Some(color) = parse_compat_color_descriptor(
            captures.name("hashes").map_or("", |value| value.as_str()),
            captures.name("color").map_or("", |value| value.as_str()),
        ) else {
            output.push_str(whole_match.as_str());
            continue;
        };
        let body = captures.name("body").map_or("", |value| value.as_str());
        let marker = color_span_marker();
        spans.push(ProtectedWikidotColorSpan {
            marker: marker.clone(),
            html: render_html(&color, body),
        });
        output.push_str(&marker);
    }

    if !matched {
        return spans;
    }

    output.push_str(&wikitext[last..]);
    *wikitext = output;
    spans
}

pub(super) fn restore(html: String, spans: &[ProtectedWikidotColorSpan]) -> String {
    if spans.is_empty() || !html.contains(SENTINEL_PREFIX) {
        return html;
    }

    let replacements: HashMap<&str, &str> = spans
        .iter()
        .map(|span| (span.marker.as_str(), span.html.as_str()))
        .collect();
    let mut output = String::with_capacity(html.len());
    let mut opaque_elements: Vec<OpaqueHtmlElement> = Vec::new();
    let mut cursor = 0;
    let mut replaced = false;

    while cursor < html.len() {
        if let Some(raw_name) = opaque_elements
            .last()
            .and_then(|element| element.raw_text_name())
        {
            let Some(tag_start) = raw_text_closing_tag_start(&html, cursor, raw_name)
            else {
                output.push_str(&html[cursor..]);
                cursor = html.len();
                break;
            };
            output.push_str(&html[cursor..tag_start]);
            let tag_end = html_tag_end(&html, tag_start).unwrap_or(html.len());
            let tag = &html[tag_start..tag_end];
            output.push_str(tag);
            update_opaque_html_elements(tag, &mut opaque_elements);
            cursor = tag_end;
            continue;
        }

        let Some(relative_start) = html[cursor..].find('<') else {
            break;
        };
        let tag_start = cursor + relative_start;
        if opaque_elements.is_empty() {
            replaced |= restore_text_markers(
                &html[cursor..tag_start],
                &replacements,
                &mut output,
            );
        } else {
            output.push_str(&html[cursor..tag_start]);
        }

        let tag_end = html_tag_end(&html, tag_start).unwrap_or(html.len());
        let tag = &html[tag_start..tag_end];
        output.push_str(tag);
        update_opaque_html_elements(tag, &mut opaque_elements);
        cursor = tag_end;
    }

    if opaque_elements.is_empty() {
        replaced |= restore_text_markers(&html[cursor..], &replacements, &mut output);
    } else {
        output.push_str(&html[cursor..]);
    }

    if replaced { output } else { html }
}

impl OpaqueHtmlElement {
    fn raw_text_name(self) -> Option<&'static str> {
        match self {
            Self::Code | Self::Pre => None,
            Self::Script => Some("script"),
            Self::Style => Some("style"),
            Self::Textarea => Some("textarea"),
        }
    }
}

pub(super) fn parse_compat_color_descriptor<'a>(
    hashes: &str,
    descriptor: &'a str,
) -> Option<Cow<'a, str>> {
    match hashes.len() {
        2 => Some(Cow::Borrowed(descriptor)),
        3 if matches!(descriptor.len(), 3 | 6)
            && descriptor.bytes().all(|byte| byte.is_ascii_hexdigit()) =>
        {
            Some(Cow::Owned(format!("#{descriptor}")))
        }
        _ => None,
    }
}

fn color_span_marker() -> String {
    format!("{SENTINEL_PREFIX}{}X", Uuid::new_v4().as_simple())
}

fn opaque_source_ranges(source: &str) -> Vec<Range<usize>> {
    let bytes = source.as_bytes();
    let mut ranges = Vec::new();
    let mut index = 0;
    let mut opaque_start = None;
    let mut opaque_kind = None;

    while index < bytes.len() {
        if opaque_kind.is_none() && is_line_start(bytes, index) {
            let line_end = source_line_end(source, index);
            let line = &source[index..line_end];
            if let Some(block_kind) = wikidot_block_open(line) {
                let block_end = wikidot_block_end(source, line_end, block_kind);
                push_opaque_range(&mut ranges, index..block_end);
                index = block_end;
                continue;
            }
        }

        match opaque_kind {
            None if bytes[index..].starts_with(b"[!--") => {
                opaque_start = Some(index);
                opaque_kind = Some(InlineOpaqueKind::Comment);
                index += 4;
            }
            None if bytes[index..].starts_with(b"@@") => {
                opaque_start = Some(index);
                opaque_kind = Some(InlineOpaqueKind::Escape);
                index += 2;
            }
            None if starts_wikidot_tag(bytes, index) => {
                opaque_start = Some(index);
                opaque_kind = Some(InlineOpaqueKind::WikidotTag);
                index += 2;
            }
            None if starts_html_tag(bytes, index) => {
                let tag_end = html_tag_end(source, index).unwrap_or(source.len());
                push_opaque_range(&mut ranges, index..tag_end);
                index = tag_end;
            }
            None => index += 1,
            Some(InlineOpaqueKind::Comment) if bytes[index..].starts_with(b"--]") => {
                index += 3;
                push_opaque_range(
                    &mut ranges,
                    opaque_start.take().expect("comment start should exist")..index,
                );
                opaque_kind = None;
            }
            Some(InlineOpaqueKind::Escape) if bytes[index..].starts_with(b"@@") => {
                index += 2;
                push_opaque_range(
                    &mut ranges,
                    opaque_start.take().expect("escape start should exist")..index,
                );
                opaque_kind = None;
            }
            Some(InlineOpaqueKind::WikidotTag) if bytes[index..].starts_with(b"]]") => {
                index += 2;
                push_opaque_range(
                    &mut ranges,
                    opaque_start.take().expect("Wikidot tag start should exist")..index,
                );
                opaque_kind = None;
            }
            Some(_) => index += 1,
        }
    }

    if let Some(start) = opaque_start {
        push_opaque_range(&mut ranges, start..source.len());
    }
    ranges
}

fn is_line_start(bytes: &[u8], index: usize) -> bool {
    index == 0 || bytes[index - 1] == b'\n'
}

fn source_line_end(source: &str, start: usize) -> usize {
    source[start..]
        .find('\n')
        .map_or(source.len(), |offset| start + offset + 1)
}

fn wikidot_block_open(line: &str) -> Option<WikidotBlockKind> {
    let line = line.trim_start();
    if ascii_starts_with(line, "[[code") {
        Some(WikidotBlockKind::Code)
    } else if ascii_starts_with(line, "[[html") {
        Some(WikidotBlockKind::Html)
    } else {
        None
    }
}

fn wikidot_block_end(
    source: &str,
    mut line_start: usize,
    kind: WikidotBlockKind,
) -> usize {
    while line_start < source.len() {
        let line_end = source_line_end(source, line_start);
        let line = source[line_start..line_end].trim_start();
        let closing = match kind {
            WikidotBlockKind::Code => "[[/code]]",
            WikidotBlockKind::Html => "[[/html]]",
        };
        if ascii_starts_with(line, closing) {
            return line_end;
        }
        line_start = line_end;
    }
    source.len()
}

fn ascii_starts_with(value: &str, prefix: &str) -> bool {
    value
        .get(..prefix.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix))
}

fn starts_wikidot_tag(bytes: &[u8], index: usize) -> bool {
    bytes[index..].starts_with(b"[[")
        && (index == 0 || bytes[index - 1] != b'[')
        && bytes.get(index + 2) != Some(&b'[')
}

fn starts_html_tag(bytes: &[u8], index: usize) -> bool {
    if bytes.get(index) != Some(&b'<') {
        return false;
    }
    match bytes.get(index + 1) {
        Some(byte) if byte.is_ascii_alphabetic() || matches!(byte, b'!' | b'?') => true,
        Some(b'/') => bytes
            .get(index + 2)
            .is_some_and(|byte| byte.is_ascii_alphabetic()),
        _ => false,
    }
}

fn push_opaque_range(ranges: &mut Vec<Range<usize>>, range: Range<usize>) {
    if let Some(previous) = ranges.last_mut() {
        if range.start <= previous.end {
            previous.end = previous.end.max(range.end);
            return;
        }
    }
    ranges.push(range);
}

fn restore_text_markers(
    text: &str,
    replacements: &HashMap<&str, &str>,
    output: &mut String,
) -> bool {
    let mut rest = text;
    let mut replaced = false;

    while let Some(offset) = rest.find(SENTINEL_PREFIX) {
        output.push_str(&rest[..offset]);
        let marker_start = &rest[offset..];
        if let Some(candidate) = marker_start.get(..SENTINEL_LENGTH) {
            if let Some(replacement) = replacements.get(candidate) {
                output.push_str(replacement);
                rest = &marker_start[SENTINEL_LENGTH..];
                replaced = true;
                continue;
            }
        }

        output.push_str(SENTINEL_PREFIX);
        rest = &marker_start[SENTINEL_PREFIX.len()..];
    }

    output.push_str(rest);
    replaced
}

fn html_tag_end(html: &str, tag_start: usize) -> Option<usize> {
    let rest = &html[tag_start..];
    if let Some(comment) = rest.strip_prefix("<!--") {
        return comment.find("-->").map(|offset| tag_start + 4 + offset + 3);
    }
    if let Some(cdata) = rest.strip_prefix("<![CDATA[") {
        return cdata.find("]]>").map(|offset| tag_start + 9 + offset + 3);
    }

    let bytes = html.as_bytes();
    let mut quote = None;
    let mut index = tag_start + 1;
    while index < bytes.len() {
        match (quote, bytes[index]) {
            (Some(expected), byte) if byte == expected => quote = None,
            (Some(_), _) => {}
            (None, found @ (b'\'' | b'"')) => quote = Some(found),
            (None, b'>') => return Some(index + 1),
            (None, _) => {}
        }
        index += 1;
    }
    None
}

fn raw_text_closing_tag_start(html: &str, start: usize, name: &str) -> Option<usize> {
    html[start..].match_indices('<').find_map(|(offset, _)| {
        let candidate_start = start + offset;
        let candidate = &html[candidate_start..];
        let after_slash = candidate.strip_prefix("</")?;
        let matched_name = after_slash.get(..name.len())?;
        if !matched_name.eq_ignore_ascii_case(name) {
            return None;
        }
        match after_slash.as_bytes().get(name.len()) {
            Some(byte) if byte.is_ascii_whitespace() || matches!(byte, b'>' | b'/') => {
                Some(candidate_start)
            }
            _ => None,
        }
    })
}

fn update_opaque_html_elements(tag: &str, stack: &mut Vec<OpaqueHtmlElement>) {
    let Some((closing, element)) = opaque_html_tag(tag) else {
        return;
    };
    if closing {
        if stack.last() == Some(&element) {
            stack.pop();
        }
    } else {
        stack.push(element);
    }
}

fn opaque_html_tag(tag: &str) -> Option<(bool, OpaqueHtmlElement)> {
    let inner = tag.strip_prefix('<')?.trim_start();
    if inner.starts_with('!') || inner.starts_with('?') {
        return None;
    }
    let (closing, inner) = if let Some(inner) = inner.strip_prefix('/') {
        (true, inner.trim_start())
    } else {
        (false, inner)
    };
    let name_end = inner
        .bytes()
        .position(|byte| !byte.is_ascii_alphanumeric())
        .unwrap_or(inner.len());
    let name = &inner[..name_end];
    let element = match_ascii_case_insensitive_html_element(name)?;
    Some((closing, element))
}

fn match_ascii_case_insensitive_html_element(name: &str) -> Option<OpaqueHtmlElement> {
    if name.eq_ignore_ascii_case("code") {
        Some(OpaqueHtmlElement::Code)
    } else if name.eq_ignore_ascii_case("pre") {
        Some(OpaqueHtmlElement::Pre)
    } else if name.eq_ignore_ascii_case("script") {
        Some(OpaqueHtmlElement::Script)
    } else if name.eq_ignore_ascii_case("style") {
        Some(OpaqueHtmlElement::Style)
    } else if name.eq_ignore_ascii_case("textarea") {
        Some(OpaqueHtmlElement::Textarea)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ProtectedWikidotColorSpan, SENTINEL_PREFIX, parse_compat_color_descriptor,
        protect, restore,
    };

    fn marker(index: usize) -> String {
        format!("{SENTINEL_PREFIX}{index:032x}X")
    }

    fn span(index: usize, html: impl Into<String>) -> ProtectedWikidotColorSpan {
        ProtectedWikidotColorSpan {
            marker: marker(index),
            html: html.into(),
        }
    }

    #[test]
    fn color_descriptor_accepts_only_supported_hash_forms() {
        assert_eq!(
            parse_compat_color_descriptor("###", "abc").as_deref(),
            Some("#abc"),
        );
        assert_eq!(
            parse_compat_color_descriptor("###", "880808").as_deref(),
            Some("#880808"),
        );
        assert!(parse_compat_color_descriptor("###", "12345").is_none());
        assert!(parse_compat_color_descriptor("###", "gggggg").is_none());
        assert!(parse_compat_color_descriptor("####", "880808").is_none());
        assert_eq!(
            parse_compat_color_descriptor("##", "blue").as_deref(),
            Some("blue"),
        );
    }

    #[test]
    fn protection_skips_source_attributes_and_opaque_regions() {
        let mut source = concat!(
            "##red|visible##\n",
            "[[[target|##green|linked label##]]]\n",
            "[[span class=\"##red|Wikidot attribute##\"]]body[[/span]]\n",
            "<span title='quoted > ##red|HTML attribute##'>body</span>\n",
            "@@##red|escaped##@@\n",
            "[!-- ##red|commented## --]\n",
            "[[code]]\n##red|code block##\n[[/code]]\n",
            "[[html]]\n##red|HTML block##\n[[/html]]\n",
            "##blue|visible after opaque block##\n",
        )
        .to_owned();

        let spans = protect(&mut source, |color, body| {
            format!("<span style=\"color: {color}\">{body}</span>")
        });

        assert_eq!(spans.len(), 3);
        assert!(source.starts_with(&spans[0].marker));
        assert!(source.contains(&format!("[[[target|{}]]]", spans[1].marker)));
        assert!(source.ends_with(&format!("{}\n", spans[2].marker)));
        assert!(source.contains("##red|Wikidot attribute##"));
        assert!(source.contains("##red|HTML attribute##"));
        assert!(source.contains("##red|escaped##"));
        assert!(source.contains("##red|commented##"));
        assert!(source.contains("##red|code block##"));
        assert!(source.contains("##red|HTML block##"));
    }

    #[test]
    fn restoration_only_replaces_html_text_nodes() {
        let replacement = r#"<span style="color: red">red</span>"#;
        let span = span(1, replacement);
        let marker = span.marker.clone();
        let html = format!(
            concat!(
                "<p title=\"quoted > {marker}\">{marker}</p>",
                "<!-- {marker} -->",
                "<code><em>{marker}</em></code>",
                "<pre>{marker}</pre>",
                "<script>if (a < b) {{ {marker} }}</script>",
                "<script/>{marker}</script>",
                "<style>{marker}</style>",
                "<textarea>{marker}</textarea>",
                " tail {marker}",
            ),
            marker = marker,
        );

        let restored = restore(html, &[span]);

        assert!(restored.contains(&format!(r#"title="quoted > {marker}""#)));
        assert!(restored.contains(&format!("<!-- {marker} -->")));
        assert!(restored.contains(&format!("<code><em>{marker}</em></code>")));
        assert!(restored.contains(&format!("<pre>{marker}</pre>")));
        assert!(
            restored.contains(&format!("<script>if (a < b) {{ {marker} }}</script>"))
        );
        assert!(restored.contains(&format!("<script/>{marker}</script>")));
        assert!(restored.contains(&format!("<style>{marker}</style>")));
        assert!(restored.contains(&format!("<textarea>{marker}</textarea>")));
        assert_eq!(restored.matches(replacement).count(), 2);
    }

    #[test]
    fn restoration_preserves_unknown_markers_and_does_not_recurse() {
        let second_marker = marker(2);
        let spans = vec![
            span(1, format!("first contains {second_marker}")),
            span(2, "second replacement"),
        ];
        let unknown = marker(3);
        let partial = format!("{SENTINEL_PREFIX}short");
        let html = format!(
            "{} direct {} unknown {unknown} partial {partial}",
            spans[0].marker, spans[1].marker,
        );

        let restored = restore(html, &spans);

        assert!(restored.starts_with(&format!("first contains {second_marker}")));
        assert!(restored.contains("direct second replacement"));
        assert!(restored.contains(&unknown));
        assert!(restored.ends_with(&partial));
    }

    #[test]
    fn restoration_handles_a_dense_marker_stream_in_one_pass() {
        let spans = (0..2_048)
            .map(|index| span(index, format!("replacement-{index}")))
            .collect::<Vec<_>>();
        let html = spans
            .iter()
            .map(|span| span.marker.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let restored = restore(html, &spans);

        assert!(restored.starts_with("replacement-0 "));
        assert!(restored.ends_with("replacement-2047"));
        assert!(!restored.contains(SENTINEL_PREFIX));
    }
}
