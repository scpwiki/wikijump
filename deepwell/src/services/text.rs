/*
 * services/text.rs
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

//! The text service, for storing large strings in the database.
//! For instance, page wikitext sources and compiled HTML outputs.
//!
//! It uses content-addressable storage, meaning that data is uniquely
//! identified by its hash.

use super::prelude::*;
use crate::hash::{k12_hash, TextHash, TEXT_HASH_LENGTH};

#[derive(Debug)]
pub struct TextService;

impl TextService {
    pub async fn get_optional(
        ctx: &ServiceContext,
        hash: &[u8],
    ) -> Result<Option<String>> {
        if hash.len() != TEXT_HASH_LENGTH {
            error!(
                "Text hash length does not match, should be {}, is {}",
                TEXT_HASH_LENGTH,
                hash.len(),
            );
            return Err(Error::BadRequest);
        }

        #[derive(Debug)]
        struct Row {
            contents: String,
        }

        let mutex = ctx.sqlx_transaction();
        let mut txn = mutex.lock().await;

        let contents =
            sqlx::query_as!(Row, r"SELECT contents FROM text WHERE hash = $1", hash)
                .fetch_optional(&mut **txn)
                .await?
                .map(|row| row.contents);

        Ok(contents)
    }

    #[inline]
    pub async fn get(ctx: &ServiceContext, hash: &[u8]) -> Result<String> {
        find_or_error!(Self::get_optional(ctx, hash), Text)
    }

    #[inline]
    pub async fn exists(ctx: &ServiceContext, hash: &[u8]) -> Result<bool> {
        Self::get_optional(ctx, hash)
            .await
            .map(|text| text.is_some())
    }

    /// Possibly retrieve text, if a flag is set.
    ///
    /// This utility conditionally retrieves the
    /// text given by the specified hash only
    /// if the flag `should_fetch` is true.
    /// Otherwise, it does no action, returning `None`.
    pub async fn get_maybe(
        ctx: &ServiceContext,
        should_fetch: bool,
        hash: &[u8],
    ) -> Result<Option<String>> {
        if should_fetch {
            let text = Self::get(ctx, hash).await?;
            Ok(Some(text))
        } else {
            Ok(None)
        }
    }

    /// Creates a text entry with this data, if it does not already exist.
    pub async fn create(ctx: &ServiceContext, contents: String) -> Result<TextHash> {
        let hash = k12_hash(contents.as_bytes());

        if !Self::exists(ctx, &hash).await? {
            let mutex = ctx.sqlx_transaction();
            let mut txn = mutex.lock().await;

            sqlx::query!(
                r"INSERT INTO text (hash, contents) VALUES ($1, $2)",
                &hash,
                contents,
            )
            .execute(&mut **txn)
            .await?;
        }

        Ok(hash)
    }

    /// Searches for any text rows which are unused.
    ///
    /// This is rare, but can happen when text is invalidated,
    /// such as rerendering pages.
    pub async fn prune(ctx: &ServiceContext) -> Result<()> {
        // All foreign keys of text.hash should have conditions here.
        // These foreign key constraints prevent us from deleting anything
        // actually used.

        let mutex = ctx.sqlx_transaction();
        let mut txn = mutex.lock().await;

        let rows_affected = sqlx::query!(
            r"
            DELETE FROM text
            WHERE hash NOT IN (SELECT wikitext_hash FROM page_revision)
            OR    hash NOT IN (SELECT compiled_hash FROM page_revision)
            OR    hash NOT IN (SELECT wikitext_hash FROM message_draft)
            OR    hash NOT IN (SELECT compiled_hash FROM message_draft)
            OR    hash NOT IN (SELECT wikitext_hash FROM message_record)
            OR    hash NOT IN (SELECT compiled_hash FROM message_record)
            "
        )
        // TODO add forum_post_revision
        .execute(&mut **txn)
        .await?
        .rows_affected();

        debug!("Pruned {rows_affected} unused text rows");
        Ok(())
    }
}
