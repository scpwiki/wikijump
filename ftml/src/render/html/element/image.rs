/*
 * render/html/element/image.rs
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
use crate::tree::{AttributeMap, FloatAlignment, ImageSource, LinkLocation};
use crate::url::normalize_link;

pub fn render_image(
    log: &Logger,
    ctx: &mut HtmlContext,
    source: &ImageSource,
    link: &Option<LinkLocation>,
    alignment: Option<FloatAlignment>,
    attributes: &AttributeMap,
) {
    debug!(
        log,
        "Rendering image element";
        "source" => source.name(),
        "link" => match link {
            Some(link) => format!("{:?}", link),
            None => str!("<none>"),
        },
        "alignment" => match alignment {
            Some(image) => image.align.name(),
            None => "<default>",
        },
        "float" => match alignment {
            Some(image) => image.float,
            None => false,
        },
    );

    let source_url = ctx.handle().get_image_link(log, ctx.info(), source);
    let (space, align_class) = match alignment {
        Some(align) => (" ", align.html_class()),
        None => ("", ""),
    };

    ctx.html()
        .div()
        .attr(attr!("class" => "wj-image-container" space align_class))
        .contents(|ctx| {
            let build_image = |ctx: &mut HtmlContext| {
                ctx.html().img().attr(attr!(
                    "class" => "wj-image",
                    "src" => &source_url,
                    "crossorigin";;
                    attributes
                ));
            };

            match link {
                Some(link) => {
                    let url = normalize_link(link, ctx.handle());
                    ctx.html()
                        .a()
                        .attr(attr!("href" => &url))
                        .contents(build_image);
                }
                None => build_image(ctx),
            };
        });
}
