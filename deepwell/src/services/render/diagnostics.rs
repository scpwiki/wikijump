/*
 * services/render/diagnostics.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Low-overhead stage accounting for trusted corpus-finalizer renders.
//!
//! Normal interactive renders never construct a trace. `StageGuard::new()`
//! therefore avoids even reading the clock when its trace argument is `None`.

use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const SCOPE_COUNT: usize = 5;
const STAGE_COUNT: usize = 37;
pub(crate) const CORPUS_RENDER_BUDGET_US: i64 = 800_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub(crate) enum CorpusRenderScope {
    Finalizer,
    Rerender,
    Body,
    TopNav,
    SideNav,
}

impl CorpusRenderScope {
    const ALL: [Self; SCOPE_COUNT] = [
        Self::Finalizer,
        Self::Rerender,
        Self::Body,
        Self::TopNav,
        Self::SideNav,
    ];

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Finalizer => "finalizer",
            Self::Rerender => "rerender",
            Self::Body => "body",
            Self::TopNav => "top_nav",
            Self::SideNav => "side_nav",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub(crate) enum CorpusRenderStage {
    TxBegin,
    Mark,
    Commit,
    PostCommit,
    RevisionLoad,
    RenderInputs,
    LinkUpdate,
    Outdate,
    RevisionUpdate,
    SiteLoad,
    ImagePrelude,
    Includes,
    PostInclude,
    ListPages,
    CountPages,
    TagCloud,
    Backlinks,
    ChildPages,
    Pages,
    PagesByTag,
    RegistryModules,
    Normalization,
    OuterProtect,
    FallbackCheck,
    WorkerQueue,
    InnerProtect,
    Preprocess,
    Tokenize,
    Parse,
    HtmlRender,
    HtmlCompat,
    FallbackTitles,
    FallbackRender,
    CompiledText,
    BlocksValidate,
    HtmlBlocks,
    CodeBlocks,
}

impl CorpusRenderStage {
    const ALL: [Self; STAGE_COUNT] = [
        Self::TxBegin,
        Self::Mark,
        Self::Commit,
        Self::PostCommit,
        Self::RevisionLoad,
        Self::RenderInputs,
        Self::LinkUpdate,
        Self::Outdate,
        Self::RevisionUpdate,
        Self::SiteLoad,
        Self::ImagePrelude,
        Self::Includes,
        Self::PostInclude,
        Self::ListPages,
        Self::CountPages,
        Self::TagCloud,
        Self::Backlinks,
        Self::ChildPages,
        Self::Pages,
        Self::PagesByTag,
        Self::RegistryModules,
        Self::Normalization,
        Self::OuterProtect,
        Self::FallbackCheck,
        Self::WorkerQueue,
        Self::InnerProtect,
        Self::Preprocess,
        Self::Tokenize,
        Self::Parse,
        Self::HtmlRender,
        Self::HtmlCompat,
        Self::FallbackTitles,
        Self::FallbackRender,
        Self::CompiledText,
        Self::BlocksValidate,
        Self::HtmlBlocks,
        Self::CodeBlocks,
    ];

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::TxBegin => "tx_begin",
            Self::Mark => "mark",
            Self::Commit => "commit",
            Self::PostCommit => "postcommit",
            Self::RevisionLoad => "revision_load",
            Self::RenderInputs => "render_inputs",
            Self::LinkUpdate => "link_update",
            Self::Outdate => "outdate",
            Self::RevisionUpdate => "revision_update",
            Self::SiteLoad => "site_load",
            Self::ImagePrelude => "image_prelude",
            Self::Includes => "includes",
            Self::PostInclude => "post_include",
            Self::ListPages => "listpages",
            Self::CountPages => "countpages",
            Self::TagCloud => "tagcloud",
            Self::Backlinks => "backlinks",
            Self::ChildPages => "childpages",
            Self::Pages => "pages",
            Self::PagesByTag => "pages-by-tag",
            Self::RegistryModules => "registry_modules",
            Self::Normalization => "normalization",
            Self::OuterProtect => "outer_protect",
            Self::FallbackCheck => "fallback_check",
            Self::WorkerQueue => "worker_queue",
            Self::InnerProtect => "inner_protect",
            Self::Preprocess => "preprocess",
            Self::Tokenize => "tokenize",
            Self::Parse => "parse",
            Self::HtmlRender => "html_render",
            Self::HtmlCompat => "html_compat",
            Self::FallbackTitles => "fallback_titles",
            Self::FallbackRender => "fallback_render",
            Self::CompiledText => "compiled_text",
            Self::BlocksValidate => "blocks_validate",
            Self::HtmlBlocks => "html_blocks",
            Self::CodeBlocks => "code_blocks",
        }
    }
}

pub(crate) fn is_corpus_render_timing(scope: &str, stage: &str) -> bool {
    CorpusRenderScope::ALL
        .iter()
        .any(|candidate| candidate.as_str() == scope)
        && CorpusRenderStage::ALL
            .iter()
            .any(|candidate| candidate.as_str() == stage)
}

pub(crate) const CORPUS_RENDER_DIMENSIONS: [&str; 4] = [
    "source_bytes",
    "expanded_bytes",
    "output_bytes",
    "included_pages",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CorpusRenderDimension {
    SourceBytes,
    ExpandedBytes,
    OutputBytes,
    IncludedPages,
}

impl CorpusRenderDimension {
    const fn index(self) -> usize {
        match self {
            Self::SourceBytes => 0,
            Self::ExpandedBytes => 1,
            Self::OutputBytes => 2,
            Self::IncludedPages => 3,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::SourceBytes => "source_bytes",
            Self::ExpandedBytes => "expanded_bytes",
            Self::OutputBytes => "output_bytes",
            Self::IncludedPages => "included_pages",
        }
    }
}

#[derive(Debug)]
struct CorpusRenderTraceInner {
    timings: [AtomicU64; SCOPE_COUNT * STAGE_COUNT],
    terminal: AtomicU64,
    dimensions: [AtomicU64; 4],
    started_at: Instant,
}

#[derive(Debug, Clone)]
pub(crate) struct CorpusRenderTrace(Arc<CorpusRenderTraceInner>);

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct CorpusRenderTraceSnapshot {
    pub pipeline_us: u64,
    pub dominant_scope: Option<String>,
    pub dominant_stage: Option<String>,
    pub terminal_scope: Option<String>,
    pub terminal_stage: Option<String>,
    pub timings: BTreeMap<String, u64>,
    pub dimensions: BTreeMap<String, u64>,
}

impl CorpusRenderTrace {
    pub(crate) fn new() -> Self {
        Self(Arc::new(CorpusRenderTraceInner {
            timings: std::array::from_fn(|_| AtomicU64::new(0)),
            terminal: AtomicU64::new(0),
            dimensions: std::array::from_fn(|_| AtomicU64::new(0)),
            started_at: Instant::now(),
        }))
    }

    #[inline]
    pub(crate) fn set_dimension(&self, dimension: CorpusRenderDimension, value: usize) {
        self.0.dimensions[dimension.index()]
            .store(u64::try_from(value).unwrap_or(u64::MAX), Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn add_us(
        &self,
        scope: CorpusRenderScope,
        stage: CorpusRenderStage,
        micros: u64,
    ) {
        let timing = &self.0.timings[timing_index(scope, stage)];
        let _ = timing.try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            Some(value.saturating_add(micros))
        });
    }

    #[inline]
    pub(crate) fn record_elapsed(
        &self,
        scope: CorpusRenderScope,
        stage: CorpusRenderStage,
        started_at: Instant,
    ) {
        self.add_us(scope, stage, duration_us(started_at.elapsed()));
    }

    pub(crate) fn snapshot(&self) -> CorpusRenderTraceSnapshot {
        let mut timings = BTreeMap::new();
        let mut dominant = None;

        for scope in CorpusRenderScope::ALL {
            for stage in CorpusRenderStage::ALL {
                let value =
                    self.0.timings[timing_index(scope, stage)].load(Ordering::Relaxed);
                if value == 0 {
                    continue;
                }
                timings.insert(format!("{}.{}", scope.as_str(), stage.as_str()), value);
                if dominant.is_none_or(|(_, _, largest)| value > largest) {
                    dominant = Some((scope, stage, value));
                }
            }
        }

        let terminal = decode_terminal(self.0.terminal.load(Ordering::Relaxed));

        let mut dimensions = BTreeMap::new();
        for dimension in [
            CorpusRenderDimension::SourceBytes,
            CorpusRenderDimension::ExpandedBytes,
            CorpusRenderDimension::OutputBytes,
            CorpusRenderDimension::IncludedPages,
        ] {
            dimensions.insert(
                dimension.as_str().to_owned(),
                self.0.dimensions[dimension.index()].load(Ordering::Relaxed),
            );
        }

        CorpusRenderTraceSnapshot {
            pipeline_us: duration_us(self.0.started_at.elapsed()),
            dominant_scope: dominant.map(|(scope, _, _)| scope.as_str().to_owned()),
            dominant_stage: dominant.map(|(_, stage, _)| stage.as_str().to_owned()),
            terminal_scope: terminal.map(|(scope, _)| scope.as_str().to_owned()),
            terminal_stage: terminal.map(|(_, stage)| stage.as_str().to_owned()),
            timings,
            dimensions,
        }
    }
}

#[derive(Debug)]
pub(crate) struct StageGuard<'a> {
    trace: Option<(&'a CorpusRenderTrace, CorpusRenderScope)>,
    stage: CorpusRenderStage,
    started_at: Option<Instant>,
}

impl<'a> StageGuard<'a> {
    #[inline]
    pub(crate) fn new(
        trace: Option<(&'a CorpusRenderTrace, CorpusRenderScope)>,
        stage: CorpusRenderStage,
    ) -> Self {
        let started_at = trace.map(|(trace, scope)| {
            trace
                .0
                .terminal
                .store((timing_index(scope, stage) + 1) as u64, Ordering::Relaxed);
            Instant::now()
        });
        Self {
            trace,
            stage,
            started_at,
        }
    }
}

impl Drop for StageGuard<'_> {
    fn drop(&mut self) {
        if let (Some((trace, scope)), Some(started_at)) = (self.trace, self.started_at) {
            trace.record_elapsed(scope, self.stage, started_at);
        }
    }
}

const fn timing_index(scope: CorpusRenderScope, stage: CorpusRenderStage) -> usize {
    scope as usize * STAGE_COUNT + stage as usize
}

fn decode_terminal(encoded: u64) -> Option<(CorpusRenderScope, CorpusRenderStage)> {
    let index = usize::try_from(encoded).ok()?.checked_sub(1)?;
    let scope = *CorpusRenderScope::ALL.get(index / STAGE_COUNT)?;
    let stage = *CorpusRenderStage::ALL.get(index % STAGE_COUNT)?;
    Some((scope, stage))
}

fn duration_us(duration: Duration) -> u64 {
    let nanos = duration.as_nanos();
    let whole_micros = nanos / 1_000;
    let rounded_micros = whole_micros + u128::from(!nanos.is_multiple_of(1_000));
    saturating_us(rounded_micros)
}

fn saturating_us(value: u128) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_accumulates_and_snapshot_uses_stable_tie_order() {
        let trace = CorpusRenderTrace::new();
        trace.add_us(CorpusRenderScope::Body, CorpusRenderStage::Parse, 7);
        trace.add_us(CorpusRenderScope::TopNav, CorpusRenderStage::Parse, 7);
        trace.add_us(CorpusRenderScope::Body, CorpusRenderStage::Parse, 5);
        let snapshot = trace.snapshot();

        assert_eq!(snapshot.timings["body.parse"], 12);
        assert_eq!(snapshot.timings["top_nav.parse"], 7);
        assert_eq!(snapshot.dominant_scope.as_deref(), Some("body"));
        assert_eq!(snapshot.dominant_stage.as_deref(), Some("parse"));
    }

    #[test]
    fn disabled_guard_does_not_start_or_record_a_stage() {
        let guard = StageGuard::new(None, CorpusRenderStage::Parse);
        assert!(guard.started_at.is_none());
    }

    #[test]
    fn dropped_guard_records_terminal_stage_after_early_return() {
        fn measured(trace: &CorpusRenderTrace) {
            let _guard = StageGuard::new(
                Some((trace, CorpusRenderScope::Body)),
                CorpusRenderStage::FallbackCheck,
            );
        }
        let trace = CorpusRenderTrace::new();
        measured(&trace);
        let snapshot = trace.snapshot();
        assert_eq!(snapshot.terminal_scope.as_deref(), Some("body"));
        assert_eq!(snapshot.terminal_stage.as_deref(), Some("fallback_check"));
    }

    #[test]
    fn accumulation_saturates_instead_of_wrapping() {
        let trace = CorpusRenderTrace::new();
        trace.add_us(
            CorpusRenderScope::Body,
            CorpusRenderStage::Parse,
            u64::MAX - 2,
        );
        trace.add_us(CorpusRenderScope::Body, CorpusRenderStage::Parse, 10);
        assert_eq!(trace.snapshot().timings["body.parse"], u64::MAX);
    }

    #[test]
    fn concurrent_navigation_scopes_do_not_overwrite_each_other() {
        let trace = CorpusRenderTrace::new();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                trace.add_us(CorpusRenderScope::TopNav, CorpusRenderStage::HtmlRender, 11)
            });
            scope.spawn(|| {
                trace.add_us(
                    CorpusRenderScope::SideNav,
                    CorpusRenderStage::HtmlRender,
                    13,
                )
            });
        });
        let snapshot = trace.snapshot();
        assert_eq!(snapshot.timings["top_nav.html_render"], 11);
        assert_eq!(snapshot.timings["side_nav.html_render"], 13);
    }

    #[test]
    fn terminal_stage_tracks_the_most_recent_stage_across_scopes() {
        let trace = CorpusRenderTrace::new();
        drop(StageGuard::new(
            Some((&trace, CorpusRenderScope::Body)),
            CorpusRenderStage::Parse,
        ));
        drop(StageGuard::new(
            Some((&trace, CorpusRenderScope::Finalizer)),
            CorpusRenderStage::PostCommit,
        ));
        let snapshot = trace.snapshot();
        assert_eq!(snapshot.terminal_scope.as_deref(), Some("finalizer"));
        assert_eq!(snapshot.terminal_stage.as_deref(), Some("postcommit"));
    }

    #[test]
    fn duration_micros_rounds_up_sub_microsecond_overruns() {
        assert_eq!(duration_us(Duration::from_millis(800)), 800_000);
        assert_eq!(
            duration_us(Duration::from_millis(800) + Duration::from_nanos(1)),
            800_001,
        );
        assert_eq!(duration_us(Duration::from_nanos(1)), 1);
    }
}
