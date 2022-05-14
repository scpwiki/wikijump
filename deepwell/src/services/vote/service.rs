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
        let reference = VoteReference::Pair(GetVote { page_id, user_id });
        if let Some(vote) = Self::get_optional(ctx, reference).await? {
            // If it's the same value, no new vote is needed
            if vote.value == value {
                return Ok(None);
            }

            // Otherwise, delete so we can insert the new one
            let mut model = vote.into_active_model();
            model.deleted_at = Set(Some(now()));
            model.update(txn).await?;
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
    pub async fn exists(
        ctx: &ServiceContext<'_>,
        reference: VoteReference,
    ) -> Result<bool> {
        Self::get_optional(ctx, reference)
            .await
            .map(|vote| vote.is_some())
    }

    pub async fn get(
        ctx: &ServiceContext<'_>,
        reference: VoteReference,
    ) -> Result<PageVoteModel> {
        match Self::get_optional(ctx, reference).await? {
            Some(vote) => Ok(vote),
            None => Err(Error::NotFound),
        }
    }

    /// Gets any current vote for the current page and user.
    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        reference: VoteReference,
    ) -> Result<Option<PageVoteModel>> {
        let txn = ctx.transaction();

        let condition = match reference {
            VoteReference::Id(vote_id) => Condition::all()
                .add(page_vote::Column::PageVoteId.eq(vote_id))
                .add(page_vote::Column::DeletedAt.is_null()),
            VoteReference::Pair(GetVote { page_id, user_id }) => Condition::all()
                .add(page_vote::Column::PageId.eq(page_id))
                .add(page_vote::Column::UserId.eq(user_id))
                .add(page_vote::Column::DeletedAt.is_null()),
        };

        let vote = PageVote::find().filter(condition).one(txn).await?;

        Ok(vote)
    }

    /// Enables or disables the vote specified.
    pub async fn action(
        ctx: &ServiceContext<'_>,
        reference: VoteReference,
        enable: bool,
        acting_user_id: i64,
    ) -> Result<PageVoteModel> {
        tide::log::info!(
            "{} vote on {:?} (being done by {})",
            if enable { "Enabling" } else { "Disabling" },
            reference,
            acting_user_id,
        );

        let txn = ctx.transaction();
        let mut vote = Self::get(ctx, reference).await?.into_active_model();

        if enable {
            // Clear "disabled" field.
            vote.disabled_at = Set(None);
            vote.disabled_by = Set(None);
        } else {
            // Set "disabled" field.
            vote.disabled_at = Set(Some(now()));
            vote.disabled_by = Set(Some(acting_user_id));
        }

        let model = vote.update(txn).await?;
        Ok(model)
    }

    /// Removes the vote specified.
    pub async fn remove(
        ctx: &ServiceContext<'_>,
        reference: VoteReference,
    ) -> Result<PageVoteModel> {
        tide::log::info!("Removing vote {reference:?}");

        let txn = ctx.transaction();
        let mut vote = Self::get(ctx, reference).await?.into_active_model();
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
        let condition =
            Self::build_history_condition(kind, vote_start_date, vote_direction);

        let votes = PageVote::find()
            .filter(condition)
            .order_by_asc(page_vote::Column::PageVoteId)
            .limit(vote_limit)
            .all(txn)
            .await?;

        Ok(votes)
    }

    /// Counts the number of votes for either a page or a user.
    pub async fn count_history(
        ctx: &ServiceContext<'_>,
        kind: VoteHistoryKind,
        vote_start_date: Option<DateTimeWithTimeZone>,
        vote_direction: FetchDirection,
    ) -> Result<usize> {
        let txn = ctx.transaction();
        let condition =
            Self::build_history_condition(kind, vote_start_date, vote_direction);

        let vote_count = PageVote::find().filter(condition).count(txn).await?;
        Ok(vote_count)
    }

    fn build_history_condition(
        kind: VoteHistoryKind,
        start_date: Option<DateTimeWithTimeZone>,
        direction: FetchDirection,
    ) -> Condition {
        let kind_condition = match kind {
            VoteHistoryKind::Page(page_id) => page_vote::Column::PageId.eq(page_id),
            VoteHistoryKind::User(user_id) => page_vote::Column::UserId.eq(user_id),
        };

        let vote_condition = start_date.map(|start_date| match direction {
            FetchDirection::Before => page_vote::Column::CreatedAt.lte(start_date),
            FetchDirection::After => page_vote::Column::CreatedAt.gte(start_date),
        });

        Condition::all()
            .add(kind_condition)
            .add_option(vote_condition)
    }
}
