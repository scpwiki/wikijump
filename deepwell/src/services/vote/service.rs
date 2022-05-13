/*
 * services/vote/service.rs
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

use super::prelude::*;
use crate::models::page_vote::{self, Entity as PageVote, Model as PageVoteModel};
use sea_orm::IntoActiveModel;

#[derive(Debug)]
pub struct VoteService;

impl VoteService {
    /// Creates a vote with the given value.
    ///
    /// # Returns
    /// Returns `Some` if a new vote was created,
    /// and `None` if the it already exists.
    pub async fn create(
        ctx: &ServiceContext<'_>,
        CreateVote {
            page_id,
            user_id,
            value,
        }: CreateVote,
    ) -> Result<Option<PageVoteModel>> {
        let txn = ctx.transaction();

        // Get previous vote, if any
        if let Some(vote) = Self::get_optional(ctx, GetVote { page_id, user_id }).await? {
            // If it's the same value, no new vote is needed
            if vote.value == value {
                return Ok(None);
            }

            // Otherwise, delete so we can insert the new one
            vote.delete(txn).await?;
        }

        // Insert the new vote
        let model = page_vote::ActiveModel {
            page_id: Set(page_id),
            user_id: Set(user_id),
            value: Set(value),
            ..Default::default()
        };

        let vote = model.insert(txn).await?;
        Ok(Some(vote))
    }

    #[inline]
    pub async fn exists(ctx: &ServiceContext<'_>, input: GetVote) -> Result<bool> {
        Self::get_optional(ctx, input)
            .await
            .map(|vote| vote.is_some())
    }

    pub async fn get(ctx: &ServiceContext<'_>, input: GetVote) -> Result<PageVoteModel> {
        match Self::get_optional(ctx, input).await? {
            Some(vote) => Ok(vote),
            None => Err(Error::NotFound),
        }
    }

    /// Gets any current vote for the current page and user.
    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        GetVote { page_id, user_id }: GetVote,
    ) -> Result<Option<PageVoteModel>> {
        let txn = ctx.transaction();
        let vote = PageVote::find()
            .filter(
                Condition::all()
                    .add(page_vote::Column::PageId.eq(page_id))
                    .add(page_vote::Column::UserId.eq(user_id))
                    .add(page_vote::Column::DeletedAt.is_null()),
            )
            .one(txn)
            .await?;

        Ok(vote)
    }

    /// Removes the vote specified.
    pub async fn remove(
        ctx: &ServiceContext<'_>,
        input: GetVote,
    ) -> Result<PageVoteModel> {
        let txn = ctx.transaction();

        let mut vote = Self::get(ctx, input).await?.into_active_model();
        vote.deleted_at = Set(Some(now()));

        let model = vote.update(txn).await?;
        Ok(model)
    }
}
