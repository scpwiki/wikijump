/*
 * ffi/css.rs
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

#[inline]
#[cfg(feature = "css")]
fn css_const() -> &'static str {
    crate::FTML_BASE_CSS
}

#[inline]
#[cfg(not(feature = "css"))]
fn css_const() -> &'static str {
    ""
}

lazy_static! {
    static ref FTML_BASE_CSS: CString = CString::new(css_const()).unwrap();
}

/// Outputs a copy of the ftml base CSS.
///
/// This string data is immutable and should not be modified.
/// This function only produces output if it was built with the "css" feature enabled.
#[no_mangle]
pub extern "C" fn ftml_base_css() -> *const c_char {
    FTML_BASE_CSS.as_ptr()
}
