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

use super::{ComponentRender, HtmlContext};
use crate::Result;

// Main struct

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
        debug_assert!(is_alphanumeric(tag));

        let HtmlBuilder { ctx } = self;
        HtmlBuilderTag::new(ctx, tag)
    }

    pub fn text(&mut self, text: &str) {
        // TODO add html escaping
        self.ctx.push_str(text);
    }
}

macro_rules! tag_method {
    ($tag:tt) => {
        impl<'c, 'i, 'h> HtmlBuilder<'c, 'i, 'h> {
            pub fn $tag(self) -> HtmlBuilderTag<'c, 'i, 'h, 'static> {
                self.tag(stringify!($tag))
            }
        }
    };
}

tag_method!(a);
tag_method!(b);
tag_method!(div);
tag_method!(em);
tag_method!(i);
tag_method!(iframe);
tag_method!(img);
tag_method!(script);
tag_method!(span);
tag_method!(strike);
tag_method!(strong);
tag_method!(sub);
tag_method!(sup);
tag_method!(tt);
tag_method!(u);

// Helper structs

#[derive(Debug)]
pub struct HtmlBuilderTag<'c, 'i, 'h, 't> {
    ctx: &'c mut HtmlContext<'i, 'h>,
    tag: &'t str,
    in_tag: bool,
    finished: bool,
}

impl<'c, 'i, 'h, 't> HtmlBuilderTag<'c, 'i, 'h, 't> {
    pub fn new(ctx: &'c mut HtmlContext<'i, 'h>, tag: &'t str) -> Self {
        ctx.push('<');
        ctx.push_str(tag);

        HtmlBuilderTag {
            ctx,
            tag,
            in_tag: true,
            finished: false,
        }
    }

    #[inline]
    pub fn attr(&mut self, key: &str, value_parts: &[&str]) -> Result<&mut Self> {
        debug_assert!(is_alphanumeric(key));
        debug_assert!(self.in_tag);
        debug_assert!(!self.finished);

        // TODO add html escaping
        self.ctx.push(' ');
        self.ctx.push_str(key);
        self.ctx.push('=');

        self.ctx.push('"');
        for part in value_parts {
            self.ctx.push_str(part);
        }
        self.ctx.push('"');

        Ok(self)
    }

    fn content_start(&mut self) {
        if !self.in_tag {
            self.ctx.push('>');
            self.in_tag = true;
        }
    }

    #[inline]
    pub fn inner(&mut self, component: &dyn ComponentRender) -> Result<&mut Self> {
        debug_assert!(!self.finished);
        self.content_start();

        component.render(self.ctx)?;
        Ok(self)
    }

    pub fn contents<F>(&mut self, mut f: F) -> Result<()>
    where
        F: FnMut(&mut HtmlContext) -> Result<()>,
    {
        debug_assert!(!self.finished);
        self.content_start();

        f(self.ctx)
    }

    pub fn end(&mut self) {
        debug_assert!(!self.finished);
        self.finished = true;

        if !self.in_tag {
            self.ctx.push_str("</");
            self.ctx.push_str(self.tag);
        }

        self.ctx.push('>');
    }
}

fn is_alphanumeric(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '-')
}
