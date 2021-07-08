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
use crate::{Backlinks, Link};

#[repr(C)]
#[derive(Debug)]
pub struct ftml_backlinks {
    // Included pages
    included_pages_present_list: *mut *mut c_char,
    included_pages_present_len: usize,
    included_pages_absent_list: *mut *mut c_char,
    included_pages_absent_len: usize,

    // Internal links
    internal_links_present_list: *mut *mut c_char,
    internal_links_present_len: usize,
    internal_links_absent_list: *mut *mut c_char,
    internal_links_absent_len: usize,

    // External links
    external_links_list: *mut *mut c_char,
    external_links_len: usize,
}

impl From<Backlinks<'_>> for ftml_backlinks {
    fn from(backlinks: Backlinks) -> ftml_backlinks {
        // Split out links, convert to C-strings
        let (included_pages_present, included_pages_absent) =
            split_links(backlinks.included_pages);

        let (internal_links_present, internal_links_absent) =
            split_links(backlinks.internal_links);

        // Only convert to C-strings, no splitting
        let external_links = backlinks
            .external_links
            .into_iter()
            .map(|cow| string_to_cstr(cow.into_owned()))
            .collect();

        // Convert into C-vectors
        let (included_pages_present_list, included_pages_present_len) =
            vec_to_cptr(included_pages_present);

        let (included_pages_absent_list, included_pages_absent_len) =
            vec_to_cptr(included_pages_absent);

        let (internal_links_present_list, internal_links_present_len) =
            vec_to_cptr(internal_links_present);

        let (internal_links_absent_list, internal_links_absent_len) =
            vec_to_cptr(internal_links_absent);

        let (external_links_list, external_links_len) = vec_to_cptr(external_links);

        // Produce final struct
        ftml_backlinks {
            included_pages_present_list,
            included_pages_present_len,
            included_pages_absent_list,
            included_pages_absent_len,
            internal_links_present_list,
            internal_links_present_len,
            internal_links_absent_list,
            internal_links_absent_len,
            external_links_list,
            external_links_len,
        }
    }
}

fn split_links<'a, I>(links: I) -> (Vec<*mut c_char>, Vec<*mut c_char>)
where
    I: IntoIterator<Item = Link<'a>>,
{
    let mut present = Vec::new();
    let mut absent = Vec::new();

    for link in links {
        let c_url = string_to_cstr(link.url.into_owned());

        if link.exists {
            present.push(c_url);
        } else {
            absent.push(c_url);
        }
    }

    (present, absent)
}
