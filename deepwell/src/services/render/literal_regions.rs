/*
 * services/render/literal_regions.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use std::ops::Range;

/// Precomputed literal regions for compatibility transforms.
///
/// Building the index is linear in the input length. Membership checks use a
/// binary search, avoiding a full prefix rescan for every parser function or
/// conditional on component-heavy pages.
#[derive(Debug, Default)]
pub(super) struct LiteralRegionIndex {
    ranges: Vec<Range<usize>>,
}

impl LiteralRegionIndex {
    pub(super) fn new(source: &str) -> Self {
        Self::build(source, true)
    }

    pub(super) fn new_wikidot_syntax(source: &str) -> Self {
        Self::build(source, false)
    }

    /// Literal and tag regions where a pre-FTML compatibility protector must
    /// not recognize authored syntax.
    pub(super) fn new_wikidot_protection(source: &str) -> Self {
        let mut index = Self::build(source, false);
        collect_wikidot_tag_ranges(source, &mut index.ranges);
        collect_html_tag_ranges(source, &mut index.ranges);
        index.merge_ranges();
        index
    }

    /// Rendered HTML regions where trusted-fragment markers must remain text.
    #[cfg(test)]
    pub(super) fn new_html_restoration(source: &str) -> Self {
        let mut index = Self::build(source, true);
        collect_html_tag_ranges(source, &mut index.ranges);
        collect_paired_ranges(source, "<!--", "-->", &mut index.ranges);
        index.merge_ranges();
        index
    }

    /// Rendered HTML regions where color-fragment markers must remain text.
    ///
    /// Inline Wikidot monospace permits color syntax, so standalone `<code>`
    /// contents are not literal for this restoration pass. Block code remains
    /// protected by its enclosing `<pre>` or `<div class="code">` range.
    pub(super) fn new_html_color_restoration(source: &str) -> Self {
        let mut index = Self::build(source, false);
        collect_html_literal_ranges(source, &mut index.ranges, false);
        collect_html_tag_ranges(source, &mut index.ranges);
        collect_paired_ranges(source, "<!--", "-->", &mut index.ranges);
        index.merge_ranges();
        index
    }

    fn build(source: &str, include_rendered_html: bool) -> Self {
        let mut ranges = Vec::new();
        collect_wikidot_block_ranges(source, &mut ranges);
        collect_paired_ranges(source, "@@", "@@", &mut ranges);
        collect_paired_ranges(source, "[!--", "--]", &mut ranges);
        if include_rendered_html {
            collect_html_literal_ranges(source, &mut ranges, true);
        }

        let mut index = Self { ranges };
        index.merge_ranges();
        index
    }

    fn merge_ranges(&mut self) {
        self.ranges
            .sort_unstable_by_key(|range| (range.start, range.end));
        let mut merged: Vec<Range<usize>> = Vec::with_capacity(self.ranges.len());
        for range in self
            .ranges
            .drain(..)
            .filter(|range| range.start < range.end)
        {
            if let Some(previous) = merged.last_mut()
                && range.start <= previous.end
            {
                previous.end = previous.end.max(range.end);
            } else {
                merged.push(range);
            }
        }
        self.ranges = merged;
    }

    pub(super) fn contains(&self, offset: usize) -> bool {
        let insertion = self.ranges.partition_point(|range| range.start <= offset);
        insertion > 0 && offset < self.ranges[insertion - 1].end
    }
}

/// Line ranges following a valid Wikidot native quote prefix.
///
/// Construction is linear and membership uses binary search so compatibility
/// protectors can share quote-context checks without rescanning prefixes for
/// every syntax candidate.
#[derive(Debug, Default)]
pub(super) struct WikidotNativeQuoteIndex {
    ranges: Vec<Range<usize>>,
}

impl WikidotNativeQuoteIndex {
    pub(super) fn new(source: &str) -> Self {
        let mut ranges = Vec::new();
        let mut line_start = 0;
        for line in source.split_inclusive('\n') {
            let bytes = line.as_bytes();
            let mut cursor = 0;
            while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
                cursor += 1;
            }
            let quote_start = cursor;
            while bytes.get(cursor) == Some(&b'>') {
                cursor += 1;
            }
            if cursor > quote_start && matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
                ranges.push(line_start + cursor + 1..line_start + line.len());
            }
            line_start += line.len();
        }
        Self { ranges }
    }

    pub(super) fn contains(&self, offset: usize) -> bool {
        let insertion = self.ranges.partition_point(|range| range.start <= offset);
        insertion > 0 && offset < self.ranges[insertion - 1].end
    }
}

fn collect_wikidot_tag_ranges(source: &str, ranges: &mut Vec<Range<usize>>) {
    let mut line_start = 0usize;
    for line in source.split_inclusive('\n') {
        let bytes = line.as_bytes();
        let mut cursor = 0usize;
        while let Some(offset) = line[cursor..].find("[[") {
            let relative_start = cursor + offset;
            let start = line_start + relative_start;
            let mut relative_end = relative_start + 2;
            let mut quote = None;
            while relative_end + 1 < bytes.len() {
                match (quote, bytes[relative_end]) {
                    (Some(expected), actual) if expected == actual => {
                        quote = None;
                        relative_end += 1;
                    }
                    (None, b'\'' | b'"') => {
                        quote = Some(bytes[relative_end]);
                        relative_end += 1;
                    }
                    (None, b']') if bytes[relative_end + 1] == b']' => {
                        relative_end += 2;
                        break;
                    }
                    _ => relative_end += 1,
                }
            }
            if relative_end + 1 >= bytes.len() {
                relative_end = line.len();
            }
            ranges.push(start..line_start + relative_end);
            cursor = relative_end;
        }
        line_start += line.len();
    }
}

fn collect_html_tag_ranges(source: &str, ranges: &mut Vec<Range<usize>>) {
    let mut cursor = 0usize;
    while let Some(relative_start) = source[cursor..].find('<') {
        let start = cursor + relative_start;
        let Some(end) = html_tag_end(source, start) else {
            ranges.push(start..source.len());
            break;
        };
        ranges.push(start..end);
        cursor = end;
    }
}

#[derive(Clone, Copy)]
enum WikidotLiteralBlock {
    Code,
    Html,
}

impl WikidotLiteralBlock {
    fn closing_marker(self) -> &'static str {
        match self {
            Self::Code => "[[/code]]",
            Self::Html => "[[/html]]",
        }
    }
}

fn collect_wikidot_block_ranges(source: &str, ranges: &mut Vec<Range<usize>>) {
    let mut offset = 0usize;
    let mut active: Option<(WikidotLiteralBlock, usize)> = None;

    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let marker_start = offset + (line.len() - trimmed.len());
        let marker = trimmed.to_ascii_lowercase();
        if let Some((kind, start)) = active {
            if marker.starts_with(kind.closing_marker()) {
                ranges.push(start..marker_start + kind.closing_marker().len());
                active = None;
            }
        } else if marker.starts_with("[[code") {
            active = Some((WikidotLiteralBlock::Code, marker_start));
        } else if marker.starts_with("[[html") {
            active = Some((WikidotLiteralBlock::Html, marker_start));
        }
        offset += line.len();
    }

    if let Some((_, start)) = active {
        ranges.push(start..source.len());
    }
}

fn collect_paired_ranges(
    source: &str,
    opening: &str,
    closing: &str,
    ranges: &mut Vec<Range<usize>>,
) {
    let mut cursor = 0usize;
    while let Some(relative_start) = source[cursor..].find(opening) {
        let start = cursor + relative_start;
        let body_start = start + opening.len();
        let end = source[body_start..]
            .find(closing)
            .map_or(source.len(), |relative_end| {
                body_start + relative_end + closing.len()
            });
        ranges.push(start..end);
        if end == source.len() {
            break;
        }
        cursor = end;
    }
}

fn collect_html_literal_ranges(
    source: &str,
    ranges: &mut Vec<Range<usize>>,
    standalone_code_is_literal: bool,
) {
    let mut cursor = 0usize;
    let mut active: Option<(String, usize, usize)> = None;

    while let Some(relative_start) = source[cursor..].find('<') {
        let tag_start = cursor + relative_start;
        let Some(tag_end) = html_tag_end(source, tag_start) else {
            break;
        };
        let tag = &source[tag_start..tag_end];
        let Some((name, closing, self_closing)) = html_tag_name(tag) else {
            cursor = tag_end;
            continue;
        };

        if let Some((root_name, content_start, depth)) = active.as_mut() {
            if name == *root_name {
                if closing {
                    *depth = depth.saturating_sub(1);
                    if *depth == 0 {
                        ranges.push(*content_start..tag_start);
                        active = None;
                    }
                } else if !self_closing {
                    *depth += 1;
                }
            }
        } else if !closing
            && !self_closing
            && html_tag_starts_literal(&name, tag, standalone_code_is_literal)
        {
            active = Some((name, tag_end, 1));
        }
        cursor = tag_end;
    }

    if let Some((_, content_start, _)) = active {
        ranges.push(content_start..source.len());
    }
}

fn html_tag_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut cursor = start + 1;
    let mut quote = None;
    while let Some(byte) = bytes.get(cursor).copied() {
        match (quote, byte) {
            (Some(expected), actual) if expected == actual => quote = None,
            (None, b'\'' | b'"') => quote = Some(byte),
            (None, b'>') => return Some(cursor + 1),
            _ => {}
        }
        cursor += 1;
    }
    None
}

fn html_tag_name(tag: &str) -> Option<(String, bool, bool)> {
    let inner = tag.strip_prefix('<')?.strip_suffix('>')?.trim();
    if inner.is_empty() || inner.starts_with('!') || inner.starts_with('?') {
        return None;
    }
    let closing = inner.starts_with('/');
    let inner = if closing {
        inner[1..].trim_start()
    } else {
        inner
    };
    let name = inner
        .split(|character: char| {
            character.is_ascii_whitespace() || character == '/' || character == '>'
        })
        .next()?
        .to_ascii_lowercase();
    (!name.is_empty()).then(|| (name, closing, inner.ends_with('/')))
}

fn html_tag_starts_literal(
    name: &str,
    tag: &str,
    standalone_code_is_literal: bool,
) -> bool {
    if name == "code" {
        return standalone_code_is_literal;
    }
    if matches!(name, "pre" | "script" | "style" | "textarea") {
        return true;
    }
    if name != "div" {
        return false;
    }
    let lower = tag.to_ascii_lowercase();
    lower.contains(r#"class="code""#) || lower.contains("class='code'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexes_wikidot_and_rendered_html_literal_regions() {
        let source = concat!(
            "outside\n",
            "[[code]]\ncode-example\n[[/code]]\n",
            "@@escaped-example@@\n",
            "[!-- comment-example --]\n",
            "[[html]]\nhtml-example\n[[/html]]\n",
            "<pre>pre-example</pre>\n",
            r#"<div class="code"><div>panel-example</div></div>"#,
        );
        let index = LiteralRegionIndex::new(source);

        assert!(!index.contains(source.find("outside").unwrap()));
        for needle in [
            "code-example",
            "escaped-example",
            "comment-example",
            "html-example",
            "pre-example",
            "panel-example",
        ] {
            assert!(index.contains(source.find(needle).unwrap()), "{needle}");
        }
    }

    #[test]
    fn color_restoration_treats_only_standalone_code_as_non_literal() {
        let source = concat!(
            r#"<code class="wj-monospace">inline-marker</code>"#,
            "\n<pre><code>pre-marker</code></pre>",
            "\n<div class=\"code\"><code>panel-marker</code></div>",
            "\n<script>script-marker</script>",
        );
        let index = LiteralRegionIndex::new_html_color_restoration(source);

        assert!(!index.contains(source.find("inline-marker").unwrap()));
        for marker in ["pre-marker", "panel-marker", "script-marker"] {
            assert!(index.contains(source.find(marker).unwrap()), "{marker}");
        }
    }

    #[test]
    fn identifies_valid_wikidot_native_quote_lines() {
        for source in [
            "> [[module CSS]]",
            ">> [[module CSS]]",
            "> > [[module CSS]]",
            " \t>> text [[module CSS]]",
        ] {
            let offset = source.find("[[module").unwrap();
            let index = WikidotNativeQuoteIndex::new(source);
            assert!(index.contains(offset), "{source:?}");
        }
        for source in [
            ">[[module CSS]]",
            "text [[module CSS]]",
            " \t[[module CSS]]",
        ] {
            let offset = source.find("[[module").unwrap();
            let index = WikidotNativeQuoteIndex::new(source);
            assert!(!index.contains(offset), "{source:?}");
        }
    }

    #[test]
    fn leaves_html_opening_attributes_outside_the_literal_body() {
        let source = r#"<code data-example="marker">body</code> tail"#;
        let index = LiteralRegionIndex::new(source);

        assert!(!index.contains(source.find("marker").unwrap()));
        assert!(index.contains(source.find("body").unwrap()));
        assert!(!index.contains(source.find("tail").unwrap()));
    }

    #[test]
    fn ends_wikidot_blocks_at_the_closing_marker() {
        let source = "[[code]]\ninside\n[[/code]] [[#expr 1+1]]";
        let index = LiteralRegionIndex::new(source);

        assert!(index.contains(source.find("inside").unwrap()));
        assert!(!index.contains(source.find("[[#expr").unwrap()));
    }

    #[test]
    fn protection_index_includes_wikidot_and_html_tag_attributes() {
        let source = concat!(
            "outside ##red|yes##\n",
            "[[span data-value=\"##red|no]] yet##\"]]body[[/span]]\n",
            "<span title='quoted > ##red|no##'>body</span>",
        );
        let index = LiteralRegionIndex::new_wikidot_protection(source);
        assert!(!index.contains(source.find("##red|yes").unwrap()));
        for offset in source.match_indices("##red|no").map(|(offset, _)| offset) {
            assert!(index.contains(offset));
        }
    }

    #[test]
    fn html_restoration_index_includes_tags_comments_and_raw_text() {
        let source =
            "marker <a title='marker'>marker</a><!-- marker --><code>marker</code>";
        let index = LiteralRegionIndex::new_html_restoration(source);
        let offsets = source
            .match_indices("marker")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        assert!(!index.contains(offsets[0]));
        assert!(index.contains(offsets[1]));
        assert!(!index.contains(offsets[2]));
        assert!(index.contains(offsets[3]));
        assert!(index.contains(offsets[4]));
    }
}
