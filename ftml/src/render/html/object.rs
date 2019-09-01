/*
 * render/html/object.rs
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

use super::finish::render_finish;
use super::prelude::*;
use crate::postfilter;
use crate::RemoteHandle;
use std::fmt::{self, Debug};
use std::rc::Rc;

pub struct HtmlRender {
    handle: Rc<dyn RemoteHandle>,
}

impl HtmlRender {
    pub fn new(handle: Rc<dyn RemoteHandle>) -> Self {
        HtmlRender { handle }
    }
}

impl Render for HtmlRender {
    type Output = HtmlOutput;

    fn render(&self, tree: &SyntaxTree, info: PageInfo) -> Result<HtmlOutput> {
        let mut ctx = HtmlContext::new(info, Rc::clone(&self.handle));
        render_paragraphs(&mut ctx, tree.paragraphs())?;
        render_finish(&mut ctx)?;
        postfilter(ctx.buffer())?;

        Ok(ctx.into())
    }
}

impl Debug for HtmlRender {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HtmlRender {{ .. }}")
    }
}

#[derive(Debug, Clone, Default)]
pub struct HtmlOutput {
    pub html: String,
    pub style: String,
    pub meta: Vec<HtmlMeta>,
}
