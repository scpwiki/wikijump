/*
 * services/view/article_cache.rs
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

use super::options::PageOptions;
use super::prelude::*;
use redis::AsyncCommands;
use sea_orm::{DatabaseBackend, FromQueryResult, Statement, Value};
use time::OffsetDateTime;

const ARTICLE_VIEW_PAGE_CACHE_PREFIX: &str = "deepwell:article-view:page:v1";

pub(super) struct ArticlePageCache;

impl ArticlePageCache {
    pub(super) async fn key(
        ctx: &ServiceContext<'_>,
        input: &GetPageView,
    ) -> Result<Option<String>> {
        if !matches!(input.session_token.as_deref(), None | Some("")) {
            return Ok(None);
        }

        let page_extra = input
            .route
            .as_ref()
            .map_or("", |route| route.extra.as_str());
        if PageOptions::parse(page_extra).rerender {
            return Ok(None);
        }

        #[derive(Debug, FromQueryResult)]
        struct ArticlePageCacheKeyRow {
            page_id: i64,
            page_updated_at: Option<OffsetDateTime>,
            latest_revision_id: Option<i64>,
            from_wikidot: bool,
            compiled_top_bar_html_hash: Option<Vec<u8>>,
            compiled_side_bar_html_hash: Option<Vec<u8>>,
        }

        let page_slug = input.route.as_ref().map(|route| route.slug.clone());
        let statement = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            str!(
                "
                SELECT
                    page.page_id,
                    page.updated_at AS page_updated_at,
                    page.latest_revision_id,
                    page.from_wikidot,
                    revision.compiled_top_bar_html_hash,
                    revision.compiled_side_bar_html_hash
                FROM site
                JOIN page
                    ON page.site_id = site.site_id
                    AND page.slug = COALESCE($2::text, site.default_page)
                    AND page.deleted_at IS NULL
                LEFT JOIN page_revision AS revision
                    ON revision.revision_id = page.latest_revision_id
                WHERE site.site_id = $1
                AND site.deleted_at IS NULL
                "
            ),
            [Value::from(input.site_id), Value::from(page_slug)],
        );

        let row = ArticlePageCacheKeyRow::find_by_statement(statement)
            .one(ctx.transaction())
            .await
            .or_raise(|| {
                Error::new(
                    "failed to load article page cache key",
                    ErrorType::DatabaseQuery,
                )
            })?;

        let Some(row) = row else {
            return Ok(None);
        };
        let Some(latest_revision_id) = row.latest_revision_id else {
            return Ok(None);
        };
        if !row.from_wikidot {
            return Ok(None);
        }

        let page_updated_at = row
            .page_updated_at
            .map(|value| value.unix_timestamp_nanos())
            .unwrap_or_default();
        let top_bar_hash = optional_hash_hex(row.compiled_top_bar_html_hash.as_deref());
        let side_bar_hash = optional_hash_hex(row.compiled_side_bar_html_hash.as_deref());
        let route_slug = input.route.as_ref().map_or("", |route| route.slug.as_str());
        let locales = input.locales.join(",");

        Ok(Some(format!(
            "{ARTICLE_VIEW_PAGE_CACHE_PREFIX}:site={}:page={}:rev={}:updated={}:top={}:side={}:slug={}:extra={}:locales={}",
            input.site_id,
            row.page_id,
            latest_revision_id,
            page_updated_at,
            top_bar_hash,
            side_bar_hash,
            hex::encode(route_slug),
            hex::encode(page_extra),
            hex::encode(locales),
        )))
    }

    pub(super) async fn get(
        ctx: &ServiceContext<'_>,
        cache_key: &str,
    ) -> Result<Option<GetPageViewOutput>> {
        let mut redis = ctx.redis();
        let cached: Option<String> = redis.get(cache_key).await.or_raise(|| {
            Error::new(
                "failed to read cached article page view",
                ErrorType::RedisQuery,
            )
        })?;

        cached
            .map(|cached| {
                serde_json::from_str(&cached).or_raise(|| {
                    Error::new(
                        "failed to parse cached article page view",
                        ErrorType::RedisQuery,
                    )
                })
            })
            .transpose()
    }

    pub(super) async fn set(
        ctx: &ServiceContext<'_>,
        cache_key: &str,
        page: &GetPageViewOutput,
    ) -> Result<()> {
        let serialized = serde_json::to_string(page).or_raise(|| {
            Error::new(
                "failed to serialize article page view for cache",
                ErrorType::RedisQuery,
            )
        })?;
        let mut redis = ctx.redis();
        redis
            .set::<_, _, ()>(cache_key, serialized)
            .await
            .or_raise(|| {
                Error::new(
                    "failed to write cached article page view",
                    ErrorType::RedisQuery,
                )
            })
    }
}

fn optional_hash_hex(hash: Option<&[u8]>) -> String {
    hash.map(hex::encode).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn article_page_cache_optional_hash_hex_handles_missing_hashes() {
        assert_eq!(optional_hash_hex(None), "");
        assert_eq!(optional_hash_hex(Some(&[0x0a, 0xff])), "0aff");
    }
}
