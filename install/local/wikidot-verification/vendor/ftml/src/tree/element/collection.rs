/*
 * tree/element/collection.rs
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

use super::Element;
use std::slice;

/// Wrapper for the result of producing element(s).
///
/// This has an enum instead of a simple `Vec<Element>`
/// since the most common output is a single element,
/// and it makes little sense to heap allocate for every
/// single return if we can easily avoid it.
///
/// It also contains a field marking whether all of the
/// contents are paragraph-safe or not, used by `ParagraphStack`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Elements<'t> {
    Multiple(Vec<Element<'t>>),
    Single(Element<'t>),
    None,
}

impl Elements<'_> {
    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            Elements::Multiple(elements) => elements.is_empty(),
            Elements::Single(_) => false,
            Elements::None => true,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Elements::Multiple(elements) => elements.len(),
            Elements::Single(_) => 1,
            Elements::None => 0,
        }
    }

    pub fn paragraph_safe(&self) -> bool {
        match self {
            Elements::Multiple(elements) => {
                elements.iter().all(|element| element.paragraph_safe())
            }
            Elements::Single(element) => element.paragraph_safe(),
            Elements::None => true,
        }
    }
}

impl<'t> AsRef<[Element<'t>]> for Elements<'t> {
    fn as_ref(&self) -> &[Element<'t>] {
        match self {
            Elements::Multiple(elements) => elements,
            Elements::Single(element) => slice::from_ref(element),
            Elements::None => &[],
        }
    }
}

impl<'t> From<Element<'t>> for Elements<'t> {
    #[inline]
    fn from(element: Element<'t>) -> Elements<'t> {
        Elements::Single(element)
    }
}

impl<'t> From<Option<Element<'t>>> for Elements<'t> {
    #[inline]
    fn from(element: Option<Element<'t>>) -> Elements<'t> {
        match element {
            Some(element) => Elements::Single(element),
            None => Elements::None,
        }
    }
}

impl<'t> From<Vec<Element<'t>>> for Elements<'t> {
    #[inline]
    fn from(elements: Vec<Element<'t>>) -> Elements<'t> {
        Elements::Multiple(elements)
    }
}
