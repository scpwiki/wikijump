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
use data_encoding::BASE32_NOPAD;
use deepwell::constants::ADMIN_USER_ID;
use deepwell::error::prelude::*;
use deepwell::models::audit_log::{Column as AuditLogColumn, Entity as AuditLogTable};
use deepwell::models::authorization_token::{
    Column as AuthorizationTokenColumn, Entity as AuthorizationTokenTable,
};
use deepwell::services::RequestContext;
use deepwell::services::password::PasswordService;
use deepwell::services::user::{CreateUser, UserService};
use deepwell::types::{Reference, UserType};
use rust_otp::{Algorithm as TotpAlgorithm, TOTP};
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
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

#[tokio::test]
async fn session_other_listing_and_invalidation_keep_current_session() {
    let runner = TestRunner::setup().await;
    let n = next_n();
    let (name, user_id) = create_auth_test_user(&runner, n, false).await;

    let first = run_endpoint!(runner, auth_login, login_params(&name));
    let current = run_endpoint!(runner, auth_login, login_params(&name));
    let third = run_endpoint!(runner, auth_login, login_params(&name));

    let sessions = run_endpoint!(
        runner,
        auth_session_get_others,
        json!({
            "user_id": user_id,
            "session_token": current.session_token,
        }),
    );
    assert_eq!(sessions.current.session_token, current.session_token);
    assert_eq!(sessions.others.len(), 2);
    assert!(
        sessions
            .others
            .iter()
            .any(|session| session.session_token == first.session_token)
    );
    assert!(
        sessions
            .others
            .iter()
            .any(|session| session.session_token == third.session_token)
    );

    let invalidated = run_endpoint!(
        runner,
        auth_session_invalidate_others,
        json!({
            "user_id": user_id,
            "session_token": current.session_token,
        }),
    );
    assert_eq!(invalidated, 2);

    assert!(
        run_endpoint!(
            runner,
            auth_session_get,
            json!([current.session_token.clone()])
        )
        .is_some()
    );
    assert!(
        run_endpoint!(runner, auth_session_get, json!([first.session_token])).is_none()
    );
    assert!(
        run_endpoint!(runner, auth_session_get, json!([third.session_token])).is_none()
    );
}

#[tokio::test]
async fn mfa_setup_reset_disable_and_totp_login_flow() {
    let runner = TestRunner::setup().await;
    let n = next_n();
    let (name, user_id) = create_auth_test_user(&runner, n, false).await;
    let login = run_endpoint!(runner, auth_login, login_params(&name));
    assert!(!login.needs_mfa);

    let setup = run_endpoint!(
        runner,
        auth_mfa_setup,
        json!({
            "user_id": user_id,
            "session_token": login.session_token,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(
        setup.recovery_codes.len(),
        runner.config().recovery_code_count,
    );

    let user = UserService::get_real(runner.context(), Reference::Id(user_id))
        .await
        .expect("user lookup after MFA setup should succeed");
    assert_eq!(
        user.multi_factor_secret.as_deref(),
        Some(setup.totp_secret.as_str())
    );
    assert_eq!(
        user.multi_factor_recovery_codes
            .as_ref()
            .expect("recovery hashes should be stored")
            .len(),
        runner.config().recovery_code_count,
    );

    let mfa_login = run_endpoint!(runner, auth_login, login_params(&name));
    assert!(mfa_login.needs_mfa);
    let secret_bytes = BASE32_NOPAD
        .decode(setup.totp_secret.as_bytes())
        .expect("generated TOTP secret should be valid base32");
    let totp = TOTP::builder()
        .secret(secret_bytes)
        .algorithm(TotpAlgorithm::SHA256)
        .digits(runner.config().totp_digits)
        .time_step(runner.config().totp_time_step)
        .build()
        .expect("TOTP builder should accept Deepwell configuration");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("test clock should be after the Unix epoch")
        .as_secs()
        .checked_add_signed(runner.config().totp_time_skew)
        .expect("configured TOTP time offset should produce a valid timestamp");
    let session_token = run_endpoint!(
        runner,
        auth_mfa_verify,
        json!({
            "session_token": mfa_login.session_token,
            "totp_or_code": totp.generate_at(timestamp).to_string(),
            "ip_address": common::IP_ADDRESS,
            "user_agent": "deepwell-auth-test-totp",
        }),
    );
    let session = run_endpoint!(runner, auth_session_get, json!([session_token]))
        .expect("TOTP-verified session should be normal");
    assert!(!session.restricted);

    let reset = run_endpoint!(
        runner,
        auth_mfa_reset_recovery,
        json!({
            "user_id": user_id,
            "session_token": login.session_token,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(
        reset.recovery_codes.len(),
        runner.config().recovery_code_count,
    );

    run_endpoint!(
        runner,
        auth_mfa_disable,
        json!({
            "user_id": user_id,
            "session_token": login.session_token,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let user = UserService::get_real(runner.context(), Reference::Id(user_id))
        .await
        .expect("user lookup after MFA disable should succeed");
    assert!(user.multi_factor_secret.is_none());
    assert!(user.multi_factor_recovery_codes.is_none());
}

#[tokio::test]
async fn authorization_token_issue_requires_admin_request_context() {
    let mut runner = TestRunner::setup().await;

    let anonymous_error = run_endpoint_err!(
        runner,
        auth_token_issue,
        json!({
            "type": "bot-user",
            "description": "anonymous bot authorization token",
            "creating_user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(anonymous_error, ErrorType::PermissionDenied);

    let n = next_n();
    let (_, user_id) = create_auth_test_user(&runner, n, false).await;
    runner.set_request_context(RequestContext {
        user_id: Some(user_id),
        ..Default::default()
    });

    let non_admin_error = run_endpoint_err!(
        runner,
        auth_token_issue,
        json!({
            "type": "bot-user",
            "description": "non-admin bot authorization token",
            "creating_user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(non_admin_error, ErrorType::PermissionDenied);

    let issued_count = AuthorizationTokenTable::find()
        .filter(AuthorizationTokenColumn::Description.is_in([
            "anonymous bot authorization token",
            "non-admin bot authorization token",
        ]))
        .count(runner.context().transaction())
        .await
        .expect("authorization token count should succeed");
    assert_eq!(issued_count, 0);
}

#[tokio::test]
async fn authorization_token_issue_uses_admin_request_actor() {
    let mut runner = TestRunner::setup().await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });

    let token = run_endpoint!(
        runner,
        auth_token_issue,
        json!({
            "type": "bot-user",
            "description": "authorized bot token from request actor",
            "creating_user_id": 12345,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(token.starts_with("B-"));

    let token_record = AuthorizationTokenTable::find()
        .filter(AuthorizationTokenColumn::TokenValue.eq(&token))
        .one(runner.context().transaction())
        .await
        .expect("authorization token lookup should succeed")
        .expect("authorization token should be persisted");
    assert_eq!(token_record.created_by, ADMIN_USER_ID);

    let audit_record = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("authorization_token.create"))
        .one(runner.context().transaction())
        .await
        .expect("authorization token audit lookup should succeed")
        .expect("authorization token issue should be audited");
    assert_eq!(audit_record.user_id, Some(ADMIN_USER_ID));
    assert_eq!(audit_record.ip_address, common::IP_ADDRESS.to_string());
    assert_eq!(audit_record.extra_string_1.as_deref(), Some("bot-user"));
    assert!(
        audit_record.extra_string_2.as_deref().is_some_and(
            |metadata| metadata.contains("authorized bot token from request actor")
        ),
        "audit metadata should include the submitted description",
    );
}
