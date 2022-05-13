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
use crate::web::FetchDirection;
use sea_orm::{prelude::DateTimeWithTimeZone, IntoActiveModel};

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
        tide::log::info!(
            "Casting new vote by user ID {} on page ID {} (value {})",
            user_id,
            page_id,
            value,
        );

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

    /// Enables or disables the vote specified.
    ///
    /// The action depends on the value of the boolean:
    /// * `value` being `true`: enable the vote
    /// * `value` being `false`: disable the vote
    pub async fn disable(
        ctx: &ServiceContext<'_>,
        input: GetVote,
        acting_user_id: i64,
        value: bool,
    ) -> Result<PageVoteModel> {
        tide::log::info!(
            "{} vote cast by user {} on page {} (being done by {})",
            if value { "Enabling" } else { "Disabling" },
            input.user_id,
            input.page_id,
            acting_user_id,
        );

        let txn = ctx.transaction();
        let mut vote = Self::get(ctx, input).await?.into_active_model();

        if value {
            // Enable, clear "disabled" field.
            vote.disabled_at = Set(None);
            vote.disabled_by = Set(None);
        } else {
            // Disable, set "disabled" field.
            vote.disabled_at = Set(Some(now()));
            vote.disabled_by = Set(Some(acting_user_id));
        }

        let model = vote.update(txn).await?;
        Ok(model)
    }

    /// Removes the vote specified.
    pub async fn remove(
        ctx: &ServiceContext<'_>,
        input: GetVote,
    ) -> Result<PageVoteModel> {
        tide::log::info!(
            "Removing vote cast by user {} on page {}",
            input.user_id,
            input.page_id,
        );

        let txn = ctx.transaction();
        let mut vote = Self::get(ctx, input).await?.into_active_model();
        vote.deleted_at = Set(Some(now()));

        let model = vote.update(txn).await?;
        Ok(model)
    }

    /// Gets the history of votes for either a page or a user.
    pub async fn get_history(
        ctx: &ServiceContext<'_>,
        kind: VoteHistoryKind,
        vote_start_date: Option<DateTimeWithTimeZone>,
        vote_direction: FetchDirection,
        vote_limit: u64,
    ) -> Result<Vec<PageVoteModel>> {
        let txn = ctx.transaction();

        let kind_condition = match kind {
            VoteHistoryKind::Page(page_id) => page_vote::Column::PageId.eq(page_id),
            VoteHistoryKind::User(user_id) => page_vote::Column::UserId.eq(user_id),
        };

        let vote_condition = vote_start_date.map(|start_date| match vote_direction {
            FetchDirection::Before => page_vote::Column::CreatedAt.lte(start_date),
            FetchDirection::After => page_vote::Column::CreatedAt.gte(start_date),
        });

        let votes = PageVote::find()
            .filter(
                Condition::all()
                    .add(kind_condition)
                    .add_option(vote_condition),
            )
            .order_by_asc(page_vote::Column::PageVoteId)
            .limit(vote_limit)
            .all(txn)
            .await?;

        Ok(votes)
    }
}
