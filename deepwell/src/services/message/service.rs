/*
 * services/message/service.rs
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
use crate::models::message::{self, Entity as Message, Model as MessageModel};
use crate::models::message_draft::{
    self, Entity as MessageDraft, Model as MessageDraftModel,
};
use crate::models::message_recipient::{
    self, Entity as MessageRecipient, Model as MessageRecipientModel,
};
use crate::models::message_record::{
    self, Entity as MessageRecord, Model as MessageRecordModel,
};
use crate::models::message_report::{
    self, Entity as MessageReport, Model as MessageReportModel,
};
use crate::services::render::{RenderOutput, RenderService};
use crate::services::{TextService, UserService};
use cuid2::cuid;
use ftml::data::{PageInfo, ScoreValue};
use ftml::settings::{WikitextMode, WikitextSettings};

#[derive(Debug)]
pub struct MessageService;

impl MessageService {
    pub async fn send(ctx: &ServiceContext<'_>, draft_id: &str) -> Result<()> {
        tide::log::info!("Sending draft ID {draft_id} as message");

        let draft = Self::get_draft(ctx, draft_id).await?;

        // TODO validate input, fail if any are not ok
        // - not too many recipients
        // - wikitext isn't too long
        // - not blocked by anyone in the recipient list

        todo!()
    }

    pub async fn create_draft(
        ctx: &ServiceContext<'_>,
        CreateMessageDraft {
            user_id,
            recipients,
            carbon_copy,
            blind_carbon_copy,
            subject,
            wikitext,
            reply_to,
            forwarded_from,
        }: CreateMessageDraft,
    ) -> Result<MessageDraftModel> {
        tide::log::info!("Creating message draft for user ID {user_id}");

        // Check foreign keys
        if let Some(record_id) = &reply_to {
            if !Self::record_exists(ctx, record_id).await? {
                tide::log::error!(
                    "Message record being replied to does not exist: {record_id}",
                );

                return Err(Error::BadRequest);
            }
        }

        if let Some(record_id) = &forwarded_from {
            if !Self::record_exists(ctx, record_id).await? {
                tide::log::error!(
                    "Message record being forwarded from does not exist: {record_id}",
                );

                return Err(Error::BadRequest);
            }
        }

        // Populate fields
        let draft_id = cuid();

        let user = UserService::get(ctx, Reference::Id(user_id)).await?;
        let recipients = serde_json::to_value(&DraftRecipients {
            recipients,
            carbon_copy,
            blind_carbon_copy,
        })?;

        let wikitext_hash = TextService::create(ctx, wikitext.clone()).await?;
        let RenderOutput {
            // TODO: use html_output
            html_output: _,
            errors,
            compiled_hash,
            compiled_at,
            compiled_generator,
        } = Self::render_draft(ctx, wikitext, &user.locale).await?;

        // Insert draft into database
        let txn = ctx.transaction();
        let model = message_draft::ActiveModel {
            external_id: Set(draft_id),
            user_id: Set(user_id),
            recipients: Set(recipients),
            subject: Set(subject),
            wikitext_hash: Set(wikitext_hash.to_vec()),
            compiled_hash: Set(compiled_hash.to_vec()),
            compiled_at: Set(compiled_at),
            compiled_generator: Set(compiled_generator),
            reply_to: Set(reply_to),
            forwarded_from: Set(forwarded_from),
            ..Default::default()
        };

        let draft = model.insert(txn).await?;
        Ok(draft)
    }

    async fn render_draft(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        user_locale: &str,
    ) -> Result<RenderOutput> {
        tide::log::info!(
            "Rendering message draft wikitext ({} bytes)",
            wikitext.len(),
        );

        let settings = WikitextSettings::from_mode(WikitextMode::DirectMessage);
        let page_info = PageInfo {
            page: cow!(""),
            category: None,
            site: cow!(""),
            title: cow!(""),
            alt_title: None,
            score: ScoreValue::Integer(0),
            tags: vec![],
            language: cow!(user_locale),
        };

        RenderService::render(ctx, wikitext, &page_info, &settings).await
    }

    pub async fn get_message_optional(
        ctx: &ServiceContext<'_>,
        record_id: &str,
    ) -> Result<Option<MessageModel>> {
        // XXX
        todo!()
    }

    pub async fn get_record_optional(
        ctx: &ServiceContext<'_>,
        record_id: &str,
    ) -> Result<Option<MessageRecordModel>> {
        // XXX
        todo!()
    }

    pub async fn record_exists(
        ctx: &ServiceContext<'_>,
        record_id: &str,
    ) -> Result<bool> {
        Self::get_record_optional(ctx, record_id)
            .await
            .map(|record| record.is_some())
    }

    pub async fn get_draft_optional(
        ctx: &ServiceContext<'_>,
        draft_id: &str,
    ) -> Result<Option<MessageDraftModel>> {
        // XXX
        todo!()
    }

    pub async fn get_draft(
        ctx: &ServiceContext<'_>,
        draft_id: &str,
    ) -> Result<MessageDraftModel> {
        find_or_error(Self::get_draft_optional(ctx, draft_id)).await
    }
}
