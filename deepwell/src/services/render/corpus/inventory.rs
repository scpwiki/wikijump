/*
 * services/render/corpus/inventory.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Whole-pass performance and completeness gate for corpus render observations.

use super::super::prelude::*;
use crate::services::render::{
    CORPUS_RENDER_BUDGET_US, CORPUS_RENDER_DIMENSIONS, is_corpus_render_timing,
};
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::env;

const MAX_REPORTED_SLOW_ROWS: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderInventoryPass {
    Pass1,
    Pass2,
}

impl RenderInventoryPass {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Pass1 => "pass1",
            Self::Pass2 => "pass2",
        }
    }

    pub(super) fn expected_joins(self) -> &'static str {
        match self {
            Self::Pass1 => "",
            Self::Pass2 => {
                "
                JOIN page
                    ON page.page_id = item.page_id
                    AND page.deleted_at IS NULL
                JOIN page_revision AS revision
                    ON revision.revision_id = page.latest_revision_id
                JOIN text AS revision_text
                    ON revision_text.hash = revision.wikitext_hash
                "
            }
        }
    }

    pub(super) fn expected_filter(self) -> &'static str {
        match self {
            Self::Pass1 => "TRUE",
            Self::Pass2 => {
                "
                (
                    revision_text.contents ~* '\\[\\[module[[:space:]]+(backlinks|listpages|countpages|tagcloud)'
                    OR page.slug = '_template'
                    OR page.slug LIKE '%:_template'
                    OR EXISTS (
                        SELECT 1
                        FROM site
                        WHERE site.site_id = page.site_id
                        AND page.slug IN (site.top_bar_page, site.side_bar_page)
                    )
                    OR EXISTS (
                        SELECT 1
                        FROM page_category AS nav_category
                        WHERE nav_category.site_id = page.site_id
                        AND page.slug IN (nav_category.top_bar_page, nav_category.side_bar_page)
                    )
                )
                "
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderInventorySettings {
    pub import_run_id: Option<i64>,
    pub pass: RenderInventoryPass,
}

impl RenderInventorySettings {
    pub fn from_env() -> Result<Self> {
        let import_run_id = env::var("DEEPWELL_RENDER_INVENTORY_IMPORT_RUN_ID")
            .ok()
            .or_else(|| env::var("DEEPWELL_RENDER_IMPORT_RUN_ID").ok())
            .map(|value| parse_positive_run_id(&value))
            .transpose()?;
        let pass = env::var("DEEPWELL_RENDER_INVENTORY_PASS")
            .ok()
            .or_else(|| env::var("DEEPWELL_RENDER_PASS").ok())
            .map(|value| parse_pass(&value))
            .transpose()?
            .unwrap_or(RenderInventoryPass::Pass1);
        Ok(Self {
            import_run_id,
            pass,
        })
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RenderInventorySummary {
    action: &'static str,
    import_run_id: Option<i64>,
    pass: &'static str,
    budget_us: i64,
    pub(super) run_exists: bool,
    run_state: Option<String>,
    run_finished: bool,
    complete_inventory: bool,
    manifest_row_count: i64,
    item_count: i64,
    global_nonquiescent: i64,
    leased_items: i64,
    failed_items: i64,
    pub(super) pass_nonquiescent: usize,
    expected: usize,
    observed: usize,
    covered: usize,
    pub(super) missing: usize,
    unexpected: usize,
    pub(super) incomplete: usize,
    pub(super) failed: usize,
    pub(super) post_commit_errors: usize,
    pub(super) corrupt: usize,
    pub(super) budget_mismatch: usize,
    pub(super) over_budget: usize,
    p50_us: Option<i64>,
    p95_us: Option<i64>,
    p99_us: Option<i64>,
    max_us: Option<i64>,
    pub(super) passed: bool,
    pub(super) stage_impact: Vec<StageImpact>,
    slow_rows: Vec<InventoryRowReport>,
    slow_rows_truncated: usize,
}

impl RenderInventorySummary {
    pub fn passed(&self) -> bool {
        self.passed
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(super) struct StageImpact {
    scope: String,
    pub(super) stage: String,
    pub(super) affected_slow_pages: usize,
    excess_us: i64,
    pub(super) stage_us: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct InventoryRowReport {
    source_fullname: String,
    expected: bool,
    attempt: Option<i32>,
    page_id: Option<i64>,
    outcome: Option<String>,
    item_state: String,
    complete: bool,
    pipeline_us: Option<i64>,
    total_us: Option<i64>,
    budget_us: i64,
    finished: bool,
    error_fingerprint: Option<String>,
    dominant_scope: Option<String>,
    dominant_stage: Option<String>,
    terminal_scope: Option<String>,
    terminal_stage: Option<String>,
    post_commit_error: bool,
    corruption_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InventoryRow {
    pub(super) source_fullname: String,
    pub(super) expected: bool,
    pub(super) attempt: Option<i32>,
    pub(super) page_id: Option<i64>,
    pub(super) outcome: Option<String>,
    pub(super) item_state: String,
    pub(super) budget_us: i64,
    pub(super) pipeline_us: Option<i64>,
    pub(super) total_us: Option<i64>,
    pub(super) complete: bool,
    pub(super) finished: bool,
    pub(super) error_fingerprint: Option<String>,
    pub(super) dominant_scope: Option<String>,
    pub(super) dominant_stage: Option<String>,
    pub(super) terminal_scope: Option<String>,
    pub(super) terminal_stage: Option<String>,
    pub(super) timings: serde_json::Value,
    pub(super) dimensions: serde_json::Value,
    pub(super) post_commit_error: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InventoryRunContext {
    pub(super) exists: bool,
    pub(super) state: Option<String>,
    pub(super) finished: bool,
    pub(super) complete_inventory: bool,
    pub(super) manifest_row_count: i64,
    pub(super) item_count: i64,
    pub(super) global_nonquiescent: i64,
    pub(super) leased_items: i64,
    pub(super) failed_items: i64,
}

impl InventoryRunContext {
    pub(super) fn missing() -> Self {
        Self {
            exists: false,
            state: None,
            finished: false,
            complete_inventory: false,
            manifest_row_count: 0,
            item_count: 0,
            global_nonquiescent: 0,
            leased_items: 0,
            failed_items: 0,
        }
    }
}

#[derive(Debug)]
pub struct CorpusRenderInventoryService;

pub(super) fn build_summary(
    import_run_id: Option<i64>,
    pass: RenderInventoryPass,
    run: InventoryRunContext,
    mut rows: Vec<InventoryRow>,
) -> RenderInventorySummary {
    let expected = rows.iter().filter(|row| row.expected).count();
    let observed = rows.iter().filter(|row| row.attempt.is_some()).count();
    let covered = rows
        .iter()
        .filter(|row| row.expected && row.attempt.is_some())
        .count();
    let missing = expected.saturating_sub(covered);
    let unexpected = rows
        .iter()
        .filter(|row| !row.expected && row.attempt.is_some())
        .count();
    let incomplete = rows
        .iter()
        .filter(|row| row.attempt.is_some() && !row.complete)
        .count();
    let failed = rows
        .iter()
        .filter(|row| row.outcome.as_deref() == Some("render_failed"))
        .count();
    let post_commit_errors = rows.iter().filter(|row| row.post_commit_error).count();
    let corrupt = rows
        .iter()
        .filter(|row| !corruption_reasons(row, pass).is_empty())
        .count();
    let budget_mismatch = rows
        .iter()
        .filter(|row| row.attempt.is_some() && row.budget_us != CORPUS_RENDER_BUDGET_US)
        .count();
    let over_budget = rows
        .iter()
        .filter(|row| {
            row.total_us
                .is_some_and(|total| total > CORPUS_RENDER_BUDGET_US)
        })
        .count();
    let pass_nonquiescent = rows
        .iter()
        .filter(|row| {
            row.expected
                && match pass {
                    RenderInventoryPass::Pass1 => !matches!(
                        row.item_state.as_str(),
                        "rendered" | "render_failed" | "done"
                    ),
                    RenderInventoryPass::Pass2 => {
                        !matches!(row.item_state.as_str(), "render_failed" | "done")
                    }
                }
        })
        .count();
    let mut totals: Vec<i64> = rows.iter().filter_map(|row| row.total_us).collect();
    totals.sort_unstable();

    let mut impacts: BTreeMap<(String, String), StageImpact> = BTreeMap::new();
    for row in rows.iter().filter(|row| {
        row.total_us
            .is_some_and(|total| total > CORPUS_RENDER_BUDGET_US)
    }) {
        let (Some(scope), Some(stage), Some(total_us)) =
            (&row.dominant_scope, &row.dominant_stage, row.total_us)
        else {
            continue;
        };
        let entry = impacts
            .entry((scope.clone(), stage.clone()))
            .or_insert_with(|| StageImpact {
                scope: scope.clone(),
                stage: stage.clone(),
                affected_slow_pages: 0,
                excess_us: 0,
                stage_us: 0,
            });
        entry.affected_slow_pages += 1;
        entry.excess_us = entry
            .excess_us
            .saturating_add(total_us.saturating_sub(CORPUS_RENDER_BUDGET_US));
        entry.stage_us = entry.stage_us.saturating_add(
            row.timings
                .get(format!("{scope}.{stage}"))
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
        );
    }
    let mut stage_impact: Vec<_> = impacts.into_values().collect();
    stage_impact.sort_by_key(|impact| {
        (
            Reverse(impact.affected_slow_pages),
            Reverse(impact.excess_us),
            Reverse(impact.stage_us),
            impact.scope.clone(),
            impact.stage.clone(),
        )
    });

    rows.sort_by_key(|row| {
        (
            Reverse(row.total_us.unwrap_or(i64::MAX)),
            row.source_fullname.clone(),
        )
    });
    let row_corruption: Vec<Vec<String>> = rows
        .iter()
        .map(|row| corruption_reasons(row, pass))
        .collect();
    let mut slow_rows: Vec<_> = rows
        .iter()
        .zip(row_corruption)
        .filter(|(row, corruption)| {
            !row.expected
                || row.attempt.is_none()
                || !row.complete
                || row.outcome.as_deref() == Some("render_failed")
                || row.post_commit_error
                || !corruption.is_empty()
                || row
                    .total_us
                    .is_some_and(|total| total > CORPUS_RENDER_BUDGET_US)
        })
        .map(|(row, corruption)| row_report(row, corruption))
        .collect();
    let slow_rows_truncated = slow_rows.len().saturating_sub(MAX_REPORTED_SLOW_ROWS);
    slow_rows.truncate(MAX_REPORTED_SLOW_ROWS);
    let run_valid = run.exists
        && run.state.as_deref() == Some("done")
        && run.finished
        && run.complete_inventory
        && run.manifest_row_count == run.item_count
        && run.global_nonquiescent == 0
        && run.leased_items == 0
        && run.failed_items == 0
        && pass_nonquiescent == 0;
    let passed = import_run_id.is_some()
        && run_valid
        && missing == 0
        && unexpected == 0
        && incomplete == 0
        && failed == 0
        && post_commit_errors == 0
        && corrupt == 0
        && budget_mismatch == 0
        && over_budget == 0
        && expected == covered;

    RenderInventorySummary {
        action: "render-inventory",
        import_run_id,
        pass: pass.as_str(),
        budget_us: CORPUS_RENDER_BUDGET_US,
        run_exists: run.exists,
        run_state: run.state,
        run_finished: run.finished,
        complete_inventory: run.complete_inventory,
        manifest_row_count: run.manifest_row_count,
        item_count: run.item_count,
        global_nonquiescent: run.global_nonquiescent,
        leased_items: run.leased_items,
        failed_items: run.failed_items,
        pass_nonquiescent,
        expected,
        observed,
        covered,
        missing,
        unexpected,
        incomplete,
        failed,
        post_commit_errors,
        corrupt,
        budget_mismatch,
        over_budget,
        p50_us: percentile(&totals, 50),
        p95_us: percentile(&totals, 95),
        p99_us: percentile(&totals, 99),
        max_us: totals.last().copied(),
        passed,
        stage_impact,
        slow_rows,
        slow_rows_truncated,
    }
}

fn corruption_reasons(row: &InventoryRow, pass: RenderInventoryPass) -> Vec<String> {
    let mut reasons = Vec::new();
    if row.attempt.is_none() {
        return reasons;
    }
    if row.budget_us != CORPUS_RENDER_BUDGET_US {
        reasons.push(str!("budget_mismatch"));
    }
    if row.finished != row.complete {
        reasons.push(str!("finished_complete_mismatch"));
    }
    if row.post_commit_error {
        reasons.push(str!("post_commit_error_set"));
    }

    let timings = parse_counter_map(&row.timings, "timings", &mut reasons);
    let dimensions = parse_counter_map(&row.dimensions, "dimensions", &mut reasons);
    if let Some(dimensions) = &dimensions
        && (dimensions.len() != CORPUS_RENDER_DIMENSIONS.len()
            || !CORPUS_RENDER_DIMENSIONS
                .iter()
                .all(|key| dimensions.contains_key(*key)))
    {
        reasons.push(str!("invalid_dimensions"));
    }

    validate_stage_pair(
        "dominant",
        row.dominant_scope.as_deref(),
        row.dominant_stage.as_deref(),
        timings.as_ref(),
        true,
        &mut reasons,
    );
    validate_stage_pair(
        "terminal",
        row.terminal_scope.as_deref(),
        row.terminal_stage.as_deref(),
        timings.as_ref(),
        false,
        &mut reasons,
    );

    let expected_success = match pass {
        RenderInventoryPass::Pass1 => "rendered",
        RenderInventoryPass::Pass2 => "done",
    };
    let outcome = row.outcome.as_deref();
    if !matches!(
        outcome,
        Some("running" | "rendered" | "done" | "render_failed")
    ) {
        reasons.push(str!("invalid_outcome"));
    }
    if matches!(pass, RenderInventoryPass::Pass1) && outcome == Some("done")
        || matches!(pass, RenderInventoryPass::Pass2) && outcome == Some("rendered")
    {
        reasons.push(str!("outcome_pass_mismatch"));
    }

    if row.complete {
        if row.pipeline_us.is_none() || row.total_us.is_none() {
            reasons.push(str!("complete_missing_duration"));
        }
        if matches!((row.pipeline_us, row.total_us), (Some(pipeline), Some(total)) if total < pipeline)
        {
            reasons.push(str!("total_before_pipeline"));
        }
        if timings.as_ref().is_some_and(BTreeMap::is_empty) {
            reasons.push(str!("complete_empty_timings"));
        }
        match outcome {
            Some("render_failed") => {
                if row.item_state != "render_failed" {
                    reasons.push(str!("failure_item_state_mismatch"));
                }
                if row.pipeline_us != row.total_us {
                    reasons.push(str!("failure_duration_mismatch"));
                }
                if !row
                    .error_fingerprint
                    .as_deref()
                    .is_some_and(valid_error_fingerprint)
                {
                    reasons.push(str!("invalid_error_fingerprint"));
                }
            }
            Some(success) if success == expected_success => {
                if row.page_id.is_none() {
                    reasons.push(str!("success_missing_page_id"));
                }
                if row.error_fingerprint.is_some() {
                    reasons.push(str!("success_has_error_fingerprint"));
                }
                let state_matches = match pass {
                    RenderInventoryPass::Pass1 => {
                        matches!(row.item_state.as_str(), "rendered" | "done")
                    }
                    RenderInventoryPass::Pass2 => row.item_state == "done",
                };
                if !state_matches {
                    reasons.push(str!("success_item_state_mismatch"));
                }
            }
            _ => reasons.push(str!("invalid_complete_outcome")),
        }
    } else {
        if row.item_state != "render_running" {
            reasons.push(str!("incomplete_item_state_mismatch"));
        }
        if row.total_us.is_some() || row.error_fingerprint.is_some() {
            reasons.push(str!("incomplete_has_terminal_fields"));
        }
        match outcome {
            Some("running") => {
                if row.pipeline_us.is_some() {
                    reasons.push(str!("running_has_pipeline"));
                }
            }
            Some(success) if success == expected_success => {
                if row.pipeline_us.is_none() {
                    reasons.push(str!("core_observation_missing_pipeline"));
                }
            }
            _ => reasons.push(str!("invalid_incomplete_outcome")),
        }
    }
    reasons
}

fn parse_counter_map(
    value: &serde_json::Value,
    name: &str,
    reasons: &mut Vec<String>,
) -> Option<BTreeMap<String, u64>> {
    match serde_json::from_value(value.clone()) {
        Ok(map) => Some(map),
        Err(_) => {
            reasons.push(format!("invalid_{name}"));
            None
        }
    }
}

fn validate_stage_pair(
    name: &str,
    scope: Option<&str>,
    stage: Option<&str>,
    timings: Option<&BTreeMap<String, u64>>,
    require_maximum: bool,
    reasons: &mut Vec<String>,
) {
    let (Some(scope), Some(stage)) = (scope, stage) else {
        if scope.is_some() || stage.is_some() {
            reasons.push(format!("{name}_pair_mismatch"));
        } else if timings.is_some_and(|timings| !timings.is_empty()) {
            reasons.push(format!("missing_{name}_stage"));
        }
        return;
    };
    if !is_corpus_render_timing(scope, stage) {
        reasons.push(format!("unknown_{name}_stage"));
        return;
    }
    let key = format!("{scope}.{stage}");
    let Some(value) = timings.and_then(|timings| timings.get(&key)) else {
        reasons.push(format!("{name}_stage_missing_timing"));
        return;
    };
    if require_maximum
        && timings
            .and_then(|timings| timings.values().max())
            .is_some_and(|maximum| value != maximum)
    {
        reasons.push(str!("dominant_stage_not_maximum"));
    }
    if let Some(timings) = timings
        && timings.keys().any(|key| {
            let Some((scope, stage)) = key.split_once('.') else {
                return true;
            };
            !is_corpus_render_timing(scope, stage)
        })
    {
        reasons.push(str!("unknown_timing_stage"));
    }
}

fn valid_error_fingerprint(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn row_report(row: &InventoryRow, corruption_reasons: Vec<String>) -> InventoryRowReport {
    InventoryRowReport {
        source_fullname: row.source_fullname.clone(),
        expected: row.expected,
        attempt: row.attempt,
        page_id: row.page_id,
        outcome: row.outcome.clone(),
        item_state: row.item_state.clone(),
        complete: row.complete,
        pipeline_us: row.pipeline_us,
        total_us: row.total_us,
        budget_us: row.budget_us,
        finished: row.finished,
        error_fingerprint: row.error_fingerprint.clone(),
        dominant_scope: row.dominant_scope.clone(),
        dominant_stage: row.dominant_stage.clone(),
        terminal_scope: row.terminal_scope.clone(),
        terminal_stage: row.terminal_stage.clone(),
        post_commit_error: row.post_commit_error,
        corruption_reasons,
    }
}

fn percentile(sorted: &[i64], percentile: usize) -> Option<i64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = (percentile * sorted.len()).div_ceil(100).saturating_sub(1);
    sorted.get(rank).copied()
}

fn parse_positive_run_id(value: &str) -> Result<i64> {
    value
        .trim()
        .parse::<i64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| {
            Error::new(
                "render inventory import run ID must be a positive integer",
                ErrorType::ConfigSetup,
            )
            .into()
        })
}

fn parse_pass(value: &str) -> Result<RenderInventoryPass> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "pass1" | "pass-1" => Ok(RenderInventoryPass::Pass1),
        "2" | "pass2" | "pass-2" => Ok(RenderInventoryPass::Pass2),
        _ => Err(Error::new(
            "render inventory pass must be 1/pass1 or 2/pass2",
            ErrorType::ConfigSetup,
        )
        .into()),
    }
}
