/*
 * services/render/literal_regions/list_pages_protection/css/candidates.rs
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
use super::super::super::left_block_start_in_run;
use super::syntax::{PinnedModuleCloseIndex, pinned_css_module_scan_start};
use std::ops::Range;

pub(in crate::services::render::literal_regions::list_pages_protection) fn collect_pinned_css_module_candidates(
    source: &str,
) -> Vec<Range<usize>> {
    if source.len() >= u32::MAX as usize {
        return Vec::new();
    }
    let heads = HeadContext::new(source);
    collect_pinned_css_module_candidates_with_heads(source, &heads)
}

pub(in crate::services::render::literal_regions::list_pages_protection) fn collect_pinned_css_module_candidates_with_heads(
    source: &str,
    heads: &HeadContext,
) -> Vec<Range<usize>> {
    let openers = collect_all_pinned_css_module_openers_with_heads(source, heads);
    if openers.is_empty() {
        return Vec::new();
    }
    let close_ends = PinnedModuleCloseIndex::new(source).first_ends_for_openers(&openers);
    openers
        .into_iter()
        .zip(close_ends)
        .filter_map(|(open, close_end)| close_end.map(|close_end| open.start..close_end))
        .collect()
}

pub(super) fn collect_all_pinned_css_module_openers(source: &str) -> Vec<Range<usize>> {
    collect_all_pinned_css_module_openers_with_work(source).0
}

pub(super) fn collect_all_pinned_css_module_openers_with_heads(
    source: &str,
    heads: &HeadContext,
) -> Vec<Range<usize>> {
    collect_css_prefixes(source)
        .0
        .into_iter()
        .filter_map(|(start, scan_start)| heads.map_end(scan_start).map(|end| start..end))
        .collect()
}

fn collect_all_pinned_css_module_openers_with_work(
    source: &str,
) -> (Vec<Range<usize>>, usize) {
    if source.len() >= u32::MAX as usize {
        return (Vec::new(), source.len());
    }
    let (prefixes, mut work) = collect_css_prefixes(source);
    if prefixes.is_empty() {
        return (Vec::new(), work);
    }

    let heads = HeadContext::new(source);
    let mut openers = Vec::with_capacity(prefixes.len());
    for (start, scan_start) in prefixes {
        if let Some(end) = heads.map_end(scan_start) {
            openers.push(start..end);
        }
        work = work.saturating_add(1);
    }
    (openers, work.saturating_add(source.len()))
}

fn collect_css_prefixes(source: &str) -> (Vec<(usize, usize)>, usize) {
    let bytes = source.as_bytes();
    let mut prefixes = Vec::new();
    let mut cursor = 0usize;
    let mut work = 0usize;
    while let Some(relative) = source[cursor..].find("[[") {
        let candidate = cursor + relative;
        let (block_start, run_end) = left_block_start_in_run(bytes, candidate);
        work = work.saturating_add(relative + 1);
        let Some(start) = block_start else {
            cursor = run_end;
            continue;
        };
        if let Some(scan_start) = pinned_css_module_scan_start(bytes, start) {
            prefixes.push((start, scan_start));
        }
        cursor = start + 2;
    }
    (prefixes, work)
}

#[cfg(test)]
mod tests;
