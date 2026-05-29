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
use deepwell::error::prelude::*;
use serde_json::json;
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

// TODO test renames / rename tokens
//      test creating users of other types
