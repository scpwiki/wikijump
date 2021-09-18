/*
 * parsing/result.rs
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

use crate::parsing::exception::{ParseException, ParseWarning};
use crate::parsing::Parser;
use crate::tree::{Elements, PartialElements};
use std::convert::TryInto;
use std::marker::PhantomData;

pub type ParseResult<'r, 't, T> = Result<ParseSuccess<'r, 't, T>, ParseWarning>;
pub type ParseSuccessTuple<'t, T> = (T, Vec<ParseException<'t>>, bool);

#[must_use]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParseSuccess<'r, 't, T>
where
    'r: 't,
    T: 't,
{
    pub item: T,
    pub exceptions: Vec<ParseException<'t>>,
    pub paragraph_safe: bool,

    // Marker field to assert that the 'r lifetime is at least as long as 't.
    #[doc(hidden)]
    _marker: PhantomData<&'r ()>,
}

impl<'r, 't, T> ParseSuccess<'r, 't, T> {
    #[inline]
    pub fn new(
        item: T,
        exceptions: Vec<ParseException<'t>>,
        paragraph_safe: bool,
    ) -> Self {
        ParseSuccess {
            item,
            exceptions,
            paragraph_safe,
            _marker: PhantomData,
        }
    }

    pub fn chain(
        self,
        all_exceptions: &mut Vec<ParseException<'t>>,
        all_paragraph_safe: &mut bool,
    ) -> T {
        let ParseSuccess {
            item,
            mut exceptions,
            paragraph_safe,
            ..
        } = self;

        // Append previous exceptions
        all_exceptions.append(&mut exceptions);

        // Update paragraph safety
        *all_paragraph_safe &= paragraph_safe;

        // Return resultant item
        item
    }
}

impl<'r, 't, T> ParseSuccess<'r, 't, T>
where
    T: 't,
{
    /// Maps the `item` field of `ParseSuccess` to a different value.
    ///
    /// This operation should not fail. If it can fail, use
    /// a special method instead.
    pub fn map<F, U>(self, f: F) -> ParseSuccess<'r, 't, U>
    where
        F: FnOnce(T) -> U,
    {
        let ParseSuccess {
            item,
            exceptions,
            paragraph_safe,
            ..
        } = self;

        let new_item = f(item);

        ParseSuccess {
            item: new_item,
            exceptions,
            paragraph_safe,
            _marker: PhantomData,
        }
    }

    /// Removes the `item` field from `ParseSuccess` and passes it to the closure.
    ///
    /// The new `item` value becomes `()`, and the `ParseSuccess` is passed with
    /// the same values as when it was called.
    #[inline]
    pub fn take<F>(self, f: F) -> ParseSuccess<'r, 't, ()>
    where
        F: FnOnce(T),
    {
        self.map(f)
    }
}

impl<'r, 't> ParseSuccess<'r, 't, PartialElements<'t>> {
    pub fn unwrap_partials(
        self,
        parser: &Parser,
    ) -> Result<ParseSuccess<'r, 't, Elements<'t>>, ParseWarning> {
        let ParseSuccess {
            item: partial,
            exceptions,
            paragraph_safe,
            ..
        } = self;

        match partial.try_into() {
            Err(warn_kind) => Err(parser.make_warn(warn_kind)),
            Ok(element) => Ok(ParseSuccess {
                item: element,
                exceptions,
                paragraph_safe,
                _marker: PhantomData,
            }),
        }
    }
}

impl<'r, 't> ParseSuccess<'r, 't, ()> {
    #[inline]
    pub fn into_exceptions(self) -> Vec<ParseException<'t>> {
        self.exceptions
    }
}

impl<'r, 't, T> From<ParseSuccess<'r, 't, T>> for ParseSuccessTuple<'t, T> {
    #[inline]
    fn from(success: ParseSuccess<'r, 't, T>) -> ParseSuccessTuple<'t, T> {
        let ParseSuccess {
            item,
            exceptions,
            paragraph_safe,
            ..
        } = success;

        (item, exceptions, paragraph_safe)
    }
}
