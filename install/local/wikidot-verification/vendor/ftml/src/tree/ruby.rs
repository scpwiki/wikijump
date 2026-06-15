/*
 * tree/ruby.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2026 Wikijump Team
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
use super::attribute::AttributeMap;
use super::clone::elements_to_owned;

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct RubyText<'t> {
    pub attributes: AttributeMap<'t>,
    pub elements: Vec<Element<'t>>,
}

impl RubyText<'_> {
    pub fn to_owned(&self) -> RubyText<'static> {
        RubyText {
            attributes: self.attributes.to_owned(),
            elements: elements_to_owned(&self.elements),
        }
    }
}
