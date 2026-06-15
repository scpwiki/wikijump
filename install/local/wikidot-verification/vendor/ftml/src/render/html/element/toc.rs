/*
 * render/html/element/toc.rs
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
use crate::tree::{Alignment, AttributeMap, FloatAlignment};

pub fn render_table_of_contents(
    ctx: &mut HtmlContext,
    align: Option<Alignment>,
    attributes: &AttributeMap,
) {
    debug!("Creating table of contents");
    let use_true_ids = ctx.settings().use_true_ids;

    let class_value = match align {
        None => "",
        Some(align) => {
            // Only valid for float left / right
            // TODO add wikidot compat
            FloatAlignment { align, float: true }.wj_html_class()
        }
    };

    ctx.html()
        .div()
        .attr(attr!(
            "id" => "wj-toc"; if use_true_ids,
            "class" => class_value; if align.is_some();;
            attributes
        ))
        .inner(|ctx| {
            // TOC buttons
            ctx.html()
                .div()
                .attr(attr!("id" => "wj-toc-action-bar"; if use_true_ids))
                .inner(|ctx| {
                    // TODO button
                    ctx.html().a().attr(attr!(
                        "href" => "javascript:;",
                        "onclick" => "WIKIJUMP.page.listeners.foldToc(event)",
                    ));
                });

            // TOC Heading
            let table_of_contents_title = ctx
                .handle()
                .get_message(ctx.language(), "table-of-contents");

            ctx.html()
                .div()
                .attr(attr!("class" => "title"))
                .contents(table_of_contents_title);

            // TOC List
            let table_of_contents = ctx.table_of_contents();

            ctx.html()
                .div()
                .attr(attr!("id" => "wj-toc-list"; if use_true_ids))
                .contents(table_of_contents);
        });
}
