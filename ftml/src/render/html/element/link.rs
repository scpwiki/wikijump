/*
 * render/html/element/link.rs
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
use crate::tree::{AnchorTarget, AttributeMap, Element, LinkLabel};

pub fn render_anchor(
    log: &slog::Logger,
    ctx: &mut HtmlContext,
    elements: &[Element],
    attributes: &AttributeMap,
    target: Option<AnchorTarget>,
) {
    let mut tag = ctx.html().a();

    // Set <a> attributes
    if let Some(target) = target {
        tag.attr("target", &[target.html_attr()]);
    }
    tag.attr_map(attributes);

    // Add <a> internals
    tag.inner(log, &elements);
}

pub fn render_link(
    log: &slog::Logger,
    ctx: &mut HtmlContext,
    url: &str,
    label: &LinkLabel,
    target: Option<AnchorTarget>,
) {
    let page_title;
    let label_text = match label {
        LinkLabel::Text(ref text) => text,
        LinkLabel::Url(Some(ref text)) => text,
        LinkLabel::Url(None) => url,
        LinkLabel::Page => {
            page_title = ctx.handle().get_page_title(url);
            &page_title
        }
    };

    // Create <a> and set attributes
    let mut tag = ctx.html().a();
    tag.attr("href", &[url]);

    if let Some(target) = target {
        tag.attr("target", &[target.html_attr()]);
    }

    // Add <a> internals, i.e. the link name
    tag.inner(log, &label_text);
}
