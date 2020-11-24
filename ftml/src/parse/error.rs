/*
 * parse/error.rs
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

use super::Token;
use std::ops::Range;
use strum_macros::IntoStaticStr;

#[derive(Debug)]
pub struct ParseResult<T> {
    value: T,
    errors: Vec<ParseError>,
}

impl<T> ParseResult<T> {
    pub fn ok(value: T) -> Self {
        ParseResult {
            value,
            errors: Vec::new(),
        }
    }

    pub fn err<I: Into<Vec<ParseError>>>(value: T, errors: I) -> Self {
        ParseResult {
            value,
            errors: errors.into(),
        }
    }
}

impl<T> Default for ParseResult<T>
where
    T: Default,
{
    #[inline]
    fn default() -> Self {
        ParseResult {
            value: T::default(),
            errors: Vec::new(),
        }
    }
}

impl<T> Clone for ParseResult<T>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        ParseResult {
            value: self.value.clone(),
            errors: self.errors.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseError {
    token: Token,
    rule: &'static str,
    span: Range<usize>,
    error: ParseErrorKind,
}

#[derive(IntoStaticStr, Debug, Copy, Clone)]
pub enum ParseErrorKind {
    RecursionDepthExceeded,
}

impl ParseErrorKind {
    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }
}
