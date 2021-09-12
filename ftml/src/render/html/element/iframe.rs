/*
 * render/html/element/iframe.rs
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
use crate::tree::AttributeMap;

pub fn render_iframe(
    log: &Logger,
    ctx: &mut HtmlContext,
    url: &str,
    attributes: &AttributeMap,
) {
    info!(
        log,
        "Rendering iframe block";
        "url" => url,
    );

    ctx.html().iframe().attr(attr!(
        "src" => url,
        "crossorigin";;
        attributes
    ));
}

pub fn render_html(log: &Logger, ctx: &mut HtmlContext, contents: &str) {
    info!(
        log,
        "Rendering html block (submitting to remote for iframe)";
        "contents" => contents,
    );

    // Submit HTML to be hosted on wjfiles, then get back its URL for the iframe.
    let iframe_url = ctx.handle().post_html(log, ctx.info(), contents);
    ctx.html().iframe().attr(attr!(
        "src" => &iframe_url,
        "crossorigin",
    ));
}
