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

#[cfg(test)]
use self::head_validation::pinned_double_quote_ends_generic_argument;
use super::super::literal_regions::{
    ListPagesScannerLiteralIndexes, ListPagesSourceProjection, LiteralRegionCursor,
    LiteralRegionIndex, TextTokenCursor, WikidotArgumentValueKind,
    WikidotTagArgumentScan, WikidotWholeHeadScan,
    classify_wikidot_right_bracket_and_advance_text_tokens,
    double_quote_ends_wikidot_argument, left_block_start_in_run,
    project_list_pages_typography_in_place, quote_is_escaped, rollback_start_in_left_run,
    scan_wikidot_whole_head_value, wikidot_trimmed_name,
};
use super::super::module_arguments::wikidot_list_pages_arguments;
use super::super::render_budget::SharedRenderCostBudget;
#[path = "scanner/count_reachability.rs"]
mod count_reachability;
#[path = "scanner/event_merge.rs"]
mod event_merge;
#[path = "scanner/head_validation.rs"]
mod head_validation;
#[path = "scanner/legacy_heads.rs"]
mod legacy_heads;
#[path = "scanner/legacy_recovery.rs"]
mod legacy_recovery;
#[path = "scanner/runtime_heads.rs"]
mod runtime_heads;

pub(in crate::services::render) use self::count_reachability::CountPagesCloseReachabilityIndex;
use self::event_merge::{
    collect_module_events, direct_event_start, map_projected_event,
    mark_projection_changed_direct_heads, merge_module_event_streams,
    merge_projected_and_direct_events, ordered_direct_event, range_contains_start,
};
use self::head_validation::{
    ModuleHeadValidation, continuation_revealed_argument_boundary,
    final_unclosed_list_pages_head_boundary, is_horizontal_whitespace,
    is_wikidot_head_spacing, list_pages_definite_invalid_head_can_execute,
    list_pages_head_double_quotes_are_balanced, list_pages_url_value_quote_ends_at,
    module_subname_end, nested_list_pages_head_token_end,
    nested_list_pages_head_token_is_module, next_physical_line_resume,
    physical_line_resume, quote_follows_argument_equals,
    right_boundary_ends_physical_line, scanner_argument_boundary_at,
    source_block_open_is_backslash_escaped, trimmed_utf8_span,
    validate_generic_head_arguments, validate_module_head,
};
use self::legacy_heads::{
    crossing_list_pages_quote_ends_before_close, double_quote_ends_scanner_argument,
    is_module_argument_spacing, legacy_single_bracket_list_pages_opening,
    list_pages_head_comment_end, list_pages_inert_at_markers_follow_quote,
    list_pages_quote_ends_before_comment, skip_count_pages_module_subname_delimiter,
    skip_horizontal_whitespace, skip_module_argument_spacing, skip_module_close_spacing,
    skip_module_subname_delimiter, surplus_list_pages_close_after_spacing,
};
use self::legacy_recovery::{
    complete_multiline_at_marker_template_boundary,
    legacy_at_marker_footnote_default_list_pages,
    nested_module_opening_has_inert_at_marker_suffix,
    preserved_at_marker_documentation_opening, trailing_at_marker_raw_row_tail_end,
    unclosed_at_marker_collapsible_prefix_end,
    unclosed_at_marker_requires_module654_preservation,
    unclosed_list_pages_has_immediate_raw_closer,
    unclosed_list_pages_owns_legacy_quoted_continuation,
};
pub(in crate::services::render) use self::runtime_heads::list_pages_runtime_head_can_execute;
use self::runtime_heads::{
    list_pages_bare_comparison_key_is_evidenced,
    list_pages_url_quote_crossing_head_can_execute, normalize_module_head,
    runtime_list_pages_key_is_supported, unresolved_block_conditional_prefix,
};
#[cfg(test)]
use self::runtime_heads::{
    list_pages_runtime_head_is_safe, runtime_regex_recognizes_entire_head,
};
#[cfg(test)]
use std::cell::Cell;
use std::ops::Range;

const SPECULATIVE_WORK_LIMIT_MULTIPLIER: usize = 8;
const SCANNER_COST_UNIT_BYTES: usize = 1024;

#[inline]
fn scanner_budget_is_exhausted(budget: Option<&SharedRenderCostBudget>) -> bool {
    budget.is_some_and(|budget| budget.is_exhausted())
}

pub(in crate::services::render) fn has_list_pages_module_opening_candidate(
    source: &str,
) -> bool {
    first_list_pages_module_opening_candidate(source).is_some()
}

pub(in crate::services::render) fn first_list_pages_module_opening_candidate(
    source: &str,
) -> Option<usize> {
    first_module_opening_candidate(source, b"listpages", true)
}

