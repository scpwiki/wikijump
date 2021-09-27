/*
 * ffi/page_info.rs
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

use super::prelude::*;
use crate::data::PageInfo;

#[repr(C)]
#[derive(Debug)]
pub struct ftml_page_info {
    pub page: *const c_char,
    pub category: *const c_char,
    pub site: *const c_char,
    pub title: *const c_char,
    pub alt_title: *const c_char,
    pub rating: f32,
    pub tags_list: *const *const c_char,
    pub tags_len: usize,
    pub language: *const c_char,
}

impl ftml_page_info {
    pub unsafe fn to_page_info<'a>(&self) -> PageInfo<'a> {
        let rust_tags = cptr_to_slice(self.tags_list, self.tags_len)
            .iter()
            .map(|ptr| cstr_to_cow(*ptr))
            .collect();

        PageInfo {
            page: cstr_to_cow(self.page),
            category: cstr_to_cow_optional(self.category),
            site: cstr_to_cow(self.site),
            title: cstr_to_cow(self.title),
            alt_title: cstr_to_cow_optional(self.alt_title),
            rating: self.rating,
            tags: rust_tags,
            language: cstr_to_cow(self.language),
        }
    }
}
