/*
 * parsing/outcome.rs
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

use super::ParseError;
use std::borrow::{Borrow, BorrowMut};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ParseOutcome<T> {
    value: T,
    errors: Vec<ParseError>,
}

impl<T> ParseOutcome<T> {
    #[inline]
    pub fn new<I>(value: T, errors: I) -> Self
    where
        I: Into<Vec<ParseError>>,
    {
        ParseOutcome {
            value,
            errors: errors.into(),
        }
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

impl<U> ParseOutcome<Vec<U>> {
    #[inline]
    pub fn push(&mut self, item: U) {
        self.value.push(item);
    }
}

impl<T> Clone for ParseOutcome<T>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        ParseOutcome {
            value: self.value.clone(),
            errors: self.errors.clone(),
        }
    }
}

impl<T> Default for ParseOutcome<T>
where
    T: Default,
{
    #[inline]
    fn default() -> Self {
        ParseOutcome {
            value: T::default(),
            errors: Vec::new(),
        }
    }
}

impl<T> Borrow<T> for ParseOutcome<T> {
    #[inline]
    fn borrow(&self) -> &T {
        &self.value
    }
}

impl<T> BorrowMut<T> for ParseOutcome<T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> From<ParseOutcome<T>> for (T, Vec<ParseError>) {
    #[inline]
    fn from(outcome: ParseOutcome<T>) -> (T, Vec<ParseError>) {
        let ParseOutcome { value, errors } = outcome;

        (value, errors)
    }
}

#[test]
fn outcome() {
    let mut outcome = ParseOutcome::new(vec!['a'], vec![]);

    assert_eq!(outcome.value(), &['a']);
    assert_eq!(outcome.errors(), &[]);

    outcome.push('b');

    assert_eq!(outcome.value(), &['a', 'b']);
    assert_eq!(outcome.errors(), &[]);

    let outcome_2 = outcome.clone();
    assert_eq!(outcome, outcome_2);
}

#[test]
fn default() {
    let mut outcome: ParseOutcome<Option<i32>> = ParseOutcome::default();

    {
        let value: &Option<i32> = outcome.borrow();
        assert_eq!(value, &None);
    }

    *outcome.borrow_mut() = Some(10);
}