pub(in crate::services::render) fn has_count_pages_module_opening_candidate(
    source: &str,
) -> bool {
    first_module_opening_candidate(source, b"countpages", false).is_some()
}

pub(in crate::services::render) fn list_pages_body_has_standalone_count_pages_opening(
    body: &str,
) -> bool {
    let literal_regions = LiteralRegionIndex::new_count_pages_syntax(body);
    let mut line_start = 0usize;
    for line_with_ending in body.split_inclusive('\n') {
        let line_without_ending = line_with_ending.trim_end_matches(['\r', '\n']);
        let leading_whitespace = line_without_ending
            .len()
            .saturating_sub(line_without_ending.trim_start().len());
        let line = line_without_ending.trim();
        let candidate_offset = line_start.saturating_add(leading_whitespace);
        if first_module_opening_candidate(line, b"countpages", false) == Some(0)
            && module_opening_occupies_entire_line(line)
            && literal_regions.containing_range(candidate_offset).is_none()
        {
            return true;
        }
        line_start += line_with_ending.len();
    }
    false
}

pub(in crate::services::render) fn list_pages_body_inline_count_pages_legacy_tail(
    body: &str,
) -> Option<String> {
    let literal_regions = LiteralRegionIndex::new_count_pages_syntax(body);
    let mut line_start = 0usize;
    for line_with_ending in body.split_inclusive('\n') {
        let line_without_ending = line_with_ending.trim_end_matches(['\r', '\n']);
        let leading_whitespace = line_without_ending
            .len()
            .saturating_sub(line_without_ending.trim_start().len());
        let line = line_without_ending.trim_start();
        let candidate_offset = line_start.saturating_add(leading_whitespace);
        if first_module_opening_candidate(line, b"countpages", false) == Some(0)
            && literal_regions.containing_range(candidate_offset).is_none()
            && let Some(opening_end) = module_opening_end(line)
            && let Some(close_start) =
                find_module_close_ascii_case_insensitive(&line[opening_end..])
        {
            let close_end =
                candidate_offset + opening_end + close_start + "[[/module]]".len();
            let mut tail = String::with_capacity(body.len() + 32);
            tail.push_str(&body[..candidate_offset]);
            tail.push_str("[[div class=\"list-pages-box\"]]");
            tail.push_str(&body[close_end..]);
            tail.push_str("[[/div]]");
            return Some(tail);
        }
        line_start += line_with_ending.len();
    }
    None
}

fn module_opening_occupies_entire_line(line: &str) -> bool {
    module_opening_end(line).is_some_and(|end| line[end..].trim().is_empty())
}

fn module_opening_end(line: &str) -> Option<usize> {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut bytes = line.bytes().enumerate();
    while let Some((index, byte)) = bytes.next() {
        match byte {
            b'\'' if !in_double_quote => in_single_quote = !in_single_quote,
            b'"' if !in_single_quote => in_double_quote = !in_double_quote,
            b']' if !in_single_quote
                && !in_double_quote
                && bytes.next().is_some_and(|(_, next_byte)| next_byte == b']') =>
            {
                return Some(index + 2);
            }
            _ => {}
        }
    }
    None
}

fn find_module_close_ascii_case_insensitive(source: &str) -> Option<usize> {
    source
        .as_bytes()
        .windows(b"[[/module]]".len())
        .position(|window| window.eq_ignore_ascii_case(b"[[/module]]"))
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
    static LOWERCASE_SOURCE_BYTES: Cell<usize> = const { Cell::new(0) };
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

#[inline]
fn record_lowercase_source_bytes(count: usize) {
    #[cfg(test)]
    LOWERCASE_SOURCE_BYTES.with(|total| total.set(total.get().saturating_add(count)));
    #[cfg(not(test))]
    let _ = count;
}

#[cfg(test)]
fn take_lowercase_source_bytes() -> usize {
    LOWERCASE_SOURCE_BYTES.with(|total| total.replace(0))
}

#[derive(Debug)]
pub(in crate::services::render) struct ListPagesModuleMatch<'a> {
    pub(in crate::services::render) start: usize,
    pub(in crate::services::render) body_start: usize,
    pub(in crate::services::render) end: usize,
    pub(in crate::services::render) head: &'a str,
    pub(in crate::services::render) body: &'a str,
    pub(in crate::services::render) original: &'a str,
    pub(in crate::services::render) runtime_safe: bool,
    pub(in crate::services::render) preserve_original: bool,
    pub(in crate::services::render) preserve_as_module654: bool,
    pub(in crate::services::render) consume_empty_tail: bool,
}

