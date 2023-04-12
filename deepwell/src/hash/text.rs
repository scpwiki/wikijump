/*
 * hash/text.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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

use tiny_keccak::{Hasher, KangarooTwelve};

/// The expected length of a text hash digest.
///
/// This is the standard output length for KangarooTwelve in bytes.
pub const TEXT_HASH_LENGTH: usize = 16;

/// The array type for a text hash digest;
pub type TextHash = [u8; 16];

/// Produces a byte array containing the KangaroTwelve hash for the given data.
pub fn k12_hash(data: &[u8]) -> TextHash {
    let mut bytes = [0; 16];
    let mut hasher = KangarooTwelve::new(data);
    hasher.update(data);
    hasher.finalize(&mut bytes);
    bytes
}
