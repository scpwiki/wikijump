/*
 * render/html/mod.rs
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

mod buffer;
mod context;
mod finish;
mod line;
mod module;
mod word;

mod prelude {
    pub use crate::{Error, Result, SyntaxTree};
    pub use crate::parse::{Line, Word};
    pub use std::fmt::{self, Display, Write};
    pub use super::line::{render_line, render_lines};
    pub use super::word::{render_word, render_words};
    pub use super::super::Render;
    pub use super::HtmlContext;

    use htmlescape::{encode_attribute_w, encode_minimal_w};
    use super::buffer::StringBuf;

    pub fn escape_attr(ctx: &mut HtmlContext, attr: &str) -> Result<()> {
        let mut writer = StringBuf(&mut ctx.html);
        encode_attribute_w(attr, &mut writer)?;
        Ok(())
    }

    pub fn escape_html(ctx: &mut HtmlContext, html: &str) -> Result<()> {
        let mut writer = StringBuf(&mut ctx.html);
        encode_minimal_w(html, &mut writer)?;
        Ok(())
    }

    pub fn write_tag_arg(ctx: &mut HtmlContext, arg_name: &str, value: &str) -> Result<()> {
        write!(ctx.html, " {}", arg_name)?;
        ctx.push_str("=\"");
        escape_attr(ctx, value)?;
        ctx.push('"');

        Ok(())
    }
}

pub use self::context::{HtmlContext, HtmlOutput};

use crate::{postfilter, ArticleHandle};
use self::finish::render_finish;
use self::prelude::*;
use std::sync::Arc;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HtmlRender;

impl Render for HtmlRender {
    type Output = HtmlOutput;

    fn render(id: u64, handle: Arc<ArticleHandle>, tree: &SyntaxTree) -> Result<HtmlOutput> {
        let mut ctx = HtmlContext::new(id, handle);
        render_lines(&mut ctx, tree.lines())?;
        render_finish(&mut ctx)?;
        postfilter(&mut ctx.html)?;

        Ok(ctx.into())
    }
}
