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
mod escape;
mod meta;
mod output;
mod render;

pub use self::meta::{HtmlMeta, HtmlMetaType};
pub use self::output::HtmlOutput;

#[cfg(test)]
use super::prelude;

use self::context::HtmlContext;
use crate::data::PageInfo;
use crate::render::Render;
use crate::tree::{Container, Element, SyntaxTree};

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
            "slug" => page_info.slug.as_ref(),
            "category" => match &page_info.category {
                Some(category) => category.as_ref(),
                None => "_default",
            },
        );

        let mut ctx = HtmlContext::new(page_info, &());

        // Add styles
        for style in &tree.styles {
            ctx.add_style(style);
        }

        // Crawl through elements and generate HTML
        for element in &tree.elements {
            render_element(&mut ctx, element);
        }

        // Build and return HtmlOutput
        ctx.into()
    }
}

fn render_element(ctx: &mut HtmlContext, element: &Element) {
    match element {
        Element::Container(container) => render_container(ctx, container),
        _ => todo!(),
    }
}

fn render_container(ctx: &mut HtmlContext, container: &Container) {
    // Get HTML tag type for this type of container
    let tag_spec = container.ctype().html_tag();

    // Build the tag
    let mut tag = ctx.html().tag(tag_spec.tag());
    tag.attr_map(container.attributes());

    if let Some(class) = tag_spec.class() {
        tag.attr("class", &[class]);
    }

    // Add container internals
    tag.contents(|ctx| {
        for inner_element in container.elements() {
            render_element(ctx, inner_element);
        }
    });
}
