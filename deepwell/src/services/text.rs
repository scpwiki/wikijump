/*
 * services/text.rs
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

//! The text service, for storing large strings in the database.
//! For instance, page wikitext sources and compiled HTML outputs.
//!
//! It uses content-addressable storage, meaning that data is uniquely
//! identified by its hash.

use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::hash::{TEXT_HASH_LENGTH, TextHash, k12_hash};
use crate::models::forum_post_revision::{self, Entity as ForumPostRevision};
use crate::models::message_draft::{self, Entity as MessageDraft};
use crate::models::message_record::{self, Entity as MessageRecord};
use crate::models::page_revision::{self, Entity as PageRevision};
use crate::models::text::{self, Entity as Text};
use crate::services::ServiceContext;
use paste::paste;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, Condition, DeleteResult, EntityTrait, QueryFilter, Set};
use sea_query::Query;

#[derive(Debug)]
pub struct TextService;

impl TextService {
    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        hash: &[u8],
    ) -> Result<Option<String>> {
        if hash.len() != TEXT_HASH_LENGTH {
            error!(
                "Text hash length does not match, should be {}, is {}",
                TEXT_HASH_LENGTH,
                hash.len(),
            );
            bail!(Error::new(
                format!(
                    "failed to get text entry, hash should be {} bytes, but is {} bytes",
                    TEXT_HASH_LENGTH,
                    hash.len(),
                ),
                ErrorType::BadRequest,
            ));
        }

        let txn = ctx.transaction();
        let contents = Text::find()
            .filter(text::Column::Hash.eq(hash))
            .one(txn)
            .await
            .or_raise(|| {
                Error::new("failed to get optional text entry", ErrorType::Text)
            })?
            .map(|model| model.contents);

        Ok(contents)
    }

    #[inline]
    pub async fn get(ctx: &ServiceContext<'_>, hash: &[u8]) -> Result<String> {
        find_or_error!(Self::get_optional(ctx, hash), "text entry", Text)
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
    pub async fn get_conditional(
        ctx: &ServiceContext<'_>,
        should_fetch: bool,
        hash: &[u8],
    ) -> Result<Option<String>> {
        if should_fetch {
            let text = Self::get(ctx, hash).await.or_raise(|| {
                Error::new("failed to conditionally get text entry", ErrorType::Text)
            })?;
            Ok(Some(text))
        } else {
            Ok(None)
        }
    }

    /// Syntactic sugar for `Option<[u8]>` → `Option<String>`.
    ///
    /// This is effectively equivalent to `Option::map()` for `TextService::get()`,
    /// but because it is `async` and returns `Result`, the actual equivalent code
    /// would be:
    /// ```rs
    /// # fn get_option(ctx: &ServiceContext<'_>, hash: Option<&[u8]>) -> Result<Option<String>> {
    /// match hash {
    ///     None => Ok(None),
    ///     Some(hash) => {
    ///         let text = TextService::get(ctx, hash).await?;
    ///         Ok(Some(text))
    ///     }
    /// }
    /// # }
    /// ```
    ///
    /// Put another way, if `hash` is `Some(_)` then the result will always be `Some(_)`,
    /// and if `hash` is `None` then the result will always be `None`.
    ///
    /// Not to be confused with the following methods:
    /// * `get_optional()` &mdash; Returns `None` if the text doesn't exist instead of an error.
    /// * `get_conditional()` &mdash; Doesn't accept an `Option` hash reference.
    /// * `get_conditional_option()` &mdash; Combination of `get_conditional()` and `get_option()`.
    pub async fn get_option<B: AsRef<[u8]>>(
        ctx: &ServiceContext<'_>,
        hash: &Option<B>,
    ) -> Result<Option<String>> {
        match hash {
            None => Ok(None),
            Some(hash) => {
                let hash = hash.as_ref();
                let text = Self::get(ctx, hash).await.or_raise(|| {
                    Error::new("failed to get optional text entry", ErrorType::Text)
                })?;

                Ok(Some(text))
            }
        }
    }

    /// A combination of `get_conditional()` and `get_option()`.
    ///
    /// That is, it will fetch a text if and only if:
    /// * `should_fetch` is true
    /// * `hash` is `Some(_)`
    ///
    /// If both conditions are met, then it is identical to returning
    /// the results of `TextService::get()` in a `Some(_)`.
    pub async fn get_conditional_option<B: AsRef<[u8]>>(
        ctx: &ServiceContext<'_>,
        should_fetch: bool,
        hash: &Option<B>,
    ) -> Result<Option<String>> {
        if should_fetch {
            Self::get_option(ctx, hash).await
        } else {
            Ok(None)
        }
    }

    /// Creates a text entry with this data, if it does not already exist.
    pub async fn create(ctx: &ServiceContext<'_>, contents: String) -> Result<TextHash> {
        let make_error =
            || Error::new("failed to create new text entry", ErrorType::Text);

        let txn = ctx.transaction();
        let hash = k12_hash(contents.as_bytes());
        let exists = Self::exists(ctx, &hash).await.or_raise(make_error)?;

        if !exists {
            let model = text::ActiveModel {
                hash: Set(hash.to_vec()),
                contents: Set(contents),
            };

            Text::insert(model)
                .on_conflict(
                    OnConflict::column(text::Column::Hash)
                        .do_nothing()
                        .to_owned(),
                )
                .exec_without_returning(txn)
                .await
                .or_raise(make_error)?;
        }

        Ok(hash)
    }

    /// Searches for any text rows which are unused.
    ///
    /// This is rare, but can happen when text is invalidated,
    /// such as rerendering pages.
    pub async fn prune(ctx: &ServiceContext<'_>) -> Result<()> {
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
        let txn = ctx.transaction();
        let DeleteResult { rows_affected, .. } = Text::delete_many()
            .filter(
                Condition::all()
                    .add(not_in_column!(
                        PageRevision,
                        page_revision::Column::WikitextHash,
                    ))
                    .add(not_in_column!(
                        PageRevision,
                        page_revision::Column::CompiledBodyHtmlHash,
                    ))
                    .add(not_in_column!(
                        PageRevision,
                        page_revision::Column::CompiledTopBarHtmlHash,
                    ))
                    .add(not_in_column!(
                        PageRevision,
                        page_revision::Column::CompiledSideBarHtmlHash,
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
                    ))
                    .add(not_in_column!(
                        ForumPostRevision,
                        forum_post_revision::Column::WikitextHash,
                    ))
                    .add(not_in_column!(
                        ForumPostRevision,
                        forum_post_revision::Column::CompiledHtmlHash,
                    )),
            )
            .exec(txn)
            .await
            .or_raise(|| {
                Error::new("failed to prune unused text entries", ErrorType::Text)
            })?;

        debug!("Pruned {rows_affected} unused text rows");
        Ok(())
    }
}
