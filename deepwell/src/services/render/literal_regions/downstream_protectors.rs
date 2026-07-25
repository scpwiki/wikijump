/*
 * services/render/literal_regions/downstream_protectors.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

mod mapped_source;
mod pretransform_plan;

use self::mapped_source::{MappedSource, OriginRangeCursor};
use self::pretransform_plan::{
    DownstreamPretransformEffect, DownstreamPretransformStage,
    DownstreamProtectorPretransformPlan,
};
use super::LiteralRegionIndex;
use super::block_candidates::{
    RuntimeModuleHeadCandidate, collect_head_candidate_streams,
};
use super::list_pages_protection::collect_list_pages_downstream_css_ranges;
use regex::Regex;
use std::ops::Range;
use std::sync::LazyLock;

#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
thread_local! {
    static COLLECTOR_WORK: Cell<usize> = const { Cell::new(0) };
}

#[inline]
fn record_collector_work(units: usize) {
    #[cfg(test)]
    COLLECTOR_WORK.with(|work| work.set(work.get() + units));
    #[cfg(not(test))]
    let _ = units;
}

#[cfg(test)]
fn take_collector_work() -> usize {
    COLLECTOR_WORK.with(|work| work.replace(0))
}

// These expressions intentionally mirror the downstream protectors in
// render/service.rs. This module reports their original-source ownership
// without changing those legacy transformation paths.
static WIKIDOT_ANCHOR_MARKER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\[#\s+(?P<name>[^\]\n]+)\]\]")
        .expect("the Wikidot anchor marker regex is valid")
});
static WIKIDOT_CURRENT_PAGE_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[#\s+(?P<label>[^\]\n]+)\]")
        .expect("the Wikidot current-page link regex is valid")
});
static WIKIDOT_STAR_LOCAL_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\*/(?P<target>[^\s\]\n]+)\s+(?P<label>[^\]\n]+)\]")
        .expect("the Wikidot star-local link regex is valid")
});
static WIKIDOT_WIKIPEDIA_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[wikipedia:(?P<target>[^\s\]\n]+)(?:\s+(?P<label>[^\]\n]+))?\]")
        .expect("the Wikidot Wikipedia link regex is valid")
});

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::services::render) enum DownstreamProtectorFamily {
    AnchorMarker,
    CurrentPageLink,
    StarLocalLink,
    WikipediaLink,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::services::render) struct DownstreamProtectorRange {
    pub(in crate::services::render) range: Range<usize>,
    pub(in crate::services::render) family: DownstreamProtectorFamily,
    /// Whether captured content, excluding the protector's own opener and
    /// closer, contains a token start that can change literal/text ownership.
    pub(in crate::services::render) contains_ownership_delimiter: bool,
}

/// Return the exact original-source ranges replaced by the page-syntax-enabled
/// downstream compatibility-link pipeline.
///
/// Compatibility protectors run sequentially. CSS ranges are removed first,
/// then accepted link ranges become nonempty delimiter-free markers before
/// later regexes and literal guards run. The virtual source retains an origin
/// map so matches created across removals and matches spanning earlier markers
/// still report their full original-source ranges.
pub(in crate::services::render) fn collect_downstream_protector_ranges(
    source: &str,
) -> Vec<DownstreamProtectorRange> {
    let runtime_heads = collect_head_candidate_streams(source)
        .runtime_modules
        .into_iter()
        .filter_map(|candidate| match candidate {
            RuntimeModuleHeadCandidate::Exact(range) => Some(range),
            RuntimeModuleHeadCandidate::RecoveryBarrier(_) => None,
        })
        .collect::<Vec<_>>();
    collect_downstream_protector_ranges_with_runtime_heads(source, &runtime_heads)
}

pub(in crate::services::render::literal_regions) fn collect_downstream_protector_ranges_with_runtime_heads(
    source: &str,
    runtime_heads: &[Range<usize>],
) -> Vec<DownstreamProtectorRange> {
    let css_ranges = collect_list_pages_downstream_css_ranges(source);
    let mut plan = DownstreamProtectorPretransformPlan::default();
    plan.extend_exact_ranges(
        DownstreamPretransformStage::RuntimeModuleHead,
        runtime_heads.iter().cloned(),
    );
    plan.extend_exact_ranges(DownstreamPretransformStage::CssModule, css_ranges);
    collect_with_pretransform_plan(source, &plan)
}

