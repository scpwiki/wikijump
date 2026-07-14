use super::super::*;

#[test]
fn list_pages_event_scanner_keeps_monotone_cursor_work_bounded_on_deep_nesting() {
    const DEPTH: usize = 20_000;
    let mut source = String::from("[[module ListPages name=\"outer\"]]before");
    for _ in 0..DEPTH {
        source.push_str("[[module ListUsers]]");
    }
    source.push_str("center");
    for _ in 0..DEPTH {
        source.push_str("[[/module]]");
    }
    source.push_str("after[[/module]]");
    source.push_str("[[module ListPages name=\"second\"]]B[[/module]]");

    let (modules, monotone_cursor_work, literal_range_advances) =
        find_list_pages_module_matches_with_cursor_work(&source);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"outer\"");
    assert!(modules[0].body.starts_with("before"));
    assert!(modules[0].body.ends_with("after"));
    assert_eq!(modules[1].head, "name=\"second\"");
    assert_eq!(literal_range_advances, 0);
    assert!(monotone_cursor_work <= source.len() * 4);
}

#[test]
fn list_pages_event_scanner_advances_each_literal_range_once() {
    const REGIONS: usize = 5_000;
    let mut source = String::new();
    for _ in 0..REGIONS {
        source.push_str("@@[[module ListPages name=\"fake\"]]X[[/module]]@@ separator ");
    }
    source.push_str("[[module ListPages name=\"live\"]]kept[[/module]]");

    let (modules, monotone_cursor_work, literal_range_advances) =
        find_list_pages_module_matches_with_cursor_work(&source);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
    assert_eq!(literal_range_advances, REGIONS);
    assert!(monotone_cursor_work <= source.len() * 2);
}

#[test]
fn list_pages_event_scanner_keeps_cursor_work_bounded_for_dense_tag_heads() {
    const HEADS: usize = 5_000;
    let mut source = String::new();
    for _ in 0..HEADS {
        source
            .push_str("[[span title=\"[[module ListPages name='fake']]X[[/module]]\"]] ");
    }
    source.push_str("[[module ListPages name=\"live\"]]kept[[/module]]");

    let (modules, monotone_cursor_work, literal_range_advances) =
        find_list_pages_module_matches_with_cursor_work(&source);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
    assert_eq!(literal_range_advances, 0);
    assert!(monotone_cursor_work <= source.len() * 7);
}

#[test]
fn list_pages_event_scanner_keeps_projected_event_merge_work_linear() {
    const MODULES: usize = 5_000;
    let mut source = String::from("[[module ListPages name=\"outer\"]]before");
    for _ in 0..MODULES {
        source.push_str("[[module\0Foo]]body[[/module]]");
    }
    source.push_str("after[[/module]]");

    let (modules, monotone_cursor_work, literal_range_advances) =
        find_list_pages_module_matches_with_cursor_work(&source);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"outer\"");
    assert_eq!(literal_range_advances, 0);
    assert!(monotone_cursor_work <= source.len() * MAX_PROJECTED_SCANNER_WORK_MULTIPLIER);
}

#[test]
fn dense_left_bracket_runs_are_scanned_as_one_token_run() {
    const BRACKETS: usize = 50_000;
    let value = "[".repeat(BRACKETS);
    let source = format!("[[module ListPages name=\"{value}\"]]body[[/module]]",);

    let (modules, monotone_cursor_work, literal_range_advances) =
        find_list_pages_module_matches_with_cursor_work(&source);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].body, "body");
    assert_eq!(literal_range_advances, 0);
    assert!(monotone_cursor_work <= source.len() * 8);
}

#[test]
fn malformed_list_pages_heads_fail_closed_without_suffix_rescans() {
    const HEADS: usize = 5_000;
    for fragment in [
        "[[module ListPages name=\"unterminated\n",
        "[[module ListPages name=foo[[aaaa",
    ] {
        let source = fragment.repeat(HEADS);
        let (modules, monotone_cursor_work, literal_range_advances) =
            find_list_pages_module_matches_with_cursor_work(&source);
        assert!(modules.is_empty(), "{fragment:?}");
        assert_eq!(literal_range_advances, 0, "{fragment:?}");
        assert!(monotone_cursor_work <= source.len() * 5, "{fragment:?}");
    }
}

#[test]
fn malformed_list_pages_heads_do_not_rescan_suffixes() {
    const HEADS: usize = 4_096;
    for (label, fragment) in [
        ("quoted newline", "[[module ListPages name=\"unterminated\n"),
        (
            "unquoted nested opener",
            "[[module ListPages name=foo[[aaaa ",
        ),
    ] {
        let source = fragment.repeat(HEADS);
        take_module_head_scan_bytes();

        let modules = find_list_pages_module_matches(&source);
        let head_scan_bytes = take_module_head_scan_bytes();

        assert!(modules.is_empty(), "{label}");
        assert!(
            head_scan_bytes >= HEADS * 10,
            "{label}: module head scanner did not examine every malformed prefix ({head_scan_bytes} bytes)",
        );
        assert!(
            head_scan_bytes <= source.len() * 2,
            "{label}: examined {head_scan_bytes} head bytes for {} source bytes",
            source.len(),
        );
    }
}

#[test]
fn projected_direct_head_checks_advance_offsets_once() {
    const MODULES: usize = 4_096;
    let source = format!(
        "\n{}",
        "[[module ListPages name=\"x\"]]B[[/module]]".repeat(MODULES),
    );
    take_projection_offset_advances();

    let (modules, monotone_cursor_work, literal_range_advances) =
        find_list_pages_module_matches_with_cursor_work(&source);
    let offset_advances = take_projection_offset_advances();

    assert_eq!(modules.len(), MODULES);
    assert_eq!(literal_range_advances, 0);
    assert!(offset_advances >= MODULES);
    assert!(offset_advances <= source.len());
    assert!(monotone_cursor_work <= source.len() * MAX_PROJECTED_SCANNER_WORK_MULTIPLIER);
}

#[test]
fn deferred_nested_head_rollbacks_exhaust_a_linear_work_budget() {
    const HEADS: usize = 4_096;
    for (label, source) in [
        ("whole head EOF", "[[user listpages ".repeat(HEADS)),
        (
            "whole head newline",
            format!("{}\n", "[[user listpages ".repeat(HEADS)),
        ),
        ("generic quote EOF", "[[span a=\"x listpages ".repeat(HEADS)),
        (
            "generic module newline",
            format!("{}\n", "[[module Foo title=\"listpages ".repeat(HEADS),),
        ),
        (
            "generic deferred close",
            format!("{}\"]]", "[[span bad=foo a=\"x listpages ".repeat(HEADS),),
        ),
    ] {
        let (modules, work, literal_range_advances) =
            find_list_pages_module_matches_with_cursor_work(&source);
        assert!(modules.is_empty(), "{label}");
        assert_eq!(literal_range_advances, 0, "{label}");
        assert!(work > source.len(), "{label}: work counter stayed vacuous");
        assert!(
            work <= source.len() * MAX_SINGLE_SCANNER_WORK_MULTIPLIER,
            "{label}: {work} work for {} source bytes",
            source.len(),
        );
    }
}
