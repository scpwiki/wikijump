/*
 * services/render/sitemap.rs
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

//! The read-only Wikidot-compatible `sitemap.xml` page inventory.
//!
//! Current Wikidot publishes a sitemap index at `/sitemap.xml` with bounded
//! child pages shaped like `/sitemap_page_<n>.xml`, and advertises the index
//! from the site's `robots.txt`. This service returns the eligible public
//! page set behind those routes: non-deleted pages the viewer may view,
//! ordered by page id, with the revision timestamp crawlers use for
//! `<lastmod>`. It reuses the standard page-view permission derivation rather
//! than a second visibility model, the same way the SiteChanges feed does.

use super::runtime_page_queries::{ViewablePageRef, view_decisions_for_scanned_pages};
use super::service::RenderService;
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::ServiceContext;
use crate::types::{Action, Permission, Resource};
use sea_orm::{ConnectionTrait, FromQueryResult, Statement, Value};
use serde::Serialize;
use std::collections::BTreeMap;
use time::OffsetDateTime;

/// The bounded number of sitemap entries one site reports.
///
/// A full crawlable inventory rather than a recent window, but still bounded
/// so one response cannot grow without limit on very large sites.
pub const SITEMAP_MAX_ENTRIES: u64 = 50_000;

#[derive(Debug, FromQueryResult)]
struct SitemapPageRow {
    page_id: i64,
    page_category_id: i64,
    slug: String,
    last_modified: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize)]
pub struct WikidotSitemapEntry {
    pub slug: String,
    #[serde(with = "time::serde::rfc3339")]
    pub last_modified: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize)]
pub struct WikidotSitemapOutput {
    pub entries: Vec<WikidotSitemapEntry>,
}

impl RenderService {
    pub async fn wikidot_sitemap(
        ctx: &ServiceContext<'_>,
        site_id: i64,
    ) -> Result<Option<WikidotSitemapOutput>> {
        let viewer_user_id = ctx.request().user_id().ok();
        let site_viewable = PermissionService::check_user_can(
            ctx,
            &CheckPermissionContext {
                user_id: viewer_user_id,
                site_id,
                page_reference: None,
            },
            Permission {
                resource_type: Resource::Page,
                resource_category: None,
                action: Action::View,
            },
        )
        .await
        .or_raise(|| {
            Error::new(
                format!("failed to check sitemap visibility for site ID {site_id}"),
                ErrorType::Render,
            )
        })?;
        if !site_viewable {
            return Ok(None);
        }

        let make_error = || {
            Error::new(
                format!("failed to load sitemap entries for site ID {site_id}"),
                ErrorType::Render,
            )
        };
        let rows = SitemapPageRow::find_by_statement(Statement::from_sql_and_values(
            ctx.transaction().get_database_backend(),
            format!(
                "SELECT page_id, page_category_id, slug, \
                        COALESCE(updated_at, created_at) AS last_modified \
                 FROM page \
                 WHERE site_id = $1 AND deleted_at IS NULL \
                 ORDER BY page_id ASC \
                 LIMIT {limit}",
                limit = SITEMAP_MAX_ENTRIES,
            ),
            [Value::from(site_id)],
        ))
        .all(ctx.transaction())
        .await
        .or_raise(make_error)?;

        let refs = rows
            .iter()
            .map(|row| ViewablePageRef {
                page_id: row.page_id,
                site_id,
                page_category_id: Some(row.page_category_id),
            })
            .collect::<Vec<_>>();
        let mut category_permissions = BTreeMap::new();
        let decisions = view_decisions_for_scanned_pages(
            ctx,
            viewer_user_id,
            &refs,
            &mut category_permissions,
        )
        .await
        .or_raise(make_error)?;

        let entries = rows
            .into_iter()
            .zip(decisions)
            .filter(|(_, viewable)| *viewable)
            .map(|(row, _)| WikidotSitemapEntry {
                slug: row.slug,
                last_modified: row.last_modified,
            })
            .collect();

        Ok(Some(WikidotSitemapOutput { entries }))
    }
}
