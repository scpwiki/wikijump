/*
 * render/html/element/bibliography.rs
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
use crate::tree::Bibliography;

pub fn render_bibcite(ctx: &mut HtmlContext, label: &str, brackets: bool) {
    debug!("Rendering bibliography citation (label {label}, brackets {brackets})");

    match ctx.get_bibliography_ref(label) {
        // Valid bibliography reference, render it
        Some((index, contents)) => {
            // TODO make this into a locale template string
            let reference_string = ctx
                .handle()
                .get_message(ctx.language(), "bibliography-reference");
            let label = format!("{reference_string} {index}.");

            // TODO: For now, copied from footnotes
            ctx.html()
                .span()
                .attr(attr!("class" => "wj-bibliography-ref"))
                .inner(|ctx| {
                    let id = str!(index);

                    // Bibliography marker that is hoverable
                    if brackets {
                        ctx.push_raw('[');
                    }

                    ctx.html()
                        .element("wj-bibliography-ref-marker")
                        .attr(attr!(
                            "class" => "wj-bibliography-ref-marker",
                            "role" => "link",
                            "aria-label" => &label,
                            "data-id" => &id,
                        ))
                        .contents(&id);

                    if brackets {
                        ctx.push_raw(']');
                    }

                    // Tooltip shown on hover.
                    // Is aria-hidden due to difficulty in getting a simultaneous
                    // tooltip and link to work. A screen reader can still navigate
                    // through to the link and read the bibliography directly.
                    ctx.html()
                        .span()
                        .attr(attr!(
                            "class" => "wj-bibliography-ref-tooltip",
                            "aria-hidden" => "true",
                        ))
                        .inner(|ctx| {
                            // Tooltip label
                            ctx.html()
                                .span()
                                .attr(
                                    attr!("class" => "wj-bibliography-ref-tooltip-label"),
                                )
                                .contents(&label);

                            // Actual tooltip contents
                            ctx.html()
                                .span()
                                .attr(attr!("class" => "wj-bibliography-ref-contents"))
                                .contents(contents);
                        });
                });
        }
        None => {
            // We need to produce an error for invalid bibliography references
            let message = ctx
                .handle()
                .get_message(ctx.language(), "bibliography-cite-not-found");

            ctx.html()
                .span()
                .attr(attr!("class" => "wj-error-inline"))
                .contents(message);
        }
    }
}

pub fn render_bibliography(
    ctx: &mut HtmlContext,
    title: Option<&str>,
    bibliography_index: usize,
    bibliography: &Bibliography,
) {
    debug!(
        "Rendering bibliography block (title {}, items {})",
        title.unwrap_or("<default>"),
        bibliography.slice().len(),
    );

    let title_default;
    let title: &str = match title {
        Some(title) => title,
        None => {
            title_default = ctx
                .handle()
                .get_message(ctx.language(), "bibliography-block-title");

            title_default
        }
    };

    ctx.html()
        .div()
        .attr(attr!("class" => "wj-bibliography bibitems"))
        .inner(|ctx| {
            ctx.html()
                .div()
                .attr(attr!("class" => "wj-bibliography-title title"))
                .contents(title);

            let mut id = String::new();
            for (entry_index, (_, elements)) in bibliography.slice().iter().enumerate() {
                // Convert to 1-indexing
                let bibliography_index = bibliography_index + 1;
                let entry_index = entry_index + 1;

                // Produce HTML ID
                id.clear();
                str_write!(
                    id,
                    "wj-bibliography-item-{}-{} bibitem-{}-{}",
                    bibliography_index,
                    entry_index,
                    bibliography_index,
                    entry_index,
                );

                // Make bibliography row
                ctx.html()
                    .div()
                    .attr(attr!("class" => "wj-bibliography-item bibitem", "id" => &id))
                    .inner(|ctx| {
                        // Number and clickable anchor
                        ctx.html()
                            .element("wj-bibliography-item-marker")
                            .attr(attr!(
                                "class" => "wj-bibliography-item-marker",
                                "type" => "button",
                                "role" => "link",
                            ))
                            .inner(|ctx| {
                                str_write!(ctx, "{entry_index}");

                                // Period after entry number. Has special class to permit styling.
                                ctx.html()
                                    .span()
                                    .attr(attr!("class" => "wj-bibliography-sep"))
                                    .contents(".");
                            });

                        render_elements(ctx, elements);
                    });
            }
        });
}
