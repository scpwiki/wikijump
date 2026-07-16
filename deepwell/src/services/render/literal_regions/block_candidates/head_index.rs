/*
 * services/render/literal_regions/block_candidates/head_index.rs
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

use super::super::token_boundaries::{
    TextTokenIndex, comment_close_is_token, right_bracket_token,
};
use super::{NO_OFFSET, compact_offset, expanded_offset};

const GENERIC_RIGHT_BLOCK: u8 = 1 << 0;
const WIKIDOT_RIGHT_BLOCK: u8 = 1 << 1;
const QUOTE_TOKEN: u8 = 1 << 0;
const ARGUMENT_CLOSE_QUOTE: u8 = 1 << 1;

pub(in crate::services::render::literal_regions) struct HeadContext {
    pub(super) next_generic_right_block: Vec<u32>,
    pub(super) next_wikidot_right_block: Vec<u32>,
    pub(super) map_head_end: Vec<u32>,
    pub(super) whitespace_end: Vec<u32>,
}

impl HeadContext {
    pub(in crate::services::render::literal_regions) fn new(source: &str) -> Self {
        let text_tokens = TextTokenIndex::new(source);
        Self::new_with_text_tokens(source, &text_tokens)
    }

    pub(in crate::services::render::literal_regions) fn new_with_text_tokens(
        source: &str,
        text_tokens: &TextTokenIndex,
    ) -> Self {
        let bytes = source.as_bytes();
        let map_spacing_end = ascii_spacing_run_ends(bytes, true);
        let inline_space_end = ascii_spacing_run_ends(bytes, false);
        let mut text_owned = vec![false; bytes.len()];
        let mut key_end = vec![NO_OFFSET; bytes.len() + 1];
        let mut text_tokens = text_tokens.cursor();
        for cursor in 0..bytes.len() {
            if let Some(end) = text_tokens.range_end_at(cursor) {
                text_owned[cursor..end].fill(true);
                key_end[cursor] = compact_offset(end);
            }
        }
        drop(text_tokens);
        index_key_components(bytes, &text_owned, &mut key_end);
        let quote_key_end = ascii_quote_key_run_ends(bytes, &text_owned);

        let mut right_flags = vec![0u8; bytes.len()];
        for cursor in 0..bytes.len() {
            if bytes[cursor] != b']' {
                continue;
            }
            let generic = right_bracket_token(bytes, cursor, bytes.len()).0;
            if generic {
                right_flags[cursor] |= GENERIC_RIGHT_BLOCK;
                if !wikidot_right_block_is_suppressed(bytes, cursor, &text_owned) {
                    right_flags[cursor] |= WIKIDOT_RIGHT_BLOCK;
                }
            }
        }

        let mut quote_flags = vec![0u8; bytes.len()];
        let mut unowned_backslash_run = 0usize;
        for cursor in 0..bytes.len() {
            if bytes[cursor] == b'\\' && !text_owned[cursor] {
                unowned_backslash_run += 1;
                continue;
            }
            if bytes[cursor] == b'"'
                && !text_owned[cursor]
                && unowned_backslash_run.is_multiple_of(2)
            {
                quote_flags[cursor] |= QUOTE_TOKEN;
                if quote_ends_argument(
                    bytes,
                    cursor,
                    &right_flags,
                    &inline_space_end,
                    &quote_key_end,
                    &text_owned,
                ) {
                    quote_flags[cursor] |= ARGUMENT_CLOSE_QUOTE;
                }
            }
            unowned_backslash_run = 0;
        }
        let next_argument_quote =
            next_marked_on_physical_line(bytes, &quote_flags, ARGUMENT_CLOSE_QUOTE);
        let map_head_end = map_head_ends(
            bytes,
            &map_spacing_end,
            &inline_space_end,
            &key_end,
            &right_flags,
            &quote_flags,
            &next_argument_quote,
            &text_owned,
        );
        drop(map_spacing_end);
        drop(inline_space_end);
        drop(key_end);
        drop(quote_key_end);
        drop(quote_flags);
        drop(next_argument_quote);
        drop(text_owned);

        let whitespace_end = unicode_whitespace_run_ends(source);
        let next_generic_right_block =
            next_marked_offset(&right_flags, GENERIC_RIGHT_BLOCK);
        let next_wikidot_right_block =
            next_marked_offset(&right_flags, WIKIDOT_RIGHT_BLOCK);
        drop(right_flags);
        Self {
            next_generic_right_block,
            next_wikidot_right_block,
            map_head_end,
            whitespace_end,
        }
    }

    pub(in crate::services::render::literal_regions) fn map_end(
        &self,
        start: usize,
    ) -> Option<usize> {
        self.map_head_end
            .get(start)
            .copied()
            .filter(|end| *end != NO_OFFSET)
            .map(expanded_offset)
    }

    #[cfg(test)]
    pub(super) fn retained_bytes(&self) -> usize {
        [
            &self.next_generic_right_block,
            &self.next_wikidot_right_block,
            &self.map_head_end,
            &self.whitespace_end,
        ]
        .into_iter()
        .map(|offsets| offsets.capacity() * std::mem::size_of::<u32>())
        .sum()
    }
}

fn wikidot_right_block_is_suppressed(
    bytes: &[u8],
    start: usize,
    text_owned: &[bool],
) -> bool {
    (start > 0 && bytes[start - 1] == b'$' && !text_owned[start - 1])
        || (start >= 2
            && bytes.get(start - 2..start) == Some(&b"--"[..])
            && comment_close_is_token(bytes, start - 2)
            && !text_owned[start - 2])
}

fn unicode_whitespace_run_ends(source: &str) -> Vec<u32> {
    let mut ends = (0..=source.len()).map(compact_offset).collect::<Vec<_>>();
    let mut next_nonspace = compact_offset(source.len());
    for (start, character) in source.char_indices().rev() {
        if character.is_whitespace() {
            ends[start] = next_nonspace;
        } else {
            next_nonspace = compact_offset(start);
        }
    }
    ends
}

fn ascii_spacing_run_ends(bytes: &[u8], include_line_breaks: bool) -> Vec<u32> {
    let mut ends = (0..=bytes.len()).map(compact_offset).collect::<Vec<_>>();
    let mut next_nonspace = compact_offset(bytes.len());
    for index in (0..bytes.len()).rev() {
        if matches!(bytes[index], b' ' | b'\t')
            || (include_line_breaks && matches!(bytes[index], b'\n' | b'\r'))
        {
            ends[index] = next_nonspace;
        } else {
            next_nonspace = compact_offset(index);
        }
    }
    ends
}

fn index_key_components(bytes: &[u8], text_owned: &[bool], key_end: &mut [u32]) {
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if text_owned[cursor] {
            cursor += 1;
            continue;
        }
        if let Some(end) = variable_token_end(bytes, cursor) {
            key_end[cursor] = compact_offset(end);
            cursor = end;
            continue;
        }
        if bytes[cursor].is_ascii_alphanumeric() || matches!(bytes[cursor], b'_' | b'-') {
            key_end[cursor] = compact_offset(cursor + 1);
        }
        cursor += 1;
    }

    for cursor in (0..bytes.len()).rev() {
        let component_end = key_end[cursor];
        if component_end == NO_OFFSET {
            continue;
        }
        let suffix_end = key_end[expanded_offset(component_end)];
        if suffix_end != NO_OFFSET {
            key_end[cursor] = suffix_end;
        }
    }
}

fn ascii_quote_key_run_ends(bytes: &[u8], text_owned: &[bool]) -> Vec<u32> {
    let mut ends = vec![NO_OFFSET; bytes.len() + 1];
    let mut next_non_key = compact_offset(bytes.len());
    for index in (0..bytes.len()).rev() {
        if !text_owned[index]
            && (bytes[index].is_ascii_alphanumeric()
                || matches!(bytes[index], b'_' | b'-'))
        {
            ends[index] = next_non_key;
        } else {
            next_non_key = compact_offset(index);
        }
    }
    ends
}

fn variable_token_end(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start..start + 2) != Some(&b"{$"[..]) {
        return None;
    }
    let mut cursor = start + 2;
    let identifier_start = cursor;
    while bytes.get(cursor).is_some_and(u8::is_ascii_alphanumeric) {
        cursor += 1;
    }
    (cursor > identifier_start && bytes.get(cursor) == Some(&b'}')).then_some(cursor + 1)
}

fn quote_ends_argument(
    bytes: &[u8],
    quote: usize,
    right_flags: &[u8],
    inline_space_end: &[u32],
    quote_key_end: &[u32],
    text_owned: &[bool],
) -> bool {
    let next = quote + 1;
    if next == bytes.len()
        || has_flag(right_flags, next, WIKIDOT_RIGHT_BLOCK)
        || matches!(bytes.get(next), Some(b'\n' | b'\r'))
    {
        return true;
    }
    let cursor = expanded_offset(inline_space_end[next]);
    if cursor == next {
        return false;
    }
    if has_flag(right_flags, cursor, WIKIDOT_RIGHT_BLOCK) {
        return true;
    }
    let key_finish = quote_key_end[cursor];
    if key_finish == NO_OFFSET {
        return false;
    }
    let key_finish = expanded_offset(key_finish);
    if has_flag(right_flags, key_finish, WIKIDOT_RIGHT_BLOCK) {
        return true;
    }
    let equals = expanded_offset(inline_space_end[key_finish]);
    bytes.get(equals) == Some(&b'=') && !text_owned[equals]
}

#[allow(clippy::too_many_arguments)]
fn map_head_ends(
    bytes: &[u8],
    whitespace_end: &[u32],
    inline_space_end: &[u32],
    key_end: &[u32],
    right_flags: &[u8],
    quote_flags: &[u8],
    next_argument_quote: &[u32],
    text_owned: &[bool],
) -> Vec<u32> {
    let mut ends = vec![NO_OFFSET; bytes.len() + 1];
    for start in (0..bytes.len()).rev() {
        let cursor = expanded_offset(whitespace_end[start]);
        if has_flag(right_flags, cursor, WIKIDOT_RIGHT_BLOCK) {
            ends[start] = compact_offset(cursor + 2);
            continue;
        }
        let key_finish = key_end[cursor];
        if key_finish == NO_OFFSET {
            continue;
        }
        let key_finish = expanded_offset(key_finish);
        if has_flag(right_flags, key_finish, WIKIDOT_RIGHT_BLOCK) {
            ends[start] = compact_offset(key_finish + 2);
            continue;
        }
        let equals = expanded_offset(inline_space_end[key_finish]);
        if bytes.get(equals) != Some(&b'=') || text_owned[equals] {
            continue;
        }
        let quote = expanded_offset(inline_space_end[equals + 1]);
        if !has_flag(quote_flags, quote, QUOTE_TOKEN) {
            continue;
        }
        let close = next_argument_quote[quote + 1];
        if close != NO_OFFSET {
            ends[start] = ends[expanded_offset(close) + 1];
        }
    }
    ends
}

fn next_marked_offset(marked: &[u8], flag: u8) -> Vec<u32> {
    let mut next = vec![NO_OFFSET; marked.len() + 1];
    let mut current = NO_OFFSET;
    for index in (0..marked.len()).rev() {
        if marked[index] & flag != 0 {
            current = compact_offset(index);
        }
        next[index] = current;
    }
    next
}

fn next_marked_on_physical_line(bytes: &[u8], marked: &[u8], flag: u8) -> Vec<u32> {
    let mut next = vec![NO_OFFSET; bytes.len() + 1];
    let mut current = NO_OFFSET;
    for index in (0..bytes.len()).rev() {
        if matches!(bytes[index], b'\n' | b'\r') {
            current = NO_OFFSET;
        } else if marked[index] & flag != 0 {
            current = compact_offset(index);
        }
        next[index] = current;
    }
    next
}

fn has_flag(flags: &[u8], offset: usize, flag: u8) -> bool {
    flags.get(offset).is_some_and(|value| value & flag != 0)
}
