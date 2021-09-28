/*
 * tree/tab.rs
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

use super::clone::elements_to_owned;
use super::Element;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Tab<'t> {
    pub name: Vec<Element<'t>>,
    pub contents: Vec<Element<'t>>,
}

impl Tab<'_> {
    pub fn to_owned(&self) -> Tab<'static> {
        Tab {
            name: elements_to_owned(&self.name),
            contents: elements_to_owned(&self.contents),
        }
    }
}
