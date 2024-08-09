/*
 *
 * services/vote/service.rs
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
    pub async fn add(
        ctx: &ServiceContext<'_>,
        CreateVote {
            page_id,
            user_id,
            value,
        }: CreateVote,
    ) -> Result<Option<PageVoteModel>> {
        let txn = ctx.seaorm_transaction();
        info!(
            "Casting new vote by user ID {} on page ID {} (value {})",
            user_id, page_id, value,
        );

        // Get previous vote, if any
        let key = GetVote { page_id, user_id };
        if let Some(vote) = Self::get_optional(ctx, key).await? {
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
    pub async fn get(ctx: &ServiceContext<'_>, key: GetVote) -> Result<PageVoteModel> {
        find_or_error!(Self::get_optional(ctx, key), Vote)
    }

    /// Gets any current vote for the current page and user.
    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        GetVote { page_id, user_id }: GetVote,
    ) -> Result<Option<PageVoteModel>> {
        let txn = ctx.seaorm_transaction();
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
    pub async fn action(
        ctx: &ServiceContext<'_>,
        key: GetVote,
        enable: bool,
        acting_user_id: i64,
    ) -> Result<PageVoteModel> {
        info!(
            "{} vote on {:?} (being done by {})",
            if enable { "Enabling" } else { "Disabling" },
            key,
            acting_user_id,
        );

        let txn = ctx.seaorm_transaction();
        let mut vote = Self::get(ctx, key).await?.into_active_model();

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
    pub async fn remove(ctx: &ServiceContext<'_>, key: GetVote) -> Result<PageVoteModel> {
        info!("Removing vote {key:?}");

        let txn = ctx.seaorm_transaction();
        let mut vote = Self::get(ctx, key).await?.into_active_model();
        vote.deleted_at = Set(Some(now()));

        let model = vote.update(txn).await?;
        Ok(model)
    }

    /// Gets votes for either a page or a user.
    ///
    /// The `start_id` argument gives the start ID to search from, exclusive.
    /// If `0`, then means "everything".
    ///
    /// The `deleted` argument:
    /// * If it is `Some(true)`, then it only returns pages which have been deleted.
    /// * If it is `Some(false)`, then it only returns pages which are extant.
    /// * If it is `None`, then it returns all pages regardless of deletion status are selected.
    pub async fn get_history(
        ctx: &ServiceContext<'_>,
        GetVoteHistory {
            kind,
            start_id,
            deleted,
            disabled,
            limit,
        }: GetVoteHistory,
    ) -> Result<Vec<PageVoteModel>> {
        let txn = ctx.seaorm_transaction();
        let condition = Self::build_history_condition(kind, start_id, deleted, disabled);

        let votes = PageVote::find()
            .filter(condition)
            .order_by_asc(page_vote::Column::PageVoteId)
            .limit(limit)
            .all(txn)
            .await?;

        Ok(votes)
    }

    /// Counts the number of historical votes for either a page or a user.
    ///
    /// See `get_history()` for more information.
    pub async fn count_history(
        ctx: &ServiceContext<'_>,
        CountVoteHistory {
            kind,
            start_id,
            deleted,
            disabled,
        }: CountVoteHistory,
    ) -> Result<u64> {
        let txn = ctx.seaorm_transaction();
        let condition = Self::build_history_condition(kind, start_id, deleted, disabled);

        let vote_count = PageVote::find().filter(condition).count(txn).await?;
        Ok(vote_count)
    }

    fn build_history_condition(
        kind: VoteHistoryKind,
        start_id: i64,
        deleted: Option<bool>,
        disabled: Option<bool>,
    ) -> Condition {
        let kind_condition = match kind {
            VoteHistoryKind::Page(page_id) => page_vote::Column::PageId.eq(page_id),
            VoteHistoryKind::User(user_id) => page_vote::Column::UserId.eq(user_id),
        };

        let deleted_condition = match deleted {
            Some(true) => Some(page_vote::Column::DeletedAt.is_not_null()),
            Some(false) => Some(page_vote::Column::DeletedAt.is_null()),
            None => None,
        };

        let disabled_condition = match disabled {
            Some(true) => Some(page_vote::Column::DisabledAt.is_not_null()),
            Some(false) => Some(page_vote::Column::DisabledAt.is_null()),
            None => None,
        };

        Condition::all()
            .add(page_vote::Column::PageVoteId.gt(start_id))
            .add(kind_condition)
            .add_option(deleted_condition)
            .add_option(disabled_condition)
    }
}
