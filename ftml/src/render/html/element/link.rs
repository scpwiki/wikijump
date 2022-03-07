/*
 * render/html/element/link.rs
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

use super::prelude::*;
use crate::tree::{AnchorTarget, AttributeMap, Element, LinkLabel, LinkLocation};
use crate::url::normalize_link;

pub fn render_anchor(
    ctx: &mut HtmlContext,
    elements: &[Element],
    attributes: &AttributeMap,
    target: Option<AnchorTarget>,
) {
    info!("Rendering anchor");

    let target_value = match target {
        Some(target) => target.html_attr(),
        None => "",
    };

    ctx.html()
        .a()
        .attr(attr!(
            "target" => target_value; if target.is_some();;
            attributes
        ))
        .inner(elements);
}

pub fn render_link(
    ctx: &mut HtmlContext,
    link: &LinkLocation,
    label: &LinkLabel,
    target: Option<AnchorTarget>,
) {
    info!("Rendering link '{link:?}'");
    let handle = ctx.handle();

    // Add to backlinks
    ctx.add_link(link);

    let url = normalize_link(link, ctx.handle());
    let target_value = match target {
        Some(target) => target.html_attr(),
        None => "",
    };

    let mut tag = ctx.html().a();
    tag.attr(attr!(
        "href" => &url,
        "target" => target_value; if target.is_some(),
    ));

    // Add <a> internals, i.e. the link name
    handle.get_link_label(link, label, |label| {
        tag.inner(label);
    });
}
