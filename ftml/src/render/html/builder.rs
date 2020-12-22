/*
 * render/html/builder.rs
 *
 * ftml - Library to parse Wikidot code
 * Copyright (C) 2019-2020 Ammon Smith
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
use super::escape::escape_char;
use super::render::ElementRender;

macro_rules! tag_method {
    ($tag:tt) => {
        pub fn $tag(self) -> HtmlBuilderTag<'c, 'i, 'h, 'static> {
            self.tag(stringify!($tag))
        }
    };
}

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

    tag_method!(a);
    tag_method!(b);
    tag_method!(blockquote);
    tag_method!(br);
    tag_method!(code);
    tag_method!(div);
    tag_method!(hr);
    tag_method!(i);
    tag_method!(iframe);
    tag_method!(img);
    tag_method!(li);
    tag_method!(ol);
    tag_method!(p);
    tag_method!(script);
    tag_method!(span);
    tag_method!(strike);
    tag_method!(sub);
    tag_method!(sup);
    tag_method!(table);
    tag_method!(tr);
    tag_method!(tt);
    tag_method!(u);
    tag_method!(ul);

    #[inline]
    pub fn text(&mut self, text: &str) {
        self.ctx.push_escaped(text);
    }
}

// Helper structs

#[derive(Debug)]
pub struct HtmlBuilderTag<'c, 'i, 'h, 't> {
    ctx: &'c mut HtmlContext<'i, 'h>,
    tag: &'t str,
    in_tag: bool,
}

impl<'c, 'i, 'h, 't> HtmlBuilderTag<'c, 'i, 'h, 't> {
    pub fn new(ctx: &'c mut HtmlContext<'i, 'h>, tag: &'t str) -> Self {
        ctx.push_raw('<');
        ctx.push_raw_str(tag);

        HtmlBuilderTag {
            ctx,
            tag,
            in_tag: true,
        }
    }

    fn attr_key(&mut self, key: &str) {
        debug_assert!(is_alphanumeric(key));
        debug_assert!(self.in_tag);

        self.ctx.push_raw(' ');
        self.ctx.push_escaped(key);
        self.ctx.push_raw('=');
    }

    pub fn attr(&mut self, key: &str, value_parts: &[&str]) -> &mut Self {
        self.attr_key(key);
        self.ctx.push_raw('"');
        for part in value_parts {
            self.ctx.push_escaped(part);
        }
        self.ctx.push_raw('"');

        self
    }

    pub fn attr_fmt<F>(&mut self, key: &str, mut value_fn: F) -> &mut Self
    where
        F: FnMut(&mut HtmlContext),
    {
        self.attr_key(key);
        self.ctx.push_raw('"');

        // Read the formatted text and escape it.
        // Assumes all escaped characters are ASCII (see html::escape).
        // Also assumes all changes to buffer were appended only.
        let mut index = self.ctx.buffer().len();
        value_fn(self.ctx);

        let buffer = self.ctx.buffer();
        while index < buffer.len() {
            let ch = {
                let remainder = &buffer[index..];
                remainder
                    .chars()
                    .next()
                    .expect("Character buffer exhausted")
            };

            if let Some(subst) = escape_char(ch) {
                buffer.replace_range(index..=index, subst);
            }

            index += ch.len_utf8();
        }
        self.ctx.push_raw('"');

        self
    }

    fn content_start(&mut self) {
        if self.in_tag {
            self.ctx.push_raw('>');
            self.in_tag = false;
        }
    }

    #[inline]
    pub fn inner(&mut self, component: &dyn ElementRender) -> &mut Self {
        self.content_start();
        component.render(self.ctx);

        self
    }

    pub fn contents<F>(&mut self, mut f: F) -> &mut Self
    where
        F: FnMut(&mut HtmlContext),
    {
        self.content_start();
        f(self.ctx);

        self
    }
}

impl<'c, 'i, 'h, 't> Drop for HtmlBuilderTag<'c, 'i, 'h, 't> {
    fn drop(&mut self) {
        if !self.in_tag {
            self.ctx.push_raw_str("</");
            self.ctx.push_raw_str(self.tag);
        }

        self.ctx.push_raw('>');
    }
}

// Helpers

fn is_alphanumeric(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '-')
}
