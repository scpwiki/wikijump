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

mod anchor_candidates;
mod base_candidates;
mod block_candidates;
mod common;
mod count_pages;
#[allow(dead_code)]
mod downstream_protectors;
pub(in crate::services::render) mod list_pages_protection;
mod parser_candidates;
mod text_owners;
mod token_boundaries;
mod wikidot;

use self::common::{collect_wikidot_block_ranges, collect_wikidot_tag_ranges};
use self::count_pages::collect_count_pages_literal_ranges;
#[allow(unused_imports)]
pub(in crate::services::render) use self::downstream_protectors::{
    DownstreamProtectorFamily, DownstreamProtectorRange,
    collect_downstream_protector_ranges,
};
pub(in crate::services::render) use self::list_pages_protection::ListPagesSourceProjection;
#[cfg(test)]
use self::list_pages_protection::collect_list_pages_runtime_recovery_ranges;
pub(super) use self::list_pages_protection::project_list_pages_typography_in_place;
use self::list_pages_protection::{
    collect_already_projected_list_pages_literal_ranges,
    collect_list_pages_downstream_css_ranges, collect_list_pages_literal_ranges,
};
pub(super) use self::token_boundaries::{
    TextTokenCursor, WikidotArgumentValueKind, WikidotTagArgumentScan, WikidotTagScan,
    WikidotWholeHeadScan, left_block_start_in_run, right_bracket_token,
    rollback_start_in_left_run, scan_wikidot_tag, scan_wikidot_whole_head_value,
    wikidot_right_bracket_token, wikidot_trimmed_name,
};
use self::wikidot::collect_wikidot_conditional_literal_ranges;
pub(super) use self::wikidot::{double_quote_ends_wikidot_argument, quote_is_escaped};
use regex::Regex;
use std::ops::Range;
use std::sync::LazyLock;

static WIKIDOT_ANCHOR_MARKER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\[#\s+(?P<name>[^\]\n]+)\]\]")
        .expect("the Wikidot anchor marker regex is valid")
});

/// Precomputed literal regions for compatibility transforms.
///
/// Building the index is linear in the input length. Membership checks use a
/// binary search, avoiding a full prefix rescan for every parser function or
/// conditional on component-heavy pages.
#[derive(Debug, Default)]
pub(crate) struct LiteralRegionIndex {
    ranges: Vec<Range<usize>>,
}

pub(super) struct ListPagesScannerLiteralIndexes {
    pub(super) direct: LiteralRegionIndex,
    pub(super) projected: Option<LiteralRegionIndex>,
    pub(super) original_css: Option<LiteralRegionIndex>,
    pub(super) original_anchors: Option<LiteralRegionIndex>,
}

pub(super) struct LiteralRegionCursor<'a> {
    ranges: &'a [Range<usize>],
    index: usize,
    last_offset: Option<usize>,
    advances: usize,
}

impl LiteralRegionIndex {
    pub(super) fn new(source: &str) -> Self {
        Self::build(source, true)
    }

    pub(super) fn new_wikidot_syntax(source: &str) -> Self {
        Self::build(source, false)
    }

    /// Literal regions used while pairing Wikidot conditional boundaries.
    ///
    /// Balanced inline raw text is confined to its physical line for this
    /// pass, while an unmatched `@@` owns no literal region. This matches FTML
    /// compatibility scanning and prevents an unmatched opener from hiding a
    /// conditional boundary. Other literals retain fail-closed behavior.
    pub(super) fn new_wikidot_conditional_syntax(source: &str) -> Self {
        let mut ranges = Vec::new();
        collect_wikidot_conditional_literal_ranges(source, &mut ranges);
        Self {
            ranges: coalesce_sorted_ranges(ranges),
        }
    }

    /// Legacy literal ownership used while expanding CountPages modules.
    ///
    /// This preserves the former fail-closed line-prefix rules while replacing their per-capture prefix rescans with one linear index build.
    pub(super) fn new_count_pages_syntax(source: &str) -> Self {
        Self {
            ranges: collect_count_pages_literal_ranges(source),
        }
    }

    /// Test oracle for the complete runtime-protection index.
    ///
    /// The structural scanner deliberately uses `new_list_pages_scanner_syntax`
    /// so recovery barriers remain visible to its own rollback logic.
    #[cfg(test)]
    pub(super) fn new_list_pages_syntax(source: &str) -> Self {
        let mut index = Self {
            ranges: collect_list_pages_literal_ranges(source),
        };
        index.merge_sorted_ranges(collect_list_pages_runtime_recovery_ranges(source));
        index.merge_sorted_ranges(collect_wikidot_anchor_ranges(source));
        index
    }

    pub(super) fn new_list_pages_scanner_syntax(source: &str) -> Self {
        let mut index = Self {
            ranges: collect_list_pages_literal_ranges(source),
        };
        index.merge_sorted_ranges(collect_wikidot_anchor_ranges(source));
        index
    }

