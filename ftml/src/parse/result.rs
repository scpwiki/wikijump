/*
 * parse/result.rs
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

use super::ParseError;
use serde::{Serialize, Serializer};
use std::borrow::{Borrow, BorrowMut};

#[derive(Debug)]
pub struct ParseResult<T> {
    value: T,
    errors: Vec<ParseError>,
}

impl<T> ParseResult<T> {
    #[inline]
    pub fn ok(value: T) -> Self {
        ParseResult {
            value,
            errors: Vec::new(),
        }
    }

    #[inline]
    pub fn new<I>(value: T, errors: I) -> Self
    where
        I: Into<Vec<ParseError>>,
    {
        ParseResult {
            value,
            errors: errors.into(),
        }
    }

    #[inline]
    pub fn append_err(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    pub fn join<U>(&mut self, other: ParseResult<U>) -> U {
        let ParseResult { value, mut errors } = other;

        self.errors.append(&mut errors);
        value
    }

    // Getters
    #[inline]
    pub fn value(&self) -> &T {
        &self.value
    }

    #[inline]
    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }
}

impl<U> ParseResult<Vec<U>> {
    #[inline]
    pub fn push(&mut self, item: U) {
        self.value.push(item);
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

impl<T> Borrow<T> for ParseResult<T> {
    #[inline]
    fn borrow(&self) -> &T {
        &self.value
    }
}

impl<T> BorrowMut<T> for ParseResult<T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> Into<(T, Vec<ParseError>)> for ParseResult<T> {
    #[inline]
    fn into(self) -> (T, Vec<ParseError>) {
        let ParseResult { value, errors } = self;

        (value, errors)
    }
}

impl<T> Serialize for ParseResult<T>
where
    T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut obj = serializer.serialize_struct("ParseResult", 2)?;
        obj.serialize_field("value", &self.value)?;
        obj.serialize_field("errors", &self.errors)?;
        obj.end()
    }
}
