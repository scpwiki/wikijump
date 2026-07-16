/*
 * services/render/list_pages_scanner/count_reachability.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use super::{ModuleEvent, ModuleEventScanner, ModuleOpenKind, collect_module_events};
use crate::services::render::literal_regions::LiteralRegionIndex;
use std::ops::Range;

#[derive(Debug, Default)]
pub(in crate::services::render) struct CountPagesCloseReachabilityIndex {
    pairs: Vec<Range<usize>>,
    ambiguous: bool,
}

pub(in crate::services::render) struct CountPagesCloseReachabilityCursor<'a> {
    pairs: &'a [Range<usize>],
    index: usize,
    advances: usize,
    ambiguous: bool,
    last_start: Option<usize>,
}

impl CountPagesCloseReachabilityIndex {
    pub(in crate::services::render) fn new(source: &str) -> Self {
        let lowercase = source.to_ascii_lowercase();
        if !lowercase.contains("[[") || !lowercase.contains("countpages") {
            return Self::default();
        }
        let literal_regions = LiteralRegionIndex::new_count_pages_syntax(source);
        let scanner = ModuleEventScanner::new(source, &lowercase, &literal_regions);
        let (events, _, _, ambiguous) = collect_module_events(scanner);
        if ambiguous {
            return Self {
                pairs: Vec::new(),
                ambiguous: true,
            };
        }

        let mut pairs = Vec::<(usize, Option<usize>)>::new();
        let mut stack = Vec::<Option<usize>>::new();
        for event in events {
            match event {
                ModuleEvent::Open {
                    kind,
                    start,
                    subname_start,
                    subname_end,
                    direct_candidate,
                    ..
                } => {
                    let pair = (kind == ModuleOpenKind::Standard
                        && direct_candidate
                        && source[subname_start..subname_end]
                            .eq_ignore_ascii_case("countpages"))
                    .then(|| {
                        let pair = pairs.len();
                        pairs.push((start, None));
                        pair
                    });
                    stack.push(pair);
                }
                ModuleEvent::Close { end, .. } => {
                    let Some(pair) = stack.pop() else {
                        continue;
                    };
                    if let Some(pair) = pair {
                        pairs[pair].1 = Some(end);
                    }
                }
            }
        }

        Self {
            pairs: pairs
                .into_iter()
                .filter_map(|(start, end)| end.map(|end| start..end))
                .collect(),
            ambiguous: false,
        }
    }

    pub(in crate::services::render) fn monotone_cursor(
        &self,
    ) -> CountPagesCloseReachabilityCursor<'_> {
        CountPagesCloseReachabilityCursor {
            pairs: &self.pairs,
            index: 0,
            advances: 0,
            ambiguous: self.ambiguous,
            last_start: None,
        }
    }
}

impl CountPagesCloseReachabilityCursor<'_> {
    pub(in crate::services::render) fn regex_capture_close_is_reachable(
        &mut self,
        capture: Range<usize>,
    ) -> bool {
        debug_assert!(
            self.last_start
                .is_none_or(|previous| previous <= capture.start),
            "CountPages captures must be queried in source order",
        );
        self.last_start = Some(capture.start);
        if self.ambiguous {
            return false;
        }
        while self
            .pairs
            .get(self.index)
            .is_some_and(|pair| pair.start < capture.start)
        {
            self.index += 1;
            self.advances += 1;
        }
        self.pairs
            .get(self.index)
            .is_some_and(|pair| *pair == capture)
    }

    pub(in crate::services::render) fn advances(&self) -> usize {
        self.advances
    }
}
