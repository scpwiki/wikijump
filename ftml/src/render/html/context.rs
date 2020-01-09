/*
 * render/html/context.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019-2020 Ammon Smith
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

//! Internal state object used during rendering.

use super::prelude::*;
use super::{builder as html, HtmlBuilder, HtmlMeta, HtmlOutput};
use crate::{PageInfo, RemoteHandle, Result};
use std::fmt::{self, Debug, Write};

#[derive(Clone)]
pub struct HtmlContext<'i, 'h> {
    html: String,
    style: String,
    meta: Vec<HtmlMeta>,
    write_mode: WriteMode,
    footnotes: FootnoteContext,
    info: PageInfo<'i>,
    handle: &'h dyn RemoteHandle,
}

impl<'i, 'h> Debug for HtmlContext<'i, 'h> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("HtmlContext")
            .field("html", &self.html)
            .field("style", &self.style)
            .field("meta", &self.meta)
            .field("write_mode", &self.write_mode)
            .field("footnotes", &self.footnotes)
            .field("info", &self.info)
            .field("handle", &"RemoteHandle {{ ... }}")
            .finish()
    }
}

impl<'i, 'h> HtmlContext<'i, 'h> {
    #[inline]
    pub fn new(info: PageInfo<'i>, handle: &'h dyn RemoteHandle) -> Self {
        HtmlContext {
            html: String::new(),
            style: String::new(),
            meta: Self::initial_metadata(&info),
            write_mode: WriteMode::Html,
            footnotes: FootnoteContext::default(),
            info,
            handle,
        }
    }

    fn initial_metadata(info: &PageInfo<'i>) -> Vec<HtmlMeta> {
        // TODO: add author(s) to PageInfo and metadata
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

                    if let Some(alt_title) = info.alt_title {
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
    pub fn handle(&self) -> &'h dyn RemoteHandle {
        self.handle
    }

    #[inline]
    pub fn footnotes(&self) -> &FootnoteContext {
        &self.footnotes
    }

    #[inline]
    pub fn footnotes_mut(&mut self) -> &mut FootnoteContext {
        &mut self.footnotes
    }

    // Operations
    pub fn substitute_footnote_block(&mut self) {
        const TOKEN: &str = "\0footnote-block\0";

        assert_eq!(self.write_mode, WriteMode::Html);
        assert_eq!(self.footnotes().needs_block(), false);

        let block = if self.footnotes.has_footnotes() {
            self.footnotes.contents()
        } else {
            ""
        };

        while let Some(idx) = self.html.find(TOKEN) {
            self.html.replace_range(idx..idx + TOKEN.len(), block);
        }
    }

    // Buffer management
    pub fn buffer(&mut self) -> &mut String {
        match self.write_mode {
            WriteMode::Html => &mut self.html,
            WriteMode::FootnoteBlock => self.footnotes.buffer(),
        }
    }

    #[inline]
    pub fn add_style(&mut self, style: &str) {
        if !self.style.is_empty() {
            self.style.push('\n');
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
        html::escape(self.buffer(), s);
    }

    #[inline]
    pub fn html(&mut self) -> HtmlBuilder<'_, 'i, 'h> {
        HtmlBuilder::new(self)
    }

    pub fn write_footnote_block<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Self) -> Result<()>,
    {
        self.write_mode = WriteMode::FootnoteBlock;
        let result = f(self);
        self.write_mode = WriteMode::Html;
        result
    }
}

impl<'i, 'h> Into<HtmlOutput> for HtmlContext<'i, 'h> {
    fn into(self) -> HtmlOutput {
        HtmlOutput {
            html: self.html,
            style: self.style,
            meta: self.meta,
        }
    }
}

impl<'i, 'h> Write for HtmlContext<'i, 'h> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.buffer().write_str(s)
    }
}

// Helper structs
#[derive(Debug, Clone, PartialEq)]
pub struct FootnoteContext {
    buffer: String,
    has_block: bool,
    count: u32,
}

impl Default for FootnoteContext {
    fn default() -> Self {
        FootnoteContext {
            buffer: str!("<div class=\"title\">Footnotes</div>"),
            has_block: false,
            count: 0,
        }
    }
}

impl FootnoteContext {
    // Field access
    #[inline]
    pub fn set_block(&mut self, value: bool) {
        self.has_block = value;
    }

    #[inline]
    pub fn has_footnotes(&self) -> bool {
        self.count > 0
    }

    #[inline]
    pub fn incr(&mut self) -> u32 {
        self.count += 1;
        self.count
    }

    #[inline]
    pub fn needs_block(&self) -> bool {
        self.count > 0 && !self.has_block
    }

    #[inline]
    pub fn contents(&self) -> &str {
        &self.buffer
    }

    #[inline]
    fn buffer(&mut self) -> &mut String {
        &mut self.buffer
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum WriteMode {
    Html,
    FootnoteBlock,
}
