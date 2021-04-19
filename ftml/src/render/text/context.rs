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

use crate::data::PageInfo;
use crate::non_empty_vec::NonEmptyVec;
use std::fmt::{self, Write};

#[derive(Debug)]
pub struct TextContext<'i, 'h> {
    output: String,
    info: &'i PageInfo<'i>,
    handle: &'h (),

    // Other fields to track
    list_depths: NonEmptyVec<usize>,
}

impl<'i, 'h> TextContext<'i, 'h> {
    #[inline]
    pub fn new(info: &'i PageInfo<'i>, handle: &'h ()) -> Self {
        TextContext {
            output: String::new(),
            info,
            handle,
            list_depths: NonEmptyVec::new(1),
        }
    }

    // Getters and setters
    #[inline]
    pub fn buffer(&mut self) -> &mut String {
        &mut self.output
    }

    #[inline]
    pub fn handle(&self) -> &'h () {
        self.handle
    }

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

    // Buffer management
    #[inline]
    pub fn push(&mut self, ch: char) {
        self.output.push(ch);
    }

    #[inline]
    pub fn push_multiple(&mut self, ch: char, count: u32) {
        for _ in 0..count {
            self.output.push(ch);
        }
    }

    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.output.push_str(s);
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
