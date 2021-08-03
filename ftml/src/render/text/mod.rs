/*
 * render/text/mod.rs
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

mod context;
mod elements;

use self::context::TextContext;
use self::elements::render_elements;
use crate::log::prelude::*;
use crate::render::{Handle, Render};
use crate::tree::{Element, SyntaxTree};
use crate::PageInfo;

#[derive(Debug)]
pub struct TextRender;

impl TextRender {
    fn render_partial(
        &self,
        log: &Logger,
        page_info: &PageInfo,
        elements: &[Element],
    ) -> String {
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

        let mut ctx = TextContext::new(page_info, &Handle);
        render_elements(log, &mut ctx, &elements);

        // Remove leading and trailing newlines
        while ctx.buffer().starts_with('\n') {
            ctx.buffer().remove(0);
        }

        while ctx.buffer().ends_with('\n') {
            ctx.buffer().pop();
        }

        ctx.into()
    }
}

impl Render for TextRender {
    type Output = String;

    #[inline]
    fn render(&self, log: &Logger, page_info: &PageInfo, tree: &SyntaxTree) -> String {
        self.render_partial(log, page_info, &tree.elements)
    }
}
