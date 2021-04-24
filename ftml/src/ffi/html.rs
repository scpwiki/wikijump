/*
 * ffi/html.rs
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
use crate::render::html::{HtmlMeta, HtmlMetaType, HtmlOutput};

#[repr(C)]
#[derive(Debug)]
pub enum ftml_html_meta_type {
    META_NAME,
    META_HTTP_EQUIV,
    META_PROPERTY,
}

impl From<HtmlMetaType> for ftml_html_meta_type {
    #[inline]
    fn from(tag_type: HtmlMetaType) -> ftml_html_meta_type {
        use ftml_html_meta_type::*;

        match tag_type {
            HtmlMetaType::Name => META_NAME,
            HtmlMetaType::HttpEquiv => META_HTTP_EQUIV,
            HtmlMetaType::Property => META_PROPERTY,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ftml_html_meta {
    pub tag_type: ftml_html_meta_type,
    pub name: *mut c_char,
    pub value: *mut c_char,
}

impl From<HtmlMeta> for ftml_html_meta {
    fn from(HtmlMeta { tag_type, name, value }: HtmlMeta) -> ftml_html_meta {
        ftml_html_meta {
            tag_type: tag_type.into(),
            name: string_to_cstr(name),
            value: string_to_cstr(value),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ftml_html_output {
    pub html: *mut c_char,
    pub style: *mut c_char,
    pub meta_list: *mut ftml_html_meta,
    pub meta_len: usize,
}

impl ftml_html_output {
    pub fn write_into(&mut self, output: &HtmlOutput) {
        self.html = string_to_cstr(&output.html);
        self.style = string_to_cstr(&output.style);

        let (meta_list, meta_len) = todo!(); // TODO list
        self.meta_list = meta_list;
        self.meta_len = meta_len;
    }
}

#[no_mangle]
pub unsafe extern "C" fn ftml_destroy_html_output(ptr: *mut ftml_html_output) {
    let this = &mut *ptr;

    drop_cstr(this.html);
    drop_cstr(this.style);
    //drop_list(this.meta_list, this.meta_len); TODO

    this.html = ptr::null_mut();
    this.style = ptr::null_mut();
    this.meta_list = ptr::null_mut();
    this.meta_len = 0;
}
