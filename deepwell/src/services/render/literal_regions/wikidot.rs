/*
 * services/render/literal_regions/wikidot.rs
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

use super::token_boundaries::{
    TextTokenCursor, WikidotTagScan, comment_close_is_token, find_block_close,
    find_right_raw, find_token_unowned_delimiter, right_bracket_token, scan_wikidot_tag,
    wikidot_trimmed_name,
};
pub(in crate::services::render) use super::token_boundaries::{
    double_quote_ends_wikidot_argument, quote_is_escaped,
};

pub(super) struct PhysicalLines<'a> {
    source: &'a str,
    cursor: usize,
}

impl<'a> PhysicalLines<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        Self { source, cursor: 0 }
    }
}

impl<'a> Iterator for PhysicalLines<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.source.len() {
            return None;
        }
        let bytes = self.source.as_bytes();
        let start = self.cursor;
        while self.cursor < bytes.len() {
            match bytes[self.cursor] {
                b'\n' => {
                    self.cursor += 1;
                    return Some(&self.source[start..self.cursor]);
                }
                b'\r' => {
                    self.cursor += 1;
                    if bytes.get(self.cursor) == Some(&b'\n') {
                        self.cursor += 1;
                    }
                    return Some(&self.source[start..self.cursor]);
                }
                _ => self.cursor += 1,
            }
        }
        Some(&self.source[start..self.cursor])
    }
}

pub(super) fn physical_line_body(line: &str) -> &str {
    line.strip_suffix("\r\n")
        .or_else(|| line.strip_suffix('\n'))
        .or_else(|| line.strip_suffix('\r'))
        .unwrap_or(line)
}

pub(super) fn wikidot_multiline_tag_end(
    bytes: &[u8],
    start: usize,
    text_tokens: &mut TextTokenCursor,
) -> Option<usize> {
    let mut lookahead_tokens = text_tokens.clone();
    let scan = scan_wikidot_tag(
        bytes,
        start,
        bytes.len(),
        true,
        false,
        &mut lookahead_tokens,
    );
    match scan {
        WikidotTagScan::Complete(end) => {
            *text_tokens = lookahead_tokens;
            Some(end)
        }
        WikidotTagScan::Malformed { .. } | WikidotTagScan::Unclosed => None,
    }
}

#[derive(Clone, Copy)]
struct WikidotLiteralBlock {
    close: &'static str,
    quote_depth: usize,
    start: usize,
    content_start: usize,
}

#[derive(Clone, Copy)]
enum WikidotLiteralState {
    Normal,
    Comment { start: usize },
    Block(WikidotLiteralBlock),
}

#[derive(Clone, Copy)]
struct WikidotLiteralPolicy {
    runtime_extended: bool,
    fail_closed_inline: bool,
    own_generic_tag_heads: bool,
    own_module_bodies: bool,
}

#[cfg(test)]
const LIST_PAGES_LITERAL_POLICY: WikidotLiteralPolicy = WikidotLiteralPolicy {
    runtime_extended: true,
    fail_closed_inline: true,
    // The ListPages event scanner validates and rolls back generic heads
    // itself. Masking those prefixes here would hide malformed module heads
    // before the scanner can apply its fail-closed policy.
    own_generic_tag_heads: false,
    own_module_bodies: false,
};

const CONDITIONAL_LITERAL_POLICY: WikidotLiteralPolicy = WikidotLiteralPolicy {
    runtime_extended: false,
    fail_closed_inline: false,
    own_generic_tag_heads: false,
    own_module_bodies: true,
};

pub(super) fn collect_wikidot_conditional_literal_ranges(
    source: &str,
    ranges: &mut Vec<Range<usize>>,
) {
    collect_wikidot_literal_ranges(source, ranges, CONDITIONAL_LITERAL_POLICY);
}

fn collect_wikidot_literal_ranges(
    source: &str,
    ranges: &mut Vec<Range<usize>>,
    policy: WikidotLiteralPolicy,
) {
    // A successfully opened literal owns all delimiter-looking text until it closes.
    let bytes = source.as_bytes();
    let mut offset = 0usize;
    let mut state = WikidotLiteralState::Normal;
    let mut skip_until = 0usize;
    let mut text_tokens = TextTokenCursor::new(source);

    for line in PhysicalLines::new(source) {
        let line_end = offset + line.len();
        let body = physical_line_body(line);
        let body_end = offset + body.len();
        let (quote_depth, logical) = quote_depth_and_body(body);
        let logical_start = offset + body.len() - logical.len();
        let last_right_raw = source[offset..body_end]
            .rfind(">@")
            .map(|relative| offset + relative);
        let mut cursor = offset.max(skip_until);

        if let WikidotLiteralState::Block(block) = state {
            if block.quote_depth > 0 && quote_depth < block.quote_depth {
                ranges.push(block.start..offset);
                state = WikidotLiteralState::Normal;
            } else {
                let close_depth_matches =
                    block.quote_depth == 0 || quote_depth == block.quote_depth;
                let close_search_start = logical_start.max(block.content_start);
                let close_start = if close_search_start < body_end {
                    find_block_close(source, close_search_start, body_end, block.close)
                } else {
                    None
                };
                if close_depth_matches && let Some(close_start) = close_start {
                    let end = close_start + block.close.len();
                    ranges.push(block.start..end);
                    state = WikidotLiteralState::Normal;
                    cursor = end;
                } else {
                    offset = line_end;
                    continue;
                }
            }
        }

        while cursor < line_end {
            let remaining = &bytes[cursor..line_end];
            match state {
                WikidotLiteralState::Normal
                    if policy.runtime_extended && remaining.starts_with(b"[[$") =>
                {
                    if let Some(close) = find_token_unowned_delimiter(
                        source,
                        cursor + 3,
                        body_end,
                        "$]]",
                        &mut text_tokens,
                    ) {
                        let end = close + 3;
                        ranges.push(cursor..end);
                        cursor = end;
                    } else if policy.fail_closed_inline {
                        ranges.push(cursor..body_end);
                        cursor = body_end;
                    } else {
                        cursor += 3;
                    }
                }
                WikidotLiteralState::Normal if remaining.starts_with(b"[[") => {
                    if let Some((close, opener_end)) = wikidot_literal_block(
                        source,
                        cursor,
                        body_end,
                        policy.runtime_extended,
                        policy.own_module_bodies,
                        &mut text_tokens,
                    ) {
                        let close_start = if opener_end < body_end {
                            find_block_close(source, opener_end, body_end, close)
                        } else {
                            None
                        };
                        if let Some(close_start) = close_start {
                            let end = close_start + close.len();
                            ranges.push(cursor..end);
                            cursor = end;
                        } else {
                            state = WikidotLiteralState::Block(WikidotLiteralBlock {
                                close,
                                quote_depth,
                                start: cursor,
                                content_start: opener_end,
                            });
                            cursor = line_end;
                        }
                    } else if wikidot_multiline_map_head(source, cursor)
                        && let Some(end) =
                            wikidot_multiline_tag_end(bytes, cursor, &mut text_tokens)
                    {
                        skip_until = end;
                        cursor = end;
                    } else {
                        match scan_wikidot_tag(
                            bytes,
                            cursor,
                            bytes.len(),
                            true,
                            true,
                            &mut text_tokens,
                        ) {
                            WikidotTagScan::Complete(end) => {
                                if policy.own_generic_tag_heads {
                                    ranges.push(cursor..end);
                                }
                                // A multiline generic tag head has already lost
                                // its opener before the next physical line. Skip
                                // its remaining physical lines so the collector
                                // cannot reinterpret quoted syntax.
                                if end > body_end {
                                    skip_until = end;
                                }
                                cursor = end;
                            }
                            WikidotTagScan::Malformed { resume } => {
                                if cursor < resume {
                                    if policy.own_generic_tag_heads
                                        && policy.fail_closed_inline
                                    {
                                        ranges.push(cursor..resume);
                                    }
                                    skip_until = skip_until.max(resume);
                                }
                                cursor = resume;
                            }
                            WikidotTagScan::Unclosed => {
                                if policy.own_generic_tag_heads
                                    && policy.fail_closed_inline
                                {
                                    ranges.push(cursor..source.len());
                                }
                                skip_until = source.len();
                                cursor = source.len();
                            }
                        }
                    }
                }
                WikidotLiteralState::Normal
                    if remaining.starts_with(b"@@") && !text_tokens.contains(cursor) =>
                {
                    let run_len = bytes[cursor..body_end]
                        .iter()
                        .take_while(|byte| **byte == b'@')
                        .count();
                    let special_len = if run_len >= 6 {
                        6
                    } else if run_len >= 4 {
                        run_len
                    } else {
                        0
                    };
                    if special_len > 0 {
                        let end = cursor + special_len;
                        ranges.push(cursor..end);
                        cursor = end;
                    } else if let Some(close) = find_token_unowned_delimiter(
                        source,
                        cursor + 2,
                        body_end,
                        "@@",
                        &mut text_tokens,
                    ) {
                        let end = close + 2;
                        ranges.push(cursor..end);
                        cursor = end;
                    } else if policy.fail_closed_inline {
                        ranges.push(cursor..body_end);
                        cursor = body_end;
                    } else {
                        // FTML aborts this raw form at a physical line break or
                        // EOF, so an unmatched opener owns no literal region.
                        cursor += 2;
                    }
                }
                WikidotLiteralState::Normal
                    if policy.runtime_extended
                        && remaining.starts_with(b"@<")
                        && !text_tokens.contains(cursor) =>
                {
                    if last_right_raw.is_some_and(|last_close| cursor < last_close)
                        && let Some(close) = find_right_raw(source, cursor + 2, body_end)
                    {
                        let end = close + 2;
                        ranges.push(cursor..end);
                        cursor = end;
                    } else if policy.fail_closed_inline {
                        ranges.push(cursor..body_end);
                        cursor = body_end;
                    } else {
                        cursor += 2;
                    }
                }
                WikidotLiteralState::Normal if remaining.starts_with(b"[!--") => {
                    state = WikidotLiteralState::Comment { start: cursor };
                    cursor += 4;
                }
                WikidotLiteralState::Comment { start }
                    if remaining.starts_with(b"--]")
                        && comment_close_is_token(bytes, cursor)
                        && !text_tokens.contains(cursor) =>
                {
                    cursor += 3;
                    ranges.push(start..cursor);
                    state = WikidotLiteralState::Normal;
                }
                WikidotLiteralState::Block(_) => {
                    unreachable!(
                        "block literal lines are advanced before inline scanning"
                    )
                }
                _ => cursor += 1,
            }
        }
        offset = line_end;
    }

    match state {
        WikidotLiteralState::Comment { start } => {
            ranges.push(start..source.len());
        }
        WikidotLiteralState::Block(block) => ranges.push(block.start..source.len()),
        WikidotLiteralState::Normal => {}
    }
}

pub(super) fn quote_depth_and_body(mut body: &str) -> (usize, &str) {
    let mut quote_depth = 0;
    body = body.trim_start_matches([' ', '\t']);
    while let Some(rest) = body.strip_prefix('>') {
        quote_depth += 1;
        body = rest.trim_start_matches([' ', '\t']);
    }
    (quote_depth, body)
}

fn wikidot_literal_block(
    source: &str,
    start: usize,
    line_end: usize,
    runtime_extended: bool,
    own_module_bodies: bool,
    text_tokens: &mut TextTokenCursor,
) -> Option<(&'static str, usize)> {
    let mut lookahead_tokens = text_tokens.clone();
    let (name, name_end) = wikidot_tag_name(source, start)?;
    if name.eq_ignore_ascii_case("raw") {
        let content_start = match source.as_bytes().get(name_end) {
            Some(b' ' | b'\t' | b'\n' | b'\r') => name_end,
            Some(b']')
                if right_bracket_token(source.as_bytes(), name_end, source.len()).0 =>
            {
                name_end + 2
            }
            _ => return None,
        };
        *text_tokens = lookahead_tokens;
        return Some(("[[/raw]]", content_start));
    }
    let is_module = own_module_bodies && name.eq_ignore_ascii_case("module");
    let (opener_end, head_end) = if name.eq_ignore_ascii_case("code")
        || name.eq_ignore_ascii_case("html")
        || is_module
        || (runtime_extended && name.eq_ignore_ascii_case("math"))
    {
        let end =
            wikidot_multiline_tag_end(source.as_bytes(), start, &mut lookahead_tokens)?;
        (end, end - 2)
    } else {
        let end = first_wikidot_tag_end(source.as_bytes(), start, line_end)?;
        (end, end - 2)
    };
    let close = if name.eq_ignore_ascii_case("code") {
        "[[/code]]"
    } else if name.eq_ignore_ascii_case("html") {
        "[[/html]]"
    } else if is_module {
        "[[/module]]"
    } else if runtime_extended && name.eq_ignore_ascii_case("math") {
        "[[/math]]"
    } else if runtime_extended
        && name.eq_ignore_ascii_case("embed")
        && source[name_end..head_end].trim().is_empty()
    {
        "[[/embed]]"
    } else {
        return None;
    };
    *text_tokens = lookahead_tokens;
    Some((close, opener_end))
}

fn wikidot_multiline_map_head(source: &str, start: usize) -> bool {
    wikidot_tag_name(source, start).is_some_and(|(name, _)| {
        name.eq_ignore_ascii_case("module") || name.eq_ignore_ascii_case("module654")
    })
}

fn wikidot_tag_name(source: &str, start: usize) -> Option<(&str, usize)> {
    let bytes = source.as_bytes();
    let mut cursor = start + 2;
    while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
        cursor += 1;
    }
    let (name, name_end) = wikidot_trimmed_name(bytes, cursor);
    let name = std::str::from_utf8(name?).ok()?;
    Some((name, name_end))
}

fn first_wikidot_tag_end(bytes: &[u8], start: usize, end: usize) -> Option<usize> {
    let mut cursor = start + 2;
    while cursor < end {
        if bytes[cursor] == b']' {
            let (right_block, token_len) = right_bracket_token(bytes, cursor, end);
            if right_block {
                return Some(cursor + token_len);
            }
            if token_len == 3 {
                return None;
            }
            cursor += token_len;
        } else {
            cursor += 1;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn literal_ranges(source: &str) -> Vec<Range<usize>> {
        let mut ranges = Vec::new();
        collect_wikidot_literal_ranges(source, &mut ranges, LIST_PAGES_LITERAL_POLICY);
        ranges
    }

    fn is_owned(ranges: &[Range<usize>], offset: usize) -> bool {
        ranges
            .iter()
            .any(|range| range.start <= offset && offset < range.end)
    }

    #[test]
    fn url_and_email_owned_raw_openers_do_not_steal_following_blocks() {
        for (source, opener) in [
            ("foo@bar.example@@x [[code]] @@ LP [[/code]]", "@@"),
            ("https://e.test/a@@b [[code]] @@ LP [[/code]]", "@@"),
            ("foo@bar.example@<x [[code]] >@ LP [[/code]]", "@<"),
            ("https://e.test/a@<b [[code]] >@ LP [[/code]]", "@<"),
        ] {
            let ranges = literal_ranges(source);

            assert!(
                !is_owned(&ranges, source.find(opener).unwrap()),
                "{source:?}"
            );
            assert!(is_owned(&ranges, source.find("LP").unwrap()), "{source:?}");
        }
    }

    #[test]
    fn url_and_email_owned_delimiters_do_not_close_inline_literals() {
        for owner in ["foo@bar.example", "https://e.test/a"] {
            for source in [
                format!("@@{owner}@@x hidden @@ live"),
                format!("[[$ {owner}$]]x hidden $]] live"),
                format!("[!--{owner}--]x hidden --] live"),
            ] {
                let ranges = literal_ranges(&source);

                assert!(
                    is_owned(&ranges, source.find("hidden").unwrap()),
                    "{source:?}",
                );
                assert!(
                    !is_owned(&ranges, source.find("live").unwrap()),
                    "{source:?}",
                );
            }
        }
    }

    #[test]
    fn list_pages_scanner_policy_does_not_premask_generic_tag_heads() {
        for line_end in ["\n", "\r\n", "\r"] {
            let source = format!(
                "[[div{line_end}class=\"[[module ListPages name='fake']]X[[/module]]\"{line_end}data-kind=\"compatibility\"]]{line_end}[[module ListPages name=\"live\"]]Y[[/module]]",
            );
            let ranges = literal_ranges(&source);

            assert!(
                !is_owned(&ranges, source.find("fake").unwrap()),
                "{line_end:?}"
            );
            assert!(
                !is_owned(&ranges, source.find("name=\"live\"").unwrap()),
                "{line_end:?}",
            );
        }
    }

    #[test]
    fn malformed_generic_heads_recover_at_pinned_boundaries() {
        for source in [
            "[[span title='unterminated\n[[module ListPages name=\"live\"]]Y[[/module]]",
            "[[span class=unterminated\r\n[[module ListPages name=\"live\"]]Y[[/module]]",
            "[[span title=\"malformed\"]]][[module ListPages name=\"live\"]]Y[[/module]]",
            "[[span [[module ListPages name=\"live\"]]Y[[/module]]",
        ] {
            let ranges = literal_ranges(source);

            assert!(
                !is_owned(&ranges, source.find("name=\"live\"").unwrap()),
                "{source:?}",
            );
        }
    }

    #[test]
    fn raw_close_after_the_name_terminator_does_not_own_the_suffix() {
        for source in [
            "[[raw [[/raw]] [[module ListPages name='live']]X[[/module]]",
            "[[raw abc [[/raw]] [[module ListPages name='live']]X[[/module]]",
        ] {
            let ranges = literal_ranges(source);

            assert!(
                is_owned(&ranges, source.find("[[raw").unwrap()),
                "{source:?}"
            );
            assert!(
                !is_owned(&ranges, source.find("name='live'").unwrap()),
                "{source:?}",
            );
        }

        let source = concat!(
            "[[raw ]]] [[module ListPages name='hidden']]X[[/module]] [[/raw]] ",
            "[[module ListPages name='live']]Y[[/module]]",
        );
        let ranges = literal_ranges(source);
        assert!(is_owned(&ranges, source.find("name='hidden'").unwrap()));
        assert!(!is_owned(&ranges, source.find("name='live'").unwrap()));
    }

    #[test]
    fn unicode_trimmed_block_names_open_literal_regions() {
        let source = "[[code\u{a0} ]]hidden[[/code]] live";
        let ranges = literal_ranges(source);

        assert!(is_owned(&ranges, source.find("hidden").unwrap()));
        assert!(!is_owned(&ranges, source.find("live").unwrap()));

        for block in ["code", "html", "math", "raw"] {
            for leading in ['\u{000b}', '\u{000c}'] {
                let source = format!("[[{leading}{block} ]]hidden[[/{block}]] live",);
                let ranges = literal_ranges(&source);
                assert!(
                    is_owned(&ranges, source.find("hidden").unwrap()),
                    "{source:?}"
                );
                assert!(
                    !is_owned(&ranges, source.find("live").unwrap()),
                    "{source:?}"
                );
            }
        }

        let source = "[[raw\u{a0}\n]]\nhidden\n[[/raw]] live";
        let ranges = literal_ranges(source);

        assert!(is_owned(&ranges, source.find("hidden").unwrap()));
        assert!(!is_owned(&ranges, source.find("live").unwrap()));
    }

    #[test]
    fn repeated_right_link_brackets_do_not_end_literal_block_heads_or_tails() {
        for block in ["code", "html", "math", "raw", "embed"] {
            let malformed_open =
                format!("[[{block}]]][[module ListPages name=\"live\"]]Y[[/module]]",);
            let ranges = literal_ranges(&malformed_open);
            assert!(
                !is_owned(&ranges, malformed_open.find("name=\"live\"").unwrap()),
                "{block}",
            );
        }

        let source = concat!(
            "[[code]]hidden[[/code]]]still-hidden[[/CoDe]] ",
            "[[module ListPages name=\"live\"]]Y[[/module]]",
        );
        let ranges = literal_ranges(source);
        assert!(is_owned(&ranges, source.find("still-hidden").unwrap()));
        assert!(!is_owned(&ranges, source.find("name=\"live\"").unwrap()));
    }

    #[test]
    fn repeated_brackets_after_independent_inline_closers_remain_live() {
        for source in [
            "@@hidden@@]]] live",
            "[[$ hidden $]]] live",
            "[!--hidden--]]] live",
        ] {
            let ranges = literal_ranges(source);

            assert!(
                is_owned(&ranges, source.find("hidden").unwrap()),
                "{source:?}"
            );
            assert!(
                !is_owned(&ranges, source.find("live").unwrap()),
                "{source:?}"
            );
        }
    }

    #[test]
    fn dense_same_line_literal_blocks_have_local_close_searches() {
        const BLOCKS: usize = 10_000;
        let source = "[[code]]x[[/CoDe]]".repeat(BLOCKS);
        let ranges = literal_ranges(&source);

        assert_eq!(ranges.len(), BLOCKS);
        assert_eq!(ranges.first().unwrap().start, 0);
        assert_eq!(ranges.last().unwrap().end, source.len());
    }
}
