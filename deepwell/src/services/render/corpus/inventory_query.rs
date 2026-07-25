/*
 * services/render/corpus/inventory_query.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! PostgreSQL inventory loading for the corpus render gate.

use super::inventory::{
    CorpusRenderInventoryService, InventoryRow, InventoryRunContext, RenderInventoryPass,
    RenderInventorySettings, RenderInventorySummary, build_summary,
};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::runtime::ServerState;
use crate::services::render::CORPUS_RENDER_BUDGET_US;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, Value};

impl CorpusRenderInventoryService {
    pub async fn run(
        state: &ServerState,
        settings: RenderInventorySettings,
    ) -> Result<RenderInventorySummary> {
        let import_run_id = match settings.import_run_id {
            Some(import_run_id) => Some(import_run_id),
            None => select_latest_import_run(state).await?,
        };
        let (run, rows) = match import_run_id {
            Some(import_run_id) => match load_run_context(state, import_run_id).await? {
                Some(run) => (
                    run,
                    load_inventory_rows(state, import_run_id, settings.pass).await?,
                ),
                None => (InventoryRunContext::missing(), Vec::new()),
            },
            None => (InventoryRunContext::missing(), Vec::new()),
        };
        Ok(build_summary(import_run_id, settings.pass, run, rows))
    }
}

async fn select_latest_import_run(state: &ServerState) -> Result<Option<i64>> {
    let make_error = || {
        Error::new(
            "failed to select render inventory import run",
            ErrorType::DatabaseQuery,
        )
    };
    let statement = Statement::from_string(
        DatabaseBackend::Postgres,
        "SELECT import_run_id FROM wikidot_corpus_import_run ORDER BY started_at DESC, import_run_id DESC LIMIT 1",
    );
    state
        .database
        .query_one(statement)
        .await
        .or_raise(make_error)?
        .map(|row| row.try_get("", "import_run_id").or_raise(make_error))
        .transpose()
}

async fn load_run_context(
    state: &ServerState,
    import_run_id: i64,
) -> Result<Option<InventoryRunContext>> {
    let make_error = || {
        Error::new(
            "failed to load corpus render inventory run context",
            ErrorType::DatabaseQuery,
        )
    };
    let statement = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        str!(
            "
            SELECT
                run.state,
                run.finished_at IS NOT NULL AS finished,
                run.complete_inventory,
                run.manifest_row_count,
                COUNT(item.source_entity_id)::bigint AS item_count,
                COUNT(*) FILTER (
                    WHERE item.source_entity_id IS NOT NULL
                    AND item.state NOT IN ('rendered', 'render_failed', 'done')
                )::bigint AS global_nonquiescent,
                COUNT(*) FILTER (
                    WHERE item.source_entity_id IS NOT NULL
                    AND item.lease_until IS NOT NULL
                )::bigint AS leased_items,
                COUNT(*) FILTER (
                    WHERE item.source_entity_id IS NOT NULL
                    AND item.state IN ('render_failed', 'failed')
                )::bigint AS failed_items
            FROM wikidot_corpus_import_run AS run
            LEFT JOIN wikidot_corpus_import_item AS item
                ON item.import_run_id = run.import_run_id
            WHERE run.import_run_id = $1
            GROUP BY run.import_run_id
            "
        ),
        [Value::from(import_run_id)],
    );
    state
        .database
        .query_one(statement)
        .await
        .or_raise(make_error)?
        .map(|row| {
            Ok(InventoryRunContext {
                exists: true,
                state: Some(row.try_get("", "state").or_raise(make_error)?),
                finished: row.try_get("", "finished").or_raise(make_error)?,
                complete_inventory: row
                    .try_get("", "complete_inventory")
                    .or_raise(make_error)?,
                manifest_row_count: row
                    .try_get("", "manifest_row_count")
                    .or_raise(make_error)?,
                item_count: row.try_get("", "item_count").or_raise(make_error)?,
                global_nonquiescent: row
                    .try_get("", "global_nonquiescent")
                    .or_raise(make_error)?,
                leased_items: row.try_get("", "leased_items").or_raise(make_error)?,
                failed_items: row.try_get("", "failed_items").or_raise(make_error)?,
            })
        })
        .transpose()
}

fn inventory_sql(pass: RenderInventoryPass) -> String {
    format!(
        "
        WITH expected AS (
            SELECT item.source_entity_id, item.source_fullname, item.page_id, item.state AS item_state
            FROM wikidot_corpus_import_item AS item
            {}
            WHERE item.import_run_id = $1
            AND {}
        ), latest AS (
            SELECT DISTINCT ON (source_entity_id)
                source_entity_id, attempt, page_id, outcome, budget_us, pipeline_us, total_us, complete,
                dominant_scope, dominant_stage, terminal_scope, terminal_stage, timings,
                dimensions, error_fingerprint, finished_at IS NOT NULL AS finished,
                post_commit_error
            FROM wikidot_corpus_render_observation
            WHERE import_run_id = $1
            AND pass = $2::text
            ORDER BY source_entity_id, attempt DESC
        )
        SELECT
            COALESCE(expected.source_fullname, item.source_fullname) AS source_fullname,
            expected.source_entity_id IS NOT NULL AS expected,
            latest.attempt,
            COALESCE(latest.page_id, expected.page_id) AS page_id,
            latest.outcome,
            COALESCE(expected.item_state, item.state) AS item_state,
            COALESCE(latest.budget_us, $3::bigint) AS budget_us,
            latest.pipeline_us,
            latest.total_us,
            COALESCE(latest.complete, FALSE) AS complete,
            COALESCE(latest.finished, FALSE) AS finished,
            latest.error_fingerprint,
            latest.dominant_scope,
            latest.dominant_stage,
            latest.terminal_scope,
            latest.terminal_stage,
            COALESCE(latest.timings, '{{}}'::jsonb) AS timings,
            COALESCE(latest.dimensions, '{{}}'::jsonb) AS dimensions,
            COALESCE(latest.post_commit_error, FALSE) AS post_commit_error
        FROM expected
        FULL OUTER JOIN latest USING (source_entity_id)
        LEFT JOIN wikidot_corpus_import_item AS item
            ON item.import_run_id = $1
            AND item.source_entity_id = COALESCE(expected.source_entity_id, latest.source_entity_id)
        ",
        pass.expected_joins(),
        pass.expected_filter(),
    )
}

async fn load_inventory_rows(
    state: &ServerState,
    import_run_id: i64,
    pass: RenderInventoryPass,
) -> Result<Vec<InventoryRow>> {
    let make_error = || {
        Error::new(
            "failed to load corpus render inventory",
            ErrorType::DatabaseQuery,
        )
    };
    let statement = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        inventory_sql(pass),
        [
            Value::from(import_run_id),
            Value::from(pass.as_str()),
            Value::from(CORPUS_RENDER_BUDGET_US),
        ],
    );
    state
        .database
        .query_all(statement)
        .await
        .or_raise(make_error)?
        .into_iter()
        .map(|row| {
            Ok(InventoryRow {
                source_fullname: row
                    .try_get("", "source_fullname")
                    .or_raise(make_error)?,
                expected: row.try_get("", "expected").or_raise(make_error)?,
                attempt: row.try_get("", "attempt").or_raise(make_error)?,
                page_id: row.try_get("", "page_id").or_raise(make_error)?,
                outcome: row.try_get("", "outcome").or_raise(make_error)?,
                item_state: row.try_get("", "item_state").or_raise(make_error)?,
                budget_us: row.try_get("", "budget_us").or_raise(make_error)?,
                pipeline_us: row.try_get("", "pipeline_us").or_raise(make_error)?,
                total_us: row.try_get("", "total_us").or_raise(make_error)?,
                complete: row.try_get("", "complete").or_raise(make_error)?,
                finished: row.try_get("", "finished").or_raise(make_error)?,
                error_fingerprint: row
                    .try_get("", "error_fingerprint")
                    .or_raise(make_error)?,
                dominant_scope: row.try_get("", "dominant_scope").or_raise(make_error)?,
                dominant_stage: row.try_get("", "dominant_stage").or_raise(make_error)?,
                terminal_scope: row.try_get("", "terminal_scope").or_raise(make_error)?,
                terminal_stage: row.try_get("", "terminal_stage").or_raise(make_error)?,
                timings: row.try_get("", "timings").or_raise(make_error)?,
                dimensions: row.try_get("", "dimensions").or_raise(make_error)?,
                post_commit_error: row
                    .try_get("", "post_commit_error")
                    .or_raise(make_error)?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_attempt_sql_is_pass_scoped_and_descending() {
        let sql = inventory_sql(RenderInventoryPass::Pass2);
        assert!(sql.contains("DISTINCT ON (source_entity_id)"));
        assert!(sql.contains("AND pass = $2::text"));
        assert!(sql.contains("ORDER BY source_entity_id, attempt DESC"));
        assert!(sql.contains("revision_text.contents"));
        assert!(!sql.contains("item.state IN ('render_running'"));
    }
}