fn collect_with_pretransform_plan(
    source: &str,
    plan: &DownstreamProtectorPretransformPlan,
) -> Vec<DownstreamProtectorRange> {
    plan.validate_for_source(source);
    let mut working = MappedSource::new(source);
    for batch in plan.batches() {
        match batch.effect {
            DownstreamPretransformEffect::Replace(replacement) => {
                working.replace_original_ranges_with_text(batch.ranges, replacement);
            }
            DownstreamPretransformEffect::Remove => {
                working.remove_original_ranges(batch.ranges);
            }
        }
    }
    collect_from_mapped_source(source, working)
}

fn collect_with_pretransform_ranges<'a>(
    source: &str,
    opaque_batches: impl IntoIterator<Item = &'a [Range<usize>]>,
    removed_ranges: &[Range<usize>],
) -> Vec<DownstreamProtectorRange> {
    let mut working = MappedSource::new(source);
    let runtime_heads = collect_head_candidate_streams(source)
        .runtime_modules
        .into_iter()
        .filter_map(|candidate| match candidate {
            RuntimeModuleHeadCandidate::Exact(range) => Some(range),
            RuntimeModuleHeadCandidate::RecoveryBarrier(_) => None,
        })
        .collect::<Vec<_>>();
    working.replace_original_ranges_with_inert_markers(&runtime_heads);
    for ranges in opaque_batches {
        working.replace_original_ranges_with_inert_markers(ranges);
    }
    working.remove_original_ranges(removed_ranges);
    collect_from_mapped_source(source, working)
}

fn collect_from_mapped_source(
    source: &str,
    mut working: MappedSource,
) -> Vec<DownstreamProtectorRange> {
    let anchors = collect_anchor_marker_ranges(source, &working);
    working.replace_with_inert_markers(&anchors);
    let current = collect_current_page_link_ranges(source, &working);
    working.replace_with_inert_markers(&current);
    let star = collect_star_local_link_ranges(source, &working);
    working.replace_with_inert_markers(&star);
    let wikipedia = collect_wikipedia_link_ranges(source, &working);

    merge_family_streams([anchors, current, star, wikipedia])
}

struct AcceptedProtectorRange {
    virtual_range: Range<usize>,
    protected: DownstreamProtectorRange,
}

fn merge_family_streams(
    streams: [Vec<AcceptedProtectorRange>; 4],
) -> Vec<DownstreamProtectorRange> {
    let mut indices = [0usize; 4];
    let mut merged = Vec::with_capacity(streams.iter().map(Vec::len).sum());
    loop {
        let next = streams
            .iter()
            .enumerate()
            .filter_map(|(stream, ranges)| {
                ranges.get(indices[stream]).map(|accepted| {
                    (
                        stream,
                        accepted.protected.range.start,
                        accepted.protected.range.end,
                        accepted.protected.family,
                    )
                })
            })
            .min_by_key(|(_, start, end, family)| (*start, *end, *family));
        let Some((stream, _, _, _)) = next else {
            break;
        };
        merged.push(streams[stream][indices[stream]].protected.clone());
        indices[stream] += 1;
    }
    merged
}

fn collect_anchor_marker_ranges(
    original: &str,
    working: &MappedSource,
) -> Vec<AcceptedProtectorRange> {
    let source = working.source();
    let literal_regions = LegacyLiteralStartIndex::new(source);
    let mut literal_regions = literal_regions.monotone_cursor();
    let mut full_origins = working.origin_cursor();
    let mut content_origins = working.origin_cursor();
    WIKIDOT_ANCHOR_MARKER_REGEX
        .captures_iter(source)
        .filter_map(|captures| {
            let matched = captures.get(0)?;
            if literal_regions.contains(matched.start()) {
                return None;
            }
            let name = captures.name("name")?;
            if name.as_str().trim().is_empty() {
                return None;
            }
            Some(protector_range(
                original,
                &mut full_origins,
                &mut content_origins,
                matched.range(),
                name.range(),
                DownstreamProtectorFamily::AnchorMarker,
            ))
        })
        .collect()
}

fn collect_current_page_link_ranges(
    original: &str,
    working: &MappedSource,
) -> Vec<AcceptedProtectorRange> {
    let source = working.source();
    let literal_regions = LegacyLiteralStartIndex::new(source);
    let mut literal_regions = literal_regions.monotone_cursor();
    let mut full_origins = working.origin_cursor();
    let mut content_origins = working.origin_cursor();
    WIKIDOT_CURRENT_PAGE_LINK_REGEX
        .captures_iter(source)
        .filter_map(|captures| {
            let matched = captures.get(0)?;
            if source[..matched.start()].ends_with('[')
                || source[matched.end()..].starts_with(']')
                || literal_regions.contains(matched.start())
            {
                return None;
            }
            let label = captures.name("label")?;
            if label.as_str().trim().is_empty() {
                return None;
            }
            Some(protector_range(
                original,
                &mut full_origins,
                &mut content_origins,
                matched.range(),
                label.range(),
                DownstreamProtectorFamily::CurrentPageLink,
            ))
        })
        .collect()
}

