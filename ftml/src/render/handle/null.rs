/*
 * render/handle/null.rs
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

use crate::Result;
use std::borrow::Cow;
use std::collections::HashSet;
use super::{ArticleHandle, User};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NullHandle;

impl ArticleHandle for NullHandle {
    #[inline]
    fn get_title(&self, _id: u64) -> Result<String> {
        Ok(str!())
    }

    #[inline]
    fn get_rating(&self, _id: u64) -> Result<Option<i32>> {
        Ok(None)
    }

    #[inline]
    fn get_tags(&self, _id: u64) -> Result<HashSet<String>> {
        Ok(HashSet::new())
    }

    #[inline]
    fn get_user<'a>(&self, name: &'a str) -> Result<Option<User<'a>>> {
        let user = User {
            name: Cow::Borrowed(name),
            id: 0,
            avatar: str!("https://d2qhngyckgiutd.cloudfront.net/default_avatar"),
        };

        Ok(Some(user))
    }
}
