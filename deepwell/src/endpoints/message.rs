/*
 * endpoints/message.rs
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
use crate::models::message_draft::Model as MessageDraftModel;
use crate::models::message_record::Model as MessageRecordModel;
use crate::services::message::{
    CreateMessageDraft, DeleteMessageDraft, SendMessageDraft, UpdateMessageDraft,
};

pub async fn message_draft_create(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<MessageDraftModel> {
    let input: CreateMessageDraft = params.parse()?;
    info!("Creating new message draft for user ID {}", input.user_id);
    MessageService::create_draft(ctx, input).await
}

pub async fn message_draft_edit(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<MessageDraftModel> {
    let input: UpdateMessageDraft = params.parse()?;
    info!(
        "Updating message draft for draft ID {}",
        input.message_draft_id,
    );
    MessageService::update_draft(ctx, input).await
}

pub async fn message_draft_delete(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<()> {
    let DeleteMessageDraft { message_draft_id } = params.parse()?;
    info!("Deleting message draft with ID {message_draft_id}");
    MessageService::delete_draft(ctx, message_draft_id).await
}

pub async fn message_draft_send(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<MessageRecordModel> {
    let SendMessageDraft { message_draft_id } = params.parse()?;
    info!("Sending message draft with ID {message_draft_id}");
    MessageService::send(ctx, &message_draft_id).await
}
