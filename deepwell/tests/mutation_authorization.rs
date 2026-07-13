/*
 * tests/mutation_authorization.rs
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
use deepwell::error::ErrorType;
use deepwell::services::RequestContext;
use serde_json::json;

#[tokio::test]
async fn payload_actor_cannot_spoof_vote_or_message_attribution() {
    let mut runner = TestRunner::setup().await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });

    let vote_error = run_endpoint_err!(
        runner,
        vote_set,
        json!({
            "page_id": 1,
            "user_id": SAMPLE_USER_ID,
            "value": 1,
        }),
    );
    assert_contains_error!(vote_error, ErrorType::PermissionDenied);

    let message_error = run_endpoint_err!(
        runner,
        message_draft_create,
        json!({
            "user_id": SAMPLE_USER_ID,
            "recipients": [ADMIN_USER_ID],
            "carbon_copy": [],
            "blind_carbon_copy": [],
            "locale": "en",
            "subject": "Spoofed sender",
            "wikitext": "This must never be stored.",
            "reply_to": null,
            "forwarded_from": null,
        }),
    );
    assert_contains_error!(message_error, ErrorType::PermissionDenied);
}

#[tokio::test]
async fn vote_moderation_requires_staff_and_matching_attribution() {
    let mut runner = TestRunner::setup().await;
    runner.set_request_context(RequestContext {
        user_id: Some(SAMPLE_USER_ID),
        ..Default::default()
    });
    let non_staff_error = run_endpoint_err!(
        runner,
        vote_action,
        json!({
            "page_id": 1,
            "user_id": SAMPLE_USER_ID,
            "enable": false,
            "acting_user_id": SAMPLE_USER_ID,
        }),
    );
    assert_contains_error!(non_staff_error, ErrorType::PermissionDenied);

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    let spoofed_staff_error = run_endpoint_err!(
        runner,
        vote_action,
        json!({
            "page_id": 1,
            "user_id": SAMPLE_USER_ID,
            "enable": false,
            "acting_user_id": SAMPLE_USER_ID,
        }),
    );
    assert_contains_error!(spoofed_staff_error, ErrorType::PermissionDenied);
}
