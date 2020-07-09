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

use crate::tree::{Element, ElementContainer, ElementContainerType, Elements, SyntaxTree};

#[derive(Debug, Clone)]
pub struct Stack<'l, 'e> {
    log: &'l slog::Logger,
    elements: Elements<'e>,
    stack: Vec<(ElementContainerType, Elements<'e>)>,
}

impl<'l, 'e> Stack<'l, 'e> {
    #[inline]
    pub fn new(log: &'l slog::Logger) -> Self {
        Stack {
            log,
            elements: Vec::new(),
            stack: Vec::new(),
        }
    }

    /// Add a new layer on the stack with the given container type.
    pub fn push(&mut self, etype: ElementContainerType) {
        debug!(
            self.log,
            "Adding new layer to the stack with type {}",
            etype.name();
            "function" => "push",
            "type" => etype,
        );

        self.stack.push((etype, Vec::new()));
    }

    /// Pop off the current element list off the stack.
    /// Returns `None` if the stack is empty.
    /// That is, there is only the base element list for the entire document.
    pub fn pop(&mut self) -> Option<ElementContainer<'e>> {
        debug!(
            self.log,
            "Removing the last layer off of the stack";
            "function" => "pop",
        );

        self.stack
            .pop()
            .map(|(etype, elements)| ElementContainer { etype, elements })
    }

    /// Appends an element to the current element list.
    pub fn append(&mut self, element: Element<'e>) {
        debug!(
            self.log,
            "Pushing new element to stack: {:?}",
            element;
            "function" => "append",
        );

        self.current().push(element);
    }

    /// Get the current, highest-level element list on the stack.
    #[inline]
    pub fn current(&mut self) -> &mut Elements<'e> {
        match self.stack.last_mut() {
            Some((_, elements)) => elements,
            None => &mut self.elements,
        }
    }

    /// Get the current element list type, if one exists.
    #[inline]
    pub fn current_type(&self) -> Option<ElementContainerType> {
        self.stack.last().map(|(etype, _)| *etype)
    }

    /// Gets the current length of the stack.
    #[inline]
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Collapses the stack and converts it into the final abstract syntax tree (AST).
    pub fn into_syntax_tree(self) -> SyntaxTree<'e> {
        let Stack {
            log,
            elements,
            stack,
        } = self;
        // TODO

        SyntaxTree { elements }
    }
}

impl PartialEq for Stack<'_, '_> {
    #[inline]
    fn eq(&self, other: &Stack) -> bool {
        self.elements == other.elements && self.stack == other.stack
    }
}

impl Eq for Stack<'_, '_> {}
