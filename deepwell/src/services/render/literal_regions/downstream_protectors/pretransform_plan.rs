/*
 * services/render/literal_regions/downstream_protectors/pretransform_plan.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use std::ops::Range;

const STAGE_COUNT: usize = 9;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::services::render) enum DownstreamPretransformStage {
    RuntimeModuleHead,
    InlineEscapedNbsp,
    InlineBoldOuterColor,
    InlineBoldColor,
    InlineBoldUnderline,
    ColorSpan,
    UnrenderedColorMarker,
    LongNativeListRun,
    CssModule,
}

impl DownstreamPretransformStage {
    pub(in crate::services::render) const PIPELINE_ORDER: [Self; STAGE_COUNT] = [
        Self::RuntimeModuleHead,
        Self::InlineEscapedNbsp,
        Self::InlineBoldOuterColor,
        Self::InlineBoldColor,
        Self::InlineBoldUnderline,
        Self::ColorSpan,
        Self::UnrenderedColorMarker,
        Self::LongNativeListRun,
        Self::CssModule,
    ];

    const fn index(self) -> usize {
        match self {
            Self::RuntimeModuleHead => 0,
            Self::InlineEscapedNbsp => 1,
            Self::InlineBoldOuterColor => 2,
            Self::InlineBoldColor => 3,
            Self::InlineBoldUnderline => 4,
            Self::ColorSpan => 5,
            Self::UnrenderedColorMarker => 6,
            Self::LongNativeListRun => 7,
            Self::CssModule => 8,
        }
    }

    pub(super) const fn effect(self) -> DownstreamPretransformEffect {
        match self {
            Self::RuntimeModuleHead => DownstreamPretransformEffect::Replace("x"),
            Self::UnrenderedColorMarker => {
                DownstreamPretransformEffect::Replace("&#35;&#35;")
            }
            Self::CssModule => DownstreamPretransformEffect::Remove,
            Self::InlineEscapedNbsp
            | Self::InlineBoldOuterColor
            | Self::InlineBoldColor
            | Self::InlineBoldUnderline
            | Self::ColorSpan
            | Self::LongNativeListRun => DownstreamPretransformEffect::Replace("x"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DownstreamPretransformEffect {
    Replace(&'static str),
    Remove,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::services::render) struct DownstreamProtectorPretransformPlan {
    ranges: [Vec<Range<usize>>; STAGE_COUNT],
}

pub(super) struct DownstreamPretransformBatch<'a> {
    pub(super) stage: DownstreamPretransformStage,
    pub(super) effect: DownstreamPretransformEffect,
    pub(super) ranges: &'a [Range<usize>],
}

impl DownstreamProtectorPretransformPlan {
    pub(in crate::services::render) fn push_exact_range(
        &mut self,
        stage: DownstreamPretransformStage,
        range: Range<usize>,
    ) {
        assert!(range.start < range.end, "exact effects must be nonempty");
        let ranges = &mut self.ranges[stage.index()];
        assert!(
            ranges
                .last()
                .is_none_or(|previous| previous.end <= range.start),
            "exact effects within one stage must be source ordered and disjoint",
        );
        ranges.push(range);
    }

    pub(in crate::services::render) fn extend_exact_ranges(
        &mut self,
        stage: DownstreamPretransformStage,
        ranges: impl IntoIterator<Item = Range<usize>>,
    ) {
        for range in ranges {
            self.push_exact_range(stage, range);
        }
    }

    pub(in crate::services::render) fn ranges(
        &self,
        stage: DownstreamPretransformStage,
    ) -> &[Range<usize>] {
        &self.ranges[stage.index()]
    }

    pub(super) fn batches(
        &self,
    ) -> impl Iterator<Item = DownstreamPretransformBatch<'_>> {
        DownstreamPretransformStage::PIPELINE_ORDER
            .into_iter()
            .map(|stage| DownstreamPretransformBatch {
                stage,
                effect: stage.effect(),
                ranges: self.ranges(stage),
            })
    }

    pub(super) fn validate_for_source(&self, source: &str) {
        for batch in self.batches() {
            for range in batch.ranges {
                assert!(
                    range.end <= source.len()
                        && source.is_char_boundary(range.start)
                        && source.is_char_boundary(range.end),
                    "exact {:?} effect must use original-source UTF-8 boundaries",
                    batch.stage,
                );
            }
        }
    }
}
