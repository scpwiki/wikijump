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
mod input;
mod link;
mod list;
mod text;

mod prelude {
    pub use super::super::context::HtmlContext;
    pub use super::render_element;
    pub use crate::tree::{Element, SyntaxTree};
}

use self::container::{render_color, render_container};
use self::input::{render_checkbox, render_radio_button};
use self::link::{render_anchor, render_link};
use self::list::render_list;
use self::text::{render_email, render_wikitext_raw};
use super::HtmlContext;
use crate::tree::Element;

pub fn render_elements(log: &slog::Logger, ctx: &mut HtmlContext, elements: &[Element]) {
    debug!(log, "Rendering elements"; "elements-len" => elements.len());

    for element in elements {
        render_element(log, ctx, element);
    }
}

pub fn render_element(log: &slog::Logger, ctx: &mut HtmlContext, element: &Element) {
    debug!(log, "Rendering element"; "element" => element.name());

    match element {
        Element::Container(container) => render_container(log, ctx, container),
        Element::Module(module) => ctx.handle().render_module(log, ctx, module),
        Element::Text(text) => ctx.push_escaped(text),
        Element::Raw(text) => render_wikitext_raw(log, ctx, text),
        Element::Email(email) => render_email(log, ctx, email),
        Element::Anchor {
            elements,
            attributes,
            target,
        } => render_anchor(log, ctx, elements, attributes, *target),
        Element::Link { url, label, target } => {
            render_link(log, ctx, &url, label, *target)
        }
        Element::List { ltype, items } => render_list(log, ctx, *ltype, items),
        Element::RadioButton {
            name,
            checked,
            attributes,
        } => render_radio_button(log, ctx, &name, *checked, attributes),
        Element::CheckBox {
            checked,
            attributes,
        } => render_checkbox(log, ctx, *checked, attributes),
        Element::Collapsible { .. } => todo!(),
        Element::Color { color, elements } => render_color(log, ctx, color, elements),
        _ => todo!(),
    }
}
