/*
 * services/render/site_changes.rs
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

//! The initial, read-only Wikidot `SiteChanges` module view.

use super::categories::wikidot_category_sort_key;
use super::compat::CompatHtmlFragments;
use super::literal_regions::LiteralRegionIndex;
use super::service::{
    RenderService, escape_list_pages_html_attr, escape_list_pages_html_text,
    format_wikidot_list_pages_date,
};
use super::wikidot_user_resources::WIKIDOT_USER_INFO_URL;
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::{CategoryService, ServiceContext};
use crate::types::{Action, Permission, Reference, Resource};
use ftml::settings::WikitextSettings;
use regex::Regex;
use sea_orm::{ConnectionTrait, FromQueryResult, Statement};
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::LazyLock;

const SITE_CHANGES_BROWSER_ROWS_PER_PAGE: usize = 20;
const SITE_CHANGES_QUERY_BATCH: usize = 250;
const SITE_CHANGES_MAX_RAW_SCAN: usize = 5_000;
const SITE_CHANGES_EMPTY: &str = "Sorry, no revisions matching your criteria.";

pub(super) static SITE_CHANGES_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[\t ]*\[\[module[\t ]+SiteChanges[\t ]*\]\][\t ]*$")
        .expect("SiteChanges module expression is valid")
});

#[derive(Debug, FromQueryResult)]
struct SiteChangesRevisionRow {
    is_new_page: bool,
    is_file: bool,
    created_at: time::OffsetDateTime,
    revision_number: i32,
    page_id: i64,
    page_category_id: i64,
    category_slug: String,
    slug: String,
    title: String,
    changes: Vec<String>,
    comments: String,
    hidden: Vec<String>,
    user_id: i64,
    wikidot_user_name: Option<String>,
    wikidot_user_slug: Option<String>,
    local_user_name: Option<String>,
}

#[derive(Debug)]
struct SiteChangesCategory {
    category_id: i64,
    slug: String,
}

#[derive(Debug)]
struct SiteChangesSnapshot {
    categories: Vec<SiteChangesCategory>,
    list: SiteChangesList,
}

#[derive(Debug)]
struct SiteChangesList {
    page: u32,
    last_link_page: u32,
    has_next: bool,
    has_dots: bool,
    revisions: Vec<SiteChangesRevisionRow>,
}

#[derive(Debug)]
pub(crate) enum SiteChangesLoad<T> {
    Complete(T),
    Saturated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WikidotSiteChangesFilter {
    All,
    Source,
    Files,
}

impl WikidotSiteChangesFilter {
    pub fn from_browser_options(options: &str) -> Option<Self> {
        match options {
            "{}" | "{\"all\":true}" => Some(Self::All),
            "{\"source\":true}" => Some(Self::Source),
            "{\"files\":true}" => Some(Self::Files),
            _ => None,
        }
    }

    pub fn from_wikidot_py_options(options: &str) -> Option<Self> {
        match options {
            "{}" | "{\"all\":true}" => Some(Self::All),
            "{\"source\":true}" => Some(Self::Source),
            "{\"files\":true}" => Some(Self::Files),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Source => "source",
            Self::Files => "files",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WikidotSiteChangesModuleRequest {
    pub page: u32,
    pub rows_per_page: usize,
    pub category_id: Option<i64>,
    pub filter: WikidotSiteChangesFilter,
}

#[derive(Clone, Debug, Serialize)]
pub struct WikidotSiteChangesModuleResponse {
    pub status: String,
    pub body: String,
}

pub(crate) fn wikidot_site_changes_empty_response() -> WikidotSiteChangesModuleResponse {
    WikidotSiteChangesModuleResponse {
        status: "ok".to_owned(),
        body: SITE_CHANGES_EMPTY.to_owned(),
    }
}

pub(super) async fn expand_site_changes_modules(
    ctx: &ServiceContext<'_>,
    wikitext: String,
    settings: &WikitextSettings,
    current_site_id: Option<i64>,
    viewer_user_id: Option<i64>,
    compat_html: &mut CompatHtmlFragments,
) -> Result<String> {
    if !settings.enable_page_syntax || !SITE_CHANGES_MODULE_REGEX.is_match(&wikitext) {
        return Ok(wikitext);
    }
    let Some(current_site_id) = current_site_id else {
        return Ok(wikitext);
    };
    let snapshot = match load_site_changes_snapshot(
        ctx,
        current_site_id,
        viewer_user_id,
        WikidotSiteChangesModuleRequest {
            page: 1,
            rows_per_page: SITE_CHANGES_BROWSER_ROWS_PER_PAGE,
            category_id: None,
            filter: WikidotSiteChangesFilter::All,
        },
    )
    .await?
    {
        SiteChangesLoad::Complete(snapshot) => snapshot,
        SiteChangesLoad::Saturated => return Ok(wikitext),
    };

    let literal_regions = LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
    let mut output = String::with_capacity(wikitext.len());
    let mut cursor = 0;
    for matched in SITE_CHANGES_MODULE_REGEX.find_iter(&wikitext) {
        if literal_regions.contains(matched.start()) {
            continue;
        }
        output.push_str(&wikitext[cursor..matched.start()]);
        output.push_str(
            &compat_html.push_block_html(render_site_changes_snapshot(&snapshot)),
        );
        cursor = matched.end();
    }
    if cursor == 0 {
        return Ok(wikitext);
    }
    output.push_str(&wikitext[cursor..]);
    Ok(output)
}

async fn load_site_changes_snapshot(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    viewer_user_id: Option<i64>,
    request: WikidotSiteChangesModuleRequest,
) -> Result<SiteChangesLoad<SiteChangesSnapshot>> {
    let list = match load_site_changes_list(ctx, site_id, viewer_user_id, request).await?
    {
        SiteChangesLoad::Complete(list) => list,
        SiteChangesLoad::Saturated => return Ok(SiteChangesLoad::Saturated),
    };
    let categories = load_site_changes_categories(ctx, site_id, viewer_user_id).await?;
    Ok(SiteChangesLoad::Complete(SiteChangesSnapshot {
        categories,
        list,
    }))
}

async fn load_site_changes_list(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    viewer_user_id: Option<i64>,
    request: WikidotSiteChangesModuleRequest,
) -> Result<SiteChangesLoad<SiteChangesList>> {
    let make_error = || {
        Error::new(
            format!("failed to load SiteChanges rows for site ID {site_id}"),
            ErrorType::Render,
        )
    };
    let page_start = usize::try_from(request.page.saturating_sub(1))
        .ok()
        .and_then(|page| page.checked_mul(request.rows_per_page))
        .unwrap_or(SITE_CHANGES_MAX_RAW_SCAN);
    if page_start >= SITE_CHANGES_MAX_RAW_SCAN {
        return Ok(SiteChangesLoad::Complete(SiteChangesList {
            page: request.page,
            last_link_page: request.page,
            has_next: false,
            has_dots: false,
            revisions: Vec::new(),
        }));
    }
    let last_candidate_page = request.page.saturating_add(2);
    let visible_row_target = usize::try_from(last_candidate_page)
        .ok()
        .and_then(|page| page.checked_mul(request.rows_per_page))
        .and_then(|rows| rows.checked_add(1))
        .unwrap_or(SITE_CHANGES_MAX_RAW_SCAN)
        .min(SITE_CHANGES_MAX_RAW_SCAN);
    let txn = ctx.transaction();
    let mut raw_offset = 0usize;
    let mut revisions = Vec::with_capacity(visible_row_target);
    let mut permission_cache = BTreeMap::<i64, bool>::new();
    let mut naturally_exhausted = false;

    while revisions.len() < visible_row_target && raw_offset < SITE_CHANGES_MAX_RAW_SCAN {
        let batch_size = SITE_CHANGES_QUERY_BATCH
            .min(SITE_CHANGES_MAX_RAW_SCAN.saturating_sub(raw_offset));
        let rows =
            SiteChangesRevisionRow::find_by_statement(Statement::from_sql_and_values(
                txn.get_database_backend(),
                format!(
                    "SELECT revision_id, is_new_page, is_file, created_at, \
                            revision_number, page_id, page_category_id, category_slug, \
                            slug, title, changes, comments, hidden, user_id, \
                            wikidot_user_name, wikidot_user_slug, local_user_name \
                     FROM ( \
                       SELECT pr.revision_id, 0::SMALLINT AS activity_kind, \
                              pr.revision_type = 'create' AS is_new_page, \
                              FALSE AS is_file, pr.created_at, pr.revision_number, \
                              pr.page_id, p.page_category_id, pc.slug AS category_slug, \
                              pr.slug, pr.title, pr.changes, pr.comments, pr.hidden, pr.user_id, \
                              wu.name AS wikidot_user_name, \
                              wu.slug AS wikidot_user_slug, \
                              local_user.name AS local_user_name \
                       FROM page_revision pr \
                       JOIN page p ON p.page_id = pr.page_id \
                                  AND p.site_id = pr.site_id \
                                  AND p.deleted_at IS NULL \
                       JOIN page_category pc ON pc.category_id = p.page_category_id \
                       LEFT JOIN wikidot_user wu \
                         ON wu.user_id = pr.user_id AND wu.is_deleted = FALSE \
                       LEFT JOIN \"user\" local_user \
                         ON local_user.user_id = pr.user_id \
                        AND local_user.deleted_at IS NULL \
                       WHERE pr.site_id = $1 \
                         AND $2 IN ('all', 'source') \
                         AND pr.revision_type NOT IN ('delete', 'undelete') \
                         AND ($2 <> 'source' OR pr.changes @> ARRAY['wikitext']::TEXT[]) \
                         AND ($3::BIGINT IS NULL OR p.page_category_id = $3) \
                       UNION ALL \
                       SELECT fr.revision_id, 1::SMALLINT AS activity_kind, \
                              FALSE AS is_new_page, TRUE AS is_file, fr.created_at, \
                              fr.revision_number, fr.page_id, p.page_category_id, \
                              pc.slug AS category_slug, current_pr.slug, current_pr.title, \
                              fr.changes, fr.comments, fr.hidden || current_pr.hidden, fr.user_id, \
                              wu.name AS wikidot_user_name, \
                              wu.slug AS wikidot_user_slug, \
                              local_user.name AS local_user_name \
                       FROM file_revision fr \
                       JOIN page p ON p.page_id = fr.page_id \
                                  AND p.site_id = fr.site_id \
                                  AND p.deleted_at IS NULL \
                       JOIN page_revision current_pr \
                         ON current_pr.revision_id = p.latest_revision_id \
                       JOIN page_category pc ON pc.category_id = p.page_category_id \
                       LEFT JOIN wikidot_user wu \
                         ON wu.user_id = fr.user_id AND wu.is_deleted = FALSE \
                       LEFT JOIN \"user\" local_user \
                         ON local_user.user_id = fr.user_id \
                        AND local_user.deleted_at IS NULL \
                       WHERE fr.site_id = $1 \
                         AND $2 IN ('all', 'files') \
                         AND ($3::BIGINT IS NULL OR p.page_category_id = $3) \
                     ) activity \
                     ORDER BY created_at DESC, activity_kind, revision_id DESC \
                     LIMIT {batch_size} OFFSET {raw_offset}",
                ),
                [
                    site_id.into(),
                    request.filter.as_str().to_owned().into(),
                    request.category_id.into(),
                ],
            ))
            .all(txn)
            .await
            .or_raise(make_error)?;
        let row_count = rows.len();
        raw_offset = raw_offset.saturating_add(row_count);

        for row in rows {
            if row.hidden.iter().any(|field| {
                ["comments", "title", "alt_title", "slug"].contains(&field.as_str())
            }) {
                continue;
            }
            let can_view = if let Some(can_view) = permission_cache.get(&row.page_id) {
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
            if can_view {
                revisions.push(row);
                if revisions.len() == visible_row_target {
                    break;
                }
            }
        }

        if row_count < batch_size {
            naturally_exhausted = true;
            break;
        }
    }

    if revisions.len() < visible_row_target && !naturally_exhausted {
        return Ok(SiteChangesLoad::Saturated);
    }

    let available_pages = revisions
        .len()
        .div_ceil(request.rows_per_page)
        .try_into()
        .unwrap_or(u32::MAX);
    let last_link_page = available_pages.min(last_candidate_page).max(request.page);
    let has_next = revisions.len()
        > usize::try_from(request.page)
            .unwrap_or(usize::MAX)
            .saturating_mul(request.rows_per_page);
    let has_dots = revisions.len()
        > usize::try_from(last_candidate_page)
            .unwrap_or(usize::MAX)
            .saturating_mul(request.rows_per_page);
    let revisions = revisions
        .into_iter()
        .skip(page_start)
        .take(request.rows_per_page)
        .collect();

    Ok(SiteChangesLoad::Complete(SiteChangesList {
        page: request.page,
        last_link_page,
        has_next,
        has_dots,
        revisions,
    }))
}

async fn load_site_changes_categories(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    viewer_user_id: Option<i64>,
) -> Result<Vec<SiteChangesCategory>> {
    let make_error = || {
        Error::new(
            format!("failed to load SiteChanges categories for site ID {site_id}"),
            ErrorType::Render,
        )
    };
    let categories = CategoryService::get_all_active(ctx, site_id)
        .await?
        .into_iter();
    let mut visible_categories = Vec::new();
    for category in categories {
        let can_view = PermissionService::check_user_can(
            ctx,
            &CheckPermissionContext {
                user_id: viewer_user_id,
                site_id,
                page_reference: None,
            },
            Permission {
                resource_type: Resource::Page,
                resource_category: Some(Reference::Id(category.category_id)),
                action: Action::View,
            },
        )
        .await
        .or_raise(make_error)?;
        if can_view {
            visible_categories.push(SiteChangesCategory {
                category_id: category.category_id,
                slug: category.slug,
            });
        }
    }
    visible_categories
        .sort_by_cached_key(|category| wikidot_category_sort_key(&category.slug));
    Ok(visible_categories)
}

impl RenderService {
    pub(crate) async fn render_wikidot_site_changes_module(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        request: WikidotSiteChangesModuleRequest,
    ) -> Result<SiteChangesLoad<WikidotSiteChangesModuleResponse>> {
        let list =
            match load_site_changes_list(ctx, site_id, ctx.request().user_id, request)
                .await?
            {
                SiteChangesLoad::Complete(list) => list,
                SiteChangesLoad::Saturated => return Ok(SiteChangesLoad::Saturated),
            };
        Ok(SiteChangesLoad::Complete(
            WikidotSiteChangesModuleResponse {
                status: "ok".to_owned(),
                body: render_site_changes_list(&list),
            },
        ))
    }
}

fn render_site_changes_snapshot(snapshot: &SiteChangesSnapshot) -> String {
    let mut output = String::from(concat!(
        r#"<div class="site-changes-box">"#,
        "\n",
        r#"<form onsubmit="return false;" action="dummy.html" method="get">"#,
        "\n<table class=\"form\">\n<tr>\n<td>Revision types:</td>\n<td>\n",
        r#"<input class="checkbox" type="checkbox" id="rev-type-all" checked="checked"/>&nbsp;ALL<br/>"#,
        "\n",
        r#"<input class="checkbox" type="checkbox" id="rev-type-new"/>&nbsp;new pages<br/>"#,
        "\n",
        r#"<input class="checkbox" type="checkbox" id="rev-type-source"/>&nbsp;source changes<br/>"#,
        "\n",
        r#"<input class="checkbox" type="checkbox" id="rev-type-title"/>&nbsp;title changes<br/>"#,
        "\n",
        r#"<input class="checkbox" type="checkbox" id="rev-type-move"/>&nbsp;page name changes<br/>"#,
        "\n",
        r#"<input class="checkbox" type="checkbox" id="rev-type-tags"/>&nbsp;tags changes<br/>"#,
        "\n",
        r#"<input class="checkbox" type="checkbox" id="rev-type-meta"/>&nbsp;metadata changes<br/>"#,
        "\n",
        r#"<input class="checkbox" type="checkbox" id="rev-type-files"/>&nbsp;files changes"#,
        "\n</td>\n</tr>\n<tr>\n<td>From categories:</td>\n<td>\n",
        r#"<select id="rev-category">"#,
        "\n",
        r#"<option value="" selected="selected">Whole site</option>"#,
        "\n",
    ));
    for category in &snapshot.categories {
        output.push_str("<option value=\"");
        output.push_str(&category.category_id.to_string());
        output.push_str("\">");
        output.push_str(&escape_list_pages_html_text(&category.slug));
        output.push_str("</option>\n");
    }
    output.push_str(concat!(
        "</select>\n</td>\n</tr>\n<tr>\n<td>Revisions per page:</td>\n<td>\n",
        r#"<select id="rev-perpage">"#,
        "\n",
        r#"<option value="10">10</option>"#,
        "\n",
        r#"<option value="20" selected="selected">20</option>"#,
        "\n",
        r#"<option value="50">50</option>"#,
        "\n",
        r#"<option value="100">100</option>"#,
        "\n",
        r#"<option value="200">200</option>"#,
        "\n</select>\n</td>\n</tr>\n</table>\n",
        r#"<div class="buttons"><input type="button" class="btn btn-default btn-sm" value="Update list" onclick="WIKIDOT.modules.SiteChangesModule.listeners.updateList(null)"/></div>"#,
        "\n</form>\n",
        r#"<div class="changes-list" id="site-changes-list">"#,
        "\n",
    ));

    output.push_str(&render_site_changes_list(&snapshot.list));
    output.push_str("\n</div>\n</div>");
    output
}

fn render_site_changes_list(list: &SiteChangesList) -> String {
    if list.revisions.is_empty() {
        return SITE_CHANGES_EMPTY.to_owned();
    }

    let mut output = String::new();
    let has_pager = list.page > 1 || list.has_next;
    if has_pager {
        push_site_changes_pager(&mut output, list);
    }
    for revision in &list.revisions {
        push_site_changes_revision(&mut output, revision);
    }
    if has_pager {
        push_site_changes_pager(&mut output, list);
    }
    output
}

fn push_site_changes_pager(output: &mut String, list: &SiteChangesList) {
    output.push_str(r#"<div class="pager"><span class="pager-no">page "#);
    output.push_str(&list.page.to_string());
    output.push_str("</span>");
    if list.page > 1 {
        push_site_changes_pager_target(output, list.page - 1, "&laquo; previous");
    }
    for page in 1..=list.last_link_page {
        if page == list.page {
            output.push_str(r#"<span class="current">"#);
            output.push_str(&page.to_string());
            output.push_str("</span>");
        } else {
            push_site_changes_pager_target(output, page, &page.to_string());
        }
    }
    if list.has_dots {
        output.push_str(r#"<span class="dots">...</span>"#);
    }
    if list.has_next {
        push_site_changes_pager_target(output, list.page + 1, "next &raquo;");
    }
    output.push_str("</div>");
}

fn push_site_changes_pager_target(output: &mut String, page: u32, label: &str) {
    output.push_str(
        r#"<span class="target"><a href="javascript:;" onclick="WIKIDOT.modules.SiteChangesModule.listeners.updateList("#,
    );
    output.push_str(&page.to_string());
    output.push_str(")\">");
    output.push_str(label);
    output.push_str("</a></span>");
}

fn push_site_changes_revision(output: &mut String, revision: &SiteChangesRevisionRow) {
    output.push_str(concat!(
        "\n",
        r#"<div class="changes-list-item"><table><tr><td class="title"><a href="/"#,
    ));
    output.push_str(&escape_list_pages_html_attr(&revision.slug));
    output.push_str("\">");
    if revision.category_slug != "_default" {
        output.push_str(&escape_list_pages_html_text(&revision.category_slug));
        output.push_str(": ");
    }
    output.push_str(&escape_list_pages_html_text(&revision.title));
    output.push_str("</a></td><td class=\"flags\">");
    push_site_changes_flags(output, revision);
    output.push_str("</td><td class=\"mod-date\"><span class=\"odate time_");
    output.push_str(&revision.created_at.unix_timestamp().to_string());
    output.push_str(" format_%25e%20%25b%20%25Y%20-%20%25H%3A%25M%3A%25S%7Cagohover\">");
    output.push_str(&format_wikidot_list_pages_date(
        revision.created_at,
        "%e %b %Y %H:%M",
    ));
    output.push_str("</span></td><td class=\"revision-no\">");
    if revision.is_new_page {
        output.push_str("(new)");
    } else {
        output.push_str("(rev. ");
        output.push_str(&revision.revision_number.to_string());
        output.push(')');
    }
    output.push_str("</td><td class=\"mod-by\">");
    push_site_changes_user(output, revision);
    output.push_str("</td></tr></table>");
    if !revision.comments.trim().is_empty() {
        output.push_str("<div class=\"comments\">");
        output.push_str(&escape_list_pages_html_text(&revision.comments));
        output.push_str("</div>");
    }
    output.push_str("</div>\n");
}

fn push_site_changes_flags(output: &mut String, revision: &SiteChangesRevisionRow) {
    if revision.is_file {
        output
            .push_str(r#"<span class="spantip" title="file/attachment action">F</span>"#);
        return;
    }
    if revision.is_new_page {
        output.push_str(r#"<span class="spantip" title="new page created">N</span>"#);
        return;
    }
    let has_change = |name: &str| revision.changes.iter().any(|change| change == name);
    if has_change("wikitext") {
        output.push_str(
            r#"<span class="spantip" title="content source text changed">S</span>"#,
        );
    }
    if has_change("title") || has_change("alt_title") {
        output.push_str(r#"<span class="spantip" title="title changed">T</span>"#);
    }
    if has_change("slug") {
        output.push_str(r#"<span class="spantip" title="page name changed">R</span>"#);
    }
    if has_change("tags") {
        output.push_str(r#"<span class="spantip" title="tags changed">A</span>"#);
    }
    if has_change("parent") {
        output.push_str(r#"<span class="spantip" title="metadata changed">M</span>"#);
    }
}

fn push_site_changes_user(output: &mut String, revision: &SiteChangesRevisionRow) {
    if let (Some(name), Some(slug)) = (
        revision
            .wikidot_user_name
            .as_deref()
            .or(revision.wikidot_user_slug.as_deref()),
        revision.wikidot_user_slug.as_deref(),
    ) {
        output.push_str("<span class=\"printuser\"><a href=\"");
        output.push_str(WIKIDOT_USER_INFO_URL);
        output.push_str(&escape_list_pages_html_attr(slug));
        output.push_str("\" onclick=\"WIKIDOT.page.listeners.userInfo(");
        output.push_str(&revision.user_id.to_string());
        output.push_str("); return false;\">");
        output.push_str(&escape_list_pages_html_text(name));
        output.push_str("</a></span>");
    } else if let Some(name) = revision
        .local_user_name
        .as_deref()
        .or(revision.wikidot_user_name.as_deref())
    {
        output.push_str("<span class=\"printuser\">");
        output.push_str(&escape_list_pages_html_text(name));
        output.push_str("</span>");
    } else {
        output.push_str(&revision.user_id.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SITE_CHANGES_MODULE_REGEX, SiteChangesRevisionRow, WikidotSiteChangesFilter,
        push_site_changes_flags,
    };
    use time::OffsetDateTime;

    fn revision(
        is_new_page: bool,
        is_file: bool,
        changes: &[&str],
    ) -> SiteChangesRevisionRow {
        SiteChangesRevisionRow {
            is_new_page,
            is_file,
            created_at: OffsetDateTime::UNIX_EPOCH,
            revision_number: 1,
            page_id: 2,
            page_category_id: 3,
            category_slug: "_default".to_owned(),
            slug: "page".to_owned(),
            title: "Page".to_owned(),
            changes: changes.iter().map(|change| (*change).to_owned()).collect(),
            comments: String::new(),
            hidden: Vec::new(),
            user_id: 4,
            wikidot_user_name: None,
            wikidot_user_slug: None,
            local_user_name: None,
        }
    }

    #[test]
    fn site_changes_accepts_only_the_evidenced_standalone_default_shape() {
        assert!(SITE_CHANGES_MODULE_REGEX.is_match("[[module SiteChanges]]"));
        assert!(SITE_CHANGES_MODULE_REGEX.is_match("  [[module SiteChanges]]  \n"));
        assert!(
            !SITE_CHANGES_MODULE_REGEX.is_match("before [[module SiteChanges]] after")
        );
        assert!(
            !SITE_CHANGES_MODULE_REGEX.is_match("[[module SiteChanges limit=\"10\"]]")
        );
    }

    #[test]
    fn site_changes_flags_derive_from_revision_authority() {
        let mut output = String::new();
        push_site_changes_flags(
            &mut output,
            &revision(
                false,
                false,
                &["wikitext", "title", "slug", "tags", "parent"],
            ),
        );
        assert!(output.contains("content source text changed\">S"));
        assert!(output.contains("title changed\">T"));
        assert!(output.contains("page name changed\">R"));
        assert!(output.contains("tags changed\">A"));
        assert!(output.contains("metadata changed\">M"));

        output.clear();
        push_site_changes_flags(&mut output, &revision(true, false, &[]));
        assert_eq!(
            output,
            r#"<span class="spantip" title="new page created">N</span>"#
        );

        output.clear();
        push_site_changes_flags(&mut output, &revision(false, true, &[]));
        assert_eq!(
            output,
            r#"<span class="spantip" title="file/attachment action">F</span>"#
        );
    }

    #[test]
    fn site_changes_options_accept_only_the_observed_read_filters() {
        assert_eq!(
            WikidotSiteChangesFilter::from_browser_options("{}"),
            Some(WikidotSiteChangesFilter::All),
        );
        assert_eq!(
            WikidotSiteChangesFilter::from_browser_options("{\"all\":true}"),
            Some(WikidotSiteChangesFilter::All),
        );
        assert_eq!(
            WikidotSiteChangesFilter::from_browser_options("{\"source\":true}"),
            Some(WikidotSiteChangesFilter::Source),
        );
        assert_eq!(
            WikidotSiteChangesFilter::from_browser_options("{\"files\":true}"),
            Some(WikidotSiteChangesFilter::Files),
        );
        for unsupported in [
            "",
            "{ \"all\": true }",
            "{\"all\":false}",
            "{\"source\":true,\"files\":true}",
            "{\"unknown\":true}",
        ] {
            assert_eq!(
                WikidotSiteChangesFilter::from_browser_options(unsupported),
                None,
            );
        }
        assert_eq!(
            WikidotSiteChangesFilter::from_wikidot_py_options("{\"all\":true}"),
            Some(WikidotSiteChangesFilter::All),
        );
        assert_eq!(
            WikidotSiteChangesFilter::from_wikidot_py_options("{\"source\":true}"),
            Some(WikidotSiteChangesFilter::Source),
        );
        assert_eq!(
            WikidotSiteChangesFilter::from_wikidot_py_options("{\"files\":true}"),
            Some(WikidotSiteChangesFilter::Files),
        );
    }
}
