/*
 * render/html/element/collapsible.rs
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
use crate::tree::{AttributeMap, Element};

#[derive(Debug, Copy, Clone)]
pub struct Collapsible<'a> {
    elements: &'a [Element<'a>],
    attributes: &'a AttributeMap<'a>,
    start_open: bool,
    show_text: Option<&'a str>,
    hide_text: Option<&'a str>,
    show_top: bool,
    show_bottom: bool,
}

impl<'a> Collapsible<'a> {
    #[inline]
    pub fn new(
        elements: &'a [Element<'a>],
        attributes: &'a AttributeMap<'a>,
        start_open: bool,
        show_text: Option<&'a str>,
        hide_text: Option<&'a str>,
        show_top: bool,
        show_bottom: bool,
    ) -> Self {
        Collapsible {
            elements,
            attributes,
            start_open,
            show_text,
            hide_text,
            show_top,
            show_bottom,
        }
    }
}

pub fn render_collapsible(log: &Logger, ctx: &mut HtmlContext, collapsible: Collapsible) {
    let Collapsible {
        elements,
        attributes,
        start_open,
        show_text,
        hide_text,
        show_top,
        show_bottom,
    } = collapsible;

    debug!(
        log,
        "Rendering collapsible";
        "elements-len" => elements.len(),
        "start-open" => start_open,
        "show-text" => show_text.unwrap_or("<default>"),
        "hide-text" => hide_text.unwrap_or("<default>"),
        "show-top" => show_top,
        "show-bottom" => show_bottom,
    );

    let show_text = show_text.unwrap_or_else(|| {
        ctx.handle()
            .get_message(log, &ctx.info().language, "collapsible-open")
    });

    let hide_text = hide_text.unwrap_or_else(|| {
        ctx.handle()
            .get_message(log, &ctx.info().language, "collapsible-hide")
    });

    fn collapsible_class(show: bool) -> &'static str {
        if show {
            "wj-collapsible-block-unfolded"
        } else {
            "wj-collapsible-block-folded"
        }
    }

    ctx.html()
        .div()
        .attr_map_prepend(attributes, ("class", "wj-collapsible-block"))
        .contents(|ctx| {
            // Open collapsible link
            ctx.html()
                .div()
                .attr("class", &[collapsible_class(!start_open)])
                .contents(|ctx| {
                    // Event-bound link to open
                    ctx.html()
                        .a()
                        .attr("class", &["wj-collapsible-block-link"])
                        .attr("href", &["javascript:;"])
                        .inner(log, &show_text);
                });

            // Close collapsible link
            ctx.html()
                .div()
                .attr("class", &[collapsible_class(start_open)])
                .contents(|ctx| {
                    // Top div to close
                    ctx.html()
                        .div()
                        .attr("class", &["wj-collapsible-block-unfolded-link"])
                        .contents(|ctx| {
                            ctx.html()
                                .a()
                                .attr(
                                    "class",
                                    &[
                                        "wj-collapsible-block-link",
                                        collapsible_class(show_top),
                                    ],
                                )
                                .attr("href", &["javascript:;"])
                                .inner(log, &hide_text);
                        });

                    // Collapsed contents
                    ctx.html()
                        .div()
                        .attr("class", &["wj-collapsible-block-content"])
                        .inner(log, &elements);

                    // Bottom div to close
                    ctx.html()
                        .div()
                        .attr("class", &["wj-collapsible-block-unfolded-link"])
                        .contents(|ctx| {
                            ctx.html()
                                .a()
                                .attr(
                                    "class",
                                    &[
                                        "wj-collapsible-block-link",
                                        collapsible_class(show_bottom),
                                    ],
                                )
                                .attr("href", &["javascript:;"])
                                .inner(log, &hide_text);
                        });
                });
        });
}
