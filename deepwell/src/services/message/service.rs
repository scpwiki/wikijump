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
use crate::models::sea_orm_active_enums::MessageRecipientType;
use crate::services::render::{RenderOutput, RenderService};
use crate::services::{InteractionService, TextService, UserService};
use cuid2::cuid;
use ftml::data::{PageInfo, ScoreValue};
use ftml::settings::{WikitextMode, WikitextSettings};
use sea_orm::DatabaseTransaction;

#[derive(Debug)]
pub struct MessageService;

impl MessageService {
    pub async fn send(ctx: &ServiceContext<'_>, draft_id: &str) -> Result<()> {
        tide::log::info!("Sending draft ID {draft_id} as message");

        // Gather resources
        let config = ctx.config();
        let draft = Self::get_draft(ctx, draft_id).await?;
        let wikitext = TextService::get(ctx, &draft.wikitext_hash).await?;
        let recipients: DraftRecipients = serde_json::from_value(draft.recipients)?;

        // Message validation checks
        if draft.subject.is_empty() {
            tide::log::error!("Subject line cannot be empty");
            return Err(Error::BadRequest);
        }

        if draft.subject.len() > config.maximum_message_subject_bytes {
            tide::log::error!(
                "Subject line is too long (is {}, max {})",
                draft.subject.len(),
                config.maximum_message_subject_bytes,
            );
            return Err(Error::BadRequest);
        }

        if wikitext.is_empty() {
            tide::log::error!("Wikitext body cannot be empty");
            return Err(Error::BadRequest);
        }

        if wikitext.len() > config.maximum_message_body_bytes {
            tide::log::error!(
                "Wikitext body is too long (is {}, max {})",
                wikitext.len(),
                config.maximum_message_body_bytes,
            );
            return Err(Error::BadRequest);
        }

        if recipients.len() > config.maximum_message_recipients {
            tide::log::error!(
                "Too many message recipients (is {}, max {})",
                recipients.len(),
                config.maximum_message_recipients,
            );
            return Err(Error::BadRequest);
        }

        for recipient_user_id in recipients.iter() {
            InteractionService::check_user_block(
                ctx,
                draft.user_id,
                recipient_user_id,
                "send a direct message to",
            )
            .await?;
        }

        // The message sending process:
        // * Insert message_draft row to message_record
        // * Delete message_draft row
        // * Insert message_recipient rows
        // * Insert inbox message rows for each recipient
        // * Except, if this is a message to self
        // * Insert outbox message row for sender
        let txn = ctx.transaction();

        // Create message record
        let record_id = draft.external_id.clone();
        let sender_id = draft.user_id;
        let model = message_record::ActiveModel {
            external_id: Set(draft.external_id),
            drafted_at: Set(draft.created_at),
            sender_id: Set(sender_id),
            subject: Set(draft.subject),
            wikitext_hash: Set(draft.wikitext_hash),
            compiled_hash: Set(draft.compiled_hash),
            compiled_at: Set(draft.compiled_at),
            compiled_generator: Set(draft.compiled_generator),
            reply_to: Set(draft.reply_to),
            forwarded_from: Set(draft.forwarded_from),
            ..Default::default()
        };
        model.insert(txn).await?;

        // Delete message draft
        MessageDraft::delete_by_id(record_id.clone())
            .exec(txn)
            .await?;

        // Add recipients
        try_join!(
            Self::add_recipients(
                txn,
                &record_id,
                &recipients.regular,
                MessageRecipientType::Regular,
            ),
            Self::add_recipients(
                txn,
                &record_id,
                &recipients.carbon_copy,
                MessageRecipientType::Cc,
            ),
            Self::add_recipients(
                txn,
                &record_id,
                &recipients.blind_carbon_copy,
                MessageRecipientType::Bcc,
            ),
        )?;

        // Add message records
        let mut has_self = false;
        for user_id in recipients.iter() {
            // Special handling for self-messages
            if sender_id == user_id {
                has_self = true;
                continue;
            }

            let model = message::ActiveModel {
                record_id: Set(record_id.clone()),
                user_id: Set(user_id),
                flag_inbox: Set(true),
                flag_outbox: Set(false),
                flag_self: Set(false),
                ..Default::default()
            };
            model.insert(txn).await?;
        }

        // Add outbox message.
        let (flag_outbox, flag_self) = if has_self {
            // For self-messages, we have two kinds of behavior.
            // If it was sent *only* to oneself, then there is not outbox message.
            // If it was sent to others in addition to oneself, then there *is* an outbox message.
            (recipients.len() > 1, true)
        } else {
            // For regular messages, then just mark the outbox.
            (true, false)
        };

        // If self-message, then mark that.
        let model = message::ActiveModel {
            record_id: Set(record_id),
            user_id: Set(sender_id),
            flag_inbox: Set(false),
            flag_outbox: Set(true),
            flag_self: Set(false),
            ..Default::default()
        };
        model.insert(txn).await?;

        Ok(())
    }

