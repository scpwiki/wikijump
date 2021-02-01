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

type DepthList<E> = Vec<DepthItem<E>>;

#[derive(Debug, Clone)]
enum DepthItem<E> {
    Element(E),
    Level(DepthList<E>),
}

#[derive(Debug, Clone)]
struct DepthStack<E> {
    tree: DepthList<E>,
    stack: Vec<Vec<E>>,
}

impl<E> DepthStack<E> {
    #[inline]
    fn new() -> Self {
        DepthStack {
            tree: vec![],
            stack: vec![vec![]],
        }
    }

    fn step_in(&mut self) {
        self.stack.push(Vec::new());
    }

    fn step_out(&mut self) {
        if let Some(level) = self.stack.pop() {
            self.tree.push(DepthItem::Level(level))
        }
    }

    fn push_element(&mut self, element: E) {
        self.stack
            .last()
            .expect("No current frame in stack")
            .push(element);
    }

    fn into_tree(self) -> Self {
        todo!()
    }
}

pub fn process_depths<I, E>(items: I) -> DepthList<E>
where
    I: IntoIterator<Item = (usize, E)>,
{
    let mut stack = DepthStack::new();

    // The depth value for the previous item
    let mut previous = 0;

    // Iterate through each of the items
    for (depth, element) in items {
        // Add or remove new depth levels as appropriate,
        // based on what our new depth value is compared
        // to the value in the previous iteration.
        //
        // If previous == depth, then neither of these for loops will run
        // If previous < depth, then only the first will run
        // If previous > depth, then only the second will run

        // Open new levels
        for _ in previous..depth {
            stack.step_in();
        }

        // Close existing levels
        for _ in depth..previous {
            stack.step_out();
        }

        stack.push_element(element);
    }

    stack.into_tree()
}

#[test]
fn depth() {
    todo!()
}
