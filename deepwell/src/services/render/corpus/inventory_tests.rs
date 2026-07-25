/*
 * services/render/corpus/inventory_tests.rs
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

use super::inventory::*;
use crate::services::render::CORPUS_RENDER_BUDGET_US;
use serde_json::json;

fn row(name: &str, total_us: Option<i64>) -> InventoryRow {
    InventoryRow {
        source_fullname: name.to_owned(),
        expected: true,
        attempt: Some(1),
        page_id: Some(1),
        outcome: Some("rendered".to_owned()),
        item_state: "rendered".to_owned(),
        budget_us: CORPUS_RENDER_BUDGET_US,
        pipeline_us: total_us.map(|total| total.min(700_000)),
        total_us,
        complete: true,
        finished: true,
        error_fingerprint: None,
        dominant_scope: Some("body".to_owned()),
        dominant_stage: Some("parse".to_owned()),
        terminal_scope: Some("finalizer".to_owned()),
        terminal_stage: Some("postcommit".to_owned()),
        timings: json!({"body.parse": 10, "finalizer.postcommit": 2}),
        dimensions: json!({
            "source_bytes": 1,
            "expanded_bytes": 1,
            "output_bytes": 1,
            "included_pages": 0,
        }),
        post_commit_error: false,
    }
}

fn finished_run(item_count: i64) -> InventoryRunContext {
    InventoryRunContext {
        exists: true,
        state: Some("done".to_owned()),
        finished: true,
        complete_inventory: true,
        manifest_row_count: item_count,
        item_count,
        global_nonquiescent: 0,
        leased_items: 0,
        failed_items: 0,
    }
}

#[test]
fn exact_budget_boundary_passes_and_one_microsecond_over_fails() {
    let passing = build_summary(
        Some(1),
        RenderInventoryPass::Pass1,
        finished_run(1),
        vec![row("exact", Some(800_000))],
    );
    assert!(passing.passed);
    assert_eq!(passing.over_budget, 0);
    let failing = build_summary(
        Some(1),
        RenderInventoryPass::Pass1,
        finished_run(1),
        vec![row("over", Some(800_001))],
    );
    assert!(!failing.passed);
    assert_eq!(failing.over_budget, 1);
}

#[test]
fn missing_incomplete_failure_and_postcommit_rows_fail_gate() {
    let mut missing = row("missing", None);
    missing.attempt = None;
    missing.complete = false;
    let mut incomplete = row("incomplete", None);
    incomplete.outcome = Some("running".to_owned());
    incomplete.item_state = "render_running".to_owned();
    incomplete.pipeline_us = None;
    incomplete.complete = false;
    incomplete.finished = false;
    incomplete.dominant_scope = None;
    incomplete.dominant_stage = None;
    incomplete.terminal_scope = None;
    incomplete.terminal_stage = None;
    incomplete.timings = json!({});
    incomplete.dimensions = json!({});
    let mut failed = row("failed", Some(20));
    failed.outcome = Some("render_failed".to_owned());
    failed.item_state = "render_failed".to_owned();
    failed.pipeline_us = Some(20);
    failed.error_fingerprint = Some("a".repeat(64));
    let mut postcommit = row("postcommit", Some(20));
    postcommit.post_commit_error = true;
    let summary = build_summary(
        Some(1),
        RenderInventoryPass::Pass1,
        finished_run(4),
        vec![missing, incomplete, failed, postcommit],
    );
    assert_eq!(
        (
            summary.missing,
            summary.incomplete,
            summary.failed,
            summary.post_commit_errors
        ),
        (1, 1, 1, 1)
    );
    assert!(!summary.passed);
}

#[test]
fn slow_stage_impact_prioritizes_affected_pages_then_excess() {
    let mut first = row("first", Some(900_000));
    first.timings = json!({"body.parse": 70});
    let mut second = row("second", Some(1_000_000));
    second.timings = json!({"body.parse": 80});
    let mut third = row("third", Some(1_500_000));
    third.dominant_stage = Some("html_compat".to_owned());
    third.timings = json!({"body.html_compat": 500});
    let summary = build_summary(
        Some(1),
        RenderInventoryPass::Pass1,
        finished_run(3),
        vec![first, second, third],
    );
    assert_eq!(summary.stage_impact[0].stage, "parse");
    assert_eq!(summary.stage_impact[0].affected_slow_pages, 2);
    assert_eq!(summary.stage_impact[0].stage_us, 150);
}

#[test]
fn nonexistent_and_nonquiescent_runs_fail_closed() {
    let missing = build_summary(
        Some(99),
        RenderInventoryPass::Pass1,
        InventoryRunContext::missing(),
        Vec::new(),
    );
    assert!(!missing.passed);
    assert!(!missing.run_exists);
    let mut run = finished_run(1);
    run.state = Some("rendering".to_owned());
    run.finished = false;
    run.global_nonquiescent = 1;
    run.leased_items = 1;
    let mut active = row("active", Some(20));
    active.item_state = "render_running".to_owned();
    let summary = build_summary(Some(1), RenderInventoryPass::Pass1, run, vec![active]);
    assert!(!summary.passed);
    assert_eq!(summary.pass_nonquiescent, 1);
}

#[test]
fn budget_mismatch_cannot_raise_the_gate_threshold() {
    let mut mismatched = row("mismatched", Some(900_000));
    mismatched.budget_us = 1_000_000;
    let summary = build_summary(
        Some(1),
        RenderInventoryPass::Pass1,
        finished_run(1),
        vec![mismatched],
    );
    assert!(!summary.passed);
    assert_eq!(summary.budget_mismatch, 1);
    assert_eq!(summary.over_budget, 1);
    assert_eq!(summary.corrupt, 1);
}

#[test]
fn null_completion_fields_wrong_pass_and_malformed_trace_fail_closed() {
    let mut missing_total = row("missing-total", None);
    missing_total.finished = false;
    let mut wrong_pass = row("wrong-pass", Some(20));
    wrong_pass.outcome = Some("done".to_owned());
    let mut malformed = row("malformed", Some(20));
    malformed.timings = json!({"body.parse": "ten"});
    malformed.dimensions = json!({"source_bytes": 1});
    let summary = build_summary(
        Some(1),
        RenderInventoryPass::Pass1,
        finished_run(3),
        vec![missing_total, wrong_pass, malformed],
    );
    assert!(!summary.passed);
    assert_eq!(summary.corrupt, 3);
}

#[test]
fn pass2_accepts_only_done_success_and_checks_all_target_states() {
    let mut done = row("done", Some(20));
    done.outcome = Some("done".to_owned());
    done.item_state = "done".to_owned();
    let passing = build_summary(
        Some(1),
        RenderInventoryPass::Pass2,
        finished_run(1),
        vec![done.clone()],
    );
    assert!(passing.passed);
    done.item_state = "rendered".to_owned();
    let failing = build_summary(
        Some(1),
        RenderInventoryPass::Pass2,
        finished_run(1),
        vec![done],
    );
    assert!(!failing.passed);
    assert_eq!(failing.pass_nonquiescent, 1);
}
