/*
 * endpoints/message.rs
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
use crate::services::message::{
    CreateMessageDraft, DeleteMessageDraft, SendMessageDraft, UpdateMessageDraft,
};

pub async fn message_draft_create(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let input: CreateMessageDraft = req.body_json().await?;
    tide::log::info!("Creating new message draft for user ID {}", input.user_id);

    let output = MessageService::create_draft(&ctx, input).await?;
    txn.commit().await?;
    build_json_response(&output, StatusCode::Ok)
}

pub async fn message_draft_update(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let input: UpdateMessageDraft = req.body_json().await?;
    tide::log::info!(
        "Updating message draft for draft ID {}",
        input.message_draft_id,
    );

    let output = MessageService::update_draft(&ctx, input).await?;
    txn.commit().await?;
    build_json_response(&output, StatusCode::Ok)
}

pub async fn message_draft_send(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let SendMessageDraft { message_draft_id } = req.body_json().await?;
    tide::log::info!("Sending message draft with ID {message_draft_id}");

    let output = MessageService::send(&ctx, &message_draft_id).await?;
    txn.commit().await?;
    build_json_response(&output, StatusCode::Ok)
}

pub async fn message_draft_delete(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let DeleteMessageDraft { message_draft_id } = req.body_json().await?;
    tide::log::info!("Deleting message draft with ID {message_draft_id}");

    MessageService::delete_draft(&ctx, message_draft_id).await?;
    txn.commit().await?;
    Ok(Response::new(StatusCode::Ok))
}
