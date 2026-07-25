/*
 * services/render/corpus/finalizer.rs
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

use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::runtime::ServerState;
use crate::services::ServiceContext;
use crate::services::render::{
    CORPUS_RENDER_BUDGET_US, CorpusRenderScope, CorpusRenderStage, CorpusRenderTrace,
    CorpusRenderTraceSnapshot, StageGuard,
};
use crate::services::{PageRevisionService, page_revision::RerenderType};
use crate::types::{PageId, RerenderDepth};
use futures::{StreamExt, stream};
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, Value};
use sea_orm::{DatabaseTransaction, TransactionTrait};
use sha2::{Digest, Sha256};
use std::future::Future;
use std::time::{Duration, Instant};
use std::{collections::BTreeMap, env};

const ACTION: &str = "render-finalize";
const DEFAULT_BATCH_SIZE: i64 = 100;
const DEFAULT_CONCURRENCY: usize = 16;
const DEFAULT_LEASE_SECONDS: i64 = 300;
const DEFAULT_MAX_ATTEMPTS: i64 = 3;

#[derive(Debug)]
pub struct CorpusRenderFinalizerService;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFinalizerSettings {
    pub import_run_id: Option<i64>,
    pub pass: RenderFinalizerPass,
    pub batch_size: i64,
    pub concurrency: usize,
    pub lease_seconds: i64,
    pub max_attempts: i64,
    pub dry_run: bool,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct RenderFinalizerSummary {
    action: &'static str,
    dry_run: bool,
    import_run_id: Option<i64>,
    pass: &'static str,
    batch_size: i64,
    concurrency: usize,
    lease_seconds: i64,
    max_attempts: i64,
    candidates: usize,
    claimed: usize,
    rendered: usize,
    done: usize,
    render_failed: usize,
    elapsed_ms: u128,
    rows_per_sec: f64,
    reason_counts: BTreeMap<String, usize>,
    items: Vec<RenderFinalizerItem>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct RenderFinalizerItem {
    #[serde(skip_serializing)]
    import_run_id: i64,
    #[serde(skip_serializing)]
    source_entity_id: String,
    source_fullname: String,
    page_id: Option<i64>,
    site_id: Option<i64>,
    page_category_id: Option<i64>,
    attempts: i32,
    reasons: Vec<String>,
    outcome: Option<String>,
    error: Option<String>,
    error_chain: Option<String>,
    duration_ms: Option<u64>,
    #[serde(skip_serializing)]
    observation_completion: Option<RenderObservationCompletion>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct RenderObservationCompletion {
    import_run_id: i64,
    source_entity_id: String,
    pass: &'static str,
    attempt: i32,
    outcome: &'static str,
    total_us: i64,
    dominant_scope: Option<String>,
    dominant_stage: Option<String>,
    terminal_scope: Option<String>,
    terminal_stage: Option<String>,
    timings: BTreeMap<String, u64>,
    dimensions: BTreeMap<String, u64>,
}

#[derive(Debug)]
enum RenderPageOutcome {
    Completed(RenderObservationCompletion),
    PostCommitFailed {
        message: String,
        error_chain: String,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RenderFinalizerPass {
    Pass1,
    Pass2,
}

impl RenderFinalizerPass {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pass1 => "pass1",
            Self::Pass2 => "pass2",
        }
    }

    fn candidate_joins(self) -> &'static str {
        match self {
            Self::Pass1 => "",
            Self::Pass2 => {
                "
                JOIN page_revision AS revision
                    ON revision.revision_id = page.latest_revision_id
                JOIN text AS revision_text
                    ON revision_text.hash = revision.wikitext_hash
                "
            }
        }
    }

    fn candidate_filter(self) -> &'static str {
        match self {
            // Retry failed rows while attempts remain, and reclaim expired
            // render_running rows abandoned by an interrupted worker.
            Self::Pass1 => {
                "
                (
                    item.state = 'render_pending'
                    OR (
                        item.state IN ('render_failed', 'render_running')
                        AND (
                            EXISTS (
                                SELECT 1
                                FROM wikidot_corpus_render_observation AS observation
                                WHERE observation.import_run_id = item.import_run_id
                                AND observation.source_entity_id = item.source_entity_id
                                AND observation.pass = 'pass1'
                                AND observation.attempt = item.attempts
                                AND (
                                    (
                                        item.state = 'render_failed'
                                        AND observation.outcome = 'render_failed'
                                        AND observation.complete = TRUE
                                    )
                                    OR (
                                        item.state = 'render_running'
                                        AND observation.outcome IN ('running', 'rendered')
                                        AND observation.complete = FALSE
                                    )
                                )
                            )
                            OR NOT EXISTS (
                                SELECT 1
                                FROM wikidot_corpus_render_observation AS observation
                                WHERE observation.import_run_id = item.import_run_id
                                AND observation.source_entity_id = item.source_entity_id
                            )
                        )
                    )
                )
                "
            }
            Self::Pass2 => {
                "
                (
                    item.state = 'rendered'
                    OR (
                        item.state IN ('render_failed', 'render_running')
                        AND EXISTS (
                            SELECT 1
                            FROM wikidot_corpus_render_observation AS observation
                            WHERE observation.import_run_id = item.import_run_id
                            AND observation.source_entity_id = item.source_entity_id
                            AND observation.pass = 'pass2'
                            AND observation.attempt = item.attempts
                            AND (
                                (
                                    item.state = 'render_failed'
                                    AND observation.outcome = 'render_failed'
                                    AND observation.complete = TRUE
                                )
                                OR (
                                    item.state = 'render_running'
                                    AND observation.outcome IN ('running', 'done')
                                    AND observation.complete = FALSE
                                )
                            )
                        )
                    )
                )
                AND (
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

    fn candidate_reasons(self) -> &'static str {
        match self {
            Self::Pass1 => "'{}'::text[]",
            Self::Pass2 => {
                "
                array_remove(ARRAY[
                    CASE
                        WHEN revision_text.contents ~* '\\[\\[module[[:space:]]+backlinks'
                        THEN 'source_backlinks_module'
                    END,
                    CASE
                        WHEN revision_text.contents ~* '\\[\\[module[[:space:]]+(listpages|countpages|tagcloud)'
                        THEN 'source_query_module'
                    END,
                    CASE
                        WHEN page.slug = '_template' OR page.slug LIKE '%:_template'
                        THEN 'template_source'
                    END,
                    CASE
                        WHEN EXISTS (
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
                        THEN 'nav_source'
                    END
                ], NULL)::text[]
                "
            }
        }
    }

    fn success_state(self) -> &'static str {
        match self {
            Self::Pass1 => "rendered",
            Self::Pass2 => "done",
        }
    }

    fn rerender_outdates_dependents(self) -> bool {
        match self {
            Self::Pass1 => false,
            Self::Pass2 => true,
        }
    }
}

impl RenderFinalizerSettings {
    pub fn from_env() -> Result<Self> {
        Self::from_getter(|name| env::var(name).ok())
    }

    fn from_getter(mut get: impl FnMut(&str) -> Option<String>) -> Result<Self> {
        Ok(Self {
            import_run_id: parse_optional_positive_i64(
                "DEEPWELL_RENDER_IMPORT_RUN_ID",
                get("DEEPWELL_RENDER_IMPORT_RUN_ID"),
            )?,
            pass: parse_render_pass("DEEPWELL_RENDER_PASS", get("DEEPWELL_RENDER_PASS"))?,
            batch_size: parse_positive_i64(
                "DEEPWELL_RENDER_BATCH_SIZE",
                get("DEEPWELL_RENDER_BATCH_SIZE"),
                DEFAULT_BATCH_SIZE,
            )?,
            concurrency: parse_positive_usize(
                "DEEPWELL_RENDER_CONCURRENCY",
                get("DEEPWELL_RENDER_CONCURRENCY"),
                DEFAULT_CONCURRENCY,
            )?,
            lease_seconds: parse_positive_i64(
                "DEEPWELL_RENDER_LEASE_SECONDS",
                get("DEEPWELL_RENDER_LEASE_SECONDS"),
                DEFAULT_LEASE_SECONDS,
            )?,
            max_attempts: parse_positive_i64(
                "DEEPWELL_RENDER_MAX_ATTEMPTS",
                get("DEEPWELL_RENDER_MAX_ATTEMPTS"),
                DEFAULT_MAX_ATTEMPTS,
            )?,
            dry_run: parse_boolish(
                "DEEPWELL_RENDER_DRY_RUN",
                get("DEEPWELL_RENDER_DRY_RUN"),
            )?
            .unwrap_or(true),
        })
    }
}

impl CorpusRenderFinalizerService {
    pub async fn run(
        state: &ServerState,
        settings: RenderFinalizerSettings,
    ) -> Result<RenderFinalizerSummary> {
        let started_at = Instant::now();
        let import_run_id = match settings.import_run_id {
            Some(import_run_id) => Some(import_run_id),
            None => Self::select_latest_import_run(state, &settings).await?,
        };

        let items = match import_run_id {
            Some(import_run_id) => {
                if settings.dry_run {
                    Self::list_candidates(state, import_run_id, &settings).await?
                } else {
                    let claimed =
                        Self::claim_candidates(state, import_run_id, &settings).await?;
                    Self::render_claimed_items(state, &settings, claimed).await?
                }
            }
            None => Vec::new(),
        };
        Ok(RenderFinalizerSummary::from_items(
            &settings,
            import_run_id,
            items,
            started_at.elapsed().as_millis(),
        ))
    }

    async fn select_latest_import_run(
        state: &ServerState,
        settings: &RenderFinalizerSettings,
    ) -> Result<Option<i64>> {
        let make_error = || {
            Error::new(
                "failed to select latest render-finalize import run",
                ErrorType::DatabaseQuery,
            )
        };

        let statement = Statement::from_string(
            DatabaseBackend::Postgres,
            format!(
                "
                SELECT import_run_id
                FROM wikidot_corpus_import_run AS run
                WHERE state IN ('running', 'rendering', 'done')
                AND EXISTS (
                    SELECT 1
                    FROM wikidot_corpus_import_item AS item
                    LEFT JOIN page
                        ON page.page_id = item.page_id
                        AND page.deleted_at IS NULL
                    {}
                    WHERE item.import_run_id = run.import_run_id
                    AND {}
                    AND (item.lease_until IS NULL OR item.lease_until <= NOW())
                    AND item.attempts < {}
                )
                ORDER BY started_at DESC, import_run_id DESC
                LIMIT 1
                ",
                settings.pass.candidate_joins(),
                settings.pass.candidate_filter(),
                settings.max_attempts,
            ),
        );

        state
            .database
            .query_one(statement)
            .await
            .or_raise(make_error)?
            .map(|row| row.try_get("", "import_run_id").or_raise(make_error))
            .transpose()
    }

    async fn list_candidates(
        state: &ServerState,
        import_run_id: i64,
        settings: &RenderFinalizerSettings,
    ) -> Result<Vec<RenderFinalizerItem>> {
        let make_error = || {
            Error::new(
                "failed to list render-finalize candidate items",
                ErrorType::DatabaseQuery,
            )
        };

        let sql = format!(
            "
            SELECT
                item.import_run_id,
                item.source_entity_id::text AS source_entity_id,
                item.source_fullname,
                item.page_id,
                page.site_id,
                page.page_category_id,
                item.attempts,
                {} AS reasons
            FROM wikidot_corpus_import_item AS item
            LEFT JOIN page
                ON page.page_id = item.page_id
                AND page.deleted_at IS NULL
            {}
            WHERE item.import_run_id = $1
            AND {}
            AND (item.lease_until IS NULL OR item.lease_until <= NOW())
            AND item.attempts < $2
            ORDER BY item.updated_at ASC, item.source_fullname ASC
            LIMIT $3
            ",
            settings.pass.candidate_reasons(),
            settings.pass.candidate_joins(),
            settings.pass.candidate_filter(),
        );

        let statement = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            sql,
            [
                Value::from(import_run_id),
                Value::from(settings.max_attempts),
                Value::from(settings.batch_size),
            ],
        );

        state
            .database
            .query_all(statement)
            .await
            .or_raise(make_error)?
            .into_iter()
            .map(|row| Self::item_from_row(row, make_error))
            .collect()
    }

    async fn claim_candidates(
        state: &ServerState,
        import_run_id: i64,
        settings: &RenderFinalizerSettings,
    ) -> Result<Vec<RenderFinalizerItem>> {
        let make_error = || {
            Error::new(
                "failed to claim render-finalize candidate items",
                ErrorType::DatabaseQuery,
            )
        };

        let sql = format!(
            "
            WITH candidates AS (
                SELECT
                    item.import_run_id,
                    item.source_entity_id,
                    {} AS reasons
                FROM wikidot_corpus_import_item AS item
                LEFT JOIN page
                    ON page.page_id = item.page_id
                    AND page.deleted_at IS NULL
                {}
                WHERE item.import_run_id = $1
                AND {}
                AND (item.lease_until IS NULL OR item.lease_until <= NOW())
                AND item.attempts < $2
                ORDER BY item.updated_at ASC, item.source_fullname ASC
                LIMIT $3
                FOR UPDATE OF item SKIP LOCKED
            )
            , claimed AS (
                UPDATE wikidot_corpus_import_item AS item
                SET
                    state = 'render_running',
                    attempts = item.attempts + 1,
                    lease_until = NOW() + ($4::bigint * INTERVAL '1 second'),
                    error = NULL,
                    updated_at = NOW()
                FROM candidates
                WHERE item.import_run_id = candidates.import_run_id
                AND item.source_entity_id = candidates.source_entity_id
                RETURNING
                    item.source_entity_id,
                    item.import_run_id,
                    item.source_fullname,
                    item.page_id,
                    item.attempts,
                    candidates.reasons
            ), observations AS (
                INSERT INTO wikidot_corpus_render_observation (
                    import_run_id,
                    source_entity_id,
                    pass,
                    attempt,
                    page_id,
                    outcome
                )
                SELECT
                    import_run_id,
                    source_entity_id,
                    $5::text,
                    attempts,
                    page_id,
                    'running'
                FROM claimed
                RETURNING import_run_id, source_entity_id, pass, attempt
            )
            SELECT
                claimed.source_entity_id::text AS source_entity_id,
                claimed.import_run_id,
                claimed.source_fullname,
                claimed.page_id,
                (
                    SELECT page.site_id
                    FROM page
                    WHERE page.page_id = claimed.page_id
                    AND page.deleted_at IS NULL
                ) AS site_id,
                (
                    SELECT page.page_category_id
                    FROM page
                    WHERE page.page_id = claimed.page_id
                    AND page.deleted_at IS NULL
                ) AS page_category_id,
                claimed.attempts,
                claimed.reasons
            FROM claimed
            JOIN observations
                ON observations.import_run_id = claimed.import_run_id
                AND observations.source_entity_id = claimed.source_entity_id
                AND observations.attempt = claimed.attempts
            ",
            settings.pass.candidate_reasons(),
            settings.pass.candidate_joins(),
            settings.pass.candidate_filter(),
        );

        let statement = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            sql,
            [
                Value::from(import_run_id),
                Value::from(settings.max_attempts),
                Value::from(settings.batch_size),
                Value::from(settings.lease_seconds),
                Value::from(settings.pass.as_str()),
            ],
        );

        state
            .database
            .query_all(statement)
            .await
            .or_raise(make_error)?
            .into_iter()
            .map(|row| Self::item_from_row(row, make_error))
            .collect()
    }

    fn item_from_row(
        row: sea_orm::QueryResult,
        make_error: impl Fn() -> Error,
    ) -> Result<RenderFinalizerItem> {
        Ok(RenderFinalizerItem {
            import_run_id: row.try_get("", "import_run_id").or_raise(&make_error)?,
            source_entity_id: row
                .try_get("", "source_entity_id")
                .or_raise(&make_error)?,
            source_fullname: row.try_get("", "source_fullname").or_raise(&make_error)?,
            page_id: row.try_get("", "page_id").or_raise(&make_error)?,
            site_id: row.try_get("", "site_id").or_raise(&make_error)?,
            page_category_id: row
                .try_get("", "page_category_id")
                .or_raise(&make_error)?,
            attempts: row.try_get("", "attempts").or_raise(&make_error)?,
            reasons: row.try_get("", "reasons").or_raise(&make_error)?,
            outcome: None,
            error: None,
            error_chain: None,
            duration_ms: None,
            observation_completion: None,
        })
    }

    async fn render_claimed_items(
        state: &ServerState,
        settings: &RenderFinalizerSettings,
        items: Vec<RenderFinalizerItem>,
    ) -> Result<Vec<RenderFinalizerItem>> {
        let pass = settings.pass;
        let items =
            Self::render_items_concurrently(
                state,
                settings.concurrency,
                items,
                |state, item| async move {
                    Self::render_claimed_item(&state, pass, item).await
                },
            )
            .await;
        Self::complete_observations_batch(state, &items).await?;
        Ok(items)
    }

    async fn render_items_concurrently<State, Item, Fut>(
        state: &State,
        concurrency: usize,
        items: Vec<Item>,
        render: impl Fn(State, Item) -> Fut + Clone,
    ) -> Vec<Item>
    where
        State: Clone,
        Fut: Future<Output = Item>,
    {
        stream::iter(items)
            .map(|item| {
                let state = state.clone();
                let render = render.clone();
                async move { render(state, item).await }
            })
            .buffer_unordered(concurrency)
            .collect()
            .await
    }

    async fn render_claimed_item(
        state: &ServerState,
        pass: RenderFinalizerPass,
        mut item: RenderFinalizerItem,
    ) -> RenderFinalizerItem {
        let started_at = Instant::now();
        let trace = CorpusRenderTrace::new();
        let result = match (item.page_id, item.site_id, item.page_category_id) {
            (Some(page_id), Some(site_id), Some(page_category_id)) => {
                Self::render_page(
                    state,
                    &item,
                    PageId {
                        site_id,
                        category_id: page_category_id,
                        page_id,
                    },
                    pass,
                    &trace,
                )
                .await
            }
            _ => Err(Error::new(
                "render-finalize item has no live imported page",
                ErrorType::Render,
            )
            .into()),
        };

        match result {
            Ok(RenderPageOutcome::Completed(completion)) => {
                item.outcome = Some(str!(pass.success_state()));
                item.observation_completion = Some(completion);
            }
            Ok(RenderPageOutcome::PostCommitFailed {
                message,
                error_chain,
            }) => {
                item.error = Some(message);
                item.error_chain = Some(error_chain);
                item.outcome = Some(str!("render_failed"));
            }
            Err(error) => {
                let message = error.to_string();
                let error_chain = format!("{error:?}");
                if let Err(update_error) = Self::mark_item_failed(
                    state,
                    &item,
                    pass,
                    &message,
                    &error_chain,
                    &trace,
                )
                .await
                {
                    item.error = Some(format!(
                        "{message}; failed to mark item failed: {update_error}"
                    ));
                    item.error_chain = Some(format!(
                        "{error_chain}; failed to mark item failed: {update_error:?}"
                    ));
                } else {
                    item.error = Some(message);
                    item.error_chain = Some(error_chain);
                }
                item.outcome = Some(str!("render_failed"));
            }
        }

        item.duration_ms = Some(duration_millis(started_at.elapsed()));
        item
    }

    async fn render_page(
        state: &ServerState,
        item: &RenderFinalizerItem,
        id: PageId,
        pass: RenderFinalizerPass,
        trace: &CorpusRenderTrace,
    ) -> Result<RenderPageOutcome> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to render-finalize imported page {}",
                    item.source_fullname,
                ),
                ErrorType::Render,
            )
        };
        let txn = {
            let _stage = StageGuard::new(
                Some((trace, CorpusRenderScope::Finalizer)),
                CorpusRenderStage::TxBegin,
            );
            state.database.begin().await.or_raise(make_error)?
        };
        let result = async {
            let ctx = ServiceContext::new(state, &txn);
            PageRevisionService::rerender_for_corpus_finalizer_traced(
                &ctx,
                id,
                RerenderDepth::default(),
                RerenderType::Full,
                pass.rerender_outdates_dependents(),
                trace,
            )
            .await
            .or_raise(make_error)?;
            let core_snapshot = trace.snapshot();
            {
                let _stage = StageGuard::new(
                    Some((trace, CorpusRenderScope::Finalizer)),
                    CorpusRenderStage::Mark,
                );
                Self::record_observation_core(
                    &txn,
                    item,
                    pass.as_str(),
                    pass.success_state(),
                    &core_snapshot,
                )
                .await?;
            }
            ctx.drain_post_commit_actions().or_raise(make_error)
        }
        .await;

        match result {
            Ok(post_commit_actions) => {
                {
                    let _stage = StageGuard::new(
                        Some((trace, CorpusRenderScope::Finalizer)),
                        CorpusRenderStage::Commit,
                    );
                    txn.commit().await.or_raise(make_error)?;
                }
                let post_commit_error = {
                    let _stage = StageGuard::new(
                        Some((trace, CorpusRenderScope::Finalizer)),
                        CorpusRenderStage::PostCommit,
                    );
                    ServiceContext::run_post_commit_actions_for_state(
                        state,
                        post_commit_actions,
                    )
                    .await
                    .err()
                };
                if let Some(error) = &post_commit_error {
                    warn!(
                        "render-finalize committed page {} but post-commit actions failed: {}",
                        item.source_fullname, error,
                    );
                    let message = format!("post-commit actions failed: {error}");
                    let error_chain = format!("post-commit actions failed: {error:?}");
                    Self::mark_item_failed(
                        state,
                        item,
                        pass,
                        &message,
                        &error_chain,
                        trace,
                    )
                    .await?;
                    return Ok(RenderPageOutcome::PostCommitFailed {
                        message,
                        error_chain,
                    });
                }
                let final_snapshot = trace.snapshot();
                if final_snapshot.pipeline_us > CORPUS_RENDER_BUDGET_US as u64 {
                    warn!(
                        "render-finalize page {} exceeded the {}us budget: {}us (dominant={}.{})",
                        item.source_fullname,
                        CORPUS_RENDER_BUDGET_US,
                        final_snapshot.pipeline_us,
                        final_snapshot
                            .dominant_scope
                            .as_deref()
                            .unwrap_or("unknown"),
                        final_snapshot
                            .dominant_stage
                            .as_deref()
                            .unwrap_or("unknown"),
                    );
                }
                Ok(RenderPageOutcome::Completed(RenderObservationCompletion {
                    import_run_id: item.import_run_id,
                    source_entity_id: item.source_entity_id.clone(),
                    pass: pass.as_str(),
                    attempt: item.attempts,
                    outcome: pass.success_state(),
                    total_us: pg_micros(final_snapshot.pipeline_us),
                    dominant_scope: final_snapshot.dominant_scope,
                    dominant_stage: final_snapshot.dominant_stage,
                    terminal_scope: final_snapshot.terminal_scope,
                    terminal_stage: final_snapshot.terminal_stage,
                    timings: final_snapshot.timings,
                    dimensions: final_snapshot.dimensions,
                }))
            }
            Err(error) => {
                txn.rollback().await.or_raise(make_error)?;
                Err(error)
            }
        }
    }

    async fn record_observation_core(
        txn: &DatabaseTransaction,
        item: &RenderFinalizerItem,
        pass: &'static str,
        success_state: &'static str,
        snapshot: &CorpusRenderTraceSnapshot,
    ) -> Result<()> {
        // claim_candidates increments attempts atomically; matching it here
        // fences out a stale worker after an expired lease is reclaimed.
        debug_assert!(matches!(success_state, "rendered" | "done"));
        let make_error = || {
            Error::new(
                format!(
                    "failed to record render-finalize core observation for {} {}",
                    item.source_fullname, success_state,
                ),
                ErrorType::DatabaseQuery,
            )
        };
        let statement = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            str!(
                "
                WITH item_guard AS (
                    SELECT page_id
                    FROM wikidot_corpus_import_item
                    WHERE import_run_id = $1
                    AND source_entity_id = $2::uuid
                    AND attempts = $3
                    AND state = 'render_running'
                    FOR UPDATE
                ), observation_update AS (
                    UPDATE wikidot_corpus_render_observation
                    SET outcome = $4::text,
                        page_id = item_guard.page_id,
                        pipeline_us = $6,
                        dominant_scope = $7,
                        dominant_stage = $8,
                        terminal_scope = $9,
                        terminal_stage = $10,
                        timings = $11::jsonb,
                        dimensions = $12::jsonb
                    FROM item_guard
                    WHERE import_run_id = $1
                    AND source_entity_id = $2::uuid
                    AND pass = $5::text
                    AND attempt = $3
                    AND outcome = 'running'
                    AND complete = FALSE
                    RETURNING 1
                )
                SELECT
                    (SELECT COUNT(*) FROM item_guard)::bigint AS item_count,
                    (SELECT COUNT(*) FROM observation_update)::bigint AS observation_count
                "
            ),
            [
                Value::from(item.import_run_id),
                Value::from(item.source_entity_id.clone()),
                Value::from(item.attempts),
                Value::from(success_state),
                Value::from(pass),
                Value::from(pg_micros(snapshot.pipeline_us)),
                Value::from(snapshot.dominant_scope.clone()),
                Value::from(snapshot.dominant_stage.clone()),
                Value::from(snapshot.terminal_scope.clone()),
                Value::from(snapshot.terminal_stage.clone()),
                json_db_value(&snapshot.timings),
                json_db_value(&snapshot.dimensions),
            ],
        );
        let row = txn
            .query_one(statement)
            .await
            .or_raise(make_error)?
            .ok_or_else(make_error)?;
        let item_count: i64 = row.try_get("", "item_count").or_raise(make_error)?;
        let observation_count: i64 =
            row.try_get("", "observation_count").or_raise(make_error)?;
        if item_count != 1 || observation_count != 1 {
            return Err(Error::new(
                format!(
                    "expected one render-finalize item guard and core observation {}, found item={} observation={}",
                    success_state, item_count, observation_count,
                ),
                ErrorType::DatabaseQuery,
            )
            .into());
        }
        Ok(())
    }

    async fn mark_item_failed(
        state: &ServerState,
        item: &RenderFinalizerItem,
        pass: RenderFinalizerPass,
        message: &str,
        error_chain: &str,
        trace: &CorpusRenderTrace,
    ) -> Result<()> {
        // Use the same claim fence as the success path so a stale failure
        // cannot overwrite the state of a newer attempt.
        let make_error = || {
            Error::new(
                format!(
                    "failed to mark render-finalize item {} failed",
                    item.source_fullname,
                ),
                ErrorType::DatabaseQuery,
            )
        };
        let snapshot = trace.snapshot();
        let txn = state.database.begin().await.or_raise(make_error)?;
        let statement = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            str!(
                "
                WITH eligible AS (
                    SELECT item.page_id
                    FROM wikidot_corpus_import_item AS item
                    JOIN wikidot_corpus_render_observation AS observation
                        ON observation.import_run_id = item.import_run_id
                        AND observation.source_entity_id = item.source_entity_id
                        AND observation.pass = $6::text
                        AND observation.attempt = item.attempts
                        AND observation.outcome IN (
                            'running',
                            CASE $6::text WHEN 'pass1' THEN 'rendered' ELSE 'done' END
                        )
                        AND observation.complete = FALSE
                    WHERE item.import_run_id = $1
                    AND item.source_entity_id = $2::uuid
                    AND item.attempts = $3
                    AND item.state = 'render_running'
                    FOR UPDATE OF item, observation
                ), item_update AS (
                    UPDATE wikidot_corpus_import_item
                    SET state = 'render_failed',
                        lease_until = NULL,
                        error = jsonb_build_object(
                            'message', $4::text,
                            'error_chain', $5::text
                        ),
                        updated_at = NOW()
                    FROM eligible
                    WHERE import_run_id = $1
                    AND source_entity_id = $2::uuid
                    AND attempts = $3
                    AND state = 'render_running'
                    RETURNING wikidot_corpus_import_item.page_id
                ), observation_update AS (
                    UPDATE wikidot_corpus_render_observation
                    SET outcome = 'render_failed',
                        page_id = item_update.page_id,
                        pipeline_us = $7,
                        total_us = $8,
                        complete = TRUE,
                        dominant_scope = $9,
                        dominant_stage = $10,
                        terminal_scope = $11,
                        terminal_stage = $12,
                        timings = $13::jsonb,
                        dimensions = $14::jsonb,
                        error_fingerprint = $15,
                        post_commit_error = FALSE,
                        finished_at = NOW()
                    FROM item_update
                    WHERE import_run_id = $1
                    AND source_entity_id = $2::uuid
                    AND pass = $6::text
                    AND attempt = $3
                    AND outcome IN (
                        'running',
                        CASE $6::text WHEN 'pass1' THEN 'rendered' ELSE 'done' END
                    )
                    RETURNING 1
                )
                SELECT
                    (SELECT COUNT(*) FROM item_update)::bigint AS item_count,
                    (SELECT COUNT(*) FROM observation_update)::bigint AS observation_count
                "
            ),
            [
                Value::from(item.import_run_id),
                Value::from(item.source_entity_id.clone()),
                Value::from(item.attempts),
                Value::from(message.to_owned()),
                Value::from(error_chain.to_owned()),
                Value::from(pass.as_str()),
                Value::from(pg_micros(snapshot.pipeline_us)),
                Value::from(pg_micros(snapshot.pipeline_us)),
                Value::from(snapshot.dominant_scope.clone()),
                Value::from(snapshot.dominant_stage.clone()),
                Value::from(snapshot.terminal_scope.clone()),
                Value::from(snapshot.terminal_stage.clone()),
                json_db_value(&snapshot.timings),
                json_db_value(&snapshot.dimensions),
                Value::from(error_fingerprint(error_chain)),
            ],
        );
        let row = txn
            .query_one(statement)
            .await
            .or_raise(make_error)?
            .ok_or_else(make_error)?;
        let item_count: i64 = row.try_get("", "item_count").or_raise(make_error)?;
        let observation_count: i64 =
            row.try_get("", "observation_count").or_raise(make_error)?;
        if item_count != 1 || observation_count != 1 {
            txn.rollback().await.or_raise(make_error)?;
            return Err(Error::new(
                format!(
                    "expected to mark one render-finalize item and observation failed, marked item={} observation={}",
                    item_count, observation_count,
                ),
                ErrorType::DatabaseQuery,
            )
            .into());
        }
        txn.commit().await.or_raise(make_error)?;
        Ok(())
    }

    async fn complete_observations_batch(
        state: &ServerState,
        items: &[RenderFinalizerItem],
    ) -> Result<()> {
        let completions: Vec<&RenderObservationCompletion> = items
            .iter()
            .filter_map(|item| item.observation_completion.as_ref())
            .collect();
        if completions.is_empty() {
            return Ok(());
        }
        let make_error = || {
            Error::new(
                "failed to complete render-finalize performance observations",
                ErrorType::DatabaseQuery,
            )
        };
        let payload = serde_json::to_value(&completions).or_raise(make_error)?;
        let statement = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            str!(
                "
                WITH patches AS (
                    SELECT *
                    FROM jsonb_to_recordset($1::jsonb) AS patch (
                        import_run_id bigint,
                        source_entity_id text,
                        pass text,
                        attempt integer,
                        outcome text,
                        total_us bigint,
                        dominant_scope text,
                        dominant_stage text,
                        terminal_scope text,
                        terminal_stage text,
                        timings jsonb,
                        dimensions jsonb
                    )
                ), item_update AS (
                    UPDATE wikidot_corpus_import_item AS item
                    SET state = patches.outcome,
                        lease_until = NULL,
                        error = NULL,
                        updated_at = NOW()
                    FROM patches
                    WHERE item.import_run_id = patches.import_run_id
                    AND item.source_entity_id = patches.source_entity_id::uuid
                    AND item.attempts = patches.attempt
                    AND item.state = 'render_running'
                    AND EXISTS (
                        SELECT 1
                        FROM wikidot_corpus_render_observation AS observation
                        WHERE observation.import_run_id = item.import_run_id
                        AND observation.source_entity_id = item.source_entity_id
                        AND observation.pass = patches.pass
                        AND observation.attempt = item.attempts
                        AND observation.outcome = patches.outcome
                        AND observation.complete = FALSE
                    )
                    RETURNING
                        item.import_run_id,
                        item.source_entity_id,
                        item.attempts,
                        patches.pass,
                        patches.outcome,
                        patches.total_us,
                        patches.dominant_scope,
                        patches.dominant_stage,
                        patches.terminal_scope,
                        patches.terminal_stage,
                        patches.timings,
                        patches.dimensions
                ), observation_update AS (
                    UPDATE wikidot_corpus_render_observation AS observation
                    SET outcome = item_update.outcome,
                        total_us = item_update.total_us,
                        complete = TRUE,
                        dominant_scope = item_update.dominant_scope,
                        dominant_stage = item_update.dominant_stage,
                        terminal_scope = item_update.terminal_scope,
                        terminal_stage = item_update.terminal_stage,
                        timings = item_update.timings,
                        dimensions = item_update.dimensions,
                        post_commit_error = FALSE,
                        finished_at = NOW()
                    FROM item_update
                    WHERE observation.import_run_id = item_update.import_run_id
                    AND observation.source_entity_id = item_update.source_entity_id
                    AND observation.pass = item_update.pass
                    AND observation.attempt = item_update.attempts
                    AND observation.outcome = item_update.outcome
                    AND observation.complete = FALSE
                    RETURNING 1
                )
                SELECT
                    (SELECT COUNT(*) FROM item_update)::bigint AS item_count,
                    (SELECT COUNT(*) FROM observation_update)::bigint AS observation_count
                "
            ),
            [Value::Json(Some(Box::new(payload)))],
        );
        let txn = state.database.begin().await.or_raise(make_error)?;
        let row = txn
            .query_one(statement)
            .await
            .or_raise(make_error)?
            .ok_or_else(make_error)?;
        let item_count: i64 = row.try_get("", "item_count").or_raise(make_error)?;
        let observation_count: i64 =
            row.try_get("", "observation_count").or_raise(make_error)?;
        let expected = i64::try_from(completions.len()).unwrap_or(i64::MAX);
        if item_count != expected || observation_count != expected {
            txn.rollback().await.or_raise(make_error)?;
            return Err(Error::new(
                format!(
                    "expected to complete {} render items and observations, completed item={} observation={}",
                    expected, item_count, observation_count,
                ),
                ErrorType::DatabaseQuery,
            )
            .into());
        }
        txn.commit().await.or_raise(make_error)?;
        Ok(())
    }
}

fn pg_micros(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn json_db_value(value: &BTreeMap<String, u64>) -> Value {
    Value::Json(Some(Box::new(
        serde_json::to_value(value).expect("render trace maps always serialize"),
    )))
}

fn error_fingerprint(error_chain: &str) -> String {
    hex::encode(Sha256::digest(error_chain.as_bytes()))
}

fn duration_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

impl RenderFinalizerSummary {
    fn from_items(
        settings: &RenderFinalizerSettings,
        import_run_id: Option<i64>,
        items: Vec<RenderFinalizerItem>,
        elapsed_ms: u128,
    ) -> Self {
        let rendered = items
            .iter()
            .filter(|item| item.outcome.as_deref() == Some("rendered"))
            .count();
        let done = items
            .iter()
            .filter(|item| item.outcome.as_deref() == Some("done"))
            .count();
        let render_failed = items
            .iter()
            .filter(|item| item.outcome.as_deref() == Some("render_failed"))
            .count();
        let completed = rendered + done;
        let reason_counts = reason_counts(&items);

        Self {
            action: ACTION,
            dry_run: settings.dry_run,
            import_run_id,
            pass: settings.pass.as_str(),
            batch_size: settings.batch_size,
            concurrency: settings.concurrency,
            lease_seconds: settings.lease_seconds,
            max_attempts: settings.max_attempts,
            candidates: items.len(),
            claimed: if settings.dry_run { 0 } else { items.len() },
            rendered,
            done,
            render_failed,
            elapsed_ms,
            rows_per_sec: calculate_rows_per_second(completed, elapsed_ms),
            reason_counts,
            items,
        }
    }
}

fn parse_optional_positive_i64(name: &str, value: Option<String>) -> Result<Option<i64>> {
    value
        .map(|value| parse_positive_i64(name, Some(value), 0))
        .transpose()
}

fn reason_counts(items: &[RenderFinalizerItem]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for reason in items.iter().flat_map(|item| item.reasons.iter()) {
        *counts.entry(reason.clone()).or_insert(0) += 1;
    }
    counts
}

fn calculate_rows_per_second(completed: usize, elapsed_ms: u128) -> f64 {
    if completed == 0 || elapsed_ms == 0 {
        0.0
    } else {
        completed as f64 / (elapsed_ms as f64 / 1_000.0)
    }
}

fn parse_render_pass(name: &str, value: Option<String>) -> Result<RenderFinalizerPass> {
    let Some(value) = value else {
        return Ok(RenderFinalizerPass::Pass1);
    };
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "pass1" | "pass-1" => Ok(RenderFinalizerPass::Pass1),
        "2" | "pass2" | "pass-2" => Ok(RenderFinalizerPass::Pass2),
        _ => Err(Error::new(
            format!("{name} must be 1/pass1 or 2/pass2"),
            ErrorType::ConfigSetup,
        )
        .into()),
    }
}

fn parse_positive_i64(name: &str, value: Option<String>, default: i64) -> Result<i64> {
    let Some(value) = value else {
        return Ok(default);
    };
    let parsed = value.trim().parse::<i64>().or_raise(|| {
        Error::new(
            format!("{name} must be a positive integer"),
            ErrorType::ConfigSetup,
        )
    })?;
    if parsed > 0 {
        Ok(parsed)
    } else {
        Err(Error::new(
            format!("{name} must be a positive integer"),
            ErrorType::ConfigSetup,
        )
        .into())
    }
}

fn parse_positive_usize(
    name: &str,
    value: Option<String>,
    default: usize,
) -> Result<usize> {
    let Some(value) = value else {
        return Ok(default);
    };
    let parsed = parse_positive_i64(name, Some(value), 0)?;
    usize::try_from(parsed).or_raise(|| {
        Error::new(
            format!("{name} must fit into usize"),
            ErrorType::ConfigSetup,
        )
    })
}

fn parse_boolish(name: &str, value: Option<String>) -> Result<Option<bool>> {
    let Some(value) = value else {
        return Ok(None);
    };
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "on" => Ok(Some(true)),
        "0" | "false" | "no" | "n" | "off" => Ok(Some(false)),
        _ => Err(Error::new(
            format!("{name} must be boolish: true/false, 1/0, yes/no, or on/off"),
            ErrorType::ConfigSetup,
        )
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tokio::sync::{mpsc, oneshot};

    fn settings_from_pairs(pairs: &[(&str, &str)]) -> Result<RenderFinalizerSettings> {
        let values: HashMap<&str, &str> = pairs.iter().copied().collect();
        RenderFinalizerSettings::from_getter(|name| {
            values.get(name).map(|value| str!(*value))
        })
    }

    #[test]
    fn render_finalizer_settings_parse_defaults_and_env_values() {
        let defaults = settings_from_pairs(&[]).unwrap();

        assert_eq!(defaults.import_run_id, None);
        assert_eq!(defaults.pass, RenderFinalizerPass::Pass1);
        assert_eq!(defaults.batch_size, DEFAULT_BATCH_SIZE);
        assert_eq!(defaults.concurrency, DEFAULT_CONCURRENCY);
        assert_eq!(defaults.lease_seconds, DEFAULT_LEASE_SECONDS);
        assert_eq!(defaults.max_attempts, DEFAULT_MAX_ATTEMPTS);
        assert!(defaults.dry_run);

        let configured = settings_from_pairs(&[
            ("DEEPWELL_RENDER_IMPORT_RUN_ID", "42"),
            ("DEEPWELL_RENDER_PASS", "pass2"),
            ("DEEPWELL_RENDER_BATCH_SIZE", "25"),
            ("DEEPWELL_RENDER_CONCURRENCY", "8"),
            ("DEEPWELL_RENDER_LEASE_SECONDS", "60"),
            ("DEEPWELL_RENDER_MAX_ATTEMPTS", "5"),
            ("DEEPWELL_RENDER_DRY_RUN", "off"),
        ])
        .unwrap();

        assert_eq!(configured.import_run_id, Some(42));
        assert_eq!(configured.pass, RenderFinalizerPass::Pass2);
        assert_eq!(configured.batch_size, 25);
        assert_eq!(configured.concurrency, 8);
        assert_eq!(configured.lease_seconds, 60);
        assert_eq!(configured.max_attempts, 5);
        assert!(!configured.dry_run);
    }

    #[test]
    fn render_finalizer_settings_reject_non_positive_batch_size() {
        assert!(settings_from_pairs(&[("DEEPWELL_RENDER_BATCH_SIZE", "0")]).is_err());
        assert!(settings_from_pairs(&[("DEEPWELL_RENDER_CONCURRENCY", "0")]).is_err());
        assert!(settings_from_pairs(&[("DEEPWELL_RENDER_PASS", "third")]).is_err());
    }

    #[test]
    fn render_finalizer_pass_two_selects_dependency_sensitive_pages() {
        let filter = RenderFinalizerPass::Pass2.candidate_filter();

        assert!(filter.contains("backlinks|listpages|countpages|tagcloud"));
        assert!(filter.contains("page.slug = '_template'"));
        assert!(filter.contains("nav_category"));
        assert!(
            RenderFinalizerPass::Pass2
                .candidate_joins()
                .contains("revision_text")
        );
        assert!(
            RenderFinalizerPass::Pass2
                .candidate_reasons()
                .contains("source_backlinks_module")
        );
    }

    #[test]
    fn render_finalizer_passes_retry_failures_and_reclaim_expired_leases() {
        for pass in [RenderFinalizerPass::Pass1, RenderFinalizerPass::Pass2] {
            let filter = pass.candidate_filter();
            assert!(filter.contains("'render_failed'"));
            assert!(filter.contains("'render_running'"));
        }
    }

    #[test]
    fn render_finalizer_pass_success_state_tracks_pass_completion() {
        assert_eq!(RenderFinalizerPass::Pass1.success_state(), "rendered");
        assert_eq!(RenderFinalizerPass::Pass2.success_state(), "done");
    }

    #[test]
    fn render_finalizer_pass_selects_rerender_outdating_mode() {
        assert!(!RenderFinalizerPass::Pass1.rerender_outdates_dependents());
        assert!(RenderFinalizerPass::Pass2.rerender_outdates_dependents());
    }

    #[test]
    fn render_finalizer_reason_counts_summarizes_item_reasons() {
        let items = vec![
            RenderFinalizerItem {
                import_run_id: 1,
                source_entity_id: str!("00000000-0000-4000-8000-000000000001"),
                source_fullname: str!("nav:top"),
                page_id: Some(10),
                site_id: Some(20),
                page_category_id: Some(30),
                attempts: 1,
                reasons: vec![str!("nav_source"), str!("source_query_module")],
                outcome: None,
                error: None,
                error_chain: None,
                duration_ms: None,
                observation_completion: None,
            },
            RenderFinalizerItem {
                import_run_id: 1,
                source_entity_id: str!("00000000-0000-4000-8000-000000000002"),
                source_fullname: str!("_template"),
                page_id: Some(11),
                site_id: Some(20),
                page_category_id: Some(31),
                attempts: 1,
                reasons: vec![str!("template_source"), str!("source_query_module")],
                outcome: None,
                error: None,
                error_chain: None,
                duration_ms: None,
                observation_completion: None,
            },
        ];
        let counts = reason_counts(&items);

        assert_eq!(counts.get("nav_source"), Some(&1));
        assert_eq!(counts.get("template_source"), Some(&1));
        assert_eq!(counts.get("source_query_module"), Some(&2));
    }

    #[test]
    fn duration_millis_returns_whole_milliseconds() {
        assert_eq!(duration_millis(Duration::from_millis(42)), 42);
        assert_eq!(duration_millis(Duration::from_micros(1_999)), 1);
    }

    #[test]
    fn render_finalizer_item_serializes_diagnostics() {
        let item = RenderFinalizerItem {
            import_run_id: 1,
            source_entity_id: str!("00000000-0000-4000-8000-000000000001"),
            source_fullname: str!("error-page"),
            page_id: Some(10),
            site_id: Some(20),
            page_category_id: Some(30),
            attempts: 2,
            reasons: vec![str!("source_query_module")],
            outcome: Some(str!("render_failed")),
            error: Some(str!("render failed")),
            error_chain: Some(str!("render failed: caused by test")),
            duration_ms: Some(123),
            observation_completion: None,
        };

        let value = serde_json::to_value(item).unwrap();

        assert_eq!(value["source_fullname"], "error-page");
        assert_eq!(value["error"], "render failed");
        assert_eq!(value["error_chain"], "render failed: caused by test");
        assert_eq!(value["duration_ms"], 123);
        assert!(value.get("import_run_id").is_none());
        assert!(value.get("source_entity_id").is_none());
    }

    #[test]
    fn render_finalizer_rows_per_second_uses_rendered_and_done_counts() {
        let summary = build_summary(
            &RenderFinalizerSettings {
                import_run_id: Some(42),
                pass: RenderFinalizerPass::Pass2,
                batch_size: 25,
                concurrency: 8,
                lease_seconds: 60,
                max_attempts: 5,
                dry_run: false,
            },
            vec![
                test_item("alpha", Some("rendered"), &["source_query_module"]),
                test_item("beta", Some("done"), &["nav_source"]),
                test_item("gamma", Some("render_failed"), &["template_source"]),
            ],
            2_000,
        );

        assert_eq!(summary.elapsed_ms, 2_000);
        assert_eq!(summary.rendered, 1);
        assert_eq!(summary.done, 1);
        assert_eq!(summary.rows_per_sec, 1.0);
    }

    #[test]
    fn render_finalizer_rows_per_second_is_zero_for_zero_duration_or_zero_completed() {
        let no_elapsed = calculate_rows_per_second(5, 0);
        let no_completed = calculate_rows_per_second(0, 2_000);

        assert_eq!(no_elapsed, 0.0);
        assert_eq!(no_completed, 0.0);
    }

    #[tokio::test]
    async fn render_items_concurrently_yields_completion_order() {
        let (started_tx, mut started_rx) = mpsc::unbounded_channel();
        let mut release_senders = HashMap::new();
        let mut release_receivers = HashMap::new();
        for item in [1, 2, 3] {
            let (tx, rx) = oneshot::channel();
            release_senders.insert(item, tx);
            release_receivers.insert(item, rx);
        }
        let release_receivers = Arc::new(Mutex::new(release_receivers));

        let task = tokio::spawn({
            let release_receivers = Arc::clone(&release_receivers);
            async move {
                CorpusRenderFinalizerService::render_items_concurrently(
                    &(),
                    3,
                    vec![1, 2, 3],
                    move |(), item| {
                        let started_tx = started_tx.clone();
                        let release_receivers = Arc::clone(&release_receivers);
                        async move {
                            started_tx.send(item).unwrap();
                            let rx =
                                release_receivers.lock().unwrap().remove(&item).unwrap();
                            rx.await.unwrap();
                            item
                        }
                    },
                )
                .await
            }
        });

        let mut started = vec![
            started_rx.recv().await.unwrap(),
            started_rx.recv().await.unwrap(),
            started_rx.recv().await.unwrap(),
        ];
        started.sort_unstable();
        assert_eq!(started, vec![1, 2, 3]);

        release_senders.remove(&3).unwrap().send(()).unwrap();
        release_senders.remove(&2).unwrap().send(()).unwrap();
        release_senders.remove(&1).unwrap().send(()).unwrap();

        let rendered = task.await.unwrap();
        assert_eq!(rendered, vec![3, 2, 1]);
    }

    fn build_summary(
        settings: &RenderFinalizerSettings,
        items: Vec<RenderFinalizerItem>,
        elapsed_ms: u128,
    ) -> RenderFinalizerSummary {
        RenderFinalizerSummary::from_items(
            settings,
            settings.import_run_id,
            items,
            elapsed_ms,
        )
    }

    fn test_item(
        source_fullname: &str,
        outcome: Option<&str>,
        reasons: &[&str],
    ) -> RenderFinalizerItem {
        RenderFinalizerItem {
            import_run_id: 1,
            source_entity_id: str!("00000000-0000-4000-8000-000000000001"),
            source_fullname: source_fullname.to_string(),
            page_id: Some(10),
            site_id: Some(20),
            page_category_id: Some(30),
            attempts: 1,
            reasons: reasons.iter().map(|reason| reason.to_string()).collect(),
            outcome: outcome.map(str::to_string),
            error: None,
            error_chain: None,
            duration_ms: None,
            observation_completion: None,
        }
    }
}
