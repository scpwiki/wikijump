/*
 * services/render/literal_regions/text_owners.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use super::parser_candidates::{ParserOwnerCandidate, ParserOwnerKind};
#[cfg(test)]
use super::parser_candidates::{ParserOwnerCertainty, select_parser_owner_candidates};
use super::token_boundaries::{TextTokenIndex, right_bracket_token};
use std::ops::Range;

#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
thread_local! {
    static TEXT_OWNER_WORK: Cell<usize> = const { Cell::new(0) };
}

#[inline]
fn record_work(units: usize) {
    #[cfg(test)]
    TEXT_OWNER_WORK.with(|work| work.set(work.get() + units));
    #[cfg(not(test))]
    let _ = units;
}

#[derive(Clone, Copy)]
enum LinkOwnerKind {
    Single,
    Anchor,
    Triple,
}

#[derive(Clone, Copy)]
enum OwnerOpenerKind {
    Link(LinkOwnerKind),
    Color,
}

struct OwnerOpener {
    kind: OwnerOpenerKind,
    event: usize,
    first: Option<usize>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(usize)]
enum Delimiter {
    Whitespace,
    LineBreak,
    ParagraphBreak,
    RightBracket,
    RightLink,
    Pipe,
    Color,
}

const DELIMITER_COUNT: usize = 7;

#[derive(Clone, Copy)]
#[repr(usize)]
enum ScopeMarker {
    Bold,
    Italics,
    Underline,
    Superscript,
    Subscript,
    LeftMonospace,
    RightMonospace,
    Raw,
    LeftRaw,
    RightRaw,
    LeftComment,
    RightComment,
}

const SCOPE_MARKER_COUNT: usize = 12;

struct TokenEvent {
    span: Range<usize>,
    delimiter: Option<Delimiter>,
    marker: Option<ScopeMarker>,
    opener: Option<usize>,
    ambiguous_recursive_start: bool,
    first_single_label: Option<usize>,
    first_triple_label: Option<usize>,
    first_color_terminal: Option<usize>,
    first_line_or_paragraph: Option<usize>,
    first_paragraph: Option<usize>,
    scope_close: Option<usize>,
}

#[derive(Clone)]
struct ScopeCandidate {
    range: Range<usize>,
    close_event: usize,
}

struct TokenIndex {
    events: Vec<TokenEvent>,
    openers: Vec<OwnerOpener>,
    non_whitespace_prefix: Vec<u32>,
}

#[cfg(test)]
pub(super) fn collect_text_owner_candidates(source: &str) -> Vec<ParserOwnerCandidate> {
    let text_tokens = TextTokenIndex::new(source);
    collect_text_owner_candidates_with_text_tokens(source, &text_tokens)
}

pub(super) fn collect_text_owner_candidates_with_text_tokens(
    source: &str,
    text_tokens: &TextTokenIndex,
) -> Vec<ParserOwnerCandidate> {
    let index = TokenIndex::new_with_text_tokens(source, text_tokens);
    let mut link_candidates = vec![None; index.events.len()];
    let mut links = Vec::with_capacity(index.openers.len());

    for opener in &index.openers {
        if !matches!(opener.kind, OwnerOpenerKind::Link(_)) {
            continue;
        }
        if let Some(scope) = index.scan_link(source, opener) {
            links.push(ParserOwnerCandidate::exact(
                scope.range.clone(),
                ParserOwnerKind::TextLink,
                None,
            ));
            link_candidates[opener.event] = Some(scope);
        }
    }

    let scope_candidates = index.collect_scope_candidates(source, link_candidates);
    let (next_scope, next_ambiguous) = index.scope_successors(&scope_candidates);
    let mut colors = Vec::new();
    for opener in &index.openers {
        if !matches!(opener.kind, OwnerOpenerKind::Color) {
            continue;
        }
        if let Some(candidate) =
            index.scan_color(opener, &scope_candidates, &next_scope, &next_ambiguous)
        {
            colors.push(candidate);
        }
    }

    merge_candidate_streams(links, colors)
}

fn merge_candidate_streams(
    links: Vec<ParserOwnerCandidate>,
    colors: Vec<ParserOwnerCandidate>,
) -> Vec<ParserOwnerCandidate> {
    let mut link = links.into_iter().peekable();
    let mut color = colors.into_iter().peekable();
    let mut merged = Vec::with_capacity(link.len() + color.len());
    loop {
        let take_link = match (link.peek(), color.peek()) {
            (Some(left), Some(right)) => {
                (left.range.start, left.range.end) <= (right.range.start, right.range.end)
            }
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (None, None) => break,
        };
        merged.push(if take_link {
            link.next().expect("peeked link candidate exists")
        } else {
            color.next().expect("peeked color candidate exists")
        });
    }
    merged
}

#[cfg(test)]
fn collect_text_owner_ranges(source: &str) -> Vec<Range<usize>> {
    let stream = collect_text_owner_candidates(source);
    let color_descriptors = stream
        .iter()
        .filter(|candidate| {
            candidate.kind == ParserOwnerKind::Color
                && candidate.certainty == ParserOwnerCertainty::ProtectionOnly
        })
        .map(|candidate| {
            ParserOwnerCandidate::exact(
                candidate.range.clone(),
                ParserOwnerKind::Color,
                None,
            )
        })
        .collect();
    select_parser_owner_candidates(&[stream, color_descriptors])
}

impl TokenIndex {
    fn new_with_text_tokens(source: &str, text_tokens: &TextTokenIndex) -> Self {
        let bytes = source.as_bytes();
        let mut index = Self {
            events: Vec::new(),
            openers: Vec::new(),
            non_whitespace_prefix: non_whitespace_prefix(source),
        };
        let mut text_tokens = text_tokens.cursor();
        let mut cursor = 0usize;

        while cursor < bytes.len() {
            record_work(1);
            if let Some(end) = text_tokens.range_end_at(cursor) {
                cursor = end;
                continue;
            }

            match bytes[cursor] {
                b' ' | b'\t' => {
                    let end = scan_horizontal_whitespace(bytes, cursor);
                    index.push_event(
                        cursor..end,
                        Some(Delimiter::Whitespace),
                        None,
                        None,
                        false,
                    );
                    cursor = end;
                }
                b'\n' | b'\r' => {
                    let (end, paragraph) = scan_newlines(bytes, cursor);
                    let delimiter = if paragraph {
                        Delimiter::ParagraphBreak
                    } else {
                        Delimiter::LineBreak
                    };
                    index.push_event(cursor..end, Some(delimiter), None, None, false);
                    cursor = end;
                }
                b'[' => {
                    let token = left_token(bytes, cursor);
                    index.push_event(
                        cursor..token.end,
                        None,
                        token.marker,
                        token.opener,
                        token.ambiguous,
                    );
                    cursor = token.end;
                }
                b']' => {
                    let (_, token_len) = right_bracket_token(bytes, cursor, bytes.len());
                    let delimiter = match token_len {
                        1 => Some(Delimiter::RightBracket),
                        3 => Some(Delimiter::RightLink),
                        _ => None,
                    };
                    if delimiter.is_some() {
                        index.push_event(
                            cursor..cursor + token_len,
                            delimiter,
                            None,
                            None,
                            false,
                        );
                    }
                    cursor += token_len;
                }
                b'|' => {
                    if bytes.get(cursor + 1) == Some(&b'|') {
                        cursor += if matches!(
                            bytes.get(cursor + 2),
                            Some(b'~' | b'>' | b'=')
                        ) {
                            3
                        } else {
                            2
                        };
                    } else {
                        index.push_event(
                            cursor..cursor + 1,
                            Some(Delimiter::Pipe),
                            None,
                            None,
                            false,
                        );
                        cursor += 1;
                    }
                }
                b'#' if bytes.get(cursor + 1) == Some(&b'#') => {
                    index.push_event(
                        cursor..cursor + 2,
                        Some(Delimiter::Color),
                        None,
                        Some(OwnerOpenerKind::Color),
                        false,
                    );
                    cursor += 2;
                }
                b'@' if bytes.get(cursor + 1) == Some(&b'@') => {
                    index.push_event(
                        cursor..cursor + 2,
                        None,
                        Some(ScopeMarker::Raw),
                        None,
                        false,
                    );
                    cursor += 2;
                }
                b'@' if bytes.get(cursor + 1) == Some(&b'<') => {
                    index.push_event(
                        cursor..cursor + 2,
                        None,
                        Some(ScopeMarker::LeftRaw),
                        None,
                        false,
                    );
                    cursor += 2;
                }
                b'>' if bytes.get(cursor + 1) == Some(&b'@') => {
                    index.push_event(
                        cursor..cursor + 2,
                        None,
                        Some(ScopeMarker::RightRaw),
                        None,
                        false,
                    );
                    cursor += 2;
                }
                b'*' if bytes.get(cursor + 1) == Some(&b'*') => {
                    index.push_event(
                        cursor..cursor + 2,
                        None,
                        Some(ScopeMarker::Bold),
                        None,
                        false,
                    );
                    cursor += 2;
                }
                b'/' if bytes.get(cursor + 1) == Some(&b'/') => {
                    index.push_event(
                        cursor..cursor + 2,
                        None,
                        Some(ScopeMarker::Italics),
                        None,
                        false,
                    );
                    cursor += 2;
                }
                b'_' if bytes.get(cursor + 1) == Some(&b'_') => {
                    index.push_event(
                        cursor..cursor + 2,
                        None,
                        Some(ScopeMarker::Underline),
                        None,
                        false,
                    );
                    cursor += 2;
                }
                b'^' if bytes.get(cursor + 1) == Some(&b'^') => {
                    index.push_event(
                        cursor..cursor + 2,
                        None,
                        Some(ScopeMarker::Superscript),
                        None,
                        false,
                    );
                    cursor += 2;
                }
                b',' if bytes.get(cursor + 1) == Some(&b',') => {
                    index.push_event(
                        cursor..cursor + 2,
                        None,
                        Some(ScopeMarker::Subscript),
                        None,
                        false,
                    );
                    cursor += 2;
                }
                b'{' if bytes.get(cursor + 1) == Some(&b'{') => {
                    index.push_event(
                        cursor..cursor + 2,
                        None,
                        Some(ScopeMarker::LeftMonospace),
                        None,
                        false,
                    );
                    cursor += 2;
                }
                b'}' if bytes.get(cursor + 1) == Some(&b'}') => {
                    index.push_event(
                        cursor..cursor + 2,
                        None,
                        Some(ScopeMarker::RightMonospace),
                        None,
                        false,
                    );
                    cursor += 2;
                }
                b'$' if bytes.get(cursor..cursor + 3) == Some(&b"$]]"[..]) => cursor += 3,
                b'-' if bytes.get(cursor..cursor + 3) == Some(&b"--]"[..]) => {
                    index.push_event(
                        cursor..cursor + 3,
                        None,
                        Some(ScopeMarker::RightComment),
                        None,
                        false,
                    );
                    cursor += 3;
                }
                b'-' if bytes.get(cursor + 1) == Some(&b'-') => {
                    cursor += 2;
                    while bytes.get(cursor) == Some(&b'-') {
                        cursor += 1;
                    }
                }
                byte if byte.is_ascii() => cursor += 1,
                _ => {
                    cursor += source[cursor..]
                        .chars()
                        .next()
                        .expect("cursor is before the UTF-8 source end")
                        .len_utf8();
                }
            }
        }
        index.build_successors();
        index
    }

    fn push_event(
        &mut self,
        span: Range<usize>,
        delimiter: Option<Delimiter>,
        marker: Option<ScopeMarker>,
        opener: Option<OwnerOpenerKind>,
        ambiguous_recursive_start: bool,
    ) {
        let event = self.events.len();
        let opener_index = opener.map(|kind| {
            let index = self.openers.len();
            self.openers.push(OwnerOpener {
                kind,
                event,
                first: None,
            });
            index
        });
        self.events.push(TokenEvent {
            span,
            delimiter,
            marker,
            opener: opener_index,
            ambiguous_recursive_start,
            first_single_label: None,
            first_triple_label: None,
            first_color_terminal: None,
            first_line_or_paragraph: None,
            first_paragraph: None,
            scope_close: None,
        });
    }

    fn build_successors(&mut self) {
        let mut delimiters = [None; DELIMITER_COUNT];
        let mut markers = [None; SCOPE_MARKER_COUNT];
        for event in (0..self.events.len()).rev() {
            record_work(1);
            let single_label = self.first_latest(
                &delimiters,
                &[
                    Delimiter::RightBracket,
                    Delimiter::LineBreak,
                    Delimiter::ParagraphBreak,
                ],
            );
            let triple_label = self.first_latest(
                &delimiters,
                &[
                    Delimiter::RightLink,
                    Delimiter::LineBreak,
                    Delimiter::ParagraphBreak,
                ],
            );
            let color_terminal = self.first_latest(
                &delimiters,
                &[Delimiter::Color, Delimiter::ParagraphBreak],
            );
            let line_or_paragraph = self.first_latest(
                &delimiters,
                &[Delimiter::LineBreak, Delimiter::ParagraphBreak],
            );
            let paragraph = delimiters[Delimiter::ParagraphBreak as usize];
            let marker = self.events[event].marker;
            let scope_close =
                marker.and_then(|marker| markers[scope_close_marker(marker) as usize]);
            let opener = self.events[event].opener;
            let opener_first = opener.map(|opener| {
                let kinds: &[Delimiter] = match self.openers[opener].kind {
                    OwnerOpenerKind::Link(
                        LinkOwnerKind::Single | LinkOwnerKind::Anchor,
                    ) => &[
                        Delimiter::Whitespace,
                        Delimiter::RightBracket,
                        Delimiter::LineBreak,
                        Delimiter::ParagraphBreak,
                    ],
                    OwnerOpenerKind::Link(LinkOwnerKind::Triple) => &[
                        Delimiter::Pipe,
                        Delimiter::RightLink,
                        Delimiter::LineBreak,
                        Delimiter::ParagraphBreak,
                    ],
                    OwnerOpenerKind::Color => &[
                        Delimiter::Pipe,
                        Delimiter::LineBreak,
                        Delimiter::ParagraphBreak,
                    ],
                };
                self.first_latest(&delimiters, kinds)
            });

            let current = &mut self.events[event];
            current.first_single_label = single_label;
            current.first_triple_label = triple_label;
            current.first_color_terminal = color_terminal;
            current.first_line_or_paragraph = line_or_paragraph;
            current.first_paragraph = paragraph;
            current.scope_close = scope_close;
            if let Some(opener) = opener {
                self.openers[opener].first = opener_first.flatten();
            }
            if let Some(delimiter) = current.delimiter {
                delimiters[delimiter as usize] = Some(event);
            }
            if let Some(marker) = marker {
                markers[marker as usize] = Some(event);
            }
        }
    }

    fn first_latest(
        &self,
        latest: &[Option<usize>; DELIMITER_COUNT],
        kinds: &[Delimiter],
    ) -> Option<usize> {
        record_work(kinds.len());
        kinds
            .iter()
            .filter_map(|kind| latest[*kind as usize])
            .min_by_key(|event| self.events[*event].span.start)
    }

    fn scan_link(&self, source: &str, opener: &OwnerOpener) -> Option<ScopeCandidate> {
        record_work(1);
        match opener.kind {
            OwnerOpenerKind::Link(LinkOwnerKind::Single) => {
                self.scan_single(source, opener)
            }
            OwnerOpenerKind::Link(LinkOwnerKind::Anchor) => self.scan_anchor(opener),
            OwnerOpenerKind::Link(LinkOwnerKind::Triple) => {
                self.scan_triple(source, opener)
            }
            OwnerOpenerKind::Color => None,
        }
    }

    fn scan_single(&self, source: &str, opener: &OwnerOpener) -> Option<ScopeCandidate> {
        let target = opener.first?;
        if self.events[target].delimiter != Some(Delimiter::Whitespace)
            || !single_link_target_valid(
                &source
                    [self.events[opener.event].span.end..self.events[target].span.start],
            )
        {
            return None;
        }
        let close = self.events[target].first_single_label?;
        (self.events[close].delimiter == Some(Delimiter::RightBracket)).then(|| {
            ScopeCandidate {
                range: self.events[opener.event].span.start..self.events[close].span.end,
                close_event: close,
            }
        })
    }

    fn scan_anchor(&self, opener: &OwnerOpener) -> Option<ScopeCandidate> {
        let target = opener.first?;
        if self.events[target].delimiter != Some(Delimiter::Whitespace) {
            return None;
        }
        let close = self.events[target].first_single_label?;
        (self.events[close].delimiter == Some(Delimiter::RightBracket)).then(|| {
            ScopeCandidate {
                range: self.events[opener.event].span.start..self.events[close].span.end,
                close_event: close,
            }
        })
    }

    fn scan_triple(&self, source: &str, opener: &OwnerOpener) -> Option<ScopeCandidate> {
        let target_end = opener.first?;
        let target =
            self.events[opener.event].span.end..self.events[target_end].span.start;
        if !self.has_non_whitespace(target.clone())
            || !triple_link_target_valid(&source[target])
        {
            return None;
        }
        let close = match self.events[target_end].delimiter {
            Some(Delimiter::RightLink) => target_end,
            Some(Delimiter::Pipe) => {
                let close = self.events[target_end].first_triple_label?;
                if self.events[close].delimiter != Some(Delimiter::RightLink) {
                    return None;
                }
                close
            }
            _ => return None,
        };
        Some(ScopeCandidate {
            range: self.events[opener.event].span.start..self.events[close].span.end,
            close_event: close,
        })
    }

    fn has_non_whitespace(&self, range: Range<usize>) -> bool {
        self.non_whitespace_prefix[range.end] > self.non_whitespace_prefix[range.start]
    }

    fn collect_scope_candidates(
        &self,
        source: &str,
        mut candidates: Vec<Option<ScopeCandidate>>,
    ) -> Vec<Option<ScopeCandidate>> {
        let bytes = source.as_bytes();
        for (event, token) in self.events.iter().enumerate() {
            if candidates[event].is_some() {
                continue;
            }
            let Some(marker) = token.marker else {
                continue;
            };
            let Some(close) = token.scope_close else {
                continue;
            };
            let invalid = match marker {
                ScopeMarker::LeftComment => false,
                ScopeMarker::Raw | ScopeMarker::LeftRaw => {
                    token.first_line_or_paragraph.is_some_and(|invalid| {
                        self.events[invalid].span.start < self.events[close].span.start
                    })
                }
                ScopeMarker::Bold
                | ScopeMarker::Italics
                | ScopeMarker::Underline
                | ScopeMarker::Superscript
                | ScopeMarker::Subscript
                | ScopeMarker::LeftMonospace => {
                    token.first_paragraph.is_some_and(|invalid| {
                        self.events[invalid].span.start < self.events[close].span.start
                    })
                }
                _ => true,
            };
            if invalid
                || formatting_is_padded(
                    bytes,
                    token.span.clone(),
                    self.events[close].span.clone(),
                    marker,
                )
            {
                continue;
            }
            candidates[event] = Some(ScopeCandidate {
                range: token.span.start..self.events[close].span.end,
                close_event: close,
            });
        }
        candidates
    }

    fn scope_successors(
        &self,
        candidates: &[Option<ScopeCandidate>],
    ) -> (Vec<Option<usize>>, Vec<Option<usize>>) {
        let mut next_scope = vec![None; self.events.len()];
        let mut next_ambiguous = vec![None; self.events.len()];
        let mut scope = None;
        let mut ambiguous = None;
        for event in (0..self.events.len()).rev() {
            next_scope[event] = scope;
            next_ambiguous[event] = ambiguous;
            if candidates[event].is_some() {
                scope = Some(event);
            }
            if self.events[event].ambiguous_recursive_start {
                ambiguous = Some(event);
            }
            record_work(1);
        }
        (next_scope, next_ambiguous)
    }

    fn scan_color(
        &self,
        opener: &OwnerOpener,
        scopes: &[Option<ScopeCandidate>],
        next_scope: &[Option<usize>],
        next_ambiguous: &[Option<usize>],
    ) -> Option<ParserOwnerCandidate> {
        record_work(1);
        let pipe = opener.first?;
        if self.events[pipe].delimiter != Some(Delimiter::Pipe) {
            return None;
        }
        let descriptor = self.events[opener.event].span.start..self.events[pipe].span.end;
        let mut cursor = pipe;

        for _ in 0..64 {
            record_work(1);
            let terminal = self.events[cursor].first_color_terminal;
            let scope = next_scope[cursor];
            let ambiguous = next_ambiguous[cursor];
            let terminal_start =
                terminal.map_or(usize::MAX, |event| self.events[event].span.start);
            let scope_start =
                scope.map_or(usize::MAX, |event| self.events[event].span.start);
            let ambiguous_start =
                ambiguous.map_or(usize::MAX, |event| self.events[event].span.start);

            if ambiguous_start < terminal_start && ambiguous_start < scope_start {
                return Some(ParserOwnerCandidate::protection(
                    descriptor,
                    ParserOwnerKind::Color,
                ));
            }
            if scope_start < terminal_start {
                cursor = scopes[scope?].as_ref()?.close_event;
                continue;
            }
            let terminal = terminal?;
            return match self.events[terminal].delimiter {
                Some(Delimiter::Color) => Some(ParserOwnerCandidate::exact(
                    descriptor,
                    ParserOwnerKind::Color,
                    Some(self.events[terminal].span.start),
                )),
                Some(Delimiter::ParagraphBreak) => None,
                _ => unreachable!(),
            };
        }

        Some(ParserOwnerCandidate::protection(
            descriptor,
            ParserOwnerKind::Color,
        ))
    }
}

struct LeftToken {
    opener: Option<OwnerOpenerKind>,
    marker: Option<ScopeMarker>,
    ambiguous: bool,
    end: usize,
}

fn left_token(bytes: &[u8], start: usize) -> LeftToken {
    let remaining = &bytes[start..];
    if remaining.starts_with(b"[!--") {
        return LeftToken {
            opener: None,
            marker: Some(ScopeMarker::LeftComment),
            ambiguous: false,
            end: start + 4,
        };
    }
    if remaining.starts_with(b"[[[[")
        && start.checked_sub(1).and_then(|index| bytes.get(index)) != Some(&b'[')
    {
        return owner_left(start, 1, LinkOwnerKind::Single);
    }
    if remaining.starts_with(b"[[[*") {
        return owner_left(start, 4, LinkOwnerKind::Triple);
    }
    if remaining.starts_with(b"[[[") {
        return owner_left(start, 3, LinkOwnerKind::Triple);
    }
    if remaining.starts_with(b"[[$")
        || remaining.starts_with(b"[[#")
        || remaining.starts_with(b"[[*")
        || remaining.starts_with(b"[[/")
    {
        return LeftToken {
            opener: None,
            marker: None,
            ambiguous: remaining.starts_with(b"[[#") || remaining.starts_with(b"[[*"),
            end: start + 3,
        };
    }
    if remaining.starts_with(b"[[") {
        return LeftToken {
            opener: None,
            marker: None,
            ambiguous: true,
            end: start + 2,
        };
    }
    if remaining.starts_with(b"[#") {
        return owner_left(start, 2, LinkOwnerKind::Anchor);
    }
    if remaining.starts_with(b"[*") {
        return owner_left(start, 2, LinkOwnerKind::Single);
    }
    owner_left(start, 1, LinkOwnerKind::Single)
}

fn owner_left(start: usize, len: usize, kind: LinkOwnerKind) -> LeftToken {
    LeftToken {
        opener: Some(OwnerOpenerKind::Link(kind)),
        marker: None,
        ambiguous: false,
        end: start + len,
    }
}

fn scope_close_marker(marker: ScopeMarker) -> ScopeMarker {
    match marker {
        ScopeMarker::LeftMonospace => ScopeMarker::RightMonospace,
        ScopeMarker::LeftRaw => ScopeMarker::RightRaw,
        ScopeMarker::LeftComment => ScopeMarker::RightComment,
        other => other,
    }
}

fn formatting_is_padded(
    bytes: &[u8],
    open: Range<usize>,
    close: Range<usize>,
    marker: ScopeMarker,
) -> bool {
    if matches!(
        marker,
        ScopeMarker::LeftComment | ScopeMarker::Raw | ScopeMarker::LeftRaw
    ) {
        return false;
    }
    matches!(bytes.get(open.end), Some(b' ' | b'\t'))
        || matches!(bytes.get(close.start.wrapping_sub(1)), Some(b' ' | b'\t'))
}

fn non_whitespace_prefix(source: &str) -> Vec<u32> {
    let mut prefix = vec![0u32; source.len() + 1];
    let mut count = 0u32;
    let mut previous = 0usize;
    for (start, character) in source.char_indices() {
        for value in &mut prefix[previous + 1..=start] {
            *value = count;
        }
        if !character.is_whitespace() {
            count = count.saturating_add(1);
        }
        let end = start + character.len_utf8();
        for value in &mut prefix[start + 1..=end] {
            *value = count;
        }
        previous = end;
    }
    prefix
}

fn scan_horizontal_whitespace(bytes: &[u8], mut cursor: usize) -> usize {
    while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
        cursor += 1;
    }
    cursor
}

fn scan_newlines(bytes: &[u8], mut cursor: usize) -> (usize, bool) {
    let mut count = 0usize;
    while let Some(byte) = bytes.get(cursor) {
        match byte {
            b'\r' if bytes.get(cursor + 1) == Some(&b'\n') => cursor += 2,
            b'\r' | b'\n' => cursor += 1,
            _ => break,
        }
        count += 1;
    }
    (cursor, count > 1)
}

fn single_link_target_valid(target: &str) -> bool {
    target.starts_with('/')
        || [
            "blob:",
            "chrome-extension://",
            "chrome://",
            "content://",
            "dns:",
            "feed:",
            "file://",
            "ftp://",
            "git://",
            "gopher://",
            "http://",
            "https://",
            "irc6://",
            "irc://",
            "ircs://",
            "mailto:",
            "resource://",
            "rtmp://",
            "sftp://",
        ]
        .into_iter()
        .any(|prefix| target.starts_with(prefix))
}

fn triple_link_target_valid(target: &str) -> bool {
    let target = target.trim();
    let Some(interwiki) = target.strip_prefix('!') else {
        return true;
    };
    let Some((prefix, path)) = interwiki.split_once(':') else {
        return false;
    };
    !path.is_empty()
        && [
            "wikipedia",
            "wp",
            "commons",
            "google",
            "duckduckgo",
            "ddg",
            "dictionary",
            "thesaurus",
        ]
        .contains(&prefix)
}

#[cfg(test)]
mod tests {
    use super::{TEXT_OWNER_WORK, collect_text_owner_ranges};

    fn owned(source: &str, needle: &str) -> bool {
        let offset = source.find(needle).expect("needle should occur in source");
        collect_text_owner_ranges(source)
            .iter()
            .any(|range| range.start <= offset && offset < range.end)
    }

    #[test]
    fn links_own_complete_parser_valid_targets_and_labels() {
        for source in [
            "[https://e.test/[[module CountPages tags=\"+target\"]]H[[/module]] label]",
            "[*https://example.test [[module CountPages tags=\"+label\"]]H[[/module]] label]",
            "[#toc[[module CountPages tags=\"+target\"]]H[[/module]] label]",
            "[[[target [[module CountPages tags=\"+target\"]]H[[/module]] suffix]]]",
            "[[[*target|[[module CountPages tags=\"+label\"]]H[[/module]] suffix]]]",
            "[[[!wp:target|[[module CountPages tags=\"+known\"]]H[[/module]] suffix]]]",
        ] {
            assert!(owned(source, "[[module CountPages"), "{source}");
        }
    }

    #[test]
    fn invalid_unclosed_and_unknown_interwiki_links_roll_back() {
        for source in [
            "[relative [[module CountPages]] label]",
            "[https://example.test [[module CountPages]] label",
            "[#toc [[module CountPages]]\nlabel]",
            "[[[ |[[module CountPages]] label]]]",
            "[[[target|[[module CountPages]]\nlabel]]]",
            "[[[!missing:target|[[module CountPages]] label]]]",
        ] {
            assert!(!owned(source, "[[module CountPages"), "{source}");
        }
    }

    #[test]
    fn color_body_skips_exact_recursive_owners() {
        let comment = "##rgb([[module CountPages]])|[!--\n\n--]body##";
        assert!(owned(comment, "[[module CountPages"));

        let bold_rollback = "##rgb([[module CountPages]])|**label ## tail**\n";
        assert!(!owned(bold_rollback, "[[module CountPages"));

        let link_close = "##red|[https://e.test label ##]##bogus [[module CountPages]]X[[/module]]|tail##";
        assert!(!owned(link_close, "[[module CountPages"));
    }

    #[test]
    fn color_descriptor_suppresses_pinned_link_candidates_but_not_body_syntax() {
        let source = "##red[https://e.test | body [[module CountPages]]X[[/module]] ]##";
        assert!(!owned(source, "[[module CountPages"));
    }

    #[test]
    fn delimiter_queries_have_linear_measured_work() {
        fn measured(count: usize) -> usize {
            let source = "[[[".repeat(count) + &" ".repeat(count) + "]]]";
            TEXT_OWNER_WORK.with(|work| work.set(0));
            let _ = collect_text_owner_ranges(&source);
            TEXT_OWNER_WORK.with(Cell::get)
        }

        let small = measured(4_000);
        let large = measured(8_000);
        assert!(large <= small * 3, "small={small}, large={large}");
    }

    use std::cell::Cell;
}
