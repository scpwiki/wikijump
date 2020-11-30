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
pub struct Container<'a> {
    etype: ContainerType,
    elements: Vec<Element<'a>>,
}

impl<'a> Container<'a> {
    #[inline]
    pub fn new(etype: ContainerType, elements: Vec<Element<'a>>) -> Self {
        Container { etype, elements }
    }

    #[inline]
    pub fn etype(&self) -> ContainerType {
        self.etype
    }

    #[inline]
    pub fn elements(&self) -> &[Element<'a>] {
        &self.elements
    }
}

impl<'a> Into<Vec<Element<'a>>> for Container<'a> {
    #[inline]
    fn into(self) -> Vec<Element<'a>> {
        let Container { elements, .. } = self;

        elements
    }
}

#[derive(Serialize, Deserialize, IntoStaticStr, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ContainerType {
    Paragraph,
    Bold,
    Italics,
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
