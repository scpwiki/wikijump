/*
 * types/reference/id.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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

//! A representation of a way of referenceing an item, either by name or by ID.
//!
//! For instance, a page can be referenced via its ID, or in the context of a site,
//! via its page slug.

use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(untagged)]
pub enum Reference<'a> {
    /// The `BIGINT` ID for this object.
    Id(i64),

    /// The string name for the object, usually the slug.
    ///
    /// However (such as in the case of files) this is the filename instead.
    /// Properties such as whether this is a normalized slug or not should not
    /// be assumed.
    ///
    /// This enum is effectively just allowing either a string or an integer
    /// to be passed in, and should be conceived of as such.
    Slug(Cow<'a, str>),
}

impl From<i64> for Reference<'static> {
    #[inline]
    fn from(id: i64) -> Reference<'static> {
        Reference::Id(id)
    }
}

impl<'a> From<&'a str> for Reference<'a> {
    #[inline]
    fn from(slug: &'a str) -> Reference<'a> {
        Reference::Slug(Cow::Borrowed(slug))
    }
}
