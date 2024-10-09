/*
 * hash/blo.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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

use arraystring::ArrayString;
use sha2::{Digest, Sha512};
use typenum::U128;

/// The expected length of a blob hash digest.
///
/// This is the output length for SHA-512 in bytes.
pub const BLOB_HASH_LENGTH: usize = 64;

/// The array type for a blob hash digest.
pub type BlobHash = [u8; 64];

/// The stack string type for a hex representation of a blob hash.
///
/// Because it is hexadecimal, it must be double the size of the
/// actual byte buffer it represents.
pub type BlobHexHash = ArrayString<U128>;

/// Produces a byte array containing the SHA-512 hash for the given data.
pub fn sha512_hash(data: &[u8]) -> BlobHash {
    // Perform hash
    let mut hasher = Sha512::new();
    hasher.update(data);
    let result = hasher.finalize();
    slice_to_blob_hash(&result)
}

/// Convert a slice into a hash array.
///
/// # Panics
/// Panics if the input slice is not the appropriate size.
pub fn slice_to_blob_hash(slice: &[u8]) -> BlobHash {
    let mut hash = [0; 64];
    hash.copy_from_slice(slice);
    hash
}

/// Converts the given SHA-512 hash into a hex array string.
pub fn blob_hash_to_hex(hash: &[u8]) -> BlobHexHash {
    debug_assert_eq!(
        hash.len(),
        BLOB_HASH_LENGTH,
        "SHA-512 hash buffer of incorrect length",
    );

    let mut hex_bytes = [0; 128];

    hex::encode_to_slice(hash, &mut hex_bytes)
        .expect("Encoding hash to hex slice failed");

    ArrayString::from_utf8(hex_bytes).expect("Encoded hash was not UTF-8")
}
