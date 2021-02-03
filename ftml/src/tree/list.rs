/*
 * tree/list.rs
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

use super::Element;
use strum_macros::IntoStaticStr;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ListItem<'t> {
    /// This item is a series of elements.
    ///
    /// It's just an item in the list, which may have multiple elements
    /// similar to any other container.
    Elements(Vec<Element<'t>>),

    /// This item in the list is a sub-list.
    ///
    /// That is, it's another, or deeper list within the list.
    SubList(Element<'t>),
}

#[derive(
    Serialize, Deserialize, IntoStaticStr, Debug, Copy, Clone, Hash, PartialEq, Eq,
)]
#[serde(rename_all = "kebab-case")]
pub enum ListType {
    Bullet,
    Numbered,
}

impl ListType {
    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }
}
