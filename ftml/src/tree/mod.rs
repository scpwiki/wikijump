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

#[derive(Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct SyntaxTree<'t> {
    pub elements: Vec<Element<'t>>,
}

impl<'t> SyntaxTree<'t> {
    pub(crate) fn from_element_result(
        result: ParseResult<Vec<Element<'t>>>,
        full_text: &'t str,
    ) -> ParseResult<Self> {
        // Extract values from result
        let (mut elements, errors) = result.into();

        // Remove null elements
        elements.retain(|e| e != &Element::Null);

        // Merge text elements together
        merge_text(&mut elements, full_text);

        // Create final SyntaxTree result
        let tree = SyntaxTree { elements };
        ParseResult::new(tree, errors)
    }
}

fn merge_text<'t>(elements: &mut Vec<Element<'t>>, full_text: &'t str) {
    // Find the first Element::Text from the given index
    fn find_next(elements: &[Element], offset: usize) -> Option<usize> {
        let elements = &elements[offset..];

        for (i, element) in elements.enumerate() {
            if matches!(element, Element::Text(_)) {
                return Some(offset + i);
            }
        }

        None
    }

    let mut index = 0;
    while index < elements.len() {
        // Find an Element::Text, and try merging all adjacent instances.
        match find_next(elements, index) {
            Some(idx) => index = idx,
            None => return,
        }

        // Get starting index for the string slice
        let mut start = match elements.

        // Merge until a non Element::Text is found
        for element in elements {
            ;
        }
    }
}
