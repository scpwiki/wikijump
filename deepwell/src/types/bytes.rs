/*
 * types/bytes.rs
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

//! Wrapper type for bytes in JSON-RPC.
//!
//! This way, instead of representing bytes as an array of integers, which takes
//! up a lot more space in the JSON textual represe, we instead produce a long
//! hexadecimal string which is more compact, readable as binary, and takes up
//! a fixed amount of string space relative to blob size.

use crate::hash::{BlobHash, TextHash};
use hex::serde::{deserialize, serialize};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Bytes<'a> {
    inner: Cow<'a, [u8]>,
}

impl<'a> Bytes<'a> {
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.as_ref().len()
    }
}

// Borrowing

impl AsRef<[u8]> for Bytes<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

// Extraction

impl From<Bytes<'_>> for Vec<u8> {
    #[inline]
    fn from(bytes: Bytes) -> Vec<u8> {
        bytes.inner.into_owned()
    }
}

// Construction

impl From<Vec<u8>> for Bytes<'static> {
    #[inline]
    fn from(bytes: Vec<u8>) -> Bytes<'static> {
        Bytes {
            inner: Cow::Owned(bytes),
        }
    }
}

impl From<BlobHash> for Bytes<'static> {
    #[inline]
    fn from(hash: BlobHash) -> Bytes<'static> {
        Bytes::from(hash.to_vec())
    }
}

impl From<TextHash> for Bytes<'static> {
    #[inline]
    fn from(hash: TextHash) -> Bytes<'static> {
        Bytes::from(hash.to_vec())
    }
}

impl<'a> From<&'a [u8]> for Bytes<'a> {
    #[inline]
    fn from(bytes: &'a [u8]) -> Bytes<'a> {
        Bytes {
            inner: Cow::Borrowed(bytes),
        }
    }
}

impl Default for Bytes<'static> {
    #[inline]
    fn default() -> Bytes<'static> {
        Bytes {
            inner: Cow::Owned(Vec::new()),
        }
    }
}

// Serialization

impl<'a> Serialize for Bytes<'a> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for Bytes<'static> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = deserialize(deserializer)?;
        Ok(Self::from(bytes))
    }
}
