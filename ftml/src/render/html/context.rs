/*
 * render/html/context.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith for Project Foundation
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

//! Module for HTML context objects.
//!
//! [`HtmlOutput`] is used externally and is the final result of rendering.
//! [`HtmlContext`] is used internally to store the internal state during rendering.
//!
//! [`HtmlOutput`]: ./HtmlOutput.html
//! [`HtmlContext`]: ./HtmlContext.html

use crate::{ArticleHandle, Result};
use std::collections::HashSet;
use std::fmt::{self, Debug};
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct HtmlOutput {
    pub html: String,
    pub styles: Vec<String>,
}

pub struct HtmlContext {
    pub html: String,
    pub styles: Vec<String>,
    pub has_footnotes: bool,
    pub has_footnote_block: bool,
    id: u64,
    handle: Arc<ArticleHandle>,
}

impl HtmlContext {
    pub fn new(id: u64, handle: Arc<ArticleHandle>) -> Self {
        HtmlContext {
            html: String::new(),
            styles: Vec::new(),
            has_footnotes: false,
            has_footnote_block: false,
            handle,
            id,
        }
    }

    // Field access
    #[inline]
    pub fn id(&self) -> u64 {
        self.id
    }

    #[inline]
    pub fn handle(&self) -> Arc<ArticleHandle> {
        Arc::clone(&self.handle)
    }

    // Buffer management
    #[inline]
    pub fn push(&mut self, ch: char) {
        self.html.push(ch);
    }

    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.html.push_str(s);
    }

    // External calls
    #[inline]
    pub fn get_title(&self) -> Result<String> {
        self.handle.get_title(self.id)
    }

    #[inline]
    pub fn get_rating(&self) -> Result<Option<i32>> {
        self.handle.get_rating(self.id)
    }

    #[inline]
    pub fn get_tags(&mut self) -> Result<HashSet<String>> {
        self.handle.get_tags(self.id)
    }
}

impl Into<HtmlOutput> for HtmlContext {
    fn into(self) -> HtmlOutput {
        HtmlOutput {
            html: self.html,
            styles: self.styles,
        }
    }
}

impl Debug for HtmlContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("HtmlContext")
            .field("html", &self.html)
            .field("styles", &self.styles)
            .field("has_footnotes", &self.has_footnotes)
            .field("has_footnote_block", &self.has_footnote_block)
            .field("id", &self.id)
            .field("handle", &"Arc<dyn ArticleHandle>")
            .finish()
    }
}
