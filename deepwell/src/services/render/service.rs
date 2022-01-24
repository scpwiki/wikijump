/*
 * services/render/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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
use crate::models::page::Model as PageModel;
use crate::services::{JobService, PageService, TextService};

#[derive(Debug)]
pub struct RenderService;

impl RenderService {
    pub async fn render(
        ctx: &ServiceContext<'_>,
        mut wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
    ) -> Result<RenderOutput> {
        let compiled_generator = VERSION.clone();

        // Run ftml to parse and render
        // TODO include
        ftml::preprocess(ctx.slog(), &mut wikitext);
        let tokens = ftml::tokenize(ctx.slog(), &wikitext);
        let result = ftml::parse(ctx.slog(), &tokens, page_info, settings);
        let (tree, warnings) = result.into();
        let html_output = HtmlRender.render(ctx.slog(), &tree, page_info, settings);

        // Insert compiled HTML into text table
        let compiled_hash = TextService::create(ctx, html_output.body.clone()).await?;

        // Build and return
        Ok(RenderOutput {
            html_output,
            warnings,
            compiled_hash,
            compiled_generator,
        })
    }

    pub async fn process_navigation(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category_slug: Option<&str>,
        page_slug: &str,
    ) -> Result<()> {
        if matches!((category_slug, page_slug), (Some("nav"), "side" | "top")) {
            // If a navigation page has been updated,
            // we need to recompile everything on that site.

            let page_ids: Vec<i64> =
                PageService::get_all(ctx, site_id, None, Some(false))
                    .await?
                    .into_iter()
                    .map(|PageModel { page_id, .. }| page_id)
                    .collect();

            JobService::enqueue_rerender_pages(ctx, &page_ids).await?;
        }

        Ok(())
    }

    pub async fn process_templates(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category_slug: Option<&str>,
        page_slug: &str,
    ) -> Result<()> {
        let category_slug = category_slug.unwrap_or("_default");
        if page_slug == "_template" {
            // If a template page has been updated,
            // we need to recompile everything in that category.

            let page_ids: Vec<i64> = PageService::get_all(
                ctx,
                site_id,
                Some(category_slug.into()),
                Some(false),
            )
            .await?
            .into_iter()
            .map(|PageModel { page_id, .. }| page_id)
            .collect();

            JobService::enqueue_rerender_pages(ctx, &page_ids).await?;
        }

        Ok(())
    }
}
