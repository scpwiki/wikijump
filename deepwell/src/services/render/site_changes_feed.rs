/*
 * services/render/site_changes_feed.rs
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

//! The read-only Wikidot `site-changes.xml` RSS feed.
//!
//! Current Wikidot serves this feed at `/feed/site-changes.xml` and links it
//! from the SiteChanges module view. It reports the same recent page activity
//! as `[[module SiteChanges]]` (source revisions and file revisions, newest
//! first) with the public revision identity, the change type, the author and a
//! preview of the page content, so it reuses the SiteChanges visibility rules
//! instead of a second permission model.

use super::service::RenderService;
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::{ServiceContext, SiteService, TextService};
use crate::types::{Action, Permission, Reference, Resource};
use sea_orm::{ConnectionTrait, FromQueryResult, Statement, Value};
use serde::Serialize;
use std::collections::BTreeMap;
use time::OffsetDateTime;

/// The bounded number of recent changes the feed reports.
///
/// The SiteChanges browser module selects its rows per page in the same
/// magnitude as the ListPages RSS limit, and an RSS feed reports one bounded
/// recent window rather than the whole revision history.
pub const SITE_CHANGES_FEED_ITEM_LIMIT: usize = 20;
const SITE_CHANGES_FEED_QUERY_BATCH: usize = 250;
const SITE_CHANGES_FEED_MAX_RAW_SCAN: usize = 5_000;

#[derive(Debug, FromQueryResult)]
struct SiteChangesFeedRevisionRow {
    revision_id: i64,
    revision_number: i32,
    is_new_page: bool,
    is_file: bool,
    created_at: OffsetDateTime,
    page_id: i64,
    page_category_id: i64,
    slug: String,
    title: String,
    changes: Vec<String>,
    hidden: Vec<String>,
    user_id: i64,
    wikidot_user_name: Option<String>,
    wikidot_user_slug: Option<String>,
    local_user_name: Option<String>,
    compiled_html_hash: Vec<u8>,
}

#[derive(Clone, Debug, Serialize)]
pub struct WikidotSiteChangesFeedItem {
    pub revision_id: i64,
    pub revision_number: i32,
    pub is_new_page: bool,
    pub is_file: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub slug: String,
    pub title: String,
    pub change_type: String,
    pub user_id: i64,
    pub user_name: String,
    pub body_html: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct WikidotSiteChangesFeedOutput {
    pub site_name: String,
    pub items: Vec<WikidotSiteChangesFeedItem>,
}

/// The change type Wikidot reports for one revision.
///
/// The SiteChanges module labels each row with one of these flags, so the feed
/// reuses that vocabulary rather than inventing one. A file revision and a newly
/// created page are their own types; other revisions report the most visible
/// field that changed.
pub fn wikidot_site_changes_change_type(
    is_file: bool,
    is_new_page: bool,
    changes: &[String],
) -> String {
    let has_change = |name: &str| changes.iter().any(|change| change == name);
    if is_file {
        return "file".to_owned();
    }
    if is_new_page {
        return "new page".to_owned();
    }
    if has_change("wikitext") {
        return "source".to_owned();
    }
    if has_change("title") || has_change("alt_title") {
        return "title".to_owned();
    }
    if has_change("slug") {
        return "page name".to_owned();
    }
    if has_change("tags") {
        return "tags".to_owned();
    }
    if has_change("parent") {
        return "metadata".to_owned();
    }
    "metadata".to_owned()
}

impl RenderService {
    pub async fn wikidot_site_changes_feed(
        ctx: &ServiceContext<'_>,
        site_id: i64,
    ) -> Result<Option<WikidotSiteChangesFeedOutput>> {
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
                format!("failed to check site changes feed site ID {site_id}"),
                ErrorType::Render,
            )
        })?;
        if !site_viewable {
            return Ok(None);
        }

        let site = SiteService::get(ctx, Reference::Id(site_id)).await?;
        let mut items = Vec::with_capacity(SITE_CHANGES_FEED_ITEM_LIMIT);
        let mut offset = 0usize;
        let mut permission_cache = BTreeMap::<i64, bool>::new();
        let make_error = || {
            Error::new(
                format!("failed to load site changes feed for site ID {site_id}"),
                ErrorType::Render,
            )
        };

        while offset < SITE_CHANGES_FEED_MAX_RAW_SCAN
            && items.len() < SITE_CHANGES_FEED_ITEM_LIMIT
        {
            let batch_size = SITE_CHANGES_FEED_QUERY_BATCH
                .min(SITE_CHANGES_FEED_MAX_RAW_SCAN.saturating_sub(offset));
            let rows = SiteChangesFeedRevisionRow::find_by_statement(
                Statement::from_sql_and_values(
                    ctx.transaction().get_database_backend(),
                    format!(
                        "SELECT revision_id, revision_number, is_new_page, \
                                is_file, created_at, page_id, page_category_id, slug, title, \
                                changes, hidden, user_id, wikidot_user_name, \
                                wikidot_user_slug, local_user_name, compiled_html_hash \
                         FROM ( \
                           SELECT pr.revision_id, 'source'::TEXT AS revision_type, \
                                  pr.revision_number, \
                                  pr.revision_type = 'create' AS is_new_page, \
                                  FALSE AS is_file, pr.created_at, pr.page_id, \
                                  p.page_category_id, pr.slug, pr.title, pr.changes, \
                                  pr.hidden, pr.user_id, pr.comments, \
                                  pr.compiled_body_html_hash AS compiled_html_hash, \
                                  wu.name AS wikidot_user_name, \
                                  wu.slug AS wikidot_user_slug, \
                                  local_user.name AS local_user_name \
                           FROM page_revision pr \
                           JOIN page p ON p.page_id = pr.page_id \
                                      AND p.site_id = pr.site_id \
                                      AND p.deleted_at IS NULL \
                           LEFT JOIN wikidot_user wu \
                             ON wu.user_id = pr.user_id AND wu.is_deleted = FALSE \
                           LEFT JOIN \"user\" local_user \
                             ON local_user.user_id = pr.user_id \
                            AND local_user.deleted_at IS NULL \
                           WHERE pr.site_id = $1 \
                             AND pr.revision_type NOT IN ('delete', 'undelete') \
                           UNION ALL \
                           SELECT fr.revision_id, 'file'::TEXT AS revision_type, \
                                  fr.revision_number, FALSE AS is_new_page, \
                                  TRUE AS is_file, fr.created_at, fr.page_id, \
                                  p.page_category_id, current_pr.slug, current_pr.title, \
                                  fr.changes, fr.hidden || current_pr.hidden, fr.user_id, \
                                  fr.comments, \
                                  current_pr.compiled_body_html_hash AS compiled_html_hash, \
                                  wu.name AS wikidot_user_name, \
                                  wu.slug AS wikidot_user_slug, \
                                  local_user.name AS local_user_name \
                           FROM file_revision fr \
                           JOIN page p ON p.page_id = fr.page_id \
                                      AND p.site_id = fr.site_id \
                                      AND p.deleted_at IS NULL \
                           JOIN page_revision current_pr \
                             ON current_pr.revision_id = p.latest_revision_id \
                           LEFT JOIN wikidot_user wu \
                             ON wu.user_id = fr.user_id AND wu.is_deleted = FALSE \
                           LEFT JOIN \"user\" local_user \
                             ON local_user.user_id = fr.user_id \
                            AND local_user.deleted_at IS NULL \
                         ) activity \
                         ORDER BY created_at DESC, revision_id DESC \
                         LIMIT {batch_size} OFFSET {offset}"
                    ),
                    [Value::from(site_id)],
                ),
            )
            .all(ctx.transaction())
            .await
            .or_raise(make_error)?;
            if rows.is_empty() {
                break;
            }
            let row_count = rows.len();
            offset = offset.saturating_add(row_count);

            for row in rows {
                // Hidden revision fields, hidden pages and deleted pages follow
                // the public SiteChanges contract.
                if row.hidden.iter().any(|field| {
                    ["comments", "title", "alt_title", "slug"].contains(&field.as_str())
                }) {
                    continue;
                }
                let can_view = if let Some(can_view) = permission_cache.get(&row.page_id)
                {
                    *can_view
                } else {
                    let can_view = PermissionService::check_user_can(
                        ctx,
                        &CheckPermissionContext {
                            user_id: viewer_user_id,
                            site_id,
                            page_reference: Some(Reference::Id(row.page_id)),
                        },
                        Permission {
                            resource_type: Resource::Page,
                            resource_category: Some(Reference::Id(row.page_category_id)),
                            action: Action::View,
                        },
                    )
                    .await
                    .or_raise(make_error)?;
                    permission_cache.insert(row.page_id, can_view);
                    can_view
                };
                if !can_view {
                    continue;
                }
                let user_name = row
                    .wikidot_user_name
                    .or(row.wikidot_user_slug)
                    .or(row.local_user_name)
                    .unwrap_or_else(|| row.user_id.to_string());
                let body_html = TextService::get(ctx, &row.compiled_html_hash)
                    .await
                    .or_raise(make_error)?;
                items.push(WikidotSiteChangesFeedItem {
                    revision_id: row.revision_id,
                    revision_number: row.revision_number,
                    is_new_page: row.is_new_page,
                    is_file: row.is_file,
                    created_at: row.created_at,
                    slug: row.slug,
                    title: row.title,
                    change_type: wikidot_site_changes_change_type(
                        row.is_file,
                        row.is_new_page,
                        &row.changes,
                    ),
                    user_id: row.user_id,
                    user_name,
                    body_html,
                });
                if items.len() == SITE_CHANGES_FEED_ITEM_LIMIT {
                    break;
                }
            }

            if row_count < batch_size {
                break;
            }
        }

        Ok(Some(WikidotSiteChangesFeedOutput {
            site_name: site.name,
            items,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::wikidot_site_changes_change_type;

    fn changes(fields: &[&str]) -> Vec<String> {
        fields.iter().map(|field| field.to_string()).collect()
    }

    #[test]
    fn site_changes_feed_change_types_reuse_the_site_changes_vocabulary() {
        assert_eq!(
            wikidot_site_changes_change_type(true, false, &changes(&["name"])),
            "file"
        );
        assert_eq!(
            wikidot_site_changes_change_type(false, true, &changes(&["wikitext"])),
            "new page"
        );
        assert_eq!(
            wikidot_site_changes_change_type(false, false, &changes(&["wikitext"])),
            "source"
        );
        assert_eq!(
            wikidot_site_changes_change_type(false, false, &changes(&["title"])),
            "title"
        );
        assert_eq!(
            wikidot_site_changes_change_type(false, false, &changes(&["alt_title"])),
            "title"
        );
        assert_eq!(
            wikidot_site_changes_change_type(false, false, &changes(&["slug"])),
            "page name"
        );
        assert_eq!(
            wikidot_site_changes_change_type(false, false, &changes(&["tags"])),
            "tags"
        );
        assert_eq!(
            wikidot_site_changes_change_type(false, false, &changes(&["parent"])),
            "metadata"
        );
        assert_eq!(
            wikidot_site_changes_change_type(false, false, &changes(&[])),
            "metadata"
        );
    }

    #[test]
    fn site_changes_feed_a_row_reports_the_most_visible_change() {
        assert_eq!(
            wikidot_site_changes_change_type(
                false,
                false,
                &changes(&["tags", "wikitext", "slug"])
            ),
            "source"
        );
        assert_eq!(
            wikidot_site_changes_change_type(false, false, &changes(&["parent", "tags"])),
            "tags"
        );
    }
}
