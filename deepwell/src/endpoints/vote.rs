/*
 * endpoints/vote.rs
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
use crate::models::page_vote::Model as PageVoteModel;
use crate::services::vote::{
    CountVoteHistory, CreateVote, GetVote, GetVoteHistory, VoteAction,
};

pub async fn vote_get(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<Option<PageVoteModel>> {
    let input: GetVote = params.parse()?;

    info!(
        "Getting vote cast by {} on page {}",
        input.user_id, input.page_id,
    );

    VoteService::get_optional(ctx, input).await
}

pub async fn vote_set(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<Option<PageVoteModel>> {
    let input: CreateVote = params.parse()?;

    info!(
        "Casting vote cast by {} on page {}",
        input.user_id, input.page_id,
    );

    VoteService::add(ctx, input).await
}

pub async fn vote_remove(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<PageVoteModel> {
    let input: GetVote = params.parse()?;

    info!(
        "Removing vote cast by {} on page {}",
        input.user_id, input.page_id,
    );

    VoteService::remove(ctx, input).await
}

pub async fn vote_action(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<PageVoteModel> {
    let VoteAction {
        page_id,
        user_id,
        enable,
        acting_user_id,
    } = params.parse()?;

    let key = GetVote { page_id, user_id };
    VoteService::action(ctx, key, enable, acting_user_id).await
}

pub async fn vote_list_get(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<Vec<PageVoteModel>> {
    let input: GetVoteHistory = params.parse()?;
    VoteService::get_history(ctx, input).await
}

pub async fn vote_list_count(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<u64> {
    let input: CountVoteHistory = params.parse()?;
    VoteService::count_history(ctx, input).await
}
