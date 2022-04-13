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
use crate::tree::{
    AnchorTarget, AttributeMap, Element, LinkLabel, LinkLocation, LinkType,
};
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
            "class" => "wj-anchor",
            "target" => target_value; if target.is_some();;
            attributes,
        ))
        .inner(elements);
}

pub fn render_link(
    ctx: &mut HtmlContext,
    link: &LinkLocation,
    label: &LinkLabel,
    target: Option<AnchorTarget>,
    ltype: LinkType,
) {
    info!("Rendering link '{:?}' (type {})", link, ltype.name());
    let handle = ctx.handle();

    // Add to backlinks
    ctx.add_link(link);

    let url = normalize_link(link, ctx.handle());

    let target_value = match target {
        Some(target) => target.html_attr(),
        None => "",
    };

    let css_class = match link {
        LinkLocation::Url(url) if url.starts_with('#') => "wj-link-anchor",
        LinkLocation::Url(url) if url.starts_with('/') => "wj-link-internal",
        LinkLocation::Url(_) => "wj-link-external",
        LinkLocation::Page(_) => "wj-link-internal",
    };

    let mut tag = ctx.html().a();
    tag.attr(attr!(
        "href" => &url,
        "class" => "wj-link " css_class,
        "target" => target_value; if target.is_some(),
        "data-link-type" => ltype.name(),
    ));

    // Add <a> internals, i.e. the link name
    handle.get_link_label(link, label, |label| {
        tag.inner(label);
    });
}
