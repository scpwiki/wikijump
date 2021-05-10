/*
 * ffi/misc.rs
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

//! Miscellaneous exports to C.

use super::prelude::*;

lazy_static! {
    static ref VERSION: CString = CString::new(crate::info::VERSION.as_bytes()).unwrap();
}

/// Returns the version string for this instance of ftml.
///
/// The string returned is statically allocated and must not be modified.
#[no_mangle]
pub extern "C" fn ftml_version() -> *const c_char {
    VERSION.as_ptr()
}
