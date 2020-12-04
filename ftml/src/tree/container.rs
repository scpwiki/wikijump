/*
 * tree/container.rs
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

use crate::enums::HeadingLevel;
use crate::tree::Element;
use strum_macros::IntoStaticStr;

/// Representation of syntax elements which wrap other elements.

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Container<'t> {
    #[serde(rename = "type")]
    etype: ContainerType<'t>,

    elements: Vec<Element<'t>>,
}

impl<'t> Container<'t> {
    #[inline]
    pub fn new(etype: ContainerType<'t>, elements: Vec<Element<'t>>) -> Self {
        Container { etype, elements }
    }

    #[inline]
    pub fn etype(&self) -> ContainerType {
        self.etype
    }

    #[inline]
    pub fn elements(&self) -> &[Element<'t>] {
        &self.elements
    }
}

impl<'t> Into<Vec<Element<'t>>> for Container<'t> {
    #[inline]
    fn into(self) -> Vec<Element<'t>> {
        let Container { elements, .. } = self;

        elements
    }
}

#[derive(Serialize, Deserialize, IntoStaticStr, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ContainerType<'t> {
    Paragraph,
    Bold,
    Italics,
    Underline,
    Superscript,
    Subscript,
    Strikethrough,
    Monospace,
    Color(&'t str),
    Header(HeadingLevel),
}

impl ContainerType<'_> {
    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }
}

impl slog::Value for ContainerType<'_> {
    fn serialize(
        &self,
        _: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, self.name())
    }
}
