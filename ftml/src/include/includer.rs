/*
 * include/includer.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Ammon Smith
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

use super::{IncludeRef, PageRef};
use std::borrow::Cow;
use std::collections::HashMap;

pub trait Includer<'t> {
    fn include_pages(
        &mut self,
        includes: &[IncludeRef<'t>],
    ) -> HashMap<PageRef<'t>, Cow<'t, str>>;

    fn no_such_include(&mut self, page_ref: &PageRef<'t>) -> Cow<'t, str>;
}

#[derive(Debug)]
pub struct NullIncluder;

impl<'t> Includer<'t> for NullIncluder {
    #[inline]
    fn include_pages(
        &mut self,
        _includes: &[IncludeRef<'t>],
    ) -> HashMap<PageRef<'t>, Cow<'t, str>> {
        HashMap::new()
    }

    #[inline]
    fn no_such_include(&mut self, page_ref: &PageRef<'t>) -> Cow<'t, str> {
        Cow::Owned(format!("No such page: {}", str!(page_ref)))
    }
}
