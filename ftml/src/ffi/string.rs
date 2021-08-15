/*
 * ffi/string.rs
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
use std::borrow::Cow;

#[inline]
pub unsafe fn cstr_to_string(ptr: *const c_char) -> String {
    let c_str = CStr::from_ptr(ptr);
    c_str.to_string_lossy().into_owned()
}

#[inline]
pub unsafe fn cstr_to_string_optional(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        Some(cstr_to_string(ptr))
    }
}

#[inline]
pub unsafe fn cstr_to_cow(ptr: *const c_char) -> Cow<'static, str> {
    Cow::Owned(cstr_to_string(ptr))
}

#[inline]
pub unsafe fn cstr_to_cow_optional(ptr: *const c_char) -> Option<Cow<'static, str>> {
    cstr_to_string_optional(ptr).map(Cow::Owned)
}

pub fn string_to_cstr(string: String) -> *mut c_char {
    CString::new(string)
        .expect("Rust string contains null bytes")
        .into_raw()
}

#[inline]
pub fn string_to_cstr_null(string: Option<String>) -> *mut c_char {
    match string {
        Some(string) => string_to_cstr(string),
        None => ptr::null_mut(),
    }
}

#[inline]
pub fn cow_to_cstr(cow: Cow<str>) -> *mut c_char {
    string_to_cstr(cow.into_owned())
}

#[inline]
pub fn cow_to_cstr_null(cow: Option<Cow<str>>) -> *mut c_char {
    string_to_cstr_null(cow.map(|cow| cow.into_owned()))
}

#[inline]
pub unsafe fn drop_cstr(ptr: *mut c_char) {
    if !ptr.is_null() {
        mem::drop(CString::from_raw(ptr));
    }
}

#[test]
fn ffi_string() {
    let cstr = string_to_cstr(str!("test"));
    unsafe { drop_cstr(cstr) };
}
