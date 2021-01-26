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
use crate::enums::HeadingLevel;
use crate::tree::Element;
use strum_macros::IntoStaticStr;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Container<'t> {
    #[serde(rename = "type")]
    ctype: ContainerType,
    elements: Vec<Element<'t>>,
}

impl<'t> Container<'t> {
    #[inline]
    pub fn new(ctype: ContainerType, elements: Vec<Element<'t>>) -> Self {
        Container { ctype, elements }
    }

    #[inline]
    pub fn ctype(&self) -> ContainerType {
        self.ctype
    }

    #[inline]
    pub fn elements(&self) -> &[Element<'t>] {
        &self.elements
    }
}

impl<'t> From<Container<'t>> for Vec<Element<'t>> {
    #[inline]
    fn from(container: Container<'t>) -> Vec<Element<'t>> {
        let Container { elements, .. } = container;

        elements
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct StyledContainer<'t> {
    #[serde(rename = "type")]
    ctype: StyledContainerType,
    elements: Vec<Element<'t>>,
    attributes: AttributeMap<'t>,
}

impl<'t> StyledContainer<'t> {
    #[inline]
    pub fn new(
        ctype: StyledContainerType,
        elements: Vec<Element<'t>>,
        attributes: AttributeMap<'t>,
    ) -> Self {
        StyledContainer {
            ctype,
            elements,
            attributes,
        }
    }

    #[inline]
    pub fn ctype(&self) -> StyledContainerType {
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

impl<'t> From<StyledContainer<'t>> for Vec<Element<'t>> {
    #[inline]
    fn from(container: StyledContainer<'t>) -> Vec<Element<'t>> {
        let StyledContainer { elements, .. } = container;

        elements
    }
}

#[derive(
    Serialize, Deserialize, IntoStaticStr, Debug, Copy, Clone, Hash, PartialEq, Eq,
)]
#[serde(rename_all = "kebab-case")]
pub enum ContainerType {
    Paragraph,
    Strong,
    Emphasis,
    Underline,
    Superscript,
    Subscript,
    Strikethrough,
    Monospace,
    Header(HeadingLevel),
}

impl ContainerType {
    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }

    #[inline]
    pub fn html_tag(self) -> &'static str {
        match self {
            ContainerType::Paragraph => "p",
            ContainerType::Strong => "strong",
            ContainerType::Emphasis => "italics",
            ContainerType::Underline => "u",
            ContainerType::Superscript => "sup",
            ContainerType::Subscript => "sub",
            ContainerType::Strikethrough => "s",
            ContainerType::Monospace => "tt",
            ContainerType::Header(level) => level.html_tag(),
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

#[derive(
    Serialize, Deserialize, IntoStaticStr, Debug, Copy, Clone, Hash, PartialEq, Eq,
)]
#[serde(rename_all = "kebab-case")]
pub enum StyledContainerType {
    Span,
    Div,
    Mark,
    Insertion,
    Deletion,
}

impl StyledContainerType {
    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }

    #[inline]
    pub fn html_tag(self) -> &'static str {
        match self {
            StyledContainerType::Span => "span",
            StyledContainerType::Div => "div",
            StyledContainerType::Mark => "mark",
            StyledContainerType::Insertion => "ins",
            StyledContainerType::Deletion => "del",
        }
    }
}

impl slog::Value for StyledContainerType {
    fn serialize(
        &self,
        _: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, self.name())
    }
}
