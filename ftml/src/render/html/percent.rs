/*
 * render/html/percent.rs
 *
 * ftml - Convert Wikidot code to HTML
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

use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use std::fmt::{self, Debug, Display};

const URL_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'/')
    .remove(b'?')
    .remove(b':')
    .remove(b'-')
    .remove(b'_');

#[derive(Copy, Clone)]
pub struct PercentEncode<'a> {
    input: &'a str,
    encode_set: &'static AsciiSet,
}

impl<'a> Debug for PercentEncode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::ptr;

        f.debug_struct("PercentEncode")
            .field("input", &self.input)
            .field(
                "encode_set",
                if ptr::eq(&self.encode_set, &URL_ENCODE_SET) {
                    &"URL_ENCODE_SET"
                } else {
                    &"UNKNOWN"
                },
            )
            .finish()
    }
}

impl<'a> Display for PercentEncode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for part in utf8_percent_encode(self.input, self.encode_set) {
            write!(f, "{}", part)?;
        }

        Ok(())
    }
}

#[inline]
pub fn percent_encode_url(input: &str) -> PercentEncode {
    PercentEncode {
        input,
        encode_set: URL_ENCODE_SET,
    }
}
