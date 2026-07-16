/*
 * services/render/literal_regions/base_candidates/tests.rs
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

use super::*;

fn runtime_candidates(source: &str) -> Vec<BaseCandidate> {
    collect_base_candidates(source, BaseCandidatePolicy::FAIL_CLOSED_RUNTIME)
}

#[test]
fn later_comment_survives_an_overlapping_raw_candidate() {
    let source = concat!(
        "[https://e.test/ @@ label] ",
        "[!-- [[module ListPages name=\"hidden\"]]X[[/module]] --] @@",
    );
    let candidates = runtime_candidates(source);
    let raw_open = source.find("@@").unwrap();
    let comment_open = source.find("[!--").unwrap();
    let raw_close = source.rfind("@@").unwrap();

    assert_eq!(candidates.len(), 3);
    assert_eq!(candidates[0].range, raw_open..raw_close + 2);
    assert_eq!(
        candidates[0].provenance,
        BaseCandidateProvenance::ClosedOwner
    );
    assert_eq!(candidates[0].terminator.unwrap().start, raw_close);
    assert_eq!(candidates[1].opener.start, comment_open);
    assert_eq!(
        candidates[1].provenance,
        BaseCandidateProvenance::ClosedOwner
    );
    assert_eq!(candidates[2].opener.start, raw_close);
    assert_eq!(
        candidates[2].provenance,
        BaseCandidateProvenance::FailClosedProtection,
    );
}

#[test]
fn a_raw_terminator_remains_a_potential_unclosed_opener() {
    let source = "[https://e.test/ @@ x] @@ [[module ListPages limit=1]]Y[[/module]]";
    let candidates = runtime_candidates(source);
    let inner = source.find("@@").unwrap();
    let outer = source.rfind("@@").unwrap();

    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].range, inner..outer + 2);
    assert_eq!(candidates[0].terminator.unwrap().start, outer);
    assert_eq!(candidates[1].range, outer..source.len());
    assert_eq!(
        candidates[1].provenance,
        BaseCandidateProvenance::FailClosedProtection,
    );
}

#[test]
fn enumerates_every_raw_token_in_compact_runs() {
    let source = "@@@@@@@@ live";
    let candidates = runtime_candidates(source);

    assert_eq!(
        candidates
            .iter()
            .map(|candidate| (
                candidate.opener.start,
                candidate.range.clone(),
                candidate.terminator.map(|token| token.start),
            ))
            .collect::<Vec<_>>(),
        vec![
            (0, 0..6, Some(4)),
            (2, 2..8, Some(6)),
            (4, 4..8, Some(6)),
            (6, 6..source.len(), None),
        ],
    );
}

#[test]
fn emits_all_closed_and_fail_closed_inline_families() {
    let source = concat!(
        "@<angle>@ [[$ math $]] [!-- comment --]\n",
        "@<open\n[[$ open\n[!-- open",
    );
    let candidates = runtime_candidates(source);

    assert_eq!(candidates.len(), 6);
    assert!(candidates[..3]
        .iter()
        .all(|candidate| candidate.provenance == BaseCandidateProvenance::ClosedOwner),);
    assert_eq!(
        candidates[..3]
            .iter()
            .map(|candidate| candidate.terminator.unwrap().kind)
            .collect::<Vec<_>>(),
        vec![
            DelimiterKind::RightRaw,
            DelimiterKind::InlineMathClose,
            DelimiterKind::CommentClose,
        ],
    );
    assert!(candidates[3..].iter().all(|candidate| {
        candidate.provenance == BaseCandidateProvenance::FailClosedProtection
    }));
    assert_eq!(
        candidates[3].range.end,
        source.find('\n').unwrap() + 1 + "@<open".len()
    );
    assert_eq!(candidates[5].range.end, source.len());
}

#[test]
fn overlapping_inline_math_close_is_not_a_token() {
    let source = "[[$]]\nlive";
    let candidates = runtime_candidates(source);

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].range, 0.."[[$]]".len());
    assert_eq!(candidates[0].opener.kind, DelimiterKind::InlineMathOpen);
    assert_eq!(
        candidates[0].provenance,
        BaseCandidateProvenance::FailClosedProtection,
    );
    assert_eq!(candidates[0].terminator, None);
}

#[test]
fn text_token_delimiters_are_not_candidate_events() {
    let source = concat!(
        "https://e.test/a@@b foo@bar.example@<tail ",
        "[!-- live --]",
    );
    let candidates = runtime_candidates(source);

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].opener.kind, DelimiterKind::CommentOpen);
}

#[test]
fn compact_comment_uses_the_pinned_close_token() {
    let source = "[!----]";
    let candidates = runtime_candidates(source);

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].range, 0..source.len());
    assert_eq!(
        candidates[0].provenance,
        BaseCandidateProvenance::ClosedOwner,
    );
    assert_eq!(
        candidates[0].terminator,
        Some(DelimiterIdentity {
            kind: DelimiterKind::CommentClose,
            start: 4,
        }),
    );
}

#[test]
fn compact_nested_comment_close_remains_available_to_each_opener() {
    let source = "[!--[!----]";
    let candidates = runtime_candidates(source);

    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].range, 0..source.len());
    assert_eq!(candidates[1].range, 4..source.len());
    assert!(candidates.iter().all(|candidate| {
        candidate.provenance == BaseCandidateProvenance::ClosedOwner
            && candidate.terminator.unwrap().start == 8
    }));
}

#[test]
fn bracket_token_precedence_controls_math_and_comment_openers() {
    for source in ["[[[$x$]]", "[[!--x--]"] {
        assert!(runtime_candidates(source).is_empty(), "{source:?}");
    }

    for (source, kind, start) in [
        ("[[[[[[$x$]]", DelimiterKind::InlineMathOpen, 4),
        ("[[[[[!--x--]", DelimiterKind::CommentOpen, 4),
    ] {
        let candidates = runtime_candidates(source);
        assert_eq!(candidates.len(), 1, "{source:?}");
        assert_eq!(candidates[0].opener.kind, kind, "{source:?}");
        assert_eq!(candidates[0].opener.start, start, "{source:?}");
        assert_eq!(
            candidates[0].provenance,
            BaseCandidateProvenance::ClosedOwner,
            "{source:?}",
        );
    }
}

#[test]
fn odd_raw_tail_remains_an_angle_raw_opener() {
    let source = "@@@@@<x>@";
    let candidates = runtime_candidates(source);

    assert_eq!(candidates.len(), 3);
    assert_eq!(candidates[0].range, 0..4);
    assert_eq!(candidates[0].terminator.unwrap().start, 2);
    assert_eq!(
        candidates[1].provenance,
        BaseCandidateProvenance::FailClosedProtection,
    );
    assert_eq!(candidates[1].opener.start, 2);
    assert_eq!(candidates[2].opener.kind, DelimiterKind::LeftRaw);
    assert_eq!(candidates[2].opener.start, 4);
    assert_eq!(candidates[2].range, 4..source.len());
}

#[test]
fn url_suffix_bytes_do_not_hide_pinned_right_raw_tokens() {
    for source in ["@<https://e.test/a>>@", "@<https://e.test/a~~~>@"] {
        let candidates = runtime_candidates(source);
        assert_eq!(candidates.len(), 1, "{source:?}");
        assert_eq!(candidates[0].range, 0..source.len(), "{source:?}");
        assert_eq!(
            candidates[0].terminator.unwrap().kind,
            DelimiterKind::RightRaw,
            "{source:?}",
        );
    }
}

#[test]
fn closed_only_policy_omits_protection_candidates() {
    let source = "@@ open\n@<open\n[[$ open\n[!-- open";
    assert!(
        collect_base_candidates(source, BaseCandidatePolicy::CLOSED_OWNERS_ONLY)
            .is_empty(),
    );
}

#[test]
fn dense_raw_tokens_are_all_enumerated_without_coalescing() {
    const TOKENS: usize = 20_000;
    let source = "@@".repeat(TOKENS);
    let candidates = runtime_candidates(&source);

    assert_eq!(candidates.len(), TOKENS);
    assert_eq!(candidates.first().unwrap().opener.start, 0);
    assert_eq!(candidates.last().unwrap().opener.start, source.len() - 2);
}
