/*
 * render/html/element/tabs.rs
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
use crate::tree::Tab;
use std::iter;

pub fn render_tabview(ctx: &mut HtmlContext, tabs: &[Tab]) {
    debug!("Rendering tabview (tabs {})", tabs.len());

    // Generate IDs for each tab
    let button_ids = generate_ids(ctx.random(), tabs.len());
    let tab_ids = generate_ids(ctx.random(), tabs.len());

    // Entire tab view
    ctx.html()
        .element("wj-tabs")
        .attr(attr!(
            "class" => "wj-tabs",
        ))
        .inner(|ctx| {
            // Tab buttons
            ctx.html()
                .div()
                .attr(attr!(
                    "class" => "wj-tabs-button-list",
                    "role" => "tablist",
                ))
                .inner(|ctx| {
                    for (i, tab) in tabs.iter().enumerate() {
                        let (tab_selected, tab_index) = if i == 0 {
                            ("true", "0")
                        } else {
                            ("false", "-1")
                        };

                        // Each tab button
                        ctx.html()
                            .element("wj-tabs-button")
                            .attr(attr!(
                                "class" => "wj-tabs-button",
                                "id" => &button_ids[i],
                                "role" => "tab",
                                "aria-label" => &tab.label,
                                "aria-selected" => tab_selected,
                                "aria-controls" => &tab_ids[i],
                                "tabindex" => tab_index,
                            ))
                            .contents(&tab.label);
                    }
                });

            // Tab panels
            ctx.html()
                .div()
                .attr(attr!(
                    "class" => "wj-tabs-panel-list",
                ))
                .inner(|ctx| {
                    for (i, tab) in tabs.iter().enumerate() {
                        // Each tab panel
                        ctx.html()
                            .div()
                            .attr(attr!(
                                "class" => "wj-tabs-panel",
                                "id" => &tab_ids[i],
                                "role" => "tabpanel",
                                "aria-labelledby" => &button_ids[i],
                                "tabindex" => "0",
                                "hidden"; if i > 0,
                            ))
                            .contents(&tab.elements);
                    }
                });
        });
}

fn generate_ids(random: &mut Random, len: usize) -> Vec<String> {
    iter::repeat_n((), len)
        .map(|_| random.generate_html_id())
        .collect()
}
