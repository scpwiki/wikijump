/*
 * endpoints/vote.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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
use crate::services::vote::{
    CountVoteHistory, CreateVote, GetVote, GetVoteHistory, VoteAction,
};
use serde::Serialize;

pub async fn vote_retrieve(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let input: GetVote = req.body_json().await?;

    tide::log::info!(
        "Getting vote cast by {} on page {}",
        input.user_id,
        input.page_id,
    );

    let model = VoteService::get(&ctx, input).await?;
    txn.commit().await?;
    build_vote_response(&model, StatusCode::Ok)
}

pub async fn vote_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let input: CreateVote = req.body_json().await?;

    tide::log::info!(
        "Casting vote cast by {} on page {}",
        input.user_id,
        input.page_id,
    );

    let created = VoteService::add(&ctx, input).await?;
    txn.commit().await?;
    match created {
        Some(model) => build_vote_response(&model, StatusCode::Created),
        None => Ok(Response::new(StatusCode::NoContent)),
    }
}

pub async fn vote_delete(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let input: GetVote = req.body_json().await?;

    tide::log::info!(
        "Removing vote cast by {} on page {}",
        input.user_id,
        input.page_id,
    );

    let vote = VoteService::remove(&ctx, input).await?;
    txn.commit().await?;
    build_vote_response(&vote, StatusCode::Ok)
}

pub async fn vote_action(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let VoteAction {
        page_id,
        user_id,
        enable,
        acting_user_id,
    } = req.body_json().await?;

    let key = GetVote { page_id, user_id };
    let vote = VoteService::action(&ctx, key, enable, acting_user_id).await?;

    txn.commit().await?;
    build_vote_response(&vote, StatusCode::Ok)
}

pub async fn vote_list_retrieve(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let input: GetVoteHistory = req.body_json().await?;
    let votes = VoteService::get_history(&ctx, input).await?;

    txn.commit().await?;
    build_vote_response(&votes, StatusCode::Ok)
}

pub async fn vote_count_retrieve(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let input: CountVoteHistory = req.body_json().await?;
    let count = VoteService::count_history(&ctx, input).await?;

    txn.commit().await?;
    build_vote_response(&count, StatusCode::Ok)
}

fn build_vote_response<T: Serialize>(data: &T, status: StatusCode) -> ApiResponse {
    let body = Body::from_json(data)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
