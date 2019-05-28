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

#[derive(Debug, Clone, Default)]
pub struct HtmlOutput {
    pub html: String,
    pub styles: Vec<String>,
}

#[derive(Debug, Default)]
pub struct HtmlContext {
    pub html: String,
    pub styles: Vec<String>,
}

impl HtmlContext {
    #[inline]
    pub fn push(&mut self, ch: char) {
        self.html.push(ch);
    }

    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.html.push_str(s);
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
