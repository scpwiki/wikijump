/*
 * render/html/element/definition_list.rs
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
use crate::tree::DefinitionListItem;

pub fn render_definition_list(
    log: &Logger,
    ctx: &mut HtmlContext,
    items: &[DefinitionListItem],
) {
    info!(
        log,
        "Rendering definition list";
        "items-len" => items.len(),
    );

    ctx.html().dl().contents(|ctx| {
        for DefinitionListItem { key, value } in items {
            ctx.html().dt().inner(log, key);
            ctx.html().dd().inner(log, value);
        }
    });
}
