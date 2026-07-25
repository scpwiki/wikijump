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
use super::structs::{GetPageView, GetPageViewOutput};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::BlueprintPageService;
use crate::services::ServiceContext;
use crate::services::blueprint::compose_template;
use crate::services::permission::PermissionCache;
use crate::services::public_cache::PublicContentCache;
use crate::services::render::{RenderDependencyClass, classify_render_dependencies};
use crate::utils::split_category;
use redis::AsyncCommands;
use sea_orm::{DatabaseBackend, FromQueryResult, Statement, Value};
use time::OffsetDateTime;

const ARTICLE_VIEW_PAGE_CACHE_PREFIX: &str = "deepwell:article-view:page:v2";

pub(super) struct ArticlePageCache;

pub(super) struct ArticlePageCacheMetadata {
    pub cache_key: String,
    pub public_content_cache_fence: String,
    pub anonymous_permission_cache_fence: String,
}

impl ArticlePageCache {
    pub(super) async fn metadata(
        ctx: &ServiceContext<'_>,
        input: &GetPageView,
    ) -> Result<Option<ArticlePageCacheMetadata>> {
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
            page_slug: String,
            page_updated_at: Option<OffsetDateTime>,
            latest_revision_id: Option<i64>,
            from_wikidot: bool,
            compiled_body_html_hash: Option<Vec<u8>>,
            compiled_body_styles_hash: Option<Vec<u8>>,
            compiled_top_bar_html_hash: Option<Vec<u8>>,
            compiled_side_bar_html_hash: Option<Vec<u8>>,
            source_contents: Option<String>,
        }

