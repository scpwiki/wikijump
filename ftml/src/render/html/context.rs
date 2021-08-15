/*
 * render/html/context.rs
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

use super::builder::HtmlBuilder;
use super::escape::escape;
use super::meta::{HtmlMeta, HtmlMetaType};
use super::output::HtmlOutput;
use crate::data::PageRef;
use crate::next_index::{NextIndex, TableOfContentsIndex};
use crate::render::Handle;
use crate::tree::{Element, LinkLocation};
use crate::url::is_url;
use crate::{info, Backlinks, PageInfo};
use std::borrow::Cow;
use std::fmt::{self, Write};
use std::num::NonZeroUsize;

#[derive(Debug)]
pub struct HtmlContext<'i, 'h, 'e, 't>
where
    'e: 't,
{
    body: String,
    styles: Vec<String>,
    meta: Vec<HtmlMeta>,
    backlinks: Backlinks<'static>,
    info: &'i PageInfo<'i>,
    handle: &'h Handle,

    // Fields from syntax tree
    table_of_contents: &'e [Element<'t>],
    footnotes: &'e [Vec<Element<'t>>],

    // Other fields to track
    code_snippet_index: NonZeroUsize,
    table_of_contents_index: usize,
}

impl<'i, 'h, 'e, 't> HtmlContext<'i, 'h, 'e, 't> {
    #[inline]
    pub fn new(
        info: &'i PageInfo<'i>,
        handle: &'h Handle,
        table_of_contents: &'e [Element<'t>],
        footnotes: &'e [Vec<Element<'t>>],
    ) -> Self {
        HtmlContext {
            body: String::new(),
            styles: Vec::new(),
            meta: Self::initial_metadata(info),
            backlinks: Backlinks::new(),
            info,
            handle,
            table_of_contents,
            footnotes,
            code_snippet_index: NonZeroUsize::new(1).unwrap(),
            table_of_contents_index: 0,
        }
    }

    fn initial_metadata(info: &PageInfo<'i>) -> Vec<HtmlMeta> {
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
                value: info::VERSION.clone(),
            },
            HtmlMeta {
                tag_type: HtmlMetaType::Name,
                name: str!("description"),
                value: {
                    let mut value = str!(info.title);

                    if let Some(ref alt_title) = info.alt_title {
                        write!(&mut value, " - {}", alt_title).unwrap();
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
    pub fn handle(&self) -> &'h Handle {
        self.handle
    }

    #[inline]
    pub fn language(&self) -> &str {
        &self.info.language
    }

    #[inline]
    pub fn table_of_contents(&self) -> &'e [Element<'t>] {
        self.table_of_contents
    }

    #[inline]
    pub fn footnotes(&self) -> &'e [Vec<Element<'t>>] {
        self.footnotes
    }

    pub fn next_code_snippet_index(&mut self) -> NonZeroUsize {
        let index = self.code_snippet_index;
        self.code_snippet_index = NonZeroUsize::new(index.get() + 1).unwrap();
        index
    }

    pub fn next_table_of_contents_index(&mut self) -> usize {
        let index = self.table_of_contents_index;
        self.table_of_contents_index += 1;
        index
    }

    // Backlinks
    #[inline]
    pub fn add_link(&mut self, link: &LinkLocation) {
        // TODO: set to internal link if domain matches site
        // See https://scuttle.atlassian.net/browse/WJ-24

        match link {
            LinkLocation::Page(page) => {
                self.backlinks.included_pages.push(page.to_owned());
            }
            LinkLocation::Url(link) => {
                let mut link: &str = &link;

                if link == "javascript:;" {
                    return;
                }

                // Also support [ links pointing to local pages.
                // e.g. [/scp-001 SCP-001] in addition to [[[SCP-001]]].
                if link.starts_with('/') {
                    link = &link[1..];
                }

                if is_url(link) {
                    let page_ref = PageRef::page_only(cow!(link));
                    self.backlinks.included_pages.push(page_ref.to_owned());
                } else {
                    let link = Cow::Owned(str!(link));
                    self.backlinks.external_links.push(link);
                }
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
    pub fn add_style(&mut self, style: String) {
        self.styles.push(style);
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
            styles,
            meta,
            backlinks,
            ..
        } = ctx;

        HtmlOutput {
            body,
            styles,
            meta,
            backlinks,
        }
    }
}

impl<'i, 'h, 'e, 't> Write for HtmlContext<'i, 'h, 'e, 't> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.buffer().write_str(s)
    }
}

impl<'i, 'h, 'e, 't> NextIndex<TableOfContentsIndex> for HtmlContext<'i, 'h, 'e, 't> {
    #[inline]
    fn next(&mut self) -> usize {
        self.next_table_of_contents_index()
    }
}
