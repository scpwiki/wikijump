/*
 * ffi/backlinks.rs
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
use crate::data::Backlinks;

#[repr(C)]
#[derive(Debug)]
pub struct ftml_backlinks {
    // Included pages
    included_pages_list: *mut *mut c_char,
    included_pages_len: usize,

    // Internal links
    internal_links_list: *mut *mut c_char,
    internal_links_len: usize,

    // External links
    external_links_list: *mut *mut c_char,
    external_links_len: usize,
}

impl From<Backlinks<'_>> for ftml_backlinks {
    fn from(backlinks: Backlinks) -> ftml_backlinks {
        macro_rules! convert_cstr_vec {
            ($list:expr) => {{
                let owned_vec = $list
                    .into_iter()
                    .map(|cow| string_to_cstr(cow.into_owned()))
                    .collect();

                vec_to_cptr(owned_vec)
            }};
        }

        let (included_pages_list, included_pages_len) =
            convert_cstr_vec!(backlinks.included_pages);
        let (internal_links_list, internal_links_len) =
            convert_cstr_vec!(backlinks.internal_links);
        let (external_links_list, external_links_len) =
            convert_cstr_vec!(backlinks.external_links);

        // Produce final struct
        ftml_backlinks {
            included_pages_list,
            included_pages_len,
            internal_links_list,
            internal_links_len,
            external_links_list,
            external_links_len,
        }
    }
}

impl ftml_backlinks {
    pub unsafe fn drop_c(&mut self) {
        drop_cptr(self.included_pages_list, self.included_pages_len, |s| {
            drop_cstr(s)
        });

        drop_cptr(self.internal_links_list, self.internal_links_len, |s| {
            drop_cstr(s)
        });

        drop_cptr(
            self.external_links_list, //
            self.external_links_len,
            |s| drop_cstr(s),
        );
    }
}
