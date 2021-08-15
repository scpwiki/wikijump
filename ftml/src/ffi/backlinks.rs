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

use super::page_ref::ftml_page_ref;
use super::prelude::*;
use crate::data::Backlinks;

#[repr(C)]
#[derive(Debug)]
pub struct ftml_backlinks {
    // Included pages
    pub included_pages_list: *mut ftml_page_ref,
    pub included_pages_len: usize,

    // Internal links
    pub internal_links_list: *mut ftml_page_ref,
    pub internal_links_len: usize,

    // External links
    pub external_links_list: *mut *mut c_char,
    pub external_links_len: usize,
}

impl From<Backlinks<'_>> for ftml_backlinks {
    fn from(backlinks: Backlinks) -> ftml_backlinks {
        macro_rules! convert_vec {
            ($list:expr, $convert:expr) => {{
                let owned_vec = $list.into_iter().map(|obj| $convert(obj)).collect();

                vec_to_cptr(owned_vec)
            }};
        }

        let (included_pages_list, included_pages_len) =
            convert_vec!(backlinks.included_pages, ftml_page_ref::from);

        let (internal_links_list, internal_links_len) =
            convert_vec!(backlinks.internal_links, ftml_page_ref::from);

        let (external_links_list, external_links_len) =
            convert_vec!(backlinks.external_links, cow_to_cstr);

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
        drop_cptr(
            self.included_pages_list,
            self.included_pages_len,
            |page_ref| {
                drop_cstr(page_ref.site);
                drop_cstr(page_ref.page);
            },
        );

        drop_cptr(
            self.internal_links_list,
            self.internal_links_len,
            |page_ref| {
                drop_cstr(page_ref.site);
                drop_cstr(page_ref.page);
            },
        );

        drop_cptr(self.external_links_list, self.external_links_len, |s| {
            drop_cstr(s)
        });
    }
}
