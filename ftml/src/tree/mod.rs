/*
 * tree/mod.rs
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

mod container;
mod element;

pub use self::container::*;
pub use self::element::*;

use crate::ParseResult;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SyntaxTree<'a> {
    pub elements: Vec<Element<'a>>,
}

impl<'a> SyntaxTree<'a> {
    pub fn from_element_result(result: ParseResult<Vec<Element<'a>>>) -> ParseResult<Self> {
        // Extract values from result
        let (mut elements, errors) = result.into();

        // Clean up element list
        elements.retain(|e| e != &Element::Null);

        // Create final SyntaxTree result
        let tree = SyntaxTree { elements };
        ParseResult::new(tree, errors)
    }
}
