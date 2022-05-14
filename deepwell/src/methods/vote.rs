/*
 * methods/vote.rs
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
use crate::services::vote::{CreateVote, GetVote, VoteAction, VoteReference};
use crate::web::{FetchDetailsQuery, FetchLimitQuery};
use serde::Serialize;

pub async fn vote_head_direct(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let vote_id = req.param("vote_id")?.parse()?;
    let reference = VoteReference::Id(vote_id);
    tide::log::info!("Checking existence of vote ID {vote_id}");

    let exists = VoteService::exists(&ctx, reference).await.to_api()?;
    txn.commit().await?;
    exists_status(exists)
}

pub async fn vote_get_direct(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let vote_id = req.param("vote_id")?.parse()?;
    let reference = VoteReference::Id(vote_id);
    tide::log::info!("Getting vote ID {vote_id}");

    let vote = VoteService::get(&ctx, reference).await.to_api()?;
    txn.commit().await?;
    build_vote_response(&vote, StatusCode::Ok)
}

pub async fn vote_head(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: GetVote = req.body_json().await?;
    let reference = VoteReference::Pair(input);

    tide::log::info!(
        "Checking existence of vote cast by {} on page {}",
        input.user_id,
        input.page_id,
    );

    let exists = VoteService::exists(&ctx, reference).await.to_api()?;
    txn.commit().await?;
    exists_status(exists)
}

pub async fn vote_get(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: GetVote = req.body_json().await?;
    let reference = VoteReference::Pair(input);

    tide::log::info!(
        "Getting vote cast by {} on page {}",
        input.user_id,
        input.page_id,
    );

    let model = VoteService::get(&ctx, reference).await.to_api()?;
    txn.commit().await?;
    build_vote_response(&model, StatusCode::Ok)
}

pub async fn vote_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: CreateVote = req.body_json().await?;

    tide::log::info!(
        "Casting vote cast by {} on page {}",
        input.user_id,
        input.page_id,
    );

    let created = VoteService::create(&ctx, input).await.to_api()?;
    txn.commit().await?;
    match created {
        Some(model) => build_vote_response(&model, StatusCode::Created),
        None => Ok(Response::new(StatusCode::NoContent)),
    }
}

pub async fn vote_delete(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: GetVote = req.body_json().await?;
    let reference = VoteReference::Pair(input);

    tide::log::info!(
        "Removing vote cast by {} on page {}",
        input.user_id,
        input.page_id,
    );

    let vote = VoteService::remove(&ctx, reference).await.to_api()?;
    txn.commit().await?;
    build_vote_response(&vote, StatusCode::Ok)
}

pub async fn vote_action(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let VoteAction {
        page_id,
        user_id,
        enable,
        acting_user_id,
    } = req.body_json().await?;

    let reference = VoteReference::Pair(GetVote { page_id, user_id });
    let vote = VoteService::action(&ctx, reference, enable, acting_user_id)
        .await
        .to_api()?;

    txn.commit().await?;
    build_vote_response(&vote, StatusCode::Ok)
}

pub async fn vote_range_get(mut req: ApiRequest) -> ApiResponse {
    todo!()
}

pub async fn vote_count_get(mut req: ApiRequest) -> ApiResponse {
    todo!()
}

fn build_vote_response<T: Serialize>(data: &T, status: StatusCode) -> ApiResponse {
    let body = Body::from_json(data)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
