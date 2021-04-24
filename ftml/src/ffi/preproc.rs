/*
 * ffi/preproc.rs
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

/// Runs the ftml preprocessor on this string.
///
/// You are responsible for managing the memory of the passed-in buffer,
/// it may come from any source.
///
/// You may modify the contents of the returned buffer, but you must not
/// modify its length (i.e. writing a null byte), or `free()` / `realloc()`.
///
/// You must call `ftml_free()` on this value after you are finished,
/// or you will have a memory leak.
#[no_mangle]
pub unsafe extern "C" fn ftml_preprocess(input: *const c_char) -> *mut c_char {
    let log = &get_logger();

    // Convert string from C to Rust
    let mut text = cptr_to_string(input);

    // Run preprocess()
    crate::preprocess(log, &mut text);

    // Release from Rust to C
    CString::new(text)
        .expect("Preprocess output contained null bytes")
        .into_raw()
}
