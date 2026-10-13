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
        password: "test-password-long".to_owned(),
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
            "password": "test-password-long",
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
async fn import_user_retries_identical_snapshot_without_creating_local_account() {
    let runner = TestRunner::setup().await;
    let (user_id, _, fixture) = fixture_ids();
    let source_user = import_user(user_id, &fixture, "idempotent");

    ImportService::add_user(runner.context(), source_user)
        .await
        .expect("first authoritative source snapshot should import");
    ImportService::add_user(
        runner.context(),
        import_user(user_id, &fixture, "idempotent"),
    )
    .await
    .expect("an identical source snapshot should be an idempotent retry");

    assert!(
        KnownUser::find_by_id(i64::from(user_id))
            .one(runner.context().transaction())
            .await
            .expect("known-user lookup should succeed")
            .is_some(),
    );
    assert!(
        WikidotUser::find_by_id(user_id)
            .one(runner.context().transaction())
            .await
            .expect("Wikidot-user lookup should succeed")
            .is_some(),
    );
    assert!(
        User::find_by_id(i64::from(user_id))
            .one(runner.context().transaction())
            .await
            .expect("local-user lookup should succeed")
            .is_none(),
        "importing a source identity must not create an authenticatable local account",
    );
}

#[tokio::test]
async fn import_user_refuses_conflicting_snapshot_for_same_source_id() {
    let runner = TestRunner::setup().await;
    let (user_id, _, fixture) = fixture_ids();

    ImportService::add_user(runner.context(), import_user(user_id, &fixture, "original"))
        .await
        .expect("first source snapshot should import");
    let conflict = ImportService::add_user(
        runner.context(),
        import_user(user_id, &fixture, "contradiction"),
    )
    .await
    .expect_err("different identity data at the same capture time must fail");
    assert_contains_error!(conflict, ErrorType::DatabaseImport);

    let stored = WikidotUser::find_by_id(user_id)
        .one(runner.context().transaction())
        .await
        .expect("Wikidot-user lookup should succeed")
        .expect("original source identity should remain");
    assert_eq!(
        stored.name.as_deref(),
        Some(format!("Import original {fixture}").as_str())
    );
    assert_eq!(
        stored.slug.as_deref(),
        Some(format!("import-original-{fixture}").as_str())
    );
}

#[tokio::test]
async fn import_user_deleted_snapshot_clears_public_profile_without_local_account() {
    let runner = TestRunner::setup().await;
    let (user_id, _, fixture) = fixture_ids();
    let mut extant = import_user(user_id, &fixture, "deleted-later");
    extant.real_name = Some("Source public name".to_owned());
    extant.biography = Some("Source public biography".to_owned());
    ImportService::add_user(runner.context(), extant)
        .await
        .expect("extant source identity should import");

    let mut deleted = import_user(user_id, &fixture, "unused");
    deleted.fetched_at += time::Duration::seconds(1);
    deleted.wikidot_user_type = ImportedUserType::Deleted;
    // A deleted source row carries no public profile data, even if a caller
    // accidentally supplied stale values alongside the tombstone.
    deleted.real_name = Some("Must be cleared".to_owned());
    deleted.biography = Some("Must also be cleared".to_owned());
    ImportService::add_user(runner.context(), deleted)
        .await
        .expect("newer deletion tombstone should refresh the source identity");

    let stored = WikidotUser::find_by_id(user_id)
        .one(runner.context().transaction())
        .await
        .expect("Wikidot-user lookup should succeed")
        .expect("deleted source identity should remain as a tombstone");
    assert!(stored.is_deleted);
    assert!(stored.name.is_none());
    assert!(stored.slug.is_none());
    assert!(stored.real_name.is_none());
    assert!(stored.biography.is_none());
    assert!(stored.avatar_s3_hash.is_none());
    assert_eq!(stored.karma, i16::from(KarmaLevel::Zero.value()));
    assert!(!stored.is_pro);
    assert!(
        User::find_by_id(i64::from(user_id))
            .one(runner.context().transaction())
            .await
            .expect("local-user lookup should succeed")
            .is_none(),
        "importing a deleted source identity must not create an authenticatable local account",
    );
}

