/*
 * render/html/buffer.rs
 *
 * wikidot-html - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith for Project Foundation
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

//! An a buffer wrapper for `String` that implements `io::Write`.
//! It performs an assertion if any written data is not valid UTF-8.
//! This is to allow us to use the htmlescape module's writer functions.

use std::io::{self, Write};
use std::str;

#[derive(Debug)]
pub struct StringBuf<'a>(pub &'a mut String);

impl<'a> Write for StringBuf<'a> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let string = str::from_utf8(bytes).expect("String written by htmlescape wasn't UTF-8");
        self.0.push_str(string);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
