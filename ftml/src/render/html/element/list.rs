/*
 * render/html/element/list.rs
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
use crate::tree::{AttributeMap, ListItem, ListType};

pub fn render_list(
    log: &Logger,
    ctx: &mut HtmlContext,
    ltype: ListType,
    list_items: &[ListItem],
    attributes: &AttributeMap,
) {
    debug!(
        log,
        "Rendering list";
        "list-type" => ltype.name(),
        "list-items-len" => list_items.len(),
    );

    let list_tag = ltype.html_tag();
    let mut tag = ctx.html().tag(list_tag);

    tag.attr_map(attributes).contents(|ctx| {
        for list_item in list_items {
            match list_item {
                ListItem::Elements(elements) => {
                    ctx.html().li().inner(log, &elements.as_slice());
                }
                ListItem::SubList(list) => {
                    render_element(log, ctx, list);
                }
            }
        }
    });
}