    pub(super) fn new_list_pages_scanner_indexes(
        source: &str,
        projection: Option<&ListPagesSourceProjection>,
    ) -> ListPagesScannerLiteralIndexes {
        let Some(projection) = projection else {
            return ListPagesScannerLiteralIndexes {
                direct: Self::new_list_pages_scanner_syntax(source),
                projected: None,
                original_css: None,
                original_anchors: None,
            };
        };

        // The direct and projected structural scanners use the same projected candidate graph. Build it once, then map a clone back to original offsets for the direct scanner instead of rebuilding the graph and source projection independently.
        let projected_ranges =
            collect_already_projected_list_pages_literal_ranges(projection.source());
        let original_css_ranges = collect_list_pages_downstream_css_ranges(source);
        let original_anchor_ranges = collect_wikidot_anchor_ranges(source);
        let projected_anchor_ranges = collect_wikidot_anchor_ranges(projection.source());

        let mapped_ranges = projection.map_ranges(projected_ranges.clone(), source.len());
        let mut direct = Self {
            ranges: merge_sorted_ranges(original_css_ranges.clone(), mapped_ranges),
        };
        direct.merge_sorted_ranges(original_anchor_ranges.clone());

        let mut projected = Self {
            ranges: projected_ranges,
        };
        projected.merge_sorted_ranges(projected_anchor_ranges);

        ListPagesScannerLiteralIndexes {
            direct,
            projected: Some(projected),
            original_css: Some(Self {
                ranges: original_css_ranges,
            }),
            original_anchors: Some(Self {
                ranges: original_anchor_ranges,
            }),
        }
    }

    #[cfg(test)]
    pub(super) fn new_already_projected_list_pages_syntax(source: &str) -> Self {
        let mut index = Self {
            ranges: collect_already_projected_list_pages_literal_ranges(source),
        };
        index.merge_sorted_ranges(collect_wikidot_anchor_ranges(source));
        index
    }

    #[cfg(test)]
    pub(super) fn new_list_pages_anchor_syntax(source: &str) -> Self {
        Self {
            ranges: collect_wikidot_anchor_ranges(source),
        }
    }

    #[cfg(test)]
    pub(super) fn new_list_pages_downstream_css_syntax(source: &str) -> Self {
        Self {
            ranges: collect_list_pages_downstream_css_ranges(source),
        }
    }

    /// Literal and tag regions where a pre-FTML compatibility protector must
    /// not recognize authored syntax.
    pub(super) fn new_wikidot_protection(source: &str) -> Self {
        let mut index = Self::build(source, false);
        let mut ranges = Vec::new();
        collect_wikidot_tag_ranges(source, &mut ranges);
        index.merge_sorted_ranges(ranges);
        let mut ranges = Vec::new();
        collect_html_tag_ranges(source, &mut ranges);
        index.merge_sorted_ranges(ranges);
        index
    }

    /// Literal and tag interiors where a compatibility module recognizer must
    /// not treat tag-shaped text as a new module. The first byte of every tag
    /// remains outside the index so a candidate can recognize the tag that
    /// starts exactly there. Shrinking before merging also preserves the gap
    /// between adjacent tags.
    pub(crate) fn new_wikidot_module_recognition(source: &str) -> Self {
        let mut index = Self::build(source, true);
        let mut ranges = Vec::new();
        collect_paired_ranges(source, "<!--", "-->", &mut ranges);
        index.merge_sorted_ranges(ranges);
        let mut ranges = Vec::new();
        collect_wikidot_tag_ranges(source, &mut ranges);
        collect_html_tag_ranges(source, &mut ranges);
        ranges.sort_unstable_by_key(|range| (range.start, range.end));
        index.merge_sorted_ranges(
            ranges
                .into_iter()
                .filter_map(|range| {
                    (range.start + 1 < range.end).then_some(range.start + 1..range.end)
                })
                .collect(),
        );
        index
    }

    /// Rendered HTML regions where trusted-fragment markers must remain text.
    #[cfg(test)]
    pub(super) fn new_html_restoration(source: &str) -> Self {
        let mut index = Self::build(source, true);
        let mut ranges = Vec::new();
        collect_html_tag_ranges(source, &mut ranges);
        index.merge_sorted_ranges(ranges);
        let mut ranges = Vec::new();
        collect_paired_ranges(source, "<!--", "-->", &mut ranges);
        index.merge_sorted_ranges(ranges);
        index
    }

    /// Rendered HTML regions where color-fragment markers must remain text.
    ///
    /// Inline Wikidot monospace permits color syntax, so standalone `<code>`
    /// contents are not literal for this restoration pass. Block code remains
    /// protected by its enclosing `<pre>` or `<div class="code">` range.
    pub(super) fn new_html_color_restoration(source: &str) -> Self {
        let mut index = Self::build(source, false);
        let mut ranges = Vec::new();
        collect_html_literal_ranges(source, &mut ranges, false);
        index.merge_sorted_ranges(ranges);
        let mut ranges = Vec::new();
        collect_html_tag_ranges(source, &mut ranges);
        index.merge_sorted_ranges(ranges);
        let mut ranges = Vec::new();
        collect_paired_ranges(source, "<!--", "-->", &mut ranges);
        index.merge_sorted_ranges(ranges);
        index
    }

