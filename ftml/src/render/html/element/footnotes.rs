/*
 * render/html/element/footnotes.rs
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

pub fn render_footnote(log: &Logger, ctx: &mut HtmlContext) {
    info!(log, "Rendering footnote reference");

    let index = ctx.next_footnote_index();

    ctx.html()
        .sup()
        .attr(attr!(
            "is" => "wj-footnote-ref",
            "class" => "wj-footnote-ref",
        ))
        .contents(|ctx| {
            let ref_id = &format!("wj-footnote-ref-{}", index);
            let content_id = &format!("wj-footnote-{}", index);

            ctx.html()
                .a()
                .attr(attr!(
                    "class" => "wj-footnote-ref",
                    "id" => ref_id,
                    "href" => "javascript:;",
                    "onclick" => "WIKIJUMP.page.utils.scrollToFootnote('" content_id "')",
                ))
                .contents(|ctx| str_write!(ctx, "{}", index));
        });
}

pub fn render_footnote_block(log: &Logger, ctx: &mut HtmlContext, title: Option<&str>) {
    info!(
        log,
        "Rendering footnote block";
        "title" => title.unwrap_or("<default>"),
    );

    let title_default;
    let title: &str = match title {
        Some(title) => title,
        None => {
            title_default =
                ctx.handle()
                    .get_message(log, ctx.language(), "footnote-block-title");
            title_default
        }
    };

    ctx.html()
        .div()
        .attr(attr!(
            "is" => "wj-footnotes-list",
            "class" => "wj-footnotes-list",
        ))
        .contents(|ctx| {
            ctx.html()
                .div()
                .attr(attr!("class" => "wj-title"))
                .inner(log, title);

            ctx.html()
                .ol()
                .contents(|ctx| {
                    let mut id = String::new();

                    for (index, contents) in ctx.footnotes().iter().enumerate() {
                        // Format ID for each footnote
                        let index = index + 1;
                        id.clear();
                        str_write!(id, "wj-footnote-{}", index);

                        // Build actual footnote item
                        ctx.html()
                            .li()
                            .attr(attr!("class" => "wj-footnote", "id" => &id))
                            .contents(|ctx| {
                                id.clear();
                                str_write!(id, "wj-footnote-ref-{}", index);

                                // Number and clickable anchor
                                let footnote_id = &id;
                                ctx.html()
                                    .a()
                                    .attr(attr!(
                                        "href" => "javascript:;",
                                        "onclick" => "WIKIJUMP.page.utils.scrollToFootnote('" footnote_id "')",
                                    ))
                                    .contents(|ctx| {
                                        str_write!(ctx, "{}", index);

                                        // Period after item number. Has special class to permit styling.
                                        ctx.html()
                                            .span()
                                            .attr(attr!("class" => "wj-footnote-sep"))
                                            .inner(log, ".");
                                    });

                                // Footnote contents
                                ctx.html()
                                    .div()
                                    .attr(attr!("class" => "wj-footnote-contents"))
                                    .inner(log, contents);
                            });
                    }
                });
        });
}
