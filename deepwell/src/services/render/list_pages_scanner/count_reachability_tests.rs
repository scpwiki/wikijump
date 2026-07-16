/*
 * services/render/list_pages_scanner/count_reachability_tests.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use super::CountPagesCloseReachabilityIndex;

fn first_regex_shaped_capture(source: &str) -> std::ops::Range<usize> {
    let close = source
        .find("[[/module]]")
        .expect("fixture contains a textual close")
        + "[[/module]]".len();
    0..close
}

#[test]
fn literal_and_nested_textual_closes_are_not_structurally_reachable() {
    for source in [
        "[[module CountPages tags=\"+x\"]]A@@[[/module]]@@B[[/module]]",
        "[[module CountPages tags=\"+x\"]]A[!--[[/module]]--]B[[/module]]",
        "[[module CountPages tags=\"+x\"]][[module Rate]][[/module]][[/module]]",
        "[[module CountPages tags=\"+x\"]][[module CountPages]][[/module]][[/module]]",
    ] {
        let index = CountPagesCloseReachabilityIndex::new(source);
        let mut cursor = index.monotone_cursor();
        assert!(
            !cursor.regex_capture_close_is_reachable(first_regex_shaped_capture(source)),
            "{source:?}",
        );
    }
}

#[test]
fn the_selected_outer_close_is_reachable() {
    let source = "[[module CountPages tags=\"+x\"]]body[[/module]]";
    let index = CountPagesCloseReachabilityIndex::new(source);
    let mut cursor = index.monotone_cursor();
    assert!(cursor.regex_capture_close_is_reachable(0..source.len()));
}

#[test]
fn count_module_inside_a_literal_owner_is_not_reachable() {
    let source = "@@[[module CountPages]]body[[/module]]@@";
    let start = source.find("[[module").unwrap();
    let end = source.find("[[/module]]").unwrap() + "[[/module]]".len();
    let index = CountPagesCloseReachabilityIndex::new(source);
    let mut cursor = index.monotone_cursor();
    assert!(!cursor.regex_capture_close_is_reachable(start..end));
}

#[test]
fn literal_captures_do_not_hide_a_later_reachable_count_module() {
    let hidden = "@@[[module CountPages]]hidden[[/module]]@@";
    let live = "[[module CountPages tags=\"+x\"]]live[[/module]]";
    let source = format!("{hidden}\n{live}");
    let hidden_start = source.find("[[module").unwrap();
    let hidden_end = source.find("[[/module]]").unwrap() + "[[/module]]".len();
    let live_start = source.rfind("[[module").unwrap();
    let live_end = source.rfind("[[/module]]").unwrap() + "[[/module]]".len();

    let index = CountPagesCloseReachabilityIndex::new(&source);
    let mut cursor = index.monotone_cursor();
    assert!(!cursor.regex_capture_close_is_reachable(hidden_start..hidden_end));
    assert!(cursor.regex_capture_close_is_reachable(live_start..live_end));
}

#[test]
fn many_capture_queries_advance_once() {
    let source = "[[module CountPages]][[/module]]\n".repeat(4_096);
    let index = CountPagesCloseReachabilityIndex::new(&source);
    let mut cursor = index.monotone_cursor();
    let mut offset = 0usize;
    for line in source.split_inclusive('\n') {
        let end = offset + line.trim_end_matches('\n').len();
        assert!(cursor.regex_capture_close_is_reachable(offset..end));
        offset += line.len();
    }
    assert!(cursor.advances() <= 4_096);
}
