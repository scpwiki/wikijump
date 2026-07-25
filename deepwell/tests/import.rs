/*
 * tests/import.rs
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
use deepwell::error::ErrorType;
use deepwell::models::known_user::Entity as KnownUser;
use deepwell::models::user::Entity as User;
use deepwell::models::wikidot_user::Entity as WikidotUser;
use deepwell::services::import::{ImportService, ImportUser, ImportedUserType};
use deepwell::services::user::{CreateUser, UserService};
use deepwell::services::{RequestContext, ServiceContext};
use deepwell::types::UserType;
use ftml::data::KarmaLevel;
use sea_orm::{EntityTrait, TransactionTrait};
use serde_json::json;
use time::OffsetDateTime;
use uuid::Uuid;

const ACTIVE_USER_ID_START: i64 = 20_000_000;

fn fixture_ids() -> (i32, i32, String) {
    let fixture = Uuid::new_v4();
    let random = u32::from_be_bytes(fixture.as_bytes()[..4].try_into().unwrap());
    let first = 15_000_000 + i32::try_from(random % 2_000_000).unwrap() * 2;
    (first, first + 1, fixture.to_string())
}

fn import_user(user_id: i32, fixture: &str, label: &str) -> ImportUser {
    ImportUser {
        user_id,
        created_at: OffsetDateTime::UNIX_EPOCH,
        fetched_at: OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(1),
        wikidot_user_type: ImportedUserType::Extant {
            name: format!("Import {label} {fixture}"),
            slug: format!("import-{label}-{fixture}"),
        },
        avatar_uploaded_blob_id: None,
        real_name: None,
        gender: None,
        birthday: None,
        location: None,
        biography: None,
        website: None,
        karma: KarmaLevel::Three,
        is_pro: false,
        importing_user_id: ADMIN_USER_ID,
        ip_address: common::IP_ADDRESS,
    }
}

fn active_user(user_id: Option<i64>, fixture: &str, label: &str) -> CreateUser {
    CreateUser {
        user_type: UserType::Regular,
        name: format!("Active {label} {fixture}"),
        email: format!("active-{label}-{fixture}@example.invalid"),
        locales: vec!["en".to_owned()],
        password: "test-password".to_owned(),
        bypass_filter: true,
        bypass_email_verification: true,
        override_user_id: user_id,
        ip_address: common::IP_ADDRESS,
    }
}

#[tokio::test]
async fn import_user_creates_known_user_and_can_be_reclaimed() {
    let mut runner = TestRunner::setup().await;
    let (user_id, _, fixture) = fixture_ids();

    ImportService::add_user(runner.context(), import_user(user_id, &fixture, "missing"))
        .await
        .expect("Wikidot user with a missing known_user row should import");

    assert!(
        KnownUser::find_by_id(i64::from(user_id))
            .one(runner.context().transaction())
            .await
            .expect("known_user lookup should succeed")
            .is_some(),
    );
    assert!(
        WikidotUser::find_by_id(user_id)
            .one(runner.context().transaction())
            .await
            .expect("wikidot_user lookup should succeed")
            .is_some(),
    );

    let reclaimed = UserService::import_wikidot(
        runner.context(),
        active_user(Some(i64::from(user_id)), &fixture, "reclaimed"),
    )
    .await
    .expect("active account should reuse the imported known_user row");
    assert_eq!(reclaimed.user_id, i64::from(user_id));
    assert!(
        User::find_by_id(reclaimed.user_id)
            .one(runner.context().transaction())
            .await
            .expect("active user lookup should succeed")
            .is_some(),
    );

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    let sequenced = run_endpoint!(
        runner,
        user_create,
        json!({
            "user_type": "regular",
            "name": format!("Active Sequenced {fixture}"),
            "email": format!("active-sequenced-{fixture}@example.invalid"),
            "locales": ["en"],
            "password": "test-password",
            "bypass_filter": true,
            "bypass_email_verification": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(sequenced.user_id >= ACTIVE_USER_ID_START);
    assert_ne!(sequenced.user_id, i64::from(user_id));
}

#[tokio::test]
async fn import_user_reuses_known_user_for_existing_active_account() {
    let runner = TestRunner::setup().await;
    let (user_id, _, fixture) = fixture_ids();
    let user_id = i64::from(user_id);

    UserService::create(
        runner.context(),
        active_user(Some(user_id), &fixture, "existing"),
    )
    .await
    .expect("active account fixture should create its known_user row");

    ImportService::add_user(
        runner.context(),
        import_user(i32::try_from(user_id).unwrap(), &fixture, "coexisting"),
    )
    .await
    .expect("Wikidot import should reuse an active account's known_user row");

    assert!(
        KnownUser::find_by_id(user_id)
            .one(runner.context().transaction())
            .await
            .expect("known_user lookup should succeed")
            .is_some(),
    );
    assert!(
        User::find_by_id(user_id)
            .one(runner.context().transaction())
            .await
            .expect("active user lookup should succeed")
            .is_some(),
    );
    assert!(
        WikidotUser::find_by_id(i32::try_from(user_id).unwrap())
            .one(runner.context().transaction())
            .await
            .expect("wikidot_user lookup should succeed")
            .is_some(),
    );
}

#[tokio::test]
async fn failed_import_rolls_back_new_known_user_with_caller_transaction() {
    let runner = TestRunner::setup().await;
    let state = runner.state().clone();
    let (existing_id, failed_id, fixture) = fixture_ids();
    let txn = state
        .database
        .begin()
        .await
        .expect("caller transaction should begin");
    let ctx = ServiceContext::new(&state, &txn);

    ImportService::add_user(&ctx, import_user(existing_id, &fixture, "collision"))
        .await
        .expect("first import should establish the unique name and slug");
    let error =
        ImportService::add_user(&ctx, import_user(failed_id, &fixture, "collision"))
            .await
            .expect_err("duplicate Wikidot identity should fail");
    assert_contains_error!(error, ErrorType::DatabaseImport);

    drop(ctx);
    txn.rollback()
        .await
        .expect("caller should be able to roll back the failed import");

    for user_id in [existing_id, failed_id] {
        assert!(
            KnownUser::find_by_id(i64::from(user_id))
                .one(&state.database)
                .await
                .expect("known_user lookup after rollback should succeed")
                .is_none(),
            "known_user row {user_id} survived transaction rollback",
        );
        assert!(
            WikidotUser::find_by_id(user_id)
                .one(&state.database)
                .await
                .expect("wikidot_user lookup after rollback should succeed")
                .is_none(),
            "wikidot_user row {user_id} survived transaction rollback",
        );
    }
}
