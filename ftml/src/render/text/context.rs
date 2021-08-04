/*
 * render/text/context.rs
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
use crate::render::Handle;
use crate::PageInfo;
use std::fmt::{self, Write};

#[derive(Debug)]
pub struct TextContext<'i, 'h> {
    output: String,
    info: &'i PageInfo<'i>,
    handle: &'h Handle,

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
}

impl<'i, 'h> TextContext<'i, 'h> {
    #[inline]
    pub fn new(info: &'i PageInfo<'i>, handle: &'h Handle) -> Self {
        TextContext {
            output: String::new(),
            info,
            handle,
            prefixes: Vec::new(),
            list_depths: NonEmptyVec::new(1),
            invisible: 0,
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
    pub fn handle(&self) -> &'h Handle {
        self.handle
    }

    #[inline]
    pub fn language(&self) -> &str {
        &self.info.language
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

impl<'i, 'h> From<TextContext<'i, 'h>> for String {
    #[inline]
    fn from(ctx: TextContext<'i, 'h>) -> String {
        ctx.output
    }
}

impl<'i, 'h> Write for TextContext<'i, 'h> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.buffer().write_str(s)
    }
}