#[derive(Debug)]
struct ActiveListPagesModule<'a> {
    start: usize,
    body_start: usize,
    head: &'a str,
    depth: usize,
    runtime_safe: bool,
    default_template: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PreservedAtMarkerDocumentationOpening {
    start: usize,
    body_start: usize,
    resume: usize,
    subname_end: usize,
    opening_end: usize,
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
        body_start: usize,
        subname_end: usize,
        runtime_safe: bool,
        default_template: bool,
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
        body_start: usize,
        direct_candidate: bool,
        runtime_safe: bool,
        default_template: bool,
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
    body_start: usize,
    runtime_safe: bool,
    default_template: bool,
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
                body_start,
                direct_candidate: true,
                runtime_safe,
                default_template,
                ..
            },
        ) = (self, event)
        {
            *direct = Some(DirectModuleOpen {
                subname_start,
                subname_end,
                opening_end,
                body_start,
                runtime_safe,
                default_template,
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
                            && unresolved_block_conditional_prefix(
                                &self.source[*marker + 3..],
                            )
                    })
                    && self
                        .literal_regions
                        .advance_to_containing_end(marker)
                        .is_none()
                {
                    self.ambiguous_whole_head = true;
                    self.advance_to(self.lowercase.len());
                    return None;
                }
                self.advance_to(run_end);
                continue;
            };
            self.advance_to(start + 2);

            if source_block_open_is_backslash_escaped(self.source, start) {
                continue;
            }

            if let Some(end) = self.literal_regions.advance_to_containing_end(start) {
                self.advance_to(end);
                continue;
            }
            if self.lowercase.as_bytes().get(start..start + 3) == Some(&b"[[#"[..])
                && unresolved_block_conditional_prefix(&self.source[start + 3..])
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
                let (
                    opening_end,
                    body_start,
                    subname_end,
                    runtime_safe,
                    default_template,
                ) = match self.module_opening_end(tag.name_end) {
                    ModuleOpeningEnd::Complete {
                        opening_end,
                        body_start,
                        subname_end,
                        runtime_safe,
                        default_template,
                    } => (
                        opening_end,
                        body_start,
                        subname_end,
                        runtime_safe,
                        default_template,
                    ),
                    ModuleOpeningEnd::Malformed { resume } => {
                        self.advance_to(resume);
                        continue;
                    }
                    ModuleOpeningEnd::Unclosed => {
                        self.advance_to(self.lowercase.len());
                        return None;
                    }
                };
                self.advance_to(body_start);
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
                    body_start,
                    direct_candidate: tag.direct_candidate,
                    runtime_safe,
                    default_template,
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
                if bytes[cursor] == b'\t' && tag_tokens.advance_and_contains(cursor) {
                    cursor += 1;
                    continue;
                } else if matches!(bytes[cursor], b' ' | b'\t' | b'\n' | b'\r') {
                    bare_image_link = false;
                } else if bytes[cursor] == b']' {
                    let (right_block, token_len) =
                        classify_wikidot_right_bracket_and_advance_text_tokens(
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
                && !tag_tokens.advance_and_contains(cursor)
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
                        && !tag_tokens.advance_and_contains(cursor)
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
                        && !tag_tokens.advance_and_contains(cursor) =>
                {
                    quote = None;
                    quote_started_owned = false;
                }
                (None, b'\'' | b'"')
                    if !quote_is_escaped(bytes, cursor, &tag_tokens)
                        && (!tag_tokens.advance_and_contains(cursor)
                            || quote_follows_argument_equals(
                                bytes,
                                cursor,
                                start + 2,
                            )) =>
                {
                    quote = Some(bytes[cursor]);
                    quote_started_owned = tag_tokens.advance_and_contains(cursor);
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
                    let (right_block, token_len) =
                        classify_wikidot_right_bracket_and_advance_text_tokens(
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
        let (right_block, token_len) =
            classify_wikidot_right_bracket_and_advance_text_tokens(
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
            if list_pages_compatibility
                && quote.is_some()
                && bytes.get(cursor..cursor + 2) == Some(&b"]]"[..])
                && (list_pages_head_double_quotes_are_balanced(
                    &source[subname_end..cursor],
                ) || list_pages_url_quote_crossing_head_can_execute(
                    source[subname_end..cursor].trim_start(),
                ))
            {
                let mut closing_tokens = head_tokens.clone();
                let (right_block, token_len) =
                    classify_wikidot_right_bracket_and_advance_text_tokens(
                        bytes,
                        cursor,
                        bytes.len(),
                        &mut closing_tokens,
                    );
                if right_block
                    && right_boundary_ends_physical_line(bytes, cursor + token_len)
                {
                    self.text_tokens = closing_tokens;
                    finish_head_scan!(
                        cursor + token_len,
                        ModuleOpeningEnd::Complete {
                            opening_end: cursor,
                            body_start: cursor + token_len,
                            subname_end,
                            runtime_safe: false,
                            default_template: false,
                        }
                    );
                }
            }
            if list_pages_compatibility
                && cursor >= subname_end
                && bytes.get(cursor..cursor + 2) == Some(&b"[["[..])
                && list_pages_head_double_quotes_are_balanced(
                    &source[subname_end..cursor],
                )
                && !nested_list_pages_head_token_is_module(bytes, cursor)
                && let Some(nested_end) = nested_list_pages_head_token_end(source, cursor)
            {
                trailing_backslashes = 0;
                cursor = nested_end;
                continue;
            }
            if bytes[cursor] == b'['
                && bytes.get(cursor + 1) == Some(&b'[')
                && !head_tokens.advance_and_contains(cursor)
            {
                let head_end =
                    source[..cursor].trim_end_matches(char::is_whitespace).len();
                let whitespace = &source[head_end..cursor];
                let validation_head =
                    source[subname_end..head_end].trim_start_matches([' ', '\t']);
                if list_pages_compatibility
                    && quote.is_none()
                    && bytes
                        .get(cursor..cursor + b"[[/module]]".len())
                        .is_some_and(|candidate| {
                            candidate.eq_ignore_ascii_case(b"[[/module]]")
                        })
                    && whitespace.contains(['\n', '\r'])
                    && !wikidot_list_pages_arguments(validation_head).is_empty()
                    && list_pages_runtime_head_can_execute(validation_head)
                {
                    finish_head_scan!(
                        cursor,
                        ModuleOpeningEnd::Complete {
                            opening_end: head_end,
                            body_start: cursor,
                            subname_end,
                            runtime_safe: false,
                            default_template: false,
                        }
                    );
                }
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
                if list_pages_compatibility && quote == Some(b'\'') {
                    self.ambiguous_whole_head = true;
                    let resume = physical_line_resume(bytes, cursor);
                    finish_head_scan!(
                        resume,
                        ModuleOpeningEnd::Malformed {
                            resume: first_rollback_marker.unwrap_or(resume),
                        }
                    );
                }
                if list_pages_compatibility
                    && quote == Some(b'"')
                    && let Some(opening_end) = final_unclosed_list_pages_head_boundary(
                        source,
                        subname_end,
                        cursor,
                    )
                {
                    let body_start = physical_line_resume(bytes, cursor);
                    self.text_tokens = head_tokens;
                    finish_head_scan!(
                        body_start,
                        ModuleOpeningEnd::Complete {
                            opening_end,
                            body_start,
                            subname_end,
                            runtime_safe: false,
                            default_template: true,
                        }
                    );
                }
                if list_pages_compatibility
                    && quote == Some(b'"')
                    && let Some((opening_end, body_start)) =
                        legacy_single_bracket_list_pages_opening(
                            source,
                            subname_end,
                            cursor,
                        )
                {
                    self.text_tokens = head_tokens;
                    finish_head_scan!(
                        body_start,
                        ModuleOpeningEnd::Complete {
                            opening_end,
                            body_start,
                            subname_end,
                            runtime_safe: false,
                            default_template: false,
                        }
                    );
                }
                if list_pages_compatibility {
                    trailing_backslashes = 0;
                    cursor = physical_line_resume(bytes, cursor);
                    continue;
                }
                if trailing_backslashes > 0 {
                    trailing_backslashes -= 1;
                    cursor = physical_line_resume(bytes, cursor);
                    continue;
                }
                let resume = physical_line_resume(bytes, cursor);
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
                        && (!head_tokens.advance_and_contains(cursor)
                            || (list_pages_compatibility
                                && list_pages_url_value_quote_ends_at(
                                    bytes,
                                    cursor,
                                    module_name_end,
                                )))
                        && (double_quote_ends_scanner_argument(
                            bytes,
                            cursor,
                            &head_tokens,
                        ) || (list_pages_compatibility
                            && (list_pages_url_value_quote_ends_at(
                                bytes,
                                cursor,
                                module_name_end,
                            ) || surplus_list_pages_close_after_spacing(
                                bytes,
                                cursor + 1,
                            ) || list_pages_inert_at_markers_follow_quote(
                                bytes, cursor,
                            ) || list_pages_quote_ends_before_comment(
                                bytes, cursor,
                            ) || crossing_list_pages_quote_ends_before_close(
                                bytes,
                                cursor,
                                &head_tokens,
                            )))) =>
                {
                    quote = None;
                }
                (Some(b'\''), b'\'')
                    if !quote_is_escaped(bytes, cursor, &head_tokens)
                        && !head_tokens.advance_and_contains(cursor) =>
                {
                    quote = None;
                }
                (None, b'\'' | b'"')
                    if !quote_is_escaped(bytes, cursor, &head_tokens)
                        && (!head_tokens.advance_and_contains(cursor)
                            || quote_follows_argument_equals(
                                bytes,
                                cursor,
                                module_name_end,
                            )) =>
                {
                    quote = Some(bytes[cursor]);
                }
                (None, b']') => {
                    if cursor < subname_end {
                        self.ambiguous_whole_head = true;
                        let resume = cursor.saturating_add(1);
                        finish_head_scan!(resume, ModuleOpeningEnd::Malformed { resume });
                    }
                    let validation_head =
                        source[subname_end..cursor].trim_start_matches([' ', '\t']);
                    let (right_block, token_len) =
                        classify_wikidot_right_bracket_and_advance_text_tokens(
                            bytes,
                            cursor,
                            bytes.len(),
                            &mut head_tokens,
                        );
                    let surplus_list_pages_close_end = (list_pages_compatibility
                        && !right_block)
                        .then(|| {
                            let mut end = cursor;
                            while bytes.get(end) == Some(&b']') {
                                end += 1;
                            }
                            (end.saturating_sub(cursor) >= 3).then_some(end)
                        })
                        .flatten();
                    let inert_at_marker_close_end = (list_pages_compatibility
                        && bytes.get(cursor..cursor + 2) == Some(&b"]]"[..])
                        && source[subname_end..cursor]
                            .trim_end_matches([' ', '\t'])
                            .ends_with("@@"))
                    .then_some(cursor + 2);
                    if right_block
                        || surplus_list_pages_close_end.is_some()
                        || inert_at_marker_close_end.is_some()
                    {
                        let closing_end = surplus_list_pages_close_end
                            .or(inert_at_marker_close_end)
                            .unwrap_or(cursor + token_len);
                        let raw_head = &source[subname_end..cursor];
                        let mut validation = validate_module_head(
                            validation_head,
                            list_pages_compatibility,
                        );
                        let runtime_recognized = list_pages_compatibility
                            && !super::super::module_arguments::wikidot_list_pages_arguments(
                                validation_head,
                            )
                            .is_empty();
                        if !self.charge_speculative(raw_head.len().saturating_mul(3)) {
                            record_module_head_scan_bytes(
                                cursor.saturating_sub(module_name_end),
                            );
                            return ModuleOpeningEnd::Unclosed;
                        }
                        if validation == ModuleHeadValidation::DefiniteInvalid {
                            validation = if list_pages_compatibility
                                && first_rollback_marker.is_none()
                                && (list_pages_url_quote_crossing_head_can_execute(
                                    validation_head,
                                ) || list_pages_definite_invalid_head_can_execute(
                                    validation_head,
                                )) {
                                ModuleHeadValidation::ValidRuntimeUnsafe
                            } else if list_pages_compatibility
                                && runtime_recognized
                                && (first_rollback_marker.is_some()
                                    || validation_head.contains(['\n', '\r']))
                            {
                                self.ambiguous_whole_head = true;
                                finish_head_scan!(
                                    closing_end,
                                    ModuleOpeningEnd::Malformed {
                                        resume: first_rollback_marker
                                            .unwrap_or(closing_end),
                                    }
                                );
                            } else if first_rollback_marker.is_none() {
                                finish_head_scan!(
                                    closing_end,
                                    ModuleOpeningEnd::Malformed {
                                        resume: closing_end,
                                    }
                                );
                            } else if let Some(resume) = first_rollback_marker {
                                finish_head_scan!(
                                    closing_end,
                                    ModuleOpeningEnd::Malformed { resume }
                                );
                            } else {
                                unreachable!(
                                    "the preceding branch owns a missing rollback marker"
                                );
                            };
                        }
                        let runtime_safe = match validation {
                            ModuleHeadValidation::DefiniteInvalid => {
                                finish_head_scan!(
                                    closing_end,
                                    ModuleOpeningEnd::Malformed {
                                        resume: closing_end,
                                    }
                                );
                            }
                            ModuleHeadValidation::ValidRuntimeBoundaryDivergence => {
                                if first_rollback_marker.is_none() {
                                    self.ambiguous_whole_head = true;
                                    finish_head_scan!(
                                        closing_end,
                                        ModuleOpeningEnd::Malformed {
                                            resume: closing_end,
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
                            closing_end,
                            ModuleOpeningEnd::Complete {
                                opening_end: closing_end - 2,
                                body_start: closing_end,
                                subname_end,
                                runtime_safe,
                                default_template: false,
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
            if list_pages_compatibility && !definite_invalid_boundary {
                self.ambiguous_whole_head = true;
                ModuleOpeningEnd::Unclosed
            } else {
                ModuleOpeningEnd::Malformed { resume }
            }
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

pub(in crate::services::render) fn find_list_pages_module_matches(
    source: &str,
) -> Vec<ListPagesModuleMatch<'_>> {
    find_list_pages_module_matches_with_cursor_work(source).0
}

/// The work value measures source-cursor displacement, speculative head bytes, literal-range cursor advances, projection-offset advances, and projected/direct event merge advances.
fn find_list_pages_module_matches_with_cursor_work(
    source: &str,
) -> (Vec<ListPagesModuleMatch<'_>>, usize, usize) {
    find_list_pages_module_matches_with_cursor_work_context(source, false)
}

fn find_list_pages_module_matches_with_cursor_work_context(
    source: &str,
    allow_literal_documentation_example: bool,
) -> (Vec<ListPagesModuleMatch<'_>>, usize, usize) {
    record_lowercase_source_bytes(source.len());
    let lowercase = source.to_ascii_lowercase();
    find_list_pages_module_matches_with_cursor_work_context_lowercase(
        source,
        &lowercase,
        allow_literal_documentation_example,
        None,
    )
}

pub(in crate::services::render) fn find_list_pages_module_matches_with_budget<'a>(
    source: &'a str,
    budget: &SharedRenderCostBudget,
) -> Vec<ListPagesModuleMatch<'a>> {
    record_lowercase_source_bytes(source.len());
    let lowercase = source.to_ascii_lowercase();
    find_list_pages_module_matches_with_cursor_work_context_lowercase(
        source,
        &lowercase,
        false,
        Some(budget),
    )
    .0
}

fn find_list_pages_module_matches_with_cursor_work_context_lowercase<'a>(
    source: &'a str,
    lowercase: &str,
    allow_literal_documentation_example: bool,
    budget: Option<&SharedRenderCostBudget>,
) -> (Vec<ListPagesModuleMatch<'a>>, usize, usize) {
    if let Some(budget) = budget {
        let requested = source.len().saturating_add(SCANNER_COST_UNIT_BYTES - 1)
            / SCANNER_COST_UNIT_BYTES;
        if budget
            .charge(requested.max(1) as u64, "ListPages source scanner")
            .is_err()
        {
            return (Vec::new(), 0, 0);
        }
    }
    if !lowercase.contains("[[") {
        return (Vec::new(), 0, 0);
    }
    if !lowercase.contains("listpages") {
        return (Vec::new(), source.len(), 0);
    }
    let mut legacy_matches = Vec::new();
    let mut legacy_source = source;
    let mut legacy_lowercase = lowercase;
    let mut legacy_offset = 0usize;
    while let Some((mut module, end)) =
        legacy_at_marker_footnote_default_list_pages(legacy_source, legacy_lowercase)
    {
        module.start += legacy_offset;
        module.body_start += legacy_offset;
        module.end += legacy_offset;
        legacy_matches.push(module);
        legacy_offset += end;
        if end == legacy_source.len() {
            return (legacy_matches, source.len(), 0);
        }
        legacy_source = &legacy_source[end..];
        legacy_lowercase = &legacy_lowercase[end..];
    }
    if legacy_offset != 0 {
        let (mut matches, work, literal_advances) =
            find_list_pages_module_matches_with_cursor_work_context_lowercase(
                legacy_source,
                legacy_lowercase,
                allow_literal_documentation_example,
                budget,
            );
        if scanner_budget_is_exhausted(budget) {
            return (Vec::new(), 0, 0);
        }
        for module in &mut matches {
            module.start += legacy_offset;
            module.body_start += legacy_offset;
            module.end += legacy_offset;
        }
        legacy_matches.extend(matches);
        return (
            legacy_matches,
            legacy_offset.saturating_add(work),
            literal_advances,
        );
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
    } = match LiteralRegionIndex::new_list_pages_scanner_indexes(
        source,
        projection.as_ref(),
    ) {
        Ok(indexes) => indexes,
        Err(whole_head_scan_work) => {
            // Preserve authored source when literal ownership cannot be
            // established within the source-proportional work budget.
            return (Vec::new(), whole_head_scan_work, 0);
        }
    };
    let direct_scanner =
        ModuleEventScanner::new(source, lowercase, &direct_literal_regions);
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

    if let Some(boundary) = preserved_at_marker_documentation_opening(
        source,
        lowercase,
        &direct_events,
        allow_literal_documentation_example,
    ) {
        let mut total_work = direct_work.saturating_add(direct_literal_advances);
        let mut total_literal_advances = direct_literal_advances;
        let (mut matches, prefix_work, prefix_literal_advances) =
            find_list_pages_module_matches_with_cursor_work_context_lowercase(
                &source[..boundary.start],
                &lowercase[..boundary.start],
                false,
                budget,
            );
        if scanner_budget_is_exhausted(budget) {
            return (Vec::new(), 0, 0);
        }
        total_work = total_work.saturating_add(prefix_work);
        total_literal_advances =
            total_literal_advances.saturating_add(prefix_literal_advances);
        matches.push(ListPagesModuleMatch {
            start: boundary.start,
            body_start: boundary.body_start,
            end: boundary.body_start,
            head: source[boundary.subname_end..boundary.opening_end]
                .trim_start()
                .trim_end_matches(']'),
            body: &source[boundary.body_start..boundary.body_start],
            original: &source[boundary.start..boundary.body_start],
            runtime_safe: false,
            preserve_original: true,
            preserve_as_module654: false,
            consume_empty_tail: false,
        });

        let (mut suffix_matches, suffix_work, suffix_literal_advances) =
            find_list_pages_module_matches_with_cursor_work_context_lowercase(
                &source[boundary.resume..],
                &lowercase[boundary.resume..],
                true,
                budget,
            );
        if scanner_budget_is_exhausted(budget) {
            return (Vec::new(), 0, 0);
        }
        for module in &mut suffix_matches {
            module.start += boundary.resume;
            module.body_start += boundary.resume;
            module.end += boundary.resume;
        }
        matches.extend(suffix_matches);
        total_work = total_work.saturating_add(suffix_work);
        total_literal_advances =
            total_literal_advances.saturating_add(suffix_literal_advances);
        return (matches, total_work, total_literal_advances);
    }

    let changed_quote_ranges = projection
        .as_ref()
        .map(|projection| projection.changed_quote_original_ranges(source))
        .unwrap_or_default();
    if !changed_quote_ranges.is_empty() {
        let recovery_literals = LiteralRegionIndex::new_wikidot_syntax(source);
        let recovery_scanner =
            ModuleEventScanner::new(source, lowercase, &recovery_literals);
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
                if original_css_cursor
                    .advance_to_containing_end(event.start())
                    .is_none()
                    && original_anchor_cursor
                        .advance_to_containing_end(event.start())
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
                end,
                direct,
                ..
            } => {
                if let Some(module) = active.as_mut() {
                    if nested_module_opening_has_inert_at_marker_suffix(source, end) {
                        continue;
                    }
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
                    body_start,
                    runtime_safe,
                    default_template,
                }) = direct
                else {
                    continue;
                };
                if source[subname_start..subname_end].eq_ignore_ascii_case("listpages") {
                    active = Some(ActiveListPagesModule {
                        start,
                        body_start,
                        head: source[subname_end..opening_end]
                            .trim_start()
                            .trim_end_matches(']'),
                        depth: 1,
                        runtime_safe,
                        default_template,
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
                    let body_start = if module.default_template {
                        start
                    } else {
                        module.body_start
                    };
                    let consume_empty_tail = module.default_template
                        || (module.body_start == start
                            && !module.head.is_empty()
                            && unclosed_list_pages_has_immediate_raw_closer(
                                source, start,
                            ));
                    matches.push(ListPagesModuleMatch {
                        start: module.start,
                        body_start,
                        end,
                        head: module.head,
                        body: &source[body_start..start],
                        original: &source[module.start..end],
                        runtime_safe: module.runtime_safe,
                        preserve_original: false,
                        preserve_as_module654: false,
                        // A recovered malformed head with a real closing
                        // module owns that close even when Wikidot selects
                        // its default row template. Keep the close inside
                        // the replacement instead of exposing it as prose.
                        consume_empty_tail,
                    });
                }
            }
        }
    }

    if let Some(module) = active {
        let executable_unclosed = module.depth == 1
            && if module.head.is_empty() {
                module.body_start == source.len()
            } else {
                module.runtime_safe || list_pages_runtime_head_can_execute(module.head)
            };
        if executable_unclosed {
            let suffix = &source[module.body_start..];
            if let Some((body_end, end)) =
                complete_multiline_at_marker_template_boundary(suffix)
            {
                matches.push(ListPagesModuleMatch {
                    start: module.start,
                    body_start: module.body_start,
                    end: module.body_start + end,
                    head: module.head,
                    body: &source[module.body_start..module.body_start + body_end],
                    original: &source[module.start..module.body_start + end],
                    runtime_safe: module.runtime_safe,
                    preserve_original: false,
                    preserve_as_module654: false,
                    consume_empty_tail: module.default_template,
                });
            } else if let Some(end) = unclosed_at_marker_collapsible_prefix_end(
                &source[module.start..module.body_start],
                suffix,
            ) {
                let end = module.body_start + end;
                matches.push(ListPagesModuleMatch {
                    start: module.start,
                    body_start: module.body_start,
                    end,
                    head: module.head,
                    body: &source[module.body_start..module.body_start],
                    original: &source[module.start..end],
                    runtime_safe: module.runtime_safe,
                    preserve_original: false,
                    preserve_as_module654: false,
                    consume_empty_tail: true,
                });
                let (mut suffix_matches, suffix_work, suffix_literal_advances) =
                    find_list_pages_module_matches_with_cursor_work_context_lowercase(
                        &source[end..],
                        &lowercase[end..],
                        false,
                        budget,
                    );
                if scanner_budget_is_exhausted(budget) {
                    return (Vec::new(), 0, 0);
                }
                for suffix_module in &mut suffix_matches {
                    suffix_module.start += end;
                    suffix_module.body_start += end;
                    suffix_module.end += end;
                }
                matches.extend(suffix_matches);
                let literal_range_advances = direct_literal_advances
                    + projected_literal_advances
                    + suffix_literal_advances;
                return (
                    matches,
                    direct_work
                        + projected_work
                        + literal_range_advances
                        + merge_work
                        + suffix_work,
                    literal_range_advances,
                );
            } else if let Some(end) =
                trailing_at_marker_raw_row_tail_end(module.head, suffix)
            {
                let end = module.body_start + end;
                matches.push(ListPagesModuleMatch {
                    start: module.start,
                    body_start: module.body_start,
                    end,
                    head: module.head,
                    body: &source[module.body_start..module.body_start],
                    original: &source[module.start..end],
                    runtime_safe: module.runtime_safe,
                    preserve_original: false,
                    preserve_as_module654: false,
                    consume_empty_tail: true,
                });
            } else {
                let owns_legacy_quoted_continuation =
                    unclosed_list_pages_owns_legacy_quoted_continuation(
                        source,
                        module.body_start,
                    );
                if !owns_legacy_quoted_continuation
                    && unclosed_at_marker_requires_module654_preservation(
                        module.head,
                        suffix,
                    )
                {
                    matches.push(ListPagesModuleMatch {
                        start: module.start,
                        body_start: module.body_start,
                        end: module.body_start,
                        head: module.head,
                        body: &source[module.body_start..module.body_start],
                        original: &source[module.start..module.body_start],
                        runtime_safe: module.runtime_safe,
                        preserve_original: true,
                        preserve_as_module654: true,
                        consume_empty_tail: false,
                    });
                    let (mut suffix_matches, suffix_work, suffix_literal_advances) =
                        find_list_pages_module_matches_with_cursor_work_context_lowercase(
                            suffix,
                            &lowercase[module.body_start..],
                            false,
                            budget,
                        );
                    if scanner_budget_is_exhausted(budget) {
                        return (Vec::new(), 0, 0);
                    }
                    for suffix_module in &mut suffix_matches {
                        suffix_module.start += module.body_start;
                        suffix_module.body_start += module.body_start;
                        suffix_module.end += module.body_start;
                    }
                    matches.extend(suffix_matches);
                    let literal_range_advances = direct_literal_advances
                        + projected_literal_advances
                        + suffix_literal_advances;
                    return (
                        matches,
                        direct_work
                            + projected_work
                            + literal_range_advances
                            + merge_work
                            + suffix_work,
                        literal_range_advances,
                    );
                }
                let end = if owns_legacy_quoted_continuation {
                    source.len()
                } else {
                    module.body_start
                };
                let consume_empty_tail = module.default_template
                    || !module.head.is_empty()
                        && unclosed_list_pages_has_immediate_raw_closer(
                            source,
                            module.body_start,
                        );
                matches.push(ListPagesModuleMatch {
                    start: module.start,
                    body_start: module.body_start,
                    end,
                    head: module.head,
                    body: &source[module.body_start..end],
                    original: &source[module.start..end],
                    runtime_safe: module.runtime_safe,
                    preserve_original: false,
                    preserve_as_module654: false,
                    consume_empty_tail,
                });
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

#[cfg(test)]
#[path = "scanner/oracle_tests.rs"]
mod oracle_tests;

#[cfg(test)]
#[path = "scanner/count_reachability_tests.rs"]
mod count_reachability_tests;

#[cfg(test)]
#[path = "scanner/tests.rs"]
mod tests;
