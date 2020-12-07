/*
 * parse/rule/collect/last.rs
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

use std::ptr;

/// When passed two slices, it returns the last element before the start of the second slice.
///
/// It is intended to have a very narrow usage for users of `try_collect`-based functions,
/// where the previous token to the `remaining` pointer is needed.
///
/// For instance:
/// ```text
/// let items = vec![4, 6, 0, 3, 1, 7];
/// let first = &items[1..]; /* [6, 0, 3, 1, 7] */
/// let second = &items[3..]; /* [3, 1, 7] */
///
/// let result = last_before_slice(first, second);
/// assert_eq!(result, 0);
/// ```
///
/// # Panics
///
/// This has several assertions:
/// * `first` and `second` must be slices from the same underlying source
/// * `second` must be a subslice of `first`
/// * The slices may not be empty
/// * `first` must have at least one item at the head that `second` does not.
///
/// If any of these are violated, this utility function may panic.
pub fn last_before_slice<'a, T>(mut first: &'a [T], second: &'a [T]) -> &'a T {
    let mut last = None;

    // Iterate one element at a time, until first == second.
    // Then return last (the element prior).
    while !ptr::eq(first, second) {
        let (item, remaining) = first
            .split_first()
            .expect("No more elements in first slice");

        first = remaining;
        last = Some(item);
    }

    last.expect("Both slices were equal, no previous item")
}

#[test]
fn last_slice() {
    {
        let items = vec![4, 6, 0, 3, 1, 7];
        let first = &items[1..];
        let second = &items[3..];

        assert_eq!(first, &[6, 0, 3, 1, 7]);
        assert_eq!(second, &[3, 1, 7]);

        let result = last_before_slice(first, second);
        assert_eq!(*result, 0);
    }

    {
        let items = vec!['a', 'b', 'c', 'd', 'e'];
        let first = &items[3..]; /* [d, e] */
        let second = &items[4..]; /* [e] */

        assert_eq!(first, &['d', 'e']);
        assert_eq!(second, &['e']);

        let result = last_before_slice(first, second);
        assert_eq!(*result, 'd');
    }

    {
        let items = vec![1, 2, 3];
        let first = &items[..];
        let second = &items[1..];

        assert_eq!(first, &[1, 2, 3]);
        assert_eq!(second, &[2, 3]);

        let result = last_before_slice(first, second);
        assert_eq!(*result, 1);
    }
}
