/*
 * render/html/element/container.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

use super::prelude::*;
use crate::tree::{Container, HtmlTag};

pub fn render_container(log: &Logger, ctx: &mut HtmlContext, container: &Container) {
    info!(log, "Rendering container"; "container" => container.ctype().name());

    // Get HTML tag type for this type of container
    let tag_spec = container.ctype().html_tag(ctx);

    // Build the tag
    let mut tag = ctx.html().tag(tag_spec.tag());

    // Merge the class attribute with the container's class, if it conflicts
    match tag_spec {
        HtmlTag::Tag(_) => tag.attr(attr!(;; container.attributes())),
        HtmlTag::TagAndClass { class, .. } => {
            tag.attr(attr!("class" => class;; container.attributes()))
        }
        HtmlTag::TagAndId { id, .. } => {
            tag.attr(attr!("id" => &id;; container.attributes()))
        }
    };

    // Add container internals
    tag.inner(log, &container.elements());
}

pub fn render_color(
    log: &Logger,
    ctx: &mut HtmlContext,
    color: &str,
    elements: &[Element],
) {
    info!(
        log,
        "Rendering color container";
        "color" => color,
        "elements-len" => elements.len(),
    );

    ctx.html()
        .span()
        .attr(attr!("style" => "color: " color ";"))
        .inner(log, &elements);
}
