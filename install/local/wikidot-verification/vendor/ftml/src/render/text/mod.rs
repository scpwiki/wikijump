/*
 * render/text/mod.rs
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
        self.render_partial_direct(RenderPartial {
            elements,
            page_info,
            settings,
            table_of_contents: &[],
            footnotes: &[],
            bibliographies: &BibliographyList::new(),
            wikitext_len,
        })
    }

    fn render_partial_direct(
        &self,
        RenderPartial {
            elements,
            page_info,
            settings,
            table_of_contents,
            footnotes,
            bibliographies,
            wikitext_len,
        }: RenderPartial,
    ) -> String {
        debug!(
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
            wikitext_len,
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
        info!(
            "Rendering text (site {}, page {}, category {})",
            page_info.site.as_ref(),
            page_info.page.as_ref(),
            match &page_info.category {
                Some(category) => category.as_ref(),
                None => "_default",
            },
        );

        self.render_partial_direct(RenderPartial {
            elements: &tree.elements,
            page_info,
            settings,
            table_of_contents: &tree.table_of_contents,
            footnotes: &tree.footnotes,
            bibliographies: &tree.bibliographies,
            wikitext_len: tree.wikitext_len,
        })
    }
}

/// Helper structure to pass in values for `render_partial_direct()`.
///
/// This exists because otherwise the function would take an excessive
/// number of parameters, which Clippy dislikes.
#[derive(Debug)]
struct RenderPartial<'a> {
    elements: &'a [Element<'a>],
    page_info: &'a PageInfo<'a>,
    settings: &'a WikitextSettings,
    table_of_contents: &'a [Element<'a>],
    footnotes: &'a [Vec<Element<'a>>],
    bibliographies: &'a BibliographyList<'a>,
    wikitext_len: usize,
}