#[tokio::test]
async fn mirror_seed_carries_source_tags_and_navigation() {
    // T14 (issues #2077, #2079): the repository-owned scp-wiki mirror
    // bootstrap must materialize the source tag sets and the source
    // navigation pages instead of collapsing to generic seed content.
    //
    // Expected tag sets are the authoritative shared-corpus page metadata
    // (corpus/en/pages/<slug>/meta.json, status active; the 2026-10-11 run
    // for scp-002/scp-3352/scp-3922/scp-5382/scp-7000, and the scp-9506 set
    // corroborated by the 2026-10-07 live observation of 12 tags). The
    // expected nav:top source is corpus/en/pages/nav:top/source.wikidot.txt
    // from the same 2026-10-11 run.
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki mirror site should exist");
    let site_id = site.site.site_id;

    for (slug, expected) in [
        (
            "scp-002",
            vec![
                "_cc",
                "_licensebox",
                "alive",
                "body-horror",
                "euclid",
                "featured",
                "horror",
                "illustrated",
                "scp",
                "structure",
                "transfiguration",
            ],
        ),
        (
            "scp-3352",
            vec![
                "_cc",
                "_licensebox",
                "anomalous-event",
                "fire",
                "indestructible",
                "inscription",
                "neutralized",
                "reviewers-spotlight",
                "scp",
                "structure",
            ],
        ),
        (
            "scp-3922",
            vec![
                "_cc",
                "_licensebox",
                "cinema",
                "electronic",
                "inscription",
                "media",
                "metafiction",
                "safe",
                "scp",
                "three-moons-initiative",
            ],
        ),
        (
            "scp-5382",
            vec![
                "_cc",
                "director-scout",
                "doctor-blank",
                "document",
                "giftschreiber",
                "historical",
                "language",
                "medical",
                "ocular",
                "on-guard-43",
                "scp",
                "skeletal",
                "thaumiel",
                "thilo-zwist",
                "visual",
            ],
        ),
        (
            "scp-7000",
            vec![
                "7000",
                "_cc",
                "_licensebox",
                "agent-adams",
                "chaos-insurgency",
                "concept",
                "doctor-blank",
                "doctor-dan",
                "doctor-sokolsky",
                "doctor-wettle",
                "ettra",
                "euclid",
                "foundation-personnel",
                "goldbaker-reinz",
                "humanoid",
                "illustrated",
                "k-class-scenario",
                "no-return",
                "on-guard-43",
                "phenomenon",
                "probability",
                "reality-bending",
                "sauelsuesor",
                "scp",
                "the-engineer",
            ],
        ),
        (
            "scp-9506",
            vec![
                "airborne",
                "dystopian",
                "esoteric-class",
                "gaseous",
                "horror",
                "illustrated",
                "scp",
                "transfiguration",
                "uncontained",
                "_cc",
                "_cc4",
                "_licensebox",
            ],
        ),
    ] {
        let page =
            run_endpoint!(runner, page_get, json!({"site_id": site_id, "page": slug}),)
                .unwrap_or_else(|| panic!("seeded mirror page '{slug}' should exist"));
        let mut actual = page.tags.clone();
        actual.sort();
        let mut expected: Vec<String> = expected.into_iter().map(str::to_owned).collect();
        expected.sort();
        assert_eq!(
            actual, expected,
            "seeded mirror page '{slug}' must carry its full source tag set",
        );
    }

    let nav_top = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "nav:top",
            "details": {"wikitext": true},
        }),
    )
    .expect("seeded mirror nav:top should exist");
    let wikitext = nav_top.wikitext.expect("nav:top source should be readable");
    assert!(
        wikitext.contains("[[div class=\"top-bar\"]]"),
        "mirror nav:top must be the source top bar, not generic seed content",
    );
    assert!(
        wikitext.contains("About Us"),
        "mirror nav:top must expose the source navigation groups",
    );
    assert!(
        !wikitext.contains("Wikijump Blog"),
        "mirror nav:top must not fall back to generic seed navigation:\n{wikitext}",
    );

    // Control: freshly created native sites keep the generic navigation.
    let native = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded native test site should exist");
    let native_nav = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": native.site.site_id,
            "page": "nav:top",
            "details": {"wikitext": true},
        }),
    )
    .expect("native nav:top should exist");
    assert!(
        native_nav
            .wikitext
            .as_deref()
            .is_some_and(|wikitext| wikitext.contains("Wikijump Blog")),
        "native sites must retain the generic default navigation",
    );

    runner.teardown().await;
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
