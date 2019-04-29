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

mod buffer;
mod line;
mod word;

mod prelude {
    pub use crate::{Error, Result, SyntaxTree};
    pub use crate::parse::{Line, Word};
    pub use std::fmt::{self, Display, Write};
    pub use super::line::{render_line, render_lines};
    pub use super::word::{render_word, render_words};
    pub use super::super::Render;
    pub use super::HtmlOutput;

    use htmlescape::{encode_attribute_w, encode_minimal_w};
    use super::buffer::StringBuf;

    pub fn escape_attr(output: &mut HtmlOutput, attr: &str) -> Result<()> {
        let mut writer = StringBuf(&mut output.html);
        encode_attribute_w(attr, &mut writer)?;
        Ok(())
    }

    pub fn escape_html(output: &mut HtmlOutput, html: &str) -> Result<()> {
        let mut writer = StringBuf(&mut output.html);
        encode_minimal_w(html, &mut writer)?;
        Ok(())
    }

    pub fn write_tag_arg(output: &mut HtmlOutput, arg_name: &str, value: &str) -> Result<()> {
        write!(output.html, " {}", arg_name)?;
        output.push_str("=\"");
        escape_attr(output, value)?;
        output.push('"');

        Ok(())
    }
}

use self::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HtmlRender;

impl Render for HtmlRender {
    type Output = HtmlOutput;

    fn render(tree: &SyntaxTree) -> Result<HtmlOutput> {
        let mut output = HtmlOutput::default();
        render_lines(&mut output, tree.lines())?;

        Ok(output)
    }
}

#[derive(Debug, Clone, Default)]
pub struct HtmlOutput {
    pub html: String,
    pub styles: Vec<String>,
}

impl HtmlOutput {
    #[inline]
    fn push(&mut self, ch: char) {
        self.html.push(ch);
    }

    #[inline]
    fn push_str(&mut self, s: &str) {
        self.html.push_str(s);
    }
}
