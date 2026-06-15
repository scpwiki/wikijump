/*
 * render/html/render.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2026 Wikijump Team
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

use super::context::HtmlContext;
use super::element::{render_element, render_elements};
use crate::tree::Element;
use std::borrow::Cow;

pub trait ItemRender {
    fn render(&self, ctx: &mut HtmlContext);
}

impl ItemRender for &'_ str {
    #[inline]
    fn render(&self, ctx: &mut HtmlContext) {
        ctx.push_escaped(self);
    }
}

impl ItemRender for &'_ Cow<'_, str> {
    #[inline]
    fn render(&self, ctx: &mut HtmlContext) {
        ctx.push_escaped(self);
    }
}

impl ItemRender for String {
    #[inline]
    fn render(&self, ctx: &mut HtmlContext) {
        ctx.push_escaped(self);
    }
}

impl ItemRender for &'_ String {
    #[inline]
    fn render(&self, ctx: &mut HtmlContext) {
        ctx.push_escaped(self);
    }
}

impl ItemRender for &'_ Element<'_> {
    #[inline]
    fn render(&self, ctx: &mut HtmlContext) {
        render_element(ctx, self)
    }
}

impl ItemRender for &'_ [Element<'_>] {
    #[inline]
    fn render(&self, ctx: &mut HtmlContext) {
        render_elements(ctx, self)
    }
}

impl ItemRender for &'_ Vec<Element<'_>> {
    #[inline]
    fn render(&self, ctx: &mut HtmlContext) {
        render_elements(ctx, self)
    }
}