    async fn add_recipients(
        txn: &DatabaseTransaction,
        record_id: &str,
        user_ids: &[i64],
        recipient_type: MessageRecipientType,
    ) -> Result<()> {
        for user_id in user_ids.iter().copied() {
            let model = message_recipient::ActiveModel {
                record_id: Set(str!(record_id)),
                recipient_type: Set(recipient_type),
                recipient_id: Set(user_id),
            };
            model.insert(txn).await?;
        }

        Ok(())
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
            Self::check_message_access(ctx, record_id, user_id, "reply").await?;
        }

        if let Some(record_id) = &forwarded_from {
            Self::check_message_access(ctx, record_id, user_id, "forward").await?;
        }

        // Populate fields
        let draft_id = cuid();

        let user = UserService::get(ctx, Reference::Id(user_id)).await?;
        let recipients = serde_json::to_value(&DraftRecipients {
            regular: recipients,
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
        } = Self::render(ctx, wikitext, &user.locale).await?;

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

    // TODO: method to edit draft

    async fn check_message_access(
        ctx: &ServiceContext<'_>,
        record_id: &str,
        user_id: i64,
        purpose: &'static str,
    ) -> Result<()> {
        // Ensure the message record exists
        let record = match Self::get_record_optional(ctx, record_id).await? {
            Some(record) => record,
            None => {
                tide::log::error!(
                    "The {purpose} message record does not exist: {record_id}",
                );

                return Err(Error::BadRequest);
            }
        };

        // Check that the user has access to the message.
        // Meaning, they are the sender or one of the recipient.
        if record.sender_id != user_id
            && Self::any_recipient_exists(ctx, record_id, user_id).await?
        {
            tide::log::error!(
                "User ID {user_id} is not a sender or recipient of the {purpose}",
            );

            return Err(Error::BadRequest);
        }

        Ok(())
    }

    async fn any_recipient_exists(
        ctx: &ServiceContext<'_>,
        record_id: &str,
        user_id: i64,
    ) -> Result<bool> {
        tide::log::info!(
            "Checking if user ID {user_id} is a recipient of record ID {record_id}",
        );

        let txn = ctx.transaction();
        let model = MessageRecipient::find()
            .filter(
                Condition::all()
                    .add(message_recipient::Column::RecordId.eq(record_id))
                    .add(message_recipient::Column::RecipientId.eq(user_id)),
            )
            .one(txn)
            .await?;

        Ok(model.is_some())
    }

    async fn render(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        user_locale: &str,
    ) -> Result<RenderOutput> {
        tide::log::info!("Rendering message wikitext ({} bytes)", wikitext.len());

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
        let txn = ctx.transaction();
        let message = Message::find()
            .filter(message::Column::RecordId.eq(record_id))
            .one(txn)
            .await?;

        Ok(message)
    }

    pub async fn get_record_optional(
        ctx: &ServiceContext<'_>,
        record_id: &str,
    ) -> Result<Option<MessageRecordModel>> {
        let txn = ctx.transaction();
        let record = MessageRecord::find()
            .filter(message_record::Column::ExternalId.eq(record_id))
            .one(txn)
            .await?;

        Ok(record)
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
        let txn = ctx.transaction();
        let draft = MessageDraft::find()
            .filter(message_draft::Column::ExternalId.eq(draft_id))
            .one(txn)
            .await?;

        Ok(draft)
    }

    pub async fn get_draft(
        ctx: &ServiceContext<'_>,
        draft_id: &str,
    ) -> Result<MessageDraftModel> {
        find_or_error(Self::get_draft_optional(ctx, draft_id)).await
    }
}
