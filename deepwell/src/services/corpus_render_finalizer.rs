/*
 * services/corpus_render_finalizer.rs
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

use super::prelude::*;
use crate::api::ServerState;
use crate::services::PageRevisionService;
use crate::types::{PageId, RerenderDepth};
use futures::{StreamExt, stream};
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, Value};
use sea_orm::{DatabaseTransaction, TransactionTrait};
use std::env;

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
    pub batch_size: i64,
    pub concurrency: usize,
    pub lease_seconds: i64,
    pub max_attempts: i64,
    pub dry_run: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct RenderFinalizerSummary {
    action: &'static str,
    dry_run: bool,
    import_run_id: Option<i64>,
    batch_size: i64,
    concurrency: usize,
    lease_seconds: i64,
    max_attempts: i64,
    candidates: usize,
    claimed: usize,
    rendered: usize,
    render_failed: usize,
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
    outcome: Option<String>,
    error: Option<String>,
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
        let import_run_id = match settings.import_run_id {
            Some(import_run_id) => Some(import_run_id),
            None => Self::select_latest_import_run(state).await?,
        };

        let items = match import_run_id {
            Some(import_run_id) => {
                if settings.dry_run {
                    Self::list_candidates(state, import_run_id, &settings).await?
                } else {
                    let claimed =
                        Self::claim_candidates(state, import_run_id, &settings).await?;
                    Self::render_claimed_items(state, &settings, claimed).await
                }
            }
            None => Vec::new(),
        };
        let rendered = items
            .iter()
            .filter(|item| item.outcome.as_deref() == Some("rendered"))
            .count();
        let render_failed = items
            .iter()
            .filter(|item| item.outcome.as_deref() == Some("render_failed"))
            .count();

        Ok(RenderFinalizerSummary {
            action: ACTION,
            dry_run: settings.dry_run,
            import_run_id,
            batch_size: settings.batch_size,
            concurrency: settings.concurrency,
            lease_seconds: settings.lease_seconds,
            max_attempts: settings.max_attempts,
            candidates: items.len(),
            claimed: if settings.dry_run { 0 } else { items.len() },
            rendered,
            render_failed,
            items,
        })
    }

    async fn select_latest_import_run(state: &ServerState) -> Result<Option<i64>> {
        let make_error = || {
            Error::new(
                "failed to select latest render-finalize import run",
                ErrorType::DatabaseQuery,
            )
        };

        let statement = Statement::from_string(
            DatabaseBackend::Postgres,
            str!(
                "
                SELECT import_run_id
                FROM wikidot_corpus_import_run AS run
                WHERE state IN ('running', 'rendering')
                AND EXISTS (
                    SELECT 1
                    FROM wikidot_corpus_import_item AS item
                    WHERE item.import_run_id = run.import_run_id
                    AND item.state = 'render_pending'
                )
                ORDER BY started_at DESC, import_run_id DESC
                LIMIT 1
                "
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

        let statement = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            str!(
                "
                SELECT
                    item.import_run_id,
                    item.source_entity_id::text AS source_entity_id,
                    item.source_fullname,
                    item.page_id,
                    page.site_id,
                    page.page_category_id,
                    item.attempts
                FROM wikidot_corpus_import_item AS item
                LEFT JOIN page
                    ON page.page_id = item.page_id
                    AND page.deleted_at IS NULL
                WHERE item.import_run_id = $1
                AND item.state = 'render_pending'
                AND (item.lease_until IS NULL OR item.lease_until <= NOW())
                AND item.attempts < $2
                ORDER BY item.updated_at ASC, item.source_fullname ASC
                LIMIT $3
                "
            ),
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

        let statement = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            str!(
                "
                WITH candidates AS (
                    SELECT item.import_run_id, item.source_entity_id
                    FROM wikidot_corpus_import_item AS item
                    WHERE item.import_run_id = $1
                    AND item.state = 'render_pending'
                    AND (item.lease_until IS NULL OR item.lease_until <= NOW())
                    AND item.attempts < $2
                    ORDER BY item.updated_at ASC, item.source_fullname ASC
                    LIMIT $3
                    FOR UPDATE SKIP LOCKED
                )
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
                    item.source_entity_id::text AS source_entity_id,
                    item.import_run_id,
                    item.source_fullname,
                    item.page_id,
                    (
                        SELECT page.site_id
                        FROM page
                        WHERE page.page_id = item.page_id
                        AND page.deleted_at IS NULL
                    ) AS site_id,
                    (
                        SELECT page.page_category_id
                        FROM page
                        WHERE page.page_id = item.page_id
                        AND page.deleted_at IS NULL
                    ) AS page_category_id,
                    item.attempts
                "
            ),
            [
                Value::from(import_run_id),
                Value::from(settings.max_attempts),
                Value::from(settings.batch_size),
                Value::from(settings.lease_seconds),
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
            outcome: None,
            error: None,
        })
    }

    async fn render_claimed_items(
        state: &ServerState,
        settings: &RenderFinalizerSettings,
        items: Vec<RenderFinalizerItem>,
    ) -> Vec<RenderFinalizerItem> {
        stream::iter(items)
            .map(|item| {
                let state = state.clone();
                async move { Self::render_claimed_item(&state, item).await }
            })
            .buffered(settings.concurrency)
            .collect()
            .await
    }

    async fn render_claimed_item(
        state: &ServerState,
        mut item: RenderFinalizerItem,
    ) -> RenderFinalizerItem {
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
            Ok(()) => {
                item.outcome = Some(str!("rendered"));
            }
            Err(error) => {
                let message = error.to_string();
                if let Err(update_error) =
                    Self::mark_item_failed(state, &item, &message).await
                {
                    item.error = Some(format!(
                        "{message}; failed to mark item failed: {update_error}"
                    ));
                } else {
                    item.error = Some(message);
                }
                item.outcome = Some(str!("render_failed"));
            }
        }

        item
    }

    async fn render_page(
        state: &ServerState,
        item: &RenderFinalizerItem,
        id: PageId,
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to render-finalize imported page {}",
                    item.source_fullname,
                ),
                ErrorType::Render,
            )
        };
        let txn = state.database.begin().await.or_raise(make_error)?;
        let result = async {
            let ctx = ServiceContext::new(state, &txn);
            PageRevisionService::rerender_without_outdating(
                &ctx,
                id,
                RerenderDepth::default(),
            )
            .await
            .or_raise(make_error)?;
            Self::mark_item_rendered(&txn, item).await?;
            ctx.drain_post_commit_actions().or_raise(make_error)
        }
        .await;

        match result {
            Ok(post_commit_actions) => {
                txn.commit().await.or_raise(make_error)?;
                if let Err(error) = ServiceContext::run_post_commit_actions_for_state(
                    state,
                    post_commit_actions,
                )
                .await
                {
                    warn!(
                        "render-finalize committed page {} but post-commit actions failed: {}",
                        item.source_fullname, error,
                    );
                }
                Ok(())
            }
            Err(error) => {
                txn.rollback().await.or_raise(make_error)?;
                Err(error)
            }
        }
    }

    async fn mark_item_rendered(
        txn: &DatabaseTransaction,
        item: &RenderFinalizerItem,
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to mark render-finalize item {} rendered",
                    item.source_fullname,
                ),
                ErrorType::DatabaseQuery,
            )
        };
        let statement = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            str!(
                "
                UPDATE wikidot_corpus_import_item
                SET state = 'rendered',
                    lease_until = NULL,
                    error = NULL,
                    updated_at = NOW()
                WHERE import_run_id = $1
                AND source_entity_id = $2::uuid
                AND state = 'render_running'
                "
            ),
            [
                Value::from(item.import_run_id),
                Value::from(item.source_entity_id.clone()),
            ],
        );
        let result = txn.execute(statement).await.or_raise(make_error)?;
        if result.rows_affected() != 1 {
            return Err(Error::new(
                format!(
                    "expected to mark one render-finalize item rendered, marked {}",
                    result.rows_affected(),
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
        message: &str,
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to mark render-finalize item {} failed",
                    item.source_fullname,
                ),
                ErrorType::DatabaseQuery,
            )
        };
        let statement = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            str!(
                "
                UPDATE wikidot_corpus_import_item
                SET state = 'render_failed',
                    lease_until = NULL,
                    error = jsonb_build_object('message', $3::text),
                    updated_at = NOW()
                WHERE import_run_id = $1
                AND source_entity_id = $2::uuid
                AND state = 'render_running'
                "
            ),
            [
                Value::from(item.import_run_id),
                Value::from(item.source_entity_id.clone()),
                Value::from(message.to_owned()),
            ],
        );
        state
            .database
            .execute(statement)
            .await
            .or_raise(make_error)
            .and_then(|result| {
                if result.rows_affected() == 1 {
                    Ok(())
                } else {
                    Err(Error::new(
                        format!(
                            "expected to mark one render-finalize item failed, marked {}",
                            result.rows_affected(),
                        ),
                        ErrorType::DatabaseQuery,
                    )
                    .into())
                }
            })?;
        Ok(())
    }
}

fn parse_optional_positive_i64(name: &str, value: Option<String>) -> Result<Option<i64>> {
    value
        .map(|value| parse_positive_i64(name, Some(value), 0))
        .transpose()
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
        assert_eq!(defaults.batch_size, DEFAULT_BATCH_SIZE);
        assert_eq!(defaults.concurrency, DEFAULT_CONCURRENCY);
        assert_eq!(defaults.lease_seconds, DEFAULT_LEASE_SECONDS);
        assert_eq!(defaults.max_attempts, DEFAULT_MAX_ATTEMPTS);
        assert!(defaults.dry_run);

        let configured = settings_from_pairs(&[
            ("DEEPWELL_RENDER_IMPORT_RUN_ID", "42"),
            ("DEEPWELL_RENDER_BATCH_SIZE", "25"),
            ("DEEPWELL_RENDER_CONCURRENCY", "8"),
            ("DEEPWELL_RENDER_LEASE_SECONDS", "60"),
            ("DEEPWELL_RENDER_MAX_ATTEMPTS", "5"),
            ("DEEPWELL_RENDER_DRY_RUN", "off"),
        ])
        .unwrap();

        assert_eq!(configured.import_run_id, Some(42));
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
    }
}
