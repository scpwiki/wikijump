/*
 * tree/element.rs
 *
 * ftml - Library to parse Wikidot code
 * Copyright (C) 2019-2020 Ammon Smith
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

use super::Container;
use crate::enums::{AnchorTarget, LinkLabel};
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "element", content = "data")]
pub enum Element<'t> {
    /// Generic element that contains other elements within it.
    ///
    /// Examples would include italics, paragraphs, divs, etc.
    Container(Container<'t>),

    /// An element only containing text.
    ///
    /// Should be formatted like typical body text.
    Text(Cow<'t, str>),

    /// Raw text.
    ///
    /// This should be formatted exactly as listed.
    /// For instance, spaces being rendered to HTML should
    /// produce a `&nbsp;`.
    Raw(Cow<'t, str>),

    /// An element indicating an email.
    ///
    /// Whether this should become a clickable href link or just text
    /// is up to the render implementation.
    Email(Cow<'t, str>),

    /// An element linking to a different page.
    ///
    /// The "label" field is an optional field denoting what the link should
    /// display. If `None`, use the link's value itself, that is, `label.unwrap_or(url)`.
    ///
    /// The "url" field is either a page name (relative URL) or full URL.
    Link {
        url: Cow<'t, str>,
        label: LinkLabel<'t>,
        anchor: AnchorTarget,
    },

    /// Element containing colored text.
    ///
    /// The CSS designation of the color is specified, followed by the elements contained within.
    Color {
        color: Cow<'t, str>,
        elements: Vec<Element<'t>>,
    },

    /// A newline or line break.
    ///
    /// This calls for a newline in the final output, such as `<br>` in HTML.
    LineBreak,

    /// A horizontal rule.
    HorizontalRule,

    /// A null element.
    ///
    /// The element equivalent of a no-op instruction. No action should be taken,
    /// and it should be skipped over.
    Null,
}

impl Element<'_> {
    pub fn name(&self) -> &'static str {
        match self {
            Element::Container(container) => container.etype().name(),
            Element::Text(_) => "Text",
            Element::Raw(_) => "Raw",
            Element::Email(_) => "Email",
            Element::Link { .. } => "Link",
            Element::Color { .. } => "Color",
            Element::LineBreak => "LineBreak",
            Element::HorizontalRule => "HorizontalRule",
            Element::Null => "Null",
        }
    }
}
