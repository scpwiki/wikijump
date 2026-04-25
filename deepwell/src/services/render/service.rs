/*
 * services/render/service.rs
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

use super::prelude::*;
use crate::{
    hash::TextHash,
    services::{
        TextService,
        settings::{NavigationPageWikitext, SettingsService},
        text_block::{MIME_HTML, TextBlock, TextBlockService, mime_for_language},
    },
    types::{PageId, TextBlockType},
};
use ftml::{prelude::*, tree::CodeBlock};
use tokio::time::timeout;

#[derive(Debug)]
pub struct RenderService;

impl RenderService {
    pub async fn render(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
    ) -> Result<RenderOutput> {
        let wikitext_len = wikitext.len();
        let make_error = || {
            Error::new(
                format!(
                    "failed to run parse and render (wikitext {} bytes, info {:?}, settings {:?})",
                    wikitext_len, page_info, settings,
                ),
                ErrorType::Render,
            )
        };

        let RenderInnerOutput {
            html_output,
            errors,
            compiled_hash,
        } = Self::render_inner(ctx, wikitext, page_info, settings, None)
            .await
            .or_raise(make_error)?;

        Ok(RenderOutput {
            html_output,
            errors,
            compiled_hash,
            compiled_at: now(),
            compiled_generator: FTML_VERSION.clone(),
        })
    }

    pub async fn render_page(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        layout: Layout,
        PageId {
            site_id,
            category_id,
            page_id,
        }: PageId,
    ) -> Result<RenderPageOutput> {
        let page_settings = WikitextSettings::from_mode(WikitextMode::Page, layout);
        let nav_settings = WikitextSettings::from_mode(WikitextMode::PageNav, layout);

        let wikitext_len = wikitext.len();
        let make_error = || {
            Error::new(
                format!(
                    "failed to run parse and render for page ID {} in site ID {} (wikitext {} bytes, info {:?}, layout {})",
                    page_id,
                    site_id,
                    wikitext_len,
                    page_info,
                    layout.description(),
                ),
                ErrorType::Render,
            )
        };

        let RenderInnerOutput {
            html_output,
            errors,
            compiled_hash: compiled_body_html_hash,
        } = Self::render_inner(ctx, wikitext, page_info, &page_settings, Some(page_id))
            .await
            .or_raise(make_error)?;

        let NavigationPageWikitext {
            top_bar_page_wikitext,
            side_bar_page_wikitext,
        } = SettingsService::get_nav_page_wikitext(ctx, site_id, Some(category_id))
            .await
            .or_raise(make_error)?;

        let render_nav_page = |wikitext| async {
            match wikitext {
                Some(wikitext) => {
                    // We are providing page_id = None because that will trigger the steps
                    // to update text blocks, which is incorrect for navigation pages.
                    //
                    // Also note that the page_info for nav pages is the page being displayed,
                    // not the nav pages themselves. This means that any variables or blocks
                    // which depend on the current page (e.g. page slug, tags), which reflect
                    // the page being viewed.
                    let result =
                        Self::render_inner(ctx, wikitext, page_info, &nav_settings, None)
                            .await;

                    match result {
                        Ok(RenderInnerOutput { compiled_hash, .. }) => {
                            Ok(Some(compiled_hash))
                        }
                        Err(error) => Err(error),
                    }
                }

                // No nav page
                None => Ok(None),
            }
        };

        let (top_bar_render_result, side_bar_render_result) = join!(
            render_nav_page(top_bar_page_wikitext),
            render_nav_page(side_bar_page_wikitext),
        );
        let (compiled_top_bar_html_hash, compiled_side_bar_html_hash) =
            raise_multiple!(top_bar_render_result, side_bar_render_result; make_error);

        Ok(RenderPageOutput {
            html_output,
            errors,
            compiled_body_html_hash,
            compiled_top_bar_html_hash,
            compiled_side_bar_html_hash,
            compiled_at: now(),
            compiled_generator: FTML_VERSION.clone(),
        })
    }

    async fn render_inner(
        ctx: &ServiceContext<'_>,
        mut wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        page_id: Option<i64>,
    ) -> Result<RenderInnerOutput> {
        let config = ctx.config();

        let make_error =
            || Error::new("failed to perform render operation", ErrorType::Render);

        // We isolate the actual tasks for rendering,
        // allowing us to time it out if it takes too long.
        //
        // The preprocess step has to be distinct for borrowing reasons,
        // since we want to do the processing for non-ftml work
        // outside the timeout guards.

        let tokens = timeout(config.preprocess_timeout, async {
            // TODO include
            ftml::preprocess(&mut wikitext);
            ftml::tokenize(&wikitext)
        })
        .await
        .or_raise(|| {
            Error::new(
                "failed to preprocess and tokenize due to timeout",
                ErrorType::RenderTimeout,
            )
        })?;

        let (tree, html_output, errors) = timeout(config.render_timeout, async {
            let result = ftml::parse(&tokens, page_info, settings);
            let (tree, errors) = result.into();
            let html_output = HtmlRender.render(&tree, page_info, settings);
            (tree, html_output, errors)
        })
        .await
        .or_raise(|| {
            Error::new(
                "failed to parse and render due to timeout",
                ErrorType::RenderTimeout,
            )
        })?;

        // Insert compiled HTML into text table
        let compiled_hash = TextService::create(ctx, html_output.body.clone())
            .await
            .or_raise(make_error)?;

        // Set up the hosted text blocks
        //
        // This only applies for published pages, in any other
        // rendering context and we should skip this step.

        if let Some(page_id) = page_id {
            // It's possible to render a page without doing text blocks
            // (e.g. blueprint pages), but all cases where text blocks
            // are done are pages.
            debug_assert_eq!(settings.mode, WikitextMode::Page);

            // [[html]]
            let html_blocks: Vec<TextBlock> = tree
                .html_blocks
                .iter()
                .map(|html| TextBlock {
                    text: html,
                    text_type: None,
                    mime: MIME_HTML,
                    name: None,
                })
                .collect();

            TextBlockService::add_blocks(ctx, page_id, TextBlockType::Html, &html_blocks)
                .await
                .or_raise(make_error)?;

            // [[code]]
            let code_blocks: Vec<TextBlock> = tree
                .code_blocks
                .iter()
                .map(
                    |CodeBlock {
                         contents,
                         language,
                         name,
                     }| TextBlock {
                        text: contents,
                        text_type: language.as_deref(),
                        mime: mime_for_language(language),
                        name: name.as_deref(),
                    },
                )
                .collect();

            TextBlockService::add_blocks(ctx, page_id, TextBlockType::Code, &code_blocks)
                .await
                .or_raise(make_error)?;
        }

        // Build and return
        Ok(RenderInnerOutput {
            html_output,
            errors,
            compiled_hash,
        })
    }
}

#[derive(Debug)]
struct RenderInnerOutput {
    html_output: HtmlOutput,
    errors: Vec<ParseError>,
    compiled_hash: TextHash,
}
