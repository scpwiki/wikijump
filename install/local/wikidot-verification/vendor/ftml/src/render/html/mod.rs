/*
 * render/html/mod.rs
 *
 * ftml - Library to parse Wikidot text
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

#[macro_use]
mod attributes;
mod builder;
mod context;
mod element;
mod escape;
mod meta;
mod output;
mod random;
mod render;

pub use self::meta::{HtmlMeta, HtmlMetaType};
pub use self::output::HtmlOutput;

use self::context::HtmlContext;
use self::element::{render_element, render_elements};
use crate::data::PageInfo;
use crate::render::{Handle, Render};
use crate::settings::WikitextSettings;
use crate::tree::{Element, SyntaxTree};

#[derive(Debug)]
pub struct HtmlRender;

impl Render for HtmlRender {
    type Output = HtmlOutput;

    fn render(
        &self,
        tree: &SyntaxTree,
        page_info: &PageInfo,
        settings: &WikitextSettings,
    ) -> HtmlOutput {
        info!(
            "Rendering HTML (site {}, page {}, category {})",
            page_info.site.as_ref(),
            page_info.page.as_ref(),
            match &page_info.category {
                Some(category) => category.as_ref(),
                None => "_default",
            },
        );

        let mut ctx = HtmlContext::new(
            page_info,
            &Handle,
            settings,
            &tree.table_of_contents,
            &tree.footnotes,
            &tree.bibliographies,
            tree.wikitext_len,
        );

        // Crawl through elements and generate HTML
        render_contents(&mut ctx, tree);

        // Build and return HtmlOutput
        ctx.into()
    }
}

fn render_contents(ctx: &mut HtmlContext, tree: &SyntaxTree) {
    render_elements(ctx, &tree.elements);

    if tree.needs_footnote_block {
        info!("Page needs footnote but one was not manually included, adding");
        render_element(
            ctx,
            &Element::FootnoteBlock {
                title: None,
                hide: false,
            },
        );
    }
}
