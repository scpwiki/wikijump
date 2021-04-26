/*
 * ffi/pool.rs
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

//! Module that implements a lazy static string pool.
//!
//! This pool produces `const char *` pointers usable by C
//! based on `&'static str` constants, to avoid unnecessary
//! mutable allocations to output in the FFI.

use super::prelude::*;
use parking_lot::Mutex;
use std::collections::HashMap;

lazy_static! {
    static ref POOL: Mutex<HashMap<&'static str, CString>> = Mutex::new(HashMap::new());
}

pub fn get_static_cstr(string: &'static str) -> *const c_char {
    let mut pool = POOL.lock();
    let cstr = pool.entry(string).or_insert_with(|| {
        CString::new(string).expect("Static Rust string contains null bytes")
    });

    cstr.as_ptr()
}
