/*
 * services/render/literal_regions/list_pages_protection/css/ranges.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use super::super::super::block_candidates::HeadContext;
use super::super::super::{LiteralRegionIndex, merge_sorted_ranges};
use super::candidates::{
    collect_all_pinned_css_module_openers,
    collect_all_pinned_css_module_openers_with_heads,
};
use super::syntax::PinnedModuleCloseIndex;
use regex::Regex;
use std::ops::Range;
use std::sync::LazyLock;

static CSS_MODULE_OPEN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\[\[module\s+css[^\]]*\]\]").unwrap());
static MODULE_CLOSE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\[\[/module\]\]").unwrap());

pub(in crate::services::render::literal_regions::list_pages_protection) fn collect_downstream_css_module_ranges(
    source: &str,
) -> Vec<Range<usize>> {
    let Some(scan) = CssLiteralScan::new(source) else {
        return Vec::new();
    };
    let native_quote_lines = collect_native_quote_ranges(source);
    collect_css_module_ranges(source, &scan, &native_quote_lines)
}

pub(in crate::services::render::literal_regions::list_pages_protection) fn collect_downstream_css_module_ranges_with_heads(
    source: &str,
    heads: &HeadContext,
) -> Vec<Range<usize>> {
    let Some(scan) = CssLiteralScan::new_with_heads(source, heads) else {
        return Vec::new();
    };
    let native_quote_lines = collect_native_quote_ranges(source);
    collect_css_module_ranges(source, &scan, &native_quote_lines)
}

pub(in crate::services::render::literal_regions::list_pages_protection) fn collect_projected_css_module_ranges(
    source: &str,
    quote_ranges: &[Range<usize>],
) -> Vec<Range<usize>> {
    let Some(scan) = CssLiteralScan::new(source) else {
        return Vec::new();
    };
    collect_css_module_ranges(source, &scan, quote_ranges)
}

pub(in crate::services::render::literal_regions::list_pages_protection) fn collect_projected_css_module_ranges_with_heads(
    source: &str,
    quote_ranges: &[Range<usize>],
    heads: &HeadContext,
) -> Vec<Range<usize>> {
    let Some(scan) = CssLiteralScan::new_with_heads(source, heads) else {
        return Vec::new();
    };
    collect_css_module_ranges(source, &scan, quote_ranges)
}

struct CssLiteralIndices {
    open: LiteralRegionIndex,
    regex_close: LiteralRegionIndex,
}

impl CssLiteralIndices {
    fn new(source: &str) -> Self {
        Self {
            open: LiteralRegionIndex::new(source),
            regex_close: LiteralRegionIndex::new_wikidot_syntax(source),
        }
    }
}

struct CssLiteralScan {
    regex_openers: Vec<Range<usize>>,
    pinned_openers: Vec<Range<usize>>,
    pinned_close_ends: Vec<Option<usize>>,
    indices: CssLiteralIndices,
}

impl CssLiteralScan {
    fn new(source: &str) -> Option<Self> {
        let regex_openers = CSS_MODULE_OPEN_REGEX
            .find_iter(source)
            .map(|matched| matched.start()..matched.end())
            .collect::<Vec<_>>();
        let pinned_openers = collect_all_pinned_css_module_openers(source);
        Self::from_openers(source, regex_openers, pinned_openers)
    }

    fn new_with_heads(source: &str, heads: &HeadContext) -> Option<Self> {
        let regex_openers = CSS_MODULE_OPEN_REGEX
            .find_iter(source)
            .map(|matched| matched.start()..matched.end())
            .collect::<Vec<_>>();
        let pinned_openers =
            collect_all_pinned_css_module_openers_with_heads(source, heads);
        Self::from_openers(source, regex_openers, pinned_openers)
    }

    fn from_openers(
        source: &str,
        regex_openers: Vec<Range<usize>>,
        pinned_openers: Vec<Range<usize>>,
    ) -> Option<Self> {
        let pinned_close_ends = if pinned_openers.is_empty() {
            Vec::new()
        } else {
            PinnedModuleCloseIndex::new(source).first_ends_for_openers(&pinned_openers)
        };
        (!regex_openers.is_empty() || !pinned_openers.is_empty()).then(|| Self {
            regex_openers,
            pinned_openers,
            pinned_close_ends,
            indices: CssLiteralIndices::new(source),
        })
    }
}

// The regex path preserves the downstream extractor contract, while the pinned path adds parser-valid CSS ownership. Each path scans independently before their ranges are coalesced so an overlapping range from one contract cannot hide a longer range from the other.
fn collect_css_module_ranges(
    source: &str,
    scan: &CssLiteralScan,
    quote_ranges: &[Range<usize>],
) -> Vec<Range<usize>> {
    let regex_ranges = collect_regex_css_module_ranges(
        source,
        &scan.regex_openers,
        &scan.indices,
        quote_ranges,
    );
    let pinned_ranges = collect_pinned_css_module_ranges(
        &scan.pinned_openers,
        &scan.pinned_close_ends,
        &scan.indices,
        quote_ranges,
    );
    merge_sorted_ranges(regex_ranges, pinned_ranges)
}

fn collect_regex_css_module_ranges(
    source: &str,
    openers: &[Range<usize>],
    indices: &CssLiteralIndices,
    quote_ranges: &[Range<usize>],
) -> Vec<Range<usize>> {
    let mut cursor = 0usize;
    let mut open_literals = indices.open.monotone_cursor();
    let mut close_literals = indices.regex_close.monotone_cursor();
    let mut quote_cursor = 0usize;
    let mut ranges = Vec::new();

    for open in openers {
        if open.start < cursor {
            continue;
        }
        if open_literals.containing_end(open.start).is_some()
            || sorted_ranges_contains(quote_ranges, &mut quote_cursor, open.start)
        {
            cursor = open.end;
            continue;
        }
        let Some(close) = find_regex_module_close(source, open.end, &mut close_literals)
        else {
            break;
        };
        ranges.push(open.start..close.end);
        cursor = close.end;
    }
    ranges
}

fn collect_pinned_css_module_ranges(
    openers: &[Range<usize>],
    close_ends: &[Option<usize>],
    indices: &CssLiteralIndices,
    quote_ranges: &[Range<usize>],
) -> Vec<Range<usize>> {
    // FTML scans a raw CSS body as context-free tokens, so comments, raw spans, math, and generic tag heads do not mask a token-valid module end block.
    let mut cursor = 0usize;
    let mut open_literals = indices.open.monotone_cursor();
    let mut quote_cursor = 0usize;
    let mut ranges = Vec::new();

    for (open, close_end) in openers.iter().zip(close_ends) {
        if open.start < cursor {
            continue;
        }
        if open_literals.containing_end(open.start).is_some()
            || sorted_ranges_contains(quote_ranges, &mut quote_cursor, open.start)
        {
            continue;
        }
        let Some(close_end) = close_end else {
            continue;
        };
        ranges.push(open.start..*close_end);
        cursor = *close_end;
    }
    ranges
}

fn find_regex_module_close(
    source: &str,
    mut cursor: usize,
    close_literals: &mut super::super::super::LiteralRegionCursor<'_>,
) -> Option<Range<usize>> {
    loop {
        let candidate = MODULE_CLOSE_REGEX.find_at(source, cursor)?;
        if close_literals.containing_end(candidate.start()).is_none() {
            return Some(candidate.start()..candidate.end());
        }
        cursor = candidate.end();
    }
}

fn collect_native_quote_ranges(source: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut line_start = 0usize;
    for line in source.split_inclusive('\n') {
        let bytes = line.as_bytes();
        let mut cursor = 0usize;
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
    ranges
}

fn sorted_ranges_contains(
    ranges: &[Range<usize>],
    cursor: &mut usize,
    offset: usize,
) -> bool {
    while ranges.get(*cursor).is_some_and(|range| range.end <= offset) {
        *cursor += 1;
    }
    ranges
        .get(*cursor)
        .is_some_and(|range| range.start <= offset && offset < range.end)
}

#[cfg(test)]
mod tests;
