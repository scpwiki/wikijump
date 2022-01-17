/*
 * services/render.rs
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
use crate::services::TextService;
use ftml::{
    self,
    data::PageInfo,
    info::VERSION,
    parsing::ParseWarning,
    render::html::{HtmlOutput, HtmlRender},
    render::Render,
    settings::WikitextSettings,
};

// Helper structs
// TODO

#[derive(Serialize, Debug)]
pub struct RenderOutput {
    #[serde(flatten)]
    html_output: HtmlOutput,
    warnings: Vec<ParseWarning>,
    compiled_hash: String,
    compiled_generator: String,
}

// Service

#[derive(Debug)]
pub struct RenderService;

impl RenderService {
    // TODO
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
        let (tree, warnings) =
            ftml::parse(ctx.slog(), &tokens, page_info, settings).into();
        let html_output = HtmlRender.render(ctx.slog(), &tree, page_info, settings);

        // Insert compiled HTML into text table
        let hash = TextService::create(ctx, html_output.body.clone()).await?;
        let compiled_hash = hex::encode(hash);

        // Build and return
        Ok(RenderOutput {
            html_output,
            warnings,
            compiled_hash,
            compiled_generator,
        })
    }
}
