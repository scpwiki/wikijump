/*
 * render/text/elements.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2023 Wikijump Team
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
//!
//! The philosophy of this renderer is essentially to output what the HTML
//! renderer would, but with all tags, styling, etc stripped.
//!
//! Only pure, unformatted text should remain. Whitespace formatting
//! (such as indenting each line of a blockquote) should not occur.
//! Any formatting present must be directly justifiable.

use super::TextContext;
use crate::tree::{ContainerType, DefinitionListItem, Element, ListItem, Tab};

pub fn render_elements(ctx: &mut TextContext, elements: &[Element]) {
    info!("Rendering elements (length {})", elements.len());

    for element in elements {
        render_element(ctx, element);
    }
}

pub fn render_element(ctx: &mut TextContext, element: &Element) {
    info!("Rendering element {}", element.name());

    match element {
        Element::Container(container) => {
            let mut invisible = false;
            let add_newlines = match container.ctype() {
                // Don't render this at all.
                ContainerType::Hidden => return,

                // Render it, but invisibly.
                // Requires setting a special mode in the context.
                ContainerType::Invisible => {
                    ctx.enable_invisible();
                    invisible = true;

                    false
                }

                // If container is "terminating" (e.g. blockquote, p), then add newlines.
                // Also, determine if we add a prefix.
                ContainerType::Div
                | ContainerType::Paragraph
                | ContainerType::Blockquote
                | ContainerType::Header(_) => true,

                // Wrap any ruby text with parentheses
                ContainerType::RubyText => {
                    ctx.push('(');
                    false
                }

                // Inline or miscellaneous container.
                _ => false,
            };

            if add_newlines {
                ctx.add_newline();
            }

            // Render internal elements
            render_elements(ctx, container.elements());

            // Wrap any ruby text with parentheses
            if container.ctype() == ContainerType::RubyText {
                ctx.push(')');
            }

            if add_newlines {
                ctx.add_newline();
            }

            if invisible {
                ctx.disable_invisible();
            }
        }
        Element::Module(_) => {
            // We don't want to render modules at all
        }
        Element::Text(text) | Element::Raw(text) | Element::Email(text) => {
            ctx.push_str(text);
        }
        Element::Variable(name) => {
            let value = match ctx.variables().get(name) {
                Some(value) => str!(value),
                None => format!("{{${name}}}"),
            };

            info!(
                "Rendering variable (name '{}', value {})",
                name.as_ref(),
                value,
            );
            ctx.push_str(&value);
        }
        Element::Table(table) => {
            if !ctx.ends_with_newline() {
                ctx.add_newline();
            }

            for row in &table.rows {
                for cell in &row.cells {
                    render_elements(ctx, &cell.elements);
                }

                ctx.add_newline();
            }

            ctx.add_newline();
        }
        Element::TabView(tabs) => {
            for Tab { label, elements } in tabs {
                // Add tab name
                ctx.push_str(label);
                ctx.add_newline();

                // Add tab contents
                render_elements(ctx, elements);
                ctx.add_newline();
            }
        }
        Element::Anchor { elements, .. } => render_elements(ctx, elements),
        Element::AnchorName(_) => {
            // Anchor names are an invisible addition to the HTML
            // to aid navigation. So in text mode, they are ignored.
        }
        Element::Link { link, label, .. } => {
            let site = ctx.info().site.as_ref();

            ctx.handle().get_link_label(site, link, label, |label| {
                // Only write the label, i.e. the part that's visible
                ctx.push_str(label);
            });
        }
        Element::Image { .. } => {
            // Text cannot render images, so we don't add anything
        }
        Element::List { items, .. } => {
            if !ctx.ends_with_newline() {
                ctx.add_newline();
            }

            for item in items {
                match item {
                    ListItem::SubList { element } => render_element(ctx, element),
                    ListItem::Elements { elements, .. } => {
                        // Don't do anything if it's empty
                        if elements.is_empty() {
                            continue;
                        }

                        // Render elements for this list item
                        render_elements(ctx, elements);
                        ctx.add_newline();
                    }
                }
            }
        }
        Element::DefinitionList(items) => {
            for DefinitionListItem {
                key_elements,
                value_elements,
                ..
            } in items
            {
                render_elements(ctx, key_elements);
                ctx.push(' ');
                render_elements(ctx, value_elements);
                ctx.add_newline();
            }

            ctx.add_newline();
        }
        Element::RadioButton { .. } | Element::CheckBox { .. } => {
            // These cannot be rendered in text mode, and so are ignored.
        }
        Element::Collapsible { elements, .. } => {
            // For collapsibles, we simply show the contents.
            // No collapsible labels (open or close) are shown.

            render_elements(ctx, elements);
        }
        Element::TableOfContents { .. } => {
            // Doesn't make sense to have a textual table of contents, skip
        }
        Element::Footnote
        | Element::FootnoteBlock { .. }
        | Element::BibliographyCite { .. }
        | Element::BibliographyBlock { .. } => {
            // Footnotes and bibliographies cannot be cleanly rendered in text mode,
            // so they are skipped.
        }
        Element::User { name, .. } => ctx.push_str(name),
        Element::Date { value, format, .. } => {
            // TEMP
            if format.is_some() {
                warn!("Time format passed, feature currently not supported!");
            }

            // TODO handle error
            match value.format() {
                Ok(datetime) => str_write!(ctx, "{}", datetime),
                Err(error) => {
                    error!("Error formatting date into string: {error}");
                    str_write!(ctx, "<ERROR>");
                }
            };
        }
        Element::Color { elements, .. } => render_elements(ctx, elements),
        Element::Code { contents, .. } => {
            ctx.add_newline();
            ctx.push_str(contents);
            ctx.add_newline();
        }
        Element::Math { .. } | Element::MathInline { .. } => {
            // No real way to render arbitrary LaTeX, so we skip it.
        }
        Element::EquationReference(name) => {
            str_write!(ctx, "[{name}]");
        }
        Element::Embed(_) | Element::Html { .. } | Element::Iframe { .. } => {
            // Interactive or HTML elements like this don't make sense in
            // text mode, so we skip them.
        }
        Element::Include {
            variables,
            elements,
            ..
        } => {
            info!(
                "Rendering include (variables length {}, elements length {})",
                variables.len(),
                elements.len(),
            );

            ctx.variables_mut().push_scope(variables);
            render_elements(ctx, elements);
            ctx.variables_mut().pop_scope();
        }
        Element::Style(_) | Element::ClearFloat(_) => {
            // Style blocks and clear float do not do anything in text mode
        }
        Element::LineBreak => ctx.add_newline(),
        Element::LineBreaks(amount) => {
            for _ in 0..amount.get() {
                ctx.add_newline();
            }
        }
        Element::HorizontalRule => {
            // We could add dashes, but that looks tacky on anything
            // that is not a fixed-width font.
            //
            // So we take the safe option of doing nothing.
        }
        Element::Partial(_) => panic!("Encountered partial element during parsing"),
    }
}
