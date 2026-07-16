/*
 * services/render/literal_regions/block_candidates/quote_lines.rs
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

use super::super::wikidot::{PhysicalLines, physical_line_body, quote_depth_and_body};
use super::MAX_NATIVE_QUOTE_DEPTH;

#[derive(Clone, Copy)]
pub(super) struct PhysicalLine {
    pub(super) start: usize,
    pub(super) body_end: usize,
    pub(super) quote_depth: usize,
    pub(super) native_quote_depth: usize,
    pub(super) tight_quote_prefix: bool,
    pub(super) shallower_start: usize,
    pub(super) next_same_depth_content: usize,
    pub(super) next_deeper_start: usize,
}

pub(super) fn collect_physical_lines(source: &str) -> Vec<PhysicalLine> {
    let mut lines = Vec::new();
    let mut start = 0usize;
    for line in PhysicalLines::new(source) {
        let body = physical_line_body(line);
        let (native_quote_depth, quote_depth, logical, tight_quote_prefix) =
            physical_quote_context(body);
        let logical_start = start + body.len() - logical.len();
        let logical_nonspace = logical
            .char_indices()
            .find(|(_, character)| !character.is_whitespace())
            .map_or(source.len(), |(relative, _)| logical_start + relative);
        lines.push(PhysicalLine {
            start,
            body_end: start + body.len(),
            quote_depth,
            native_quote_depth,
            tight_quote_prefix,
            shallower_start: source.len(),
            next_same_depth_content: logical_nonspace,
            next_deeper_start: source.len(),
        });
        start += line.len();
    }

    let mut next_content_at_depth = [source.len(); MAX_NATIVE_QUOTE_DEPTH + 1];
    let mut next_deeper_at_depth = [source.len(); MAX_NATIVE_QUOTE_DEPTH + 1];
    let mut next_shallower_at_depth = [source.len(); MAX_NATIVE_QUOTE_DEPTH + 1];
    for index in (0..lines.len()).rev() {
        let native_depth = lines[index].native_quote_depth;
        let absolute_depth = lines[index].quote_depth;
        let content = lines[index].next_same_depth_content;
        if native_depth <= MAX_NATIVE_QUOTE_DEPTH {
            lines[index].shallower_start = next_shallower_at_depth[native_depth];
            lines[index].next_same_depth_content = next_content_at_depth[native_depth];
            lines[index].next_deeper_start = next_deeper_at_depth[native_depth];
        }
        if absolute_depth <= MAX_NATIVE_QUOTE_DEPTH && content < source.len() {
            next_content_at_depth[absolute_depth] = content;
        }
        for next_deeper in next_deeper_at_depth
            .iter_mut()
            .take(absolute_depth.min(MAX_NATIVE_QUOTE_DEPTH + 1))
        {
            *next_deeper = lines[index].start;
        }
        for next_shallower in next_shallower_at_depth
            .iter_mut()
            .skip((absolute_depth + 1).min(MAX_NATIVE_QUOTE_DEPTH + 1))
        {
            *next_shallower = lines[index].start;
        }
    }
    lines
}

fn physical_quote_context(body: &str) -> (usize, usize, &str, bool) {
    let trimmed = body.trim_start_matches([' ', '\t']);
    let native_depth = trimmed.bytes().take_while(|byte| *byte == b'>').count();
    let tight_quote_prefix = native_depth > 0
        && !matches!(trimmed.as_bytes().get(native_depth), Some(b' ' | b'\t'));
    let (absolute_depth, logical) = quote_depth_and_body(body);
    (native_depth, absolute_depth, logical, tight_quote_prefix)
}
