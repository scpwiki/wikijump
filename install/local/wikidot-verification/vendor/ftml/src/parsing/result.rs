/*
 * parsing/result.rs
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

use crate::parsing::Parser;
use crate::parsing::error::ParseError;
use crate::tree::{Element, Elements};
use std::marker::PhantomData;

pub type ParseResult<'r, 't, T> = Result<ParseSuccess<'r, 't, T>, ParseError>;
pub type ParseSuccessTuple<T> = (T, Vec<ParseError>, bool);

#[must_use]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParseSuccess<'r, 't, T>
where
    T: 't,
    'r: 't,
{
    pub item: T,
    pub errors: Vec<ParseError>,
    pub paragraph_safe: bool,

    // Marker fields to assert that the 'r lifetime is at least as long as 't.
    #[doc(hidden)]
    _ref_marker: PhantomData<&'r ()>,
    #[doc(hidden)]
    _text_marker: PhantomData<&'t str>,
}

impl<T> ParseSuccess<'_, '_, T> {
    #[inline]
    pub fn new(item: T, errors: Vec<ParseError>, paragraph_safe: bool) -> Self {
        ParseSuccess {
            item,
            errors,
            paragraph_safe,
            _ref_marker: PhantomData,
            _text_marker: PhantomData,
        }
    }

    pub fn chain(
        self,
        all_errors: &mut Vec<ParseError>,
        all_paragraph_safe: &mut bool,
    ) -> T {
        let ParseSuccess {
            item,
            mut errors,
            paragraph_safe,
            ..
        } = self;

        // Append previous errors
        all_errors.append(&mut errors);

        // Update paragraph safety
        *all_paragraph_safe &= paragraph_safe;

        // Return resultant item
        item
    }
}

impl<'r, 't, T> ParseSuccess<'r, 't, T> {
    pub fn map<F, U>(self, f: F) -> ParseSuccess<'r, 't, U>
    where
        F: FnOnce(T) -> U,
    {
        let ParseSuccess {
            item,
            errors,
            paragraph_safe,
            ..
        } = self;

        let new_item = f(item);

        ParseSuccess {
            item: new_item,
            errors,
            paragraph_safe,
            _ref_marker: PhantomData,
            _text_marker: PhantomData,
        }
    }

    #[inline]
    pub fn map_ok<F, U>(self, f: F) -> ParseResult<'r, 't, U>
    where
        F: FnOnce(T) -> U,
    {
        Ok(self.map(f))
    }
}

impl<'t> ParseSuccess<'_, 't, Elements<'t>> {
    pub fn check_partials(&self, parser: &Parser) -> Result<(), ParseError> {
        for element in &self.item {
            // This check only applies if the element is a partial.
            if let Element::Partial(partial) = element {
                // Check if the current rule is looking for a partial.
                if !parser.accepts_partial().matches(partial) {
                    // Found a partial when not looking for one. Raise the appropriate error.
                    return Err(parser.make_err(partial.parse_error_kind()));
                }
            }
        }

        Ok(())
    }
}

impl ParseSuccess<'_, '_, ()> {
    #[inline]
    pub fn into_errors(self) -> Vec<ParseError> {
        self.errors
    }
}

impl<'r, 't, T> From<ParseSuccess<'r, 't, T>> for ParseSuccessTuple<T> {
    #[inline]
    fn from(success: ParseSuccess<'r, 't, T>) -> ParseSuccessTuple<T> {
        let ParseSuccess {
            item,
            errors,
            paragraph_safe,
            ..
        } = success;

        (item, errors, paragraph_safe)
    }
}
