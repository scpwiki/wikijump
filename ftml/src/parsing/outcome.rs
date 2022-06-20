/*
 * parsing/outcome.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2022 Wikijump Team
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

use super::ParseWarning;
use std::borrow::{Borrow, BorrowMut};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ParseOutcome<T> {
    value: T,
    warnings: Vec<ParseWarning>,
}

impl<T> ParseOutcome<T> {
    #[inline]
    pub fn new<I>(value: T, warnings: I) -> Self
    where
        I: Into<Vec<ParseWarning>>,
    {
        ParseOutcome {
            value,
            warnings: warnings.into(),
        }
    }

    // Getters
    #[inline]
    pub fn value(&self) -> &T {
        &self.value
    }

    #[inline]
    pub fn warnings(&self) -> &[ParseWarning] {
        &self.warnings
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
            warnings: self.warnings.clone(),
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
            warnings: Vec::new(),
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

impl<T> From<ParseOutcome<T>> for (T, Vec<ParseWarning>) {
    #[inline]
    fn from(outcome: ParseOutcome<T>) -> (T, Vec<ParseWarning>) {
        let ParseOutcome { value, warnings } = outcome;

        (value, warnings)
    }
}

#[test]
fn outcome() {
    let mut outcome = ParseOutcome::new(vec!['a'], vec![]);

    assert_eq!(outcome.value(), &['a']);
    assert_eq!(outcome.warnings(), &[]);

    outcome.push('b');

    assert_eq!(outcome.value(), &['a', 'b']);
    assert_eq!(outcome.warnings(), &[]);

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
