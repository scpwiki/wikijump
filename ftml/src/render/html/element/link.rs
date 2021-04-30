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
use crate::url::is_url;
use std::borrow::Cow;
use wikidot_normalize::normalize;

pub fn render_anchor(
    log: &Logger,
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
    log: &Logger,
    ctx: &mut HtmlContext,
    url: &str,
    label: &LinkLabel,
    target: Option<AnchorTarget>,
) {
    let handle = ctx.handle();

    // Normalize URL for href
    let normal_url = if is_url(url) {
        Cow::Borrowed(url)
    } else {
        let mut url = str!(url);
        normalize(&mut url);
        url.insert(0, '/');
        Cow::Owned(url)
    };

    // Create <a> and set attributes
    let mut tag = ctx.html().a();
    tag.attr("href", &[&normal_url]);

    if let Some(target) = target {
        tag.attr("target", &[target.html_attr()]);
    }

    // Add <a> internals, i.e. the link name
    handle.get_link_label(log, url, label, |label| {
        tag.inner(log, &label);
    });
}
