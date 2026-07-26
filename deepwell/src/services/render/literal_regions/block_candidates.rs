/*
 * services/render/literal_regions/block_candidates.rs
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

mod generic_heads;
mod head_index;
mod quote_lines;

pub(super) use self::generic_heads::{
    RuntimeModuleHeadCandidate, collect_head_candidate_streams,
    collect_head_candidate_streams_with_context,
};
pub(super) use self::head_index::HeadContext;
use self::quote_lines::{PhysicalLine, collect_physical_lines};

use super::base_candidates::{BaseCandidateProvenance, DelimiterIdentity, DelimiterKind};
use super::token_boundaries::{left_block_start_in_run, wikidot_trimmed_name};
use std::ops::Range;

const NO_OFFSET: u32 = u32::MAX;
const MAX_NATIVE_QUOTE_DEPTH: usize = 30;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BlockCandidate {
    pub(super) range: Range<usize>,
    pub(super) provenance: BaseCandidateProvenance,
    pub(super) opener: DelimiterIdentity,
    /// This identity becomes unavailable only if the global selector chooses this candidate.
    pub(super) terminator: Option<DelimiterIdentity>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum BlockFamily {
    Code,
    Html,
    Raw,
    Math,
    Embed,
}

impl BlockFamily {
    fn index(self) -> usize {
        match self {
            Self::Code => 0,
            Self::Html => 1,
            Self::Raw => 2,
            Self::Math => 3,
            Self::Embed => 4,
        }
    }

    fn open_kind(self) -> DelimiterKind {
        match self {
            Self::Code => DelimiterKind::CodeOpen,
            Self::Html => DelimiterKind::HtmlOpen,
            Self::Raw => DelimiterKind::RawBlockOpen,
            Self::Math => DelimiterKind::MathBlockOpen,
            Self::Embed => DelimiterKind::EmbedOpen,
        }
    }

    fn close_kind(self) -> DelimiterKind {
        match self {
            Self::Code => DelimiterKind::CodeClose,
            Self::Html => DelimiterKind::HtmlClose,
            Self::Raw => DelimiterKind::RawBlockClose,
            Self::Math => DelimiterKind::MathBlockClose,
            Self::Embed => DelimiterKind::EmbedClose,
        }
    }

    fn parser_accepts_quote(self) -> bool {
        matches!(self, Self::Html | Self::Math)
    }
}

#[derive(Clone, Copy)]
struct BlockOpener {
    family: BlockFamily,
    identity: DelimiterIdentity,
    content_start: usize,
    quote_depth: usize,
    boundary: usize,
    exact_head: bool,
    parser_reachable: bool,
}

#[derive(Clone, Copy)]
struct BlockClose {
    family: BlockFamily,
    identity: DelimiterIdentity,
    end: usize,
    quote_depth: usize,
}

/// Enumerate every recognizable literal block opener without selecting an owner.
///
/// FTML `4fc7df28` rejects quoted `code` and `raw`, but accepts quote-aware
/// `html` and `math`. Deepwell's pre-FTML runtime deliberately protects
/// recognizable malformed or unclosed heads and the compatibility-only
/// empty-head `embed` form. These cases retain protection provenance so the
/// global selector can still resurrect an overlapping exact parser candidate.
pub(super) fn collect_block_candidates(source: &str) -> Vec<BlockCandidate> {
    if source.len() >= NO_OFFSET as usize {
        return collect_oversized_fail_closed(source);
    }
    let heads = HeadContext::new(source);
    collect_block_candidates_with_heads(source, &heads)
}

pub(super) fn collect_block_candidates_with_heads(
    source: &str,
    heads: &HeadContext,
) -> Vec<BlockCandidate> {
    let bytes = source.as_bytes();
    let lines = collect_physical_lines(source);
    let mut openers = Vec::new();
    let mut closes = Vec::new();

    for (line_index, line) in lines.iter().enumerate() {
        let mut cursor = line.start;
        while cursor + 1 < line.body_end {
            if bytes[cursor] != b'[' {
                cursor += utf8_character_len(source, cursor);
                continue;
            }

            let run_start = cursor;
            while cursor < line.body_end && bytes[cursor] == b'[' {
                cursor += 1;
            }
            if cursor - run_start < 2 {
                continue;
            }
            let (block_start, _) = left_block_start_in_run(bytes, run_start);
            let Some(block_start) = block_start else {
                continue;
            };

            if bytes.get(block_start + 2) == Some(&b'/') {
                if let Some(close) = parse_block_close(bytes, block_start, line, heads) {
                    closes.push(close);
                }
            } else if let Some(opener) =
                parse_block_opener(source, block_start, line_index, &lines, heads)
            {
                openers.push(opener);
            }
        }
    }

    pair_candidates(openers, closes)
}

fn collect_oversized_fail_closed(source: &str) -> Vec<BlockCandidate> {
    let bytes = source.as_bytes();
    let mut candidates = Vec::new();
    let mut cursor = 0usize;
    while cursor + 1 < bytes.len() {
        if bytes[cursor] != b'[' {
            cursor += utf8_character_len(source, cursor);
            continue;
        }
        let run_start = cursor;
        while cursor < bytes.len() && bytes[cursor] == b'[' {
            cursor += 1;
        }
        if cursor - run_start < 2 {
            continue;
        }
        let (Some(start), _) = left_block_start_in_run(bytes, run_start) else {
            continue;
        };
        if bytes.get(start + 2) == Some(&b'/') {
            continue;
        }
        let Some((family, _)) = block_family_at(bytes, start + 2) else {
            continue;
        };
        candidates.push(BlockCandidate {
            range: start..source.len(),
            provenance: BaseCandidateProvenance::FailClosedProtection,
            opener: DelimiterIdentity {
                kind: family.open_kind(),
                start,
            },
            terminator: None,
        });
    }
    candidates
}

fn compact_offset(offset: usize) -> u32 {
    debug_assert!(offset < NO_OFFSET as usize);
    offset as u32
}

fn expanded_offset(offset: u32) -> usize {
    offset as usize
}

fn parse_block_opener(
    source: &str,
    start: usize,
    line_index: usize,
    lines: &[PhysicalLine],
    heads: &HeadContext,
) -> Option<BlockOpener> {
    let bytes = source.as_bytes();
    let (family, name_end) = block_family_at(bytes, start + 2)?;
    let line = lines[line_index];

    let (content_start, exact_head) = match family {
        BlockFamily::Code | BlockFamily::Html => match heads.map_head_end[name_end] {
            NO_OFFSET => (name_end, false),
            end => {
                let content_start = expanded_offset(end);
                let exact = family != BlockFamily::Html
                    || source[name_end..content_start - 2].trim().is_empty();
                (content_start, exact)
            }
        },
        BlockFamily::Raw => match bytes.get(name_end) {
            Some(_) if expanded_offset(heads.whitespace_end[name_end]) > name_end => {
                (name_end, true)
            }
            Some(b']')
                if heads.next_generic_right_block[name_end]
                    == compact_offset(name_end) =>
            {
                (name_end + 2, true)
            }
            _ => (name_end, false),
        },
        BlockFamily::Math => {
            let close = expanded_offset(heads.next_wikidot_right_block[name_end]);
            if close < line.body_end {
                (close + 2, true)
            } else {
                (name_end, false)
            }
        }
        BlockFamily::Embed => {
            let close = expanded_offset(heads.next_generic_right_block[name_end]);
            if close < line.body_end {
                if expanded_offset(heads.whitespace_end[name_end]) >= close {
                    (close + 2, false)
                } else {
                    // Named service embeds are bodyless and are not literal candidates.
                    return None;
                }
            } else {
                (name_end, false)
            }
        }
    };

    let boundary = if line.native_quote_depth == 0 {
        source.len()
    } else {
        line.shallower_start
    };
    Some(BlockOpener {
        family,
        identity: DelimiterIdentity {
            kind: family.open_kind(),
            start,
        },
        content_start,
        quote_depth: line.native_quote_depth,
        boundary,
        exact_head,
        parser_reachable: !line.tight_quote_prefix
            && line.native_quote_depth <= MAX_NATIVE_QUOTE_DEPTH,
    })
}

fn parse_block_close(
    bytes: &[u8],
    start: usize,
    line: &PhysicalLine,
    heads: &HeadContext,
) -> Option<BlockClose> {
    debug_assert_eq!(bytes.get(start..start + 3), Some(&b"[[/"[..]));
    let mut cursor = skip_horizontal_whitespace(bytes, start + 3);
    let (name, name_end) = wikidot_trimmed_name(bytes, cursor);
    let mut name = name?;
    if let Some(without_score) = name.strip_suffix(b"_") {
        name = without_score;
    }
    let family = block_family(name)?;
    cursor = skip_horizontal_whitespace(bytes, name_end);
    if heads.next_generic_right_block[cursor] != compact_offset(cursor) {
        return None;
    }
    let end = cursor + 2;
    Some(BlockClose {
        family,
        identity: DelimiterIdentity {
            kind: family.close_kind(),
            start,
        },
        end,
        quote_depth: line.quote_depth,
    })
}

fn block_family_at(bytes: &[u8], mut cursor: usize) -> Option<(BlockFamily, usize)> {
    cursor = skip_horizontal_whitespace(bytes, cursor);
    let (name, name_end) = wikidot_trimmed_name(bytes, cursor);
    Some((block_family(name?)?, name_end))
}

fn block_family(name: &[u8]) -> Option<BlockFamily> {
    if name.eq_ignore_ascii_case(b"code") {
        Some(BlockFamily::Code)
    } else if name.eq_ignore_ascii_case(b"html") {
        Some(BlockFamily::Html)
    } else if name.eq_ignore_ascii_case(b"raw") {
        Some(BlockFamily::Raw)
    } else if name.eq_ignore_ascii_case(b"math") {
        Some(BlockFamily::Math)
    } else if name.eq_ignore_ascii_case(b"embed") {
        Some(BlockFamily::Embed)
    } else {
        None
    }
}

fn pair_candidates(
    openers: Vec<BlockOpener>,
    closes: Vec<BlockClose>,
) -> Vec<BlockCandidate> {
    let mut next_any = [NO_OFFSET; 5];
    let mut next_at_depth = [[NO_OFFSET; 5]; MAX_NATIVE_QUOTE_DEPTH + 1];
    let mut matches = vec![NO_OFFSET; openers.len()];
    let mut close_cursor = closes.len();
    let query_order = opener_indices_by_content_start(&openers);
    for opener_index in query_order.into_iter().rev().map(expanded_offset) {
        let opener = openers[opener_index];
        while close_cursor > 0
            && closes[close_cursor - 1].identity.start >= opener.content_start
        {
            close_cursor -= 1;
            let close = closes[close_cursor];
            let close_index = compact_offset(close_cursor);
            next_any[close.family.index()] = close_index;
            if close.quote_depth <= MAX_NATIVE_QUOTE_DEPTH {
                next_at_depth[close.quote_depth][close.family.index()] = close_index;
            }
        }
        let close_index = if opener.quote_depth == 0 {
            next_any[opener.family.index()]
        } else if opener.quote_depth <= MAX_NATIVE_QUOTE_DEPTH {
            next_at_depth[opener.quote_depth][opener.family.index()]
        } else {
            NO_OFFSET
        };
        if close_index != NO_OFFSET
            && closes[expanded_offset(close_index)].identity.start < opener.boundary
        {
            matches[opener_index] = close_index;
        }
    }

    openers
        .into_iter()
        .enumerate()
        .map(|(index, opener)| {
            let close = (matches[index] != NO_OFFSET)
                .then(|| closes[expanded_offset(matches[index])]);
            let exact = close.is_some()
                && opener.exact_head
                && opener.parser_reachable
                && opener.family != BlockFamily::Embed
                && (opener.quote_depth == 0 || opener.family.parser_accepts_quote());
            BlockCandidate {
                range: opener.identity.start
                    ..close.map_or(opener.boundary, |close| close.end),
                provenance: if exact {
                    BaseCandidateProvenance::ClosedOwner
                } else {
                    BaseCandidateProvenance::FailClosedProtection
                },
                opener: opener.identity,
                terminator: close.map(|close| close.identity),
            }
        })
        .collect()
}

fn opener_indices_by_content_start(openers: &[BlockOpener]) -> Vec<u32> {
    let mut from = (0..openers.len()).map(compact_offset).collect::<Vec<_>>();
    let mut to = vec![0u32; openers.len()];
    for shift in [0, 16] {
        let mut counts = vec![0u32; 1 << 16];
        for &index in &from {
            let key = compact_offset(openers[expanded_offset(index)].content_start);
            counts[((key >> shift) & 0xffff) as usize] += 1;
        }
        let mut next = 0u32;
        for count in &mut counts {
            let current = *count;
            *count = next;
            next += current;
        }
        for &index in &from {
            let key = compact_offset(openers[expanded_offset(index)].content_start);
            let bucket = ((key >> shift) & 0xffff) as usize;
            to[expanded_offset(counts[bucket])] = index;
            counts[bucket] += 1;
        }
        std::mem::swap(&mut from, &mut to);
    }
    from
}

fn skip_horizontal_whitespace(bytes: &[u8], mut cursor: usize) -> usize {
    while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
        cursor += 1;
    }
    cursor
}

fn utf8_character_len(source: &str, cursor: usize) -> usize {
    source[cursor..]
        .chars()
        .next()
        .expect("cursor is before the UTF-8 source end")
        .len_utf8()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ftml::data::{PageInfo, ScoreValue};
    use ftml::layout::Layout;
    use ftml::settings::{WikitextMode, WikitextSettings};
    use ftml::tree::Element;
    use std::borrow::Cow;

    fn candidates(source: &str) -> Vec<BlockCandidate> {
        collect_block_candidates(source)
    }

    fn pinned_tree(source: &str) -> ftml::tree::SyntaxTree<'static> {
        assert!(source.len() <= 256);
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
        let mut source = source.to_owned();
        ftml::preprocess_for_layout(&mut source, settings.layout);
        let tokenization = ftml::tokenize(&source);
        let (tree, _) = ftml::parse(&tokenization, &page_info, &settings).into();
        tree.to_owned()
    }

    #[test]
    fn distinguishes_exact_parser_owners_from_runtime_protection() {
        let source = concat!(
            "[[code type=\"rust\"]]code[[/code]]\n",
            "[[raw]]raw[[/raw]]\n",
            "> [[code]]\n> quoted code\n> [[/code]]\n",
            "> [[raw]]\n> quoted raw\n> [[/raw]]\n",
            "> [[html]]\n> html\n> [[/html]]\n",
            "> [[math name]]\n> x\n> [[/math]]\n",
            "[[embed]]legacy[[/embed]]\n",
            "[[code @=\"bad\"]]guarded[[/code]]",
        );
        let candidates = candidates(source);

        assert_eq!(candidates.len(), 8);
        for index in [0, 1, 4, 5] {
            assert_eq!(
                candidates[index].provenance,
                BaseCandidateProvenance::ClosedOwner,
                "candidate {index}: {:?}",
                candidates[index],
            );
        }
        for index in [2, 3, 6, 7] {
            assert_eq!(
                candidates[index].provenance,
                BaseCandidateProvenance::FailClosedProtection,
                "candidate {index}: {:?}",
                candidates[index],
            );
        }
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.terminator.is_some())
        );
    }

    #[test]
    fn quote_depth_selects_exact_closes_and_stops_at_shallower_lines() {
        let source = concat!(
            "> [[math]]\n",
            ">> [[/math]]\n",
            "> x\n",
            "> [[/math]]\n",
            "[[html]]\n",
            "> [[/html]]\n",
            "> [[html]]\n",
            "> unclosed\n",
            "outside",
        );
        let candidates = candidates(source);

        assert_eq!(candidates.len(), 3);
        let math_close = source.rfind("> [[/math]]\n").unwrap() + 2;
        assert_eq!(candidates[0].terminator.unwrap().start, math_close);
        assert_eq!(
            candidates[0].provenance,
            BaseCandidateProvenance::ClosedOwner
        );
        assert_eq!(
            candidates[1].terminator.unwrap().start,
            source.find("[[/html]]").unwrap(),
        );
        assert_eq!(
            candidates[1].provenance,
            BaseCandidateProvenance::ClosedOwner
        );
        assert_eq!(candidates[2].range.end, source.find("outside").unwrap());
        assert_eq!(candidates[2].terminator, None);
        assert_eq!(
            candidates[2].provenance,
            BaseCandidateProvenance::FailClosedProtection,
        );
    }

    #[test]
    fn pinned_map_head_oddities_determine_exact_provenance() {
        let sources = [
            "[[code garbage]]x[[/code]]",
            "[[code a=\"x\" trailing]]x[[/code]]",
            "[[code {$k}=\"v\"]]x[[/code]]",
            "[[code a\u{a0}=\"x\"]]x[[/code]]",
        ];
        for (source, exact) in sources.into_iter().zip([true, true, true, false]) {
            let candidates = candidates(source);
            assert_eq!(candidates.len(), 1, "{source:?}");
            assert_eq!(
                candidates[0].provenance == BaseCandidateProvenance::ClosedOwner,
                exact,
                "{source:?}: {:?}",
                candidates[0],
            );
            assert_eq!(
                !pinned_tree(source).code_blocks.is_empty(),
                exact,
                "{source:?}"
            );
        }

        let html = "[[html garbage]]x[[/html]]";
        assert_eq!(
            candidates(html)[0].provenance,
            BaseCandidateProvenance::FailClosedProtection,
        );
        assert!(pinned_tree(html).html_blocks.is_empty());
    }

    #[test]
    fn empty_math_is_owned_by_the_pinned_parser() {
        for source in ["[[math]][[/math]]", "> [[math]]\n> \u{a0}\n> [[/math]]"] {
            let candidates = candidates(source);
            assert_eq!(candidates.len(), 1, "{source:?}");
            assert_eq!(
                candidates[0].provenance,
                BaseCandidateProvenance::ClosedOwner,
                "{source:?}",
            );
            assert!(candidates[0].terminator.is_some(), "{source:?}");
        }
        assert!(
            pinned_tree("[[math]][[/math]]")
                .elements
                .iter()
                .any(|element| matches!(element, Element::Math { .. }))
        );
    }

    #[test]
    fn native_quote_context_distinguishes_tight_and_spaced_markers() {
        let tight = ">[[html]]\n>x\n>[[/html]]";
        let tight_candidates = candidates(tight);
        assert_eq!(tight_candidates.len(), 1);
        assert_eq!(
            tight_candidates[0].provenance,
            BaseCandidateProvenance::FailClosedProtection,
        );

        let spaced =
            concat!("> > [[html]]\n", "> > [[/html]]\n", "> x\n", "> [[/html]]",);
        let spaced_candidates = candidates(spaced);
        assert_eq!(spaced_candidates.len(), 1);
        assert_eq!(
            spaced_candidates[0].provenance,
            BaseCandidateProvenance::ClosedOwner,
        );
        assert_eq!(
            spaced_candidates[0].terminator.unwrap().start,
            spaced.rfind("[[/html]]").unwrap(),
        );

        let boundary = "> [[html]]\nunquoted [[/html]]";
        let boundary_candidates = candidates(boundary);
        assert_eq!(boundary_candidates.len(), 1);
        assert_eq!(
            boundary_candidates[0].range.end,
            boundary.find("unquoted").unwrap(),
        );
        assert_eq!(boundary_candidates[0].terminator, None);
        assert_eq!(
            boundary_candidates[0].provenance,
            BaseCandidateProvenance::FailClosedProtection,
        );
    }

    #[test]
    fn crossing_and_nested_openers_remain_independent_candidates() {
        let source = concat!(
            "[[code]]",
            "[[html]]",
            "[[code]]x",
            "[[/code]]",
            "y[[/html]]",
            "z[[/code]]",
        );
        let candidates = candidates(source);
        let first_code_close = source.find("[[/code]]").unwrap();

        assert_eq!(candidates.len(), 3);
        assert_eq!(candidates[0].terminator.unwrap().start, first_code_close);
        assert_eq!(
            candidates[1].terminator.unwrap().start,
            source.find("[[/html]]").unwrap()
        );
        assert_eq!(candidates[2].terminator.unwrap().start, first_code_close);
        assert_eq!(candidates[0].terminator, candidates[2].terminator);
        assert!(
            candidates
                .windows(2)
                .all(|pair| pair[0].opener.start < pair[1].opener.start)
        );
    }

    #[test]
    fn bare_code_head_closes_before_a_spilling_inner_text_link() {
        let source = "[[code foo]][[[target | hidden [[/code]] visible]]]";
        let candidates = candidates(source);
        let close = source.find("[[/code]]").unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].provenance,
            BaseCandidateProvenance::ClosedOwner,
        );
        assert_eq!(candidates[0].terminator.unwrap().start, close);
        assert_eq!(candidates[0].range.end, close + "[[/code]]".len());
    }

    #[test]
    fn spaced_residual_quote_html_owns_depth_one_listpages_text() {
        let source = concat!(
            "> > [[html]]\n",
            "> [[module ListPages category=\"*\"]]HIDDEN[[/module]]\n",
            "> [[/html]]\n",
            "VISIBLE",
        );
        let candidates = candidates(source);

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].provenance,
            BaseCandidateProvenance::ClosedOwner,
        );
        assert!(
            candidates[0]
                .range
                .contains(&source.find("HIDDEN").unwrap())
        );
        assert!(
            !candidates[0]
                .range
                .contains(&source.find("VISIBLE").unwrap())
        );
    }

    #[test]
    fn named_embed_is_not_a_literal_candidate() {
        let source = concat!(
            "[[embed youtube video=\"abc\"]]",
            "[[embed]]legacy[[/embed]]",
        );
        let candidates = candidates(source);

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].opener.start,
            source.find("[[embed]]").unwrap()
        );
        assert_eq!(
            candidates[0].provenance,
            BaseCandidateProvenance::FailClosedProtection,
        );
    }

    #[test]
    fn dense_crossing_and_malformed_heads_are_enumerated_without_skipping() {
        const COUNT: usize = 20_000;
        let mut source = "[[code]]".repeat(COUNT);
        source.push_str("x[[/code]]");
        let source_candidates = candidates(&source);

        assert_eq!(source_candidates.len(), COUNT);
        let terminator = source.rfind("[[/code]]").unwrap();
        assert!(source_candidates.iter().all(|candidate| {
            candidate
                .terminator
                .is_some_and(|close| close.start == terminator)
        }));

        let malformed = "[[code x=\"".repeat(COUNT);
        let malformed_candidates = candidates(&malformed);
        assert_eq!(malformed_candidates.len(), COUNT);
        assert!(malformed_candidates.iter().all(|candidate| {
            candidate.provenance == BaseCandidateProvenance::FailClosedProtection
                && candidate.range.end == malformed.len()
        }));

        let mut shared_suffix = "[[code a=\"".repeat(COUNT);
        shared_suffix.push('"');
        shared_suffix.push_str("]]x[[/code]]");
        let suffix_candidates = candidates(&shared_suffix);
        assert_eq!(suffix_candidates.len(), COUNT);
        assert!(suffix_candidates.iter().all(|candidate| {
            candidate.provenance == BaseCandidateProvenance::ClosedOwner
        }));

        let mut nested_embed = "[[embed ".repeat(COUNT);
        nested_embed.push_str("]]x[[/embed]]");
        let candidates = candidates(&nested_embed);
        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].opener.start,
            nested_embed.rfind("[[embed ").unwrap(),
        );
    }

    #[test]
    fn retained_head_index_uses_sixteen_bytes_per_source_byte() {
        let source = "x".repeat(1_000_000);
        let heads = HeadContext::new(&source);

        assert!(heads.retained_bytes() <= (source.len() + 1) * 16);
    }
}
