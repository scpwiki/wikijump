/*
 * services/render/literal_regions/block_candidates/generic_heads.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use super::HeadContext;
use crate::services::render::literal_regions::token_boundaries::{
    TextTokenCursor, TextTokenIndex, WikidotTagScan, WikidotWholeHeadScan,
    left_block_start_in_run, right_bracket_token, scan_wikidot_tag,
    scan_wikidot_whole_head_value, wikidot_trimmed_name,
};
use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::services::render::literal_regions) enum RuntimeModuleHeadCandidate {
    Exact(Range<usize>),
    RecoveryBarrier(Range<usize>),
}

pub(in crate::services::render::literal_regions) struct HeadCandidateStreams {
    pub(in crate::services::render::literal_regions) generic:
        Vec<RuntimeModuleHeadCandidate>,
    pub(in crate::services::render::literal_regions) runtime_modules:
        Vec<RuntimeModuleHeadCandidate>,
}

/// Enumerate complete heads consumed by pinned FTML block rules.
///
/// The collector deliberately does not mask nested candidates. Unknown and
/// malformed heads emit nothing, allowing a valid runtime module inside them
/// to be reconsidered by the global selector.
#[cfg(test)]
pub(in crate::services::render::literal_regions) fn collect_generic_head_candidates(
    source: &str,
) -> Vec<Range<usize>> {
    if source.len() >= u32::MAX as usize {
        return Vec::new();
    }
    let heads = HeadContext::new(source);
    collect_head_candidate_streams_with_heads(source, &heads)
        .generic
        .into_iter()
        .filter_map(|candidate| match candidate {
            RuntimeModuleHeadCandidate::Exact(range) => Some(range),
            RuntimeModuleHeadCandidate::RecoveryBarrier(_) => None,
        })
        .collect()
}

pub(in crate::services::render::literal_regions) fn collect_head_candidate_streams(
    source: &str,
) -> HeadCandidateStreams {
    if source.len() >= u32::MAX as usize {
        return HeadCandidateStreams {
            generic: Vec::new(),
            runtime_modules: Vec::new(),
        };
    }
    let text_tokens = TextTokenIndex::new(source);
    let heads = HeadContext::new_with_text_tokens(source, &text_tokens);
    collect_head_candidate_streams_with_context(source, &heads, &text_tokens)
}

#[cfg(test)]
pub(in crate::services::render::literal_regions) fn collect_head_candidate_streams_with_heads(
    source: &str,
    heads: &HeadContext,
) -> HeadCandidateStreams {
    let text_tokens = TextTokenIndex::new(source);
    collect_head_candidate_streams_with_context(source, heads, &text_tokens)
}

pub(in crate::services::render::literal_regions) fn collect_head_candidate_streams_with_context(
    source: &str,
    heads: &HeadContext,
    text_tokens: &TextTokenIndex,
) -> HeadCandidateStreams {
    let bytes = source.as_bytes();
    let mut text_tokens = text_tokens.cursor();
    let mut generic = Vec::new();
    let mut runtime_modules = Vec::new();
    let mut cursor = 0usize;
    while let Some(relative) = source[cursor..].find("[[") {
        let candidate = cursor + relative;
        let (block_start, run_end) = left_block_start_in_run(bytes, candidate);
        cursor = candidate + 1;
        if block_start != Some(candidate)
            || text_tokens.contains(candidate)
            || matches!(bytes.get(candidate + 2), Some(b'/' | b'#' | b'$'))
        {
            cursor = cursor.max(run_end);
            continue;
        }

        let mut name_start = candidate + 2;
        let starred = bytes.get(name_start) == Some(&b'*');
        name_start += usize::from(starred);
        name_start = skip_horizontal(bytes, name_start);
        let (Some(name), name_end) = wikidot_trimmed_name(bytes, name_start) else {
            continue;
        };
        let name = name.strip_suffix(b"_").unwrap_or(name);
        if !starred
            && (name.eq_ignore_ascii_case(b"module")
                || name.eq_ignore_ascii_case(b"module654"))
            && let Some(subname_start) = skip_name_delimiter(bytes, name_end)
        {
            let (subname, _) = wikidot_trimmed_name(bytes, subname_start);
            if subname.is_some_and(|subname| {
                subname.eq_ignore_ascii_case(b"ListPages")
                    || subname.eq_ignore_ascii_case(b"CountPages")
            }) {
                let mut target_tokens = text_tokens.clone();
                match scan_wikidot_tag(
                    bytes,
                    candidate,
                    bytes.len(),
                    true,
                    false,
                    &mut target_tokens,
                ) {
                    WikidotTagScan::Complete(end) => runtime_modules
                        .push(RuntimeModuleHeadCandidate::Exact(candidate..end)),
                    WikidotTagScan::Malformed { .. } | WikidotTagScan::Unclosed => {
                        runtime_modules.push(
                            RuntimeModuleHeadCandidate::RecoveryBarrier(
                                candidate..physical_line_resume(bytes, candidate),
                            ),
                        );
                    }
                }
                continue;
            }
        }
        let recognized = is_name_map_block(name)
            || is_whole_value_block(name)
            || is_no_head_block(name)
            || is_map_block(name);
        let end = if is_name_map_block(name) {
            name_map_end(bytes, name, name_end, heads)
        } else if is_whole_value_block(name) {
            whole_value_end(bytes, name_end, &mut text_tokens)
        } else if is_no_head_block(name) {
            no_head_end(bytes, name_end)
        } else if is_map_block(name) {
            map_end(heads, name_end)
        } else {
            None
        };
        if recognized && (!starred || accepts_star(name)) {
            if let Some(end) = end {
                generic.push(RuntimeModuleHeadCandidate::Exact(candidate..end));
            } else if !name.eq_ignore_ascii_case(b"module")
                && !name.eq_ignore_ascii_case(b"module654")
            {
                generic.push(RuntimeModuleHeadCandidate::RecoveryBarrier(
                    candidate..physical_line_resume(bytes, candidate),
                ));
            }
        }
    }
    HeadCandidateStreams {
        generic,
        runtime_modules,
    }
}

fn physical_line_resume(bytes: &[u8], start: usize) -> usize {
    let mut cursor = start;
    while !matches!(bytes.get(cursor), None | Some(b'\n' | b'\r')) {
        cursor += 1;
    }
    match bytes.get(cursor) {
        Some(b'\r') if bytes.get(cursor + 1) == Some(&b'\n') => cursor + 2,
        Some(b'\n' | b'\r') => cursor + 1,
        _ => cursor,
    }
}

fn map_end(heads: &HeadContext, start: usize) -> Option<usize> {
    heads.map_end(start)
}

fn name_map_end(
    bytes: &[u8],
    name: &[u8],
    mut cursor: usize,
    heads: &HeadContext,
) -> Option<usize> {
    cursor = skip_name_delimiter(bytes, cursor)?;
    let (subname, subname_end) = wikidot_trimmed_name(bytes, cursor);
    let subname = subname?;
    if (name.eq_ignore_ascii_case(b"module") || name.eq_ignore_ascii_case(b"module654"))
        && (subname.eq_ignore_ascii_case(b"ListPages")
            || subname.eq_ignore_ascii_case(b"CountPages"))
    {
        return None;
    }
    map_end(heads, subname_end)
}

fn whole_value_end(
    bytes: &[u8],
    name_end: usize,
    text_tokens: &mut TextTokenCursor,
) -> Option<usize> {
    match scan_wikidot_whole_head_value(bytes, name_end, bytes.len(), text_tokens) {
        WikidotWholeHeadScan::Complete { end, .. } => Some(end),
        WikidotWholeHeadScan::Malformed { .. }
        | WikidotWholeHeadScan::Unclosed { .. } => None,
    }
}

fn no_head_end(bytes: &[u8], mut cursor: usize) -> Option<usize> {
    cursor = skip_argument_spacing(bytes, cursor);
    let (right_block, token_len) = right_bracket_token(bytes, cursor, bytes.len());
    right_block.then_some(cursor + token_len)
}

fn skip_horizontal(bytes: &[u8], mut cursor: usize) -> usize {
    while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
        cursor += 1;
    }
    cursor
}

fn skip_argument_spacing(bytes: &[u8], mut cursor: usize) -> usize {
    loop {
        cursor = skip_horizontal(bytes, cursor);
        match bytes.get(cursor) {
            Some(b'\r') if bytes.get(cursor + 1) == Some(&b'\n') => cursor += 2,
            Some(b'\r' | b'\n') => cursor += 1,
            _ => return cursor,
        }
    }
}

fn skip_name_delimiter(bytes: &[u8], cursor: usize) -> Option<usize> {
    match bytes.get(cursor) {
        Some(b' ' | b'\t') => Some(skip_horizontal(bytes, cursor)),
        Some(b'\r' | b'\n') => Some(skip_argument_spacing(bytes, cursor)),
        _ => None,
    }
}

fn is_name_map_block(name: &[u8]) -> bool {
    matches_name(
        name,
        &[
            "audio",
            "date",
            "embed",
            "iframe",
            "image",
            "=image",
            "<image",
            ">image",
            "f<image",
            "f>image",
            "include-elements",
            "module",
            "module654",
            "radio",
            "radio-button",
            "video",
        ],
    )
}

fn is_whole_value_block(name: &[u8]) -> bool {
    matches_name(
        name,
        &[
            "anchortarget",
            "bibcite",
            "char",
            "character",
            "equation",
            "eqref",
            "eref",
            "ifcategory",
            "iftags",
            "lines",
            "math",
            "newlines",
            "rb",
            "ruby2",
            "size",
            "tab",
            "target",
            "user",
        ],
    )
}

fn is_no_head_block(name: &[u8]) -> bool {
    matches_name(name, &["footnote", "later", "tabview", "tabs"])
}

fn is_map_block(name: &[u8]) -> bool {
    matches_name(
        name,
        &[
            "a",
            "anchor",
            "b",
            "bibliography",
            "blockquote",
            "bold",
            "cell",
            "checkbox",
            "code",
            "collapsible",
            "del",
            "deletion",
            "div",
            "em",
            "emphasis",
            "footnoteblock",
            "hcell",
            "hidden",
            "highlight",
            "html",
            "i",
            "include",
            "ins",
            "insertion",
            "invisible",
            "italics",
            "li",
            "mark",
            "mono",
            "monospace",
            "ol",
            "p",
            "paragraph",
            "quote",
            "raw",
            "row",
            "ruby",
            "s",
            "span",
            "strikethrough",
            "strong",
            "sub",
            "subscript",
            "sup",
            "super",
            "superscript",
            "table",
            "toc",
            "f<toc",
            "f>toc",
            "tt",
            "u",
            "ul",
            "underline",
        ],
    )
}

fn accepts_star(name: &[u8]) -> bool {
    matches_name(
        name,
        &["a", "anchor", "image", "=image", "<image", ">image"],
    )
}

fn matches_name(name: &[u8], accepted: &[&str]) -> bool {
    accepted
        .iter()
        .any(|accepted| name.eq_ignore_ascii_case(accepted.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::collect_generic_head_candidates;

    #[test]
    fn complete_known_heads_own_only_their_head() {
        let source = "[[span title=\"[[module ListPages name='hidden']]\"]] tail";
        assert_eq!(
            collect_generic_head_candidates(source),
            vec![0..source.find(" tail").unwrap()]
        );
    }

    #[test]
    fn runtime_module_heads_do_not_mask_themselves() {
        for name in ["ListPages", "CountPages"] {
            let source = format!("[[module {name} name=\"live\"]]");
            assert!(collect_generic_head_candidates(&source).is_empty());
        }
    }

    #[test]
    fn unknown_and_malformed_heads_roll_back() {
        for source in [
            "[[unknown value=\"x\"]]",
            "[[span title='x']]",
            "[[span title=\"unterminated]]",
        ] {
            assert!(
                collect_generic_head_candidates(source).is_empty(),
                "{source:?}"
            );
        }
    }
}
