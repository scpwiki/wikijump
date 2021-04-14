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
use crate::data::PageInfo;
use std::fmt::{self, Write};

#[derive(Debug)]
pub struct HtmlContext<'i, 'h> {
    html: String,
    style: String,
    meta: Vec<HtmlMeta>,
    info: &'i PageInfo<'i>,
    handle: &'h (),
}

impl<'i, 'h> HtmlContext<'i, 'h> {
    #[inline]
    pub fn new(info: &'i PageInfo<'i>, handle: &'h ()) -> Self {
        HtmlContext {
            html: String::new(),
            style: String::new(),
            meta: Self::initial_metadata(&info),
            info,
            handle,
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
                value: format!("ftml {}", env!("CARGO_PKG_VERSION")),
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
        &self.info
    }

    #[inline]
    pub fn handle(&self) -> &'h () {
        self.handle
    }

    // Buffer management
    #[inline]
    pub fn buffer(&mut self) -> &mut String {
        &mut self.html
    }

    #[inline]
    pub fn add_style(&mut self, style: &str) {
        if !self.style.is_empty() {
            self.style.push_str("\n/*****/\n");
        }

        self.style.push_str(style);
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
    fn from(context: HtmlContext<'i, 'h>) -> HtmlOutput {
        let HtmlContext {
            html, style, meta, ..
        } = context;

        HtmlOutput { html, style, meta }
    }
}

impl<'i, 'h> Write for HtmlContext<'i, 'h> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.buffer().write_str(s)
    }
}
