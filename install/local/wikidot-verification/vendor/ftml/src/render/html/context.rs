/*
 * render/html/context.rs
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

use super::builder::HtmlBuilder;
use super::escape::escape;
use super::meta::{HtmlMeta, HtmlMetaType};
use super::output::HtmlOutput;
use super::random::Random;
use crate::data::PageRef;
use crate::data::{Backlinks, PageInfo};
use crate::info;
use crate::layout::Layout;
use crate::next_index::{Incrementer, NextIndex, TableOfContentsIndex};
use crate::render::Handle;
use crate::settings::WikitextSettings;
use crate::tree::{
    Bibliography, BibliographyList, Element, LinkLocation, VariableScopes,
};
use crate::url::is_url;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{self, Write};
use std::num::NonZeroUsize;

#[derive(Debug)]
pub struct HtmlContext<'i, 'h, 'e, 't>
where
    'e: 't,
{
    body: String,
    meta: Vec<HtmlMeta>,
    backlinks: Backlinks<'static>,
    info: &'i PageInfo<'i>,
    handle: &'h Handle,
    settings: &'e WikitextSettings,
    random: Random,

    //
    // Included page scopes
    //
    variables: VariableScopes,

    //
    // Fields from syntax tree
    //
    table_of_contents: &'e [Element<'t>],
    footnotes: &'e [Vec<Element<'t>>],
    bibliographies: &'e BibliographyList<'t>,

    //
    // Cached data
    //
    pages_exists: HashMap<PageRef, bool>,

    //
    // Other fields to track
    //
    code_snippet_index: NonZeroUsize,
    table_of_contents_index: Incrementer,
    equation_index: NonZeroUsize,
    footnote_index: NonZeroUsize,
}

impl<'i, 'h, 'e, 't> HtmlContext<'i, 'h, 'e, 't> {
    #[inline]
    pub fn new(
        info: &'i PageInfo<'i>,
        handle: &'h Handle,
        settings: &'e WikitextSettings,
        table_of_contents: &'e [Element<'t>],
        footnotes: &'e [Vec<Element<'t>>],
        bibliographies: &'e BibliographyList<'t>,
        wikitext_len: usize,
    ) -> Self {
        // Heuristic for improving rendering performance by avoiding reallocating.
        //
        // Looking at test data, the outputted HTML byte length usually stays
        // below ~12% of the wikitext input byte length, with the greatest differences
        // being small inputs.
        let capacity = {
            let input = wikitext_len as f32;
            let output = input * 1.12;

            // Basic sanity check, if this fails
            // just return 0 to avoid weirdness.
            if output.is_finite() {
                output as usize
            } else {
                0
            }
        };

        // Build and return
        HtmlContext {
            body: String::with_capacity(capacity),
            meta: Self::initial_metadata(info, settings.layout),
            backlinks: Backlinks::new(),
            info,
            handle,
            settings,
            random: Random::default(),
            variables: VariableScopes::new(),
            table_of_contents,
            footnotes,
            bibliographies,
            pages_exists: HashMap::new(),
            code_snippet_index: NonZeroUsize::new(1).unwrap(),
            table_of_contents_index: settings.id_indexer(),
            equation_index: NonZeroUsize::new(1).unwrap(),
            footnote_index: NonZeroUsize::new(1).unwrap(),
        }
    }

    fn initial_metadata(info: &PageInfo<'i>, layout: Layout) -> Vec<HtmlMeta> {
        // Initial version, we can tune how the metadata is generated later.

        vec![
            HtmlMeta {
                tag_type: HtmlMetaType::HttpEquiv,
                name: str!("Content-Type"),
                value: str!("text/html"),
            },
            HtmlMeta {
                tag_type: HtmlMetaType::Name,
                name: str!("generator"),
                value: format!("{} {}", *info::VERSION, layout.description()),
            },
            HtmlMeta {
                tag_type: HtmlMetaType::Name,
                name: str!("description"),
                value: {
                    let mut value = str!(info.title);

                    if let Some(ref alt_title) = info.alt_title {
                        str_write!(value, " - {alt_title}");
                    }

                    value
                },
            },
            HtmlMeta {
                tag_type: HtmlMetaType::Name,
                name: str!("keywords"),
                value: info.tags.join(","),
            },
        ]
    }

    // Field access
    #[inline]
    pub fn info(&self) -> &PageInfo<'i> {
        self.info
    }

    #[inline]
    pub fn settings(&self) -> &WikitextSettings {
        self.settings
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        self.settings.layout
    }

    #[inline]
    pub fn handle(&self) -> &'h Handle {
        self.handle
    }

    #[inline]
    pub fn random(&mut self) -> &mut Random {
        &mut self.random
    }

    #[inline]
    pub fn language(&self) -> &str {
        &self.info.language
    }

    #[inline]
    pub fn variables(&self) -> &VariableScopes {
        &self.variables
    }

    #[inline]
    pub fn variables_mut(&mut self) -> &mut VariableScopes {
        &mut self.variables
    }

    #[inline]
    pub fn table_of_contents(&self) -> &'e [Element<'t>] {
        self.table_of_contents
    }

    #[inline]
    pub fn footnotes(&self) -> &'e [Vec<Element<'t>>] {
        self.footnotes
    }

    #[inline]
    pub fn get_bibliography(&self, index: usize) -> &'e Bibliography<'t> {
        self.bibliographies.get_bibliography(index)
    }

    pub fn get_bibliography_ref(
        &self,
        label: &str,
    ) -> Option<(usize, &'e [Element<'t>])> {
        self.bibliographies.get_reference(label)
    }

    pub fn next_code_snippet_index(&mut self) -> NonZeroUsize {
        let index = self.code_snippet_index;
        self.code_snippet_index = NonZeroUsize::new(index.get() + 1).unwrap();
        index
    }

    #[inline]
    pub fn next_table_of_contents_index(&mut self) -> Option<usize> {
        self.table_of_contents_index.next()
    }

    pub fn next_equation_index(&mut self) -> NonZeroUsize {
        let index = self.equation_index;
        self.equation_index = NonZeroUsize::new(index.get() + 1).unwrap();
        index
    }

    pub fn next_footnote_index(&mut self) -> NonZeroUsize {
        let index = self.footnote_index;
        self.footnote_index = NonZeroUsize::new(index.get() + 1).unwrap();
        index
    }

    #[inline]
    pub fn get_footnote(&self, index_one: NonZeroUsize) -> Option<&'e [Element<'t>]> {
        self.footnotes
            .get(usize::from(index_one) - 1)
            .map(|elements| elements.as_slice())
    }

    // Backlinks
    #[inline]
    pub fn add_link(&mut self, link: &LinkLocation) {
        // TODO: set to internal link if domain matches site
        // See https://scuttle.atlassian.net/browse/WJ-24

        match link {
            LinkLocation::Page(page) => {
                self.backlinks.internal_links.push(page.to_owned());
            }
            LinkLocation::Url(link) => {
                let mut link: &str = link;

                if link == "javascript:;" {
                    return;
                }

                // Also support [ links pointing to local pages.
                // e.g. [/scp-001 SCP-001] in addition to [[[SCP-001]]].
                if link.starts_with('/') {
                    link = &link[1..];
                }

                if is_url(link) {
                    let link = Cow::Owned(str!(link));
                    self.backlinks.external_links.push(link);
                } else {
                    let page_ref = PageRef::page_only(cow!(link));
                    self.backlinks.internal_links.push(page_ref.to_owned());
                }
            }
        }
    }

    pub fn page_exists(&mut self, page_ref: &PageRef) -> bool {
        let (site, page, _) = page_ref.fields_or(&self.info.site);

        // Get from cache, or fetch and add
        match self.pages_exists.get(page_ref) {
            Some(exists) => *exists,
            None => {
                let exists = self.handle.get_page_exists(site, page);
                self.pages_exists.insert(page_ref.to_owned(), exists);
                exists
            }
        }
    }

    // TODO
    #[allow(dead_code)]
    #[inline]
    pub fn add_include(&mut self, page: PageRef) {
        self.backlinks.included_pages.push(page.to_owned());
    }

    // Buffer management
    #[inline]
    pub fn buffer(&mut self) -> &mut String {
        &mut self.body
    }

    #[inline]
    pub fn push_raw(&mut self, ch: char) {
        self.buffer().push(ch);
    }

    #[inline]
    pub fn push_raw_str(&mut self, s: &str) {
        self.buffer().push_str(s);
    }

    #[inline]
    pub fn push_escaped(&mut self, s: &str) {
        escape(self.buffer(), s);
    }

    #[inline]
    pub fn html(&mut self) -> HtmlBuilder<'_, 'i, 'h, 'e, 't> {
        HtmlBuilder::new(self)
    }
}

impl<'i, 'h, 'e, 't> From<HtmlContext<'i, 'h, 'e, 't>> for HtmlOutput {
    #[inline]
    fn from(ctx: HtmlContext<'i, 'h, 'e, 't>) -> HtmlOutput {
        let HtmlContext {
            body,
            meta,
            backlinks,
            ..
        } = ctx;

        HtmlOutput {
            body,
            meta,
            backlinks,
        }
    }
}

impl Write for HtmlContext<'_, '_, '_, '_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.buffer().write_str(s)
    }
}

impl NextIndex<TableOfContentsIndex> for HtmlContext<'_, '_, '_, '_> {
    #[inline]
    fn next(&mut self) -> Option<usize> {
        self.next_table_of_contents_index()
    }
}
