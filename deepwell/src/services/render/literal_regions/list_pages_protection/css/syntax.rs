/*
 * services/render/literal_regions/list_pages_protection/css/syntax.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use super::super::super::{
    left_block_start_in_run, right_bracket_token, wikidot_trimmed_name,
};
use std::ops::Range;

pub(super) struct PinnedModuleCloseIndex {
    ranges: Vec<Range<usize>>,
}

impl PinnedModuleCloseIndex {
    pub(super) fn new(source: &str) -> Self {
        let bytes = source.as_bytes();
        let mut ranges = Vec::new();
        let mut cursor = 0usize;

        while let Some(relative_start) = source[cursor..].find("[[/") {
            let start = cursor + relative_start;
            cursor = start + 3;
            if left_block_start_in_run(bytes, start).0 != Some(start) {
                continue;
            }
            if let Some(end) = pinned_module_close_end(bytes, start) {
                ranges.push(start..end);
                cursor = end;
            }
        }
        Self { ranges }
    }

    pub(super) fn first_ends_for_openers(
        &self,
        openers: &[Range<usize>],
    ) -> Vec<Option<usize>> {
        if openers.is_empty() {
            return Vec::new();
        }
        let mut from = (0..openers.len())
            .map(|index| u32::try_from(index).expect("CSS opener count fits u32"))
            .collect::<Vec<_>>();
        let mut to = vec![0u32; openers.len()];
        for shift in [0, 16] {
            let mut counts = vec![0u32; 1 << 16];
            for &index in &from {
                let end = u32::try_from(openers[index as usize].end)
                    .expect("pinned CSS source length fits u32");
                counts[((end >> shift) & 0xffff) as usize] += 1;
            }
            let mut next = 0u32;
            for count in &mut counts {
                let current = *count;
                *count = next;
                next += current;
            }
            for &index in &from {
                let end = u32::try_from(openers[index as usize].end)
                    .expect("pinned CSS source length fits u32");
                let bucket = ((end >> shift) & 0xffff) as usize;
                to[counts[bucket] as usize] = index;
                counts[bucket] += 1;
            }
            std::mem::swap(&mut from, &mut to);
        }

        let mut output = vec![None; openers.len()];
        let mut close = 0usize;
        for index in from {
            let index = index as usize;
            while self
                .ranges
                .get(close)
                .is_some_and(|range| range.start < openers[index].end)
            {
                close += 1;
            }
            output[index] = self.ranges.get(close).map(|range| range.end);
        }
        output
    }
}

pub(super) fn pinned_css_module_scan_start(bytes: &[u8], start: usize) -> Option<usize> {
    let mut cursor = start + 2;
    skip_horizontal_whitespace(bytes, &mut cursor);
    let (name, name_end) = wikidot_trimmed_name(bytes, cursor);
    let name = name?;
    if !name.eq_ignore_ascii_case(b"module") && !name.eq_ignore_ascii_case(b"module654") {
        return None;
    }

    cursor = name_end;
    if !skip_module_subname_delimiter(bytes, &mut cursor) {
        return None;
    }
    let (subname, scan_start) = wikidot_trimmed_name(bytes, cursor);
    subname
        .is_some_and(|subname| subname.eq_ignore_ascii_case(b"css"))
        .then_some(scan_start)
}

fn skip_module_subname_delimiter(bytes: &[u8], cursor: &mut usize) -> bool {
    if matches!(bytes.get(*cursor), Some(b' ' | b'\t')) {
        skip_horizontal_whitespace(bytes, cursor);
        return true;
    }
    if matches!(bytes.get(*cursor), Some(b'\n' | b'\r')) {
        skip_physical_line_endings(bytes, cursor);
        skip_horizontal_whitespace(bytes, cursor);
        return true;
    }
    false
}

fn skip_horizontal_whitespace(bytes: &[u8], cursor: &mut usize) {
    while matches!(bytes.get(*cursor), Some(b' ' | b'\t')) {
        *cursor += 1;
    }
}

fn skip_physical_line_endings(bytes: &[u8], cursor: &mut usize) {
    while matches!(bytes.get(*cursor), Some(b'\n' | b'\r')) {
        if bytes[*cursor] == b'\r' && bytes.get(*cursor + 1) == Some(&b'\n') {
            *cursor += 2;
        } else {
            *cursor += 1;
        }
    }
}

pub(super) fn pinned_module_close_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut cursor = start + 3;
    skip_horizontal_whitespace(bytes, &mut cursor);
    if bytes.get(cursor..cursor + 2) == Some(&b"[["[..])
        && left_block_start_in_run(bytes, cursor).0 == Some(cursor)
    {
        cursor += 2;
        skip_horizontal_whitespace(bytes, &mut cursor);
    }
    let (name, name_end) = wikidot_trimmed_name(bytes, cursor);
    let name = name?;
    let name = name.strip_suffix(b"_").unwrap_or(name);
    if !name.eq_ignore_ascii_case(b"module") && !name.eq_ignore_ascii_case(b"module654") {
        return None;
    }

    cursor = name_end;
    if matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
        skip_horizontal_whitespace(bytes, &mut cursor);
    } else if matches!(bytes.get(cursor), Some(b'\n' | b'\r')) {
        skip_physical_line_endings(bytes, &mut cursor);
        skip_horizontal_whitespace(bytes, &mut cursor);
    }
    if bytes.get(cursor) != Some(&b']')
        || !right_bracket_token(bytes, cursor, bytes.len()).0
    {
        return None;
    }
    Some(cursor + 2)
}

#[cfg(test)]
mod tests;
