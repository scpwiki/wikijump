/*
 * tree/element/iter_ref.rs
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

impl<'e, 't> IntoIterator for &'e Elements<'t> {
    type Item = &'e Element<'t>;
    type IntoIter = BorrowedElementsIterator<'e, 't>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Elements::None => BorrowedElementsIterator::None,
            Elements::Single(element) => BorrowedElementsIterator::Single(Some(element)),
            Elements::Multiple(elements) => {
                BorrowedElementsIterator::Multiple(elements, 0)
            }
        }
    }
}

/// Borrowed iterator implementation for `Elements`.
#[derive(Debug)]
pub enum BorrowedElementsIterator<'e, 't> {
    None,
    Single(Option<&'e Element<'t>>),
    Multiple(&'e [Element<'t>], usize),
}

impl<'e, 't> Iterator for BorrowedElementsIterator<'e, 't> {
    type Item = &'e Element<'t>;

    #[inline]
    fn next(&mut self) -> Option<&'e Element<'t>> {
        match self {
            BorrowedElementsIterator::None => None,
            BorrowedElementsIterator::Single(element) => element.take(),
            BorrowedElementsIterator::Multiple(elements, index) => {
                let next = elements.get(*index);
                *index += 1;
                next
            }
        }
    }
}

#[test]
fn iter() {
    macro_rules! test {
        ($elements:expr, $expected:expr $(,)?) => {{
            let elements = &$elements;

            let actual: Vec<&Element> = elements.into_iter().collect();
            let expected: Vec<&Element> = $expected;

            assert_eq!(
                actual, expected,
                "Actual element iteration doesn't match expected",
            );
        }};
    }

    test!(Elements::None, vec![]);
    test!(Elements::Single(text!("a")), vec![&text!("a")]);
    test!(
        Elements::Multiple(vec![]), //
        vec![],
    );
    test!(
        Elements::Multiple(vec![text!("a")]), //
        vec![&text!("a")],
    );
    test!(
        Elements::Multiple(vec![text!("a"), text!("b")]),
        vec![&text!("a"), &text!("b")],
    );
    test!(
        Elements::Multiple(vec![text!("a"), text!("b"), text!("c")]),
        vec![&text!("a"), &text!("b"), &text!("c")],
    );
}
