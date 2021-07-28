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

mod collapsible;
mod container;
mod iframe;
mod image;
mod input;
mod link;
mod list;
mod text;
mod user;

mod prelude {
    pub use super::super::context::HtmlContext;
    pub use super::{render_element, render_elements};
    pub use crate::log::prelude::*;
    pub use crate::tree::{Element, SyntaxTree};
}

use self::collapsible::{render_collapsible, Collapsible};
use self::container::{render_color, render_container};
use self::iframe::{render_html, render_iframe};
use self::image::render_image;
use self::input::{render_checkbox, render_radio_button};
use self::link::{render_anchor, render_link};
use self::list::render_list;
use self::text::{render_code, render_email, render_wikitext_raw};
use self::user::render_user;
use super::super::condition::{check_ifcategory, check_iftags};
use super::HtmlContext;
use crate::log::prelude::*;
use crate::render::ModuleRenderMode;
use crate::tree::Element;
use ref_map::OptionRefMap;

pub fn render_elements(log: &Logger, ctx: &mut HtmlContext, elements: &[Element]) {
    debug!(log, "Rendering elements"; "elements-len" => elements.len());

    for element in elements {
        render_element(log, ctx, element);
    }
}

pub fn render_element(log: &Logger, ctx: &mut HtmlContext, element: &Element) {
    macro_rules! ref_cow {
        ($input:expr) => {
            $input.ref_map(|s| s.as_ref())
        };
    }

    debug!(log, "Rendering element"; "element" => element.name());

    match element {
        Element::Container(container) => render_container(log, ctx, container),
        Element::Module(module) => {
            ctx.handle()
                .render_module(log, ctx.buffer(), module, ModuleRenderMode::Html);
        }
        Element::Text(text) => ctx.push_escaped(text),
        Element::Raw(text) => render_wikitext_raw(log, ctx, text),
        Element::Email(email) => render_email(log, ctx, email),
        Element::Anchor {
            elements,
            attributes,
            target,
        } => render_anchor(log, ctx, elements, attributes, *target),
        Element::Link { url, label, target } => {
            render_link(log, ctx, url, label, *target)
        }
        Element::Image {
            source,
            link,
            alignment,
            attributes,
        } => render_image(log, ctx, source, ref_cow!(link), *alignment, attributes),
        Element::List {
            ltype,
            items,
            attributes,
        } => render_list(log, ctx, *ltype, items, attributes),
        Element::ListItem(_) => panic!("Reached ancillary element"),
        Element::RadioButton {
            name,
            checked,
            attributes,
        } => render_radio_button(log, ctx, name, *checked, attributes),
        Element::CheckBox {
            checked,
            attributes,
        } => render_checkbox(log, ctx, *checked, attributes),
        Element::Collapsible {
            elements,
            attributes,
            start_open,
            show_text,
            hide_text,
            show_top,
            show_bottom,
        } => render_collapsible(
            log,
            ctx,
            Collapsible::new(
                elements,
                attributes,
                *start_open,
                ref_cow!(show_text),
                ref_cow!(hide_text),
                *show_top,
                *show_bottom,
            ),
        ),
        Element::IfCategory {
            conditions,
            elements,
        } => {
            if check_ifcategory(log, ctx.info(), conditions) {
                render_elements(log, ctx, elements);
            }
        }
        Element::IfTags {
            conditions,
            elements,
        } => {
            if check_iftags(log, ctx.info(), conditions) {
                render_elements(log, ctx, elements);
            }
        }
        Element::User { name, show_avatar } => render_user(log, ctx, name, *show_avatar),
        Element::Color { color, elements } => render_color(log, ctx, color, elements),
        Element::Code { contents, language } => {
            render_code(log, ctx, ref_cow!(language), contents)
        }
        Element::Html { contents } => render_html(log, ctx, contents),
        Element::Iframe { url, attributes } => render_iframe(log, ctx, url, attributes),
        Element::LineBreak => {
            ctx.html().br();
        }
        Element::LineBreaks(amount) => {
            let amount = amount.get();

            for _ in 0..amount {
                ctx.html().br();
            }
        }
        Element::HorizontalRule => {
            ctx.html().hr();
        }
    }
}
