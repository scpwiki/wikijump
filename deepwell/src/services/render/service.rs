/*
 * services/render/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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
use tokio::time::timeout;

#[derive(Debug)]
pub struct RenderService;

impl RenderService {
    pub async fn render(
        ctx: &ServiceContext,
        mut wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
    ) -> Result<RenderOutput> {
        let compiled_generator = FTML_VERSION.clone();

        // Isolate the actual render task.
        // This way we can cut it off if it times out.

        let config = ctx.config();
        let (html_output, errors) = timeout(config.render_timeout, async {
            // Run ftml to parse and render
            // TODO include
            ftml::preprocess(&mut wikitext);
            let tokens = ftml::tokenize(&wikitext);
            let result = ftml::parse(&tokens, page_info, settings);
            let (tree, errors) = result.into();
            let html_output = HtmlRender.render(&tree, page_info, settings);
            (html_output, errors)
        })
        .await
        // Not using Error::from() because timeouts could occur in other places,
        // and this error variant is not specific to all timeouts.
        .map_err(|_| Error::RenderTimeout)?;

        // Insert compiled HTML into text table
        let compiled_hash = TextService::create(ctx, html_output.body.clone()).await?;

        // Build and return
        Ok(RenderOutput {
            html_output,
            errors,
            compiled_hash,
            compiled_at: now(),
            compiled_generator,
        })
    }
}
