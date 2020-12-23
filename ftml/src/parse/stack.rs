/*
 * parse/stack.rs
 *
 * ftml - Library to parse Wikidot text
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

use super::{ExtractedToken, ParseError, ParseException};
use crate::parse::consume::GenericConsumption;
use crate::tree::{Container, ContainerType, Element};
use std::mem;

#[derive(Debug)]
pub struct ParagraphStack<'l, 't> {
    /// The `slog::Logger` instance used for logging stack operations.
    log: &'l slog::Logger,

    /// Elements being accumulated in the current paragraph.
    current: Vec<Element<'t>>,

    /// Previous elements created, to be outputted in the final `SyntaxTree`.
    finished: Vec<Element<'t>>,

    /// Gathered exceptions from paragraph parsing.
    exceptions: Vec<ParseException<'t>>,
}

impl<'l, 't> ParagraphStack<'l, 't> {
    #[inline]
    pub fn new(log: &'l slog::Logger) -> Self {
        ParagraphStack {
            log,
            current: Vec::new(),
            finished: Vec::new(),
            exceptions: Vec::new(),
        }
    }

    #[inline]
    pub fn push_element(&mut self, element: Element<'t>) {
        debug!(
            self.log,
            "Pushing element to stack";
            "element" => element.name(),
        );

        self.current.push(element);
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

    #[inline]
    pub fn push_error(&mut self, error: ParseError) {
        debug!(
            self.log,
            "Pushing error to stack";
            "error" => error.kind().name(),
        );

        self.exceptions.push(ParseException::Error(error));
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
        let elements = mem::replace(&mut self.current, Vec::new());
        let container = Container::new(ContainerType::Paragraph, elements);
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

    pub fn into_consumption<'r>(
        mut self,
        remaining: &'r [ExtractedToken<'t>],
    ) -> GenericConsumption<'r, 't, Vec<Element<'t>>> {
        debug!(
            self.log,
            "Converting paragraph parse stack into consumption",
        );

        self.end_paragraph();

        let ParagraphStack {
            log: _,
            current: _,
            finished: elements,
            exceptions,
        } = self;

        GenericConsumption::warn(elements, remaining, exceptions)
    }
}
