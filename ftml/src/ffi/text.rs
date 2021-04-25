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

#[repr(C)]
#[derive(Debug)]
pub struct ftml_text_output {
    pub text: *mut c_char,
}

impl ftml_text_output {
    pub fn write_from(&mut self, text: String) {
        self.text = string_to_cstr(text);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ftml_destroy_text_output(ptr: *mut ftml_text_output) {
    let this = &mut *ptr;

    drop_cstr(this.text);
}
