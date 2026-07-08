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
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, Value};
use std::env;

const ACTION: &str = "render-finalize";
const DEFAULT_BATCH_SIZE: i64 = 100;
const DEFAULT_LEASE_SECONDS: i64 = 300;
const DEFAULT_MAX_ATTEMPTS: i64 = 3;

#[derive(Debug)]
pub struct CorpusRenderFinalizerService;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFinalizerSettings {
    pub import_run_id: Option<i64>,
    pub batch_size: i64,
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
    lease_seconds: i64,
    max_attempts: i64,
    candidates: usize,
    items: Vec<RenderFinalizerItem>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct RenderFinalizerItem {
    source_fullname: String,
    page_id: Option<i64>,
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
        if !settings.dry_run {
            return Err(Error::new(
                "render-finalize without dry-run is not implemented; actual rendering lands in Task 6b",
                ErrorType::Render,
            )
            .into());
        }

        let import_run_id = match settings.import_run_id {
            Some(import_run_id) => Some(import_run_id),
            None => Self::select_latest_import_run(state).await?,
        };

        let items = match import_run_id {
            Some(import_run_id) => {
                Self::list_candidates(state, import_run_id, &settings).await?
            }
            None => Vec::new(),
        };

        Ok(RenderFinalizerSummary {
            action: ACTION,
            dry_run: settings.dry_run,
            import_run_id,
            batch_size: settings.batch_size,
            lease_seconds: settings.lease_seconds,
            max_attempts: settings.max_attempts,
            candidates: items.len(),
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
                SELECT source_fullname, page_id
                FROM wikidot_corpus_import_item
                WHERE import_run_id = $1
                AND state = 'render_pending'
                AND (lease_until IS NULL OR lease_until <= NOW())
                AND attempts < $2
                ORDER BY updated_at ASC, source_fullname ASC
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
            .map(|row| {
                Ok(RenderFinalizerItem {
                    source_fullname: row
                        .try_get("", "source_fullname")
                        .or_raise(make_error)?,
                    page_id: row.try_get("", "page_id").or_raise(make_error)?,
                })
            })
            .collect()
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
        assert_eq!(defaults.lease_seconds, DEFAULT_LEASE_SECONDS);
        assert_eq!(defaults.max_attempts, DEFAULT_MAX_ATTEMPTS);
        assert!(defaults.dry_run);

        let configured = settings_from_pairs(&[
            ("DEEPWELL_RENDER_IMPORT_RUN_ID", "42"),
            ("DEEPWELL_RENDER_BATCH_SIZE", "25"),
            ("DEEPWELL_RENDER_LEASE_SECONDS", "60"),
            ("DEEPWELL_RENDER_MAX_ATTEMPTS", "5"),
            ("DEEPWELL_RENDER_DRY_RUN", "off"),
        ])
        .unwrap();

        assert_eq!(configured.import_run_id, Some(42));
        assert_eq!(configured.batch_size, 25);
        assert_eq!(configured.lease_seconds, 60);
        assert_eq!(configured.max_attempts, 5);
        assert!(!configured.dry_run);
    }

    #[test]
    fn render_finalizer_settings_reject_non_positive_batch_size() {
        assert!(settings_from_pairs(&[("DEEPWELL_RENDER_BATCH_SIZE", "0")]).is_err());
    }
}