fn collect_star_local_link_ranges(
    original: &str,
    working: &MappedSource,
) -> Vec<AcceptedProtectorRange> {
    let source = working.source();
    let literal_regions = LegacyLiteralStartIndex::new(source);
    let mut literal_regions = literal_regions.monotone_cursor();
    let mut full_origins = working.origin_cursor();
    let mut content_origins = working.origin_cursor();
    WIKIDOT_STAR_LOCAL_LINK_REGEX
        .captures_iter(source)
        .filter_map(|captures| {
            let matched = captures.get(0)?;
            if literal_regions.contains(matched.start()) {
                return None;
            }
            let target = captures.name("target")?;
            let label = captures.name("label")?;
            if label.as_str().trim().is_empty() {
                return None;
            }
            Some(protector_range(
                original,
                &mut full_origins,
                &mut content_origins,
                matched.range(),
                target.start()..label.end(),
                DownstreamProtectorFamily::StarLocalLink,
            ))
        })
        .collect()
}

fn collect_wikipedia_link_ranges(
    original: &str,
    working: &MappedSource,
) -> Vec<AcceptedProtectorRange> {
    let source = working.source();
    let literal_regions = LiteralRegionIndex::new_wikidot_syntax(source);
    let mut literal_regions = literal_regions.monotone_cursor();
    let mut full_origins = working.origin_cursor();
    let mut content_origins = working.origin_cursor();
    let ranges = WIKIDOT_WIKIPEDIA_LINK_REGEX
        .captures_iter(source)
        .filter_map(|captures| {
            let matched = captures.get(0)?;
            if literal_regions.containing_end(matched.start()).is_some() {
                return None;
            }
            let target = captures.name("target")?;
            let content_end = captures
                .name("label")
                .map_or(target.end(), |label| label.end());
            Some(protector_range(
                original,
                &mut full_origins,
                &mut content_origins,
                matched.range(),
                target.start()..content_end,
                DownstreamProtectorFamily::WikipediaLink,
            ))
        })
        .collect();
    record_collector_work(literal_regions.advances());
    ranges
}

fn protector_range(
    source: &str,
    full_origins: &mut OriginRangeCursor<'_>,
    content_origins: &mut OriginRangeCursor<'_>,
    virtual_range: Range<usize>,
    virtual_content: Range<usize>,
    family: DownstreamProtectorFamily,
) -> AcceptedProtectorRange {
    let range = full_origins.map_range(virtual_range.clone());
    let content = content_origins.map_range(virtual_content);
    debug_assert!(source.is_char_boundary(content.start));
    debug_assert!(source.is_char_boundary(content.end));
    AcceptedProtectorRange {
        virtual_range,
        protected: DownstreamProtectorRange {
            range,
            family,
            contains_ownership_delimiter: contains_ownership_delimiter(&source[content]),
        },
    }
}

fn contains_ownership_delimiter(content: &str) -> bool {
    const DELIMITERS: [&str; 16] = [
        "[[", "@@", "@<", ">@", "[!--", "--]", "[[$", "$]]", "##", "[", "]", "**", "//",
        "__", "^^", ",,",
    ];
    DELIMITERS
        .into_iter()
        .any(|delimiter| content.contains(delimiter))
        || content.contains("{{")
        || content.contains("}}")
}

struct LegacyLiteralStartIndex {
    ranges: Vec<Range<usize>>,
}

struct LegacyLiteralStartCursor<'a> {
    ranges: &'a [Range<usize>],
    cursor: usize,
}

