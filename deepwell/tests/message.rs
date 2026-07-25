/*
 * tests/message.rs
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

#[macro_use]
mod common;

use self::common::TestRunner;
use deepwell::constants::{ADMIN_USER_ID, SAMPLE_USER_ID};
use deepwell::services::{MessageService, RequestContext};
use serde_json::json;

fn draft_params(subject: &str, wikitext: &str) -> serde_json::Value {
    json!({
        "user_id": ADMIN_USER_ID,
        "recipients": [SAMPLE_USER_ID],
        "carbon_copy": [],
        "blind_carbon_copy": [],
        "locale": "en",
        "subject": subject,
        "wikitext": wikitext,
        "reply_to": null,
        "forwarded_from": null,
    })
}

#[tokio::test]
async fn message_draft_lifecycle_sends_and_deletes_drafts() {
    let mut runner = TestRunner::setup().await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });

    let draft = run_endpoint!(
        runner,
        message_draft_create,
        draft_params("Initial subject", "Initial **body**"),
    );
    assert_eq!(draft.user_id, ADMIN_USER_ID);
    assert_eq!(draft.subject, "Initial subject");

    runner.set_request_context(RequestContext {
        user_id: Some(SAMPLE_USER_ID),
        ..Default::default()
    });
    let error = run_endpoint_err!(
        runner,
        message_draft_edit,
        json!({
            "message_draft_id": draft.external_id,
            "recipients": [SAMPLE_USER_ID],
            "carbon_copy": [],
            "blind_carbon_copy": [],
            "locale": "en",
            "subject": "Unauthorized update",
            "wikitext": "Unauthorized body",
        }),
    );
    assert_contains_error!(error, deepwell::error::ErrorType::PermissionDenied);
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });

    let edited = run_endpoint!(
        runner,
        message_draft_edit,
        json!({
            "message_draft_id": draft.external_id,
            "recipients": [SAMPLE_USER_ID],
            "carbon_copy": [],
            "blind_carbon_copy": [],
            "locale": "en",
            "subject": "Updated subject",
            "wikitext": "Updated body",
        }),
    );
    assert_eq!(edited.external_id, draft.external_id);
    assert_eq!(edited.subject, "Updated subject");
    assert!(edited.updated_at.is_some());

    let record = run_endpoint!(
        runner,
        message_draft_send,
        json!({"message_draft_id": edited.external_id}),
    );
    assert_eq!(record.sender_id, ADMIN_USER_ID);
    assert_eq!(record.subject, "Updated subject");

    assert!(
        MessageService::get_draft_optional(runner.context(), &record.external_id)
            .await
            .unwrap()
            .is_none(),
    );

    let recipient_message = MessageService::get_message(
        runner.context(),
        &record.external_id,
        SAMPLE_USER_ID,
    )
    .await
    .unwrap();
    assert!(recipient_message.flag_inbox);
    assert!(!recipient_message.flag_outbox);
    assert!(!recipient_message.flag_self);

    let sender_message =
        MessageService::get_message(runner.context(), &record.external_id, ADMIN_USER_ID)
            .await
            .unwrap();
    assert!(!sender_message.flag_inbox);
    assert!(sender_message.flag_outbox);
    assert!(!sender_message.flag_self);

    let deletable = run_endpoint!(
        runner,
        message_draft_create,
        draft_params("Delete me", "Temporary body"),
    );
    run_endpoint!(
        runner,
        message_draft_delete,
        json!({"message_draft_id": deletable.external_id}),
    );
    assert!(
        MessageService::get_draft_optional(runner.context(), &deletable.external_id)
            .await
            .unwrap()
            .is_none(),
    );
}
