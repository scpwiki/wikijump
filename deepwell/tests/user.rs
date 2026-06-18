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
use deepwell::models::wikidot_user::{Entity as WikidotUser, Model as WikidotUserModel};
use deepwell::services::import::ImportUserOutput;
use sea_orm::EntityTrait;
use serde_json::json;
use time::macros::{date, datetime};
use time::{Date, Month};

#[tokio::test]
async fn basic_update() {
    let runner = TestRunner::setup().await;

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
            "password": "letmein",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_ne!(user.password, old_password);
}

#[tokio::test]
async fn wikidot_user() {
    let runner = TestRunner::setup().await;

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
            "password": "hunter2",
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
