/*
 * services/render/backlinks.rs
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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

//! The Wikidot `Backlinks` module: which pages link to this one.
//!
//! Recognition, the row query, the anonymous-view filter, and the rendered
//! box live together here. A head carrying any argument is left literal,
//! because no argument form has a live capture behind it.

use super::compat::CompatHtmlFragments;
use super::prelude::*;
use super::service::{
    RenderService, escape_list_pages_html_attr, escape_list_pages_html_text,
};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::types::{Action, Permission, Resource};
use regex::Regex;
use sea_orm::{ConnectionTrait, FromQueryResult, Statement};
use std::sync::LazyLock;

/// The most Backlinks rows one module render will load.
pub(super) const MAX_BACKLINKS_MODULE_ROWS: usize = 500;

pub(super) static BACKLINKS_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+Backlinks(?P<head>[^\]]*)\]\]").unwrap()
});

#[derive(Debug, FromQueryResult)]
pub(in crate::services::render) struct BacklinksModulePage {
    pub(in crate::services::render) page_id: i64,
    pub(in crate::services::render) page_category_id: i64,
    pub(in crate::services::render) slug: String,
    pub(in crate::services::render) title: String,
}

pub(super) fn render_backlinks_module_box(pages: &[BacklinksModulePage]) -> String {
    let mut output = String::from(
        "\n<div class=\"backlinks-module-box\" data-wikijump-compat-backlinks=\"1\"><ul>",
    );

    for page in pages {
        output.push_str(r#"<li><a href="/"#);
        output.push_str(&escape_list_pages_html_attr(&page.slug));
        output.push_str(r#"">"#);
        output.push_str(&escape_list_pages_html_text(&page.title));
        output.push_str("</a></li>");
    }

    output.push_str("</ul></div>\n");
    output
}

impl RenderService {
    pub(super) async fn expand_backlinks_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        current_page_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !BACKLINKS_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }

        let (Some(current_site_id), Some(current_page_id)) =
            (current_site_id, current_page_id)
        else {
            return Ok(wikitext);
        };

        let mut expanded = String::with_capacity(wikitext.len());
        let mut cursor = 0;

        for captures in BACKLINKS_MODULE_REGEX.captures_iter(&wikitext) {
            let mtch = captures.get(0).unwrap();
            expanded.push_str(&wikitext[cursor..mtch.start()]);

            if Self::is_inside_wikidot_literal_region(&wikitext, mtch.start()) {
                expanded.push_str(mtch.as_str());
                cursor = mtch.end();
                continue;
            }

            let head = captures.name("head").map_or("", |mtch| mtch.as_str());
            if !head.trim().is_empty() {
                expanded.push_str(mtch.as_str());
                cursor = mtch.end();
                continue;
            }

            let pages =
                Self::load_backlinks_module_pages(ctx, current_site_id, current_page_id)
                    .await?;
            expanded
                .push_str(&compat_html.push_html(render_backlinks_module_box(&pages)));
            cursor = mtch.end();
        }

        expanded.push_str(&wikitext[cursor..]);
        Ok(expanded)
    }

    async fn load_backlinks_module_pages(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
    ) -> Result<Vec<BacklinksModulePage>> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to load Backlinks module rows for page ID {} in site ID {}",
                    current_page_id, current_site_id,
                ),
                ErrorType::Render,
            )
        };
        let txn = ctx.transaction();
        let statement = Statement::from_string(
            txn.get_database_backend(),
            format!(
                "SELECT p.page_id, p.page_category_id, p.slug, pr.title \
                 FROM page_connection pc \
                 JOIN page p ON p.page_id = pc.from_page_id \
                 JOIN page_revision pr ON pr.revision_id = p.latest_revision_id \
                 WHERE pc.to_page_id = {current_page_id} \
                   AND pc.connection_type = 'link' \
                   AND p.site_id = {current_site_id} \
                   AND p.deleted_at IS NULL \
                 ORDER BY lower(pr.title), p.slug \
                 LIMIT {MAX_BACKLINKS_MODULE_ROWS}",
            ),
        );

        let rows = BacklinksModulePage::find_by_statement(statement)
            .all(txn)
            .await
            .or_raise(make_error)?;

        let mut viewable = Vec::with_capacity(rows.len());
        for row in rows {
            let anonymously_viewable = PermissionService::check_user_can(
                ctx,
                &CheckPermissionContext {
                    user_id: None,
                    site_id: current_site_id,
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

            if anonymously_viewable {
                viewable.push(row);
            }
        }

        Ok(viewable)
    }
}
