/*
 * services/render/literal_regions/anchor_candidates.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use super::token_boundaries::{
    TextTokenIndex, left_block_start_in_run, right_bracket_token,
};
use std::ops::Range;

/// Enumerate complete pinned `[[# name]]` anchors without masking siblings.
#[cfg(test)]
pub(super) fn collect_pinned_anchor_candidates(source: &str) -> Vec<Range<usize>> {
    let text_tokens = TextTokenIndex::new(source);
    collect_pinned_anchor_candidates_with_text_tokens(source, &text_tokens)
}

pub(super) fn collect_pinned_anchor_candidates_with_text_tokens(
    source: &str,
    text_tokens: &TextTokenIndex,
) -> Vec<Range<usize>> {
    let bytes = source.as_bytes();
    let mut text_tokens = text_tokens.cursor();
    let mut output = Vec::new();
    let mut cursor = 0usize;
    while let Some(relative) = source[cursor..].find("[[#") {
        let candidate = cursor + relative;
        let (block_start, run_end) = left_block_start_in_run(bytes, candidate);
        cursor = candidate + 1;
        if block_start != Some(candidate) || text_tokens.contains(candidate) {
            cursor = cursor.max(run_end);
            continue;
        }

        let mut scan = candidate + 3;
        if !matches!(bytes.get(scan), Some(b' ' | b'\t')) {
            continue;
        }
        while matches!(bytes.get(scan), Some(b' ' | b'\t')) {
            scan += 1;
        }
        let name_start = scan;
        while scan < bytes.len() {
            match bytes[scan] {
                b' ' | b'\t' | b'\n' | b'\r' => break,
                b']' => {
                    let (right_block, token_len) =
                        right_bracket_token(bytes, scan, bytes.len());
                    if right_block && name_start < scan {
                        output.push(candidate..scan + token_len);
                        cursor = candidate + 3;
                        break;
                    }
                    scan += token_len;
                }
                _ => {
                    scan += source[scan..]
                        .chars()
                        .next()
                        .expect("scan is before the UTF-8 source end")
                        .len_utf8();
                }
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::collect_pinned_anchor_candidates;

    #[test]
    fn accepts_only_complete_pinned_anchor_shape() {
        assert_eq!(collect_pinned_anchor_candidates("[[# name]]"), vec![0..10]);
        for source in [
            "[[#name]]",
            "[[# ]]",
            "[[# two names]]",
            "[[# name\n]]",
            "[[[# name]]]",
        ] {
            assert!(
                collect_pinned_anchor_candidates(source).is_empty(),
                "{source:?}",
            );
        }
    }

    #[test]
    fn projected_line_join_does_not_broaden_anchor_grammar() {
        let source = "[[# foo\\\n [[module ListPages name=\"live\"]]X[[/module]]]]";
        assert!(collect_pinned_anchor_candidates(source).is_empty());
    }
}
