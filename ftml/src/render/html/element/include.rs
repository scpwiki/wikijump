/*
 * render/html/element/include.rs
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
use crate::data::PageRef;
use crate::tree::VariableMap;

pub fn render_include(
    log: &Logger,
    ctx: &mut HtmlContext,
    _location: &PageRef,
    variables: &VariableMap,
    elements: &[Element],
) {
    info!(
        log,
        "Rendering include";
        "location" => str!(_location),
        "variables-len" => variables.len(),
        "elements-len" => elements.len(),
    );

    ctx.variables_mut().push_scope(variables);

    render_elements(log, ctx, elements);

    ctx.variables_mut().pop_scope();
}

pub fn render_variable(log: &Logger, ctx: &mut HtmlContext, name: &str) {
    info!(log, "Rendering variable"; "name" => name);

    todo!();
}
