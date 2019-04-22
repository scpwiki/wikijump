/*
 * render/html/mod.rs
 *
 * wikidot-html - Convert Wikidot code to HTML
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

mod line;
mod word;

mod prelude {
    pub use crate::{Error, Result, SyntaxTree};
    pub use crate::parse::{Line, Word};
    pub use htmlescape::{encode_attribute_w as escape_attr, encode_minimal_w as escape_html};
    pub use std::fmt::{self, Display, Write};
    pub use super::line::{render_line, render_lines};
    pub use super::word::{render_word, render_words};
    pub use super::super::Render;
}

use self::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HtmlRender;

impl Render for HtmlRender {
    type Output = String;

    fn render(_tree: &SyntaxTree) -> Result<String> {
        Err(Error::StaticMsg("Not implemented yet"))
    }
}
