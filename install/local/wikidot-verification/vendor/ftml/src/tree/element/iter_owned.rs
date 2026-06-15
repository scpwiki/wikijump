/*
 * tree/element/iter_owned.rs
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

use super::{Element, Elements};

impl<'t> IntoIterator for Elements<'t> {
    type Item = Element<'t>;
    type IntoIter = OwnedElementsIterator<'t>;

    fn into_iter(self) -> Self::IntoIter {
        // Collect items into a Vec for iteration.
        //
        // We reverse it so that each .pop() yields
        // the next item during iteration.
        //
        // See commit 13058b8e7f89b0 for why we are not using
        // an enum with variants for each case.
        let elements = match self {
            Elements::None => vec![],
            Elements::Single(element) => vec![element],
            Elements::Multiple(mut elements) => {
                // So we can just pop for each step
                elements.reverse();
                elements
            }
        };

        OwnedElementsIterator { elements }
    }
}

/// Owned iterator implementation for `Elements`.
#[derive(Debug)]
pub struct OwnedElementsIterator<'t> {
    elements: Vec<Element<'t>>,
}

impl<'t> Iterator for OwnedElementsIterator<'t> {
    type Item = Element<'t>;

    #[inline]
    fn next(&mut self) -> Option<Element<'t>> {
        self.elements.pop()
    }
}

#[test]
fn iter() {
    macro_rules! test {
        ($elements:expr, $expected:expr $(,)?) => {{
            let elements = $elements;

            let actual: Vec<Element> = elements.into_iter().collect();
            let expected = $expected;

            assert_eq!(
                actual, expected,
                "Actual element iteration doesn't match expected",
            );
        }};
    }

    test!(Elements::None, vec![]);
    test!(Elements::Single(text!("a")), vec![text!("a")]);
    test!(
        Elements::Multiple(vec![]), //
        vec![],
    );
    test!(
        Elements::Multiple(vec![text!("a")]), //
        vec![text!("a")],
    );
    test!(
        Elements::Multiple(vec![text!("a"), text!("b")]),
        vec![text!("a"), text!("b")],
    );
    test!(
        Elements::Multiple(vec![text!("a"), text!("b"), text!("c")]),
        vec![text!("a"), text!("b"), text!("c")],
    );
}
