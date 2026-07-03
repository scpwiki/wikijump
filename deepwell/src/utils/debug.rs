/*
 * utils/debug.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
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

//! Utilities to help with `Debug` formatting on type definitions.

/// Returns the raw pointer to the given item.
///
/// Used in `Debug` to emit a pointer address for the given item.
#[inline]
pub fn debug_pointer<T>(item: &T) -> *const () {
    item as *const T as *const ()
}

#[test]
fn debug_pointer_returns_item_address() {
    let value = 42_u32;

    assert_eq!(debug_pointer(&value), (&value as *const u32).cast::<()>(),);
}
