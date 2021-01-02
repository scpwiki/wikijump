/*
 * parse/result.rs
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

use crate::parse::error::{ParseError, ParseException};

pub type ParseResult<'t, T> = Result<ParseSuccess<'t, T>, ParseError>;
pub type ParseSuccessTuple<'t, T> = (T, Vec<ParseException<'t>>);

#[must_use]
#[derive(Debug, Clone)]
pub struct ParseSuccess<'t, T>
where
    T: 't,
{
    pub item: T,
    pub exceptions: Vec<ParseException<'t>>,
}

impl<'t, T> ParseSuccess<'t, T> {
    pub fn chain(self, all_exceptions: &mut Vec<ParseException<'t>>) -> T {
        let ParseSuccess {
            item,
            mut exceptions,
        } = self;

        // Append previous exceptions
        all_exceptions.append(&mut exceptions);

        // Return extracted item
        item
    }
}

impl<'t, T> ParseSuccess<'t, T>
where
    T: 't,
{
    pub fn map<F, U>(self, f: F) -> ParseSuccess<'t, U>
    where
        F: FnOnce(T) -> U,
    {
        let ParseSuccess { item, exceptions } = self;

        let new_item = f(item);

        ParseSuccess {
            item: new_item,
            exceptions,
        }
    }

    #[inline]
    pub fn map_ok<F, U>(self, f: F) -> ParseResult<'t, U>
    where
        F: FnOnce(T) -> U,
    {
        Ok(self.map(f))
    }
}

impl<'t> ParseSuccess<'t, ()> {
    #[inline]
    pub fn into_exceptions(self) -> Vec<ParseException<'t>> {
        let ParseSuccess {
            item: _,
            exceptions,
        } = self;

        exceptions
    }
}

impl<'t, T> Into<ParseSuccessTuple<'t, T>> for ParseSuccess<'t, T> {
    #[inline]
    fn into(self) -> ParseSuccessTuple<'t, T> {
        let ParseSuccess { item, exceptions } = self;

        (item, exceptions)
    }
}
