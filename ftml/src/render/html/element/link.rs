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
use crate::url::normalize_url;

pub fn render_anchor(
    log: &Logger,
    ctx: &mut HtmlContext,
    elements: &[Element],
    attributes: &AttributeMap,
    target: Option<AnchorTarget>,
) {
    debug!(
        log,
        "Rendering anchor";
        "elements-len" => elements.len(),
        "target" => target_str(target),
    );

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
    debug!(
        log,
        "Rendering link";
        "url" => url,
        "target" => target_str(target),
    );

    let handle = ctx.handle();

    // Add to backlinks
    ctx.add_link(url);

    // Create <a> and set attributes
    let mut tag = ctx.html().a();
    tag.attr("href", &[&normalize_url(url)]);

    if let Some(target) = target {
        tag.attr("target", &[target.html_attr()]);
    }

    // Add <a> internals, i.e. the link name
    handle.get_link_label(log, url, label, |label| {
        tag.inner(log, &label);
    });
}

fn target_str(target: Option<AnchorTarget>) -> &'static str {
    match target {
        Some(target) => target.name(),
        None => "<none>",
    }
}
