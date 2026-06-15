/*
 * render/html/element/definition_list.rs
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
use crate::tree::DefinitionListItem;

pub fn render_definition_list(ctx: &mut HtmlContext, items: &[DefinitionListItem]) {
    debug!("Rendering definition list (length {})", items.len());

    ctx.html().dl().inner(|ctx| {
        for DefinitionListItem {
            key_elements,
            value_elements,
            ..
        } in items
        {
            ctx.html().dt().contents(key_elements);
            ctx.html().dd().contents(value_elements);
        }
    });
}
