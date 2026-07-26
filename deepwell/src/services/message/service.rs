/*
 * services/message/service.rs
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

use super::structs::{CreateMessageDraft, DraftRecipients, UpdateMessageDraft};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::message::{self, Entity as Message, Model as MessageModel};
use crate::models::message_draft::{
    self, Entity as MessageDraft, Model as MessageDraftModel,
};
use crate::models::message_recipient::{self, Entity as MessageRecipient};
use crate::models::message_record::{
    self, Entity as MessageRecord, Model as MessageRecordModel,
};
use crate::services::ServiceContext;
use crate::services::render::{RenderOutput, RenderService};
use crate::services::{RelationService, TextService, UserService};
use crate::types::{Maybe, Reference};
use crate::types::{MessageRecipientType, UserType};
use crate::utils::now;
use crate::utils::validate_locale;
use cuid2::cuid;
use ftml::data::{PageInfo, ScoreValue};
use ftml::layout::Layout;
use ftml::settings::{WikitextMode, WikitextSettings};
use paste::paste;
use sea_orm::DatabaseTransaction;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};

#[derive(Debug)]
pub struct MessageService;

impl MessageService {
    // Message draft methods

    pub async fn create_draft(
        ctx: &ServiceContext<'_>,
        CreateMessageDraft {
            user_id,
            recipients,
            carbon_copy,
            blind_carbon_copy,
            locale,
            subject,
            wikitext,
            reply_to,
            forwarded_from,
        }: CreateMessageDraft,
    ) -> Result<MessageDraftModel> {
        info!("Creating message draft for user ID {user_id}");

        let wikitext_len = wikitext.len();
        let recipients_len = recipients.len();
        let is_reply = reply_to.is_some();
        let is_forward = forwarded_from.is_some();
        let make_error = || {
            Error::new(
                format!(
                    "failed to create message draft for user ID {} to {} recipients, wikitext {} bytes, reply {}, forward {}",
                    user_id, recipients_len, wikitext_len, is_reply, is_forward,
                ),
                ErrorType::MessageDraft,
            )
        };

        // Check locale
        validate_locale(&locale).or_raise(make_error)?;

        // Check foreign keys
        if let Some(record_id) = &reply_to {
            Self::check_message_access(ctx, record_id, user_id, "reply")
                .await
                .or_raise(make_error)?;
        }

        if let Some(record_id) = &forwarded_from {
            Self::check_message_access(ctx, record_id, user_id, "forward")
                .await
                .or_raise(make_error)?;
        }

        // Insert draft into database
        let txn = ctx.transaction();
        let draft = Self::draft_process(
            ctx,
            DraftProcess {
                is_update: false,
                user_id,
                draft_id: cuid(),
                recipients,
                carbon_copy,
                blind_carbon_copy,
                locale,
                subject,
                wikitext,
                reply_to: Maybe::Set(reply_to),
                forwarded_from: Maybe::Set(forwarded_from),
            },
        )
        .await
        .or_raise(make_error)?
        .insert(txn)
        .await
        .or_raise(make_error)?;

        Ok(draft)
    }

    pub async fn update_draft(
        ctx: &ServiceContext<'_>,
        UpdateMessageDraft {
            message_draft_id: draft_id,
            recipients,
            carbon_copy,
            blind_carbon_copy,
            locale,
            subject,
            wikitext,
        }: UpdateMessageDraft,
    ) -> Result<MessageDraftModel> {
        info!("Updating message draft {draft_id}");

        let draft_id_2 = draft_id.clone();
        let wikitext_len = wikitext.len();
        let recipients_len = recipients.len();
        let make_error = || {
            Error::new(
                format!(
                    "failed to update message draft for draft ID {} to {} recipients, wikitext {} bytes",
                    draft_id_2, recipients_len, wikitext_len,
                ),
                ErrorType::MessageDraft,
            )
        };

        // Validate parameters
        validate_locale(&locale)?;

        // Get current draft
        let current_draft = Self::get_draft(ctx, &draft_id).await.or_raise(make_error)?;

        // Update the draft
        let txn = ctx.transaction();
        let draft = Self::draft_process(
            ctx,
            DraftProcess {
                is_update: true,
                user_id: current_draft.user_id,
                draft_id,
                recipients,
                carbon_copy,
                blind_carbon_copy,
                locale,
                subject,
                wikitext,
                reply_to: Maybe::Unset,
                forwarded_from: Maybe::Unset,
            },
        )
        .await
        .or_raise(make_error)?
        .update(txn)
        .await
        .or_raise(make_error)?;

        Ok(draft)
    }

    /// Helper method to perform functionality common to creating and updating drafts.
    async fn draft_process(
        ctx: &ServiceContext<'_>,
        DraftProcess {
            is_update,
            user_id,
            draft_id,
            recipients,
            carbon_copy,
            blind_carbon_copy,
            locale,
            subject,
            wikitext,
            reply_to,
            forwarded_from,
        }: DraftProcess,
    ) -> Result<message_draft::ActiveModel> {
        let make_error = || {
            Error::new(
                format!("failed to process message draft ID {}", draft_id),
                ErrorType::MessageDraft,
            )
        };

        // Check constraints
        let recipients = DraftRecipients {
            regular: recipients,
            carbon_copy,
            blind_carbon_copy,
        };

        for recipient_id in recipients.iter() {
            let recipient_exists = UserService::exists(ctx, Reference::Id(recipient_id))
                .await
                .or_raise(make_error)?;

            if !recipient_exists {
                error!("Recipient user ID {recipient_id} does not exist!");
                bail!(Error::new(
                    format!(
                        "failed to process message draft ID {}, as recipient user ID {} does not exist",
                        draft_id, recipient_id
                    ),
                    ErrorType::MessageDraft
                ));
            }
        }

        // Populate fields
        let recipients = serde_json::to_value(&recipients).or_raise(make_error)?;

        let config = ctx.config();
        let wikitext_hash = TextService::create(ctx, wikitext.clone())
            .await
            .or_raise(make_error)?;

        let RenderOutput {
            // TODO: use html_output
            html_output: _,
            // TODO: use ftml errors
            errors: _,
            compiled_hash,
            compiled_at,
            compiled_generator,
        } = Self::render(ctx, wikitext, &locale, config.message_layout)
            .await
            .or_raise(make_error)?;

        Ok(message_draft::ActiveModel {
            updated_at: Set(if is_update { Some(now()) } else { None }),
            external_id: Set(draft_id),
            user_id: Set(user_id),
            recipients: Set(recipients),
            subject: Set(subject),
            wikitext_hash: Set(wikitext_hash.to_vec()),
            compiled_hash: Set(compiled_hash.to_vec()),
            compiled_at: Set(compiled_at),
            compiled_generator: Set(compiled_generator),
            reply_to: reply_to.into_active_value(),
            forwarded_from: forwarded_from.into_active_value(),
            ..Default::default()
        })
    }

    pub async fn delete_draft(ctx: &ServiceContext<'_>, draft_id: String) -> Result<()> {
        let make_error = || {
            Error::new(
                format!("failed to delete message draft ID {}", draft_id),
                ErrorType::MessageDraft,
            )
        };

        let txn = ctx.transaction();

        MessageDraft::delete_by_id(draft_id.clone())
            .exec(txn)
            .await
            .or_raise(make_error)?;

        Ok(())
    }

    // Message methods

    pub async fn send(
        ctx: &ServiceContext<'_>,
        draft_id: &str,
    ) -> Result<MessageRecordModel> {
        info!("Sending draft ID {draft_id} as message");

        let make_error = || {
            Error::new(
                format!("failed to send message draft ID {}", draft_id),
                ErrorType::MessageDraft,
            )
        };

        // Gather resources
        let config = ctx.config();
        let draft = Self::get_draft(ctx, draft_id).await.or_raise(make_error)?;
        let wikitext = TextService::get(ctx, &draft.wikitext_hash)
            .await
            .or_raise(make_error)?;

        let mut recipients: DraftRecipients =
            serde_json::from_value(draft.recipients).or_raise(make_error)?;

        // Message validation checks
        //
        // Checking things which are valid for drafts but invalid for sent messages

        if draft.subject.is_empty() {
            error!("Subject line cannot be empty");
            bail!(Error::new(
                format!(
                    "cannot send message from draft ID {}, subject line is empty",
                    draft_id,
                ),
                ErrorType::MessageSubjectEmpty
            ));
        }

        if draft.subject.len() > config.maximum_message_subject_bytes {
            error!(
                "Subject line is too long (is {}, max {})",
                draft.subject.len(),
                config.maximum_message_subject_bytes,
            );
            bail!(Error::new(
                format!(
                    "cannot send message from draft ID {}, subject line is too long ({} > {} bytes)",
                    draft_id,
                    draft.subject.len(),
                    config.maximum_message_subject_bytes,
                ),
                ErrorType::MessageSubjectTooLong,
            ));
        }

        if wikitext.is_empty() {
            error!("Wikitext body cannot be empty");
            bail!(Error::new(
                format!(
                    "cannot send message from draft ID {}, message body is empty",
                    draft_id,
                ),
                ErrorType::MessageBodyEmpty
            ));
        }

        if wikitext.len() > config.maximum_message_body_bytes {
            error!(
                "Wikitext body is too long (is {}, max {})",
                wikitext.len(),
                config.maximum_message_body_bytes,
            );
            bail!(Error::new(
                format!(
                    "cannot send message from draft ID {}, message body is too long ({} > {} bytes)",
                    draft_id,
                    wikitext.len(),
                    config.maximum_message_body_bytes,
                ),
                ErrorType::MessageBodyTooLong,
            ));
        }

        if recipients.is_empty() {
            error!("Must have at least one message recipient");
            bail!(Error::new(
                format!(
                    "cannot send message from draft ID {}, must have at least one recipient",
                    draft_id,
                ),
                ErrorType::MessageNoRecipients,
            ));
        }

        if recipients.len() > config.maximum_message_recipients {
            error!(
                "Too many message recipients (is {}, max {})",
                recipients.len(),
                config.maximum_message_recipients,
            );
            bail!(Error::new(
                format!(
                    "cannot send message from draft ID {}, recipient list is too long ({} > {})",
                    draft_id,
                    recipients.len(),
                    config.maximum_message_recipients,
                ),
                ErrorType::MessageTooManyRecipients,
            ));
        }

        let mut recipients_to_add = Vec::new();
        for recipient_user_id in recipients.iter() {
            // Ensure user is not blocked
            RelationService::check_user_block(
                ctx,
                draft.user_id,
                recipient_user_id,
                "send a direct message to",
            )
            .await
            .or_raise(make_error)?;

            // If recipient is a site user, then forward to corresponding site staff.
            // It is only possible to send messages to Wikijump users.
            let user = UserService::get_real(ctx, Reference::Id(recipient_user_id))
                .await
                .or_raise(make_error)?;

            if user.user_type == UserType::Site {
                // TODO what to do if user is banned from site? needs to be possible to block
                //      permabanned bad actors, but also allow normal banned users to message
                //      to appeal bans etc
                // TODO get the listed site staff, add them to recipients
                let _site_id =
                    RelationService::get_site_id_for_site_user(ctx, user.user_id)
                        .await
                        .or_raise(make_error)?;

                let _ = &recipients_to_add;
            }
        }
        recipients.carbon_copy.append(&mut recipients_to_add);

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
        let record_model = model.insert(txn).await.or_raise(make_error)?;

        // Delete message draft
        Self::delete_draft(ctx, record_id.clone())
            .await
            .or_raise(make_error)?;

        // Add recipients
        let (result1, result2, result3) = join!(
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
        );
        raise_multiple!(result1, result2, result3; make_error);

        // Add message records
        let mut has_self = false;
        let mut added_user_ids = Vec::new();
        for user_id in recipients.iter() {
            // Ensure user isn't added twice
            //
            // NOTE: Because recipient lists are generally short, well under 100,
            //       there are no practical issues with using Vec over HashSet.
            if added_user_ids.contains(&user_id) {
                continue;
            }

            // Special handling for self-messages, skip here
            if sender_id == user_id {
                has_self = true;
                continue;
            }

            let model = message::ActiveModel {
                record_id: Set(record_id.clone()),
                user_id: Set(user_id),
                flag_inbox: Set(true), // uninvolved recipient just received message, inbox and nothing else
                flag_outbox: Set(false),
                flag_self: Set(false),
                ..Default::default()
            };
            model.insert(txn).await.or_raise(make_error)?;
            added_user_ids.push(user_id);
        }

        // Add outbox message.
        let (flag_outbox, flag_self) = if has_self {
            // For self-messages, we have two kinds of behavior.
            // If it was sent *only* to oneself, then there is not outbox message.
            // If it was sent to others in addition to oneself, then there *is* an outbox message.
            debug!("Self message, checking recipients list");
            (recipients.only_has(sender_id), true)
        } else {
            // For regular messages, then just mark the outbox.
            debug!("Regular message, marking outbox only");
            (true, false)
        };

        // If self-message, then mark that.
        let model = message::ActiveModel {
            record_id: Set(record_id),
            user_id: Set(sender_id),
            flag_inbox: Set(false), // messages from you are never in inbox
            flag_outbox: Set(flag_outbox), // message you sent to others
            flag_self: Set(flag_self), // message you sent to yourself
            ..Default::default()
        };
        model.insert(txn).await.or_raise(make_error)?;

        Ok(record_model)
    }

    #[allow(dead_code)] // TEMP
    pub async fn mark_read(
        ctx: &ServiceContext<'_>,
        record_id: &str,
        user_id: i64,
        value: bool,
    ) -> Result<()> {
        info!("Setting message read status for {record_id} / {user_id}: read={value}");

        let make_error = || {
            Error::new(
                format!(
                    "failed to mark message ID {} from user ID {} as {}",
                    record_id,
                    user_id,
                    if value { "read" } else { "unread" },
                ),
                ErrorType::Message,
            )
        };

        let txn = ctx.transaction();
        let message = Self::get_message(ctx, record_id, user_id)
            .await
            .or_raise(make_error)?;

        let model = message::ActiveModel {
            internal_id: Set(message.internal_id),
            flag_read: Set(value),
            ..Default::default()
        };
        model.update(txn).await.or_raise(make_error)?;

        Ok(())
    }

    // Getters

    pub async fn get_message_optional(
        ctx: &ServiceContext<'_>,
        record_id: &str,
        user_id: i64,
    ) -> Result<Option<MessageModel>> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to get message ID {} from user ID {}",
                    record_id, user_id,
                ),
                ErrorType::Message,
            )
        };

        let txn = ctx.transaction();
        let message = Message::find()
            .filter(
                Condition::all()
                    .add(message::Column::RecordId.eq(record_id))
                    .add(message::Column::UserId.eq(user_id)),
            )
            .one(txn)
            .await
            .or_raise(make_error)?;

        Ok(message)
    }

    pub async fn get_message(
        ctx: &ServiceContext<'_>,
        record_id: &str,
        user_id: i64,
    ) -> Result<MessageModel> {
        find_or_error!(
            Self::get_message_optional(ctx, record_id, user_id),
            "message",
            Message,
        )
    }

    pub async fn get_record_optional(
        ctx: &ServiceContext<'_>,
        record_id: &str,
    ) -> Result<Option<MessageRecordModel>> {
        let make_error = || {
            Error::new(
                format!("failed to get message record ID {}", record_id),
                ErrorType::MessageRecord,
            )
        };

        let txn = ctx.transaction();
        let record = MessageRecord::find()
            .filter(message_record::Column::ExternalId.eq(record_id))
            .one(txn)
            .await
            .or_raise(make_error)?;

        Ok(record)
    }

    pub async fn get_draft_optional(
        ctx: &ServiceContext<'_>,
        draft_id: &str,
    ) -> Result<Option<MessageDraftModel>> {
        let make_error = || {
            Error::new(
                format!("failed to get message draft ID {}", draft_id),
                ErrorType::MessageDraft,
            )
        };

        let txn = ctx.transaction();
        let draft = MessageDraft::find()
            .filter(message_draft::Column::ExternalId.eq(draft_id))
            .one(txn)
            .await
            .or_raise(make_error)?;

        Ok(draft)
    }

    pub async fn get_draft(
        ctx: &ServiceContext<'_>,
        draft_id: &str,
    ) -> Result<MessageDraftModel> {
        find_or_error!(
            Self::get_draft_optional(ctx, draft_id),
            "message draft",
            MessageDraft,
        )
    }

    // Helper methods

    /// Helper method to insert a group of `message_recipient` rows.
    async fn add_recipients(
        txn: &DatabaseTransaction,
        record_id: &str,
        user_ids: &[i64],
        recipient_type: MessageRecipientType,
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to add recipients to database for message record ID {}: user IDs {:?}",
                    record_id, user_ids,
                ),
                ErrorType::MessageRecord,
            )
        };

        let mut added_user_ids = Vec::new();
        for user_id in user_ids.iter().copied() {
            // NOTE: Because recipient lists are generally short, well under 100,
            //       there are no practical issues with using Vec over HashSet.
            if added_user_ids.contains(&user_id) {
                debug!("Skipping message recipient (already added)");
                continue;
            }

            debug!("Adding message recipient {recipient_type:?} with ID {user_id}");

            let model = message_recipient::ActiveModel {
                record_id: Set(str!(record_id)),
                recipient_type: Set(recipient_type),
                recipient_id: Set(user_id),
            };
            model.insert(txn).await.or_raise(make_error)?;
            added_user_ids.push(user_id);
        }

        Ok(())
    }

    /// Helper method to determine if a message can be "seen" by a user.
    ///
    /// This prevents you from replying to or forwarding a message you cannot
    /// actually otherwise see if you only have its record ID.
    ///
    /// This method checks if a given message record was either sent by the user
    /// in question, or if they are a recipient (in any category).
    ///
    /// It also checks that the message record actually exists.
    async fn check_message_access(
        ctx: &ServiceContext<'_>,
        record_id: &str,
        user_id: i64,
        purpose: &'static str,
    ) -> Result<()> {
        // To protect privacy, if the user doesn't have access to a message with a
        // given ID, we pretend it does not exist for the purposes of returning errors.
        let make_error = || {
            Error::new(
                format!(
                    "the {} message record with ID {} does not exist",
                    purpose, record_id,
                ),
                ErrorType::MessageNotFound,
            )
        };

        // Ensure the message record exists
        let record = match Self::get_record_optional(ctx, record_id)
            .await
            .or_raise(make_error)?
        {
            Some(record) => record,
            None => {
                error!("The {purpose} message record does not exist: {record_id}");
                bail!(make_error());
            }
        };

        // Check that the user has access to the message.
        // That is, the user is the sender or one of the recipients.
        if record.sender_id != user_id
            && Self::any_recipient_exists(ctx, record_id, user_id)
                .await
                .or_raise(make_error)?
        {
            error!("User ID {user_id} is not a sender or recipient of the {purpose}");
            bail!(make_error());
        }

        Ok(())
    }

    /// Helper method which checks if a user is a recipient of a message record.
    async fn any_recipient_exists(
        ctx: &ServiceContext<'_>,
        record_id: &str,
        user_id: i64,
    ) -> Result<bool> {
        info!("Checking if user ID {user_id} is a recipient of record ID {record_id}");

        let make_error = || {
            Error::new(
                format!(
                    "failed checking if user ID {} is a recipient of record ID {}",
                    user_id, record_id,
                ),
                ErrorType::MessageRecord,
            )
        };

        let txn = ctx.transaction();
        let model = MessageRecipient::find()
            .filter(
                Condition::all()
                    .add(message_recipient::Column::RecordId.eq(record_id))
                    .add(message_recipient::Column::RecipientId.eq(user_id)),
            )
            .one(txn)
            .await
            .or_raise(make_error)?;

        Ok(model.is_some())
    }

    /// Helper method to render message contents.
    async fn render(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        locale: &str,
        layout: Layout,
    ) -> Result<RenderOutput> {
        info!("Rendering message wikitext ({} bytes)", wikitext.len());

        let settings = WikitextSettings::from_mode(WikitextMode::DirectMessage, layout);
        let page_info = PageInfo {
            page: cow!(""),
            category: None,
            site: cow!(""),
            title: cow!(""),
            alt_title: None,
            score: ScoreValue::Integer(0),
            tags: vec![],
            language: cow!(locale),
        };

        let output = RenderService::render(ctx, wikitext, &page_info, &settings)
            .await
            .or_raise(|| {
                Error::new("failed to render message contents", ErrorType::Message)
            })?;

        Ok(output)
    }
}

/// Helper structure used by `draft_process()`.
#[derive(Debug)]
struct DraftProcess {
    is_update: bool,
    user_id: i64,
    draft_id: String,
    recipients: Vec<i64>,
    carbon_copy: Vec<i64>,
    blind_carbon_copy: Vec<i64>,
    locale: String,
    subject: String,
    wikitext: String,
    reply_to: Maybe<Option<String>>,
    forwarded_from: Maybe<Option<String>>,
}
