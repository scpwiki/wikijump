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

use crate::Result;
use std::collections::HashSet;
use std::mem;

#[derive(Debug, Clone, Default)]
pub struct HtmlOutput {
    pub html: String,
    pub styles: Vec<String>,
}

#[derive(Debug)]
pub struct HtmlContext {
    pub html: String,
    pub styles: Vec<String>,
    pub has_footnotes: bool,
    pub has_footnote_block: bool,
    tags: Option<HashSet<String>>,
}

impl HtmlContext {
    pub fn new() -> Self {
        HtmlContext {
            html: String::new(),
            styles: Vec::new(),
            has_footnotes: false,
            has_footnote_block: false,
            tags: None,
        }
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
    pub fn get_title(&mut self) -> Result<&str> {
        // TODO fetch title
        Ok("Article")
    }

    pub fn get_rating(&mut self) -> Result<Option<i32>> {
        // TODO fetch rating
        Ok(None)
    }

    pub fn get_tags(&mut self) -> Result<&HashSet<String>> {
        match self.tags {
            Some(ref tags) => Ok(tags),
            None => {
                // TODO fetch tags
                let tags = HashSet::new();

                mem::replace(&mut self.tags, Some(tags));

                Ok(self.tags.as_ref().unwrap())
            }
        }
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
