/*
 * services/render/literal_regions/count_pages.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use super::list_pages_protection::collect_count_pages_inherited_ranges;
use super::merge_sorted_ranges;
use super::token_boundaries::{
    TextTokenCursor, WikidotTagScan, left_block_start_in_run, scan_wikidot_tag,
    wikidot_trimmed_name,
};
use std::ops::Range;

pub(super) fn collect_count_pages_literal_ranges(source: &str) -> Vec<Range<usize>> {
    let block_ranges = collect_line_prefix_block_ranges(source);
    let raw_ranges = collect_alternating_ranges(source, "@@");
    let comment_ranges = collect_comment_ranges(source);
    let legacy_ranges = merge_sorted_ranges(
        merge_sorted_ranges(block_ranges, raw_ranges),
        comment_ranges,
    );
    let list_pages_ranges = collect_count_pages_inherited_ranges(source);
    let generic_tag_head_ranges = collect_generic_tag_head_ranges(source);
    let parser_ranges =
        select_list_pages_precedence(list_pages_ranges, generic_tag_head_ranges);
    merge_sorted_ranges(legacy_ranges, parser_ranges)
}

fn select_list_pages_precedence(
    list_pages_ranges: Vec<Range<usize>>,
    generic_tag_head_ranges: Vec<Range<usize>>,
) -> Vec<Range<usize>> {
    let mut list_pages_ranges = list_pages_ranges.into_iter().peekable();
    let mut generic_tag_head_ranges = generic_tag_head_ranges.into_iter().peekable();
    let mut selected = Vec::new();

    while list_pages_ranges.peek().is_some() || generic_tag_head_ranges.peek().is_some() {
        let take_list_pages =
            match (list_pages_ranges.peek(), generic_tag_head_ranges.peek()) {
                (Some(list_pages), Some(generic)) => list_pages.start <= generic.start,
                (Some(_), None) => true,
                _ => false,
            };
        let range = if take_list_pages {
            list_pages_ranges
                .next()
                .expect("ListPages range should exist")
        } else {
            generic_tag_head_ranges
                .next()
                .expect("generic tag-head range should exist")
        };
        if selected
            .last()
            .is_none_or(|previous: &Range<usize>| previous.end <= range.start)
        {
            selected.push(range);
        }
    }
    selected
}

fn collect_generic_tag_head_ranges(source: &str) -> Vec<Range<usize>> {
    let bytes = source.as_bytes();
    let mut ranges = Vec::new();
    let mut text_tokens = TextTokenCursor::new(source);
    let mut cursor = 0usize;

    while let Some(relative) = source[cursor..].find("[[") {
        let candidate = cursor + relative;
        let (block_start, run_end) = left_block_start_in_run(bytes, candidate);
        let Some(start) = block_start else {
            cursor = run_end;
            continue;
        };
        let mut lookahead_tokens = text_tokens.clone();
        match scan_wikidot_tag(
            bytes,
            start,
            bytes.len(),
            true,
            true,
            &mut lookahead_tokens,
        ) {
            WikidotTagScan::Complete(end) => {
                if !is_structural_module_tag(source, start) {
                    ranges.push(start..end);
                }
                text_tokens = lookahead_tokens;
                cursor = end;
            }
            WikidotTagScan::Malformed { resume } => cursor = resume,
            WikidotTagScan::Unclosed => break,
        }
    }
    ranges
}

fn is_structural_module_tag(source: &str, start: usize) -> bool {
    let bytes = source.as_bytes();
    let mut cursor = start + 2;
    if bytes.get(cursor) == Some(&b'/') {
        cursor += 1;
    }
    while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
        cursor += 1;
    }
    if bytes.get(cursor..cursor + 2) == Some(&b"[["[..]) {
        cursor += 2;
        while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
            cursor += 1;
        }
    }
    let (Some(name), name_end) = wikidot_trimmed_name(bytes, cursor) else {
        return false;
    };
    let name = name.strip_suffix(b"_").unwrap_or(name);
    if name.eq_ignore_ascii_case(b"module") {
        return true;
    }
    if !name.eq_ignore_ascii_case(b"module654") {
        return false;
    }
    let mut subname_start = name_end;
    while matches!(bytes.get(subname_start), Some(b' ' | b'\t' | b'\n' | b'\r')) {
        subname_start += 1;
    }
    let (subname, _) = wikidot_trimmed_name(bytes, subname_start);
    !subname.is_some_and(|subname| subname.eq_ignore_ascii_case(b"CountPages"))
}

fn collect_line_prefix_block_ranges(source: &str) -> Vec<Range<usize>> {
    let mut code_ranges = Vec::new();
    let mut html_ranges = Vec::new();
    let mut code_start = None;
    let mut html_start = None;
    let mut offset = 0usize;

    for line in source.split_inclusive('\n') {
        let marker = line.trim_start();
        let marker_start = offset + line.len() - marker.len();
        let marker = marker.to_ascii_lowercase();

        if marker.starts_with("[[code") {
            code_start.get_or_insert(marker_start);
        } else if marker.starts_with("[[/code]]")
            && let Some(start) = code_start.take()
        {
            code_ranges.push(start..marker_start + "[[/code]]".len());
        }

        if marker.starts_with("[[html") {
            html_start.get_or_insert(marker_start);
        } else if marker.starts_with("[[/html]]")
            && let Some(start) = html_start.take()
        {
            html_ranges.push(start..marker_start + "[[/html]]".len());
        }

        offset += line.len();
    }

    if let Some(start) = code_start {
        code_ranges.push(start..source.len());
    }
    if let Some(start) = html_start {
        html_ranges.push(start..source.len());
    }
    merge_sorted_ranges(code_ranges, html_ranges)
}

fn collect_alternating_ranges(source: &str, delimiter: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut open = None;
    let mut cursor = 0usize;
    while let Some(relative) = source[cursor..].find(delimiter) {
        let start = cursor + relative;
        cursor = start + delimiter.len();
        if let Some(open) = open.take() {
            ranges.push(open..cursor);
        } else {
            open = Some(start);
        }
    }
    if let Some(open) = open {
        ranges.push(open..source.len());
    }
    ranges
}

fn collect_comment_ranges(source: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut open = None;
    let bytes = source.as_bytes();
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        if bytes[cursor..].starts_with(b"[!--")
            && bytes.get(cursor + b"[!--".len()) != Some(&b']')
        {
            open.get_or_insert(cursor);
        } else if bytes[cursor..].starts_with(b"--]")
            && let Some(start) = open.take()
        {
            ranges.push(start..cursor + "--]".len());
        }
        cursor += 1;
    }

    if let Some(open) = open {
        ranges.push(open..source.len());
    }
    ranges
}

#[cfg(test)]
mod tests {
    use super::super::LiteralRegionIndex;

    #[test]
    fn preserves_legacy_fail_closed_count_pages_prefixes() {
        for opener in ["[[code malformed", "[[codec]]", "[[html malformed"] {
            let source =
                format!("{opener}\n[[module CountPages tags=\"+hidden\"]]H[[/module]]");
            let module = source.find("[[module CountPages").unwrap();
            assert!(
                LiteralRegionIndex::new_count_pages_syntax(&source).contains(module),
                "{opener}",
            );
        }
    }

    #[test]
    fn preserves_legacy_raw_and_comment_prefix_state() {
        for source in [
            "@@ [[module CountPages tags=\"+hidden\"]]H[[/module]]",
            "[!-- [[module CountPages tags=\"+hidden\"]]H[[/module]]",
        ] {
            let module = source.find("[[module CountPages").unwrap();
            assert!(
                LiteralRegionIndex::new_count_pages_syntax(source).contains(module),
                "{source}",
            );
        }

        let source = "@@@@ [[module CountPages tags=\"+live\"]]L[[/module]]";
        let module = source.find("[[module CountPages").unwrap();
        assert!(!LiteralRegionIndex::new_count_pages_syntax(source).contains(module));

        let source = "[!--] [[module CountPages tags=\"+live\"]]L[[/module]]";
        let module = source.find("[[module CountPages").unwrap();
        assert!(!LiteralRegionIndex::new_count_pages_syntax(source).contains(module));
    }

    #[test]
    fn count_pages_inherits_list_pages_text_and_css_ownership() {
        for source in [
            "[https://example.test [[module CountPages tags=\"+hidden\"]]H[[/module]] label]",
            "[#toc [[module CountPages tags=\"+hidden\"]]H[[/module]] label]",
            "[[[target [[module CountPages tags=\"+hidden\"]]H[[/module]] suffix]]]",
            "##rgb([[module CountPages tags=\"+hidden\"]])|body##",
            "[[ module CSS]]\n[[module CountPages tags=\"+hidden\"]]H[[/module]]",
        ] {
            let module = source.find("[[module CountPages").unwrap();
            assert!(
                LiteralRegionIndex::new_count_pages_syntax(source).contains(module),
                "{source}",
            );
        }

        let source = "##red|body [[module CountPages tags=\"+live\"]]L[[/module]]##";
        let module = source.find("[[module CountPages").unwrap();
        assert!(!LiteralRegionIndex::new_count_pages_syntax(source).contains(module));
    }

    #[test]
    fn generic_heads_hide_nested_count_pages_without_hiding_the_target() {
        let target = "[[module CountPages tags=\"+live\"]]L[[/module]]";
        assert!(!LiteralRegionIndex::new_count_pages_syntax(target).contains(0));

        let legacy_name = "[[module654 CountPages tags=\"+text\"]]";
        assert!(LiteralRegionIndex::new_count_pages_syntax(legacy_name).contains(0));

        let source = "[[div data=\"[[module CountPages tags='+hidden']]\"]]body[[/div]]";
        let module = source.find("[[module CountPages").unwrap();
        assert!(LiteralRegionIndex::new_count_pages_syntax(source).contains(module));
    }
}
