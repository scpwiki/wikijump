/*
 * ffi/text.rs
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
use super::warning::ftml_warning;
use crate::parsing::ParseWarning;

#[repr(C)]
#[derive(Debug)]
pub struct ftml_text_output {
    pub text: *mut c_char,
    pub warning_list: *mut ftml_warning,
    pub warning_len: usize,
}

impl ftml_text_output {
    pub fn write_from(&mut self, text: String, warnings: &[ParseWarning]) {
        self.text = string_to_cstr(text);

        let c_warnings = warnings.iter().map(ftml_warning::from).collect();
        let (warning_ptr, warning_len) = vec_to_cptr(c_warnings);
        self.warning_list = warning_ptr;
        self.warning_len = warning_len;
    }
}

/// Destructs the given ftml_text_output structure, freeing all resources.
#[no_mangle]
pub unsafe extern "C" fn ftml_destroy_text_output(ptr: *mut ftml_text_output) {
    let this = &mut *ptr;

    drop_cstr(this.text);

    this.text = ptr::null_mut();
}
