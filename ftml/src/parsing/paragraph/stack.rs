/*
 * parsing/paragraph/stack.rs
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

use crate::parsing::prelude::*;
use crate::tree::{AttributeMap, Container, ContainerType};
use std::mem;

#[derive(Debug)]
pub struct ParagraphStack<'t> {
    /// The `Logger` instance used for logging stack operations.
    log: Logger,

    /// Elements being accumulated in the current paragraph.
    current: Vec<Element<'t>>,

    /// Previous elements created, to be outputted in the final `SyntaxTree`.
    finished: Vec<Element<'t>>,

    /// Gathered exceptions from paragraph parsing.
    exceptions: Vec<ParseException<'t>>,
}

impl<'t> ParagraphStack<'t> {
    #[inline]
    pub fn new(log: &Logger) -> Self {
        ParagraphStack {
            log: Logger::clone(log),
            current: Vec::new(),
            finished: Vec::new(),
            exceptions: Vec::new(),
        }
    }

    #[inline]
    pub fn current_empty(&self) -> bool {
        self.current.is_empty()
    }

    #[inline]
    pub fn reserve_elements(&mut self, additional: usize) {
        self.current.reserve(additional);
    }

    #[inline]
    pub fn push_element(&mut self, element: Element<'t>, paragraph_safe: bool) {
        debug!(
            self.log,
            "Pushing element to stack";
            "element" => element.name(),
            "paragraph-safe" => paragraph_safe,
        );

        if paragraph_safe {
            // Add it to the current (or new) paragraph. Nothing special.

            self.current.push(element);
        } else {
            // This has to be its own "finished" element, outside of any
            // paragraph wrapper. So finish up what we have, then add this element.

            self.end_paragraph();
            self.finished.push(element);
        }
    }

    #[inline]
    pub fn push_exceptions(&mut self, exceptions: &mut Vec<ParseException<'t>>) {
        debug!(
            self.log,
            "Pushing exception to stack";
            "exceptions-len" => exceptions.len(),
        );

        self.exceptions.append(exceptions);
    }

    pub fn build_paragraph(&mut self) -> Option<Element<'t>> {
        debug!(
            self.log,
            "Building paragraph from current stack state";
            "current-len" => self.current.len(),
        );

        // Don't create empty paragraphs
        if self.current.is_empty() {
            trace!(
                self.log,
                "No paragraph created, no pending elements in stack",
            );

            return None;
        }

        // Pull out gathered elements, then make a new paragraph container
        let elements = mem::take(&mut self.current);
        let container =
            Container::new(ContainerType::Paragraph, elements, AttributeMap::new());
        let element = Element::Container(container);

        Some(element)
    }

    pub fn end_paragraph(&mut self) {
        debug!(
            self.log,
            "Ending the current paragraph to push as a completed element",
        );

        if let Some(paragraph) = self.build_paragraph() {
            self.finished.push(paragraph);
        }
    }

    pub fn into_result<'r>(mut self) -> ParseResult<'r, 't, Vec<Element<'t>>> {
        debug!(
            self.log,
            "Converting paragraph parse stack into ParseResult",
        );

        // Finish current paragraph, if any
        self.end_paragraph();

        // Destruct stack
        let ParagraphStack {
            log: _,
            current: _,
            finished: elements,
            exceptions,
        } = self;

        // If this has any paragraphs in it, or other incompatible elements,
        // it's not fit to be wrapped in <p>.
        //
        // Otherwise it's just a listing of internal elements.
        // This is definitely not the common case here, this mostly will happen
        // if the element list is empty.
        let paragraph_safe = elements.iter().all(|element| element.paragraph_safe());

        // Return finished element list
        ok!(paragraph_safe; elements, exceptions)
    }
}
