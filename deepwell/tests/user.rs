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
use cuid2::cuid;
use deepwell::config::Config;
use deepwell::constants::{ADMIN_USER_ID, SAMPLE_USER_ID};
use deepwell::error::prelude::*;
use deepwell::hash::{blob_hash_to_hex, sha512_hash};
use deepwell::models::audit_log::{Column as AuditLogColumn, Entity as AuditLogTable};
use deepwell::models::blob_pending::{self, Entity as BlobPending};
use deepwell::models::wikidot_user::{Entity as WikidotUser, Model as WikidotUserModel};
use deepwell::models::{known_user, user, wikidot_user};
use deepwell::services::import::ImportUserOutput;
use deepwell::services::user::UserService;
use deepwell::services::view::GetUserViewOutput;
use deepwell::services::{BlobService, RequestContext};
use deepwell::types::Reference;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde_json::json;
use time::macros::{date, datetime};
use time::{Date, Month, OffsetDateTime};

async fn insert_wikidot_user(
    runner: &TestRunner,
    user_id: i32,
    name: &str,
    slug: &str,
    is_deleted: bool,
    karma: i16,
    is_pro: bool,
) {
    known_user::ActiveModel {
        user_id: Set(i64::from(user_id)),
    }
    .insert(runner.context().transaction())
    .await
    .expect("known_user fixture should insert");

    wikidot_user::ActiveModel {
        user_id: Set(user_id),
        created_at: Set(datetime!(2008-07-19 21:26:10 UTC)),
        fetched_at: Set(datetime!(2026-08-13 00:00:00 UTC)),
        is_deleted: Set(is_deleted),
        name: Set(Some(name.to_owned())),
        slug: Set(Some(slug.to_owned())),
        avatar_s3_hash: Set(None),
        real_name: Set(None),
        gender: Set(None),
        birthday: Set(None),
        location: Set(None),
        biography: Set(None),
        website: Set(None),
        karma: Set(karma),
        is_pro: Set(is_pro),
    }
    .insert(runner.context().transaction())
    .await
    .expect("wikidot_user fixture should insert");
}

#[tokio::test]
async fn user_view_resolves_only_active_imported_slugs_without_changing_local_lookup() {
    let runner = TestRunner::setup().await;
    let site_id = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("test site should exist")
        .site
        .site_id;

    insert_wikidot_user(
        &runner,
        700_011,
        "Imported Profile",
        "imported-profile",
        false,
        3,
        false,
    )
    .await;
    insert_wikidot_user(
        &runner,
        700_012,
        "Deleted Imported Profile",
        "deleted-imported-profile",
        true,
        0,
        false,
    )
    .await;

    let imported = run_endpoint!(
        runner,
        user_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "user": "imported-profile",
            "locales": ["en"],
        }),
    );
    let imported = serde_json::to_value(imported)
        .expect("imported user view response should serialize");
    assert_eq!(
        imported,
        json!({
            "type": "user_found",
            "data": {
                "user": {
                    "user_id": 700_011,
                    "user_type": "wikidot",
                    "created_at": "2008-07-19T21:26:10Z",
                    "fetched_at": "2026-08-13T00:00:00Z",
                    "is_deleted": false,
                    "name": "Imported Profile",
                    "slug": "imported-profile",
                    "avatar_s3_hash": null,
                    "real_name": null,
                    "gender": null,
                    "birthday": null,
                    "location": null,
                    "biography": null,
                    "website": null,
                    "karma": 3,
                    "is_pro": false,
                }
            }
        })
    );

    let imported_by_id = run_endpoint!(
        runner,
        user_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "user": 700_011,
            "locales": ["en"],
        }),
    );
    let GetUserViewOutput::UserFound { user } = imported_by_id else {
        panic!("active imported numeric ID should remain found");
    };
    assert!(user.is_wikidot());
    assert_eq!(user.user_id(), 700_011);

    for target in ["deleted-imported-profile", "missing-imported-profile"] {
        let output = run_endpoint!(
            runner,
            user_view,
            json!({
                "site_id": site_id,
                "session_token": null,
                "user": target,
                "locales": ["en"],
            }),
        );
        assert!(
            matches!(output, GetUserViewOutput::UserMissing),
            "{target} must stay missing"
        );
    }

    let deleted_imported_id = run_endpoint!(
        runner,
        user_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "user": 700_012,
            "locales": ["en"],
        }),
    );
    assert!(
        matches!(deleted_imported_id, GetUserViewOutput::UserMissing),
        "deleted imported numeric ID must stay missing"
    );

    let local = run_endpoint!(
        runner,
        user_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "user": "user",
            "locales": ["en"],
        }),
    );
    let GetUserViewOutput::UserFound { user } = local else {
        panic!("seeded local user should remain found");
    };
    assert!(user.is_wikijump());
    assert_eq!(user.user_id(), SAMPLE_USER_ID);

    let local_by_id = run_endpoint!(
        runner,
        user_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "user": SAMPLE_USER_ID,
            "locales": ["en"],
        }),
    );
    let GetUserViewOutput::UserFound { user } = local_by_id else {
        panic!("seeded local numeric ID should remain found");
    };
    assert!(user.is_wikijump());
    assert_eq!(user.user_id(), SAMPLE_USER_ID);
}