        let page_slug = input.route.as_ref().map(|route| route.slug.clone());
        let statement = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            str!(
                "
                SELECT
                    page.page_id,
                    page.slug AS page_slug,
                    page.updated_at AS page_updated_at,
                    page.latest_revision_id,
                    page.from_wikidot,
                    revision.compiled_body_html_hash,
                    revision.compiled_body_styles_hash,
                    revision.compiled_top_bar_html_hash,
                    revision.compiled_side_bar_html_hash,
                    source_text.contents AS source_contents
                FROM site
                JOIN page
                    ON page.site_id = site.site_id
                    AND page.slug = COALESCE($2::text, site.default_page)
                    AND page.deleted_at IS NULL
                LEFT JOIN page_revision AS revision
                    ON revision.revision_id = page.latest_revision_id
                LEFT JOIN text AS source_text
                    ON source_text.hash = revision.wikitext_hash
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

        let (category, page) = split_category(&row.page_slug);
        let template_source =
            BlueprintPageService::get_page_template(ctx, input.site_id, category, page)
                .await?;

        let page_updated_at = row
            .page_updated_at
            .map(|value| value.unix_timestamp_nanos())
            .unwrap_or_default();
        let public_content_cache_fence =
            PublicContentCache::cache_fence(ctx, input.site_id).await?;
        let permission_fence =
            PermissionCache::cache_fence(ctx, input.site_id, None).await?;
        let permission_fence = permission_fence.cache_key_fragment();
        let route_slug = input.route.as_ref().map_or("", |route| route.slug.as_str());
        let locales = input.locales.join(",");

        Ok(format_article_page_cache_key_if_source_eligible(
            row.source_contents.as_deref(),
            template_source.as_deref(),
            ArticlePageCacheKeyParts {
                site_id: input.site_id,
                page_id: row.page_id,
                latest_revision_id,
                page_updated_at,
                permission_fence: &permission_fence,
                compiled_body_html_hash: row.compiled_body_html_hash.as_deref(),
                compiled_body_styles_hash: row.compiled_body_styles_hash.as_deref(),
                compiled_top_bar_html_hash: row.compiled_top_bar_html_hash.as_deref(),
                compiled_side_bar_html_hash: row.compiled_side_bar_html_hash.as_deref(),
                route_slug,
                page_extra,
                locales: &locales,
            },
        )
        .map(|cache_key| ArticlePageCacheMetadata {
            cache_key,
            public_content_cache_fence,
            anonymous_permission_cache_fence: permission_fence,
        }))
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

struct ArticlePageCacheKeyParts<'a> {
    site_id: i64,
    page_id: i64,
    latest_revision_id: i64,
    page_updated_at: i128,
    permission_fence: &'a str,
    compiled_body_html_hash: Option<&'a [u8]>,
    compiled_body_styles_hash: Option<&'a [u8]>,
    compiled_top_bar_html_hash: Option<&'a [u8]>,
    compiled_side_bar_html_hash: Option<&'a [u8]>,
    route_slug: &'a str,
    page_extra: &'a str,
    locales: &'a str,
}

fn format_article_page_cache_key(parts: ArticlePageCacheKeyParts<'_>) -> String {
    let body_hash = optional_hash_hex(parts.compiled_body_html_hash);
    let styles_hash = optional_hash_hex(parts.compiled_body_styles_hash);
    let top_bar_hash = optional_hash_hex(parts.compiled_top_bar_html_hash);
    let side_bar_hash = optional_hash_hex(parts.compiled_side_bar_html_hash);

    format!(
        "{ARTICLE_VIEW_PAGE_CACHE_PREFIX}:site={}:page={}:rev={}:updated={}:permission={}:body={}:styles={}:top={}:side={}:slug={}:extra={}:locales={}",
        parts.site_id,
        parts.page_id,
        parts.latest_revision_id,
        parts.page_updated_at,
        parts.permission_fence,
        body_hash,
        styles_hash,
        top_bar_hash,
        side_bar_hash,
        hex::encode(parts.route_slug),
        hex::encode(parts.page_extra),
        hex::encode(parts.locales),
    )
}

fn format_article_page_cache_key_if_source_eligible(
    source_contents: Option<&str>,
    template_source: Option<&str>,
    parts: ArticlePageCacheKeyParts<'_>,
) -> Option<String> {
    let source_contents = source_contents?;
    let composed_source =
        template_source.map(|template| compose_template(template, source_contents));
    let effective_source = composed_source.as_deref().unwrap_or(source_contents);
    if !anonymous_article_cache_source_eligible(effective_source) {
        return None;
    }

    Some(format_article_page_cache_key(parts))
}

pub(super) fn anonymous_article_cache_source_eligible(source: &str) -> bool {
    let classes = classify_render_dependencies(source);
    !classes.contains(RenderDependencyClass::ViewerDependent)
        && !classes.contains(RenderDependencyClass::RequestDependent)
        && !classes.contains(RenderDependencyClass::UnsupportedUnverified)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn article_page_cache_optional_hash_hex_handles_missing_hashes() {
        assert_eq!(optional_hash_hex(None), "");
        assert_eq!(optional_hash_hex(Some(&[0x0a, 0xff])), "0aff");
    }

    #[test]
    fn article_page_cache_key_includes_compiled_body_hash() {
        let key = format_article_page_cache_key(ArticlePageCacheKeyParts {
            site_id: 7,
            page_id: 11,
            latest_revision_id: 13,
            page_updated_at: 17,
            permission_fence: "site=19,user=23",
            compiled_body_html_hash: Some(&[0x01, 0x23]),
            compiled_body_styles_hash: Some(&[0x34]),
            compiled_top_bar_html_hash: Some(&[0x45]),
            compiled_side_bar_html_hash: Some(&[0x67]),
            route_slug: "start",
            page_extra: "noredirect",
            locales: "en,ja",
        });

        assert_eq!(
            key,
            "deepwell:article-view:page:v2:site=7:page=11:rev=13:updated=17:permission=site=19,user=23:body=0123:styles=34:top=45:side=67:slug=7374617274:extra=6e6f7265646972656374:locales=656e2c6a61",
        );
    }

    #[test]
    fn article_page_cache_key_source_gate_allows_anonymous_safe_sources() {
        for source in [
            "Plain imported page text.\n\n[[div]]Static[[/div]]",
            "[[include component:license-box]]",
            "[[module ListPages category=\"fragment\"]]%%content%%[[/module]]",
            "[[module CountPages category=\"news\"]][[/module]]",
            "[[*user example]]",
            "[[[empty-label|]]]",
        ] {
            let key = format_article_page_cache_key_if_source_eligible(
                Some(source),
                None,
                ArticlePageCacheKeyParts {
                    site_id: 7,
                    page_id: 11,
                    latest_revision_id: 13,
                    page_updated_at: 17,
                    permission_fence: "site=19,user=23",
                    compiled_body_html_hash: Some(&[0x01, 0x23]),
                    compiled_body_styles_hash: Some(&[0x34]),
                    compiled_top_bar_html_hash: Some(&[0x45]),
                    compiled_side_bar_html_hash: Some(&[0x67]),
                    route_slug: "start",
                    page_extra: "noredirect",
                    locales: "en,ja",
                },
            );

            assert_eq!(
                key.as_deref(),
                Some(
                    "deepwell:article-view:page:v2:site=7:page=11:rev=13:updated=17:permission=site=19,user=23:body=0123:styles=34:top=45:side=67:slug=7374617274:extra=6e6f7265646972656374:locales=656e2c6a61"
                ),
                "{source}",
            );
        }
    }

    #[test]
    fn article_page_cache_key_source_gate_denies_missing_or_unsafe_source() {
        let parts = ArticlePageCacheKeyParts {
            site_id: 7,
            page_id: 11,
            latest_revision_id: 13,
            page_updated_at: 17,
            permission_fence: "site=19,user=23",
            compiled_body_html_hash: None,
            compiled_body_styles_hash: None,
            compiled_top_bar_html_hash: None,
            compiled_side_bar_html_hash: None,
            route_slug: "start",
            page_extra: "",
            locales: "en",
        };

        assert_eq!(
            format_article_page_cache_key_if_source_eligible(None, None, parts),
            None
        );

        for source in [
            "[[module CountPages offset=\"@URL|1\"]][[/module]]",
            "Request value @URL|0",
            "[[module Rate]]",
            "[[module UnknownWidget]]",
            "[[module]]",
        ] {
            let parts = ArticlePageCacheKeyParts {
                site_id: 7,
                page_id: 11,
                latest_revision_id: 13,
                page_updated_at: 17,
                permission_fence: "site=19,user=23",
                compiled_body_html_hash: None,
                compiled_body_styles_hash: None,
                compiled_top_bar_html_hash: None,
                compiled_side_bar_html_hash: None,
                route_slug: "start",
                page_extra: "",
                locales: "en",
            };

            assert_eq!(
                format_article_page_cache_key_if_source_eligible(
                    Some(source),
                    None,
                    parts,
                ),
                None,
                "{source}",
            );
        }
    }

    #[test]
    fn article_page_cache_key_source_gate_classifies_page_and_template_sources() {
        let parts = || ArticlePageCacheKeyParts {
            site_id: 7,
            page_id: 11,
            latest_revision_id: 13,
            page_updated_at: 17,
            permission_fence: "site=19,user=23",
            compiled_body_html_hash: None,
            compiled_body_styles_hash: None,
            compiled_top_bar_html_hash: None,
            compiled_side_bar_html_hash: None,
            route_slug: "category:article",
            page_extra: "",
            locales: "en",
        };
        let request_dependent_list_pages =
            "[[module ListPages offset=\"@URL|1\"]]%%title_linked%%[[/module]]";
        let split_request_dependent_template =
            "[[module ListPages offset=\"@U%%content%%\"]]%%title_linked%%[[/module]]";

        assert!(
            format_article_page_cache_key_if_source_eligible(
                Some("cache-safe page source"),
                Some("cache-safe template\n%%content%%"),
                parts(),
            )
            .is_some(),
        );
        assert_eq!(
            format_article_page_cache_key_if_source_eligible(
                Some(request_dependent_list_pages),
                None,
                parts(),
            ),
            None,
        );
        assert_eq!(
            format_article_page_cache_key_if_source_eligible(
                Some("cache-safe page source"),
                Some(request_dependent_list_pages),
                parts(),
            ),
            None,
        );
        assert_eq!(
            format_article_page_cache_key_if_source_eligible(
                Some("RL|1"),
                Some(split_request_dependent_template),
                parts(),
            ),
            None,
        );
    }

    #[test]
    fn article_page_cache_eligibility_allows_anonymous_safe_sources() {
        for source in [
            "Plain imported page text.\n\n[[div]]Static[[/div]]",
            "[[include component:license-box]]",
            "[[module ListPages category=\"fragment\"]]%%content%%[[/module]]",
            "[[module CountPages category=\"news\"]][[/module]]",
            "[[*user example]]",
        ] {
            assert!(anonymous_article_cache_source_eligible(source), "{source}");
        }
    }

    #[test]
    fn article_page_cache_eligibility_denies_unsafe_or_unverified_sources() {
        for source in [
            "[[module CountPages offset=\"@URL|1\"]][[/module]]",
            "Request value @URL|0",
            "[[module Rate]]",
            "[[module Members]]",
            "[[module NewPage]]",
            "[[module Clone]]",
            "[[module UnknownWidget]]",
            "[[module]]",
        ] {
            assert!(!anonymous_article_cache_source_eligible(source));
        }
    }
}
