/*
 * non_empty_vec.rs
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

//! A `Vec<T>` which always has at least one element.

#[derive(Debug, Clone, Hash, Default, PartialEq, Eq)]
pub struct NonEmptyVec<T> {
    first: T,
    others: Vec<T>,
}

impl<T> NonEmptyVec<T> {
    #[inline]
    pub fn new(first: T) -> Self {
        NonEmptyVec {
            first,
            others: Vec::new(),
        }
    }

    #[inline]
    pub fn with_capacity(first: T, other_capacity: usize) -> Self {
        NonEmptyVec {
            first,
            others: Vec::with_capacity(other_capacity),
        }
    }

    #[inline]
    pub fn first(&self) -> &T {
        &self.first
    }

    #[inline]
    pub fn last(&self) -> &T {
        self.others.last().unwrap_or(&self.first)
    }

    #[inline]
    pub fn others(&self) -> &[T] {
        &self.others
    }

    #[inline]
    pub fn push(&mut self, item: T) {
        self.others.push(item);
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.others.pop()
    }
}

impl<T> From<NonEmptyVec<T>> for (T, Vec<T>) {
    #[inline]
    fn from(vec: NonEmptyVec<T>) -> (T, Vec<T>) {
        let NonEmptyVec { first, others } = vec;

        (first, others)
    }
}

#[test]
fn non_empty_vec() {
    macro_rules! check {
        ($vec:expr, $values:expr) => {{
            assert_eq!($vec.first(), &$values[0], "First value doesn't match expected");
            assert_eq!($vec.others(), &$values[1..], "Remaining values don't match expected");
        }};
    }

    let mut vec = NonEmptyVec::new(0);
    check!(vec, [0]);

    vec.push(1);
    check!(vec, [0, 1]);

    vec.push(2);
    check!(vec, [0, 1, 2]);

    assert_eq!(vec.pop(), Some(2));
    check!(vec, [0, 1]);
}
