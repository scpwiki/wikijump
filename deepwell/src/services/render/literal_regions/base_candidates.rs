/*
 * services/render/literal_regions/base_candidates.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

mod token_stream;

#[cfg(test)]
mod tests;

use self::token_stream::{DelimiterIndex, DoubleAtToken, LineToken};
use super::token_boundaries::TextTokenIndex;
use std::ops::Range;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum DelimiterKind {
    DoubleAt,
    LeftRaw,
    RightRaw,
    InlineMathOpen,
    InlineMathClose,
    CommentOpen,
    CommentClose,
    CodeOpen,
    CodeClose,
    HtmlOpen,
    HtmlClose,
    RawBlockOpen,
    RawBlockClose,
    MathBlockOpen,
    MathBlockClose,
    EmbedOpen,
    EmbedClose,
}

impl DelimiterKind {
    fn source_order(self) -> u8 {
        match self {
            Self::DoubleAt => 0,
            Self::LeftRaw => 1,
            Self::RightRaw => 2,
            Self::InlineMathOpen => 3,
            Self::InlineMathClose => 4,
            Self::CommentOpen => 5,
            Self::CommentClose => 6,
            Self::CodeOpen => 7,
            Self::CodeClose => 8,
            Self::HtmlOpen => 9,
            Self::HtmlClose => 10,
            Self::RawBlockOpen => 11,
            Self::RawBlockClose => 12,
            Self::MathBlockOpen => 13,
            Self::MathBlockClose => 14,
            Self::EmbedOpen => 15,
            Self::EmbedClose => 16,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct DelimiterIdentity {
    pub(super) kind: DelimiterKind,
    pub(super) start: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BaseCandidateProvenance {
    /// The pinned parser consumes the complete candidate when its opener wins.
    ClosedOwner,
    /// Early runtime handling protects an unclosed candidate without claiming parser ownership.
    FailClosedProtection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BaseCandidate {
    pub(super) range: Range<usize>,
    pub(super) provenance: BaseCandidateProvenance,
    pub(super) opener: DelimiterIdentity,
    /// The delimiter becomes unavailable as an opener only if this candidate is selected.
    pub(super) terminator: Option<DelimiterIdentity>,
}

impl BaseCandidate {
    fn closed(
        range: Range<usize>,
        opener: DelimiterIdentity,
        terminator: DelimiterIdentity,
    ) -> Self {
        Self {
            range,
            provenance: BaseCandidateProvenance::ClosedOwner,
            opener,
            terminator: Some(terminator),
        }
    }

    fn protection(range: Range<usize>, opener: DelimiterIdentity) -> Self {
        Self {
            range,
            provenance: BaseCandidateProvenance::FailClosedProtection,
            opener,
            terminator: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct BaseCandidatePolicy {
    pub(super) protect_unclosed_inline: bool,
    pub(super) protect_unclosed_multiline: bool,
}

impl BaseCandidatePolicy {
    #[cfg(test)]
    pub(super) const CLOSED_OWNERS_ONLY: Self = Self {
        protect_unclosed_inline: false,
        protect_unclosed_multiline: false,
    };

    pub(super) const FAIL_CLOSED_RUNTIME: Self = Self {
        protect_unclosed_inline: true,
        protect_unclosed_multiline: true,
    };
}

/// Enumerate overlapping inline parser and protection candidates in source order.
///
/// Every emitted delimiter token remains a potential opener, including a token
/// that is also another candidate's terminator. The global owner selector may
/// consume that identity only after choosing the candidate that terminates
/// there. This collector performs no sibling-parser selection or pre-masking;
/// exact and protection candidates remain alternatives for one global pass.
#[cfg(test)]
pub(super) fn collect_base_candidates(
    source: &str,
    policy: BaseCandidatePolicy,
) -> Vec<BaseCandidate> {
    let text_tokens = TextTokenIndex::new(source);
    collect_base_candidates_with_text_tokens(source, policy, &text_tokens)
}

pub(super) fn collect_base_candidates_with_text_tokens(
    source: &str,
    policy: BaseCandidatePolicy,
    text_tokens: &TextTokenIndex,
) -> Vec<BaseCandidate> {
    let index = DelimiterIndex::new_with_text_tokens(source, text_tokens);
    merge_candidate_streams([
        collect_double_at_candidates(source, &index.double_at, policy),
        collect_paired_line_candidates(
            &index.left_raw,
            &index.right_raw,
            DelimiterKind::RightRaw,
            2,
            2,
            policy,
        ),
        collect_paired_line_candidates(
            &index.inline_math_open,
            &index.inline_math_close,
            DelimiterKind::InlineMathClose,
            3,
            3,
            policy,
        ),
        collect_comment_candidates(
            source.len(),
            &index.comment_open,
            &index.comment_close,
            policy,
        ),
    ])
}

fn collect_double_at_candidates(
    source: &str,
    tokens: &[DoubleAtToken],
    policy: BaseCandidatePolicy,
) -> Vec<BaseCandidate> {
    let bytes = source.as_bytes();
    let mut candidates = Vec::with_capacity(tokens.len());
    for (index, token) in tokens.iter().enumerate() {
        let start = token.token.identity.start;
        let adjacent = tokens
            .get(index + 1)
            .filter(|next| next.token.identity.start == start + 2);
        if let Some(next) = adjacent {
            let second = tokens
                .get(index + 2)
                .filter(|second| second.token.identity.start == start + 4);
            let (end, terminator) = if let Some(second) = second {
                (start + 6, second.token.identity)
            } else if bytes.get(start + 4) == Some(&b'@')
                && bytes.get(start + 5) != Some(&b'<')
            {
                (start + 5, next.token.identity)
            } else {
                (start + 4, next.token.identity)
            };
            candidates.push(BaseCandidate::closed(
                start..end,
                token.token.identity,
                terminator,
            ));
            continue;
        }

        let close = tokens
            .get(index + 1)
            .filter(|close| close.token.identity.start < token.token.line_end);
        if let Some(close) = close {
            candidates.push(BaseCandidate::closed(
                start..close.token.identity.start + 2,
                token.token.identity,
                close.token.identity,
            ));
        } else if policy.protect_unclosed_inline {
            candidates.push(BaseCandidate::protection(
                start..token.token.line_end,
                token.token.identity,
            ));
        }
    }
    candidates
}

fn collect_paired_line_candidates(
    openers: &[LineToken],
    closes: &[DelimiterIdentity],
    close_kind: DelimiterKind,
    opener_len: usize,
    close_len: usize,
    policy: BaseCandidatePolicy,
) -> Vec<BaseCandidate> {
    let mut candidates = Vec::with_capacity(openers.len());
    let mut close_index = 0usize;
    for opener in openers {
        while closes
            .get(close_index)
            .is_some_and(|close| close.start < opener.identity.start + opener_len)
        {
            close_index += 1;
        }
        let close = closes
            .get(close_index)
            .filter(|close| close.start < opener.line_end);
        if let Some(close) = close {
            debug_assert_eq!(close.kind, close_kind);
            candidates.push(BaseCandidate::closed(
                opener.identity.start..close.start + close_len,
                opener.identity,
                *close,
            ));
        } else if policy.protect_unclosed_inline {
            candidates.push(BaseCandidate::protection(
                opener.identity.start..opener.line_end,
                opener.identity,
            ));
        }
    }
    candidates
}

fn collect_comment_candidates(
    source_end: usize,
    openers: &[DelimiterIdentity],
    closes: &[DelimiterIdentity],
    policy: BaseCandidatePolicy,
) -> Vec<BaseCandidate> {
    let mut candidates = Vec::with_capacity(openers.len());
    let mut close_index = 0usize;
    for opener in openers {
        while closes
            .get(close_index)
            .is_some_and(|close| close.start < opener.start + 4)
        {
            close_index += 1;
        }
        if let Some(close) = closes.get(close_index) {
            candidates.push(BaseCandidate::closed(
                opener.start..close.start + 3,
                *opener,
                *close,
            ));
        } else if policy.protect_unclosed_multiline {
            candidates.push(BaseCandidate::protection(opener.start..source_end, *opener));
        }
    }
    candidates
}

fn merge_candidate_streams<const N: usize>(
    streams: [Vec<BaseCandidate>; N],
) -> Vec<BaseCandidate> {
    let mut indices = [0usize; N];
    let capacity = streams.iter().map(Vec::len).sum();
    let mut merged = Vec::with_capacity(capacity);
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
                            candidate.opener.kind.source_order(),
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
        merged.push(candidate.clone());
    }
    merged
}
