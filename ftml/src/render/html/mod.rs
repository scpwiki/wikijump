/*
 * render/html/mod.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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
mod html;
mod meta;
mod module;
mod object;
mod paragraph;
mod percent;
mod word;

mod prelude {
    pub use super::super::Render;
    use super::buffer::StringBuf;
    pub use super::paragraph::{render_paragraph, render_paragraphs};
    pub use super::percent::*;
    pub use super::word::{render_word, render_words};
    pub use super::{ComponentRender, HtmlContext, HtmlMeta};
    pub use crate::data;
    pub use crate::enums::HtmlMetaType;
    pub use crate::parse::{Paragraph, Word};
    pub use crate::{Error, PageInfo, Result, SyntaxTree};
    use htmlescape::encode_attribute_w;
    pub use std::fmt::{self, Display, Write};

    pub fn escape_attr_str(buffer: &mut String, attr: &str) -> Result<()> {
        let mut writer = StringBuf(buffer);
        encode_attribute_w(attr, &mut writer)?;
        Ok(())
    }
}

pub use self::context::HtmlContext;
pub use self::html::HtmlBuilder;
pub use self::meta::HtmlMeta;
pub use self::object::{HtmlOutput, HtmlRender};
use self::prelude::*;

pub trait ComponentRender {
    fn render(&self, ctx: &mut HtmlContext) -> Result<()>;
}

impl<'a> ComponentRender for &'a str {
    fn render(&self, ctx: &mut HtmlContext) -> Result<()> {
        ctx.push_str(self);
        Ok(())
    }
}
