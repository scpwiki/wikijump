/*
 * tree/collection.rs
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

//! A collection of multiple items, intended to be returned from parse functions.
//!
//! Unrelated to "collect" helper functions in the parser, which are procedures
//! to collect tokens rather than composing data structures.

use std::slice;

/// Module to generalize a collection of zero, one, or many items.
///
/// This is used for both `Elements` and `PartialElements` as type aliases.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Collection<T> {
    Multiple(Vec<T>),
    Single(T),
    None,
}

impl<T> Collection<T> {
    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            Collection::Multiple(items) => items.is_empty(),
            Collection::Single(_) => false,
            Collection::None => true,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Collection::Multiple(items) => items.len(),
            Collection::Single(_) => 1,
            Collection::None => 0,
        }
    }

    pub fn all<F>(&self, f: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        match self {
            Collection::Multiple(items) => items.iter().all(f),
            Collection::Single(item) => f(item),
            Collection::None => true,
        }
    }

    pub fn any<F>(&self, f: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        match self {
            Collection::Multiple(items) => items.iter().any(f),
            Collection::Single(item) => f(item),
            Collection::None => false,
        }
    }

    pub fn append_to(self, all_items: &mut Vec<T>) {
        match self {
            Collection::Multiple(mut items) => all_items.append(&mut items),
            Collection::Single(item) => all_items.push(item),
            Collection::None => (),
        }
    }
}

impl<T: ParagraphSafe> Collection<T> {
    #[inline]
    pub fn paragraph_safe(&self) -> bool {
        self.all(ParagraphSafe::paragraph_safe)
    }
}

impl<T> AsRef<[T]> for Collection<T> {
    fn as_ref(&self) -> &[T] {
        match self {
            Collection::Multiple(items) => items,
            Collection::Single(item) => slice::from_ref(item),
            Collection::None => &[],
        }
    }
}

impl<T> From<T> for Collection<T> {
    #[inline]
    fn from(item: T) -> Collection<T> {
        Collection::Single(item)
    }
}

impl<T> From<Option<T>> for Collection<T> {
    #[inline]
    fn from(item: Option<T>) -> Collection<T> {
        match item {
            Some(item) => Collection::Single(item),
            None => Collection::None,
        }
    }
}

impl<T> From<Vec<T>> for Collection<T> {
    #[inline]
    fn from(items: Vec<T>) -> Collection<T> {
        Collection::Multiple(items)
    }
}

impl<T> IntoIterator for Collection<T> {
    type Item = T;
    type IntoIter = CollectionIterator<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Collection::None => CollectionIterator::None,
            Collection::Single(item) => CollectionIterator::Single(Some(item)),
            Collection::Multiple(mut items) => {
                // So we can just pop for each step
                items.reverse();
                CollectionIterator::Multiple(items)
            }
        }
    }
}

/// Iterator implementation for `Collection`.
#[derive(Debug)]
pub enum CollectionIterator<T> {
    Multiple(Vec<T>),
    Single(Option<T>),
    None,
}

impl<T> Iterator for CollectionIterator<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        match self {
            CollectionIterator::Multiple(ref mut items) => items.pop(),
            CollectionIterator::Single(ref mut item) => item.take(),
            CollectionIterator::None => None,
        }
    }
}

/// Helper trait to determine if an element is paragraph safe or not.
pub trait ParagraphSafe {
    fn paragraph_safe(&self) -> bool;
}

#[test]
fn iter() {
    macro_rules! check {
        ($elements:expr, $expected:expr $(,)?) => {{
            let actual: Vec<char> = $elements.into_iter().collect();
            let expected: Vec<char> = $expected;

            assert_eq!(
                actual, expected,
                "Actual item iteration doesn't match expected",
            );
        }};
    }

    check!(Collection::None, vec![]);
    check!(Collection::Single('a'), vec!['a']);
    check!(Collection::Multiple(vec![]), vec![]);
    check!(Collection::Multiple(vec!['a']), vec!['a']);
    check!(Collection::Multiple(vec!['a', 'b']), vec!['a', 'b']);
    check!(
        Collection::Multiple(vec!['a', 'b', 'c']),
        vec!['a', 'b', 'c'],
    );
}
