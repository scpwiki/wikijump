/*
 * services/render/literal_regions/base_candidates/token_stream.rs
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

use super::{DelimiterIdentity, DelimiterKind};
use crate::services::render::literal_regions::token_boundaries::{
    TextTokenIndex, right_bracket_token,
};
use crate::services::render::literal_regions::wikidot::{
    PhysicalLines, physical_line_body,
};
#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
thread_local! {
    static EVENT_SCAN_ADVANCES: Cell<usize> = const { Cell::new(0) };
}

#[inline]
fn record_event_scan_advance() {
    #[cfg(test)]
    EVENT_SCAN_ADVANCES.with(|advances| advances.set(advances.get() + 1));
}

#[derive(Clone, Copy)]
pub(super) struct LineToken {
    pub(super) identity: DelimiterIdentity,
    pub(super) line_end: usize,
}

#[derive(Clone, Copy)]
pub(super) struct DoubleAtToken {
    pub(super) token: LineToken,
}

#[derive(Default)]
pub(super) struct DelimiterIndex {
    pub(super) double_at: Vec<DoubleAtToken>,
    pub(super) left_raw: Vec<LineToken>,
    pub(super) right_raw: Vec<DelimiterIdentity>,
    pub(super) inline_math_open: Vec<LineToken>,
    pub(super) inline_math_close: Vec<DelimiterIdentity>,
    pub(super) comment_open: Vec<DelimiterIdentity>,
    pub(super) comment_close: Vec<DelimiterIdentity>,
}

impl DelimiterIndex {
    #[cfg(test)]
    pub(super) fn new(source: &str) -> Self {
        let text_tokens = TextTokenIndex::new(source);
        Self::new_with_text_tokens(source, &text_tokens)
    }

    pub(super) fn new_with_text_tokens(
        source: &str,
        text_tokens: &TextTokenIndex,
    ) -> Self {
        let mut index = Self::default();
        let bytes = source.as_bytes();
        let mut text_tokens = text_tokens.cursor();
        let mut lines = PhysicalLineEndCursor::new(source);
        let mut cursor = 0usize;

        while cursor < bytes.len() {
            record_event_scan_advance();
            if let Some(end) = text_tokens.range_end_at(cursor) {
                cursor = end;
                continue;
            }
            let step = next_token_step(source, cursor);
            match step.event {
                Some(DelimiterKind::DoubleAt) => index.double_at.push(DoubleAtToken {
                    token: LineToken {
                        identity: DelimiterIdentity {
                            kind: DelimiterKind::DoubleAt,
                            start: cursor,
                        },
                        line_end: lines.body_end_at(cursor),
                    },
                }),
                Some(DelimiterKind::LeftRaw) => {
                    index.left_raw.push(LineToken {
                        identity: DelimiterIdentity {
                            kind: DelimiterKind::LeftRaw,
                            start: cursor,
                        },
                        line_end: lines.body_end_at(cursor),
                    });
                }
                Some(DelimiterKind::RightRaw) => {
                    index.right_raw.push(DelimiterIdentity {
                        kind: DelimiterKind::RightRaw,
                        start: cursor,
                    });
                }
                Some(DelimiterKind::InlineMathOpen) => {
                    index.inline_math_open.push(LineToken {
                        identity: DelimiterIdentity {
                            kind: DelimiterKind::InlineMathOpen,
                            start: cursor,
                        },
                        line_end: lines.body_end_at(cursor),
                    });
                }
                Some(DelimiterKind::InlineMathClose) => {
                    index.inline_math_close.push(DelimiterIdentity {
                        kind: DelimiterKind::InlineMathClose,
                        start: cursor,
                    });
                }
                Some(DelimiterKind::CommentOpen) => {
                    index.comment_open.push(DelimiterIdentity {
                        kind: DelimiterKind::CommentOpen,
                        start: cursor,
                    });
                }
                Some(DelimiterKind::CommentClose) => {
                    index.comment_close.push(DelimiterIdentity {
                        kind: DelimiterKind::CommentClose,
                        start: cursor,
                    });
                }
                Some(_) | None => {}
            }
            cursor = step.end;
        }
        index
    }
}

#[derive(Clone, Copy)]
struct TokenStep {
    event: Option<DelimiterKind>,
    end: usize,
}

impl TokenStep {
    fn other(end: usize) -> Self {
        Self { event: None, end }
    }

    fn delimiter(kind: DelimiterKind, end: usize) -> Self {
        Self {
            event: Some(kind),
            end,
        }
    }
}

fn next_token_step(source: &str, start: usize) -> TokenStep {
    let bytes = source.as_bytes();
    let byte = bytes[start];
    if byte.is_ascii_alphanumeric() {
        return TokenStep::other(scan_ascii_alphanumeric(bytes, start));
    }
    if let Some(end) = scan_variable(bytes, start) {
        return TokenStep::other(end);
    }
    if let Some(end) = scan_newlines(bytes, start) {
        return TokenStep::other(end);
    }
    if matches!(byte, b' ' | b'\t') {
        return TokenStep::other(scan_space(bytes, start));
    }
    if let Some(step) = scan_literal(bytes, start) {
        return step;
    }
    if let Some(end) = scan_repeated_symbol(bytes, start) {
        return TokenStep::other(end);
    }
    TokenStep::other(
        start
            + source[start..]
                .chars()
                .next()
                .expect("cursor is before the UTF-8 source end")
                .len_utf8(),
    )
}

fn scan_literal(bytes: &[u8], start: usize) -> Option<TokenStep> {
    let step = match bytes[start] {
        b'@' if has(bytes, start, b"@@") => {
            TokenStep::delimiter(DelimiterKind::DoubleAt, start + 2)
        }
        b'@' if has(bytes, start, b"@<") => {
            TokenStep::delimiter(DelimiterKind::LeftRaw, start + 2)
        }
        b'>' if has(bytes, start, b">@") => {
            TokenStep::delimiter(DelimiterKind::RightRaw, start + 2)
        }
        b'[' if has(bytes, start, b"[!--") => {
            TokenStep::delimiter(DelimiterKind::CommentOpen, start + 4)
        }
        b'-' if has(bytes, start, b"--]") => {
            TokenStep::delimiter(DelimiterKind::CommentClose, start + 3)
        }
        b'[' if has(bytes, start, b"[[[[")
            && start.checked_sub(1).and_then(|index| bytes.get(index)) != Some(&b'[') =>
        {
            TokenStep::other(start + 1)
        }
        b'[' if has(bytes, start, b"[[[*") => TokenStep::other(start + 4),
        b'[' if has(bytes, start, b"[[[") => TokenStep::other(start + 3),
        b'[' if has(bytes, start, b"[[$") => {
            TokenStep::delimiter(DelimiterKind::InlineMathOpen, start + 3)
        }
        b'[' if has(bytes, start, b"[[#")
            || has(bytes, start, b"[[*")
            || has(bytes, start, b"[[/") =>
        {
            TokenStep::other(start + 3)
        }
        b'[' if has(bytes, start, b"[[")
            || has(bytes, start, b"[#")
            || has(bytes, start, b"[*") =>
        {
            TokenStep::other(start + 2)
        }
        b'[' => TokenStep::other(start + 1),
        b'$' if has(bytes, start, b"$]]") => {
            TokenStep::delimiter(DelimiterKind::InlineMathClose, start + 3)
        }
        b']' => {
            TokenStep::other(start + right_bracket_token(bytes, start, bytes.len()).1)
        }
        b'|' if has(bytes, start, b"||~")
            || has(bytes, start, b"||>")
            || has(bytes, start, b"||=") =>
        {
            TokenStep::other(start + 3)
        }
        _ if is_other_two_byte_literal(bytes, start) => TokenStep::other(start + 2),
        _ => return None,
    };
    Some(step)
}

fn scan_repeated_symbol(bytes: &[u8], start: usize) -> Option<usize> {
    match bytes[start] {
        b'~' => {
            let end = scan_same(bytes, start, b'~');
            match end - start {
                count if count >= 3 => {
                    Some(end + usize::from(matches!(bytes.get(end), Some(b'<' | b'>'))))
                }
                2 => Some(end),
                _ => None,
            }
        }
        b'-' => {
            let end = scan_same(bytes, start, b'-');
            (end - start >= 2).then_some(end)
        }
        b'>' => Some(scan_same(bytes, start, b'>')),
        b'+' => {
            let mut end = start;
            while bytes.get(end) == Some(&b'+') && end - start < 6 {
                end += 1;
            }
            if bytes.get(end) == Some(&b'*') && bytes.get(end + 1) != Some(&b'*') {
                end += 1;
            }
            Some(end)
        }
        _ => None,
    }
}

fn scan_ascii_alphanumeric(bytes: &[u8], mut end: usize) -> usize {
    while bytes.get(end).is_some_and(u8::is_ascii_alphanumeric) {
        end += 1;
    }
    end
}

fn scan_variable(bytes: &[u8], start: usize) -> Option<usize> {
    if !has(bytes, start, b"{$") {
        return None;
    }
    let identifier_start = start + 2;
    let identifier_end = scan_ascii_alphanumeric(bytes, identifier_start);
    (identifier_end > identifier_start && bytes.get(identifier_end) == Some(&b'}'))
        .then_some(identifier_end + 1)
}

fn scan_newlines(bytes: &[u8], start: usize) -> Option<usize> {
    let mut end = start;
    let mut found = false;
    loop {
        match bytes.get(end) {
            Some(b'\r') if bytes.get(end + 1) == Some(&b'\n') => end += 2,
            Some(b'\r' | b'\n') => end += 1,
            _ => break,
        }
        found = true;
    }
    found.then_some(end)
}

fn scan_space(bytes: &[u8], mut end: usize) -> usize {
    while matches!(bytes.get(end), Some(b' ' | b'\t')) {
        end += 1;
    }
    end
}

fn scan_same(bytes: &[u8], mut end: usize, byte: u8) -> usize {
    while bytes.get(end) == Some(&byte) {
        end += 1;
    }
    end
}

fn has(bytes: &[u8], start: usize, literal: &[u8]) -> bool {
    bytes
        .get(start..start.saturating_add(literal.len()))
        .is_some_and(|candidate| candidate == literal)
}

fn is_other_two_byte_literal(bytes: &[u8], start: usize) -> bool {
    const LITERALS: [&[u8]; 14] = [
        b"((", b"))", b"**", b"//", b"__", b"^^", b",,", b"##", b"{{", b"}}", b"||",
        b"<<", b"\\\"", b"\\\\",
    ];
    LITERALS.iter().any(|literal| has(bytes, start, literal))
}

struct PhysicalLineEndCursor<'a> {
    lines: PhysicalLines<'a>,
    line_start: usize,
    line_end: usize,
    body_end: usize,
}

impl<'a> PhysicalLineEndCursor<'a> {
    fn new(source: &'a str) -> Self {
        let mut cursor = Self {
            lines: PhysicalLines::new(source),
            line_start: 0,
            line_end: 0,
            body_end: 0,
        };
        cursor.advance();
        cursor
    }

    fn advance(&mut self) {
        self.line_start = self.line_end;
        if let Some(line) = self.lines.next() {
            self.line_end += line.len();
            self.body_end = self.line_start + physical_line_body(line).len();
        }
    }

    fn body_end_at(&mut self, offset: usize) -> usize {
        while self.line_end <= offset {
            let previous_end = self.line_end;
            self.advance();
            debug_assert!(self.line_end > previous_end);
        }
        self.body_end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ftml::parsing::Token;

    fn indexed_delimiters(source: &str) -> Vec<DelimiterIdentity> {
        let index = DelimiterIndex::new(source);
        let mut delimiters = Vec::new();
        delimiters.extend(index.double_at.iter().map(|token| token.token.identity));
        delimiters.extend(index.left_raw.iter().map(|token| token.identity));
        delimiters.extend(index.right_raw);
        delimiters.extend(index.inline_math_open.iter().map(|token| token.identity));
        delimiters.extend(index.inline_math_close);
        delimiters.extend(index.comment_open);
        delimiters.extend(index.comment_close);
        delimiters.sort_unstable_by_key(|token| (token.start, token.kind.source_order()));
        delimiters
    }

    fn pinned_delimiters(source: &str) -> Vec<DelimiterIdentity> {
        ftml::tokenize(source)
            .tokens()
            .iter()
            .filter_map(|token| {
                let kind = match token.token {
                    Token::Raw => DelimiterKind::DoubleAt,
                    Token::LeftRaw => DelimiterKind::LeftRaw,
                    Token::RightRaw => DelimiterKind::RightRaw,
                    Token::LeftMath => DelimiterKind::InlineMathOpen,
                    Token::RightMath => DelimiterKind::InlineMathClose,
                    Token::LeftComment => DelimiterKind::CommentOpen,
                    Token::RightComment => DelimiterKind::CommentClose,
                    _ => return None,
                };
                Some(DelimiterIdentity {
                    kind,
                    start: token.span.start,
                })
            })
            .collect()
    }

    #[test]
    fn event_index_matches_the_bounded_pinned_token_oracle() {
        for source in [
            "[!----]",
            "[!--[!----]",
            "[[[$x$]]",
            "[[!--x--]",
            "[[[[[[$x$]]",
            "[[[[[!--x--]",
            "@@@<x>@",
            "@@@@@<x>@",
            "@<https://e.test/a>>@",
            "@<https://e.test/a~~~>@",
            "https://e.test/a@@b foo@bar.example@<tail",
        ] {
            assert_eq!(
                indexed_delimiters(source),
                pinned_delimiters(source),
                "{source:?}"
            );
        }
    }

    #[test]
    fn dense_failed_email_prefixes_keep_the_event_scan_monotone() {
        const SEGMENTS: usize = 4_096;
        let source = "a!".repeat(SEGMENTS);

        EVENT_SCAN_ADVANCES.with(|advances| advances.set(0));
        let index = DelimiterIndex::new(&source);
        let advances = EVENT_SCAN_ADVANCES.with(Cell::get);

        assert!(index.double_at.is_empty());
        assert!(index.left_raw.is_empty());
        assert!(index.inline_math_open.is_empty());
        assert!(index.comment_open.is_empty());
        assert!(advances <= source.len(), "{advances} > {}", source.len());
    }

    #[test]
    fn dense_raw_tokens_are_indexed_with_monotone_work() {
        const TOKENS: usize = 20_000;
        let source = "@@".repeat(TOKENS);

        EVENT_SCAN_ADVANCES.with(|advances| advances.set(0));
        let index = DelimiterIndex::new(&source);
        let advances = EVENT_SCAN_ADVANCES.with(Cell::get);

        assert_eq!(index.double_at.len(), TOKENS);
        assert!(advances <= source.len(), "{advances} > {}", source.len());
    }
}
