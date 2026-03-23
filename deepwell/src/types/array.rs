/*
 * types/array.rs
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

//! Allows statically determining or comparing the length of an array.
//!
//! Arrays in Rust are fixed-length, essentially blocks of memory.
//! This type enables extraction of its length value for static assetions
//! and other const operations.

pub trait ArrayLength {
    const LENGTH: usize;
}

impl<T, const LENGTH: usize> ArrayLength for [T; LENGTH] {
    const LENGTH: usize = LENGTH;
}
