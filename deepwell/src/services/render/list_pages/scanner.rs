/*
 * services/render/list_pages_scanner.rs
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

use super::literal_regions::{
    ListPagesScannerLiteralIndexes, ListPagesSourceProjection, LiteralRegionCursor,
    LiteralRegionIndex, TextTokenCursor, WikidotArgumentValueKind,
    WikidotTagArgumentScan, WikidotWholeHeadScan, double_quote_ends_wikidot_argument,
    left_block_start_in_run, project_list_pages_typography_in_place, quote_is_escaped,
    right_bracket_token, rollback_start_in_left_run, scan_wikidot_whole_head_value,
    wikidot_right_bracket_token, wikidot_trimmed_name,
};
#[path = "scanner/count_reachability.rs"]
mod count_reachability;

pub(super) use self::count_reachability::CountPagesCloseReachabilityIndex;
#[cfg(test)]
use std::cell::Cell;
use std::ops::Range;

const SPECULATIVE_WORK_LIMIT_MULTIPLIER: usize = 8;

pub(super) fn has_list_pages_module_opening_candidate(source: &str) -> bool {
    first_list_pages_module_opening_candidate(source).is_some()
}

pub(super) fn first_list_pages_module_opening_candidate(source: &str) -> Option<usize> {
    first_module_opening_candidate(source, b"listpages", true)
}

pub(super) fn has_count_pages_module_opening_candidate(source: &str) -> bool {
    first_module_opening_candidate(source, b"countpages", false).is_some()
}

fn first_module_opening_candidate(
    source: &str,
    subname: &[u8],
    allow_legacy_654: bool,
) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut search = 0;
    while search + 1 < bytes.len() {
        let relative_start = bytes[search..].windows(2).position(|pair| pair == b"[[")?;
        let start = search + relative_start;
        search = start + 2;

        let mut cursor = search;
        skip_horizontal_whitespace(bytes, &mut cursor);
        let (name, raw_name_end) = wikidot_trimmed_name(bytes, cursor);
        let Some(name) = name else {
            continue;
        };
        let name = name.strip_suffix(b"_").unwrap_or(name);
        let standard = name.eq_ignore_ascii_case(b"module");
        let legacy = allow_legacy_654 && name.eq_ignore_ascii_case(b"module654");
        if (!standard && !legacy)
            || !bytes
                .get(raw_name_end)
                .is_some_and(|byte| is_wikidot_head_spacing(*byte))
        {
            continue;
        }

        cursor = raw_name_end;
        let has_subname_delimiter = if subname.eq_ignore_ascii_case(b"countpages") {
            skip_count_pages_module_subname_delimiter(bytes, &mut cursor).is_some()
        } else {
            skip_module_subname_delimiter(bytes, &mut cursor).is_some()
        };
        if !has_subname_delimiter {
            continue;
        }
        let subname_start = cursor;
        while bytes
            .get(cursor)
            .is_some_and(|byte| !is_wikidot_head_spacing(*byte) && *byte != b']')
        {
            cursor += 1;
        }
        if bytes[subname_start..cursor].eq_ignore_ascii_case(subname) {
            return Some(start);
        }
    }
    None
}
#[cfg(test)]
const MAX_SINGLE_SCANNER_WORK_MULTIPLIER: usize = 15;
#[cfg(test)]
const MAX_PROJECTED_SCANNER_WORK_MULTIPLIER: usize = 32;

#[cfg(test)]
thread_local! {
    static MODULE_HEAD_SCAN_BYTES: Cell<usize> = const { Cell::new(0) };
    static PROJECTION_OFFSET_ADVANCES: Cell<usize> = const { Cell::new(0) };
}

#[inline]
fn record_module_head_scan_bytes(count: usize) {
    #[cfg(test)]
    MODULE_HEAD_SCAN_BYTES.with(|total| total.set(total.get().saturating_add(count)));
    #[cfg(not(test))]
    let _ = count;
}

#[cfg(test)]
fn take_module_head_scan_bytes() -> usize {
    MODULE_HEAD_SCAN_BYTES.with(|total| total.replace(0))
}

#[inline]
fn record_projection_offset_advances(count: usize) {
    #[cfg(test)]
    PROJECTION_OFFSET_ADVANCES.with(|total| total.set(total.get().saturating_add(count)));
    #[cfg(not(test))]
    let _ = count;
}

#[cfg(test)]
fn take_projection_offset_advances() -> usize {
    PROJECTION_OFFSET_ADVANCES.with(|total| total.replace(0))
}

#[derive(Debug)]
pub(super) struct ListPagesModuleMatch<'a> {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) head: &'a str,
    pub(super) body: &'a str,
    pub(super) original: &'a str,
    pub(super) runtime_safe: bool,
}

#[derive(Debug)]
struct ActiveListPagesModule<'a> {
    start: usize,
    body_start: usize,
    head: &'a str,
    depth: usize,
    runtime_safe: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModuleOpenKind {
    Standard,
    Legacy654,
}

impl ModuleOpenKind {
    fn name(self) -> &'static [u8] {
        match self {
            Self::Standard => b"module",
            Self::Legacy654 => b"module654",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ModuleOpenTag {
    kind: ModuleOpenKind,
    name_end: usize,
    direct_candidate: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModuleOpeningEnd {
    Complete {
        opening_end: usize,
        subname_end: usize,
        runtime_safe: bool,
    },
    Malformed {
        resume: usize,
    },
    Unclosed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModuleEvent {
    Open {
        kind: ModuleOpenKind,
        start: usize,
        subname_start: usize,
        subname_end: usize,
        opening_end: usize,
        direct_candidate: bool,
        runtime_safe: bool,
        projection_guard_start: Option<usize>,
    },
    Close {
        start: usize,
        end: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DirectModuleOpen {
    subname_start: usize,
    subname_end: usize,
    opening_end: usize,
    runtime_safe: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OrderedModuleEvent {
    Open {
        kind: ModuleOpenKind,
        start: usize,
        end: usize,
        direct: Option<DirectModuleOpen>,
        projection_guard_start: Option<usize>,
    },
    Close {
        start: usize,
        end: usize,
    },
}

impl OrderedModuleEvent {
    fn start(self) -> usize {
        match self {
            Self::Open { start, .. } | Self::Close { start, .. } => start,
        }
    }

    fn attach_direct(&mut self, event: ModuleEvent) {
        if let (
            Self::Open { direct, .. },
            ModuleEvent::Open {
                subname_start,
                subname_end,
                opening_end,
                direct_candidate: true,
                runtime_safe,
                ..
            },
        ) = (self, event)
        {
            *direct = Some(DirectModuleOpen {
                subname_start,
                subname_end,
                opening_end,
                runtime_safe,
            });
        }
    }
}

struct ModuleEventScanner<'a> {
    source: &'a str,
    lowercase: &'a str,
    literal_regions: LiteralRegionCursor<'a>,
    text_tokens: TextTokenCursor,
    cursor: usize,
    scanned_bytes: usize,
    speculative_bytes: usize,
    speculative_limit: usize,
    ambiguous_whole_head: bool,
    pending_projection_guard: Option<(usize, usize)>,
}

impl<'a> ModuleEventScanner<'a> {
    fn new(
        source: &'a str,
        lowercase: &'a str,
        literal_regions: &'a LiteralRegionIndex,
    ) -> Self {
        Self {
            source,
            lowercase,
            literal_regions: literal_regions.monotone_cursor(),
            text_tokens: TextTokenCursor::new(source),
            cursor: 0,
            scanned_bytes: 0,
            speculative_bytes: 0,
            speculative_limit: source
                .len()
                .saturating_mul(SPECULATIVE_WORK_LIMIT_MULTIPLIER),
            ambiguous_whole_head: false,
            pending_projection_guard: None,
        }
    }

    fn next(&mut self) -> Option<ModuleEvent> {
        while self.cursor < self.lowercase.len() {
            let search_start = self.cursor;
            let Some(relative_start) = self.lowercase[search_start..].find("[[") else {
                self.advance_to(self.lowercase.len());
                return None;
            };
            let candidate = search_start + relative_start;
            let (block_start, run_end) =
                left_block_start_in_run(self.lowercase.as_bytes(), candidate);
            let Some(start) = block_start else {
                if let Some(marker) =
                    (candidate..run_end.saturating_sub(1)).find(|marker| {
                        self.lowercase.as_bytes().get(*marker..*marker + 3)
                            == Some(&b"[[#"[..])
                            && unresolved_parser_function_prefix(
                                &self.source[*marker + 3..],
                            )
                    })
                    && self.literal_regions.containing_end(marker).is_none()
                {
                    self.ambiguous_whole_head = true;
                    self.advance_to(self.lowercase.len());
                    return None;
                }
                self.advance_to(run_end);
                continue;
            };
            self.advance_to(start + 2);

            if let Some(end) = self.literal_regions.containing_end(start) {
                self.advance_to(end);
                continue;
            }
            if self.lowercase.as_bytes().get(start..start + 3) == Some(&b"[[#"[..])
                && unresolved_parser_function_prefix(&self.source[start + 3..])
            {
                self.ambiguous_whole_head = true;
                self.advance_to(self.lowercase.len());
                return None;
            }
            if let Some(end) = self.module_close_end(start) {
                self.advance_to(end);
                return Some(ModuleEvent::Close { start, end });
            }
            if let Some(tag) = self.module_open_tag(start) {
                let (opening_end, subname_end, runtime_safe) =
                    match self.module_opening_end(tag.name_end) {
                        ModuleOpeningEnd::Complete {
                            opening_end,
                            subname_end,
                            runtime_safe,
                        } => (opening_end, subname_end, runtime_safe),
                        ModuleOpeningEnd::Malformed { resume } => {
                            self.advance_to(resume);
                            continue;
                        }
                        ModuleOpeningEnd::Unclosed => {
                            self.advance_to(self.lowercase.len());
                            return None;
                        }
                    };
                self.advance_to(opening_end + 2);
                let Some((subname_start, subname_end)) =
                    self.module_subname_span(tag.name_end, subname_end)
                else {
                    continue;
                };
                return Some(ModuleEvent::Open {
                    kind: tag.kind,
                    start,
                    subname_start,
                    subname_end,
                    opening_end,
                    direct_candidate: tag.direct_candidate,
                    runtime_safe,
                    projection_guard_start: self.take_projection_guard(start),
                });
            }
            self.advance_past_wikidot_tag(start);
        }
        None
    }

    fn advance_past_wikidot_tag(&mut self, start: usize) {
        let bytes = self.lowercase.as_bytes();
        let mut tag_tokens = self.text_tokens.clone();
        let mut cursor = start + 2;
        let mut quote = None;
        let mut quote_started_owned = false;
        let mut malformed_unquoted_value = false;
        let mut bare_image_link = false;
        let mut trailing_backslashes = 0usize;
        let mut first_rollback_marker = None;
        let argument_scan = WikidotTagArgumentScan::new(bytes, start, &mut tag_tokens);
        let scan_start = start + 2;
        macro_rules! finish_generic_scan {
            ($examined_end:expr, $extra_work:expr, $action:block) => {{
                let work = $examined_end
                    .saturating_sub(scan_start)
                    .saturating_mul(3)
                    .saturating_add($extra_work);
                if self.charge_speculative(work) {
                    $action
                } else {
                    self.advance_to(bytes.len());
                }
                return;
            }};
        }
        if argument_scan.whole_head_value() {
            let scan_start = argument_scan.name_end();
            let scan = scan_wikidot_whole_head_value(
                bytes,
                scan_start,
                bytes.len(),
                &mut tag_tokens,
            );
            let examined_end = match scan {
                WikidotWholeHeadScan::Complete { end, .. } => end,
                WikidotWholeHeadScan::Malformed { resume, .. } => resume,
                WikidotWholeHeadScan::Unclosed { .. } => bytes.len(),
            };
            if !self.charge_speculative(
                examined_end.saturating_sub(scan_start).saturating_mul(2),
            ) {
                self.advance_to(bytes.len());
                return;
            }
            match scan {
                WikidotWholeHeadScan::Complete {
                    end,
                    first_rollback_marker: _,
                } => {
                    self.text_tokens = tag_tokens;
                    self.advance_to(end);
                }
                WikidotWholeHeadScan::Malformed {
                    resume,
                    first_rollback_marker,
                } => self.advance_to(first_rollback_marker.unwrap_or(resume)),
                WikidotWholeHeadScan::Unclosed {
                    first_rollback_marker,
                } => self.advance_to(first_rollback_marker.unwrap_or(bytes.len())),
            }
            return;
        }
        while cursor < bytes.len() {
            if bare_image_link {
                if bytes[cursor] == b'\t' && tag_tokens.contains(cursor) {
                    cursor += 1;
                    continue;
                } else if matches!(bytes[cursor], b' ' | b'\t' | b'\n' | b'\r') {
                    bare_image_link = false;
                } else if bytes[cursor] == b']' {
                    let (right_block, token_len) = wikidot_right_bracket_token(
                        bytes,
                        cursor,
                        bytes.len(),
                        &mut tag_tokens,
                    );
                    if right_block {
                        finish_generic_scan!(cursor + token_len, 0, {
                            self.text_tokens = tag_tokens;
                            self.advance_to(cursor + token_len);
                        });
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
            if bytes[cursor] == b'['
                && bytes.get(cursor + 1) == Some(&b'[')
                && !tag_tokens.contains(cursor)
            {
                let (block_start, run_end) = left_block_start_in_run(bytes, cursor);
                let rollback_marker =
                    rollback_start_in_left_run(bytes, cursor, block_start, run_end);
                if quote.is_some() {
                    first_rollback_marker = first_rollback_marker.or(rollback_marker);
                    trailing_backslashes = 0;
                    cursor = run_end;
                    continue;
                }
                if malformed_unquoted_value {
                    finish_generic_scan!(run_end, 0, {
                        self.advance_to(
                            first_rollback_marker.or(rollback_marker).unwrap_or(run_end),
                        );
                    });
                }
            }
            if matches!(bytes[cursor], b'\n' | b'\r') {
                if quote.is_some() && quote_started_owned && trailing_backslashes == 0 {
                    self.ambiguous_whole_head = true;
                    finish_generic_scan!(bytes.len(), 0, {
                        self.advance_to(bytes.len());
                    });
                }
                if quote.is_some()
                    && !malformed_unquoted_value
                    && trailing_backslashes > 0
                {
                    trailing_backslashes -= 1;
                    cursor = physical_line_resume(bytes, cursor);
                    continue;
                }
                if quote.is_some() || malformed_unquoted_value {
                    let resume = physical_line_resume(bytes, cursor);
                    finish_generic_scan!(resume, 0, {
                        self.advance_to(first_rollback_marker.unwrap_or(resume));
                    });
                }
                trailing_backslashes = 0;
                cursor = physical_line_resume(bytes, cursor);
                continue;
            }
            match (quote, bytes[cursor]) {
                (Some(b'"'), b'"')
                    if !quote_is_escaped(bytes, cursor, &tag_tokens)
                        && !tag_tokens.contains(cursor)
                        && double_quote_ends_scanner_argument(
                            bytes,
                            cursor,
                            &tag_tokens,
                        ) =>
                {
                    quote = None;
                    quote_started_owned = false;
                }
                (Some(b'\''), b'\'')
                    if !quote_is_escaped(bytes, cursor, &tag_tokens)
                        && !tag_tokens.contains(cursor) =>
                {
                    quote = None;
                    quote_started_owned = false;
                }
                (None, b'\'' | b'"')
                    if !quote_is_escaped(bytes, cursor, &tag_tokens)
                        && (!tag_tokens.contains(cursor)
                            || quote_follows_argument_equals(
                                bytes,
                                cursor,
                                start + 2,
                            )) =>
                {
                    quote = Some(bytes[cursor]);
                    quote_started_owned = tag_tokens.contains(cursor);
                }
                (None, b'=') => {
                    let mut value_start = cursor + 1;
                    skip_horizontal_whitespace(bytes, &mut value_start);
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
                (None, b'[') if bytes.get(cursor + 1) == Some(&b'[') => {
                    finish_generic_scan!(cursor + 2, 0, {
                        let rollback = first_rollback_marker.unwrap_or(cursor);
                        self.pending_projection_guard = Some((rollback, start));
                        self.advance_to(rollback);
                    });
                }
                (None, b']') => {
                    let (right_block, token_len) = wikidot_right_bracket_token(
                        bytes,
                        cursor,
                        bytes.len(),
                        &mut tag_tokens,
                    );
                    if right_block {
                        let validation = first_rollback_marker.map(|_| {
                            validate_generic_head_arguments(
                                bytes,
                                cursor,
                                argument_scan,
                                &self.text_tokens,
                            )
                        });
                        let validation_work = validation
                            .as_ref()
                            .map_or(0, |validation| validation.inspected);
                        finish_generic_scan!(cursor + token_len, validation_work, {
                            if let Some(rollback) = first_rollback_marker
                                && (malformed_unquoted_value
                                    || validation
                                        .is_some_and(|validation| !validation.valid))
                            {
                                self.pending_projection_guard = Some((rollback, start));
                                self.advance_to(rollback);
                            } else {
                                self.text_tokens = tag_tokens;
                                self.advance_to(cursor + token_len);
                            }
                        });
                    }
                    if token_len == 3
                        && bytes.get(start + 2) != Some(&b'/')
                        && !argument_scan.in_positional_value(cursor)
                    {
                        let resume = next_physical_line_resume(bytes, cursor);
                        finish_generic_scan!(resume, 0, {
                            self.advance_to(resume);
                        });
                    }
                    cursor += token_len;
                    continue;
                }
                _ => {}
            }
            if bytes[cursor] == b'\\' {
                trailing_backslashes += 1;
            } else {
                trailing_backslashes = 0;
            }
            cursor += 1;
        }
        finish_generic_scan!(bytes.len(), 0, {
            self.advance_to(first_rollback_marker.unwrap_or(bytes.len()));
        });
    }

    fn module_close_end(&mut self, start: usize) -> Option<usize> {
        let bytes = self.lowercase.as_bytes();
        if bytes.get(start + 2) != Some(&b'/') {
            return None;
        }
        let mut cursor = start + 3;
        skip_horizontal_whitespace(bytes, &mut cursor);
        if bytes.get(cursor..cursor + 2) == Some(&b"[["[..])
            && left_block_start_in_run(bytes, cursor).0 == Some(cursor)
        {
            cursor += 2;
            skip_horizontal_whitespace(bytes, &mut cursor);
        }
        let (name, raw_name_end) = wikidot_trimmed_name(bytes, cursor);
        let name = name?;
        let name = name.strip_suffix(b"_").unwrap_or(name);
        let _kind = [ModuleOpenKind::Legacy654, ModuleOpenKind::Standard]
            .into_iter()
            .find(|kind| name == kind.name())?;
        cursor = raw_name_end;
        skip_module_close_spacing(bytes, &mut cursor);
        if bytes.get(cursor) != Some(&b']') {
            return None;
        }
        let (right_block, token_len) = wikidot_right_bracket_token(
            bytes,
            cursor,
            bytes.len(),
            &mut self.text_tokens,
        );
        right_block.then_some(cursor + token_len)
    }

    fn module_open_tag(&self, start: usize) -> Option<ModuleOpenTag> {
        let bytes = self.lowercase.as_bytes();
        let mut cursor = start + 2;
        skip_horizontal_whitespace(bytes, &mut cursor);
        let (name, raw_name_end) = wikidot_trimmed_name(bytes, cursor);
        let name = name?;
        [ModuleOpenKind::Legacy654, ModuleOpenKind::Standard]
            .into_iter()
            .find_map(|kind| {
                let delimiter = bytes.get(raw_name_end)?;
                (name == kind.name() && is_wikidot_head_spacing(*delimiter)).then_some(
                    ModuleOpenTag {
                        kind,
                        name_end: raw_name_end,
                        direct_candidate: true,
                    },
                )
            })
    }

    fn module_subname_span(
        &self,
        module_name_end: usize,
        raw_subname_end: usize,
    ) -> Option<(usize, usize)> {
        let bytes = self.lowercase.as_bytes();
        let mut subname_start = module_name_end;
        skip_module_subname_delimiter(bytes, &mut subname_start)?;
        if subname_start >= raw_subname_end
            || is_wikidot_head_spacing(bytes[subname_start])
        {
            return None;
        }
        let (subname_start, subname_end) =
            trimmed_utf8_span(self.lowercase, subname_start, raw_subname_end);
        (subname_start < subname_end).then_some((subname_start, subname_end))
    }

    fn module_opening_end(&mut self, module_name_end: usize) -> ModuleOpeningEnd {
        let source = self.source;
        let bytes = self.lowercase.as_bytes();
        macro_rules! finish_head_scan {
            ($end:expr, $result:expr) => {{
                let examined = $end.saturating_sub(module_name_end);
                record_module_head_scan_bytes(examined);
                if !self.charge_speculative(examined.saturating_mul(3)) {
                    return ModuleOpeningEnd::Unclosed;
                }
                return $result;
            }};
        }
        // Opening-head recognition is speculative. Keep tokenizer ownership local
        // until a complete head commits; malformed rollback must leave the outer
        // scanner's monotone cursor at its prior position.
        let mut head_tokens = self.text_tokens.clone();
        let mut subname_end = module_name_end;
        if skip_module_subname_delimiter(bytes, &mut subname_end).is_some() {
            let mut lookahead_tokens = head_tokens.clone();
            subname_end = module_subname_end(bytes, subname_end, &mut lookahead_tokens);
        }
        let mut subname_start = module_name_end;
        let has_subname_delimiter =
            skip_module_subname_delimiter(bytes, &mut subname_start).is_some();
        let (trimmed_start, trimmed_end) =
            trimmed_utf8_span(self.lowercase, subname_start, subname_end);
        let list_pages_compatibility = has_subname_delimiter
            && self.lowercase[trimmed_start..trimmed_end]
                .eq_ignore_ascii_case("listpages");
        let mut cursor = module_name_end;
        let mut quote = None;
        let mut trailing_backslashes = 0usize;
        let mut first_rollback_marker = None;
        let mut definite_invalid_boundary = false;
        while cursor + 1 < bytes.len() {
            if bytes[cursor] == b'['
                && bytes.get(cursor + 1) == Some(&b'[')
                && !head_tokens.contains(cursor)
            {
                let (block_start, run_end) = left_block_start_in_run(bytes, cursor);
                let rollback_marker =
                    rollback_start_in_left_run(bytes, cursor, block_start, run_end);
                if quote.is_some() || cursor < subname_end {
                    first_rollback_marker = first_rollback_marker.or(rollback_marker);
                    trailing_backslashes = 0;
                    cursor = run_end;
                    continue;
                }
                let resume = rollback_marker.unwrap_or(run_end);
                if list_pages_compatibility && !definite_invalid_boundary {
                    // An unquoted `[[` makes the pinned head malformed whether
                    // the run owns a block token or a competing link token.
                    // Runtime bare-value parsing can still consume that head,
                    // so it is unsafe to choose either structure.
                    self.ambiguous_whole_head = true;
                    finish_head_scan!(run_end, ModuleOpeningEnd::Malformed { resume });
                }
                finish_head_scan!(
                    run_end,
                    ModuleOpeningEnd::Malformed {
                        resume: first_rollback_marker.unwrap_or(resume),
                    }
                );
            }
            if quote.is_some() && matches!(bytes[cursor], b'\n' | b'\r') {
                if trailing_backslashes > 0 {
                    trailing_backslashes -= 1;
                    cursor = physical_line_resume(bytes, cursor);
                    continue;
                }
                let resume = physical_line_resume(bytes, cursor);
                if list_pages_compatibility {
                    // An unescaped physical newline terminates the pinned
                    // quoted head but can remain part of the runtime regex's
                    // bare-value alternative. Do not evaluate either reading.
                    self.ambiguous_whole_head = true;
                    finish_head_scan!(resume, ModuleOpeningEnd::Malformed { resume });
                }
                finish_head_scan!(
                    resume,
                    ModuleOpeningEnd::Malformed {
                        resume: first_rollback_marker.unwrap_or(resume),
                    }
                );
            }
            match (quote, bytes[cursor]) {
                (Some(b'"'), b'"')
                    if !quote_is_escaped(bytes, cursor, &head_tokens)
                        && !head_tokens.contains(cursor)
                        && double_quote_ends_scanner_argument(
                            bytes,
                            cursor,
                            &head_tokens,
                        ) =>
                {
                    quote = None;
                }
                (Some(b'\''), b'\'')
                    if !quote_is_escaped(bytes, cursor, &head_tokens)
                        && !head_tokens.contains(cursor) =>
                {
                    quote = None;
                }
                (None, b'\'' | b'"')
                    if !quote_is_escaped(bytes, cursor, &head_tokens)
                        && (!head_tokens.contains(cursor)
                            || quote_follows_argument_equals(
                                bytes,
                                cursor,
                                module_name_end,
                            )) =>
                {
                    quote = Some(bytes[cursor]);
                }
                (None, b']') => {
                    let (right_block, token_len) = wikidot_right_bracket_token(
                        bytes,
                        cursor,
                        bytes.len(),
                        &mut head_tokens,
                    );
                    if right_block {
                        let raw_head = &source[subname_end..cursor];
                        let mut validation =
                            validate_module_head(raw_head, list_pages_compatibility);
                        let runtime_recognized = list_pages_compatibility
                            && runtime_regex_recognizes_entire_head(raw_head);
                        if !self.charge_speculative(raw_head.len().saturating_mul(3)) {
                            record_module_head_scan_bytes(
                                cursor.saturating_sub(module_name_end),
                            );
                            return ModuleOpeningEnd::Unclosed;
                        }
                        if (first_rollback_marker.is_some() || runtime_recognized)
                            && validation == ModuleHeadValidation::DefiniteInvalid
                        {
                            validation = if list_pages_compatibility {
                                ModuleHeadValidation::AmbiguousFailClosed
                            } else {
                                finish_head_scan!(
                                    cursor + token_len,
                                    ModuleOpeningEnd::Malformed {
                                        resume: first_rollback_marker
                                            .expect("suppressed rollback marker exists"),
                                    }
                                );
                            };
                        }
                        let runtime_safe = match validation {
                            ModuleHeadValidation::DefiniteInvalid => {
                                finish_head_scan!(
                                    cursor + token_len,
                                    ModuleOpeningEnd::Malformed {
                                        resume: cursor + token_len,
                                    }
                                );
                            }
                            ModuleHeadValidation::AmbiguousFailClosed => {
                                self.ambiguous_whole_head = true;
                                finish_head_scan!(
                                    cursor + token_len,
                                    ModuleOpeningEnd::Malformed {
                                        resume: cursor + token_len,
                                    }
                                );
                            }
                            ModuleHeadValidation::ValidRuntimeBoundaryDivergence => {
                                if first_rollback_marker.is_none() {
                                    self.ambiguous_whole_head = true;
                                    finish_head_scan!(
                                        cursor + token_len,
                                        ModuleOpeningEnd::Malformed {
                                            resume: cursor + token_len,
                                        }
                                    );
                                }
                                false
                            }
                            ModuleHeadValidation::ValidRuntimeUnsafe => false,
                            ModuleHeadValidation::RuntimeSafe => true,
                        };
                        self.text_tokens = head_tokens;
                        finish_head_scan!(
                            cursor + token_len,
                            ModuleOpeningEnd::Complete {
                                opening_end: cursor,
                                subname_end,
                                runtime_safe,
                            }
                        );
                    }
                    if token_len == 3 {
                        let resume = next_physical_line_resume(bytes, cursor);
                        finish_head_scan!(
                            resume,
                            ModuleOpeningEnd::Malformed {
                                resume: first_rollback_marker.unwrap_or(resume),
                            }
                        );
                    }
                    definite_invalid_boundary |= (cursor > 0
                        && bytes[cursor - 1] == b'$')
                        || (cursor >= 2
                            && bytes.get(cursor - 2..cursor) == Some(&b"--"[..]));
                    cursor += token_len;
                    continue;
                }
                _ => {}
            }
            if bytes[cursor] == b'\\' {
                trailing_backslashes += 1;
            } else {
                trailing_backslashes = 0;
            }
            cursor += 1;
        }
        let examined = bytes.len().saturating_sub(module_name_end);
        record_module_head_scan_bytes(examined);
        if !self.charge_speculative(examined.saturating_mul(3)) {
            return ModuleOpeningEnd::Unclosed;
        }
        if let Some(resume) = first_rollback_marker {
            self.ambiguous_whole_head |=
                list_pages_compatibility && !definite_invalid_boundary;
            ModuleOpeningEnd::Malformed { resume }
        } else {
            ModuleOpeningEnd::Unclosed
        }
    }

    fn advance_to(&mut self, end: usize) {
        debug_assert!(end >= self.cursor);
        self.scanned_bytes += end - self.cursor;
        self.cursor = end;
    }

    fn take_projection_guard(&mut self, candidate: usize) -> Option<usize> {
        match self.pending_projection_guard {
            Some((expected, guard)) if expected == candidate => {
                self.pending_projection_guard = None;
                Some(guard)
            }
            Some((expected, _)) if expected <= candidate => {
                self.pending_projection_guard = None;
                None
            }
            _ => None,
        }
    }

    fn charge_speculative(&mut self, count: usize) -> bool {
        self.speculative_bytes = self.speculative_bytes.saturating_add(count);
        if self.speculative_bytes > self.speculative_limit {
            self.ambiguous_whole_head = true;
            false
        } else {
            true
        }
    }
}

#[derive(Clone, Copy)]
struct GenericHeadValidation {
    valid: bool,
    inspected: usize,
}

fn validate_generic_head_arguments(
    bytes: &[u8],
    head_end: usize,
    argument_scan: WikidotTagArgumentScan,
    text_tokens: &TextTokenCursor,
) -> GenericHeadValidation {
    let mut text_tokens = text_tokens.clone();
    let mut cursor = argument_scan.name_end();
    let baseline_work = head_end.saturating_sub(cursor);
    let mut lookahead_work = 0usize;
    macro_rules! finish_validation {
        ($valid:expr) => {
            return GenericHeadValidation {
                valid: $valid,
                inspected: baseline_work.saturating_add(lookahead_work),
            }
        };
    }

    while cursor < head_end && argument_scan.in_positional_value(cursor) {
        cursor += 1;
    }

    loop {
        skip_module_argument_spacing(bytes, &mut cursor);
        if cursor == head_end {
            finish_validation!(true);
        }
        if cursor > head_end {
            finish_validation!(false);
        }

        let key_start = cursor;
        while cursor < head_end
            && (bytes[cursor].is_ascii_alphanumeric()
                || matches!(bytes[cursor], b'_' | b'-'))
        {
            cursor += 1;
        }
        if cursor == key_start {
            finish_validation!(false);
        }
        skip_horizontal_whitespace(bytes, &mut cursor);
        if cursor >= head_end || bytes[cursor] != b'=' {
            finish_validation!(false);
        }
        let equals = cursor;
        cursor += 1;
        skip_horizontal_whitespace(bytes, &mut cursor);
        if cursor >= head_end {
            finish_validation!(false);
        }

        match bytes[cursor] {
            b'"' => {
                cursor += 1;
                let mut closed = false;
                let mut trailing_backslashes = 0usize;
                while cursor < head_end {
                    if bytes[cursor] == b'"'
                        && !quote_is_escaped(bytes, cursor, &text_tokens)
                        && !text_tokens.contains(cursor)
                    {
                        let (ends, inspected) = pinned_double_quote_ends_generic_argument(
                            bytes,
                            cursor,
                            head_end,
                            &text_tokens,
                        );
                        lookahead_work = lookahead_work.saturating_add(inspected);
                        if ends {
                            cursor += 1;
                            closed = true;
                            break;
                        }
                    }
                    if matches!(bytes[cursor], b'\n' | b'\r') {
                        if trailing_backslashes > 0 {
                            trailing_backslashes -= 1;
                            cursor = physical_line_resume(bytes, cursor);
                            continue;
                        }
                        finish_validation!(false);
                    }
                    if bytes[cursor] == b'\\' {
                        trailing_backslashes += 1;
                    } else {
                        trailing_backslashes = 0;
                    }
                    cursor += 1;
                }
                if !closed {
                    finish_validation!(false);
                }
            }
            _ if argument_scan.classify(bytes, equals, cursor)
                == WikidotArgumentValueKind::BareImageLink =>
            {
                let value_start = cursor;
                while cursor < head_end && !is_wikidot_head_spacing(bytes[cursor]) {
                    cursor += 1;
                }
                if cursor == value_start {
                    finish_validation!(false);
                }
            }
            _ => finish_validation!(false),
        }

        if cursor < head_end && !is_wikidot_head_spacing(bytes[cursor]) {
            finish_validation!(false);
        }
    }
}

fn pinned_double_quote_ends_generic_argument(
    bytes: &[u8],
    quote: usize,
    head_end: usize,
    text_tokens: &TextTokenCursor,
) -> (bool, usize) {
    let mut text_tokens = text_tokens.clone();
    let start = quote + 1;
    let mut cursor = start;
    if cursor >= head_end {
        return (true, cursor.saturating_sub(start));
    }
    if matches!(bytes[cursor], b'\n' | b'\r') {
        return (true, 1);
    }
    if bytes[cursor] == b']'
        && wikidot_right_bracket_token(bytes, cursor, bytes.len(), &mut text_tokens).0
    {
        return (true, 1);
    }
    if !matches!(bytes[cursor], b' ' | b'\t') {
        return (false, 1);
    }

    let mut saw_key = false;
    loop {
        while cursor < head_end && matches!(bytes[cursor], b' ' | b'\t') {
            cursor += 1;
        }
        if cursor >= head_end {
            return (true, cursor.saturating_sub(start));
        }
        if matches!(bytes[cursor], b'\n' | b'\r') {
            return (false, cursor + 1 - start);
        }
        if bytes[cursor] == b']'
            && wikidot_right_bracket_token(bytes, cursor, bytes.len(), &mut text_tokens).0
        {
            return (true, cursor + 1 - start);
        }
        if bytes[cursor] == b'=' && !text_tokens.contains(cursor) {
            return (saw_key, cursor + 1 - start);
        }

        let key_start = cursor;
        while cursor < head_end
            && (bytes[cursor].is_ascii_alphanumeric()
                || matches!(bytes[cursor], b'_' | b'-'))
        {
            cursor += 1;
        }
        if cursor == key_start || text_tokens.contains(key_start) {
            return (false, cursor.saturating_add(1).saturating_sub(start));
        }
        saw_key = true;
    }
}

fn is_horizontal_whitespace(byte: &u8) -> bool {
    matches!(byte, b' ' | b'\t')
}

fn is_wikidot_head_spacing(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
}

fn trimmed_utf8_span(source: &str, start: usize, end: usize) -> (usize, usize) {
    let raw = &source[start..end];
    let trimmed_start = start + raw.len() - raw.trim_start().len();
    let trimmed_end = start + raw.trim_end().len();
    (trimmed_start.min(trimmed_end), trimmed_end)
}

fn module_subname_end(
    bytes: &[u8],
    mut cursor: usize,
    text_tokens: &mut TextTokenCursor,
) -> usize {
    while cursor < bytes.len() && !is_wikidot_head_spacing(bytes[cursor]) {
        if bytes[cursor] == b']' {
            let (right_block, token_len) =
                wikidot_right_bracket_token(bytes, cursor, bytes.len(), text_tokens);
            if right_block {
                break;
            }
            cursor += token_len;
        } else {
            cursor += 1;
        }
    }
    cursor
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModuleHeadValidation {
    DefiniteInvalid,
    AmbiguousFailClosed,
    ValidRuntimeBoundaryDivergence,
    ValidRuntimeUnsafe,
    RuntimeSafe,
}

fn validate_module_head(
    source: &str,
    list_pages_compatibility: bool,
) -> ModuleHeadValidation {
    let normalized = normalize_module_head(source);
    let mut runtime_safe = normalized == source;
    let mut projected = normalized.into_bytes();
    if project_list_pages_typography_in_place(&mut projected) {
        runtime_safe = false;
    }
    let source = String::from_utf8(projected)
        .expect("module head typography projection preserves UTF-8");
    let bytes = source.as_bytes();
    let mut text_tokens = TextTokenCursor::new(&source);
    let mut cursor = 0usize;
    let mut runtime_boundary_divergence = false;

    loop {
        skip_module_argument_spacing(bytes, &mut cursor);
        if cursor == bytes.len() {
            return if runtime_boundary_divergence {
                ModuleHeadValidation::ValidRuntimeBoundaryDivergence
            } else if list_pages_compatibility && !runtime_safe {
                ModuleHeadValidation::ValidRuntimeUnsafe
            } else {
                ModuleHeadValidation::RuntimeSafe
            };
        }

        let key_start = cursor;
        let mut syntax_crossing_token_end = None;
        while cursor < bytes.len() {
            if bytes[cursor] == b'='
                || (bytes[cursor] == b'!' && bytes.get(cursor + 1) == Some(&b'='))
                || is_module_argument_spacing(bytes[cursor])
            {
                break;
            }
            if let Some(end) = text_tokens.range_end_at(cursor) {
                runtime_safe = false;
                if bytes[cursor..end].contains(&b'=') {
                    syntax_crossing_token_end = Some(end);
                } else {
                    cursor = end;
                    continue;
                }
            }
            if bytes[cursor..].starts_with(b"{$") {
                let Some(relative_end) = source[cursor + 2..].find('}') else {
                    return ModuleHeadValidation::DefiniteInvalid;
                };
                let end = cursor + 2 + relative_end + 1;
                if cursor + 2 == end - 1
                    || !bytes[cursor + 2..end - 1]
                        .iter()
                        .all(u8::is_ascii_alphanumeric)
                {
                    return ModuleHeadValidation::DefiniteInvalid;
                }
                cursor = end;
                runtime_safe = false;
                continue;
            }
            if bytes[cursor..].starts_with(b"[!--") {
                cursor += 4;
                runtime_safe = false;
                continue;
            }
            if bytes[cursor..].starts_with(b"--]") {
                cursor += 3;
                runtime_safe = false;
                continue;
            }
            if bytes[cursor] == b'-' {
                while bytes.get(cursor) == Some(&b'-') {
                    cursor += 1;
                }
                continue;
            }
            if !bytes[cursor].is_ascii() {
                return ModuleHeadValidation::DefiniteInvalid;
            }
            if !bytes[cursor].is_ascii_alphanumeric()
                && !matches!(bytes[cursor], b'_' | b'-')
            {
                return ModuleHeadValidation::DefiniteInvalid;
            }
            cursor += 1;
        }
        if cursor == key_start {
            return ModuleHeadValidation::DefiniteInvalid;
        }
        let runtime_key_supported =
            runtime_list_pages_key_is_supported(&source[key_start..cursor]);
        if list_pages_compatibility && !runtime_key_supported {
            runtime_safe = false;
        }
        skip_horizontal_whitespace(bytes, &mut cursor);

        if bytes.get(cursor) == Some(&b'!') {
            if !list_pages_compatibility
                || !runtime_key_supported
                || bytes.get(key_start) != Some(&b'_')
            {
                return ModuleHeadValidation::DefiniteInvalid;
            }
            cursor += 1;
        }
        if bytes.get(cursor) != Some(&b'=') {
            return ModuleHeadValidation::DefiniteInvalid;
        }
        cursor += 1;
        skip_horizontal_whitespace(bytes, &mut cursor);
        if cursor == bytes.len() || is_module_argument_spacing(bytes[cursor]) {
            return ModuleHeadValidation::DefiniteInvalid;
        }

        let quote = bytes[cursor];
        let quote_owned = matches!(quote, b'\'' | b'"')
            && (syntax_crossing_token_end.is_some_and(|end| cursor < end)
                || text_tokens.contains(cursor));
        if matches!(quote, b'\'' | b'"') {
            let quote_crosses_syntax_token =
                quote_owned && syntax_crossing_token_end.is_some_and(|end| cursor < end);
            runtime_safe &= !quote_owned;
            if quote == b'\'' && (!list_pages_compatibility || !runtime_key_supported) {
                return ModuleHeadValidation::DefiniteInvalid;
            }
            cursor += 1;
            let mut closed = false;
            while cursor < bytes.len() {
                if quote == b'"' && bytes[cursor] == b'\\' {
                    runtime_safe = false;
                }
                if bytes[cursor] == quote {
                    if syntax_crossing_token_end.is_some_and(|end| cursor < end)
                        || text_tokens.contains(cursor)
                    {
                        runtime_safe = false;
                        cursor += 1;
                        continue;
                    }
                    if quote_is_escaped(bytes, cursor, &text_tokens)
                        || (quote == b'"'
                            && !double_quote_ends_scanner_argument(
                                bytes,
                                cursor,
                                &text_tokens,
                            ))
                    {
                        runtime_safe = false;
                        cursor += 1;
                        continue;
                    }
                    cursor += 1;
                    closed = true;
                    runtime_boundary_divergence |= quote_crosses_syntax_token;
                    break;
                }
                cursor += 1;
            }
            if !closed {
                return ModuleHeadValidation::DefiniteInvalid;
            }
        } else if list_pages_compatibility && runtime_key_supported {
            runtime_safe &= !quote_owned;
            let value_start = cursor;
            while cursor < bytes.len() {
                if is_module_argument_spacing(bytes[cursor]) {
                    break;
                }
                if bytes[cursor] == b']' {
                    return ModuleHeadValidation::DefiniteInvalid;
                }
                if matches!(bytes[cursor], b'\'' | b'"') {
                    runtime_safe = false;
                }
                let character = source[cursor..]
                    .chars()
                    .next()
                    .expect("cursor is before the module head end");
                if character.is_whitespace() {
                    return ModuleHeadValidation::DefiniteInvalid;
                }
                cursor += character.len_utf8();
            }
            if cursor == value_start {
                return ModuleHeadValidation::DefiniteInvalid;
            }
        } else {
            return ModuleHeadValidation::DefiniteInvalid;
        }

        if cursor < bytes.len() && !is_module_argument_spacing(bytes[cursor]) {
            return ModuleHeadValidation::DefiniteInvalid;
        }
    }
}

fn runtime_list_pages_key_is_supported(key: &str) -> bool {
    let mut bytes = key.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

pub(super) fn runtime_regex_recognizes_entire_head(source: &str) -> bool {
    super::service::list_pages_runtime_regex_recognizes_entire_head(source)
}

fn unresolved_parser_function_prefix(source: &str) -> bool {
    ["ifexpr", "if", "expr"].into_iter().any(|name| {
        source
            .get(..name.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(name))
            && source[name.len()..]
                .chars()
                .next()
                .is_some_and(char::is_whitespace)
    })
}

pub(super) fn list_pages_runtime_head_is_safe(head: &str) -> bool {
    validate_module_head(head, true) == ModuleHeadValidation::RuntimeSafe
}

fn normalize_module_head(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut normalized = String::with_capacity(source.len());
    let mut cursor = 0usize;
    let mut line_leading = false;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\0' => {
                normalized.push(' ');
                cursor += 1;
                line_leading = false;
            }
            b'\n' | b'\r' => {
                let continued = normalized.ends_with('\\');
                if continued {
                    normalized.pop();
                } else {
                    normalized.push('\n');
                }
                cursor = physical_line_resume(bytes, cursor);
                line_leading = true;
            }
            b'\t' => {
                normalized.push_str("    ");
                cursor += 1;
                line_leading = false;
            }
            _ => {
                let character = source[cursor..]
                    .chars()
                    .next()
                    .expect("cursor is before the module head end");
                if line_leading && matches!(character, '\u{00a0}' | '\u{2007}') {
                    normalized.push(' ');
                } else {
                    normalized.push(character);
                    line_leading = false;
                }
                cursor += character.len_utf8();
            }
        }
    }
    normalized
}

fn is_module_argument_spacing(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
}

fn skip_module_argument_spacing(bytes: &[u8], cursor: &mut usize) {
    while bytes
        .get(*cursor)
        .is_some_and(|byte| is_module_argument_spacing(*byte))
    {
        *cursor = if matches!(bytes[*cursor], b'\n' | b'\r') {
            physical_line_resume(bytes, *cursor)
        } else {
            *cursor + 1
        };
    }
}

fn skip_horizontal_whitespace(bytes: &[u8], cursor: &mut usize) {
    while bytes.get(*cursor).is_some_and(is_horizontal_whitespace) {
        *cursor += 1;
    }
}

fn skip_module_subname_delimiter(bytes: &[u8], cursor: &mut usize) -> Option<()> {
    if bytes.get(*cursor).is_some_and(is_horizontal_whitespace) {
        skip_horizontal_whitespace(bytes, cursor);
        return Some(());
    }
    if matches!(bytes.get(*cursor), Some(b'\n' | b'\r')) {
        skip_physical_line_endings(bytes, cursor);
        skip_horizontal_whitespace(bytes, cursor);
        return Some(());
    }
    None
}

fn skip_count_pages_module_subname_delimiter(
    bytes: &[u8],
    cursor: &mut usize,
) -> Option<()> {
    if !bytes
        .get(*cursor)
        .is_some_and(|byte| is_wikidot_head_spacing(*byte))
    {
        return None;
    }
    skip_module_argument_spacing(bytes, cursor);
    Some(())
}

fn skip_module_close_spacing(bytes: &[u8], cursor: &mut usize) {
    if bytes.get(*cursor).is_some_and(is_horizontal_whitespace) {
        skip_horizontal_whitespace(bytes, cursor);
    } else if matches!(bytes.get(*cursor), Some(b'\n' | b'\r')) {
        skip_physical_line_endings(bytes, cursor);
        skip_horizontal_whitespace(bytes, cursor);
    }
}

fn skip_physical_line_endings(bytes: &[u8], cursor: &mut usize) {
    while matches!(bytes.get(*cursor), Some(b'\n' | b'\r')) {
        *cursor = physical_line_resume(bytes, *cursor);
    }
}

fn double_quote_ends_scanner_argument(
    bytes: &[u8],
    quote: usize,
    text_tokens: &TextTokenCursor,
) -> bool {
    if bytes.get(quote + 1..quote + 4) == Some(&b"]]]"[..]) {
        return true;
    }
    if double_quote_ends_wikidot_argument(bytes, quote, text_tokens) {
        return true;
    }
    if let Some(next) = continuation_revealed_argument_boundary(bytes, quote + 1) {
        return scanner_argument_boundary_at(bytes, next, text_tokens);
    }
    matches!(bytes.get(quote + 1), Some(b' ' | b'\t' | b'\0'))
        && scanner_argument_boundary_at(bytes, quote + 1, text_tokens)
}

fn continuation_revealed_argument_boundary(
    bytes: &[u8],
    mut cursor: usize,
) -> Option<usize> {
    let mut pending_backslashes = 0usize;
    let mut consumed_line_end = false;
    loop {
        match bytes.get(cursor) {
            Some(b'\\') => {
                pending_backslashes += 1;
                cursor += 1;
            }
            Some(b'\n' | b'\r') if pending_backslashes > 0 => {
                pending_backslashes -= 1;
                cursor = physical_line_resume(bytes, cursor);
                consumed_line_end = true;
            }
            _ => break,
        }
    }
    (consumed_line_end && pending_backslashes == 0).then_some(cursor)
}

fn scanner_argument_boundary_at(
    bytes: &[u8],
    mut cursor: usize,
    text_tokens: &TextTokenCursor,
) -> bool {
    let mut lookahead_tokens = text_tokens.clone();
    if cursor >= bytes.len()
        || matches!(bytes[cursor], b'\n' | b'\r')
        || (bytes[cursor] == b']'
            && wikidot_right_bracket_token(
                bytes,
                cursor,
                bytes.len(),
                &mut lookahead_tokens,
            )
            .0)
    {
        return true;
    }
    while matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'\0')) {
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
    while matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'\0')) {
        cursor += 1;
    }
    if bytes.get(cursor) == Some(&b'!') {
        cursor += 1;
    }
    bytes.get(cursor) == Some(&b'=') && !lookahead_tokens.contains(cursor)
}

fn physical_line_resume(bytes: &[u8], line_end: usize) -> usize {
    debug_assert!(matches!(bytes.get(line_end), Some(b'\n' | b'\r')));
    if bytes[line_end] == b'\r' && bytes.get(line_end + 1) == Some(&b'\n') {
        line_end + 2
    } else {
        line_end + 1
    }
}

fn next_physical_line_resume(bytes: &[u8], mut cursor: usize) -> usize {
    while !matches!(bytes.get(cursor), None | Some(b'\n' | b'\r')) {
        cursor += 1;
    }
    if cursor == bytes.len() {
        cursor
    } else {
        physical_line_resume(bytes, cursor)
    }
}

fn quote_follows_argument_equals(bytes: &[u8], quote: usize, lower_bound: usize) -> bool {
    let mut cursor = quote;
    while cursor > lower_bound && matches!(bytes[cursor - 1], b' ' | b'\t') {
        cursor -= 1;
    }
    cursor > lower_bound && bytes[cursor - 1] == b'='
}

pub(super) fn find_list_pages_module_matches(
    source: &str,
) -> Vec<ListPagesModuleMatch<'_>> {
    find_list_pages_module_matches_with_cursor_work(source).0
}

/// The work value measures source-cursor displacement, speculative head bytes, literal-range cursor advances, projection-offset advances, and projected/direct event merge advances.
fn find_list_pages_module_matches_with_cursor_work(
    source: &str,
) -> (Vec<ListPagesModuleMatch<'_>>, usize, usize) {
    let lowercase = source.to_ascii_lowercase();
    if !lowercase.contains("[[") {
        return (Vec::new(), 0, 0);
    }
    if !lowercase.contains("listpages") {
        return (Vec::new(), source.len(), 0);
    }
    let projection = ListPagesSourceProjection::new(source);
    let projected_lowercase = projection
        .as_ref()
        .map(|projection| projection.source().to_ascii_lowercase());
    let ListPagesScannerLiteralIndexes {
        direct: direct_literal_regions,
        projected: projected_literal_regions,
        original_css: original_css_regions,
        original_anchors: original_anchor_regions,
    } = LiteralRegionIndex::new_list_pages_scanner_indexes(source, projection.as_ref());
    let direct_scanner =
        ModuleEventScanner::new(source, &lowercase, &direct_literal_regions);
    let (
        mut direct_events,
        mut direct_work,
        mut direct_literal_advances,
        direct_ambiguous,
    ) = collect_module_events(direct_scanner);
    if direct_ambiguous {
        return (
            Vec::new(),
            direct_work + direct_literal_advances,
            direct_literal_advances,
        );
    }

    let changed_quote_ranges = projection
        .as_ref()
        .map(|projection| projection.changed_quote_original_ranges(source))
        .unwrap_or_default();
    if !changed_quote_ranges.is_empty() {
        let recovery_literals = LiteralRegionIndex::new_wikidot_syntax(source);
        let recovery_scanner =
            ModuleEventScanner::new(source, &lowercase, &recovery_literals);
        let (recovery_events, recovery_work, recovery_advances, recovery_ambiguous) =
            collect_module_events(recovery_scanner);
        if recovery_ambiguous {
            return (
                Vec::new(),
                direct_work + recovery_work + direct_literal_advances + recovery_advances,
                direct_literal_advances + recovery_advances,
            );
        }
        let recovery_events = recovery_events
            .into_iter()
            .filter(|event| {
                range_contains_start(&changed_quote_ranges, direct_event_start(*event))
            })
            .collect();
        direct_events = merge_module_event_streams(direct_events, recovery_events);
        direct_work += recovery_work;
        direct_literal_advances += recovery_advances;
    }

    let (events, projected_work, projected_literal_advances, merge_work) =
        if let Some(projection) = projection.as_ref() {
            let projected_lowercase = projected_lowercase
                .as_ref()
                .expect("projected lowercase accompanies a source projection");
            let projected_literal_regions = projected_literal_regions
                .as_ref()
                .expect("projected literal regions accompany a source projection");
            let projected_scanner = ModuleEventScanner::new(
                projection.source(),
                projected_lowercase,
                projected_literal_regions,
            );
            let (
                projected_events,
                projected_work,
                mut projected_literal_advances,
                projected_ambiguous,
            ) = collect_module_events(projected_scanner);
            if projected_ambiguous {
                return (
                    Vec::new(),
                    direct_work + projected_work + projected_literal_advances,
                    direct_literal_advances + projected_literal_advances,
                );
            }
            let original_css_regions = original_css_regions
                .as_ref()
                .expect("original CSS regions accompany a source projection");
            let mut original_css_cursor = original_css_regions.monotone_cursor();
            let original_anchor_regions = original_anchor_regions
                .as_ref()
                .expect("original anchor regions accompany a source projection");
            let mut original_anchor_cursor = original_anchor_regions.monotone_cursor();
            let mut events = Vec::with_capacity(projected_events.len());
            for event in projected_events {
                let event = map_projected_event(event, projection, source.len());
                if original_css_cursor.containing_end(event.start()).is_none()
                    && original_anchor_cursor
                        .containing_end(event.start())
                        .is_none()
                {
                    events.push(event);
                }
            }
            projected_literal_advances +=
                original_css_cursor.advances() + original_anchor_cursor.advances();
            let (mut events, merge_work) = merge_projected_and_direct_events(
                events,
                &direct_events,
                &changed_quote_ranges,
            );
            let projection_head_work =
                mark_projection_changed_direct_heads(&mut events, projection, source);
            (
                events,
                projected_work,
                projected_literal_advances,
                merge_work + projection_head_work,
            )
        } else {
            (
                direct_events
                    .into_iter()
                    .map(ordered_direct_event)
                    .collect(),
                0,
                0,
                0,
            )
        };

    let mut matches = Vec::new();
    let mut active = None::<ActiveListPagesModule<'_>>;

    for event in events {
        match event {
            OrderedModuleEvent::Open {
                kind,
                start,
                direct,
                ..
            } => {
                if let Some(module) = active.as_mut() {
                    let Some(depth) = module.depth.checked_add(1) else {
                        break;
                    };
                    module.depth = depth;
                    continue;
                }
                if kind != ModuleOpenKind::Standard {
                    continue;
                }

                let Some(DirectModuleOpen {
                    subname_start,
                    subname_end,
                    opening_end,
                    runtime_safe,
                }) = direct
                else {
                    continue;
                };
                if source[subname_start..subname_end].eq_ignore_ascii_case("listpages") {
                    active = Some(ActiveListPagesModule {
                        start,
                        body_start: opening_end + 2,
                        head: source[subname_end..opening_end].trim_start(),
                        depth: 1,
                        runtime_safe,
                    });
                }
            }
            OrderedModuleEvent::Close { start, end } => {
                let Some(module) = active.as_mut() else {
                    continue;
                };
                module.depth -= 1;
                if module.depth == 0 {
                    let module = active.take().unwrap();
                    matches.push(ListPagesModuleMatch {
                        start: module.start,
                        end,
                        head: module.head,
                        body: &source[module.body_start..start],
                        original: &source[module.start..end],
                        runtime_safe: module.runtime_safe,
                    });
                }
            }
        }
    }

    let literal_range_advances = direct_literal_advances + projected_literal_advances;
    (
        matches,
        direct_work + projected_work + literal_range_advances + merge_work,
        literal_range_advances,
    )
}

fn collect_module_events(
    mut scanner: ModuleEventScanner<'_>,
) -> (Vec<ModuleEvent>, usize, usize, bool) {
    let mut events = Vec::new();
    while let Some(event) = scanner.next() {
        events.push(event);
    }
    let literal_advances = scanner.literal_regions.advances();
    (
        events,
        scanner
            .scanned_bytes
            .saturating_add(scanner.speculative_bytes),
        literal_advances,
        scanner.ambiguous_whole_head,
    )
}

fn ordered_direct_event(event: ModuleEvent) -> OrderedModuleEvent {
    match event {
        ModuleEvent::Open {
            kind,
            start,
            subname_start,
            subname_end,
            opening_end,
            direct_candidate,
            runtime_safe,
            ..
        } => OrderedModuleEvent::Open {
            kind,
            start,
            end: opening_end + 2,
            direct: direct_candidate.then_some(DirectModuleOpen {
                subname_start,
                subname_end,
                opening_end,
                runtime_safe,
            }),
            projection_guard_start: None,
        },
        ModuleEvent::Close { start, end } => OrderedModuleEvent::Close { start, end },
    }
}

fn map_projected_event(
    event: ModuleEvent,
    projection: &ListPagesSourceProjection,
    original_len: usize,
) -> OrderedModuleEvent {
    match event {
        ModuleEvent::Open {
            kind,
            start,
            opening_end,
            projection_guard_start,
            ..
        } => {
            let mapped = projection.map_range(start..opening_end + 2, original_len);
            let projection_guard_start = projection_guard_start.map(|guard| {
                projection
                    .map_range(guard..opening_end + 2, original_len)
                    .start
            });
            OrderedModuleEvent::Open {
                kind,
                start: mapped.start,
                end: mapped.end,
                direct: None,
                projection_guard_start,
            }
        }
        ModuleEvent::Close { start, end } => {
            let mapped = projection.map_range(start..end, original_len);
            OrderedModuleEvent::Close {
                start: mapped.start,
                end: mapped.end,
            }
        }
    }
}

fn merge_projected_and_direct_events(
    projected: Vec<OrderedModuleEvent>,
    direct: &[ModuleEvent],
    restorable: &[Range<usize>],
) -> (Vec<OrderedModuleEvent>, usize) {
    let mut merged = Vec::with_capacity(projected.len() + direct.len());
    let mut projected = projected.into_iter().peekable();
    let mut direct_cursor = 0usize;
    let mut advances = 0usize;
    while let Some(mut projected_event) = projected.next() {
        advances += 1;
        while direct.get(direct_cursor).is_some_and(|direct_event| {
            direct_event_start(*direct_event) < projected_event.start()
        }) {
            let direct_event = direct[direct_cursor];
            if range_contains_start(restorable, direct_event_start(direct_event)) {
                merged.push(ordered_direct_event(direct_event));
            }
            direct_cursor += 1;
            advances += 1;
        }
        let Some(direct_event) = direct.get(direct_cursor).copied() else {
            merged.push(projected_event);
            merged.extend(projected);
            return (merged, advances);
        };
        if direct_event_start(direct_event) == projected_event.start() {
            if direct_event_matches_ordered(direct_event, projected_event) {
                projected_event.attach_direct(direct_event);
            }
            direct_cursor += 1;
            advances += 1;
        }
        merged.push(projected_event);
    }
    while let Some(event) = direct.get(direct_cursor).copied() {
        if range_contains_start(restorable, direct_event_start(event)) {
            merged.push(ordered_direct_event(event));
        }
        direct_cursor += 1;
        advances += 1;
    }
    (merged, advances)
}

fn merge_module_event_streams(
    primary: Vec<ModuleEvent>,
    recovery: Vec<ModuleEvent>,
) -> Vec<ModuleEvent> {
    let mut merged = Vec::with_capacity(primary.len() + recovery.len());
    let mut primary = primary.into_iter().peekable();
    let mut recovery = recovery.into_iter().peekable();
    while let (Some(left), Some(right)) = (primary.peek(), recovery.peek()) {
        match direct_event_start(*left).cmp(&direct_event_start(*right)) {
            std::cmp::Ordering::Less => merged.push(primary.next().unwrap()),
            std::cmp::Ordering::Greater => merged.push(recovery.next().unwrap()),
            std::cmp::Ordering::Equal => {
                merged.push(primary.next().unwrap());
                recovery.next();
            }
        }
    }
    merged.extend(primary);
    merged.extend(recovery);
    merged
}

fn range_contains_start(ranges: &[Range<usize>], start: usize) -> bool {
    let insertion = ranges.partition_point(|range| range.start <= start);
    insertion > 0 && start < ranges[insertion - 1].end
}

fn mark_projection_changed_direct_heads(
    events: &mut [OrderedModuleEvent],
    projection: &ListPagesSourceProjection,
    original: &str,
) -> usize {
    let mut guard_ranges = projection.original_range_cursor();
    let mut head_ranges = projection.original_range_cursor();

    for event in events {
        let OrderedModuleEvent::Open {
            direct,
            projection_guard_start,
            ..
        } = event
        else {
            continue;
        };
        let Some(current) = direct.as_ref() else {
            continue;
        };
        if projection_guard_start.is_some_and(|guard| {
            !guard_ranges.range_is_unchanged(original, guard..current.opening_end)
        }) {
            *direct = None;
            continue;
        }
        let direct = direct
            .as_mut()
            .expect("direct module remains attached after its projection guard");
        if !head_ranges
            .range_is_unchanged(original, direct.subname_end..direct.opening_end)
        {
            direct.runtime_safe = false;
        }
    }

    let offset_advances = guard_ranges.advances() + head_ranges.advances();
    record_projection_offset_advances(offset_advances);
    offset_advances
}

fn direct_event_start(event: ModuleEvent) -> usize {
    match event {
        ModuleEvent::Open { start, .. } | ModuleEvent::Close { start, .. } => start,
    }
}

fn direct_event_matches_ordered(
    direct: ModuleEvent,
    projected: OrderedModuleEvent,
) -> bool {
    match (direct, projected) {
        (
            ModuleEvent::Open {
                kind: direct_kind,
                start: direct_start,
                opening_end,
                ..
            },
            OrderedModuleEvent::Open {
                kind: projected_kind,
                start: projected_start,
                end: projected_end,
                ..
            },
        ) => {
            direct_kind == projected_kind
                && direct_start == projected_start
                && opening_end + 2 == projected_end
        }
        (
            ModuleEvent::Close {
                start: direct_start,
                end: direct_end,
            },
            OrderedModuleEvent::Close {
                start: projected_start,
                end: projected_end,
            },
        ) => direct_start == projected_start && direct_end == projected_end,
        _ => false,
    }
}

#[cfg(test)]
#[path = "scanner/oracle_tests.rs"]
mod oracle_tests;

#[cfg(test)]
#[path = "scanner/count_reachability_tests.rs"]
mod count_reachability_tests;

#[cfg(test)]
#[path = "scanner/tests.rs"]
mod tests;