impl LegacyLiteralStartIndex {
    fn new(source: &str) -> Self {
        const CODE_ON: u8 = 1 << 0;
        const CODE_OFF: u8 = 1 << 1;
        const HTML_ON: u8 = 1 << 2;
        const HTML_OFF: u8 = 1 << 3;

        let mut line_events = vec![0u8; source.len() + 1];
        let mut line_start = 0usize;
        for line in source.split_inclusive('\n') {
            record_collector_work(line.len());
            let had_newline = line.ends_with('\n');
            let mut body = line.strip_suffix('\n').unwrap_or(line);
            if had_newline {
                body = body.strip_suffix('\r').unwrap_or(body);
            }
            let logical = body.trim_start();
            let logical_start = line_start + body.len() - logical.len();
            if logical
                .as_bytes()
                .get(..b"[[code".len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"[[code"))
            {
                line_events[logical_start + b"[[code".len()] |= CODE_ON;
            } else if logical
                .as_bytes()
                .get(..b"[[/code]]".len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"[[/code]]"))
            {
                line_events[logical_start + b"[[/code]]".len()] |= CODE_OFF;
            }
            if logical
                .as_bytes()
                .get(..b"[[html".len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"[[html"))
            {
                line_events[logical_start + b"[[html".len()] |= HTML_ON;
            } else if logical
                .as_bytes()
                .get(..b"[[/html]]".len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"[[/html]]"))
            {
                line_events[logical_start + b"[[/html]]".len()] |= HTML_OFF;
            }
            line_start += line.len();
        }

        let bytes = source.as_bytes();
        let mut code = false;
        let mut html = false;
        let mut raw = false;
        let mut raw_search_start = 0usize;
        let mut comment = false;
        let mut active_start = None;
        let mut ranges = Vec::new();

        for (offset, events) in line_events.iter().copied().enumerate().take(source.len())
        {
            record_collector_work(1);
            if events & CODE_ON != 0 {
                code = true;
            } else if events & CODE_OFF != 0 {
                code = false;
            }
            if events & HTML_ON != 0 {
                html = true;
            } else if events & HTML_OFF != 0 {
                html = false;
            }

            if offset >= 2
                && offset - 2 >= raw_search_start
                && bytes.get(offset - 2..offset) == Some(&b"@@"[..])
            {
                raw = !raw;
                raw_search_start = offset;
            }
            if offset >= 4 && bytes.get(offset - 4..offset) == Some(&b"[!--"[..]) {
                comment = true;
            }
            if offset >= 3 && bytes.get(offset - 3..offset) == Some(&b"--]"[..]) {
                comment = false;
            }

            let literal = code || html || raw || comment;
            match (active_start, literal) {
                (None, true) => active_start = Some(offset),
                (Some(start), false) => {
                    ranges.push(start..offset);
                    active_start = None;
                }
                _ => {}
            }
        }
        if let Some(start) = active_start {
            ranges.push(start..source.len());
        }
        Self { ranges }
    }

    fn monotone_cursor(&self) -> LegacyLiteralStartCursor<'_> {
        LegacyLiteralStartCursor {
            ranges: &self.ranges,
            cursor: 0,
        }
    }
}

