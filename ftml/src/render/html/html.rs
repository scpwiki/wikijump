/*
 * render/html/html.rs
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

use super::HtmlContext;
use crate::Result;

#[derive(Debug)]
pub struct HtmlBuilder<'c, 'i, 'h> {
    ctx: &'c mut HtmlContext<'i, 'h>,
}

impl<'c, 'i, 'h> HtmlBuilder<'c, 'i, 'h> {
    #[inline]
    pub fn new(ctx: &'c mut HtmlContext<'i, 'h>) -> Self {
        HtmlBuilder { ctx }
    }

    #[inline]
    pub fn tag<'t>(self, tag: &'t str) -> HtmlBuilderTag<'c, 'i, 'h, 't> {
        debug_assert!(tag
            .chars()
            .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit()));

        let HtmlBuilder { ctx } = self;
        HtmlBuilderTag { ctx, tag }.start()
    }
}

#[derive(Debug)]
pub struct HtmlBuilderTag<'c, 'i, 'h, 't> {
    ctx: &'c mut HtmlContext<'i, 'h>,
    tag: &'t str,
}

impl<'c, 'i, 'h, 't> HtmlBuilderTag<'c, 'i, 'h, 't> {
    fn start(self) -> Self {
        self.ctx.push('<');
        self.ctx.push_str(self.tag);
        self
    }

    pub fn contents(&mut self, _: ()) -> Result<()> {
        unimplemented!()
    }

    pub fn single(self) {
        self.ctx.push('>');
    }
}
