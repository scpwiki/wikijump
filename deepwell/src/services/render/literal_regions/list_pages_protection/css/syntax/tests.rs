/*
 * services/render/literal_regions/list_pages_protection/css/syntax/tests.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use super::pinned_module_close_end;
use ftml::data::{PageInfo, ScoreValue};
use ftml::layout::Layout;
use ftml::settings::{WikitextMode, WikitextSettings};
use ftml::tree::Element;
use std::borrow::Cow;

const MAX_PINNED_ORACLE_SOURCE_BYTES: usize = 256;

#[test]
fn pinned_css_closer_scanner_matches_the_bounded_ftml_oracle() {
    let valid = [
        "[[/module]]",
        "[[/ module]]",
        "[[/module ]]",
        "[[/module654]]",
        "[[/[[module]]",
        "[[/ [[ module]]",
        "[[/module\n]]",
        "[[/module\r]]",
        "[[/module\r\n]]",
        "[[/module\n\n]]",
        "[[/module\n \t]]",
        "[[/module\u{00a0}\n]]",
        "[[/module\u{000b}\n]]",
        "[[/module\u{000c}\n]]",
        "[[/[[module\r\n\t ]]",
    ];
    let invalid = [
        "[[/module]]]",
        "[[/module]]]]",
        "[[/module \n]]",
        "[[/module\t\r\n]]",
        "[[/module_]]",
        "[[/module654_]]",
        "[[/module__]]",
        "[[/[[module_]]",
        "[[/ [[ module654_]]",
        "[[/module other]]",
        "[[/[[[module]]",
        "[[/[[module]]]",
        "[[/module",
    ];

    for (expected, closers) in [(true, valid.as_slice()), (false, invalid.as_slice())] {
        for closer in closers {
            assert_eq!(
                pinned_module_close_end(closer.as_bytes(), 0) == Some(closer.len()),
                expected,
                "local close scan disagrees with the expected matrix: {closer:?}",
            );
            assert_eq!(
                pinned_ftml_accepts_css_closer(closer),
                expected,
                "pinned FTML disagrees with the expected matrix: {closer:?}",
            );
        }
    }
}

fn pinned_ftml_accepts_css_closer(closer: &str) -> bool {
    let source = format!("[[module CSS]]x{closer}");
    assert!(source.len() <= MAX_PINNED_ORACLE_SOURCE_BYTES);
    let page_info = PageInfo {
        page: Cow::Borrowed("oracle"),
        category: None,
        site: Cow::Borrowed("oracle"),
        title: Cow::Borrowed("Oracle"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("default"),
    };
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut source = source;
    ftml::preprocess_for_layout(&mut source, settings.layout);
    let tokenization = ftml::tokenize(&source);
    let (tree, _) = ftml::parse(&tokenization, &page_info, &settings).into();
    tree.elements
        .iter()
        .any(|element| matches!(element, Element::Style(_)))
}
