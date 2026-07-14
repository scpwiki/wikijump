/*
 * services/render/literal_regions/parser_candidates.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

mod arena;
mod selector;

pub(super) use self::arena::{EmitRangeIndex, EmitSetArena};
pub(super) use self::selector::{
    DelimiterIdentity as ParserDelimiterIdentity, ExactEffect, ParserCandidate,
    ParserDomain, select_two_phase_candidates,
};

#[cfg(test)]
use std::collections::HashSet;
use std::ops::Range;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ParserOwnerKind {
    TextLink,
    Color,
    #[cfg(test)]
    Base,
}

#[cfg(test)]
impl ParserOwnerKind {
    fn priority(self) -> u8 {
        match self {
            Self::TextLink => 2,
            Self::Color => 3,
            Self::Base => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ParserOwnerCertainty {
    /// Selecting this candidate proves that its parser consumes its range.
    Exact,
    /// Early-runtime safety protects this range without claiming parser consumption.
    ProtectionOnly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ParserOwnerCandidate {
    pub(super) range: Range<usize>,
    pub(super) kind: ParserOwnerKind,
    pub(super) certainty: ParserOwnerCertainty,
    /// A same-token opener at this byte is consumed only when this candidate is selected.
    pub(super) terminator_start: Option<usize>,
}

impl ParserOwnerCandidate {
    pub(super) fn exact(
        range: Range<usize>,
        kind: ParserOwnerKind,
        terminator_start: Option<usize>,
    ) -> Self {
        Self {
            range,
            kind,
            certainty: ParserOwnerCertainty::Exact,
            terminator_start,
        }
    }

    pub(super) fn protection(range: Range<usize>, kind: ParserOwnerKind) -> Self {
        Self {
            range,
            kind,
            certainty: ParserOwnerCertainty::ProtectionOnly,
            terminator_start: None,
        }
    }
}

/// Resolve parser-owned candidates once, after every source of candidates has
/// retained its overlaps. Protection-only alternatives contribute a safety
/// union but never suppress a later exact candidate.
#[cfg(test)]
pub(super) fn select_parser_owner_candidates(
    streams: &[Vec<ParserOwnerCandidate>],
) -> Vec<Range<usize>> {
    let mut indices = vec![0usize; streams.len()];
    let mut exact_owned_until = 0usize;
    let mut consumed_terminators = HashSet::new();
    let mut selected = Vec::new();

    loop {
        let next = streams
            .iter()
            .enumerate()
            .filter_map(|(stream, candidates)| {
                candidates.get(indices[stream]).map(|candidate| {
                    (
                        stream,
                        candidate,
                        (
                            candidate.range.start,
                            candidate.kind.priority(),
                            stream,
                            candidate.range.end,
                        ),
                    )
                })
            })
            .min_by_key(|(_, _, key)| *key);
        let Some((stream, candidate, _)) = next else {
            break;
        };
        indices[stream] += 1;

        if candidate.range.start < exact_owned_until
            || consumed_terminators.contains(&candidate.range.start)
        {
            continue;
        }

        push_union_range(&mut selected, candidate.range.clone());
        if candidate.certainty == ParserOwnerCertainty::Exact {
            exact_owned_until = candidate.range.end;
            if let Some(terminator) = candidate.terminator_start {
                consumed_terminators.insert(terminator);
            }
        }
    }

    selected
}

#[cfg(test)]
fn push_union_range(ranges: &mut Vec<Range<usize>>, range: Range<usize>) {
    if range.start >= range.end {
        return;
    }
    if let Some(previous) = ranges.last_mut()
        && range.start <= previous.end
    {
        previous.end = previous.end.max(range.end);
    } else {
        ranges.push(range);
    }
}

#[cfg(test)]
mod tests {
    use super::{ParserOwnerCandidate, ParserOwnerKind, select_parser_owner_candidates};

    #[test]
    fn discarded_outer_candidate_does_not_suppress_a_later_inner_candidate() {
        let base = vec![
            ParserOwnerCandidate::exact(20..80, ParserOwnerKind::Base, Some(70)),
            ParserOwnerCandidate::exact(50..65, ParserOwnerKind::Base, Some(62)),
        ];
        let links = vec![
            ParserOwnerCandidate::exact(0..40, ParserOwnerKind::TextLink, None),
            ParserOwnerCandidate::exact(45..55, ParserOwnerKind::TextLink, None),
        ];

        assert_eq!(
            select_parser_owner_candidates(&[base, links]),
            vec![0..40, 45..55],
        );
    }

    #[test]
    fn selected_terminator_is_not_reinterpreted_as_an_opener() {
        let first_stream = vec![
            ParserOwnerCandidate::exact(0..6, ParserOwnerKind::Color, Some(12)),
            ParserOwnerCandidate::exact(12..20, ParserOwnerKind::Color, Some(24)),
        ];
        let second_stream = vec![ParserOwnerCandidate::exact(
            12..18,
            ParserOwnerKind::Base,
            Some(16),
        )];
        assert_eq!(
            select_parser_owner_candidates(&[first_stream, second_stream]),
            vec![0..6],
        );
    }

    #[test]
    fn protection_only_is_discarded_inside_an_exact_owner() {
        let protection = vec![ParserOwnerCandidate::protection(
            2..30,
            ParserOwnerKind::Base,
        )];
        let exact = vec![ParserOwnerCandidate::exact(
            0..8,
            ParserOwnerKind::Color,
            Some(40),
        )];
        assert_eq!(
            select_parser_owner_candidates(&[exact, protection]),
            vec![0..8],
        );
    }

    #[test]
    fn selected_protection_does_not_advance_exact_parser_state() {
        let protection = vec![ParserOwnerCandidate::protection(
            0..20,
            ParserOwnerKind::Base,
        )];
        let exact = vec![ParserOwnerCandidate::exact(
            10..30,
            ParserOwnerKind::TextLink,
            None,
        )];
        assert_eq!(
            select_parser_owner_candidates(&[protection, exact]),
            vec![0..30],
        );
    }
}
