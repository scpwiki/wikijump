/*
 * render/html/element/table_of_contents.rs
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
use crate::tree::{Alignment, FloatAlignment};

pub fn render_table_of_contents(
    log: &Logger,
    ctx: &mut HtmlContext,
    align: Option<Alignment>,
) {
    debug!(
        log,
        "Creating table of contents";
        "align" => align.map(|a| a.name()),
    );

    let mut tag = ctx.html().div();
    tag.attr("id", &["toc"]);

    // Only valid for float left/right
    if let Some(align) = align {
        let image_align = FloatAlignment { align, float: true };

        tag.attr("class", &[image_align.html_class()]);
    }

    tag.contents(|ctx| {
        // TOC buttons
        ctx.html()
            .div()
            .attr("id", &["wj-toc-action-bar"])
            .contents(|ctx| {
                // TODO button
                ctx.html()
                    .a()
                    .attr("href", &["javascript:;"])
                    .attr("onclick", &["WIKIJUMP.page.listeners.foldToc(event)"]);
            });

        // TOC Heading
        let table_of_contents_title =
            ctx.handle()
                .get_message(log, &ctx.info().language, "table-of-contents");

        ctx.html()
            .div()
            .attr("class", &["title"])
            .inner(log, &table_of_contents_title);

        // TOC List
        // TODO

        let _x = |ctx: &mut HtmlContext| {
            ctx.html().div().contents(|ctx| {
                let toc_link = format!("toc{}", 0);

                ctx.html().a().attr("href", &[&toc_link]);
            });
        };
    });
}
