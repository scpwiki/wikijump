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
use crate::render::Handle;
use crate::url::is_url;
use crate::{info, Backlinks, PageInfo};
use std::borrow::Cow;
use std::fmt::{self, Write};
use std::num::NonZeroUsize;

#[derive(Debug)]
pub struct HtmlContext<'i, 'h> {
    body: String,
    styles: Vec<String>,
    meta: Vec<HtmlMeta>,
    backlinks: Backlinks<'static>,
    info: &'i PageInfo<'i>,
    handle: &'h Handle,

    // Other fields to track
    code_snippet_index: NonZeroUsize,
}

impl<'i, 'h> HtmlContext<'i, 'h> {
    #[inline]
    pub fn new(info: &'i PageInfo<'i>, handle: &'h Handle) -> Self {
        HtmlContext {
            body: String::new(),
            styles: Vec::new(),
            meta: Self::initial_metadata(info),
            backlinks: Backlinks::new(),
            info,
            handle,
            code_snippet_index: NonZeroUsize::new(1).unwrap(),
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

    pub fn next_code_snippet_index(&mut self) -> NonZeroUsize {
        let index = self.code_snippet_index;
        self.code_snippet_index = NonZeroUsize::new(index.get() + 1).unwrap();
        index
    }

    // Backlinks
    #[inline]
    pub fn add_link(&mut self, link: &str) {
        // TODO: set to internal link if domain matches site
        // See https://scuttle.atlassian.net/browse/WJ-24

        let link_owned = Cow::Owned(str!(link));

        if is_url(link) {
            self.backlinks.external_links.push(link_owned);
        } else {
            self.backlinks.internal_links.push(link_owned);
        }
    }

    // TODO
    #[allow(dead_code)]
    #[inline]
    pub fn add_include(&mut self, page: &str) {
        let page_owned = Cow::Owned(str!(page));

        self.backlinks.included_pages.push(page_owned);
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
    pub fn html(&mut self) -> HtmlBuilder<'_, 'i, 'h> {
        HtmlBuilder::new(self)
    }
}

impl<'i, 'h> From<HtmlContext<'i, 'h>> for HtmlOutput {
    #[inline]
    fn from(ctx: HtmlContext<'i, 'h>) -> HtmlOutput {
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

impl<'i, 'h> Write for HtmlContext<'i, 'h> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.buffer().write_str(s)
    }
}
