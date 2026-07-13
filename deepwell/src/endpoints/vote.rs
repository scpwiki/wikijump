/*
 * endpoints/vote.rs
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
use crate::models::page_vote::Model as PageVoteModel;
use crate::services::MutationAuthorization;
use crate::services::vote::{
    CountVoteHistory, CreateVote, GetVote, GetVoteHistory, VoteAction,
};

pub async fn vote_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<PageVoteModel>> {
    let input: GetVote = parse!(params, PageVote);
    let page_id = input.page_id;
    let user_id = input.user_id;

    info!(
        "Getting vote cast by user ID {} on page ID {}",
        user_id, page_id,
    );

    VoteService::get_optional(ctx, input).await.or_raise(|| {
        Error::new(
            format!(
                "failed to get vote cast by user ID {} on page ID {}",
                user_id, page_id,
            ),
            ErrorType::PageVote,
        )
    })
}

pub async fn vote_set(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<PageVoteModel>> {
    let input: CreateVote = parse!(params, PageVote);
    let page_id = input.page_id;
    let user_id = input.user_id;
    MutationAuthorization::require_matching_actor(ctx, user_id, "cast a page vote")?;

    info!("Casting vote cast by {} on page {}", user_id, page_id,);

    VoteService::add(ctx, input).await.or_raise(|| {
        Error::new(
            format!(
                "failed to set vote on page ID {} from user ID {}",
                page_id, user_id,
            ),
            ErrorType::PageVote,
        )
    })
}

pub async fn vote_remove(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<PageVoteModel> {
    let input: GetVote = parse!(params, PageVote);
    let page_id = input.page_id;
    let user_id = input.user_id;
    MutationAuthorization::require_matching_actor(ctx, user_id, "remove a page vote")?;

    info!("Removing vote cast by {} on page {}", user_id, page_id,);

    VoteService::remove(ctx, input).await.or_raise(|| {
        Error::new(
            format!(
                "failed to remove vote on page ID {} from user ID {}",
                page_id, user_id,
            ),
            ErrorType::PageVote,
        )
    })
}

pub async fn vote_action(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<PageVoteModel> {
    let VoteAction {
        page_id,
        user_id,
        enable,
        acting_user_id,
    } = parse!(params, PageVote);
    let actor_user_id =
        MutationAuthorization::require_platform_staff(ctx, "moderate a page vote")?;
    if acting_user_id != actor_user_id {
        return Err(Error::new(
            "request actor does not match the page vote moderator attribution",
            ErrorType::PermissionDenied,
        )
        .into());
    }

    // e.g. enable or disable a vote
    let key = GetVote { page_id, user_id };
    VoteService::action(ctx, key, enable, acting_user_id)
        .await
        .or_raise(|| Error::new(
            format!(
                "failed to {} vote on page ID {} for user ID {} (performed by user ID {})",
                if enable { "enable" } else { "disable" },
                page_id,
                user_id,
                acting_user_id,
            ),
            ErrorType::PageVote,
        )
    )
}

pub async fn vote_list_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<PageVoteModel>> {
    let input: GetVoteHistory = parse!(params);

    VoteService::get_history(ctx, input)
        .await
        .or_raise(|| Error::new("failed to list votes", ErrorType::PageVote))
}

pub async fn vote_list_count(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<u64> {
    let input: CountVoteHistory = parse!(params);

    VoteService::count_history(ctx, input)
        .await
        .or_raise(|| Error::new("failed to get vote count", ErrorType::PageVote))
}
