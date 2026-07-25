/*
 * tests/user.rs
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
use deepwell::constants::ADMIN_USER_ID;
use deepwell::error::prelude::*;
use deepwell::models::{known_user, wikidot_user};
use deepwell::services::RequestContext;
use sea_orm::{ActiveModelTrait, Set};
use serde_json::json;
use time::{Date, Month, OffsetDateTime};

#[tokio::test]
async fn user_import_reclaims_existing_wikidot_user() {
    let mut runner = TestRunner::setup().await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    let user_id = 700_001_i64;

    known_user::ActiveModel {
        user_id: Set(user_id),
    }
    .insert(runner.context().transaction())
    .await
    .expect("known_user fixture should insert");

    wikidot_user::ActiveModel {
        user_id: Set(i32::try_from(user_id).expect("fixture ID should fit i32")),
        created_at: Set(OffsetDateTime::UNIX_EPOCH),
        fetched_at: Set(OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(1)),
        is_deleted: Set(false),
        name: Set(Some("Imported User".to_owned())),
        slug: Set(Some("imported-user".to_owned())),
        real_name: Set(None),
        gender: Set(None),
        birthday: Set(None),
        location: Set(None),
        biography: Set(None),
        website: Set(None),
        karma: Set(0),
        is_pro: Set(false),
    }
    .insert(runner.context().transaction())
    .await
    .expect("wikidot_user fixture should insert");

    let imported = run_endpoint!(
        runner,
        user_import,
        json!({
            "user_type": "regular",
            "name": "Imported User",
            "email": "imported-user@example.invalid",
            "locales": ["en"],
            "password": "hunter2",
            "bypass_email_verification": true,
            "override_user_id": user_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    assert_eq!(imported.user_id, user_id);
    assert_eq!(imported.slug, "imported-user");

    let output = run_endpoint!(runner, user_get, json!({ "user": "imported-user" }))
        .expect("imported Wikidot user should be fetchable as a Wikijump user");
    assert_eq!(output.user.user_id, user_id);
    assert_eq!(output.user.slug, "imported-user");
}

#[tokio::test]
async fn user_import_requires_admin_request_context() {
    let runner = TestRunner::setup().await;

    let error = run_endpoint_err!(
        runner,
        user_import,
        json!({
            "user_type": "regular",
            "name": "Unauthorized Import User",
            "email": "unauthorized-import-user@example.invalid",
            "locales": ["en"],
            "password": "test-password",
            "bypass_email_verification": true,
            "override_user_id": 700_003_i64,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    assert_contains_error!(error, ErrorType::PermissionDenied);
}

#[tokio::test]
async fn user_create_rejects_existing_override_user_id() {
    let mut runner = TestRunner::setup().await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    let user_id = 700_002_i64;

    known_user::ActiveModel {
        user_id: Set(user_id),
    }
    .insert(runner.context().transaction())
    .await
    .expect("known_user fixture should insert");

    let error = run_endpoint_err!(
        runner,
        user_create,
        json!({
            "user_type": "regular",
            "name": "Plain Override User",
            "email": "plain-override-user@example.invalid",
            "locales": ["en"],
            "password": "test-password",
            "bypass_email_verification": true,
            "override_user_id": user_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    assert_contains_error!(error, ErrorType::BadRequest);
}

#[tokio::test]
async fn basic_update() {
    let mut runner = TestRunner::setup().await;

    const USER_NAME: &str = "Jane Doe";
    const USER_SLUG: &str = "jane-doe";

    // Doesn't exist yet

    let user = run_endpoint!(runner, user_get, json!({ "user": USER_SLUG }));

    assert!(user.is_none(), "User exists before creation");

    // Create user

    let user = run_endpoint!(
        runner,
        user_create,
        json!({
            "user_type": "regular",
            "name": USER_NAME,
            "email": "jane@private.me",
            "locales": ["en_GB"],
            "password": "hunter2",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let user_id = user.user_id;
    assert_eq!(user.slug, USER_SLUG);
    runner.set_request_context(RequestContext {
        user_id: Some(user_id),
        ..Default::default()
    });

    // Get via slug

    let output = run_endpoint!(runner, user_get, json!({ "user": USER_SLUG }))
        .expect("User does not exist after creation");

    assert_eq!(output.user.user_id, user_id);
    assert_eq!(output.user.name, USER_NAME);
    assert_eq!(output.user.slug, USER_SLUG);
    assert!(output.user.updated_at.is_none());
    assert!(output.user.deleted_at.is_none());
    assert_eq!(output.user.name_changes_left, 2); // set in Config::integration_testing()
    assert!(output.user.last_renamed_at.is_none());
    assert!(!output.user.password.is_empty());
    assert_eq!(output.user.email, "jane@private.me");
    assert!(output.user.email_verified_at.is_none());
    assert!(output.user.email_validation_info.is_some());
    assert!(output.user.email_validation_at.is_some());
    assert_eq!(output.user.locales.len(), 1);
    assert_eq!(&output.user.locales[0], "en_GB");
    assert!(output.user.real_name.is_none());
    assert!(output.user.gender.is_none());
    assert!(output.user.birthday.is_none());
    assert!(output.user.location.is_none());
    assert!(output.user.biography.is_none());
    assert!(output.user.user_page.is_none());
    assert!(output.aliases.is_empty());

    // Update bio fields

    let user = run_endpoint!(
        runner,
        user_edit,
        json!({
            "user": user_id,
            "real_name": "Jane H. Doe",
            "user_page": "https://example.net",
            "gender": "she/they",
            "birthday": "1986-02-01",
            "location": "Edinburgh, Scotland",
            "ip_address": common::IP_ADDRESS,
        }),
    );

    // Get and check

    let output = run_endpoint!(runner, user_get, json!({ "user": user_id }))
        .expect("User does not exist");

    let birthday = Date::from_calendar_date(1986, Month::February, 1).unwrap();
    assert_eq!(user, output.user); // ensures that the model returned by user_edit is latest
    assert_str_eq!(user.real_name, Some("Jane H. Doe"));
    assert_str_eq!(user.gender, Some("she/they"));
    assert_eq!(user.birthday, Some(birthday));
    assert_str_eq!(user.location, Some("Edinburgh, Scotland"));
    assert!(user.biography.is_none());
    assert_str_eq!(user.user_page, Some("https://example.net"));
    let old_password = user.password;

    // Update email (valid)

    let user = run_endpoint!(
        runner,
        user_edit,
        json!({
            "user": USER_SLUG,
            "email": "jane@wikijump.dev",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(user.email_validation_info.is_some());
    assert!(user.email_validation_at.is_some());
    assert_eq!(user.user_id, user_id);
    assert_eq!(user.email, "jane@wikijump.dev");
    assert!(user.biography.is_none());

    // Update email (spam)

    let error = run_endpoint_err!(
        runner,
        user_edit,
        json!({
            "user": USER_SLUG,
            "email": "jane@spam.xxx",
            "biography": "This is a spam account now",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::DisallowedEmail);

    // Update password

    let user = run_endpoint!(
        runner,
        user_edit,
        json!({
            "user": USER_SLUG,
            "password": "letmein",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_ne!(user.password, old_password);
}

#[tokio::test]
async fn user_create_only_verified_email_blocks_conflict() {
    let mut runner = TestRunner::setup().await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });

    let first = run_endpoint!(
        runner,
        user_create,
        json!({
            "user_type": "regular",
            "name": "First Unverified Email User",
            "email": "shared-unverified@example.invalid",
            "locales": ["en"],
            "password": "hunter2",
            "bypass_email_verification": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let second = run_endpoint!(
        runner,
        user_create,
        json!({
            "user_type": "regular",
            "name": "Second Unverified Email User",
            "email": "shared-unverified@example.invalid",
            "locales": ["en"],
            "password": "letmein",
            "bypass_email_verification": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_ne!(first.user_id, second.user_id);

    let error = run_endpoint_err!(
        runner,
        auth_login,
        json!({
            "name_or_email": "shared-unverified@example.invalid",
            "password": "hunter2",
            "ip_address": common::IP_ADDRESS,
            "user_agent": "verified-email-test",
        }),
    );
    assert_contains_error!(error, ErrorType::InvalidAuthentication);

    let verified = run_endpoint!(
        runner,
        user_edit,
        json!({
            "user": first.user_id,
            "email_verified": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(verified.email_verified_at.is_some());

    let name_owner = run_endpoint!(
        runner,
        user_create,
        json!({
            "user_type": "regular",
            "name": "shared-unverified@example.invalid",
            "email": "name-owner@example.invalid",
            "locales": ["en"],
            "password": "hunter2",
            "bypass_filter": true,
            "bypass_email_verification": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_ne!(name_owner.user_id, first.user_id);

    let login = run_endpoint!(
        runner,
        auth_login,
        json!({
            "name_or_email": "shared-unverified@example.invalid",
            "password": "hunter2",
            "ip_address": common::IP_ADDRESS,
            "user_agent": "verified-email-test",
        }),
    );
    let session = run_endpoint!(runner, auth_session_get, json!([login.session_token]),)
        .expect("verified email login should create a session");
    assert_eq!(session.user_id, first.user_id);

    let error = run_endpoint_err!(
        runner,
        user_edit,
        json!({
            "user": second.user_id,
            "email_verified": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::UserExists);

    let error = run_endpoint_err!(
        runner,
        user_create,
        json!({
            "user_type": "regular",
            "name": "Blocked Verified Email User",
            "email": "shared-unverified@example.invalid",
            "locales": ["en"],
            "password": "hunter2",
            "bypass_email_verification": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    assert_contains_error!(error, ErrorType::UserExists);
}

#[tokio::test]
async fn changing_email_clears_verified_ownership() {
    let mut runner = TestRunner::setup().await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });

    let user = run_endpoint!(
        runner,
        user_create,
        json!({
            "user_type": "regular",
            "name": "Verified Email Change User",
            "email": "verified-before-change@example.invalid",
            "locales": ["en"],
            "password": "hunter2",
            "bypass_email_verification": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let verified = run_endpoint!(
        runner,
        user_edit,
        json!({
            "user": user.user_id,
            "email_verified": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(verified.email_verified_at.is_some());

    let unchanged = run_endpoint!(
        runner,
        user_edit,
        json!({
            "user": user.user_id,
            "email": "verified-before-change@example.invalid",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(unchanged.email_verified_at.is_some());

    let error = run_endpoint_err!(
        runner,
        user_edit,
        json!({
            "user": user.user_id,
            "email": "unverified-after-change@example.invalid",
            "email_verified": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::BadRequest);

    let changed = run_endpoint!(
        runner,
        user_edit,
        json!({
            "user": user.user_id,
            "email": "unverified-after-change@example.invalid",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(changed.email_verified_at.is_none());
}

#[tokio::test]
async fn user_mutations_enforce_request_actor_and_staff_only_fields() {
    let mut runner = TestRunner::setup().await;

    let target = run_endpoint!(
        runner,
        user_create,
        json!({
            "user_type": "regular",
            "name": "Mutation Target User",
            "email": "mutation-target@example.invalid",
            "locales": ["en"],
            "password": "hunter2",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let other = run_endpoint!(
        runner,
        user_create,
        json!({
            "user_type": "regular",
            "name": "Other Mutation User",
            "email": "other-mutation-user@example.invalid",
            "locales": ["en"],
            "password": "hunter2",
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let error = run_endpoint_err!(
        runner,
        user_edit,
        json!({
            "user": target.user_id,
            "biography": "unauthenticated edit",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    runner.set_request_context(RequestContext {
        user_id: Some(other.user_id),
        ..Default::default()
    });
    let error = run_endpoint_err!(runner, user_delete, json!({"user": target.user_id}),);
    assert_contains_error!(error, ErrorType::PermissionDenied);

    runner.set_request_context(RequestContext {
        user_id: Some(target.user_id),
        ..Default::default()
    });
    let updated = run_endpoint!(
        runner,
        user_edit,
        json!({
            "user": target.user_id,
            "biography": "self-service edit",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_str_eq!(updated.biography, Some("self-service edit"));

    let error = run_endpoint_err!(
        runner,
        user_edit,
        json!({
            "user": target.user_id,
            "email_verified": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    let error = run_endpoint_err!(
        runner,
        user_add_name_change,
        json!({"user": target.user_id}),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    let name_changes = run_endpoint!(
        runner,
        user_add_name_change,
        json!({"user": target.user_id}),
    );
    assert_eq!(name_changes, runner.config().maximum_name_changes);

    runner.set_request_context(RequestContext {
        user_id: Some(target.user_id),
        ..Default::default()
    });
    let deleted = run_endpoint!(runner, user_delete, json!({"user": target.user_id}),);
    assert_eq!(deleted.user_id, target.user_id);
    assert!(deleted.deleted_at.is_some());
}

#[tokio::test]
async fn public_user_creation_rejects_privileged_fields() {
    let runner = TestRunner::setup().await;

    for privileged_fields in [
        json!({"bypass_filter": true}),
        json!({"bypass_email_verification": true}),
        json!({"override_user_id": 700_100_i64}),
        json!({"user_type": "system"}),
    ] {
        let mut input = json!({
            "user_type": "regular",
            "name": "Privileged Public User",
            "email": "privileged-public-user@example.invalid",
            "locales": ["en"],
            "password": "hunter2",
            "ip_address": common::IP_ADDRESS,
        });
        input
            .as_object_mut()
            .expect("fixture input should be an object")
            .extend(
                privileged_fields
                    .as_object()
                    .expect("fixture fields should be an object")
                    .clone(),
            );

        let error = run_endpoint_err!(runner, user_create, input);
        assert_contains_error!(error, ErrorType::PermissionDenied);
    }
}

// TODO test renames / rename tokens
//      test creating users of other types