    fn build(source: &str, include_rendered_html: bool) -> Self {
        let mut block_ranges = Vec::new();
        collect_wikidot_block_ranges(source, &mut block_ranges);
        let mut raw_ranges = Vec::new();
        collect_paired_ranges(source, "@@", "@@", &mut raw_ranges);
        let mut comment_ranges = Vec::new();
        collect_paired_ranges(source, "[!--", "--]", &mut comment_ranges);
        let mut ranges =
            select_owned_ranges([&block_ranges, &raw_ranges, &comment_ranges]);
        if include_rendered_html {
            let mut html_ranges = Vec::new();
            collect_html_literal_ranges(source, &mut html_ranges, true);
            ranges = merge_sorted_ranges(ranges, html_ranges);
        }

        Self {
            ranges: coalesce_sorted_ranges(ranges),
        }
    }

    fn merge_sorted_ranges(&mut self, ranges: Vec<Range<usize>>) {
        self.ranges = merge_sorted_ranges(std::mem::take(&mut self.ranges), ranges);
    }

    pub(crate) fn contains(&self, offset: usize) -> bool {
        let insertion = self.ranges.partition_point(|range| range.start <= offset);
        insertion > 0 && offset < self.ranges[insertion - 1].end
    }

    pub(super) fn monotone_cursor(&self) -> LiteralRegionCursor<'_> {
        LiteralRegionCursor {
            ranges: &self.ranges,
            index: 0,
            last_offset: None,
            advances: 0,
        }
    }
}

fn collect_wikidot_anchor_ranges(source: &str) -> Vec<Range<usize>> {
    WIKIDOT_ANCHOR_MARKER_REGEX
        .find_iter(source)
        .map(|matched| matched.range())
        .collect()
}

fn select_owned_ranges<const N: usize>(
    streams: [&[Range<usize>]; N],
) -> Vec<Range<usize>> {
    let mut indices = [0usize; N];
    let capacity = streams.iter().map(|stream| stream.len()).sum();
    let mut selected = Vec::with_capacity(capacity);

    loop {
        let next = streams
            .iter()
            .enumerate()
            .filter_map(|(stream, ranges)| {
                ranges.get(indices[stream]).map(|range| (stream, range))
            })
            .min_by_key(|(stream, range)| (range.start, range.end, *stream));
        let Some((stream, range)) = next else {
            break;
        };
        indices[stream] += 1;

        if selected
            .last()
            .is_none_or(|previous: &Range<usize>| previous.end <= range.start)
        {
            selected.push(range.clone());
        }
    }

    selected
}

impl LiteralRegionCursor<'_> {
    #[cfg(test)]
    pub(super) fn contains(&mut self, offset: usize) -> bool {
        self.containing_end(offset).is_some()
    }

    pub(super) fn containing_end(&mut self, offset: usize) -> Option<usize> {
        debug_assert!(
            self.last_offset.is_none_or(|previous| previous <= offset),
            "literal-region cursor offsets must be monotone",
        );
        self.last_offset = Some(offset);
        while self
            .ranges
            .get(self.index)
            .is_some_and(|range| range.end <= offset)
        {
            self.index += 1;
            self.advances += 1;
        }
        self.ranges
            .get(self.index)
            .filter(|range| range.start <= offset && offset < range.end)
            .map(|range| range.end)
    }

    pub(super) fn advances(&self) -> usize {
        self.advances
    }
}

fn merge_sorted_ranges(
    left: Vec<Range<usize>>,
    right: Vec<Range<usize>>,
) -> Vec<Range<usize>> {
    let mut left = left.into_iter().peekable();
    let mut right = right.into_iter().peekable();
    let mut merged = Vec::with_capacity(left.len() + right.len());

    while left.peek().is_some() || right.peek().is_some() {
        let take_left = match (left.peek(), right.peek()) {
            (Some(left), Some(right)) => {
                (left.start, left.end) <= (right.start, right.end)
            }
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (None, None) => break,
        };
        let range = if take_left {
            left.next().expect("left range exists")
        } else {
            right.next().expect("right range exists")
        };
        push_coalesced_range(&mut merged, range);
    }
    merged
}

fn coalesce_sorted_ranges(ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    let mut merged = Vec::with_capacity(ranges.len());
    for range in ranges {
        push_coalesced_range(&mut merged, range);
    }
    merged
}

fn push_coalesced_range(ranges: &mut Vec<Range<usize>>, range: Range<usize>) {
    if range.start >= range.end {
        return;
    }
    if let Some(previous) = ranges.last_mut()
        && range.start <= previous.end
    {
        previous.end = previous.end.max(range.end);
    } else {
        ranges.push(range);
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
mod tests;
