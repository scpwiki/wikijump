/*
 * ffi/vec.rs
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
use std::slice;

#[inline]
pub unsafe fn cptr_to_slice<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    slice::from_raw_parts(ptr, len)
}

pub fn vec_to_cptr<T>(vec: Vec<T>) -> (*mut T, usize) {
    // Special handling for empty lists:
    // Always report pointer as NULL, discard vector like normal.
    if vec.is_empty() {
        return (ptr::null_mut(), 0);
    }

    // Otherwise, get raw aspects of Vec for export
    let mut slice = vec.into_boxed_slice(); // shrinks capacity to len
    let ptr = slice.as_mut_ptr();
    let len = slice.len();

    // Don't deallocate at end of scope, this belongs to C now
    mem::forget(slice);

    (ptr, len)
}

#[inline]
pub unsafe fn drop_cptr<T, F>(ptr: *mut T, len: usize, drop: F)
where
    F: FnMut(T),
{
    // Special handling for empty lists,
    // since it's always (NULL, 0).
    if ptr.is_null() {
        return;
    }

    Vec::from_raw_parts(ptr, len, len).drain(..).for_each(drop);
}

#[test]
fn vec() {
    let v = vec![10, 20, 30];
    let (ptr, len) = vec_to_cptr(v);
    unsafe { drop_cptr(ptr, len, mem::drop) }
}
