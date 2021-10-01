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

#[cfg(test)]
use super::prelude;

use self::attributes::AddedAttributes;
use self::context::HtmlContext;
use crate::data::PageInfo;
use crate::log::prelude::*;
use crate::render::{Handle, Render};
use crate::tree::SyntaxTree;

#[derive(Debug)]
pub struct HtmlRender;

impl Render for HtmlRender {
    type Output = HtmlOutput;

    fn render(
        &self,
        log: &Logger,
        page_info: &PageInfo,
        tree: &SyntaxTree,
    ) -> HtmlOutput {
        info!(
            log,
            "Rendering syntax tree";
            "target" => "html",
            "site" => page_info.site.as_ref(),
            "page" => page_info.page.as_ref(),
            "category" => match &page_info.category {
                Some(category) => category.as_ref(),
                None => "_default",
            },
        );

        let mut ctx = HtmlContext::new(
            page_info,
            &Handle,
            &tree.table_of_contents,
            &tree.footnotes,
        );

        // Add styles
        for style in &tree.styles {
            ctx.add_style(str!(style));
        }

        // Crawl through elements and generate HTML
        ctx.html()
            .element("wj-body")
            .attr(attr!("class" => "wj-body"))
            .inner(log, &tree.elements);

        // Build and return HtmlOutput
        ctx.into()
    }
}
