/*
 * render/html/mod.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

#[cfg(test)]
mod test;

#[macro_use]
mod macros;

mod builder;
mod context;
mod element;
mod escape;
mod handle;
mod meta;
mod output;
mod render;

pub use self::meta::{HtmlMeta, HtmlMetaType};
pub use self::output::HtmlOutput;

#[cfg(test)]
use super::prelude;

use self::context::HtmlContext;
use self::element::render_elements;
use self::handle::Handle;
use crate::data::PageInfo;
use crate::render::Render;
use crate::tree::SyntaxTree;

#[derive(Debug)]
pub struct HtmlRender;

impl Render for HtmlRender {
    type Output = HtmlOutput;

    fn render(
        &self,
        log: &slog::Logger,
        page_info: &PageInfo,
        tree: &SyntaxTree,
    ) -> HtmlOutput {
        info!(
            log,
            "Rendering syntax tree";
            "target" => "html",
            "slug" => page_info.slug.as_ref(),
            "category" => match &page_info.category {
                Some(category) => category.as_ref(),
                None => "_default",
            },
        );

        let mut ctx = HtmlContext::new(page_info, &Handle);

        // Add styles
        for style in &tree.styles {
            ctx.add_style(style);
        }

        // Crawl through elements and generate HTML
        render_elements(log, &mut ctx, &tree.elements);

        // Build and return HtmlOutput
        ctx.into()
    }
}
