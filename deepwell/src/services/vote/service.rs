/*
 *
 * services/vote/service.rs
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
        rating_system: &str,
    ) -> Result<Option<PageVoteModel>> {
        let txn = ctx.transaction();
        info!(
            "Casting new vote by user ID {} on page ID {} (value {})",
            user_id, page_id, value,
        );

        let make_error = || {
            Error::new(
                format!(
                    "failed to add new vote by user ID {} on page ID {} (value {})",
                    user_id, page_id, value,
                ),
                ErrorType::PageVote,
            )
        };

        // Get previous vote, if any
        let key = GetVote { page_id, user_id };
        if let Some(vote) = Self::get_optional(ctx, key, rating_system)
            .await
            .or_raise(make_error)?
        {
            // If it's the same value, no new vote is needed
            if vote.value == value {
                return Ok(None);
            }

            // Otherwise, delete so we can insert the new one
            let mut model = vote.into_active_model();
            model.deleted_at = Set(Some(now()));
            model.update(txn).await.or_raise(make_error)?;
        }

        // Insert the new vote
        let model = page_vote::ActiveModel {
            page_id: Set(page_id),
            user_id: Set(user_id),
            rating_system: Set(rating_system.to_owned()),
            value: Set(value),
            ..Default::default()
        };

        let vote = model.insert(txn).await.or_raise(make_error)?;
        Ok(Some(vote))
    }

    #[inline]
    pub async fn get(
        ctx: &ServiceContext<'_>,
        key: GetVote,
        rating_system: &str,
    ) -> Result<PageVoteModel> {
        find_or_error!(Self::get_optional(ctx, key, rating_system), "vote", Vote)
    }

    /// Gets any current vote for the current page and user.
    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        GetVote { page_id, user_id }: GetVote,
        rating_system: &str,
    ) -> Result<Option<PageVoteModel>> {
        let txn = ctx.transaction();
        let vote = PageVote::find()
            .filter(
                Condition::all()
                    .add(page_vote::Column::PageId.eq(page_id))
                    .add(page_vote::Column::UserId.eq(user_id))
                    .add(page_vote::Column::RatingSystem.eq(rating_system))
                    .add(page_vote::Column::DeletedAt.is_null()),
            )
            .one(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to get individual vote by user ID {} on page ID {}",
                        user_id, page_id,
                    ),
                    ErrorType::PageVote,
                )
            })?;

        Ok(vote)
    }

    /// Enables or disables the vote specified.
    pub async fn action(
        ctx: &ServiceContext<'_>,
        key: GetVote,
        rating_system: &str,
        enable: bool,
        acting_user_id: i64,
    ) -> Result<PageVoteModel> {
        info!(
            "{} vote on {:?} (performed by user ID {})",
            if enable { "Enabling" } else { "Disabling" },
            key,
            acting_user_id,
        );

        let make_error = || {
            Error::new(
                format!(
                    "failed to {} vote on {:?} (performed by user ID {})",
                    if enable { "enable" } else { "disable" },
                    key,
                    acting_user_id,
                ),
                ErrorType::PageVote,
            )
        };

        let txn = ctx.transaction();
        let mut vote = Self::get(ctx, key, rating_system)
            .await
            .or_raise(make_error)?
            .into_active_model();

        if enable {
            // Clear "disabled" field.
            vote.disabled_at = Set(None);
            vote.disabled_by = Set(None);
        } else {
            // Set "disabled" field.
            vote.disabled_at = Set(Some(now()));
            vote.disabled_by = Set(Some(acting_user_id));
        }

        let model = vote.update(txn).await.or_raise(make_error)?;
        Ok(model)
    }

    /// Removes the vote specified.
    pub async fn remove(
        ctx: &ServiceContext<'_>,
        key: GetVote,
        rating_system: &str,
    ) -> Result<PageVoteModel> {
        info!("Removing vote {key:?}");

        let make_error = || {
            Error::new(
                format!(
                    "failed to remove vote by user ID {} on page ID {}",
                    key.user_id, key.page_id,
                ),
                ErrorType::PageVote,
            )
        };

        let txn = ctx.transaction();
        let mut vote = Self::get(ctx, key, rating_system)
            .await
            .or_raise(make_error)?
            .into_active_model();

        vote.deleted_at = Set(Some(now()));

        let model = vote.update(txn).await.or_raise(make_error)?;
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
        rating_system: Option<&str>,
    ) -> Result<Vec<PageVoteModel>> {
        let txn = ctx.transaction();
        let condition = Self::build_history_condition(
            kind,
            start_id,
            deleted,
            disabled,
            rating_system,
        );

        let votes = PageVote::find()
            .filter(condition)
            .order_by_asc(page_vote::Column::PageVoteId)
            .limit(limit)
            .all(txn)
            .await
            .or_raise(|| Self::build_history_error(kind, start_id, deleted, disabled))?;

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
        rating_system: Option<&str>,
    ) -> Result<u64> {
        let txn = ctx.transaction();
        let condition = Self::build_history_condition(
            kind,
            start_id,
            deleted,
            disabled,
            rating_system,
        );

        let vote_count = PageVote::find()
            .filter(condition)
            .count(txn)
            .await
            .or_raise(|| Self::build_history_error(kind, start_id, deleted, disabled))?;

        Ok(vote_count)
    }

    fn build_history_condition(
        kind: VoteHistoryKind,
        start_id: i64,
        deleted: Option<bool>,
        disabled: Option<bool>,
        rating_system: Option<&str>,
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

        let mut condition = Condition::all()
            .add(page_vote::Column::PageVoteId.gt(start_id))
            .add(kind_condition)
            .add_option(deleted_condition)
            .add_option(disabled_condition);
        if let Some(rating_system) = rating_system {
            condition = condition.add(page_vote::Column::RatingSystem.eq(rating_system));
        }
        condition
    }

    fn build_history_error(
        kind: VoteHistoryKind,
        start_id: i64,
        deleted: Option<bool>,
        disabled: Option<bool>,
    ) -> Error {
        let mut message = str!("failed to get vote history for ");

        match deleted {
            Some(true) => message.push_str("deleted "),
            Some(false) => message.push_str("current "),
            None => (),
        }

        match disabled {
            Some(true) => message.push_str("disabled "),
            Some(false) => message.push_str("enabled "),
            None => (),
        }

        match kind {
            VoteHistoryKind::Page(page_id) => {
                str_write!(&mut message, "votes on page ID {}", page_id)
            }
            VoteHistoryKind::User(user_id) => {
                str_write!(&mut message, "votes cast by user ID {}", user_id)
            }
        }

        str_write!(&mut message, ", starting at vote ID {}", start_id);

        Error::new(message, ErrorType::PageVote)
    }
}
