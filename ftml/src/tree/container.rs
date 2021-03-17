/*
 * tree/container.rs
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

//! Representation of generic syntax elements which wrap other elements.

use super::AttributeMap;
use crate::tree::{Element, HeadingLevel};
use strum_macros::IntoStaticStr;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Container<'t> {
    #[serde(rename = "type")]
    ctype: ContainerType,
    elements: Vec<Element<'t>>,
    attributes: AttributeMap<'t>,
}

impl<'t> Container<'t> {
    #[inline]
    pub fn new(
        ctype: ContainerType,
        elements: Vec<Element<'t>>,
        attributes: AttributeMap<'t>,
    ) -> Self {
        Container {
            ctype,
            elements,
            attributes,
        }
    }

    #[inline]
    pub fn ctype(&self) -> ContainerType {
        self.ctype
    }

    #[inline]
    pub fn elements(&self) -> &[Element<'t>] {
        &self.elements
    }

    #[inline]
    pub fn attributes(&self) -> &AttributeMap<'t> {
        &self.attributes
    }
}

impl<'t> From<Container<'t>> for Vec<Element<'t>> {
    #[inline]
    fn from(container: Container<'t>) -> Vec<Element<'t>> {
        let Container { elements, .. } = container;

        elements
    }
}

#[derive(
    Serialize, Deserialize, IntoStaticStr, Debug, Copy, Clone, Hash, PartialEq, Eq,
)]
#[serde(rename_all = "kebab-case")]
pub enum ContainerType {
    Bold,
    Italics,
    Underline,
    Superscript,
    Subscript,
    Strikethrough,
    Monospace,
    Span,
    Div,
    Mark,
    Blockquote,
    Insertion,
    Deletion,
    Hidden,
    Invisible,
    Size,
    Paragraph,
    Header(HeadingLevel),
}

impl ContainerType {
    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }

    #[inline]
    pub fn html_tag_and_class(self) -> (&'static str, Option<&'static str>) {
        match self {
            ContainerType::Bold => ("strong", None),
            ContainerType::Italics => ("italics", None),
            ContainerType::Underline => ("u", None),
            ContainerType::Superscript => ("sup", None),
            ContainerType::Subscript => ("sub", None),
            ContainerType::Strikethrough => ("s", None),
            ContainerType::Monospace => ("tt", None),
            ContainerType::Span => ("span", None),
            ContainerType::Div => ("div", None),
            ContainerType::Mark => ("mark", None),
            ContainerType::Blockquote => ("blockquote", None),
            ContainerType::Insertion => ("ins", None),
            ContainerType::Deletion => ("del", None),
            ContainerType::Hidden => ("span", Some("hidden")),
            ContainerType::Invisible => ("span", Some("invisible")),
            ContainerType::Size => ("span", None),
            ContainerType::Paragraph => ("p", None),
            ContainerType::Header(level) => (level.html_tag(), None),
        }
    }
}

impl slog::Value for ContainerType {
    fn serialize(
        &self,
        _: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, self.name())
    }
}
