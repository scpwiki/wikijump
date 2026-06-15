/*
 * render/html/element/link.rs
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
    debug!("Rendering anchor");

    let layout = ctx.layout();
    let target_value = match target {
        Some(target) => target.html_attr(),
        None => "",
    };

    ctx.html()
        .a()
        .attr(attr!(
            "class" => "wj-anchor"; if layout == Layout::Wikijump,
            "target" => target_value; if target.is_some();;
            attributes,
        ))
        .contents(elements);
}

pub fn render_anchor_target(ctx: &mut HtmlContext, target: &str) {
    debug!("Rendering anchor target");

    match ctx.layout() {
        Layout::Wikidot => {
            ctx.html().a().attr(attr!("name" => target));
        }
        Layout::Wikijump => {
            ctx.html().a().attr(attr!(
                "class" => "wj-anchor-target",
                "id" => target,
            ));
        }
    }
}

pub fn render_link(
    ctx: &mut HtmlContext,
    link: &LinkLocation,
    label: &LinkLabel,
    target: Option<AnchorTarget>,
    ltype: LinkType,
) {
    debug!("Rendering link '{:?}' (type {})", link, ltype.name());
    let layout = ctx.layout();
    let handle = ctx.handle();

    // Add to backlinks
    ctx.add_link(link);

    let site = ctx.info().site.as_ref().to_string();
    let url = normalize_link(link, ctx.handle());

    let target_value = match target {
        Some(target) => target.html_attr(),
        None => "",
    };

    macro_rules! write_a {
        ($attr:expr) => {{
            let mut tag = ctx.html().a();
            tag.attr($attr);
            handle.get_link_label(&site, link, label, |label| {
                // Add <a> internals, i.e. the link name
                tag.contents(label);
            });
        }};
    }

    match layout {
        Layout::Wikidot => match link {
            LinkLocation::Url(url) => {
                write_a!(attr!(
                    "href" => url,
                    "target" => target_value; if target.is_some(),
                ));
            }
            LinkLocation::Page(page) => {
                let class = if ctx.page_exists(page) {
                    "active"
                } else {
                    "newpage"
                };
                write_a!(attr!(
                    "class" => class,
                    "href" => &url,
                    "target" => target_value; if target.is_some(),
                ));
            }
        },
        Layout::Wikijump => {
            let css_class = match link {
                LinkLocation::Url(url) if url == "javascript:;" => "wj-link-anchor",
                LinkLocation::Url(url) if url.starts_with('#') => "wj-link-anchor",
                LinkLocation::Url(url) if url.starts_with('/') => "wj-link-internal",
                LinkLocation::Url(_) => "wj-link-external",
                LinkLocation::Page(page) => {
                    if ctx.page_exists(page) {
                        "wj-link-internal"
                    } else {
                        "wj-link-internal wj-link-missing"
                    }
                }
            };

            let interwiki_class = if ltype == LinkType::Interwiki {
                " wj-link-interwiki"
            } else {
                ""
            };

            write_a!(attr!(
                "class" => "wj-link " css_class interwiki_class,
                "data-link-type" => ltype.name(),
                "href" => &url,
                "target" => target_value; if target.is_some(),
            ));
        }
    }
}
