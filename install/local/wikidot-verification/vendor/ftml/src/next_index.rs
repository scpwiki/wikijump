/*
 * next_index.rs
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

/// Trait to represent an incrementing index.
///
/// This allows us to generically represent "we need the next index, conditionally"
/// without tying that function to a particular implementation of its context or state.
pub trait NextIndex<Kind> {
    /// Yield the next index in the series.
    ///
    /// This should always return `None` if indexes are disabled.
    /// Otherwise, it should return only `Some(_)` values with
    /// unique values after each invocation.
    fn next(&mut self) -> Option<usize>;
}

// Indexer kinds

#[derive(Debug)]
pub struct TableOfContentsIndex;

// Basic implementation

#[derive(Debug)]
pub struct Incrementer(Option<usize>);

impl Incrementer {
    #[inline]
    pub fn disabled() -> Self {
        Incrementer(None)
    }
}

impl Default for Incrementer {
    #[inline]
    fn default() -> Self {
        Incrementer(Some(0))
    }
}

impl NextIndex<TableOfContentsIndex> for Incrementer {
    fn next(&mut self) -> Option<usize> {
        match self.0 {
            None => None,
            Some(ref mut value) => {
                let index = *value;
                *value += 1;
                Some(index)
            }
        }
    }
}
