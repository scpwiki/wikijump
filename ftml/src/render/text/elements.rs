/*
 * render/text/elements.rs
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

//! Module that implements text rendering for `Element` and its children.

use super::TextContext;
use crate::log::prelude::*;
use crate::render::ModuleRenderMode;
use crate::tree::{
    ContainerType, DefinitionListItem, Element, LinkLocation, ListItem, ListType,
};
use crate::url::normalize_link;
use std::borrow::Cow;

pub fn render_elements(log: &Logger, ctx: &mut TextContext, elements: &[Element]) {
    info!(log, "Rendering elements"; "elements-len" => elements.len());

    for element in elements {
        render_element(log, ctx, element);
    }
}

pub fn render_element(log: &Logger, ctx: &mut TextContext, element: &Element) {
    info!(log, "Rendering element"; "element" => element.name());

    match element {
        Element::Container(container) => {
            let mut invisible = false;
            let (add_newlines, prefix) = match container.ctype() {
                // Don't render this at all.
                ContainerType::Hidden => return,

                // Render it, but invisibly.
                // Requires setting a special mode in the context.
                ContainerType::Invisible => {
                    ctx.enable_invisible();
                    invisible = true;

                    (false, None)
                }

                // If container is "terminating" (e.g. blockquote, p), then add newlines.
                // Also, determine if we add a prefix.
                ContainerType::Div | ContainerType::Paragraph => (true, None),
                ContainerType::Blockquote => (true, Some("    ")),
                ContainerType::Header(heading) => {
                    (true, Some(heading.level.prefix_with_space()))
                }

                // Inline or miscellaneous container.
                _ => (false, None),
            };

            if add_newlines {
                // Add prefix, if there's one
                if let Some(prefix) = prefix {
                    ctx.push_prefix(prefix);
                }

                ctx.add_newline();
            }

            // Render internal elements
            render_elements(log, ctx, container.elements());

            if add_newlines {
                // Pop prefix, if there's one
                if prefix.is_some() {
                    ctx.pop_prefix();
                }

                ctx.add_newline();
            }

            if invisible {
                ctx.disable_invisible();
            }
        }
        Element::Module(module) => {
            ctx.handle()
                .render_module(log, ctx.buffer(), module, ModuleRenderMode::Text)
        }
        Element::Text(text) | Element::Raw(text) | Element::Email(text) => {
            ctx.push_str(text)
        }
        Element::Variable(name) => {
            let value = ctx.variables().get(name);

            info!(
                log,
                "Rendering variable";
                "name" => name.as_ref(),
                "value" => value,
            );

            let value = match value {
                Some(value) => str!(value),
                None => format!("{{${}}}", name),
            };

            ctx.push_str(&value);
        }
        Element::Table(table) => {
            if !ctx.ends_with_newline() {
                ctx.add_newline();
            }

            for row in &table.rows {
                ctx.push_str("|| ");

                for (i, cell) in row.cells.iter().enumerate() {
                    render_elements(log, ctx, &cell.elements);

                    if i < row.cells.len() - 1 {
                        ctx.push_str(" || ");
                    }
                }

                ctx.push_str(" ||");
                ctx.add_newline();
            }

            ctx.add_newline();
        }
        Element::Anchor {
            elements,
            attributes,
            ..
        } => {
            render_elements(log, ctx, elements);

            if let Some(href) = attributes.get().get("href") {
                let link = LinkLocation::parse(cow!(href));
                let url = get_url_from_link(ctx, &link);

                str_write!(ctx, " [{}]", url);
            }
        }
        Element::Link { link, label, .. } => {
            let url = get_url_from_link(ctx, link);

            ctx.handle().get_link_label(log, link, label, |label| {
                ctx.push_str(label);

                // Don't show URL if it's a name link, or an anchor
                if url != label && !url.starts_with('#') {
                    str_write!(ctx, " [{}]", url);
                }
            });
        }
        Element::Image {
            source,
            link,
            alignment,
            attributes,
        } => {
            let source_url = ctx.handle().get_image_link(log, ctx.info(), source);

            str_write!(ctx, "Image: {}", &source_url);

            if let Some(image) = alignment {
                let float = if image.float { " float" } else { "" };
                str_write!(ctx, " [Align: {}{}]", image.align.name(), float);
            }

            if let Some(link) = link {
                str_write!(ctx, " [Link: {}]", get_url_from_link(ctx, link));
            }

            if let Some(alt_text) = attributes.get().get("alt") {
                str_write!(ctx, " [Alt: {}]", alt_text);
            }

            if let Some(title) = attributes.get().get("title") {
                str_write!(ctx, " [Title: {}]", title);
            }
        }
        Element::List { ltype, items, .. } => {
            if !ctx.ends_with_newline() {
                ctx.add_newline();
            }

            for item in items {
                match item {
                    ListItem::Elements { elements, .. } => {
                        // Don't do anything if it's empty
                        if elements.is_empty() {
                            continue;
                        }

                        // Render bullet and its depth
                        let depth = ctx.list_depth();
                        for _ in 0..depth {
                            ctx.push(' ');
                        }

                        match *ltype {
                            ListType::Bullet => ctx.push_str("* "),
                            ListType::Numbered => {
                                let index = ctx.next_list_index();
                                str_write!(ctx, "{}. ", index);
                            }
                            ListType::Generic => (),
                        }

                        // Render elements for this list item
                        render_elements(log, ctx, elements);
                        ctx.add_newline();
                    }
                    ListItem::SubList { element } => {
                        // Update bullet depth
                        ctx.incr_list_depth();
                        render_element(log, ctx, element);
                        ctx.decr_list_depth();
                    }
                }
            }
        }
        Element::DefinitionList(items) => {
            for DefinitionListItem { key, value } in items {
                str_write!(ctx, ": ");
                render_elements(log, ctx, key);
                str_write!(ctx, " : ");
                render_elements(log, ctx, value);
                ctx.add_newline();
            }

            ctx.add_newline();
        }
        Element::RadioButton { checked, .. } => {
            str_write!(ctx, "({}) ", if *checked { '*' } else { ' ' })
        }
        Element::CheckBox { checked, .. } => {
            str_write!(ctx, "[{}] ", if *checked { 'X' } else { ' ' })
        }
        Element::Collapsible {
            elements,
            show_text,
            hide_text,
            show_top,
            show_bottom,
            ..
        } => {
            macro_rules! get_text {
                ($input:expr, $message:expr) => {
                    match $input {
                        Some(ref text) => &text,
                        None => ctx.handle().get_message(log, ctx.language(), $message),
                    }
                };
            }

            let show_text = get_text!(show_text, "collapsible-open");
            let hide_text = get_text!(hide_text, "collapsible-hide");

            // Top of collapsible
            ctx.add_newline();
            ctx.push_str(show_text);
            ctx.add_newline();

            if *show_top {
                ctx.push_str(hide_text);
                ctx.add_newline();
            }

            // Collapsible contents
            render_elements(log, ctx, elements);

            // Bottom of collapsible
            if *show_bottom {
                ctx.add_newline();
                ctx.push_str(hide_text);
                ctx.add_newline();
            }
        }
        Element::TableOfContents { .. } => {
            info!(log, "Rendering table of contents");

            let table_of_contents_title =
                ctx.handle()
                    .get_message(log, ctx.language(), "table-of-contents");

            ctx.add_newline();
            ctx.push_str(table_of_contents_title);
            ctx.add_newline();
            render_elements(log, ctx, ctx.table_of_contents());
        }
        Element::Footnote => {
            info!(log, "Rendering footnote reference");

            let index = ctx.next_footnote_index();
            str_write!(ctx, "[{}]", index);
        }
        Element::FootnoteBlock { title, hide } => {
            info!(log, "Rendering footnote block");

            if *hide || ctx.footnotes().is_empty() {
                return;
            }

            // Render footnote title
            let title_default;
            let title: &str = match title {
                Some(title) => title.as_ref(),
                None => {
                    title_default = ctx.handle().get_message(
                        log,
                        ctx.language(),
                        "footnote-block-title",
                    );
                    title_default
                }
            };

            ctx.add_newline();
            ctx.push_str(title);
            ctx.add_newline();

            // Render footnotes in order.
            for (index, contents) in ctx.footnotes().iter().enumerate() {
                str_write!(ctx, "{}. ", index + 1);

                render_elements(log, ctx, contents);
                ctx.add_newline();
            }
        }
        Element::User { name, .. } => ctx.push_str(name),
        Element::Color { elements, .. } => render_elements(log, ctx, elements),
        Element::Code { contents, language } => {
            let language = match language {
                Some(language) => language,
                None => "",
            };

            str_write!(ctx, "```{}\n{}\n```", language, contents);
        }
        Element::Math { name, latex_source } => {
            let index = ctx.next_equation_index();

            str_write!(ctx, "{}. ({})\n```latex\n{}\n```", index, name, latex_source);
        }
        Element::MathInline { latex_source } => todo!(),
        Element::Html { contents } => {
            str_write!(ctx, "```html\n{}\n```", contents);
        }
        Element::Iframe { url, .. } => str_write!(ctx, "[iframe: {}]", url),
        Element::Include {
            variables,
            elements,
            location: _location,
            ..
        } => {
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
        Element::LineBreak => ctx.add_newline(),
        Element::LineBreaks(amount) => {
            for _ in 0..amount.get() {
                ctx.add_newline();
            }
        }
        Element::ClearFloat(_) => {
            if !ctx.ends_with_newline() {
                ctx.add_newline();
            }

            ctx.push_str("~~~~~~");
            ctx.add_newline();
        }
        Element::HorizontalRule => {
            if !ctx.ends_with_newline() {
                ctx.add_newline();
            }

            ctx.push_str("------");
            ctx.add_newline();
        }
        Element::Partial(_) => panic!("Encountered partial element during parsing"),
    }
}

fn get_url_from_link<'a>(ctx: &TextContext, link: &'a LinkLocation<'a>) -> Cow<'a, str> {
    let url = normalize_link(link, ctx.handle());

    // TODO: when we remove inline javascript stuff
    if url.as_ref() == "javascript:;" {
        return Cow::Borrowed("#");
    }

    url
}
