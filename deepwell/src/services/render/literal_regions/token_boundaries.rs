/*
 * services/render/literal_regions/token_boundaries.rs
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
use std::sync::Arc;

#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
thread_local! {
    static EMAIL_SCAN_BYTES_EXAMINED: Cell<usize> = const { Cell::new(0) };
}

#[inline]
fn record_email_bytes_examined(count: usize) {
    #[cfg(test)]
    EMAIL_SCAN_BYTES_EXAMINED.with(|total| total.set(total.get() + count));
    #[cfg(not(test))]
    let _ = count;
}

enum EmailScan {
    Token {
        end: usize,
    },
    /// No alphanumeric start before this stop can produce an email token.
    Failed {
        safe_until: usize,
    },
}

/// URL and email spans owned by the pinned FTML tokenizer.
///
/// Literal-looking bytes inside either token must not be reinterpreted as
/// source-level delimiters. Ranges and membership checks are both monotone.
#[derive(Clone)]
pub(in crate::services::render) struct TextTokenIndex {
    ranges: Arc<[Range<usize>]>,
}

#[derive(Clone)]
pub(in crate::services::render) struct TextTokenCursor {
    ranges: Arc<[Range<usize>]>,
    cursor: usize,
    previous: Option<Range<usize>>,
}

impl TextTokenIndex {
    pub(in crate::services::render) fn new(source: &str) -> Self {
        let bytes = source.as_bytes();
        let mut ranges = Vec::new();
        let mut cursor = 0usize;
        let mut failed_email_until = 0usize;
        while cursor < bytes.len() {
            if bytes[cursor].is_ascii_alphanumeric() {
                if matches!(bytes[cursor], b'f' | b'h')
                    && let Some(end) = scan_url(bytes, cursor)
                {
                    ranges.push(cursor..end);
                    cursor = end;
                    continue;
                }
                let identifier_end = scan_identifier(bytes, cursor);
                // Failed email scans share the same next reserved stop for every
                // later identifier in the scanned span. URL recognition still
                // runs above before this proof lets us skip the email rescan.
                if cursor >= failed_email_until {
                    match scan_email(bytes, cursor, identifier_end) {
                        EmailScan::Token { end } => {
                            ranges.push(cursor..end);
                            cursor = end;
                            continue;
                        }
                        EmailScan::Failed { safe_until } => {
                            debug_assert!(safe_until >= identifier_end);
                            failed_email_until = failed_email_until.max(safe_until);
                        }
                    }
                }
                cursor = identifier_end;
            } else {
                cursor += source[cursor..]
                    .chars()
                    .next()
                    .expect("cursor is before the UTF-8 source end")
                    .len_utf8();
            }
        }
        Self {
            ranges: ranges.into(),
        }
    }

    pub(in crate::services::render) fn cursor(&self) -> TextTokenCursor {
        TextTokenCursor {
            ranges: Arc::clone(&self.ranges),
            cursor: 0,
            previous: None,
        }
    }
}

impl TextTokenCursor {
    pub(in crate::services::render) fn new(source: &str) -> Self {
        TextTokenIndex::new(source).cursor()
    }

    pub(in crate::services::render) fn contains(&mut self, offset: usize) -> bool {
        while self
            .ranges
            .get(self.cursor)
            .is_some_and(|range| range.end <= offset)
        {
            self.previous = self.ranges.get(self.cursor).cloned();
            self.cursor += 1;
        }
        self.ranges
            .get(self.cursor)
            .is_some_and(|range| range.start <= offset && offset < range.end)
            || self
                .previous
                .as_ref()
                .is_some_and(|range| range.start <= offset && offset < range.end)
    }

    pub(in crate::services::render) fn range_end_at(
        &mut self,
        offset: usize,
    ) -> Option<usize> {
        self.contains(offset);
        self.ranges
            .get(self.cursor)
            .filter(|range| range.start == offset)
            .map(|range| range.end)
    }
}

fn scan_identifier(bytes: &[u8], start: usize) -> usize {
    let mut end = start;
    while bytes.get(end).is_some_and(u8::is_ascii_alphanumeric) {
        end += 1;
    }
    end
}

fn is_discarded_control(byte: u8) -> bool {
    matches!(
        byte,
        b'\0'
            | b'\x01'
            | b'\x08'
            | b'\x0b'
            | b'\x0c'
            | b'\x0e'
            | b'\x1a'
            | b'\x1b'
            | b'\x1c'
            | b'\x1f'
            | b'\x7f'
    )
}

fn scan_email(bytes: &[u8], start: usize, identifier_end: usize) -> EmailScan {
    match bytes.get(identifier_end) {
        Some(b' ' | b'\t' | b'\n' | b'\r') | None => {
            return EmailScan::Failed {
                safe_until: identifier_end,
            };
        }
        _ => {}
    }

    // Keep these stops in lockstep with FTML 4fc7df28's context-free email
    // scanner. Punctuation outside this set remains owned by the email token.
    let mut at = identifier_end;
    while at < bytes.len()
        && !is_discarded_control(bytes[at])
        && !matches!(
            bytes[at],
            b' ' | b'\t' | b'@' | b'[' | b']' | b'{' | b'}' | b'<' | b'>' | b'\n' | b'\r'
        )
    {
        at += 1;
    }
    record_email_bytes_examined(at - identifier_end);
    if at == start || bytes.get(at) != Some(&b'@') {
        return EmailScan::Failed { safe_until: at };
    }

    let mut dot = at + 1;
    while dot < bytes.len()
        && !is_discarded_control(bytes[dot])
        && !matches!(
            bytes[dot],
            b' ' | b'\t'
                | b'@'
                | b'.'
                | b'['
                | b']'
                | b'{'
                | b'}'
                | b'<'
                | b'>'
                | b'\n'
                | b'\r'
        )
    {
        dot += 1;
    }
    record_email_bytes_examined(dot - (at + 1));
    if dot == at + 1 || bytes.get(dot) != Some(&b'.') {
        return EmailScan::Failed { safe_until: dot };
    }

    let mut end = dot + 1;
    while end < bytes.len()
        && !is_discarded_control(bytes[end])
        && !matches!(
            bytes[end],
            b' ' | b'\t' | b'[' | b']' | b'{' | b'}' | b'<' | b'>' | b'\n' | b'\r'
        )
    {
        end += 1;
    }
    record_email_bytes_examined(end - (dot + 1));
    if end > dot + 1 {
        EmailScan::Token { end }
    } else {
        EmailScan::Failed { safe_until: end }
    }
}

fn scan_url(bytes: &[u8], start: usize) -> Option<usize> {
    let body_start = if bytes[start..].starts_with(b"http://") {
        start + 7
    } else if bytes[start..].starts_with(b"https://") {
        start + 8
    } else if bytes[start..].starts_with(b"ftp://") {
        start + 6
    } else {
        return None;
    };
    let mut end = body_start;
    while end < bytes.len()
        && !is_discarded_control(bytes[end])
        && !matches!(bytes[end], b'\n' | b'\r' | b' ' | b'"' | b'|' | b'[' | b']')
        && !bytes[end..].starts_with(b">@")
    {
        end += 1;
    }
    (end > body_start).then_some(end)
}

pub(super) fn right_raw_is_token(bytes: &[u8], start: usize) -> bool {
    if bytes.get(start.wrapping_sub(1)) == Some(&b'>') {
        return false;
    }
    let tilde_count = bytes[..start]
        .iter()
        .rev()
        .take_while(|byte| **byte == b'~')
        .count();
    tilde_count < 3
}

pub(super) fn comment_close_is_token(bytes: &[u8], start: usize) -> bool {
    bytes.get(start.wrapping_sub(1)) != Some(&b'-')
}

pub(super) fn block_close_is_token(bytes: &[u8], start: usize, end: usize) -> bool {
    left_block_start_in_run(bytes, start).0 == Some(start)
        && right_bracket_token(bytes, end - 2, bytes.len()) == (true, 2)
}

pub(super) fn find_token_unowned_delimiter(
    source: &str,
    start: usize,
    end: usize,
    delimiter: &str,
    text_tokens: &mut TextTokenCursor,
) -> Option<usize> {
    let mut cursor = start;
    while let Some(relative) = source[cursor..end].find(delimiter) {
        let candidate = cursor + relative;
        if !text_tokens.contains(candidate) {
            return Some(candidate);
        }
        cursor = candidate + 1;
    }
    None
}

pub(super) fn find_right_raw(source: &str, start: usize, end: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut cursor = start;
    while let Some(relative) = source[cursor..end].find(">@") {
        let candidate = cursor + relative;
        if right_raw_is_token(bytes, candidate) {
            return Some(candidate);
        }
        cursor = candidate + 1;
    }
    None
}

pub(super) fn find_block_close(
    source: &str,
    start: usize,
    end: usize,
    close: &str,
) -> Option<usize> {
    let bytes = source.as_bytes();
    let close = close.as_bytes();
    debug_assert!(close.is_ascii());
    let mut cursor = start;
    while cursor + close.len() <= end {
        let relative = bytes[cursor..end]
            .iter()
            .position(|byte| *byte == close[0])?;
        let candidate = cursor + relative;
        let candidate_end = candidate + close.len();
        if candidate_end <= end
            && bytes[candidate..candidate_end].eq_ignore_ascii_case(close)
            && block_close_is_token(bytes, candidate, candidate_end)
        {
            return Some(candidate);
        }
        cursor = candidate + 1;
    }
    None
}

/// Return whether the pinned tokenizer emits `RightBlock` at `start` and the
/// number of bytes consumed by the bracket token it emits there.
pub(in crate::services::render) fn right_bracket_token(
    bytes: &[u8],
    start: usize,
    end: usize,
) -> (bool, usize) {
    debug_assert_eq!(bytes.get(start), Some(&b']'));
    let remaining = &bytes[start..end];
    if remaining.starts_with(b"]]]]") {
        return (false, 3);
    }
    let trailing_right_link = start >= 3
        && bytes[start - 3..start].iter().all(|byte| *byte == b']')
        && start.checked_sub(4).and_then(|index| bytes.get(index)) != Some(&b']');
    if remaining.starts_with(b"]]]") && !trailing_right_link {
        (false, 3)
    } else if remaining.starts_with(b"]]") && !trailing_right_link {
        (true, 2)
    } else {
        (false, 1)
    }
}

pub(in crate::services::render) fn wikidot_right_bracket_token(
    bytes: &[u8],
    start: usize,
    end: usize,
    text_tokens: &mut TextTokenCursor,
) -> (bool, usize) {
    if start > 0 && bytes[start - 1] == b'$' && !text_tokens.contains(start - 1) {
        return (false, 2.min(end - start));
    }
    if start >= 2
        && bytes.get(start - 2..start) == Some(&b"--"[..])
        && comment_close_is_token(bytes, start - 2)
        && !text_tokens.contains(start - 2)
    {
        return (false, 1);
    }
    right_bracket_token(bytes, start, end)
}

/// Return the pinned tokenizer's possible `LeftBlock` or `LeftBlockEnd`
/// start in the contiguous `[` run containing `candidate`, plus that run's
/// exclusive end. The four-bracket precedence consumes one bracket followed
/// by three-bracket link tokens; only a final two-byte remainder is a block
/// token.
pub(in crate::services::render) fn left_block_start_in_run(
    bytes: &[u8],
    candidate: usize,
) -> (Option<usize>, usize) {
    debug_assert_eq!(bytes.get(candidate..candidate + 2), Some(&b"[["[..]));
    let mut run_start = candidate;
    while run_start > 0 && bytes[run_start - 1] == b'[' {
        run_start -= 1;
    }
    let mut run_end = candidate + 2;
    while bytes.get(run_end) == Some(&b'[') {
        run_end += 1;
    }
    let run_len = run_end - run_start;
    let block_start = if run_len == 2 {
        Some(run_start)
    } else if run_len >= 6 && run_len.is_multiple_of(3) {
        Some(run_end - 2)
    } else {
        None
    };
    (block_start.filter(|start| *start >= candidate), run_end)
}

/// Return the earliest nested position that must be reconsidered if an outer
/// speculative head rolls back. A competing bracket run may contain no
/// `LeftBlock` token while still containing an unresolved `[[#` marker.
pub(in crate::services::render) fn rollback_start_in_left_run(
    bytes: &[u8],
    candidate: usize,
    block_start: Option<usize>,
    run_end: usize,
) -> Option<usize> {
    let parser_function = (candidate..run_end.saturating_sub(1))
        .find(|start| bytes.get(*start..*start + 3) == Some(&b"[[#"[..]));
    match (block_start, parser_function) {
        (Some(block), Some(function)) => Some(block.min(function)),
        (Some(block), None) => Some(block),
        (None, Some(function)) => Some(function),
        (None, None) => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::services::render) enum WikidotTagScan {
    Complete(usize),
    Malformed { resume: usize },
    Unclosed,
}

pub(in crate::services::render) fn scan_wikidot_tag(
    bytes: &[u8],
    start: usize,
    end: usize,
    allow_multiline: bool,
    reject_unquoted_values: bool,
    text_tokens: &mut TextTokenCursor,
) -> WikidotTagScan {
    debug_assert_eq!(bytes.get(start..start + 2), Some(&b"[["[..]));
    let (block_start, run_end) = left_block_start_in_run(bytes, start);
    if block_start != Some(start) {
        return WikidotTagScan::Malformed {
            resume: block_start.unwrap_or(run_end),
        };
    }
    let mut cursor = start + 2;
    let mut quote = None;
    let mut malformed_unquoted_value = false;
    let mut bare_image_link = false;
    let argument_scan = WikidotTagArgumentScan::new(bytes, start, text_tokens);
    if argument_scan.whole_head_value() {
        return match scan_wikidot_whole_head_value(
            bytes,
            argument_scan.name_end(),
            end,
            text_tokens,
        ) {
            WikidotWholeHeadScan::Complete { end, .. } => WikidotTagScan::Complete(end),
            WikidotWholeHeadScan::Malformed { resume, .. } => {
                WikidotTagScan::Malformed { resume }
            }
            WikidotWholeHeadScan::Unclosed { .. } => WikidotTagScan::Unclosed,
        };
    }
    while cursor < end {
        if bare_image_link {
            if bytes[cursor] == b'\t' && text_tokens.contains(cursor) {
                cursor += 1;
                continue;
            } else if matches!(bytes[cursor], b' ' | b'\t' | b'\n' | b'\r') {
                bare_image_link = false;
            } else if bytes[cursor] == b']' {
                let (right_block, token_len) =
                    wikidot_right_bracket_token(bytes, cursor, end, text_tokens);
                if right_block {
                    return WikidotTagScan::Complete(cursor + token_len);
                }
                cursor += token_len;
                continue;
            } else {
                cursor += 1;
                continue;
            }
        }
        if argument_scan.in_positional_value(cursor) {
            cursor += 1;
            continue;
        }
        if matches!(bytes[cursor], b'\n' | b'\r') {
            if !allow_multiline || quote.is_some() || malformed_unquoted_value {
                return WikidotTagScan::Malformed {
                    resume: physical_line_resume(bytes, cursor, end),
                };
            }
            cursor = physical_line_resume(bytes, cursor, end);
            continue;
        }
        if quote.is_none() && cursor > start + 2 && bytes[cursor..end].starts_with(b"[[")
        {
            let (block_start, run_end) = left_block_start_in_run(bytes, cursor);
            return WikidotTagScan::Malformed {
                resume: block_start.unwrap_or(run_end),
            };
        }
        match (quote, bytes[cursor]) {
            (Some(b'"'), b'"')
                if !quote_is_escaped(bytes, cursor, text_tokens)
                    && !text_tokens.contains(cursor)
                    && double_quote_ends_wikidot_argument(bytes, cursor, text_tokens) =>
            {
                quote = None;
            }
            (Some(b'\''), b'\'')
                if !quote_is_escaped(bytes, cursor, text_tokens)
                    && !text_tokens.contains(cursor) =>
            {
                quote = None;
            }
            (None, b'\'' | b'"')
                if !quote_is_escaped(bytes, cursor, text_tokens)
                    && !text_tokens.contains(cursor) =>
            {
                quote = Some(bytes[cursor]);
            }
            (None, b'=') if reject_unquoted_values && !text_tokens.contains(cursor) => {
                let mut value_start = cursor + 1;
                while matches!(bytes.get(value_start), Some(b' ' | b'\t')) {
                    value_start += 1;
                }
                match argument_scan.classify(bytes, cursor, value_start) {
                    WikidotArgumentValueKind::Accepted => {}
                    WikidotArgumentValueKind::BareImageLink => {
                        bare_image_link = true;
                        cursor = value_start;
                        continue;
                    }
                    WikidotArgumentValueKind::Malformed => {
                        malformed_unquoted_value = true;
                    }
                }
            }
            (None, b']') => {
                let (right_block, token_len) =
                    wikidot_right_bracket_token(bytes, cursor, end, text_tokens);
                if right_block {
                    return WikidotTagScan::Complete(cursor + token_len);
                }
                if token_len == 3 && !argument_scan.in_positional_value(cursor) {
                    return WikidotTagScan::Malformed {
                        resume: cursor + token_len,
                    };
                }
                cursor += token_len;
                continue;
            }
            _ => {}
        }
        cursor += 1;
    }
    WikidotTagScan::Unclosed
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::services::render) enum WikidotArgumentValueKind {
    Accepted,
    BareImageLink,
    Malformed,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::services::render) struct WikidotTagArgumentScan {
    positional_end: Option<usize>,
    image: bool,
    whole_head_value: bool,
    name_end: usize,
}

impl WikidotTagArgumentScan {
    pub(in crate::services::render) fn new(
        bytes: &[u8],
        start: usize,
        text_tokens: &mut TextTokenCursor,
    ) -> Self {
        let (positional, image, whole_head_value, name_end) =
            wikidot_tag_argument_shape(bytes, start);
        let positional_end = positional.then(|| {
            let mut lookahead_tokens = text_tokens.clone();
            let mut cursor = name_end;
            skip_wikidot_name_delimiter(bytes, &mut cursor, bytes.len());
            while let Some(byte) = bytes.get(cursor) {
                if *byte == b'\t' && lookahead_tokens.contains(cursor) {
                    cursor += 1;
                    continue;
                }
                if matches!(byte, b' ' | b'\t' | b'\n' | b'\r') {
                    break;
                }
                if *byte == b']' {
                    let (right_block, token_len) = wikidot_right_bracket_token(
                        bytes,
                        cursor,
                        bytes.len(),
                        &mut lookahead_tokens,
                    );
                    if right_block {
                        break;
                    }
                    cursor += token_len;
                } else {
                    cursor += 1;
                }
            }
            *text_tokens = lookahead_tokens;
            cursor
        });
        Self {
            positional_end,
            image,
            whole_head_value,
            name_end,
        }
    }

    pub(in crate::services::render) fn whole_head_value(self) -> bool {
        self.whole_head_value
    }

    pub(in crate::services::render) fn name_end(self) -> usize {
        self.name_end
    }

    pub(in crate::services::render) fn in_positional_value(self, offset: usize) -> bool {
        self.positional_end.is_some_and(|end| offset < end)
    }

    pub(in crate::services::render) fn classify(
        self,
        bytes: &[u8],
        equals: usize,
        value_start: usize,
    ) -> WikidotArgumentValueKind {
        if self
            .positional_end
            .is_some_and(|source_end| equals < source_end)
        {
            return WikidotArgumentValueKind::Accepted;
        }
        let Some(key) = wikidot_argument_key_before_equals(bytes, equals) else {
            return WikidotArgumentValueKind::Accepted;
        };
        if matches!(bytes.get(value_start), Some(b'\'' | b'"')) {
            return WikidotArgumentValueKind::Accepted;
        }
        if self.image
            && key.eq_ignore_ascii_case(b"link")
            && bytes.get(value_start).is_some_and(|byte| {
                !(matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
                    || *byte == b']'
                        && right_bracket_token(bytes, value_start, bytes.len()).0)
            })
        {
            WikidotArgumentValueKind::BareImageLink
        } else {
            WikidotArgumentValueKind::Malformed
        }
    }
}

fn wikidot_tag_argument_shape(bytes: &[u8], start: usize) -> (bool, bool, bool, usize) {
    let mut name_start = start + 2;
    let star = bytes.get(name_start) == Some(&b'*');
    if star {
        name_start += 1;
    }
    while matches!(bytes.get(name_start), Some(b' ' | b'\t')) {
        name_start += 1;
    }
    let (name, name_end) = wikidot_trimmed_name(bytes, name_start);
    let Some(unstarred) = name else {
        return (false, false, false, name_end);
    };
    let star_allowed = !star
        || unstarred.eq_ignore_ascii_case(b"radio")
        || unstarred.eq_ignore_ascii_case(b"radio-button")
        || unstarred.eq_ignore_ascii_case(b"user");
    let image = !star
        && (unstarred.eq_ignore_ascii_case(b"image")
            || unstarred.eq_ignore_ascii_case(b"=image")
            || unstarred.eq_ignore_ascii_case(b"<image")
            || unstarred.eq_ignore_ascii_case(b">image")
            || unstarred.eq_ignore_ascii_case(b"f<image")
            || unstarred.eq_ignore_ascii_case(b"f>image"));
    let positional = star_allowed
        && (image
            || unstarred.eq_ignore_ascii_case(b"date")
            || unstarred.eq_ignore_ascii_case(b"embed")
            || unstarred.eq_ignore_ascii_case(b"iframe")
            || unstarred.eq_ignore_ascii_case(b"include-elements")
            || unstarred.eq_ignore_ascii_case(b"audio")
            || unstarred.eq_ignore_ascii_case(b"video")
            || unstarred.eq_ignore_ascii_case(b"radio")
            || unstarred.eq_ignore_ascii_case(b"radio-button")
            || unstarred.eq_ignore_ascii_case(b"module")
            || unstarred.eq_ignore_ascii_case(b"module654"));
    let (unscored, score) = unstarred
        .strip_suffix(b"_")
        .map_or((unstarred, false), |name| (name, true));
    let whole_head_value = star_allowed
        && matches_whole_head_value_name(unscored)
        && (!star || unscored.eq_ignore_ascii_case(b"user"))
        && (!score || unscored.eq_ignore_ascii_case(b"bibcite"));
    (positional, image, whole_head_value, name_end)
}

pub(in crate::services::render) fn wikidot_trimmed_name(
    bytes: &[u8],
    mut cursor: usize,
) -> (Option<&[u8]>, usize) {
    while !wikidot_name_delimiter(bytes, cursor) {
        let Some((character, len)) = next_utf8_character(bytes, cursor) else {
            return (None, cursor);
        };
        if !character.is_whitespace() {
            break;
        }
        cursor += len;
    }
    let name_start = cursor;
    while !wikidot_name_delimiter(bytes, cursor) {
        let Some((character, len)) = next_utf8_character(bytes, cursor) else {
            return (None, cursor);
        };
        if character.is_whitespace() {
            break;
        }
        cursor += len;
        if cursor - name_start > 24 {
            return (None, cursor);
        }
    }
    let name_end = cursor;
    while !wikidot_name_delimiter(bytes, cursor) {
        let Some((character, len)) = next_utf8_character(bytes, cursor) else {
            return (None, cursor);
        };
        if !character.is_whitespace() {
            return (None, cursor);
        }
        cursor += len;
    }
    (
        (name_start < name_end).then_some(&bytes[name_start..name_end]),
        cursor,
    )
}

fn wikidot_name_delimiter(bytes: &[u8], cursor: usize) -> bool {
    match bytes.get(cursor) {
        None | Some(b' ' | b'\t' | b'\n' | b'\r') => true,
        Some(b']') => right_bracket_token(bytes, cursor, bytes.len()).0,
        _ => false,
    }
}

fn next_utf8_character(bytes: &[u8], cursor: usize) -> Option<(char, usize)> {
    let width = match *bytes.get(cursor)? {
        0x00..=0x7f => 1,
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => return None,
    };
    let character = std::str::from_utf8(bytes.get(cursor..cursor + width)?)
        .ok()?
        .chars()
        .next()?;
    Some((character, character.len_utf8()))
}

fn matches_whole_head_value_name(name: &[u8]) -> bool {
    [
        &b"target"[..],
        b"anchortarget",
        b"equation",
        b"eref",
        b"eqref",
        b"tab",
        b"bibcite",
        b"lines",
        b"newlines",
        b"rb",
        b"ruby2",
        b"char",
        b"character",
        b"math",
        b"ifcategory",
        b"size",
        b"user",
        b"iftags",
    ]
    .into_iter()
    .any(|accepted| name.eq_ignore_ascii_case(accepted))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::services::render) enum WikidotWholeHeadScan {
    Complete {
        end: usize,
        first_rollback_marker: Option<usize>,
    },
    Malformed {
        resume: usize,
        first_rollback_marker: Option<usize>,
    },
    Unclosed {
        first_rollback_marker: Option<usize>,
    },
}

pub(in crate::services::render) fn scan_wikidot_whole_head_value(
    bytes: &[u8],
    mut cursor: usize,
    end: usize,
    text_tokens: &mut TextTokenCursor,
) -> WikidotWholeHeadScan {
    skip_wikidot_name_delimiter(bytes, &mut cursor, end);
    let mut first_rollback_marker = None;
    while cursor < end {
        match bytes[cursor] {
            b'\n' | b'\r' => {
                return WikidotWholeHeadScan::Malformed {
                    resume: physical_line_resume(bytes, cursor, end),
                    first_rollback_marker,
                };
            }
            b'[' if bytes.get(cursor + 1) == Some(&b'[') => {
                let (block_start, run_end) = left_block_start_in_run(bytes, cursor);
                first_rollback_marker = first_rollback_marker.or(
                    rollback_start_in_left_run(bytes, cursor, block_start, run_end),
                );
                cursor = run_end;
            }
            b']' => {
                let (right_block, token_len) =
                    wikidot_right_bracket_token(bytes, cursor, end, text_tokens);
                cursor += token_len;
                if right_block {
                    return WikidotWholeHeadScan::Complete {
                        end: cursor,
                        first_rollback_marker,
                    };
                }
            }
            _ => cursor += 1,
        }
    }
    WikidotWholeHeadScan::Unclosed {
        first_rollback_marker,
    }
}

fn skip_wikidot_name_delimiter(bytes: &[u8], cursor: &mut usize, end: usize) {
    if matches!(bytes.get(*cursor), Some(b' ' | b'\t')) {
        while matches!(bytes.get(*cursor), Some(b' ' | b'\t')) {
            *cursor += 1;
        }
    } else if matches!(bytes.get(*cursor), Some(b'\n' | b'\r')) {
        while *cursor < end && matches!(bytes.get(*cursor), Some(b'\n' | b'\r')) {
            *cursor = physical_line_resume(bytes, *cursor, end);
        }
        while matches!(bytes.get(*cursor), Some(b' ' | b'\t')) {
            *cursor += 1;
        }
    }
}

fn wikidot_argument_key_before_equals(bytes: &[u8], equals: usize) -> Option<&[u8]> {
    let mut key_end = equals;
    while key_end > 0 && matches!(bytes[key_end - 1], b' ' | b'\t') {
        key_end -= 1;
    }
    let mut key_start = key_end;
    while key_start > 0
        && (bytes[key_start - 1].is_ascii_alphanumeric()
            || matches!(bytes[key_start - 1], b'_' | b'-'))
    {
        key_start -= 1;
    }
    let boundary = key_start.checked_sub(1).and_then(|index| bytes.get(index));
    (key_start < key_end
        && boundary.is_some_and(|byte| {
            byte.is_ascii_whitespace() || matches!(byte, b'\'' | b'"')
        }))
    .then_some(&bytes[key_start..key_end])
}

fn physical_line_resume(bytes: &[u8], cursor: usize, end: usize) -> usize {
    if bytes.get(cursor) == Some(&b'\r')
        && cursor + 1 < end
        && bytes.get(cursor + 1) == Some(&b'\n')
    {
        cursor + 2
    } else {
        cursor + 1
    }
}

pub(in crate::services::render) fn quote_is_escaped(
    bytes: &[u8],
    quote: usize,
    text_tokens: &TextTokenCursor,
) -> bool {
    if bytes.get(quote.wrapping_sub(1)) != Some(&b'\\') {
        return false;
    }
    let mut lookbehind_tokens = text_tokens.clone();
    if lookbehind_tokens.contains(quote - 1) {
        return false;
    }
    let backslashes = bytes[..quote]
        .iter()
        .rev()
        .take_while(|byte| **byte == b'\\')
        .count();
    backslashes % 2 == 1
}

pub(in crate::services::render) fn double_quote_ends_wikidot_argument(
    bytes: &[u8],
    quote: usize,
    text_tokens: &TextTokenCursor,
) -> bool {
    let mut lookahead_tokens = text_tokens.clone();
    let mut cursor = quote + 1;
    if cursor >= bytes.len()
        || (bytes[cursor] == b']'
            && wikidot_right_bracket_token(
                bytes,
                cursor,
                bytes.len(),
                &mut lookahead_tokens,
            )
            .0)
        || matches!(bytes[cursor], b'\n' | b'\r')
    {
        return true;
    }
    if !matches!(bytes[cursor], b' ' | b'\t') {
        return false;
    }

    while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
        cursor += 1;
    }
    if cursor >= bytes.len() {
        return true;
    }
    if bytes.get(cursor) == Some(&b']')
        && wikidot_right_bracket_token(bytes, cursor, bytes.len(), &mut lookahead_tokens)
            .0
    {
        return true;
    }
    let key_start = cursor;
    while bytes
        .get(cursor)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        cursor += 1;
    }
    if cursor == key_start {
        return false;
    }
    if lookahead_tokens.contains(key_start) {
        return false;
    }
    while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
        cursor += 1;
    }
    bytes.get(cursor) == Some(&b'=') && !lookahead_tokens.contains(cursor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_trimmed_tag_names_keep_the_consumed_name_end() {
        for (source, positional, whole_head_value) in [
            ("[[user\u{a0} alice]]", false, true),
            ("[[*user\u{a0} alice]]", false, true),
            ("[[image\u{a0} source]]", true, false),
        ] {
            let (actual_positional, _, actual_whole_head_value, name_end) =
                wikidot_tag_argument_shape(source.as_bytes(), 0);

            assert_eq!(actual_positional, positional, "{source:?}");
            assert_eq!(actual_whole_head_value, whole_head_value, "{source:?}");
            assert_eq!(name_end, source.find(' ').unwrap(), "{source:?}");
        }
    }

    #[test]
    fn text_token_ranges_match_pinned_url_and_email_scans() {
        let source = concat!(
            "foo@bar.example@@x ",
            "foo@bar.example$]] ",
            "foo@bar.example--] ",
            "foo@bar.example@<x ",
            "https://e.test/a@@b ",
            "https://e.test/a$]] ",
            "https://e.test/a--] ",
            "https://e.test/a@<b",
        );
        let cursor = TextTokenCursor::new(source);
        let expected = [
            "foo@bar.example@@x",
            "foo@bar.example$",
            "foo@bar.example--",
            "foo@bar.example@",
            "https://e.test/a@@b",
            "https://e.test/a$",
            "https://e.test/a--",
            "https://e.test/a@<b",
        ];
        let mut search_start = 0;
        let ranges = expected
            .iter()
            .map(|token| {
                let start = search_start + source[search_start..].find(token).unwrap();
                let range = start..start + token.len();
                search_start = range.end;
                range
            })
            .collect::<Vec<_>>();

        assert_eq!(cursor.ranges.as_ref(), ranges.as_slice());
    }

    #[test]
    fn failed_email_suffix_scans_stay_linear_before_list_pages() {
        const SEGMENTS: usize = 8_192;
        let source = format!(
            "{}[[module ListPages name=\"live\"]]body[[/module]]",
            "a!".repeat(SEGMENTS),
        );
        let list_pages_start = source.find("[[module ListPages").unwrap();

        EMAIL_SCAN_BYTES_EXAMINED.with(|total| total.set(0));
        let index = super::super::LiteralRegionIndex::new_list_pages_syntax(&source);
        let examined = EMAIL_SCAN_BYTES_EXAMINED.with(Cell::get);

        assert!(!index.contains(list_pages_start));
        assert!(
            examined >= SEGMENTS,
            "instrumentation did not observe the dense suffix"
        );
        assert!(
            examined <= source.len() * 2,
            "email scans examined {examined} bytes for a {}-byte source",
            source.len(),
        );
    }

    #[test]
    fn repeated_brackets_follow_pinned_right_link_precedence() {
        assert_eq!(right_bracket_token(b"]]", 0, 2), (true, 2));
        assert_eq!(right_bracket_token(b"]]]", 0, 3), (false, 3));
        assert_eq!(right_bracket_token(b"]]]]", 0, 4), (false, 3));

        let six = b"]]]]]]";
        assert_eq!(right_bracket_token(six, 0, six.len()), (false, 3));
        assert_eq!(right_bracket_token(six, 3, six.len()), (false, 1));
        assert_eq!(right_bracket_token(six, 4, six.len()), (true, 2));

        let triple_function = b"[[[#if true";
        let (block_start, run_end) = left_block_start_in_run(triple_function, 0);
        assert_eq!(block_start, None);
        assert_eq!(
            rollback_start_in_left_run(triple_function, 0, block_start, run_end,),
            Some(1),
        );
    }

    #[test]
    fn block_close_search_is_ascii_insensitive_and_skips_right_links() {
        let source = "λ [[/code]]] ignored [[/CoDe]] live";
        let expected = source.rfind("[[/CoDe]]").unwrap();

        assert_eq!(
            find_block_close(source, 0, source.len(), "[[/code]]"),
            Some(expected)
        );
    }
}
