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
use crate::models::message_draft::{self, Entity as MessageDraft};
use crate::models::message_record::{self, Entity as MessageRecord};
use crate::models::page_revision::{self, Entity as PageRevision};
use crate::models::text::{self, Entity as Text};
use sea_query::Query;

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
            sqlx::query_as!(Row, r"SELECT contents FROM text WHERE hash = $1", hash,)
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
        let txn = ctx.seaorm_transaction();
        let hash = k12_hash(contents.as_bytes());

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
    pub async fn prune(ctx: &ServiceContext) -> Result<()> {
        macro_rules! not_in_column {
            ($table:expr, $column:expr $(,)?) => {
                text::Column::Hash.not_in_subquery(
                    Query::select().column($column).from($table).to_owned(),
                )
            };
        }

        // All foreign keys of text.hash should have conditions here.
        // These foreign key constraints prevent us from deleting anything
        // actually used.
        let txn = ctx.seaorm_transaction();
        let DeleteResult { rows_affected, .. } = Text::delete_many()
            .filter(
                Condition::all()
                    .add(not_in_column!(
                        PageRevision,
                        page_revision::Column::WikitextHash,
                    ))
                    .add(not_in_column!(
                        PageRevision,
                        page_revision::Column::CompiledHash,
                    ))
                    .add(not_in_column!(
                        MessageDraft,
                        message_draft::Column::WikitextHash,
                    ))
                    .add(not_in_column!(
                        MessageDraft,
                        message_draft::Column::CompiledHash,
                    ))
                    .add(not_in_column!(
                        MessageRecord,
                        message_record::Column::WikitextHash,
                    ))
                    .add(not_in_column!(
                        MessageRecord,
                        message_record::Column::CompiledHash,
                    )),
                // TODO add forum_post_revision
            )
            .exec(txn)
            .await?;

        debug!("Pruned {rows_affected} unused text rows");
        Ok(())
    }
}
