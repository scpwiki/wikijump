/*
 * render/text/context.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2026 Wikijump Team
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

use crate::data::PageInfo;
use crate::non_empty_vec::NonEmptyVec;
use crate::render::Handle;
use crate::settings::WikitextSettings;
use crate::tree::{Bibliography, BibliographyList, Element, VariableScopes};
use std::fmt::{self, Write};
use std::num::NonZeroUsize;

#[derive(Debug)]
pub struct TextContext<'i, 'h, 'e, 't>
where
    'e: 't,
{
    output: String,
    info: &'i PageInfo<'i>,
    handle: &'h Handle,
    settings: &'e WikitextSettings,

    //
    // Included page scopes
    //
    variables: VariableScopes,

    //
    // Elements from the syntax tree
    //
    table_of_contents: &'e [Element<'t>],
    footnotes: &'e [Vec<Element<'t>>],
    bibliographies: &'e BibliographyList<'t>,

    //
    // Other fields to track
    //
    /// Strings to prepended to each new line.
    prefixes: Vec<&'static str>,

    /// How deep we currently are in the list.
    list_depths: NonEmptyVec<usize>,

    /// Whether we're in "invisible mode".
    /// When this is non-zero, all non-newline characters
    /// added are instead replaced with spaces.
    invisible: usize,

    /// The current equation index, for rendering.
    equation_index: NonZeroUsize,

    /// The current footnote index, for rendering.
    footnote_index: NonZeroUsize,
}

impl<'i, 'h, 'e, 't> TextContext<'i, 'h, 'e, 't>
where
    'e: 't,
{
    #[inline]
    pub fn new(
        info: &'i PageInfo<'i>,
        handle: &'h Handle,
        settings: &'e WikitextSettings,
        table_of_contents: &'e [Element<'t>],
        footnotes: &'e [Vec<Element<'t>>],
        bibliographies: &'e BibliographyList<'t>,
        wikitext_len: usize,
    ) -> Self {
        TextContext {
            output: String::with_capacity(wikitext_len),
            info,
            handle,
            settings,
            variables: VariableScopes::new(),
            table_of_contents,
            footnotes,
            bibliographies,
            prefixes: Vec::new(),
            list_depths: NonEmptyVec::new(1),
            invisible: 0,
            equation_index: NonZeroUsize::new(1).unwrap(),
            footnote_index: NonZeroUsize::new(1).unwrap(),
        }
    }

    // Getters
    #[inline]
    pub fn buffer(&mut self) -> &mut String {
        &mut self.output
    }

    #[inline]
    pub fn info(&self) -> &'i PageInfo<'i> {
        self.info
    }

    #[inline]
    pub fn settings(&self) -> &WikitextSettings {
        self.settings
    }

    #[inline]
    pub fn language(&self) -> &str {
        &self.info.language
    }

    #[inline]
    pub fn handle(&self) -> &'h Handle {
        self.handle
    }

    #[inline]
    pub fn variables(&self) -> &VariableScopes {
        &self.variables
    }

    #[inline]
    pub fn variables_mut(&mut self) -> &mut VariableScopes {
        &mut self.variables
    }

    #[inline]
    pub fn table_of_contents(&self) -> &'e [Element<'t>] {
        self.table_of_contents
    }

    #[inline]
    pub fn footnotes(&self) -> &'e [Vec<Element<'t>>] {
        self.footnotes
    }

    #[inline]
    pub fn get_bibliography(&self, index: usize) -> &'e Bibliography<'t> {
        self.bibliographies.get_bibliography(index)
    }

    pub fn get_bibliography_ref(
        &self,
        label: &str,
    ) -> Option<(usize, &'e [Element<'t>])> {
        self.bibliographies.get_reference(label)
    }

    pub fn next_equation_index(&mut self) -> NonZeroUsize {
        let index = self.equation_index;
        self.equation_index = NonZeroUsize::new(index.get() + 1).unwrap();
        index
    }

    pub fn next_footnote_index(&mut self) -> NonZeroUsize {
        let index = self.footnote_index;
        self.footnote_index = NonZeroUsize::new(index.get() + 1).unwrap();
        index
    }

    // Prefixes
    #[inline]
    pub fn push_prefix(&mut self, prefix: &'static str) {
        self.prefixes.push(prefix);
    }

    #[inline]
    pub fn pop_prefix(&mut self) {
        self.prefixes.pop();
    }

    // List depth
    #[inline]
    pub fn list_depth(&self) -> usize {
        self.list_depths.len()
    }

    #[inline]
    pub fn incr_list_depth(&mut self) {
        self.list_depths.push(1);
    }

    #[inline]
    pub fn decr_list_depth(&mut self) {
        self.list_depths.pop();
    }

    pub fn next_list_index(&mut self) -> usize {
        let index = *self.list_depths.last();
        *self.list_depths.last_mut() += 1;
        index
    }

    // Invisible mode
    #[inline]
    fn invisible(&self) -> bool {
        self.invisible > 0
    }

    #[inline]
    pub fn enable_invisible(&mut self) {
        self.invisible += 1;
    }

    #[inline]
    pub fn disable_invisible(&mut self) {
        self.invisible -= 1;
    }

    // Buffer management
    pub fn push(&mut self, ch: char) {
        if self.invisible() {
            self.output.push(' ');
        } else {
            self.output.push(ch);
        }
    }

    pub fn push_str(&mut self, s: &str) {
        if self.invisible() {
            let chars = s.chars().count();
            for _ in 0..chars {
                self.output.push(' ');
            }
        } else {
            self.output.push_str(s);
        }
    }

    pub fn add_newline(&mut self) {
        self.output.push('\n');

        for prefix in &self.prefixes {
            self.output.push_str(prefix);
        }
    }

    #[inline]
    pub fn ends_with_newline(&self) -> bool {
        self.output.ends_with('\n')
    }
}

impl<'i, 'h, 'e, 't> From<TextContext<'i, 'h, 'e, 't>> for String {
    #[inline]
    fn from(ctx: TextContext<'i, 'h, 'e, 't>) -> String {
        ctx.output
    }
}

impl<'e, 't> Write for TextContext<'_, '_, 'e, 't>
where
    'e: 't,
{
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.buffer().write_str(s)
    }
}
