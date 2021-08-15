/*
 * ffi/page_ref.rs
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
use crate::data::PageRef;

#[repr(C)]
#[derive(Debug)]
pub struct ftml_page_ref {
    pub site: *mut c_char,
    pub page: *mut c_char,
}

impl From<PageRef<'_>> for ftml_page_ref {
    fn from(page_ref: PageRef) -> ftml_page_ref {
        let PageRef { site, page } = page_ref;

        ftml_page_ref {
            site: cow_to_cstr_null(site),
            page: cow_to_cstr(page),
        }
    }
}
