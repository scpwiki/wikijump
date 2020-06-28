/*
 * parse/stack.rs
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

use crate::tree::{Element, Elements, SyntaxTree};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Stack<'a> {
    elements: Elements<'a>,
    stack: Vec<Elements<'a>>,
}

impl<'a> Stack<'a> {
    #[inline]
    pub fn new() -> Self {
        Stack::default()
    }

    /// Add a new layer on the stack.
    pub fn push(&mut self) {
        self.stack.push(Vec::new());
    }

    /// Pop off the current element list off the stack.
    /// Returns `None` if the stack is empty.
    /// That is, there is only the base element list for the entire document.
    pub fn pop(&mut self) -> Option<Elements<'a>> {
        self.stack.pop()
    }

    /// Appends an element to the current element list.
    pub fn append(&mut self, element: Element<'a>) {
        self.current().push(element);
    }

    /// Get the current, highest-level element list on the stack.
    pub fn current(&mut self) -> &mut Elements<'a> {
        self.stack.last_mut().unwrap_or(&mut self.elements)
    }

    /// Destructs the stack and returns the base element list.
    /// If there is existing stack context it is appended naively to the element list.
    fn into_elements(self) -> Elements<'a> {
        let Stack { elements, stack } = self;
        // TODO

        elements
    }

    /// Collapses the stack and converts it into the final abstract syntax tree (AST).
    pub fn into_syntax_tree(self) -> SyntaxTree<'a> {
        let elements = self.into_elements();

        SyntaxTree { elements }
    }
}
