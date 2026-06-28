/*
 * tests/auth.rs
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
use deepwell::services::password::PasswordService;
use deepwell::services::user::{CreateUser, UserService};
use deepwell::types::UserType;
use sea_orm::ActiveValue;
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use str_macro::str;

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);
const PASSWORD: &str = "password";
const RECOVERY_CODE: &str = "mfa-recovery-code";

fn next_n() -> u64 {
    FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

async fn create_auth_test_user(runner: &TestRunner, n: u64, mfa: bool) -> (String, i64) {
    let name = format!("Auth Test {n}");
    let user = UserService::create(
        runner.context(),
        CreateUser {
            user_type: UserType::Regular,
            name: name.clone(),
            email: format!("auth-test-{n}@email.com"),
            locales: vec![str!("en")],
            password: PASSWORD.to_owned(),
            bypass_filter: true,
            bypass_email_verification: true,
            override_user_id: None,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("auth test user should be created");

    if mfa {
        let recovery_hash =
            PasswordService::new_hash(RECOVERY_CODE).expect("recovery code should hash");
        UserService::set_mfa_secrets(
            runner.context(),
            user.user_id,
            ActiveValue::Set(Some("ABCDEFGHIJKLMNOP".to_owned())),
            ActiveValue::Set(Some(vec![recovery_hash])),
        )
        .await
        .expect("auth test user should have MFA enabled");
    }

    (name, user.user_id)
}

fn login_params(name: &str) -> serde_json::Value {
    json!({
        "name_or_email": name,
        "password": PASSWORD,
        "ip_address": common::IP_ADDRESS,
        "user_agent": "deepwell-auth-test",
    })
}

#[tokio::test]
async fn restricted_mfa_sessions_are_not_normal_login_sessions() {
    let runner = TestRunner::setup().await;
    let n = next_n();
    let (name, user_id) = create_auth_test_user(&runner, n, true).await;

    let restricted_login = run_endpoint!(runner, auth_login, login_params(&name));
    assert!(restricted_login.needs_mfa);

    let restricted_session = run_endpoint!(
        runner,
        auth_session_get,
        json!([restricted_login.session_token])
    );
    assert!(
        restricted_session.is_none(),
        "restricted MFA session must not be returned by normal session_get",
    );

    let renew_login = run_endpoint!(runner, auth_login, login_params(&name));
    assert!(renew_login.needs_mfa);
    let renew_error = run_endpoint_err!(
        runner,
        auth_session_renew,
        json!([{
            "old_session_token": renew_login.session_token,
            "user_id": user_id,
            "ip_address": common::IP_ADDRESS,
            "user_agent": "deepwell-auth-test-renew",
        }]),
    );
    assert_contains_error!(renew_error, ErrorType::InvalidSessionToken);

    let mfa_login = run_endpoint!(runner, auth_login, login_params(&name));
    assert!(mfa_login.needs_mfa);
    let session_token = run_endpoint!(
        runner,
        auth_mfa_verify,
        json!({
            "session_token": mfa_login.session_token,
            "totp_or_code": RECOVERY_CODE,
            "ip_address": common::IP_ADDRESS,
            "user_agent": "deepwell-auth-test-mfa",
        }),
    );
    let session = run_endpoint!(runner, auth_session_get, json!([session_token]))
        .expect("MFA-verified session should be a normal session");
    assert!(!session.restricted);

    let replay_error = run_endpoint_err!(
        runner,
        auth_mfa_verify,
        json!({
            "session_token": mfa_login.session_token,
            "totp_or_code": RECOVERY_CODE,
            "ip_address": common::IP_ADDRESS,
            "user_agent": "deepwell-auth-test-mfa-replay",
        }),
    );
    assert_contains_error!(replay_error, ErrorType::InvalidSessionToken);
}

#[tokio::test]
async fn unrestricted_sessions_still_get_and_renew_normally() {
    let runner = TestRunner::setup().await;
    let n = next_n();
    let (name, _) = create_auth_test_user(&runner, n, false).await;

    let login = run_endpoint!(runner, auth_login, login_params(&name));
    assert!(!login.needs_mfa);

    let session = run_endpoint!(
        runner,
        auth_session_get,
        json!([login.session_token.clone()])
    )
    .expect("normal login session should be returned by session_get");
    assert!(!session.restricted);

    let renewed = run_endpoint!(
        runner,
        auth_session_renew,
        json!([{
            "old_session_token": login.session_token,
            "user_id": session.user_id,
            "ip_address": common::IP_ADDRESS,
            "user_agent": "deepwell-auth-test-renew",
        }]),
    );
    let renewed_session = run_endpoint!(runner, auth_session_get, json!([renewed]))
        .expect("renewed normal session should be returned by session_get");
    assert!(!renewed_session.restricted);
}
