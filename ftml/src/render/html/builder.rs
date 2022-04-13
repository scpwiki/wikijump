/*
 * render/html/builder.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2022 Wikijump Team
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

use super::attributes::AddedAttributes;
use super::context::HtmlContext;
use super::render::ItemRender;
use crate::id_prefix::isolate_ids;
use std::collections::HashSet;

macro_rules! tag_method {
    ($tag:tt) => {
        pub fn $tag(self) -> HtmlBuilderTag<'c, 'i, 'h, 'e, 't> {
            self.tag(stringify!($tag))
        }
    };
}

/// These are HTML tags which do not need a closing pair.
const SOLO_HTML_TAGS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
    "source", "track", "wbr",
];

// Main struct

#[derive(Debug)]
pub struct HtmlBuilder<'c, 'i, 'h, 'e, 't>
where
    'e: 't,
{
    ctx: &'c mut HtmlContext<'i, 'h, 'e, 't>,
}

impl<'c, 'i, 'h, 'e, 't> HtmlBuilder<'c, 'i, 'h, 'e, 't>
where
    'e: 't,
{
    #[inline]
    pub fn new(ctx: &'c mut HtmlContext<'i, 'h, 'e, 't>) -> Self {
        HtmlBuilder { ctx }
    }

    /// Create a new HTML element with the given tag type.
    #[inline]
    pub fn tag(self, tag: &'t str) -> HtmlBuilderTag<'c, 'i, 'h, 'e, 't> {
        debug_assert!(is_alphanumeric(tag));

        let HtmlBuilder { ctx } = self;
        HtmlBuilderTag::new(ctx, tag)
    }

    /// Create a new custom element. Tag must start with `wj-`.
    #[inline]
    pub fn element(self, tag: &'t str) -> HtmlBuilderTag<'c, 'i, 'h, 'e, 't> {
        debug_assert!(tag.starts_with("wj-"));

        self.tag(tag)
    }

    #[inline]
    pub fn table_cell(self, header: bool) -> HtmlBuilderTag<'c, 'i, 'h, 'e, 't> {
        if header {
            self.tag("th")
        } else {
            self.tag("td")
        }
    }

    /// Creates an inline `<svg>` using the `ui.svg` spritesheet.
    pub fn sprite(self, id: &'t str) {
        let viewbox = match id {
            "wj-karma" => "0 0 64 114",
            _ => "0 0 24 24",
        };

        let class = format!("wj-sprite sprite-{id}");
        let href = format!("/files--static/media/ui.svg#{id}");

        self.tag("svg")
            .attr(attr!(
                "class" => &class,
                "viewBox" => viewbox,
            ))
            .contents(|ctx| {
                ctx.html().tag("use").attr(attr!("href" => &href));
            });
    }

    tag_method!(a);
    tag_method!(br);
    tag_method!(code);
    tag_method!(dd);
    tag_method!(details);
    tag_method!(div);
    tag_method!(dl);
    tag_method!(dt);
    tag_method!(hr);
    tag_method!(iframe);
    tag_method!(img);
    tag_method!(input);
    tag_method!(li);
    tag_method!(ol);
    tag_method!(pre);
    tag_method!(script);
    tag_method!(span);
    tag_method!(sub);
    tag_method!(sup);
    tag_method!(summary);
    tag_method!(table);
    tag_method!(tbody);
    tag_method!(tr);
    tag_method!(ul);

    #[inline]
    pub fn text(&mut self, text: &str) {
        self.ctx.push_escaped(text);
    }
}

// Helper structs

#[derive(Debug)]
pub struct HtmlBuilderTag<'c, 'i, 'h, 'e, 't>
where
    'e: 't,
{
    ctx: &'c mut HtmlContext<'i, 'h, 'e, 't>,
    tag: &'t str,
    in_tag: bool,
    in_contents: bool,
}

impl<'c, 'i, 'h, 'e, 't> HtmlBuilderTag<'c, 'i, 'h, 'e, 't> {
    pub fn new(ctx: &'c mut HtmlContext<'i, 'h, 'e, 't>, tag: &'t str) -> Self {
        ctx.push_raw('<');
        ctx.push_raw_str(tag);

        HtmlBuilderTag {
            ctx,
            tag,
            in_tag: true,
            in_contents: false,
        }
    }

    fn attr_key(&mut self, key: &str, has_value: bool) {
        debug_assert!(is_alphanumeric(key));
        debug_assert!(self.in_tag);

        self.ctx.push_raw(' ');
        self.ctx.push_escaped(key);

        if has_value {
            self.ctx.push_raw('=');
        }
    }

    pub fn attr_single(&mut self, key: &str, value_parts: &[&str]) -> &mut Self {
        // If value_parts is empty, then we just give the key.
        //
        // For instance, ("checked", &[]) in input produces
        // <input checked> rather than <input checked="...">
        //
        // Alternatively, if it's only composed of empty strings,
        // the same intent is signalled.
        //
        // Because .all() is true for empty slices, this expression
        // checks both:

        let has_value = !value_parts.iter().all(|s| s.is_empty());

        self.attr_key(key, has_value);

        if has_value {
            self.ctx.push_raw('"');
            for part in value_parts {
                self.ctx.push_escaped(part);
            }
            self.ctx.push_raw('"');
        }

        self
    }

    pub fn attr(&mut self, attributes: AddedAttributes) -> &mut Self {
        fn filter_entries<'a>(
            attributes: &AddedAttributes<'a>,
        ) -> impl Iterator<Item = (&'a str, &'a [&'a str])> {
            attributes.entries.iter().filter_map(
                |(item, accept)| {
                    if *accept {
                        Some(*item)
                    } else {
                        None
                    }
                },
            )
        }

        let mut merged = HashSet::new();
        let mut merged_value = Vec::new();

        // Merge any attributes in common.
        if let Some(attribute_map) = attributes.map {
            let attribute_map = attribute_map.get();

            for (key, value_parts) in filter_entries(&attributes) {
                if let Some(map_value) = attribute_map.get(&cow!(key)) {
                    // Merge keys by prepending value_parts before
                    // the attribute map value.

                    merged_value.clear();
                    merged_value.extend(value_parts);
                    merged_value.push(" ");
                    merged_value.push(map_value);

                    self.attr_single(key, &merged_value);
                    merged.insert(key);
                }
            }
        }

        // Add attributes from renderer.
        for (key, value_parts) in filter_entries(&attributes) {
            if !merged.contains(key) {
                self.attr_single(key, value_parts);
            }
        }

        // Add attributes from user-provided map.
        if let Some(attribute_map) = attributes.map {
            for (key, value) in attribute_map.get() {
                if !merged.contains(key.as_ref()) {
                    self.attr_single(key, &[value]);
                }
            }
        }

        self
    }

    fn content_start(&mut self) {
        if self.in_tag {
            self.ctx.push_raw('>');
            self.in_tag = false;
        }

        assert!(!self.in_contents, "Already in tag contents");
        self.in_contents = true;
    }

    #[inline]
    pub fn inner<R: ItemRender>(&mut self, item: R) -> &mut Self {
        self.content_start();
        item.render(self.ctx);

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

impl<'c, 'i, 'h, 'e, 't> Drop for HtmlBuilderTag<'c, 'i, 'h, 'e, 't> {
    fn drop(&mut self) {
        if self.in_tag && !self.in_contents {
            self.ctx.push_raw('>');
        }

        if should_close_tag(self.tag) {
            self.ctx.push_raw_str("</");
            self.ctx.push_raw_str(self.tag);
            self.ctx.push_raw('>');
        }
    }
}

// Helpers

fn is_alphanumeric(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '-')
}

#[inline]
fn should_close_tag(tag: &str) -> bool {
    !SOLO_HTML_TAGS.contains(&tag)
}
