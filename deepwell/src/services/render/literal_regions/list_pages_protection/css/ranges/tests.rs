/*
 * services/render/literal_regions/list_pages_protection/css/ranges/tests.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use super::super::super::super::LiteralRegionIndex;
use super::collect_downstream_css_module_ranges;

#[test]
fn tight_quote_css_uses_original_downstream_ownership() {
    let source = concat!(
        ">[[module CSS]]\n",
        "[[module ListPages name=\"hidden\"]]B\n",
        "[[/module]]\n",
        "[[module ListPages name=\"live\"]]C[[/module]]",
    );

    assert_css_range_owns_hidden_only(source);
}

#[test]
fn unicode_casefolded_css_uses_the_downstream_regex_contract() {
    let source = concat!(
        "[[module CſS]]\n",
        "[[module ListPages name=\"hidden\"]]B\n",
        "[[/module]]\n",
        "[[module ListPages name=\"live\"]]C[[/module]]",
    );

    assert_css_range_owns_hidden_only(source);
}

#[test]
fn projection_created_quotes_do_not_erase_original_css_ownership() {
    for prefix in ["> \\\n", ">\0"] {
        let source = format!(
            "{prefix}[[module CSS]]\n\
             [[module ListPages name=\"hidden\"]]B\n\
             [[/module]]\n\
             [[module ListPages name=\"live\"]]C[[/module]]",
        );

        assert_css_range_owns_hidden_only(&source);
    }
}

#[test]
fn pinned_css_heads_with_quoted_brackets_and_spacing_own_runtime_modules() {
    for opener in [
        r#"[[module CSS note="x]y"]]"#,
        "[[ module CSS]]",
        "[[module654 CSS]]",
    ] {
        let source = format!(
            "{opener}\n\
             [[module ListPages name=\"hidden-list\"]]X\n\
             [[module CountPages category=\"hidden-count\"]]\n\
             [[/module]]\n\
             [[module ListPages name=\"live\"]]C[[/module]]",
        );
        assert_runtime_modules_are_owned_until_live(&source);
    }
}

#[test]
fn pinned_css_closer_variants_own_runtime_modules() {
    for closer in [
        "[[/ module]]",
        "[[/module ]]",
        "[[/module654]]",
        "[[/[[module]]",
        "[[/ [[ module]]",
    ] {
        let source = format!(
            "[[module CSS]]\n\
             [[module ListPages name=\"hidden-list\"]]X\n\
             [[module CountPages category=\"hidden-count\"]]\n\
             {closer}\n\
             [[module ListPages name=\"live\"]]C",
        );
        assert_runtime_modules_are_owned_until_live(&source);
    }
}

#[test]
fn downstream_regex_closers_keep_the_legacy_literal_mask() {
    let source = concat!(
        "[[module CſS]]\n",
        "[!-- [[/module]] --]\n",
        "@@[[/module]]@@\n",
        "[[module ListPages name=\"hidden-list\"]]X\n",
        "[[module CountPages category=\"hidden-count\"]]\n",
        "[[/module]]\n",
        "[[module ListPages name=\"live\"]]C",
    );

    assert_runtime_modules_are_owned_until_live(source);
}

#[test]
fn pinned_css_closers_reject_right_link_false_closers() {
    let source = concat!(
        "[[ module CSS]]\n",
        "[[/module]]]\n",
        "[[module ListPages name=\"hidden-list\"]]X\n",
        "[[module CountPages category=\"hidden-count\"]]\n",
        "[[/module654]]\n",
        "[[module ListPages name=\"live\"]]C",
    );

    assert_runtime_modules_are_owned_until_live(source);
}

#[test]
fn pinned_css_raw_body_closes_on_context_free_tokens_without_a_later_close() {
    for context in [
        "[!-- [[/module]] --]",
        "@@[[/module]]@@",
        "[[$ [[/module]] $]]",
        "[[span value=\"[[/module]]\"]]",
    ] {
        let source = format!(
            "[[ module CSS]]\n\
             [[module CountPages tags=\"+owned-count\"]]H\n\
             [[module ListPages name=\"owned-list\"]]H\n\
             {context}\n\
             [[module CountPages tags=\"+live-count\"]]H\n\
             [[module ListPages name=\"live-list\"]]H",
        );
        let ranges = collect_downstream_css_module_ranges(&source);
        let index = LiteralRegionIndex::new_list_pages_syntax(&source);
        let close_end = source.find("[[/module]]").unwrap() + "[[/module]]".len();

        assert_eq!(ranges, vec![0..close_end], "{context:?}");
        assert!(
            ranges[0].contains(&source.find("owned-count").unwrap()),
            "{context:?}"
        );
        assert!(
            ranges[0].contains(&source.find("owned-list").unwrap()),
            "{context:?}"
        );
        assert!(
            !ranges[0].contains(&source.find("live-count").unwrap()),
            "{context:?}"
        );
        assert!(
            !ranges[0].contains(&source.find("live-list").unwrap()),
            "{context:?}"
        );
        assert!(
            index.contains(source.find("owned-count").unwrap()),
            "{context:?}"
        );
        assert!(
            index.contains(source.find("owned-list").unwrap()),
            "{context:?}"
        );
        assert!(
            !index.contains(source.find("live-count").unwrap()),
            "{context:?}"
        );
        assert!(
            !index.contains(source.find("live-list").unwrap()),
            "{context:?}"
        );
    }
}

#[test]
fn pinned_css_comment_close_owns_the_preceding_count_pages_opener() {
    let source = concat!(
        "[[ module CSS]]\n",
        "[[module CountPages tags=\"+x\"]]H\n",
        "[!-- [[/module]] --]",
    );
    let ranges = collect_downstream_css_module_ranges(source);
    let index = LiteralRegionIndex::new_list_pages_syntax(source);
    let count_start = source.find("[[module CountPages").unwrap();
    let close_end = source.find("[[/module]]").unwrap() + "[[/module]]".len();

    assert_eq!(ranges, vec![0..close_end]);
    assert!(ranges[0].contains(&count_start));
    assert!(index.contains(count_start));
}

#[test]
fn pinned_css_closer_line_break_matrix_matches_block_name_consumption() {
    for closer in [
        "[[/module\n]]",
        "[[/module\r]]",
        "[[/module\r\n]]",
        "[[/module\n\n]]",
        "[[/module\n \t]]",
        "[[/module\u{00a0}\n]]",
        "[[/module\u{000b}\n]]",
        "[[/module\u{000c}\n]]",
        "[[/[[module\r\n\t ]]",
    ] {
        let source = format!(
            "[[module CSS]]\n\
             [[module ListPages name=\"owned\"]]X\n\
             {closer}\n\
             [[module ListPages name=\"live\"]]Y",
        );
        assert_css_range_owns_named_module_only(&source, "owned", "live");
    }

    for false_closer in ["[[/module \n]]", "[[/module\t\r\n]]"] {
        let source = format!(
            "[[module CSS]]\n\
             {false_closer}\n\
             [[module ListPages name=\"owned\"]]X\n\
             [[/module]]\n\
             [[module ListPages name=\"live\"]]Y[[/module]]",
        );
        assert_css_range_owns_named_module_only(&source, "owned", "live");
    }
}

#[test]
fn same_start_regex_and_pinned_ranges_are_unioned_in_both_directions() {
    let regex_extends_farther = concat!(
        "[[module CSS]]\n",
        "[[/ module]]\n",
        "[[module ListPages name=\"owned\"]]X[[/module]]\n",
        "[[module ListPages name=\"live\"]]Y[[/module]]",
    );
    assert_css_range_owns_named_module_only(regex_extends_farther, "owned", "live");

    let pinned_extends_farther = concat!(
        "[[module CSS]]\n",
        "[[/module]]]\n",
        "[[module ListPages name=\"owned\"]]X\n",
        "[[/ module]]\n",
        "[[module ListPages name=\"live\"]]Y[[/module]]",
    );
    assert_css_range_owns_named_module_only(pinned_extends_farther, "owned", "live");
}

#[test]
fn cross_path_overlaps_are_unioned_without_skipping_inner_openers() {
    let pinned_inner_extends_farther = concat!(
        "[[module CſS]]\n",
        "[[ module CSS]]\n",
        "[[/module]]]\n",
        "[[module ListPages name=\"owned\"]]X\n",
        "[[/ module]]\n",
        "[[module ListPages name=\"live\"]]Y[[/module]]",
    );
    assert_css_range_owns_named_module_only(
        pinned_inner_extends_farther,
        "owned",
        "live",
    );

    let regex_inner_extends_farther = concat!(
        "[[ module CSS]]\n",
        "[[module CſS]]\n",
        "[[/ module]]\n",
        "[[module ListPages name=\"owned\"]]X\n",
        "[[/module]]\n",
        "[[module ListPages name=\"live\"]]Y[[/module]]",
    );
    assert_css_range_owns_named_module_only(regex_inner_extends_farther, "owned", "live");
}

#[test]
fn malformed_quoted_css_head_resurrects_inner_css_in_actual_ranges() {
    for suffix in ["", "\n"] {
        let source = format!(
            "[[ module CSS note=\"unterminated [[ module CSS]][[module CountPages]][[/module]]{suffix}",
        );
        let inner_start = source.rfind("[[ module CSS]]").unwrap();
        let count_start = source.find("[[module CountPages]]").unwrap();
        let close_end = source.find("[[/module]]").unwrap() + "[[/module]]".len();
        assert_eq!(
            collect_downstream_css_module_ranges(&source),
            vec![inner_start..close_end],
            "{suffix:?}",
        );
        assert!(
            LiteralRegionIndex::new_list_pages_syntax(&source).contains(count_start),
            "{suffix:?}",
        );
    }
}

#[test]
fn dense_divergent_same_start_ranges_use_independent_forward_passes() {
    const PAIRS: usize = 2_048;
    let pair = concat!(
        "[[module CSS]][[/ module]]x[[/module]]",
        "[[module CSS]][[/module]]]x[[/ module]]",
    );
    let source = pair.repeat(PAIRS);
    let ranges = collect_downstream_css_module_ranges(&source);

    assert_eq!(ranges, vec![0..source.len()]);
}

#[test]
fn dense_malformed_email_heads_preserve_forward_token_cursor_progress() {
    const HEADS: usize = 4_096;
    let mut source = "a@b.example [[module CSS x=y\n".repeat(HEADS);
    let live_start = source.len();
    source.push_str("[[ module CSS note=\"x]y\"]]x[[/module]]");
    let ranges = collect_downstream_css_module_ranges(&source);

    assert_eq!(ranges, vec![live_start..source.len()]);
}

#[test]
fn dense_native_quote_checks_are_monotone() {
    const QUOTED: usize = 4_096;
    let mut source = "> [[ module CSS]]\n".repeat(QUOTED);
    let live_start = source.len();
    source.push_str("[[ module CSS]]x[[/module]]");
    let ranges = collect_downstream_css_module_ranges(&source);

    assert_eq!(ranges, vec![live_start..source.len()]);
}

#[test]
fn downstream_css_union_keeps_non_extracted_boundaries_live() {
    for opener in [
        "> [[module CSS]]",
        "[[moduleCſS]]",
        "[[module CſX]]",
        "@@[[module CſS]]@@",
    ] {
        let source = format!("{opener}\n[[module ListPages name=\"live\"]]C[[/module]]",);
        let index = LiteralRegionIndex::new_list_pages_syntax(&source);

        assert!(
            !index.contains(source.find("[[module ListPages").unwrap()),
            "{opener:?}: {:?}",
            index.ranges,
        );
    }
}

fn assert_css_range_owns_hidden_only(source: &str) {
    let index = LiteralRegionIndex::new_list_pages_syntax(source);

    assert!(index.contains(source.find("hidden").unwrap()), "{source:?}");
    assert!(!index.contains(source.find("live").unwrap()), "{source:?}");
    assert!(
        index
            .ranges
            .windows(2)
            .all(|pair| pair[0].end < pair[1].start),
        "{source:?}",
    );
}

fn assert_runtime_modules_are_owned_until_live(source: &str) {
    let index = LiteralRegionIndex::new_list_pages_syntax(source);

    assert!(
        index.contains(source.find("hidden-list").unwrap()),
        "{source:?}"
    );
    assert!(
        index.contains(source.find("hidden-count").unwrap()),
        "{source:?}"
    );
    assert!(
        !index.contains(source.find("name=\"live\"").unwrap()),
        "{source:?}"
    );
}

fn assert_css_range_owns_named_module_only(source: &str, owned: &str, live: &str) {
    let index = LiteralRegionIndex::new_list_pages_syntax(source);

    assert!(index.contains(source.find(owned).unwrap()), "{source:?}");
    assert!(!index.contains(source.find(live).unwrap()), "{source:?}");
}
