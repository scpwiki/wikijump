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
    let id = str!(index);

    // TODO make this into a locale template string
    let footnote_string = ctx.handle().get_message(log, ctx.language(), "footnote");
    let label = format!("{} {}.", footnote_string, index);

    let contents = ctx
        .get_footnote(index)
        .expect("Footnote index out of bounds from gathered footnote list");

    ctx.html()
        .span()
        .attr(attr!("class" => "wj-footnote-ref"))
        .contents(|ctx| {
            // Footnote marker that is hoverable
            ctx.html()
                .element("wj-footnote-ref-marker")
                .attr(attr!(
                    "class" => "wj-footnote-ref-marker",
                    "role" => "link",
                    "aria-label" => &label,
                    "data-id" => &id,
                ))
                .inner(log, &id);

            // Tooltip shown on hover.
            // Is aria-hidden due to difficulty in getting a simultaneous
            // tooltip and link to work. A screen reader can still navigate
            // through to the link and read the footnote directly.
            ctx.html()
                .span()
                .attr(attr!(
                    "class" => "wj-footnote-ref-tooltip",
                    "aria-hidden" => "true",
                ))
                .contents(|ctx| {
                    // Tooltip label
                    ctx.html()
                        .span()
                        .attr(attr!("class" => "wj-footnote-ref-tooltip-label"))
                        .inner(log, &label);

                    // Actual tooltip contents
                    ctx.html()
                        .span()
                        .attr(attr!("class" => "wj-footnote-ref-contents"))
                        .inner(log, contents);
                });
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
        .attr(attr!("class" => "wj-footnote-list"))
        .contents(|ctx| {
            ctx.html()
                .div()
                .attr(attr!("class" => "wj-title"))
                .inner(log, title);

            ctx.html().ol().contents(|ctx| {
                // TODO make this into a footnote helper method
                for (index, contents) in ctx.footnotes().iter().enumerate() {
                    let index = index + 1;
                    let id = &format!("{}", index);

                    // Build actual footnote item
                    ctx.html()
                        .li()
                        .attr(attr!(
                            "class" => "wj-footnote-list-item",
                            "data-id" => id,
                        ))
                        .contents(|ctx| {
                            // Number and clickable anchor
                            ctx.html()
                                .element("wj-footnote-list-item-marker")
                                .attr(attr!(
                                    "class" => "wj-footnote-list-item-marker",
                                    "type" => "button",
                                    "role" => "link",
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
                                .span()
                                .attr(attr!("class" => "wj-footnote-list-item-contents"))
                                .inner(log, contents);
                        });
                }
            });
        });
}
