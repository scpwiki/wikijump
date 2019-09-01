/*
 * render/handle/wikidot.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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

// TODO
use super::{ArticleHandle, User};
#[allow(dead_code)]
use crate::Result;
use std::collections::HashSet;

#[derive(Debug)]
pub struct WikidotHandle;

impl ArticleHandle for WikidotHandle {
    fn get_title(&self, _id: u64) -> Result<String> {
        unimplemented!()
    }

    fn get_rating(&self, _id: u64) -> Result<Option<i32>> {
        unimplemented!()
    }

    fn get_tags(&self, _id: u64) -> Result<HashSet<String>> {
        unimplemented!()
    }

    fn get_user<'a>(&self, _name: &'a str) -> Result<Option<User<'a>>> {
        unimplemented!()
    }
}