impl LegacyLiteralStartCursor<'_> {
    fn contains(&mut self, offset: usize) -> bool {
        while self
            .ranges
            .get(self.cursor)
            .is_some_and(|range| range.end <= offset)
        {
            self.cursor += 1;
            record_collector_work(1);
        }
        self.ranges
            .get(self.cursor)
            .is_some_and(|range| range.start <= offset && offset < range.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn protected_text<'a>(source: &'a str, range: &DownstreamProtectorRange) -> &'a str {
        &source[range.range.clone()]
    }

    #[test]
    fn collects_every_downstream_family_in_source_order() {
        let source = concat!(
            "[[# anchor]] ",
            "[# current] ",
            "[*/target star] ",
            "[wikipedia:Rust language]",
        );
        let ranges = collect_downstream_protector_ranges(source);

        assert_eq!(
            ranges
                .iter()
                .map(|range| (range.family, protected_text(source, range)))
                .collect::<Vec<_>>(),
            [
                (DownstreamProtectorFamily::AnchorMarker, "[[# anchor]]"),
                (DownstreamProtectorFamily::CurrentPageLink, "[# current]"),
                (DownstreamProtectorFamily::StarLocalLink, "[*/target star]"),
                (
                    DownstreamProtectorFamily::WikipediaLink,
                    "[wikipedia:Rust language]",
                ),
            ],
        );
        assert!(
            ranges
                .iter()
                .all(|range| !range.contains_ownership_delimiter)
        );
    }

    #[test]
    fn reports_module_opener_inside_protected_label() {
        let source = r#"[# label [[module ListPages name="x"]] tail]"#;
        let ranges = collect_downstream_protector_ranges(source);

        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].family, DownstreamProtectorFamily::CurrentPageLink);
        assert!(ranges[0].contains_ownership_delimiter);
        assert!(protected_text(source, &ranges[0]).contains("[[module ListPages"));
    }

    #[test]
    fn exact_inline_html_effect_hides_module_opener_inside_link_shape() {
        let source = r#"**__[# [[module ListPages category="_"]] ]__**"#;
        let underline = 0..source.len();

        assert!(
            collect_with_pretransform_ranges(
                source,
                [std::slice::from_ref(&underline)],
                &[],
            )
            .is_empty()
        );
    }

    #[test]
    fn staged_outer_effect_can_enclose_an_earlier_opaque_marker() {
        let source = r#"**__@<&nbsp;>@ [# [[module ListPages]] ]__**"#;
        let escaped_start = source.find("@<&nbsp;>@").expect("escaped NBSP exists");
        let escaped = escaped_start..escaped_start + "@<&nbsp;>@".len();
        let underline = 0..source.len();

        assert!(
            collect_with_pretransform_ranges(
                source,
                [
                    std::slice::from_ref(&escaped),
                    std::slice::from_ref(&underline),
                ],
                &[],
            )
            .is_empty()
        );
    }

    #[test]
    fn leftover_color_marker_effect_preserves_original_link_mapping() {
        let source = "[# before ## [[module ListPages]] after]";
        let hashes_start = source.find("##").expect("color marker exists");
        let hashes = hashes_start..hashes_start + 2;
        let ranges = collect_with_pretransform_ranges(
            source,
            [std::slice::from_ref(&hashes)],
            &[],
        );

        assert_eq!(ranges.len(), 1);
        assert_eq!(protected_text(source, &ranges[0]), source);
        assert!(ranges[0].contains_ownership_delimiter);
    }

    #[test]
    fn exact_native_list_effect_hides_links_in_the_rendered_run() {
        let source =
            "* one\n* two\n* three\n* four [# hidden]\n* five\n* six\n* seven\n* eight\n";
        let native_list = 0..source.len();

        assert!(
            collect_with_pretransform_ranges(
                source,
                [std::slice::from_ref(&native_list)],
                &[],
            )
            .is_empty()
        );
    }

    #[test]
    fn exact_color_effect_hides_its_recursive_body() {
        let source = "##red|[# [[module ListPages]] hidden]##";
        let color = 0..source.len();

        assert!(
            collect_with_pretransform_ranges(
                source,
                [std::slice::from_ref(&color)],
                &[],
            )
            .is_empty()
        );
    }

    #[test]
    fn raw_filter_rejects_every_family() {
        let source = concat!(
            "@@[[# hidden-anchor]] ",
            "[# hidden-current] ",
            "[*/target hidden-star] ",
            "[wikipedia:hidden]@@",
        );

        assert!(collect_downstream_protector_ranges(source).is_empty());
    }

    #[test]
    fn earlier_protector_masking_matches_sequential_legacy_guards() {
        let source = "[[# removes-@@]] [# current] [wikipedia:Rust]";
        let ranges = collect_downstream_protector_ranges(source);

        assert_eq!(
            ranges.iter().map(|range| range.family).collect::<Vec<_>>(),
            [
                DownstreamProtectorFamily::AnchorMarker,
                DownstreamProtectorFamily::CurrentPageLink,
                DownstreamProtectorFamily::WikipediaLink,
            ],
        );
    }

    #[test]
    fn initial_css_removal_filters_body_and_preserves_original_mapping() {
        let source = concat!("[[module CSS]][# hidden][[/module]]", "[# visible]",);
        let css_end =
            source.find("[[/module]]").expect("CSS closer exists") + "[[/module]]".len();
        let css = 0..css_end;
        let ranges = collect_with_pretransform_ranges(
            source,
            std::iter::empty(),
            std::slice::from_ref(&css),
        );

        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].family, DownstreamProtectorFamily::CurrentPageLink);
        assert_eq!(protected_text(source, &ranges[0]), "[# visible]");

        let joined = "[# [[module CSS]]ignored[[/module]]joined]";
        let css_start = joined.find("[[module CSS]]").expect("CSS opener exists");
        let css_end =
            joined.find("[[/module]]").expect("CSS closer exists") + "[[/module]]".len();
        let css = css_start..css_end;
        let ranges = collect_with_pretransform_ranges(
            joined,
            std::iter::empty(),
            std::slice::from_ref(&css),
        );
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].family, DownstreamProtectorFamily::CurrentPageLink);
        assert_eq!(protected_text(joined, &ranges[0]), joined);
    }

    #[test]
    fn css_removal_can_enclose_an_earlier_opaque_effect() {
        let source = "[[module CSS]]@<&nbsp;>@ [# hidden][[/module]][# visible]";
        let escaped_start = source.find("@<&nbsp;>@").expect("escaped NBSP exists");
        let escaped = escaped_start..escaped_start + "@<&nbsp;>@".len();
        let css_end =
            source.find("[[/module]]").expect("CSS closer exists") + "[[/module]]".len();
        let css = 0..css_end;
        let ranges = collect_with_pretransform_ranges(
            source,
            [std::slice::from_ref(&escaped)],
            std::slice::from_ref(&css),
        );

        assert_eq!(ranges.len(), 1);
        assert_eq!(protected_text(source, &ranges[0]), "[# visible]");
    }

    #[test]
    fn legacy_literal_index_matches_prefix_contract() {
        let source = concat!(
            "[[CoDe-anything [# code]\n",
            "[[/code]] [# after-code]\n",
            "[[HTMLx [# html]\n",
            "[[/html]] [# after-html]\n",
            "@@[# raw]@@ [# after-raw]\n",
            "[!--[# comment]--] [# after-comment]",
        );
        let index = LegacyLiteralStartIndex::new(source);
        let mut cursor = index.monotone_cursor();
        let observed = WIKIDOT_CURRENT_PAGE_LINK_REGEX
            .find_iter(source)
            .map(|matched| (matched.as_str(), cursor.contains(matched.start())))
            .collect::<Vec<_>>();

        assert_eq!(
            observed,
            [
                ("[# code]", true),
                ("[# after-code]", false),
                ("[# html]", true),
                ("[# after-html]", false),
                ("[# raw]", true),
                ("[# after-raw]", false),
                ("[# comment]", true),
                ("[# after-comment]", false),
            ],
        );
    }

    #[test]
    fn dense_mixed_protectors_have_linear_collector_work() {
        use std::fmt::Write;

        const MATCHES: usize = 4_096;
        let mut source = String::new();
        for index in 0..MATCHES {
            match index % 4 {
                0 => write!(source, "[[# anchor-{index}]] ").unwrap(),
                1 => write!(source, "[# current-{index}] ").unwrap(),
                2 => write!(source, "[*/target-{index} star-{index}] ").unwrap(),
                _ => write!(source, "[wikipedia:Page_{index}] ").unwrap(),
            }
        }

        take_collector_work();
        let ranges = collect_downstream_protector_ranges(&source);
        let work = take_collector_work();

        assert_eq!(ranges.len(), MATCHES);
        assert!(ranges.windows(2).all(|pair| {
            (pair[0].range.start, pair[0].range.end, pair[0].family)
                <= (pair[1].range.start, pair[1].range.end, pair[1].family)
        }));
        assert!(
            work <= source.len() * 20,
            "collector work {work} exceeded linear bound for {} bytes",
            source.len(),
        );
    }

    #[test]
    fn dense_interleaved_raw_and_wikipedia_ranges_have_linear_work() {
        use std::fmt::Write;

        const MATCHES: usize = 4_096;
        let mut source = String::new();
        for index in 0..MATCHES {
            write!(source, "@@raw-{index}@@ [wikipedia:Page_{index}] ").unwrap();
        }

        take_collector_work();
        let ranges = collect_downstream_protector_ranges(&source);
        let work = take_collector_work();

        assert_eq!(ranges.len(), MATCHES);
        assert!(
            ranges
                .iter()
                .all(|range| range.family == DownstreamProtectorFamily::WikipediaLink)
        );
        assert!(
            work <= source.len() * 10,
            "collector work {work} exceeded linear bound for {} bytes",
            source.len(),
        );
    }

    #[test]
    fn mirrors_empty_label_and_nested_current_link_guards() {
        let source = concat!(
            "[[# anchor]] ",
            "[[# nested-current]] ",
            "[[#   ]] ",
            "[#   ] ",
            "[*/target   ] ",
            "[wikipedia:Rust   ]",
        );
        let ranges = collect_downstream_protector_ranges(source);

        assert_eq!(
            ranges
                .iter()
                .map(|range| (range.family, protected_text(source, range)))
                .collect::<Vec<_>>(),
            [
                (DownstreamProtectorFamily::AnchorMarker, "[[# anchor]]"),
                (
                    DownstreamProtectorFamily::AnchorMarker,
                    "[[# nested-current]]",
                ),
                (
                    DownstreamProtectorFamily::WikipediaLink,
                    "[wikipedia:Rust   ]",
                ),
            ],
        );
    }
}
