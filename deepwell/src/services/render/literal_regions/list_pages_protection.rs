/*
 * services/render/literal_regions/list_pages_protection.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

mod candidate_graph;
mod css;
mod typography_projection;

use self::candidate_graph::collect_candidate_graph_ranges;
use self::css::collect_downstream_css_module_ranges;
use self::typography_projection::project_typography_in_place;
#[cfg(test)]
use super::block_candidates::{
    RuntimeModuleHeadCandidate, collect_head_candidate_streams,
};
use super::merge_sorted_ranges;
use super::wikidot::{PhysicalLines, physical_line_body, quote_depth_and_body};
use std::ops::Range;

pub(in crate::services::render) fn project_list_pages_typography_in_place(
    source: &mut [u8],
) -> bool {
    project_typography_in_place(source)
}

pub(super) fn collect_list_pages_literal_ranges(source: &str) -> Vec<Range<usize>> {
    collect_literal_ranges(source, true)
}

pub(super) fn collect_count_pages_inherited_ranges(source: &str) -> Vec<Range<usize>> {
    collect_literal_ranges(source, false)
}

fn collect_literal_ranges(
    source: &str,
    include_base_candidates: bool,
) -> Vec<Range<usize>> {
    if let Some(normalized) = ListPagesSourceProjection::new(source) {
        let downstream_css_ranges = collect_downstream_css_module_ranges(source);
        let normalized_ranges = collect_projected_literal_ranges(
            normalized.source(),
            false,
            include_base_candidates,
        );
        let projected_ranges = normalized.map_ranges(normalized_ranges, source.len());
        merge_sorted_ranges(downstream_css_ranges, projected_ranges)
    } else {
        collect_projected_literal_ranges(source, true, include_base_candidates)
    }
}

pub(super) fn collect_already_projected_list_pages_literal_ranges(
    source: &str,
) -> Vec<Range<usize>> {
    collect_projected_literal_ranges(source, false, true)
}

#[cfg(test)]
pub(super) fn collect_list_pages_runtime_recovery_ranges(
    source: &str,
) -> Vec<Range<usize>> {
    if let Some(projection) = ListPagesSourceProjection::new(source) {
        collect_runtime_recovery_ranges(projection.source())
            .into_iter()
            .map(|range| projection.map_literal_range(range, source.len()))
            .collect()
    } else {
        collect_runtime_recovery_ranges(source)
    }
}

#[cfg(test)]
fn collect_runtime_recovery_ranges(source: &str) -> Vec<Range<usize>> {
    collect_head_candidate_streams(source)
        .runtime_modules
        .into_iter()
        .filter_map(|candidate| match candidate {
            RuntimeModuleHeadCandidate::Exact(_) => None,
            RuntimeModuleHeadCandidate::RecoveryBarrier(range) => Some(range),
        })
        .collect()
}

pub(super) fn collect_list_pages_downstream_css_ranges(
    source: &str,
) -> Vec<Range<usize>> {
    collect_downstream_css_module_ranges(source)
}

fn collect_projected_literal_ranges(
    source: &str,
    include_downstream_css: bool,
    include_base_candidates: bool,
) -> Vec<Range<usize>> {
    let (original_quote_ranges, compat_quote_ranges) =
        collect_list_pages_quote_ranges(source);
    collect_candidate_graph_ranges(
        source,
        &original_quote_ranges,
        &compat_quote_ranges,
        include_downstream_css,
        include_base_candidates,
    )
}

pub(in crate::services::render) struct ListPagesSourceProjection {
    source: String,
    original_offsets: Vec<usize>,
}

pub(in crate::services::render) struct ListPagesOriginalRangeCursor<'a> {
    projection: &'a ListPagesSourceProjection,
    projected_cursor: usize,
    advances: usize,
}

impl ListPagesSourceProjection {
    // This early-runtime projection follows the preprocessing steps that can change literal ownership. The scanner still reads the original source, so projection cannot create a runtime module candidate.
    pub(in crate::services::render) fn new(source: &str) -> Option<Self> {
        let bytes = source.as_bytes();
        let mut prepared = Vec::with_capacity(bytes.len());
        let mut prepared_offsets = Vec::with_capacity(bytes.len());
        let mut cursor = 0usize;
        while cursor < bytes.len() {
            match bytes[cursor] {
                b' ' | b'\t' | b'\n' => cursor += 1,
                b'\r' if bytes.get(cursor + 1) == Some(&b'\n') => cursor += 2,
                b'\r' => cursor += 1,
                _ => break,
            }
        }
        let mut changed = cursor > 0;

        while cursor < bytes.len() {
            let line_start = cursor;
            while cursor < bytes.len() && !matches!(bytes[cursor], b'\n' | b'\r') {
                cursor += 1;
            }
            let line = &source[line_start..cursor];
            let whitespace_only =
                !line.is_empty() && line.chars().all(char::is_whitespace);
            if whitespace_only {
                changed = true;
            } else {
                append_projected_line_prefix(
                    line,
                    line_start,
                    &mut prepared,
                    &mut prepared_offsets,
                    &mut changed,
                );
            }
            if cursor == bytes.len() {
                break;
            }

            let newline_start = cursor;
            let byte = bytes[cursor];
            cursor += if byte == b'\r' && bytes.get(cursor + 1) == Some(&b'\n') {
                2
            } else {
                1
            };
            changed |= byte == b'\r';
            prepared.push(b'\n');
            prepared_offsets.push(newline_start);
        }

        let mut compressed = Vec::with_capacity(prepared.len());
        let mut compressed_offsets = Vec::with_capacity(prepared.len());
        for (byte, original_offset) in prepared.into_iter().zip(prepared_offsets) {
            if byte == b'\n'
                && compressed.last() == Some(&b'\n')
                && compressed.get(compressed.len().saturating_sub(2)) == Some(&b'\n')
            {
                changed = true;
                continue;
            }
            compressed.push(byte);
            compressed_offsets.push(original_offset);
        }

        let mut normalized = Vec::with_capacity(compressed.len());
        let mut original_offsets = Vec::with_capacity(compressed.len());
        for (byte, original_offset) in compressed.into_iter().zip(compressed_offsets) {
            if byte == b'\n' && normalized.last() == Some(&b'\\') {
                normalized.pop();
                original_offsets.pop();
                changed = true;
            } else {
                normalized.push(byte);
                original_offsets.push(original_offset);
            }
        }

        while normalized.last() == Some(&b'\n') {
            normalized.pop();
            original_offsets.pop();
            changed = true;
        }

        changed |= project_typography_in_place(&mut normalized);

        changed.then(|| Self {
            source: String::from_utf8(normalized)
                .expect("normalizing ASCII line endings preserves UTF-8"),
            original_offsets,
        })
    }

    pub(in crate::services::render) fn source(&self) -> &str {
        &self.source
    }

    pub(in crate::services::render) fn map_range(
        &self,
        range: Range<usize>,
        _original_len: usize,
    ) -> Range<usize> {
        debug_assert!(range.start < range.end);
        debug_assert!(range.end <= self.source.len());
        let start = self.original_offsets[range.start];
        let end = self.original_offsets[range.end - 1] + 1;
        start..end
    }

    pub(in crate::services::render) fn original_range_cursor(
        &self,
    ) -> ListPagesOriginalRangeCursor<'_> {
        ListPagesOriginalRangeCursor {
            projection: self,
            projected_cursor: 0,
            advances: 0,
        }
    }

    pub(in crate::services::render) fn changed_quote_original_ranges(
        &self,
        original: &str,
    ) -> Vec<Range<usize>> {
        let (_, quote_ranges) = collect_list_pages_quote_ranges(&self.source);
        let mut unchanged = self.original_range_cursor();
        quote_ranges
            .into_iter()
            .filter_map(|projected| {
                let mapped = self.map_range(projected.clone(), original.len());
                let disconnected_start = if projected.start == 0 {
                    mapped.start != 0
                } else {
                    self.original_offsets[projected.start - 1] + 1 != mapped.start
                };
                (disconnected_start
                    || !unchanged.range_is_unchanged(original, mapped.clone()))
                .then_some(mapped)
            })
            .collect()
    }

    pub(in crate::services::render) fn map_literal_range(
        &self,
        range: Range<usize>,
        original_len: usize,
    ) -> Range<usize> {
        let reaches_projected_end = range.end == self.source.len();
        let mut mapped = self.map_range(range, original_len);
        if reaches_projected_end {
            mapped.end = original_len;
        }
        mapped
    }

    pub(in crate::services::render) fn map_ranges(
        &self,
        ranges: Vec<Range<usize>>,
        original_len: usize,
    ) -> Vec<Range<usize>> {
        ranges
            .into_iter()
            .filter(|range| range.start < range.end)
            .map(|range| self.map_literal_range(range, original_len))
            .collect()
    }
}

impl ListPagesOriginalRangeCursor<'_> {
    pub(in crate::services::render) fn range_is_unchanged(
        &mut self,
        original: &str,
        range: Range<usize>,
    ) -> bool {
        let original = original.as_bytes();
        if range.start > range.end || range.end > original.len() {
            return false;
        }
        let original_offsets = &self.projection.original_offsets;
        while original_offsets
            .get(self.projected_cursor)
            .is_some_and(|offset| *offset < range.start)
        {
            self.projected_cursor += 1;
            self.advances += 1;
        }
        let projected_start = self.projected_cursor;
        while original_offsets
            .get(self.projected_cursor)
            .is_some_and(|offset| *offset < range.end)
        {
            self.projected_cursor += 1;
            self.advances += 1;
        }
        let projected_end = self.projected_cursor;
        let projected_offsets = &original_offsets[projected_start..projected_end];
        projected_offsets.len() == range.len()
            && projected_offsets.iter().copied().eq(range.clone())
            && self.projection.source.as_bytes()[projected_start..projected_end]
                == original[range]
    }

    pub(in crate::services::render) fn advances(&self) -> usize {
        self.advances
    }
}

fn append_projected_line_prefix(
    line: &str,
    original_start: usize,
    projected: &mut Vec<u8>,
    original_offsets: &mut Vec<usize>,
    changed: &mut bool,
) {
    let mut leading_nonstandard = true;
    for (relative, character) in line.char_indices() {
        let bytes = character.len_utf8();
        let original = original_start + relative;
        let project_nonstandard =
            leading_nonstandard && matches!(character, '\u{00a0}' | '\u{2007}');
        let project_nul = character == '\0';
        let project_tab = character == '\t';
        if project_tab {
            projected.extend(std::iter::repeat_n(b' ', 4));
            original_offsets.extend(std::iter::repeat_n(original, 4));
            *changed = true;
        } else if project_nonstandard || project_nul {
            projected.push(b' ');
            original_offsets.push(original);
            *changed = true;
        } else {
            projected.extend_from_slice(&line.as_bytes()[relative..relative + bytes]);
            original_offsets.extend(original..original + bytes);
        }
        leading_nonstandard &= project_nonstandard;
    }
}

fn collect_list_pages_quote_ranges(
    source: &str,
) -> (Vec<Range<usize>>, Vec<Range<usize>>) {
    let mut original = Vec::new();
    let mut compat = Vec::new();
    let mut offset = 0usize;
    for line in PhysicalLines::new(source) {
        let body = physical_line_body(line);
        if quote_depth_and_body(body).0 > 0 {
            let range = offset..offset + line.len();
            compat.push(range.clone());
            let trimmed = body.trim_start_matches([' ', '\t']);
            let depth = trimmed.bytes().take_while(|byte| *byte == b'>').count();
            if matches!(trimmed.as_bytes().get(depth), Some(b' ' | b'\t')) {
                original.push(range);
            }
        }
        offset += line.len();
    }
    (original, compat)
}

#[cfg(test)]
mod tests {
    use super::super::LiteralRegionIndex;

    #[test]
    fn unclosed_inline_literals_are_opaque_only_to_the_line_boundary() {
        for opener in ["@@", "@<", "[[$"] {
            let source = format!(
                "{opener}opaque [[module ListPages name=\"hidden\"]]\n\
                 [[module ListPages name=\"live\"]]body[[/module]]",
            );
            let index = LiteralRegionIndex::new_list_pages_syntax(&source);

            assert!(index.contains(source.find("hidden").unwrap()), "{opener}");
            assert!(
                !index.contains(source.rfind("[[module ListPages").unwrap()),
                "{opener}",
            );
        }
    }

    #[test]
    fn inline_collectors_own_other_runtime_delimiters() {
        for source in [
            "@<before @@ [!-- [[module ListPages name=\"hidden\"]] after>@ live",
            "[[$ x + [[module ListPages name=\"hidden\"]] + y $]] live",
        ] {
            let index = LiteralRegionIndex::new_list_pages_syntax(source);

            assert!(index.contains(source.find("[[module ListPages").unwrap()));
            assert!(!index.contains(source.find("live").unwrap()));
        }
    }

    #[test]
    fn inline_closers_embedded_in_other_tokens_do_not_end_literals() {
        for source in [
            "@@before https://example.test/a@@b [[module ListPages name=\"hidden\"]] @@",
            "[!-- https://example.test/a--]b [[module ListPages name=\"hidden\"]] --]",
            "[[$ https://example.test/a$]]b [[module ListPages name=\"hidden\"]] $]]",
            "@<before >>@ [[module ListPages name=\"hidden\"]] >@",
            "@<before ~~~>@ [[module ListPages name=\"hidden\"]] >@",
            "[!-- ---] [[module ListPages name=\"hidden\"]] --]",
        ] {
            let index = LiteralRegionIndex::new_list_pages_syntax(source);
            assert!(
                index.contains(source.find("[[module ListPages").unwrap()),
                "{source:?}",
            );
        }
    }

    #[test]
    fn overlapping_inline_math_close_is_line_bounded_without_panicking() {
        let source = concat!(
            "[[$]]\n",
            "[[module ListPages name=\"live\"]]body[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        assert!(index.contains(0));
        assert!(!index.contains(source.find("[[module ListPages").unwrap()));
    }

    #[test]
    fn covers_exact_runtime_literal_blocks_only() {
        for (open, close) in [
            ("[[code]]", "[[/code]]"),
            ("[[html]]", "[[/html]]"),
            ("[[raw]]", "[[/raw]]"),
            ("[[math theorem]]", "[[/math]]"),
            ("[[embed]]", "[[/embed]]"),
            ("[[module CSS show=\"head\"]]", "[[/module]]"),
            ("[[module css-reset]]", "[[/module]]"),
        ] {
            let source = format!(
                "{open} [[module ListPages name=\"hidden\"]] {close}\n\
                 [[module ListPages name=\"live\"]]body[[/module]]",
            );
            let index = LiteralRegionIndex::new_list_pages_syntax(&source);

            assert!(index.contains(source.find("hidden").unwrap()), "{open}");
            assert!(
                !index.contains(source.rfind("[[module ListPages").unwrap()),
                "{open}"
            );
        }

        for lookalike in [
            "[[codeexample]]",
            "[[html5]]",
            "[[raw-data]]",
            "[[mathref theorem]]",
            "[[embed youtube video=abc]]",
        ] {
            let source =
                format!("{lookalike} [[module ListPages name=\"live\"]]body[[/module]]",);
            let index = LiteralRegionIndex::new_list_pages_syntax(&source);

            assert!(
                !index.contains(source.find("[[module ListPages").unwrap()),
                "{lookalike}",
            );
        }
    }

    #[test]
    fn runtime_only_forms_do_not_change_the_common_index() {
        for source in [
            "@<[[module ListPages name=\"hidden\"]]>@",
            "[[$ [[module ListPages name=\"hidden\"]] $]]",
            "[[math]][[module ListPages name=\"hidden\"]][[/math]]",
            "[[embed]][[module ListPages name=\"hidden\"]][[/embed]]",
            "[[module CSS]][[module ListPages name=\"hidden\"]][[/module]]",
        ] {
            let offset = source.find("[[module ListPages").unwrap();
            assert!(!LiteralRegionIndex::new_wikidot_syntax(source).contains(offset));
            assert!(LiteralRegionIndex::new_list_pages_syntax(source).contains(offset));
        }
    }

    #[test]
    fn recognizes_a_second_literal_block_after_a_same_line_close() {
        let source = concat!(
            "[[code]]x[[/code]]",
            "[[raw]][[module ListPages name=\"hidden\"]][[/module]][[/raw]]",
            " [[module ListPages name=\"live\"]]body[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        assert!(index.contains(source.find("hidden").unwrap()));
        assert!(!index.contains(source.rfind("[[module ListPages").unwrap()));
    }

    #[test]
    fn raw_newline_head_starts_a_literal_block() {
        let source = concat!(
            "[[raw\n]]\n",
            "[[module ListPages name=\"hidden\"]]B[[/module]]\n",
            "[[/raw]]\n",
            "[[module ListPages name=\"live\"]]C[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        assert!(index.contains(source.find("hidden").unwrap()));
        assert!(!index.contains(source.find("live").unwrap()));
    }

    #[test]
    fn block_boundary_lookalikes_do_not_close_literal_blocks() {
        let source = concat!(
            "[[code]]\n",
            "[[[/code]]]\n",
            "[[module ListPages name=\"hidden-left\"]]B[[/module]]\n",
            "[[/code]]]\n",
            "[[module ListPages name=\"hidden-right\"]]C[[/module]]\n",
            "[[/code]]\n",
            "[[module ListPages name=\"live\"]]D[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        assert!(index.contains(source.find("hidden-left").unwrap()));
        assert!(index.contains(source.find("hidden-right").unwrap()));
        assert!(!index.contains(source.find("live").unwrap()));
    }

    #[test]
    fn math_value_heads_end_at_the_first_right_block_marker() {
        let source = concat!(
            "[[math \"label\" suffix]]\n",
            "[[module ListPages name=\"hidden\"]]B[[/module]]\n",
            "[[/math]]\n",
            "[[module ListPages name=\"live\"]]C[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        assert!(index.contains(source.find("hidden").unwrap()));
        assert!(!index.contains(source.find("live").unwrap()));
    }

    #[test]
    fn multiline_runtime_heads_preserve_literal_and_module_ownership() {
        for (open, close) in [
            ("[[code\ntype=\"rust\"]]", "[[/code]]"),
            ("[[html\nclass=\"x\"]]", "[[/html]]"),
            ("[[module CSS\nshow=\"head\"]]", "[[/module]]"),
        ] {
            let source = format!(
                "{open}\n[[module ListPages name=\"hidden\"]]B[[/module]]\n{close}\n\
                 [[module ListPages name=\"live\"]]C[[/module]]",
            );
            let index = LiteralRegionIndex::new_list_pages_syntax(&source);

            assert!(index.contains(source.find("hidden").unwrap()), "{open:?}");
            assert!(!index.contains(source.find("live").unwrap()), "{open:?}");
        }

        let source = concat!(
            "[[module ListPages\n",
            "name=\"live\" prependLine=\"@@ [!--\"]]B[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);
        assert!(!index.contains(source.find("[[module ListPages").unwrap()));
        assert!(!index.contains(source.find("[[/module]]").unwrap()));
    }

    #[test]
    fn tag_heads_do_not_open_literal_regions() {
        let source = concat!(
            "[[module ListPages name=\"x\" prependLine=\"@@\" comment=\"[!--\"]]X[[/module]]\n",
            "[[module ListPages name=\"quoted-end\" value=\"]] @@ [!--\"]]Q[[/module]]\n",
            "[[module ListPages name=\"live\"]]Y[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        for (offset, _) in source.match_indices("@@") {
            assert!(!index.contains(offset));
        }
        for (offset, _) in source.match_indices("[!--") {
            assert!(!index.contains(offset));
        }
        assert!(!index.contains(source.rfind("[[module ListPages").unwrap()));
    }

    #[test]
    fn tag_heads_follow_pinned_embedded_and_escaped_quote_rules() {
        let source = concat!(
            r#"[[module ListPages name="embedded" prependLine="the "literal ]] @@ [!-- value" suffix"]]C[[/module]]"#,
            "\n",
            r#"[[module ListPages name="escaped" prependLine="before \" @@ [!-- after" suffix="x"]]D[[/module]]"#,
            "\n[[module ListPages name=\"live\"]]E[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        for (offset, _) in source.match_indices("@@") {
            assert!(!index.contains(offset));
        }
        for (offset, _) in source.match_indices("[!--") {
            assert!(!index.contains(offset));
        }
        for (offset, _) in source.match_indices("[[/module]]") {
            assert!(!index.contains(offset));
        }
        assert!(!index.contains(source.rfind("[[module ListPages").unwrap()));
    }

    #[test]
    fn escaped_quote_outside_an_argument_does_not_consume_the_next_tag() {
        let source = concat!(
            r#"[[span data=x\" ]] "#,
            "[[module ListPages name=\"live\"]]B[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        assert!(!index.contains(source.find("[[module ListPages").unwrap()));
    }

    #[test]
    fn unclosed_tag_is_opaque_only_through_its_physical_line() {
        let source = concat!(
            "[[module ListPages prependLine=\"@@ [!-- @<\n",
            "[[module ListPages name=\"live\"]]Y[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        assert!(index.contains(0));
        assert!(!index.contains(source.rfind("[[module ListPages").unwrap()));
    }

    #[test]
    fn css_module_skips_common_literal_false_closers() {
        let source = concat!(
            "[[module CSS]]\n",
            "[!-- [[/module]] --]\n",
            "@@[[/module]]@@\n",
            "[[module ListPages name=\"hidden\"]]body[[/module]]\n",
            "[[/module]]\n",
            "[[module ListPages name=\"live\"]]body[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        assert!(index.contains(source.find("hidden").unwrap()));
        assert!(!index.contains(source.rfind("[[module ListPages").unwrap()));
    }

    #[test]
    fn css_module_unions_downstream_and_pinned_parser_ownership() {
        // Keep the downstream CSS-reset contract while adding pinned-valid module spellings.
        let source = concat!(
            "[[module\nCSS-reset anything]]\n",
            "[[ /module]]\n",
            "[[module ListPages name=\"hidden\"]]B[[/module]]\n",
            "[[/module]]\n",
            "[[module ListPages name=\"live\"]]C[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        assert!(index.contains(source.find("hidden").unwrap()));
        assert!(!index.contains(source.find("live").unwrap()));

        for pinned_open in ["[[module654 CSS]]", "[[ module CSS]]"] {
            let source = format!(
                "{pinned_open}\n[[module ListPages name=\"owned\"]]C[[/module]]",
            );
            let index = LiteralRegionIndex::new_list_pages_syntax(&source);
            assert!(
                index.contains(source.find("owned").unwrap()),
                "{pinned_open:?}"
            );
        }
    }

    #[test]
    fn css_quote_math_and_tag_attribute_closers_match_the_current_extractor() {
        for false_owner in [
            "> [[/module]]",
            "[[$ [[/module]] $]]",
            "[[span value=\"[[/module]]\"]]",
        ] {
            let source = format!(
                "[[module CSS]]\n{false_owner}\n\
                 [[module ListPages name=\"live\"]]C[[/module]]",
            );
            let index = LiteralRegionIndex::new_list_pages_syntax(&source);
            assert!(
                !index.contains(source.find("live").unwrap()),
                "{false_owner:?}"
            );
        }
    }

    #[test]
    fn unclosed_css_open_does_not_create_an_extracted_span() {
        let source = concat!("[[module CSS]]\n", "[[module ListPages name=\"live\"]]C",);
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        assert!(!index.contains(source.find("live").unwrap()));
    }

    #[test]
    fn line_continuations_preserve_runtime_literal_ownership() {
        for continuation in ["\\\n", "\\\r\n", "\\\r"] {
            let source = format!(
                "@@before{continuation}[[module ListPages name=\"hidden\"]]@@\n\
                 [[module ListPages name=\"live\"]]body[[/module]]",
            );
            let index = LiteralRegionIndex::new_list_pages_syntax(&source);

            assert!(
                index.contains(source.find("hidden").unwrap()),
                "{continuation:?}"
            );
            assert!(
                !index.contains(source.rfind("[[module ListPages").unwrap()),
                "{continuation:?}",
            );
        }
    }

    #[test]
    fn source_projection_maps_structural_event_boundaries_to_original_offsets() {
        let source = "prefix\\\r\n[[module\0CSS]]";
        let projection = super::ListPagesSourceProjection::new(source).unwrap();
        let projected = projection.source();
        let start = projected.find("[[module CSS]]").unwrap();
        let range = start..start + "[[module CSS]]".len();

        assert_eq!(projected, "prefix[[module CSS]]");
        assert_eq!(
            projection.map_range(range, source.len()),
            source.find("[[module").unwrap()..source.len(),
        );
    }

    #[test]
    fn source_projection_matches_tab_expansion_and_preserves_offsets() {
        let source = "https://e.test/a\t@@raw@@";
        let projection = super::ListPagesSourceProjection::new(source).unwrap();
        let projected = projection.source();
        let raw_start = projected.find("@@raw@@").unwrap();

        assert_eq!(projected, "https://e.test/a    @@raw@@");
        assert_eq!(
            projection.map_range(raw_start..raw_start + 2, source.len()),
            source.find("@@").unwrap()..source.find("@@").unwrap() + 2,
        );

        let source = concat!(
            "https://e.test/a\t@@[[module ListPages name=\"hidden\"]]@@\n",
            "[[module ListPages name=\"live\"]]body[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);
        assert!(index.contains(source.find("hidden").unwrap()));
        assert!(!index.contains(source.find("live").unwrap()));
    }

    #[test]
    fn leading_nonstandard_spaces_project_one_scalar_to_one_space() {
        let source = "\u{00a0}\u{2007}\u{00a0}> text";
        let projection = super::ListPagesSourceProjection::new(source).unwrap();

        assert_eq!(projection.source(), "   > text");
        assert_eq!(projection.original_offsets.len(), projection.source().len());
        assert_eq!(
            projection.map_range(3..projection.source().len(), source.len()),
            source.find('>').unwrap()..source.len(),
        );
    }

    #[test]
    fn structural_and_literal_ranges_distinguish_deleted_eof_suffixes() {
        for suffix in ["\n", "\r\n", "\r", "\\\n"] {
            let source = format!("[[/module]]{suffix}");
            let projection = super::ListPagesSourceProjection::new(&source).unwrap();
            let range = 0.."[[/module]]".len();

            assert_eq!(projection.source(), "[[/module]]", "{suffix:?}");
            assert_eq!(
                projection.map_range(range.clone(), source.len()),
                0.."[[/module]]".len(),
                "{suffix:?}",
            );
            assert_eq!(
                projection.map_literal_range(range, source.len()),
                0..source.len(),
                "{suffix:?}",
            );
        }
    }

    #[test]
    fn normalized_continuations_replace_physical_literal_ownership() {
        for source in [
            "@@before\\\n[[module ListPages name=\"hidden\"]]@@",
            "> \\\n[[module ListPages name=\"hidden\"]]body[[/module]]",
        ] {
            let index = LiteralRegionIndex::new_list_pages_syntax(source);
            assert!(
                index.contains(source.find("[[module ListPages").unwrap()),
                "{source:?}",
            );
        }

        for source in [
            "@@\\\n@@ [[module ListPages name=\"live\"]]body[[/module]]",
            "prefix\\\n> [[module ListPages name=\"live\"]]body[[/module]]",
        ] {
            let index = LiteralRegionIndex::new_list_pages_syntax(source);
            assert!(
                !index.contains(source.find("[[module ListPages").unwrap()),
                "{source:?}",
            );
        }
    }

    #[test]
    fn cascading_continuations_match_the_ftml_stack_normalizer() {
        let source = concat!(
            "@@before\\\\\n\n",
            "[[module ListPages name=\"hidden\"]]@@\n",
            "[[module ListPages name=\"live\"]]body[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        assert!(index.contains(source.find("hidden").unwrap()));
        assert!(!index.contains(source.rfind("[[module ListPages").unwrap()));
    }

    #[test]
    fn blank_line_compression_precedes_cascading_continuations() {
        let source = concat!(
            "@@before\\\\\n\n\n",
            "[[module ListPages name=\"hidden\"]]@@\n",
            "[[module ListPages name=\"live\"]]body[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        assert!(index.contains(source.find("hidden").unwrap()));
        assert!(!index.contains(source.find("live").unwrap()));
    }

    #[test]
    fn whitespace_only_lines_are_stripped_before_cascading_continuations() {
        let source = concat!(
            "@@before\\\\\n   \n",
            "[[module ListPages name=\"hidden\"]]@@\n",
            "[[module ListPages name=\"live\"]]body[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        assert!(index.contains(source.find("hidden").unwrap()));
        assert!(!index.contains(source.find("live").unwrap()));
    }

    #[test]
    fn unicode_whitespace_only_lines_expose_cascading_continuations() {
        for whitespace in ["\u{2003}", "\u{000b}", "\u{000c}"] {
            let source = format!(
                "@@before\\\\\n{whitespace}\n\
                 [[module ListPages name=\"hidden\"]]@@\n\
                 [[module ListPages name=\"live\"]]body[[/module]]",
            );
            let index = LiteralRegionIndex::new_list_pages_syntax(&source);

            assert!(
                index.contains(source.find("hidden").unwrap()),
                "{whitespace:?}"
            );
            assert!(
                !index.contains(source.find("live").unwrap()),
                "{whitespace:?}"
            );
        }
    }

    #[test]
    fn bare_carriage_return_terminates_inline_raw() {
        let source = "@@before\r[[module ListPages name=\"live\"]]body[[/module]]@@";
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        assert!(!index.contains(source.find("[[module ListPages").unwrap()));
    }

    #[test]
    fn suppresses_all_quote_prefixed_physical_lines() {
        let source = concat!(
            "> [[module ListPages name=\"spaced\"]]\n",
            ">>[[module ListPages name=\"tight\"]]\n",
            "> > [[module ListPages name=\"split\"]]\n",
            "  > [[module ListPages name=\"indented\"]]\n",
            "[[module ListPages name=\"live\"]]body[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);
        let offsets = source
            .match_indices("[[module ListPages")
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();

        for offset in &offsets[..4] {
            assert!(index.contains(*offset));
        }
        assert!(!index.contains(*offsets.last().unwrap()));
    }

    #[test]
    fn color_children_use_the_final_quote_candidate_universe() {
        let source = concat!(
            "##red|\n",
            "> [[module ListPages name=\"hidden\"]]B[[/module]]\n",
            "##\n",
            "[[module ListPages name=\"live\"]]C[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        assert!(index.contains(source.find("hidden").unwrap()));
        assert!(!index.contains(source.find("live").unwrap()));
    }

    #[test]
    fn runtime_module_heads_remain_visible_to_the_structural_scanner() {
        for source in [
            "[[module ListPages limit=1]]x[[/module]]",
            "[[module CountPages tags=\"+x\"]]x[[/module]]",
        ] {
            let index = LiteralRegionIndex::new_list_pages_syntax(source);
            assert!(!index.contains(0), "{source:?}: {:?}", index.ranges);
        }

        let count = "[[module CountPages tags=\"+x\"]]x[[/module]]";
        let index = LiteralRegionIndex::new_count_pages_syntax(count);
        assert!(!index.contains(0), "{count:?}: {:?}", index.ranges);
    }

    #[test]
    fn projects_pinned_line_leading_space_rewrites_for_quote_ownership() {
        let source = " \t> [[module ListPages name=\"hidden\"]]body[[/module]]";
        let index = LiteralRegionIndex::new_list_pages_syntax(source);
        assert!(index.contains(source.find("hidden").unwrap()));

        for prefix in ["  ", "\u{00a0}", "\u{2007}", "\0", " \0"] {
            let source = format!(
                "prefix\n{prefix}> [[module ListPages name=\"live\"]]body[[/module]]",
            );
            let index = LiteralRegionIndex::new_list_pages_syntax(&source);

            assert!(index.contains(source.find("live").unwrap()), "{prefix:?}");
        }
    }

    #[test]
    fn global_nul_projection_changes_only_literal_ownership() {
        let source = concat!(
            "@@before\0[[module ListPages name=\"hidden\"]]@@\n",
            "[[module\0CSS]][[module ListPages name=\"css-hidden\"]]C[[/module]][[/module]]\n",
            "[[module\0ListPages name=\"not-a-candidate\"]]B[[/module]]",
        );
        let index = LiteralRegionIndex::new_list_pages_syntax(source);

        assert!(index.contains(source.find("hidden").unwrap()));
        assert!(index.contains(source.find("css-hidden").unwrap()));
        assert!(!index.contains(source.find("not-a-candidate").unwrap()));
    }
}
