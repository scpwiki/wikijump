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

use std::mem;

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

    // Read-only getters
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

    /// Returns the total number of elements, including the one required element.
    #[inline]
    pub fn len(&self) -> usize {
        self.others.len() + 1
    }

    /// Returns if this list has only one element, or has more.
    ///
    /// Compare to `Vec::is_empty()`.
    #[inline]
    pub fn is_single(&self) -> bool {
        self.others.is_empty()
    }

    // Mutable getters
    #[inline]
    pub fn first_mut(&mut self) -> &mut T {
        &mut self.first
    }

    #[inline]
    pub fn last_mut(&mut self) -> &mut T {
        self.others.last_mut().unwrap_or(&mut self.first)
    }

    #[inline]
    pub fn others_mut(&mut self) -> &mut Vec<T> {
        &mut self.others
    }

    // Mutation methods
    #[inline]
    pub fn push(&mut self, item: T) {
        self.others.push(item);
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.others.pop()
    }

    /// Consumes this `NonEmptyVec`, converting it into a regular `Vec`.
    ///
    /// The object is empty following this consumption,
    /// minus the new starting element of course.
    pub fn consume(&mut self, mut first: T) -> Vec<T> {
        // Swap out first element.
        mem::swap(&mut self.first, &mut first);

        // Get remaining elements, then append the old first element.
        let mut result = mem::replace(&mut self.others, Vec::new());
        result.insert(0, first);
        result
    }
}

impl<T> From<NonEmptyVec<T>> for (T, Vec<T>) {
    #[inline]
    fn from(vec: NonEmptyVec<T>) -> (T, Vec<T>) {
        let NonEmptyVec { first, others } = vec;

        (first, others)
    }
}

impl<T> From<NonEmptyVec<T>> for Vec<T> {
    #[inline]
    fn from(vec: NonEmptyVec<T>) -> Vec<T> {
        let NonEmptyVec { first, mut others } = vec;

        others.push(first);
        others
    }
}

#[test]
fn non_empty_vec() {
    macro_rules! check {
        ($vec:expr, $values:expr) => {{
            assert_eq!(
                $vec.first(),
                &$values[0],
                "First value doesn't match expected",
            );
            assert_eq!(
                $vec.others(),
                &$values[1..],
                "Remaining values don't match expected",
            );
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
