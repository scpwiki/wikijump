/*
 * hash/text.rs
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

use arraystring::ArrayString;
use tiny_keccak::{Hasher, KangarooTwelve};
use typenum::U32;

/// The expected length of a text hash digest.
///
/// This is the standard output length for KangarooTwelve in bytes.
pub const TEXT_HASH_LENGTH: usize = 16;

/// The array type for a text hash digest;
pub type TextHash = [u8; 16];

/// The stack string type for a hex representation of a text hash.
///
/// Because it is hexadecimal, it must be double the size of the
/// actual byte buffer it represents.
pub type TextHexHash = ArrayString<U32>;

/// Produces a byte array containing the KangaroTwelve hash for the given data.
pub fn k12_hash(data: &[u8]) -> TextHash {
    let mut bytes = [0; 16];
    let mut hasher = KangarooTwelve::new(data);
    hasher.update(data);
    hasher.finalize(&mut bytes);
    bytes
}

/// Converts the given KangarooTwelve hash into a hex array string.
pub fn text_hash_to_hex(hash: &[u8]) -> TextHexHash {
    debug_assert_eq!(
        hash.len(),
        TEXT_HASH_LENGTH,
        "KangarooTwelve hash buffer of incorrect length",
    );

    let mut hex_bytes = [0; 32];

    hex::encode_to_slice(hash, &mut hex_bytes)
        .expect("Encoding hash to hex slice failed");

    ArrayString::from_utf8(hex_bytes).expect("Encoded hash was not UTF-8")
}

#[test]
fn k12_hash_is_stable_and_hex_encoded() {
    let hash = k12_hash(b"wikijump");

    assert_eq!(hash.len(), TEXT_HASH_LENGTH);
    assert_ne!(hash, [0; TEXT_HASH_LENGTH]);
    assert_eq!(hash, k12_hash(b"wikijump"));
    assert_ne!(hash, k12_hash(b"wikijump!"));
    assert_eq!(text_hash_to_hex(&hash).to_string(), hex::encode(hash));
}
