/*
 * services/text.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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

//! The text service, for storing large strings in the database.
//! For instance, page wikitext sources and compiled HTML outputs.
//!
//! It uses content-addressable storage, meaning that data is uniquely
//! identified by its hash.

use super::prelude::*;
use crate::models::text::{self, Entity as Text};
use sha2::{Digest, Sha512};

/// The expected length of a hash digest.
///
/// This is the output length for SHA-512 in bytes.
pub const HASH_LENGTH: usize = 64;

/// The array type for a hash digest.
pub type Hash = [u8; 64];

#[derive(Debug)]
pub struct TextService;

impl TextService {
    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        hash: &[u8],
    ) -> Result<Option<String>> {
        assert_eq!(hash.len(), HASH_LENGTH);

        let txn = ctx.transaction();
        let contents = Text::find()
            .filter(text::Column::Hash.eq(hash))
            .one(txn)
            .await?
            .map(|model| model.contents);

        Ok(contents)
    }

    pub async fn get(ctx: &ServiceContext<'_>, hash: &[u8]) -> Result<String> {
        match Self::get_optional(ctx, hash).await? {
            Some(string) => Ok(string),
            None => Err(Error::NotFound),
        }
    }

    #[inline]
    pub async fn exists(ctx: &ServiceContext<'_>, hash: &[u8]) -> Result<bool> {
        Self::get_optional(ctx, hash)
            .await
            .map(|text| text.is_some())
    }

    pub async fn create(ctx: &ServiceContext<'_>, contents: String) -> Result<Hash> {
        let txn = ctx.transaction();
        let hash = Self::hash(&contents);

        if !Self::exists(ctx, &hash).await? {
            let model = text::ActiveModel {
                hash: Set(hash.to_vec()),
                contents: Set(contents),
            };

            Text::insert(model).exec(txn).await?;
        }

        Ok(hash)
    }

    pub fn hash(contents: &str) -> Hash {
        // Perform hash
        let mut hasher = Sha512::new();
        hasher.update(contents.as_bytes());
        let result = hasher.finalize();

        // Copy data into regular Rust array
        let mut bytes = [0; 64];
        bytes.copy_from_slice(&result);
        bytes
    }
}