#[tokio::test]
async fn oversized_avatar_rejection_does_not_promote_blob_or_change_target() {
    const AVATAR_DATA: &[u8] = b"\xf0\x0d";

    let mut config = Config::integration_testing();
    config.maximum_avatar_size = 1;
    config.maximum_blob_size = 2;
    let mut runner = TestRunner::setup_with_config(config).await;
    runner.set_request_context(RequestContext {
        user_id: Some(SAMPLE_USER_ID),
        ..Default::default()
    });

    let target_before =
        run_endpoint!(runner, user_get, json!({ "user": SAMPLE_USER_ID }),)
            .expect("sample user should exist")
            .user
            .unwrap_wikijump()
            .expect("sample user should be a Wikijump user");

    let pending_blob_id = cuid();
    let s3_path = format!("uploads/{pending_blob_id}");
    let avatar_hash = sha512_hash(AVATAR_DATA);
    assert!(
        BlobService::get_optional(runner.context(), &avatar_hash)
            .await
            .expect("avatar fixture permanent lookup should succeed")
            .is_none(),
        "avatar fixture hash must not already exist"
    );
    let upload = runner
        .state()
        .s3_files_bucket
        .put_object(&s3_path, AVATAR_DATA)
        .await
        .expect("avatar fixture upload should succeed");
    assert_eq!(upload.status_code(), 200);
    let created_at = OffsetDateTime::now_utc();
    blob_pending::ActiveModel {
        external_id: Set(pending_blob_id.clone()),
        created_by: Set(SAMPLE_USER_ID),
        created_at: Set(created_at),
        expires_at: Set(created_at + time::Duration::minutes(5)),
        expected_length: Set(2),
        s3_path: Set(s3_path.clone()),
        s3_hash: Set(None),
        presign_url: Set("not-used-in-test".to_owned()),
        site_id: Set(None),
        page_id: Set(None),
        content_type_label: Set(None),
        content_type_description: Set(None),
    }
    .insert(&runner.state().database)
    .await
    .expect("avatar pending blob fixture should be inserted");

    let error = run_endpoint_err!(
        runner,
        user_edit,
        json!({
            "user": SAMPLE_USER_ID,
            "avatar_uploaded_blob_id": pending_blob_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let target_after =
        run_endpoint!(runner, user_get, json!({ "user": SAMPLE_USER_ID }),)
            .expect("sample user should remain readable")
            .user
            .unwrap_wikijump()
            .expect("sample user should remain a Wikijump user");
    let permanent_after_rejection =
        BlobService::get_optional(runner.context(), &avatar_hash)
            .await
            .expect("rejected avatar permanent lookup should succeed");

    BlobPending::delete_by_id(&pending_blob_id)
        .exec(&runner.state().database)
        .await
        .expect("avatar pending blob fixture cleanup should succeed");
    runner
        .state()
        .s3_files_bucket
        .delete_object(&s3_path)
        .await
        .expect("avatar temporary fixture cleanup should succeed");
    if permanent_after_rejection.is_some() {
        // The hash was absent before this fixture ran, so an unexpected object here
        // was created by this request and is safe for the RED-run cleanup to remove.
        runner
            .state()
            .s3_files_bucket
            .delete_object(blob_hash_to_hex(&avatar_hash))
            .await
            .expect("unexpected promoted avatar fixture cleanup should succeed");
    }

    assert_contains_error!(error, ErrorType::BlobTooBig);
    assert_eq!(target_after.avatar_s3_hash, target_before.avatar_s3_hash);
    assert!(
        permanent_after_rejection.is_none(),
        "rejected avatar bytes must not reach permanent content-addressed storage"
    );
}

#[tokio::test]
async fn regular_account_password_policy_rejects_short_create_passwords() {
    let mut runner = TestRunner::setup().await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });

    let error = run_endpoint_err!(
        runner,
        user_create,
        json!({
            "user_type": "regular",
            "name": "Short Password User",
            "email": "short-password@example.invalid",
            "locales": ["en"],
            "password": "a",
            "bypass_email_verification": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    assert_contains_error!(
        error,
        ErrorType::UserPasswordTooShort {
            length: 1,
            minimum: 15,
        },
    );

    assert!(
        run_endpoint!(runner, user_get, json!({ "user": "short-password-user" }))
            .is_none()
    );

    let created = run_endpoint!(
        runner,
        user_create,
        json!({
            "user_type": "regular",
            "name": "Compliant Password User",
            "email": "compliant-password@example.invalid",
            "locales": ["en"],
            "password": "fifteen-chars!!",
            "bypass_email_verification": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    runner.set_request_context(RequestContext {
        user_id: Some(created.user_id),
        ..Default::default()
    });
    let before_password =
        run_endpoint!(runner, user_get, json!({ "user": created.user_id }))
            .unwrap()
            .user
            .unwrap_wikijump()
            .unwrap()
            .password;

    let update_error = run_endpoint_err!(
        runner,
        user_edit,
        json!({
            "user": created.user_id,
            "password": " ",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(
        update_error,
        ErrorType::UserPasswordTooShort {
            length: 1,
            minimum: 15,
        },
    );
    let unchanged_password =
        run_endpoint!(runner, user_get, json!({ "user": created.user_id }))
            .unwrap()
            .user
            .unwrap_wikijump()
            .unwrap()
            .password;
    assert_eq!(unchanged_password, before_password);

    let updated = run_endpoint!(
        runner,
        user_edit,
        json!({
            "user": created.user_id,
            "password": "ééééééééééééééé",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_ne!(updated.password, before_password);
}

#[tokio::test]
async fn display_locale_preferences_round_trip_only_for_the_session_actor() {
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
            "name": "Display Locale Settings User",
            "email": "display-locale-settings@example.invalid",
            "locales": ["en"],
            "password": "password-fixture",
            "bypass_email_verification": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    runner.set_request_context(RequestContext {
        user_id: Some(user.user_id),
        ..Default::default()
    });
    let updated = run_endpoint!(
        runner,
        user_edit,
        json!({
            "user": user.user_id,
            "locales": ["ja-JP", "en"],
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(updated.locales, ["ja-JP", "en"]);
    let persisted = run_endpoint!(runner, user_get, json!({"user": user.user_id}))
        .expect("display locale settings user should remain readable")
        .user
        .unwrap_wikijump()
        .expect("display locale settings belong to a Wikijump user");
    assert_eq!(persisted.locales, ["ja-JP", "en"]);

    let invalid = run_endpoint_err!(
        runner,
        user_edit,
        json!({
            "user": user.user_id,
            "locales": [],
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(invalid, ErrorType::User);

    runner.set_request_context(RequestContext {
        user_id: Some(SAMPLE_USER_ID),
        ..Default::default()
    });
    let denied = run_endpoint_err!(
        runner,
        user_edit,
        json!({
            "user": user.user_id,
            "locales": ["ko"],
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(denied, ErrorType::PermissionDenied);

    let unchanged = run_endpoint!(runner, user_get, json!({"user": user.user_id}))
        .expect("display locale settings user should remain readable")
        .user
        .unwrap_wikijump()
        .expect("display locale settings belong to a Wikijump user");
    assert_eq!(unchanged.locales, ["ja-JP", "en"]);
}

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
        avatar_s3_hash: Set(None),
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
            "password": "password-fixture",
            "bypass_email_verification": true,
            "override_user_id": user_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    assert_eq!(imported.user_id, user_id);
    assert_eq!(imported.slug, "imported-user");

    let output = run_endpoint!(runner, user_get, json!({ "user": "imported-user" }))
        .expect("imported Wikidot user should be fetchable as a Wikijump user");
    let imported_user = output
        .user
        .unwrap_wikijump()
        .expect("imported Wikidot user should now be a Wikijump user");
    assert_eq!(imported_user.user_id, user_id);
    assert_eq!(imported_user.slug, "imported-user");
}

#[tokio::test]
async fn real_user_lookup_treats_wikidot_only_identity_as_absent() {
    let runner = TestRunner::setup().await;
    let user_id = 700_010_i64;

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
        name: Set(Some("Wikidot Only User".to_owned())),
        slug: Set(Some("wikidot-only-user".to_owned())),
        avatar_s3_hash: Set(None),
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

    let generic = UserService::get_optional(runner.context(), Reference::Id(user_id))
        .await
        .expect("generic lookup should succeed")
        .expect("generic lookup should find the Wikidot identity");
    assert!(generic.is_wikidot());

    let real = UserService::get_real_optional(runner.context(), Reference::Id(user_id))
        .await
        .expect("optional real-user lookup should not error");
    assert!(real.is_none());

    let error = UserService::get_real(runner.context(), Reference::Id(user_id))
        .await
        .expect_err("strict real-user lookup should report a missing user");
    assert_contains_error!(error, ErrorType::UserNotFound);
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
            "password": "test-password-long",
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
            "password": "test-password-long",
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
            "password": "password-fixture",
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

    let user = output
        .user
        .unwrap_wikijump()
        .expect("Returned user was not of type Wikijump");

    assert_eq!(user.user_id, user_id);
    assert_eq!(user.name, USER_NAME);
    assert_eq!(user.slug, USER_SLUG);
    assert!(user.updated_at.is_none());
    assert!(user.deleted_at.is_none());
    assert_eq!(user.name_changes_left, 2); // set in Config::integration_testing()
    assert!(user.last_renamed_at.is_none());
    assert!(!user.password.is_empty());
    assert_eq!(user.email, "jane@private.me");
    assert!(user.email_verified_at.is_none());
    assert!(user.email_validation_info.is_some());
    assert!(user.email_validation_at.is_some());
    assert_eq!(user.locales.len(), 1);
    assert_eq!(&user.locales[0], "en_GB");
    assert!(user.real_name.is_none());
    assert!(user.gender.is_none());
    assert!(user.birthday.is_none());
    assert!(user.location.is_none());
    assert!(user.biography.is_none());
    assert!(user.user_page.is_none());
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

    let last_user = user;
    let output = run_endpoint!(runner, user_get, json!({ "user": user_id }))
        .expect("User does not exist");

    let user = output
        .user
        .unwrap_wikijump()
        .expect("Returned user not of type Wikijump");

    let birthday = Date::from_calendar_date(1986, Month::February, 1).unwrap();
    assert_eq!(user, last_user); // ensures that the model returned by user_edit is latest
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
            "password": "changed-password-fixture",
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
            "password": "password-fixture",
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
            "password": "changed-password-fixture",
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
            "password": "password-fixture",
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
            "password": "password-fixture",
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
            "password": "password-fixture",
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
            "password": "password-fixture",
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
            "password": "password-fixture",
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
            "password": "password-fixture",
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
            "password": "password-fixture",
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
    let error = run_endpoint_err!(
        runner,
        user_delete,
        json!({
            "user": target.user_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );
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

    let updated = run_endpoint!(
        runner,
        user_edit,
        json!({
            "user": target.user_id,
            "forum_signature": "**Forum signature**\nSecond line",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_str_eq!(
        updated.forum_signature,
        Some("**Forum signature**\nSecond line"),
    );

    for invalid_signature in ["x".repeat(401), "one\ntwo\nthree\nfour\nfive".to_owned()] {
        let error = run_endpoint_err!(
            runner,
            user_edit,
            json!({
                "user": target.user_id,
                "forum_signature": invalid_signature,
                "ip_address": common::IP_ADDRESS,
            }),
        );
        assert_contains_error!(error, ErrorType::BadRequest);
    }
    let unchanged = run_endpoint!(runner, user_get, json!({"user": target.user_id}))
        .expect("signature target should still exist")
        .user
        .unwrap_wikijump()
        .expect("signature target should remain a Wikijump user");
    assert_str_eq!(
        unchanged.forum_signature,
        Some("**Forum signature**\nSecond line"),
    );

    let cleared = run_endpoint!(
        runner,
        user_edit,
        json!({
            "user": target.user_id,
            "forum_signature": "",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(cleared.forum_signature.is_none());

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
    let deleted = run_endpoint!(
        runner,
        user_delete,
        json!({
            "user": target.user_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(deleted.user_id, target.user_id);
    assert!(deleted.deleted_at.is_some());
}

#[tokio::test]
async fn user_delete_audits_the_authenticated_actor_target_and_request_ip_once() {
    let mut runner = TestRunner::setup().await;
    let target = run_endpoint!(
        runner,
        user_create,
        json!({
            "user_type": "regular",
            "name": "Delete Audit Target",
            "email": "delete-audit-target@example.invalid",
            "locales": ["en"],
            "password": "password-fixture",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });

    let deleted = run_endpoint!(
        runner,
        user_delete,
        json!({
            "user": target.user_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(deleted.deleted_at.is_some());

    let events = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("user.delete"))
        .filter(AuditLogColumn::ExtraId1.eq(target.user_id))
        .all(runner.context().transaction())
        .await
        .expect("user deletion audit lookup should succeed");
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.user_id, Some(ADMIN_USER_ID));
    assert_eq!(event.extra_id_1, Some(target.user_id));
    assert_eq!(event.ip_address, common::IP_ADDRESS.to_string());
    assert_eq!(event.site_id, None);
    assert_eq!(event.page_id, None);
    assert_eq!(event.extra_id_2, None);
    assert_eq!(event.extra_string_1, None);
    assert_eq!(event.extra_string_2, None);
    assert_eq!(event.extra_number, None);

    run_endpoint!(
        runner,
        user_delete,
        json!({
            "user": target.user_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let event_count = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("user.delete"))
        .filter(AuditLogColumn::ExtraId1.eq(target.user_id))
        .count(runner.context().transaction())
        .await
        .expect("user deletion audit count should succeed");
    assert_eq!(
        event_count, 1,
        "an already-deleted user must not be audited"
    );
}

#[tokio::test]
async fn user_delete_denial_and_missing_target_do_not_emit_audit_events() {
    let mut runner = TestRunner::setup().await;
    let target = run_endpoint!(
        runner,
        user_create,
        json!({
            "user_type": "regular",
            "name": "Denied Delete Audit Target",
            "email": "denied-delete-audit-target@example.invalid",
            "locales": ["en"],
            "password": "password-fixture",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    runner.set_request_context(RequestContext {
        user_id: Some(SAMPLE_USER_ID),
        ..Default::default()
    });

    let error = run_endpoint_err!(
        runner,
        user_delete,
        json!({
            "user": target.user_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);
    let denied_count = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("user.delete"))
        .filter(AuditLogColumn::ExtraId1.eq(target.user_id))
        .count(runner.context().transaction())
        .await
        .expect("denied user deletion audit count should succeed");
    assert_eq!(denied_count, 0);

    const MISSING_USER_ID: i64 = 8_765_432_109;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    let error = run_endpoint_err!(
        runner,
        user_delete,
        json!({
            "user": MISSING_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::User);
    let missing_count = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("user.delete"))
        .filter(AuditLogColumn::ExtraId1.eq(MISSING_USER_ID))
        .count(runner.context().transaction())
        .await
        .expect("failed user deletion audit count should succeed");
    assert_eq!(missing_count, 0);
}

#[tokio::test]
async fn user_delete_and_its_audit_event_roll_back_together() {
    let mut runner = TestRunner::setup().await;
    let baseline_count = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("user.delete"))
        .filter(AuditLogColumn::ExtraId1.eq(SAMPLE_USER_ID))
        .count(runner.context().transaction())
        .await
        .expect("baseline user deletion audit count should succeed");
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    let deleted = run_endpoint!(
        runner,
        user_delete,
        json!({
            "user": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(deleted.deleted_at.is_some());
    let in_transaction_count = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("user.delete"))
        .filter(AuditLogColumn::ExtraId1.eq(SAMPLE_USER_ID))
        .count(runner.context().transaction())
        .await
        .expect("in-transaction user deletion audit count should succeed");
    assert_eq!(in_transaction_count, baseline_count + 1);
    runner.teardown().await;

    let runner = TestRunner::setup().await;
    let user = run_endpoint!(runner, user_get, json!({"user": SAMPLE_USER_ID}),)
        .expect("rolled-back user should remain present")
        .user
        .unwrap_wikijump()
        .expect("sample user should remain a Wikijump user");
    assert!(user.deleted_at.is_none());
    let rolled_back_count = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("user.delete"))
        .filter(AuditLogColumn::ExtraId1.eq(SAMPLE_USER_ID))
        .count(runner.context().transaction())
        .await
        .expect("rolled-back user deletion audit count should succeed");
    assert_eq!(rolled_back_count, baseline_count);
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
            "password": "password-fixture",
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

#[tokio::test]
async fn wikidot_user() {
    let mut runner = TestRunner::setup().await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });

    const USER_ID: i32 = 12345;
    const USER_NAME: &str = "Old Guy";
    const USER_SLUG: &str = "old-guy";

    // Set up Wikidot user record

    let ImportUserOutput { user_id } = run_endpoint!(
        runner,
        import_wikidot_user,
        json!({
            "user_id": USER_ID,
            "created_at": "2009-05-01T16:32:20+00:00",
            "fetched_at": "2026-02-02T10:00:00+00:00",
            "user_type": "extant",
            "name": USER_NAME,
            "slug": USER_SLUG,
            "avatar_uploaded_blob_id": null,
            "real_name": "Bob Smith",
            "gender": "male",
            "birthday": null,
            "location": null,
            "biography": "Just some old guy who made an account on Wikidot",
            "website": null,
            "karma": 2,
            "is_pro": false,
            "importing_user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(user_id, USER_ID, "Outputted user ID does not match input");

    // Check user data (Wikidot)

    fn check_wikidot_user(user: &WikidotUserModel) {
        assert_eq!(user.user_id, USER_ID);
        assert_eq!(user.created_at, datetime!(2009-05-01 16:32:20 UTC));
        assert_eq!(user.fetched_at, datetime!(2026-02-02 10:00:00 UTC));
        assert_str_eq!(user.name, Some(USER_NAME));
        assert_str_eq!(user.slug, Some(USER_SLUG));
        assert!(user.avatar_s3_hash.is_none());
        assert_str_eq!(user.real_name, Some("Bob Smith"));
        assert_str_eq!(user.gender, Some("male"));
        assert!(user.birthday.is_none());
        assert!(user.location.is_none());
        assert_str_eq!(
            user.biography,
            Some("Just some old guy who made an account on Wikidot"),
        );
        assert!(user.website.is_none());
        assert_eq!(user.karma, 2);
        assert!(!user.is_pro);
    }

    let output = run_endpoint!(runner, user_get, json!({ "user": USER_ID }))
        .expect("No user exists after Wikidot user creation");

    let user = output
        .user
        .unwrap_wikidot()
        .expect("Returned user was not of type Wikidot");

    check_wikidot_user(&user);

    // Activate user (Wikidot -> Wikijump)

    let wikijump_user = run_endpoint!(
        runner,
        user_activate_from_wikidot,
        json!({
            "user_id": USER_ID,
            "user_type": "regular",
            "email": "bob@wikijump",
            "locales": ["en-AU", "en"],
            "password": "password-fixture",
            "ip_address": common::IP_ADDRESS,
        }),
    );

    // Check user data (Wikijump)

    let output = run_endpoint!(runner, user_get, json!({ "user": USER_SLUG }))
        .expect("User does not exist");

    let user = output
        .user
        .unwrap_wikijump()
        .expect("Returned user not of type Wikijump");

    assert_eq!(
        user, wikijump_user,
        "Wikijump user data doesn't match returned",
    );
    assert_eq!(user.created_at, datetime!(2009-05-01 16:32:20 UTC));
    assert_eq!(user.name, USER_NAME);
    assert_eq!(user.slug, USER_SLUG);
    assert_eq!(user.email, "bob@wikijump");
    assert_eq!(user.locales, ["en-AU", "en"]);
    assert_str_eq!(user.real_name, Some("Bob Smith"));
    assert_str_eq!(user.gender, Some("male"));
    assert!(user.birthday.is_none());
    assert!(user.location.is_none());

    // Update Wikijump user data

    run_endpoint!(
        runner,
        user_edit,
        json!({
            "user": USER_ID,
            "real_name": "Robert A. Smith",
            "birthday": "1955-03-03",
            "location": "Australia",
            "ip_address": common::IP_ADDRESS,
        }),
    );

    // Check user data (Wikijump)

    let output = run_endpoint!(runner, user_get, json!({ "user": USER_SLUG }))
        .expect("User does not exist");

    let user = output
        .user
        .unwrap_wikijump()
        .expect("Returned user not of type Wikijump");

    assert_str_eq!(user.real_name, Some("Robert A. Smith"));
    assert_str_eq!(user.location, Some("Australia"));
    assert_eq!(user.birthday, Some(date!(1955 - 03 - 03)));

    // Check Wikidot user data hasn't changed
    // We need to manually query since it gets shadowed in UserService::get().

    let txn = runner.context().transaction();
    let user: WikidotUserModel = WikidotUser::find_by_id(USER_ID)
        .one(txn)
        .await
        .expect("Unable to fetch wikidot_user row")
        .expect("No wikidot_user row found");

    check_wikidot_user(&user);
}

// TODO test renames / rename tokens
//      test creating users of other types

async fn stored_avatar(runner: &TestRunner, user_id: i64) -> Option<Vec<u8>> {
    run_endpoint!(runner, user_get, json!({ "user": user_id }))
        .expect("avatar fixture user should exist")
        .user
        .unwrap_wikijump()
        .expect("fixture user should be a Wikijump user")
        .avatar_s3_hash
}

#[tokio::test]
async fn avatar_removal_is_owner_only_and_clears_only_the_association() {
    let mut runner = TestRunner::setup().await;

    let owner = run_endpoint!(
        runner,
        user_create,
        json!({
            "user_type": "regular",
            "name": "Avatar Owner",
            "email": "avatar-owner@example.invalid",
            "locales": ["en"],
            "password": "password-fixture",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let other = run_endpoint!(
        runner,
        user_create,
        json!({
            "user_type": "regular",
            "name": "Avatar Bystander",
            "email": "avatar-bystander@example.invalid",
            "locales": ["en"],
            "password": "password-fixture",
            "ip_address": common::IP_ADDRESS,
        }),
    );

    // Stored avatar hashes are set directly inside this test's transaction,
    // so the upload and blob storage paths are not involved.
    const OWNER_HASH: [u8; 64] = [0xab; 64];
    const OTHER_HASH: [u8; 64] = [0xcd; 64];
    for (user_id, hash) in [(owner.user_id, OWNER_HASH), (other.user_id, OTHER_HASH)] {
        user::ActiveModel {
            user_id: Set(user_id),
            avatar_s3_hash: Set(Some(hash.to_vec())),
            ..Default::default()
        }
        .update(runner.context().transaction())
        .await
        .expect("avatar fixture should be stored");
    }

    // An unauthenticated caller and another user cannot clear the owner's avatar.
    let error = run_endpoint_err!(
        runner,
        user_edit,
        json!({
            "user": owner.user_id,
            "avatar_uploaded_blob_id": null,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    runner.set_request_context(RequestContext {
        user_id: Some(other.user_id),
        ..Default::default()
    });
    let error = run_endpoint_err!(
        runner,
        user_edit,
        json!({
            "user": owner.user_id,
            "avatar_uploaded_blob_id": null,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);
    assert_eq!(
        stored_avatar(&runner, owner.user_id).await,
        Some(OWNER_HASH.to_vec())
    );

    // The owner's ordinary edit omits the avatar field, so it keeps the image.
    runner.set_request_context(RequestContext {
        user_id: Some(owner.user_id),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        user_edit,
        json!({
            "user": owner.user_id,
            "biography": "edited without touching the avatar",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(
        stored_avatar(&runner, owner.user_id).await,
        Some(OWNER_HASH.to_vec())
    );

    // The owner's explicit null clears only the owner's association.
    run_endpoint!(
        runner,
        user_edit,
        json!({
            "user": owner.user_id,
            "avatar_uploaded_blob_id": null,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(stored_avatar(&runner, owner.user_id).await, None);
    assert_eq!(
        stored_avatar(&runner, other.user_id).await,
        Some(OTHER_HASH.to_vec())
    );

    // Repeating the removal is harmless.
    run_endpoint!(
        runner,
        user_edit,
        json!({
            "user": owner.user_id,
            "avatar_uploaded_blob_id": null,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(stored_avatar(&runner, owner.user_id).await, None);
    assert_eq!(
        stored_avatar(&runner, other.user_id).await,
        Some(OTHER_HASH.to_vec())
    );
}
