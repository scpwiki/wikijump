/*
 * render/html/element/mod.rs
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

//! Module that implements HTML rendering for `Element` and its children.

mod container;

mod prelude {
    pub use super::super::context::HtmlContext;
    pub use super::render_element;
    pub use crate::tree::{Element, SyntaxTree};
}

use self::container::render_container;
use super::HtmlContext;
use crate::tree::Element;

pub fn render_element(log: &slog::Logger, ctx: &mut HtmlContext, element: &Element) {
    debug!(log, "Rendering element"; "element" => element.name());

    match element {
        Element::Container(container) => render_container(log, ctx, container),
        Element::Module(module) => ctx.handle().render_module(log, ctx, module),
        _ => todo!(),
    }
}
