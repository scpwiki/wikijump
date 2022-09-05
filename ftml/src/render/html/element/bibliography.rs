/*
 * render/html/element/bibliography.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2022 Wikijump Team
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
    info!("Rendering bibliography citation (label {label}, brackets {brackets})");

    todo!()
}

pub fn render_bibliography(
    ctx: &mut HtmlContext,
    title: Option<&str>,
    bibliography_index: usize,
    bibliography: &Bibliography,
) {
    info!(
        "Rendering bibliography block (title {}, items {})",
        title.unwrap_or("<default>"),
        bibliography.slice().len(),
    );

    let title_default;
    let title: &str = match title {
        Some(title) => title.as_ref(),
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
