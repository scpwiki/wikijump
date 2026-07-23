/*
 * services/render/literal_regions/list_pages_protection/candidate_graph.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use super::css::{
    collect_downstream_css_module_ranges,
    collect_downstream_css_module_ranges_with_heads,
    collect_pinned_css_module_candidates,
    collect_pinned_css_module_candidates_with_heads, collect_projected_css_module_ranges,
    collect_projected_css_module_ranges_with_heads,
};
use crate::services::render::literal_regions::anchor_candidates::collect_pinned_anchor_candidates_with_text_tokens;
use crate::services::render::literal_regions::base_candidates::{
    BaseCandidate, BaseCandidatePolicy, BaseCandidateProvenance, DelimiterIdentity,
    collect_base_candidates_with_text_tokens,
};
use crate::services::render::literal_regions::block_candidates::{
    BlockCandidate, HeadContext, RuntimeModuleHeadCandidate, collect_block_candidates,
    collect_block_candidates_with_heads, collect_head_candidate_streams,
    collect_head_candidate_streams_with_context,
};
use crate::services::render::literal_regions::downstream_protectors::collect_downstream_protector_ranges_with_runtime_heads;
use crate::services::render::literal_regions::parser_candidates::{
    EmitRangeIndex, EmitSetArena, ExactEffect, ParserCandidate, ParserDelimiterIdentity,
    ParserDomain, ParserOwnerCandidate, ParserOwnerCertainty, ParserOwnerKind,
    select_two_phase_candidates,
};
use crate::services::render::literal_regions::text_owners::collect_text_owner_candidates_with_text_tokens;
use crate::services::render::literal_regions::token_boundaries::TextTokenIndex;
use std::ops::Range;

const PRECEDENCE_COMPAT_CSS: u16 = 0;
const PRECEDENCE_PINNED_CSS: u16 = 10;
const PRECEDENCE_PINNED_ANCHOR: u16 = 15;
const PRECEDENCE_TEXT_LINK: u16 = 20;
const PRECEDENCE_COLOR: u16 = 30;
const PRECEDENCE_BLOCK: u16 = 40;
const PRECEDENCE_GENERIC_HEAD: u16 = 45;
const PRECEDENCE_BASE: u16 = 50;
const PRECEDENCE_QUOTE: u16 = 60;

const DELIMITER_NAMESPACE_BASE: u16 = 0;
const DELIMITER_NAMESPACE_TEXT_LINK: u16 = 100;
const DELIMITER_NAMESPACE_COLOR: u16 = 101;
const DELIMITER_NAMESPACE_CSS: u16 = 102;
const DELIMITER_NAMESPACE_ANCHOR: u16 = 103;
const DELIMITER_NAMESPACE_GENERIC_HEAD: u16 = 104;

pub(super) fn collect_candidate_graph_ranges(
    source: &str,
    original_quote_ranges: &[Range<usize>],
    compat_quote_ranges: &[Range<usize>],
    include_downstream_css: bool,
    include_base_candidates: bool,
) -> Vec<Range<usize>> {
    let mut arena = EmitSetArena::default();
    let text_tokens = TextTokenIndex::new(source);
    let heads = (source.len() < u32::MAX as usize)
        .then(|| HeadContext::new_with_text_tokens(source, &text_tokens));
    let base = if include_base_candidates {
        adapt_base_candidates(
            &mut arena,
            collect_base_candidates_with_text_tokens(
                source,
                BaseCandidatePolicy::FAIL_CLOSED_RUNTIME,
                &text_tokens,
            ),
            PRECEDENCE_BASE,
        )
    } else {
        Vec::new()
    };
    let blocks = adapt_block_candidates(
        &mut arena,
        heads.as_ref().map_or_else(
            || collect_block_candidates(source),
            |heads| collect_block_candidates_with_heads(source, heads),
        ),
    );
    let (text_links, colors) = partition_text_candidates(
        collect_text_owner_candidates_with_text_tokens(source, &text_tokens),
    );
    let links = adapt_text_candidates(&mut arena, text_links);
    let color_descriptors = adapt_color_descriptor_candidates(&mut arena, colors.clone());
    let pinned_css_ranges = heads.as_ref().map_or_else(
        || collect_pinned_css_module_candidates(source),
        |heads| collect_pinned_css_module_candidates_with_heads(source, heads),
    );
    let pinned_css = adapt_exact_ranges(
        &mut arena,
        pinned_css_ranges.clone(),
        ParserDomain::Ftml,
        PRECEDENCE_PINNED_CSS,
        DELIMITER_NAMESPACE_CSS,
    );
    let pinned_anchors = adapt_exact_ranges(
        &mut arena,
        collect_pinned_anchor_candidates_with_text_tokens(source, &text_tokens),
        ParserDomain::Ftml,
        PRECEDENCE_PINNED_ANCHOR,
        DELIMITER_NAMESPACE_ANCHOR,
    );
    let head_candidates = heads.as_ref().map_or_else(
        || collect_head_candidate_streams(source),
        |heads| collect_head_candidate_streams_with_context(source, heads, &text_tokens),
    );
    let generic_heads = adapt_runtime_module_heads(head_candidates.generic);
    let runtime_head_ranges = head_candidates
        .runtime_modules
        .iter()
        .filter_map(|candidate| match candidate {
            RuntimeModuleHeadCandidate::Exact(range) => Some(range.clone()),
            RuntimeModuleHeadCandidate::RecoveryBarrier(_) => None,
        })
        .collect::<Vec<_>>();
    let runtime_heads = adapt_runtime_module_heads(head_candidates.runtime_modules);
    let css_ranges = match (include_downstream_css, heads.as_ref()) {
        (true, Some(heads)) => {
            collect_downstream_css_module_ranges_with_heads(source, heads)
        }
        (true, None) => collect_downstream_css_module_ranges(source),
        (false, Some(heads)) => collect_projected_css_module_ranges_with_heads(
            source,
            original_quote_ranges,
            heads,
        ),
        (false, None) => {
            collect_projected_css_module_ranges(source, original_quote_ranges)
        }
    };
    let compat_quote_ranges = compat_quote_ranges
        .iter()
        .filter(|quote| {
            original_quote_ranges
                .iter()
                .any(|original| original == *quote)
                || !css_ranges
                    .iter()
                    .any(|css| quote.start <= css.start && css.start < quote.end)
        })
        .cloned()
        .collect::<Vec<_>>();
    let compat_css = adapt_exact_ranges(
        &mut arena,
        css_ranges,
        ParserDomain::CompatPreFtml,
        PRECEDENCE_COMPAT_CSS,
        DELIMITER_NAMESPACE_CSS,
    );
    let quotes = adapt_exact_ranges(
        &mut arena,
        original_quote_ranges.to_vec(),
        ParserDomain::Ftml,
        PRECEDENCE_QUOTE,
        DELIMITER_NAMESPACE_BASE + 90,
    );
    let compat_quotes = adapt_exact_ranges(
        &mut arena,
        compat_quote_ranges,
        ParserDomain::CompatPreFtml,
        PRECEDENCE_QUOTE,
        DELIMITER_NAMESPACE_BASE + 91,
    );
    let original_child_ranges = select_two_phase_candidates(
        &arena,
        &[
            base.clone(),
            blocks.clone(),
            links.clone(),
            pinned_css.clone(),
            pinned_anchors.clone(),
            generic_heads.clone(),
            runtime_heads.clone(),
            quotes.clone(),
            color_descriptors.clone(),
        ],
        &[],
    )
    .ranges;
    let original_child_index = EmitRangeIndex::new(&mut arena, original_child_ranges);
    let original_colors =
        adapt_color_candidates(&mut arena, &original_child_index, colors.clone());
    let original_stage_selection = select_two_phase_candidates(
        &arena,
        &[
            base.clone(),
            blocks.clone(),
            links.clone(),
            original_colors,
            pinned_css.clone(),
            pinned_anchors.clone(),
            generic_heads.clone(),
            runtime_heads.clone(),
            quotes.clone(),
        ],
        &[],
    );
    let child_ranges = select_two_phase_candidates(
        &arena,
        &[
            base.clone(),
            blocks.clone(),
            links.clone(),
            pinned_css.clone(),
            pinned_anchors.clone(),
            generic_heads.clone(),
            runtime_heads.clone(),
            quotes.clone(),
            color_descriptors,
        ],
        &[compat_css.clone(), compat_quotes.clone()],
    )
    .ranges;
    let child_index = EmitRangeIndex::new(&mut arena, child_ranges);
    let colors = adapt_color_candidates(&mut arena, &child_index, colors);

    let selected = select_two_phase_candidates(
        &arena,
        &[
            base,
            blocks,
            links,
            colors,
            pinned_css,
            pinned_anchors,
            generic_heads,
            runtime_heads,
            quotes,
        ],
        &[compat_css, compat_quotes],
    );
    let selected_runtime_heads = runtime_head_ranges
        .into_iter()
        .filter(|range| {
            original_stage_selection
                .selected_openers
                .contains(&ParserDelimiterIdentity {
                    namespace: DELIMITER_NAMESPACE_GENERIC_HEAD,
                    start: range.start,
                })
        })
        .collect::<Vec<_>>();
    let downstream = collect_downstream_protector_ranges_with_runtime_heads(
        source,
        &selected_runtime_heads,
    )
    .into_iter()
    .map(|protector| protector.range)
    .collect();
    let persistent_css = pinned_css_ranges
        .into_iter()
        .filter(|range| {
            original_stage_selection
                .selected_openers
                .contains(&ParserDelimiterIdentity {
                    namespace: DELIMITER_NAMESPACE_CSS,
                    start: range.start,
                })
        })
        .collect();
    let selected_with_css =
        super::super::merge_sorted_ranges(selected.ranges, persistent_css);
    super::super::merge_sorted_ranges(selected_with_css, downstream)
}

fn adapt_runtime_module_heads(
    candidates: Vec<RuntimeModuleHeadCandidate>,
) -> Vec<ParserCandidate> {
    candidates
        .into_iter()
        .map(|candidate| match candidate {
            RuntimeModuleHeadCandidate::Exact(range) => ParserCandidate::exact(
                PRECEDENCE_GENERIC_HEAD,
                Some(ParserDelimiterIdentity {
                    namespace: DELIMITER_NAMESPACE_GENERIC_HEAD,
                    start: range.start,
                }),
                None,
                ExactEffect {
                    claim: range,
                    domain: ParserDomain::Ftml,
                    own_emit: None,
                    child_emit: EmitSetArena::EMPTY,
                },
            ),
            RuntimeModuleHeadCandidate::RecoveryBarrier(range) => {
                ParserCandidate::recovery_barrier(
                    range.start,
                    PRECEDENCE_GENERIC_HEAD,
                    range.end,
                )
            }
        })
        .collect()
}

fn adapt_color_descriptor_candidates(
    arena: &mut EmitSetArena,
    candidates: Vec<ParserOwnerCandidate>,
) -> Vec<ParserCandidate> {
    candidates
        .into_iter()
        .map(|candidate| {
            let (leaf, _) = arena.leaf(candidate.range.clone());
            ParserCandidate::exact(
                PRECEDENCE_COLOR,
                Some(ParserDelimiterIdentity {
                    namespace: DELIMITER_NAMESPACE_COLOR,
                    start: candidate.range.start,
                }),
                None,
                ExactEffect {
                    claim: candidate.range,
                    domain: ParserDomain::Ftml,
                    own_emit: Some(leaf),
                    child_emit: EmitSetArena::EMPTY,
                },
            )
        })
        .collect()
}

fn partition_text_candidates(
    candidates: Vec<ParserOwnerCandidate>,
) -> (Vec<ParserOwnerCandidate>, Vec<ParserOwnerCandidate>) {
    let mut links = Vec::new();
    let mut colors = Vec::new();
    for candidate in candidates {
        if candidate.kind == ParserOwnerKind::Color {
            colors.push(candidate);
        } else {
            links.push(candidate);
        }
    }
    (links, colors)
}

fn adapt_base_candidates(
    arena: &mut EmitSetArena,
    candidates: Vec<BaseCandidate>,
    precedence: u16,
) -> Vec<ParserCandidate> {
    candidates
        .into_iter()
        .map(|candidate| {
            adapt_delimited_candidate(
                arena,
                candidate.range,
                candidate.provenance,
                candidate.opener,
                candidate.terminator,
                precedence,
            )
        })
        .collect()
}

fn adapt_block_candidates(
    arena: &mut EmitSetArena,
    candidates: Vec<BlockCandidate>,
) -> Vec<ParserCandidate> {
    candidates
        .into_iter()
        .map(|candidate| {
            adapt_delimited_candidate(
                arena,
                candidate.range,
                candidate.provenance,
                candidate.opener,
                candidate.terminator,
                PRECEDENCE_BLOCK,
            )
        })
        .collect()
}

fn adapt_delimited_candidate(
    arena: &mut EmitSetArena,
    range: Range<usize>,
    provenance: BaseCandidateProvenance,
    opener: DelimiterIdentity,
    terminator: Option<DelimiterIdentity>,
    precedence: u16,
) -> ParserCandidate {
    let (leaf, emit) = arena.leaf(range.clone());
    match provenance {
        BaseCandidateProvenance::ClosedOwner => ParserCandidate::exact(
            precedence,
            Some(adapt_delimiter(opener)),
            terminator.map(adapt_delimiter),
            ExactEffect {
                claim: range,
                domain: ParserDomain::Ftml,
                own_emit: Some(leaf),
                child_emit: EmitSetArena::EMPTY,
            },
        ),
        BaseCandidateProvenance::FailClosedProtection => {
            ParserCandidate::policy(range.start, precedence, emit)
        }
    }
}

fn adapt_text_candidates(
    arena: &mut EmitSetArena,
    candidates: Vec<ParserOwnerCandidate>,
) -> Vec<ParserCandidate> {
    candidates
        .into_iter()
        .map(|candidate| {
            let (leaf, emit) = arena.leaf(candidate.range.clone());
            let (precedence, namespace) = match candidate.kind {
                ParserOwnerKind::TextLink => {
                    (PRECEDENCE_TEXT_LINK, DELIMITER_NAMESPACE_TEXT_LINK)
                }
                ParserOwnerKind::Color => (PRECEDENCE_COLOR, DELIMITER_NAMESPACE_COLOR),
                #[cfg(test)]
                _ => (PRECEDENCE_BASE, DELIMITER_NAMESPACE_BASE + 80),
            };
            match candidate.certainty {
                ParserOwnerCertainty::Exact => {
                    let claim_end = candidate
                        .terminator_start
                        .map_or(candidate.range.end, |start| start.saturating_add(2));
                    ParserCandidate::exact(
                        precedence,
                        Some(ParserDelimiterIdentity {
                            namespace,
                            start: candidate.range.start,
                        }),
                        candidate
                            .terminator_start
                            .map(|start| ParserDelimiterIdentity { namespace, start }),
                        ExactEffect {
                            claim: candidate.range.start..claim_end,
                            domain: ParserDomain::Ftml,
                            own_emit: Some(leaf),
                            child_emit: EmitSetArena::EMPTY,
                        },
                    )
                }
                ParserOwnerCertainty::ProtectionOnly => {
                    ParserCandidate::policy(candidate.range.start, precedence, emit)
                }
            }
        })
        .collect()
}

fn adapt_color_candidates(
    arena: &mut EmitSetArena,
    child_index: &EmitRangeIndex,
    candidates: Vec<ParserOwnerCandidate>,
) -> Vec<ParserCandidate> {
    candidates
        .into_iter()
        .map(|candidate| {
            let (leaf, emit) = arena.leaf(candidate.range.clone());
            match candidate.certainty {
                ParserOwnerCertainty::Exact => {
                    let terminator = candidate
                        .terminator_start
                        .expect("exact color candidates have a terminator");
                    let child_emit =
                        child_index.contained_set(arena, candidate.range.end..terminator);
                    ParserCandidate::exact(
                        PRECEDENCE_COLOR,
                        Some(ParserDelimiterIdentity {
                            namespace: DELIMITER_NAMESPACE_COLOR,
                            start: candidate.range.start,
                        }),
                        Some(ParserDelimiterIdentity {
                            namespace: DELIMITER_NAMESPACE_COLOR,
                            start: terminator,
                        }),
                        ExactEffect {
                            claim: candidate.range.start..terminator.saturating_add(2),
                            domain: ParserDomain::Ftml,
                            own_emit: Some(leaf),
                            child_emit,
                        },
                    )
                }
                ParserOwnerCertainty::ProtectionOnly => {
                    ParserCandidate::policy(candidate.range.start, PRECEDENCE_COLOR, emit)
                }
            }
        })
        .collect()
}

fn adapt_exact_ranges(
    arena: &mut EmitSetArena,
    ranges: Vec<Range<usize>>,
    domain: ParserDomain,
    precedence: u16,
    namespace: u16,
) -> Vec<ParserCandidate> {
    ranges
        .into_iter()
        .map(|range| {
            let (leaf, _) = arena.leaf(range.clone());
            ParserCandidate::exact(
                precedence,
                Some(ParserDelimiterIdentity {
                    namespace,
                    start: range.start,
                }),
                None,
                ExactEffect {
                    claim: range,
                    domain,
                    own_emit: Some(leaf),
                    child_emit: EmitSetArena::EMPTY,
                },
            )
        })
        .collect()
}

fn adapt_delimiter(identity: DelimiterIdentity) -> ParserDelimiterIdentity {
    ParserDelimiterIdentity {
        namespace: DELIMITER_NAMESPACE_BASE + identity.kind as u16,
        start: identity.start,
    }
}
