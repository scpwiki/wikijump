/*
 * render/text/mod.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2022 Wikijump Team
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
use crate::data::PageInfo;
use crate::render::{Handle, Render};
use crate::settings::WikitextSettings;
use crate::tree::{BibliographyList, Element, SyntaxTree};

#[derive(Debug)]
pub struct TextRender;

impl TextRender {
    #[inline]
    pub fn render_partial(
        &self,
        elements: &[Element],
        page_info: &PageInfo,
        settings: &WikitextSettings,
        wikitext_len: usize,
    ) -> String {
        self.render_partial_direct(
            elements,
            page_info,
            settings,
            &[],
            &[],
            &BibliographyList::new(),
            wikitext_len,
        )
    }

    fn render_partial_direct(
        &self,
        elements: &[Element],
        page_info: &PageInfo,
        settings: &WikitextSettings,
        table_of_contents: &[Element],
        footnotes: &[Vec<Element>],
        bibliographies: &BibliographyList,
        wikitext_len: usize,
    ) -> String {
        info!(
            "Rendering text (site {}, page {}, category {})",
            page_info.site.as_ref(),
            page_info.page.as_ref(),
            match &page_info.category {
                Some(category) => category.as_ref(),
                None => "_default",
            },
        );

        let mut ctx = TextContext::new(
            page_info,
            &Handle,
            settings,
            table_of_contents,
            footnotes,
            bibliographies,
        );
        render_elements(&mut ctx, elements);

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
    fn render(
        &self,
        tree: &SyntaxTree,
        page_info: &PageInfo,
        settings: &WikitextSettings,
    ) -> String {
        self.render_partial_direct(
            &tree.elements,
            page_info,
            settings,
            &tree.table_of_contents,
            &tree.footnotes,
            &tree.bibliographies,
            tree.wikitext_len,
        )
    }
}
