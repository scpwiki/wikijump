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

use super::{ParseError, ParseResult};
use crate::tree::{Container, ContainerType, Element, SyntaxTree};
use std::borrow::Cow;
use std::mem;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ParseStack<'t> {
    /// Elements being accumulated in the current paragraph.
    current: Vec<Element<'t>>,

    /// Previous elements created, to be outputted in the final `SyntaxTree`.
    finished: Vec<Element<'t>>,

    /// Gathered CSS styles, to be outputted in the final `SyntaxTree`.
    styles: Vec<Cow<'t, str>>,

    /// All errors generated during parsing so far.
    errors: Vec<ParseError>,
}

impl<'t> ParseStack<'t> {
    #[inline]
    pub fn new() -> Self {
        ParseStack::default()
    }

    #[inline]
    pub fn push_element(&mut self, element: Element<'t>) {
        self.current.push(element);
    }

    #[inline]
    pub fn push_finished(&mut self, element: Element<'t>) {
        self.finished.push(element);
    }

    #[inline]
    pub fn push_style(&mut self, style: Cow<'t, str>) {
        self.styles.push(style);
    }

    #[inline]
    pub fn push_error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    pub fn end_paragraph(&mut self) {
        let elements = mem::replace(&mut self.current, Vec::new());
        let container = Container::new(ContainerType::Paragraph, elements);
        let paragraph = Element::Container(container);
        self.finished.push(paragraph);
    }

    #[cold]
    pub fn into_syntax_tree(mut self) -> ParseResult<SyntaxTree<'t>> {
        self.end_paragraph();

        let ParseStack {
            current: _,
            finished: elements,
            styles,
            errors,
        } = self;

        SyntaxTree::from_element_result(elements, errors, styles)
    }
}
