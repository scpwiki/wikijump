/*
 * render/handle/object.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith for Project Foundation
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

//! Trait for interfacing with external state and retrieving article metadata.

use crate::Result;
use std::collections::HashSet;

pub trait ArticleHandle {
    /// Gets the article's title.
    fn get_title(&self, id: u64) -> Result<String>;

    /// Gets the article's rating, if it has one.
    fn get_rating(&self, id: u64) -> Result<Option<i32>>;

    /// Gets the tags currently associated with the article.
    fn get_tags(&self, id: u64) -> Result<HashSet<String>>;
}
