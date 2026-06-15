/*
 * render/html/element/include.rs
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
use crate::data::PageRef;
use crate::tree::VariableMap;

pub fn render_include(
    ctx: &mut HtmlContext,
    location: &PageRef,
    variables: &VariableMap,
    elements: &[Element],
) {
    debug!("Rendering include (location {location:?})");
    ctx.variables_mut().push_scope(variables);
    render_elements(ctx, elements);
    ctx.variables_mut().pop_scope();
}

pub fn render_variable(ctx: &mut HtmlContext, name: &str) {
    let value = ctx.variables().get(name);
    debug!(
        "Rendering variable (name '{}', value '{}'",
        name,
        value.unwrap_or("<none>"),
    );

    // Write to a separate buffer since we can't borrow &mut for buffer and & for variables.
    let value = match value {
        // Value exists, substitute normally.
        Some(value) => str!(value),

        // Value is absent, leave as original value.
        // Variables are {$name}, so just write that back.
        None => format!("{{${name}}}"),
    };

    // Append the formatted string
    ctx.push_escaped(&value);
}
