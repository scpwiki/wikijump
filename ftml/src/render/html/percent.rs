/*
 * render/html/percent.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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

use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use std::fmt::{self, Display};

const ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC.remove(b':').remove(b'-').remove(b'_');

#[derive(Debug, Copy, Clone)]
pub struct PercentEncode<'a>(&'a str);

impl<'a> Display for PercentEncode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for part in utf8_percent_encode(self.0, ENCODE_SET) {
            write!(f, "{}", part)?;
        }

        Ok(())
    }
}

#[inline]
pub fn percent_encode(input: &str) -> PercentEncode {
    PercentEncode(input)
}
