/*
 * services/text.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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
use crate::hash::{sha512_hash, Hash, HASH_LENGTH};
use crate::models::text::{self, Entity as Text};

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

    /// Possibly retrieve text, if a flag is set.
    ///
    /// This utility conditionally retrieves the
    /// text given by the specified hash only
    /// if the flag `should_fetch` is true.
    /// Otherwise, it does no action, returning `None`.
    pub async fn get_maybe(
        ctx: &ServiceContext<'_>,
        should_fetch: bool,
        hash: &[u8],
    ) -> Result<Option<String>> {
        if should_fetch {
            let text = TextService::get(ctx, hash).await?;
            Ok(Some(text))
        } else {
            Ok(None)
        }
    }

    /// Creates a text entry with this data, if it does not already exist.
    pub async fn create(ctx: &ServiceContext<'_>, contents: String) -> Result<Hash> {
        let txn = ctx.transaction();
        let hash = sha512_hash(contents.as_bytes());

        if !Self::exists(ctx, &hash).await? {
            let model = text::ActiveModel {
                hash: Set(hash.to_vec()),
                contents: Set(contents),
            };

            Text::insert(model).exec(txn).await?;
        }

        Ok(hash)
    }

    /// Searches for any text rows which are unused.
    ///
    /// This is rare, but can happen when text is invalidated,
    /// such as rerendering pages.
    #[allow(dead_code)]
    pub async fn prune(_ctx: &ServiceContext<'_>) -> Result<()> {
        todo!();

        // Postgres Query:
        //
        // SELECT hash
        // FROM text
        // WHERE hash NOT IN (
        //     SELECT wikitext_hash AS hash
        //     FROM page_revision
        //     UNION
        //     SELECT compiled_hash AS hash
        //     FROM page_revision
        // )
    }
}
