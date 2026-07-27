/*
 * services/view/module_render.rs
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

//! Rendering a page body for a request that carried Wikidot URL arguments.
//!
//! A revision's stored HTML answers the page's bare URL. When the request path
//! supplies an argument a module reads, that stored HTML is an answer to a
//! different question, so the body is rendered again for this request. Nothing
//! is written back: the revision keeps the HTML it was compiled with, and the
//! link table is not touched, because a page view is a read.

use super::module_arguments::PageModuleArguments;
use crate::error::prelude::Result;
use crate::models::page::Model as PageModel;
use crate::models::page_revision::Model as PageRevisionModel;
use crate::models::site::Model as SiteModel;
use crate::services::ServiceContext;
use crate::services::blueprint::BlueprintPageService;
use crate::services::render::{
    RenderService, UrlArguments, wikitext_reads_url_arguments,
    wikitext_requires_runtime_render,
};
use crate::services::score::ScoreService;
use crate::services::settings::SettingsService;
use crate::types::PageId;
use crate::utils::{locale_for_ftml, split_category};
use ftml::data::PageInfo;
use ref_map::OptionRefMap;

/// Decides whether this request needs its own render, and performs it.
///
/// Returns `stored_body_html` unchanged when the effective source reads no URL
/// argument and has no module that always needs current runtime state.
#[allow(clippy::too_many_arguments)]
pub(super) async fn render_body_for_module_arguments(
    ctx: &ServiceContext<'_>,
    module_arguments: &PageModuleArguments,
    wikitext: &str,
    page: &PageModel,
    page_revision: &PageRevisionModel,
    site: &SiteModel,
    stored_body_html: String,
) -> Result<String> {
    let (category_slug, page_slug) = split_category(&page_revision.slug);
    let wikitext = BlueprintPageService::apply_page_template(
        ctx,
        page.site_id,
        category_slug,
        page_slug,
        wikitext.to_owned(),
    )
    .await?;
    let reads_request_arguments =
        !module_arguments.is_empty() && wikitext_reads_url_arguments(&wikitext);
    if !reads_request_arguments && !wikitext_requires_runtime_render(&wikitext) {
        return Ok(stored_body_html);
    }

    let id = PageId {
        site_id: page.site_id,
        category_id: page.page_category_id,
        page_id: page.page_id,
    };

    let (score, layout) = try_join!(
        ScoreService::score(ctx, page.page_id),
        SettingsService::get_layout(ctx, page.site_id, Some(page.page_id)),
    )?;

    let alt_title = page_revision.alt_title.ref_map(|title| title.as_str());
    let page_info = PageInfo {
        page: cow!(page_slug),
        category: cow_opt!(category_slug),
        site: cow!(&site.slug),
        title: cow!(&page_revision.title),
        alt_title: cow_opt!(alt_title),
        score,
        tags: page_revision.tags.iter().map(|tag| cow!(tag)).collect(),
        language: cow!(locale_for_ftml(&site.locale)),
    };

    let output = RenderService::render_page(
        ctx,
        wikitext,
        &page_info,
        layout,
        id,
        UrlArguments {
            tag: module_arguments.tag.as_deref(),
            page: module_arguments.page,
            category: module_arguments.category.as_deref(),
            offset: module_arguments.offset,
        },
    )
    .await?;

    Ok(output.html_output.body)
}
