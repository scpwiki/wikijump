/*
 * services/render/literal_regions/parser_candidates/selector.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use super::arena::{EmitSetArena, EmitSetId, LeafId};
use std::collections::HashSet;
use std::ops::Range;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::services::render::literal_regions) enum ParserDomain {
    Ftml,
    CompatPreFtml,
}

impl ParserDomain {
    const fn tie_priority(self) -> u8 {
        match self {
            Self::CompatPreFtml => 0,
            Self::Ftml => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::services::render::literal_regions) struct DelimiterIdentity {
    pub(in crate::services::render::literal_regions) namespace: u16,
    pub(in crate::services::render::literal_regions) start: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::services::render::literal_regions) struct ExactEffect {
    pub(in crate::services::render::literal_regions) claim: Range<usize>,
    pub(in crate::services::render::literal_regions) domain: ParserDomain,
    pub(in crate::services::render::literal_regions) own_emit: Option<LeafId>,
    pub(in crate::services::render::literal_regions) child_emit: EmitSetId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::services::render::literal_regions) enum CandidateEffect {
    Exact(ExactEffect),
    /// Fail closed through a deterministic recovery boundary without claiming
    /// the source as successfully parsed FTML.
    RecoveryBarrier {
        hard_sync: usize,
    },
    /// Runtime fail-closed protection emits a range without claiming parser input.
    PolicyEmit(EmitSetId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::services::render::literal_regions) struct ParserCandidate {
    pub(in crate::services::render::literal_regions) start: usize,
    pub(in crate::services::render::literal_regions) precedence: u16,
    pub(in crate::services::render::literal_regions) opener: Option<DelimiterIdentity>,
    pub(in crate::services::render::literal_regions) terminator:
        Option<DelimiterIdentity>,
    pub(in crate::services::render::literal_regions) effect: CandidateEffect,
}

impl ParserCandidate {
    pub(in crate::services::render::literal_regions) fn exact(
        precedence: u16,
        opener: Option<DelimiterIdentity>,
        terminator: Option<DelimiterIdentity>,
        effect: ExactEffect,
    ) -> Self {
        let start = effect.claim.start;
        Self {
            start,
            precedence,
            opener,
            terminator,
            effect: CandidateEffect::Exact(effect),
        }
    }

    pub(in crate::services::render::literal_regions) fn policy(
        start: usize,
        precedence: u16,
        emit: EmitSetId,
    ) -> Self {
        Self {
            start,
            precedence,
            opener: None,
            terminator: None,
            effect: CandidateEffect::PolicyEmit(emit),
        }
    }

    pub(in crate::services::render::literal_regions) fn recovery_barrier(
        start: usize,
        precedence: u16,
        hard_sync: usize,
    ) -> Self {
        Self {
            start,
            precedence,
            opener: None,
            terminator: None,
            effect: CandidateEffect::RecoveryBarrier { hard_sync },
        }
    }

    fn domain(&self) -> ParserDomain {
        match &self.effect {
            CandidateEffect::Exact(effect) => effect.domain,
            CandidateEffect::RecoveryBarrier { .. } | CandidateEffect::PolicyEmit(_) => {
                ParserDomain::Ftml
            }
        }
    }
}

#[derive(Debug, Default)]
struct PhaseSelection {
    emit_roots: Vec<EmitSetId>,
    claims: Vec<Range<usize>>,
    selected_openers: HashSet<DelimiterIdentity>,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(in crate::services::render::literal_regions) struct CandidateSelection {
    pub(in crate::services::render::literal_regions) ranges: Vec<Range<usize>>,
    pub(in crate::services::render::literal_regions) final_claims: Vec<Range<usize>>,
    pub(in crate::services::render::literal_regions) selected_openers:
        HashSet<DelimiterIdentity>,
}

/// Select final literal ownership after compatibility candidates are present.
///
/// Original-source effects must not survive when compatibility parsing makes
/// their owning candidate unreachable. Later output protectors are merged by
/// the caller after parser ownership has been resolved.
pub(in crate::services::render::literal_regions) fn select_two_phase_candidates(
    arena: &EmitSetArena,
    original_streams: &[Vec<ParserCandidate>],
    compat_streams: &[Vec<ParserCandidate>],
) -> CandidateSelection {
    validate_streams(arena, original_streams);
    validate_streams(arena, compat_streams);

    let mut rebalanced_refs: Vec<_> = compat_streams.iter().map(Vec::as_slice).collect();
    rebalanced_refs.extend(original_streams.iter().map(Vec::as_slice));
    let rebalanced = select_phase(arena, &rebalanced_refs);

    CandidateSelection {
        ranges: arena.materialize(rebalanced.emit_roots),
        final_claims: coalesce_claims(rebalanced.claims),
        selected_openers: rebalanced.selected_openers,
    }
}

fn select_phase(arena: &EmitSetArena, streams: &[&[ParserCandidate]]) -> PhaseSelection {
    let mut indices = vec![0usize; streams.len()];
    let mut must_claim_until = 0usize;
    let mut blocked_until = 0usize;
    let mut consumed = HashSet::new();
    let mut selection = PhaseSelection::default();

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
                            candidate.start,
                            candidate.domain().tie_priority(),
                            candidate.precedence,
                            stream,
                        ),
                    )
                })
            })
            .min_by_key(|(_, _, key)| *key);
        let Some((stream, candidate, _)) = next else {
            break;
        };
        indices[stream] += 1;

        if candidate
            .opener
            .is_some_and(|identity| consumed.contains(&identity))
        {
            continue;
        }
        if candidate.start < must_claim_until || candidate.start < blocked_until {
            continue;
        }

        match &candidate.effect {
            CandidateEffect::Exact(effect) => {
                push_exact_emits(arena, &mut selection.emit_roots, effect);
                must_claim_until = effect.claim.end;
                selection.claims.push(effect.claim.clone());
                if let Some(opener) = candidate.opener {
                    selection.selected_openers.insert(opener);
                }
                if let Some(terminator) = candidate.terminator {
                    consumed.insert(terminator);
                }
            }
            CandidateEffect::RecoveryBarrier { hard_sync } => {
                blocked_until = blocked_until.max(*hard_sync);
            }
            CandidateEffect::PolicyEmit(emit) => selection.emit_roots.push(*emit),
        }
    }
    selection
}

fn push_exact_emits(
    arena: &EmitSetArena,
    roots: &mut Vec<EmitSetId>,
    effect: &ExactEffect,
) {
    if let Some(leaf) = effect.own_emit {
        roots.push(arena.leaf_set(leaf));
    }
    roots.push(effect.child_emit);
}

fn validate_streams(arena: &EmitSetArena, streams: &[Vec<ParserCandidate>]) {
    for stream in streams {
        assert!(
            stream.windows(2).all(|pair| pair[0].start <= pair[1].start),
            "candidate streams must be source ordered",
        );
        for candidate in stream {
            match &candidate.effect {
                CandidateEffect::Exact(effect) => {
                    assert!(
                        effect.claim.start == candidate.start
                            && effect.claim.start < effect.claim.end,
                        "exact claims must be nonempty and start at the candidate",
                    );
                    if let Some(leaf) = effect.own_emit {
                        let range = arena.range(leaf);
                        assert!(range.start < range.end, "exact emits must be nonempty");
                    }
                }
                CandidateEffect::RecoveryBarrier { hard_sync, .. } => assert!(
                    *hard_sync >= candidate.start,
                    "recovery hard sync cannot precede its candidate",
                ),
                CandidateEffect::PolicyEmit(_) => {}
            }
        }
    }
}

fn coalesce_claims(mut claims: Vec<Range<usize>>) -> Vec<Range<usize>> {
    claims.sort_unstable_by_key(|range| (range.start, range.end));
    let mut output: Vec<Range<usize>> = Vec::with_capacity(claims.len());
    for claim in claims {
        if let Some(previous) = output.last_mut()
            && claim.start <= previous.end
        {
            previous.end = previous.end.max(claim.end);
        } else {
            output.push(claim);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exact(
        arena: &mut EmitSetArena,
        claim: Range<usize>,
        emit: Range<usize>,
        domain: ParserDomain,
        terminator: Option<DelimiterIdentity>,
    ) -> ParserCandidate {
        let (leaf, _) = arena.leaf(emit);
        ParserCandidate::exact(
            0,
            Some(DelimiterIdentity {
                namespace: 1,
                start: claim.start,
            }),
            terminator,
            ExactEffect {
                claim,
                domain,
                own_emit: Some(leaf),
                child_emit: EmitSetArena::EMPTY,
            },
        )
    }

    #[test]
    fn policy_emit_never_blocks_same_start_exact() {
        let mut arena = EmitSetArena::default();
        let (_, output) = arena.leaf(0..3);
        let exact = exact(&mut arena, 0..8, 4..8, ParserDomain::Ftml, None);
        let selected = select_two_phase_candidates(
            &arena,
            &[vec![ParserCandidate::policy(0, 0, output)], vec![exact]],
            &[],
        );
        assert_eq!(selected.ranges, vec![0..3, 4..8]);
        assert_eq!(selected.final_claims, vec![0..8]);
    }

    #[test]
    fn recovery_barrier_suppresses_inner_candidates_without_claiming() {
        let mut arena = EmitSetArena::default();
        let first = exact(&mut arena, 4..10, 4..5, ParserDomain::Ftml, None);
        let second = exact(&mut arena, 6..14, 12..14, ParserDomain::Ftml, None);
        let stream = vec![ParserCandidate::recovery_barrier(0, 0, 6), first, second];
        let selected = select_two_phase_candidates(&arena, &[stream], &[]);
        assert_eq!(selected.ranges, vec![12..14]);
        assert_eq!(selected.final_claims, vec![6..14]);
    }

    #[test]
    fn compat_rebalance_removes_unreachable_original_leaf() {
        let mut arena = EmitSetArena::default();
        let original = exact(&mut arena, 2..12, 4..8, ParserDomain::Ftml, None);
        let compat = exact(&mut arena, 0..6, 0..1, ParserDomain::CompatPreFtml, None);
        let selected =
            select_two_phase_candidates(&arena, &[vec![original]], &[vec![compat]]);
        assert_eq!(selected.ranges, vec![0..1]);
        assert_eq!(selected.final_claims, vec![0..6]);
    }

    #[test]
    fn selected_terminator_identity_is_not_reopened() {
        let mut arena = EmitSetArena::default();
        let terminator = DelimiterIdentity {
            namespace: 7,
            start: 8,
        };
        let first = exact(&mut arena, 0..4, 0..2, ParserDomain::Ftml, Some(terminator));
        let mut second = exact(&mut arena, 8..12, 8..10, ParserDomain::Ftml, None);
        second.opener = Some(terminator);
        let selected = select_two_phase_candidates(&arena, &[vec![first, second]], &[]);
        assert_eq!(selected.ranges, vec![0..2]);
    }
}
