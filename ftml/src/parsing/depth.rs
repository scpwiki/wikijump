/*
 * parsing/depth.rs
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

use crate::non_empty_vec::NonEmptyVec;
use std::mem;

pub type DepthList<E> = Vec<DepthItem<E>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DepthItem<T> {
    Item(T),
    List(DepthList<T>),
}

#[derive(Debug)]
struct DepthStack<T> {
    stack: NonEmptyVec<Vec<DepthItem<T>>>,
}

impl<T> DepthStack<T> {
    #[inline]
    pub fn new() -> Self {
        DepthStack {
            stack: NonEmptyVec::new(Vec::new()),
        }
    }

    pub fn increase_depth(&mut self) {
        self.stack.push(Vec::new());
    }

    pub fn decrease_depth(&mut self) {
        if let Some(list) = self.stack.pop() {
            self.push(DepthItem::List(list));
        }
    }

    fn push(&mut self, item: DepthItem<T>) {
        self.stack.last_mut().push(item);
    }

    #[inline]
    pub fn push_item(&mut self, item: T) {
        self.push(DepthItem::Item(item));
    }

    pub fn into_tree(mut self) -> DepthList<T> {
        // Wrap all opened layers
        // Start at 1 since it's a non-empty vec
        for _ in 1..self.stack.len() {
            self.decrease_depth();
        }

        debug_assert_eq!(self.stack.len(), 1, "Open layers remain after collapsing");

        // Return top-level layer
        mem::replace(self.stack.first_mut(), Vec::new())
    }
}

pub fn process_depths<I, L, T>(items: I) -> DepthList<T>
where
    I: IntoIterator<Item = (usize, L, T)>,
    L: Copy + PartialEq,
{
    let mut stack = DepthStack::new();

    // The depth value for the previous item
    let mut previous = 0;

    // Iterate through each of the items
    for (depth, ltype, item) in items {
        // Add or remove new depth levels as appropriate,
        // based on what our new depth value is compared
        // to the value in the previous iteration.
        //
        // If previous == depth, then neither of these for loops will run
        // If previous < depth, then only the first will run
        // If previous > depth, then only the second will run

        // Open new levels
        for _ in previous..depth {
            stack.increase_depth();
        }

        // Close existing levels
        for _ in depth..previous {
            stack.decrease_depth();
        }

        // Push element and update state
        stack.push_item(item);
        previous = depth;
    }

    stack.into_tree()
}

#[test]
fn depth() {
    macro_rules! check {
        ($depths:expr, $list:expr $(,)?) => {{
            let expected: Vec<DepthItem<char>> = $list;
            let actual = process_depths($depths);

            assert_eq!(
                actual, expected,
                "Actual produced depth list doesn't match expected",
            );
        }};
    }

    macro_rules! item {
        ($item:expr) => {
            DepthItem::Item($item)
        };
    }

    macro_rules! list {
        () => {
            DepthItem::List(vec![])
        };
        ($($x:expr),+ $(,)?) => {
            DepthItem::List(vec![$($x),+])
        };
    }

    check!(vec![], vec![]);
    check!(
        vec![(0, 'a')], //
        vec![item!('a')],
    );
    check!(
        vec![(0, 'a'), (0, 'b')], //
        vec![item!('a'), item!('b')],
    );
    check!(
        vec![(0, 'a'), (0, 'b'), (1, 'c')],
        vec![item!('a'), item!('b'), list![item!('c')]]
    );
    check!(
        vec![(0, 'a'), (0, 'b'), (2, 'c')],
        vec![item!('a'), item!('b'), list![list![item!('c')]]],
    );
    check!(
        vec![(1, 'a'), (1, 'b')],
        vec![list![item!('a'), item!('b')]],
    );
    check!(
        vec![(2, 'a'), (2, 'b')],
        vec![list![list![item!('a'), item!('b')]]],
    );
    check!(
        vec![(2, 'a'), (1, 'b')],
        vec![list![list![item!('a')], item!('b')]],
    );
    check!(
        vec![(5, 'a')],
        vec![list![list![list![list![list![item!('a')]]]]]],
    );
    check!(
        vec![(2, 'a'), (3, 'b'), (1, 'c'), (1, 'd'), (2, 'e'), (0, 'f')],
        vec![
            list![
                list![item!('a'), list![item!('b')]],
                item!('c'),
                item!('d'),
                list![item!('e')],
            ],
            item!('f'),
        ],
    );
}
