/*
 * tests/rpc_boundary.rs
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

use data_encoding::BASE32_NOPAD;
use deepwell::api::{build_server_at, build_server_state_without_workers};
use deepwell::config::{Config, Secrets};
use deepwell::constants::ADMIN_USER_ID;
use deepwell::error::ErrorType;
use deepwell::models::known_user::Entity as KnownUser;
use deepwell::models::session::Entity as SessionTable;
use deepwell::services::session::CreateSession;
use deepwell::services::{ServiceContext, SessionService};
use rust_otp::{Algorithm as TotpAlgorithm, TOTP};
use sea_orm::{ConnectionTrait, EntityTrait, TransactionTrait};
use serde_json::{Value, json};
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{SystemTime, UNIX_EPOCH};

async fn rpc_request(method: &str, params: Value) -> Value {
    let state = build_server_state_without_workers(
        Config::integration_testing(),
        Secrets::load(),
    )
    .await
    .expect("Unable to set up server state");
    let (address, handle) =
        build_server_at(state, SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
            .await
            .expect("Unable to start RPC server");

    let response = reqwest::Client::new()
        .post(format!("http://{address}"))
        .bearer_auth(
            env::var("DEEPWELL_RPC_TOKEN").expect("test RPC token must be configured"),
        )
        .header("X-Deepwell-Site-Id", "42")
        .header("X-Deepwell-Page", "category:page")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        }))
        .send()
        .await
        .expect("RPC request should complete")
        .json()
        .await
        .expect("RPC response should be JSON");

    handle.stop().expect("RPC server should stop");
    handle.stopped().await;
    response
}

async fn local_authoring_request(
    client: &reqwest::Client,
    address: SocketAddr,
    site_id: i64,
    marker: bool,
    id: u64,
    params: Value,
) -> Value {
    let mut request = client
        .post(format!("http://{address}"))
        .bearer_auth(
            env::var("DEEPWELL_RPC_TOKEN").expect("test RPC token must be configured"),
        )
        .header("X-Deepwell-Site-Id", site_id.to_string());
    if marker {
        request = request.header("X-Deepwell-Local-Page-Mutation-Actor", "-1");
    }
    request
        .json(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "page_create",
            "params": params,
        }))
        .send()
        .await
        .expect("local-authoring RPC request should complete")
        .json()
        .await
        .expect("local-authoring RPC response should be JSON")
}

#[tokio::test]
async fn production_rpc_binds_local_page_actor_only_to_the_editable_site() {
    let mut config = Config::integration_testing();
    config.main_domain_no_dot = "wikijump.localhost".to_owned();
    let state = build_server_state_without_workers(config, Secrets::load())
        .await
        .expect("local-authoring RPC state should build");
    let (address, handle) =
        build_server_at(state, SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
            .await
            .expect("local-authoring RPC server should start");
    let client = reqwest::Client::new();

    let local_site = rpc_request("site_get", json!({"site": "scpaiueouiuiuiui"})).await;
    let local_site_id = local_site["result"]["site_id"]
        .as_i64()
        .expect("seeded editable local site should exist");
    let mirror = rpc_request("site_get", json!({"site": "scp-wiki"})).await;
    let mirror_site_id = mirror["result"]["site_id"]
        .as_i64()
        .expect("seeded SCP-Wiki mirror should exist");
    let slug = format!(
        "local-actor-boundary-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after UNIX epoch")
            .as_nanos()
    );
    let page_params = |site_id| {
        json!({
            "site_id": site_id,
            "wikitext": "Local authoring actor RPC boundary fixture.",
            "title": "Local authoring actor RPC boundary fixture",
            "alt_title": null,
            "tags": [],
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "Exercise the local authoring RPC actor boundary",
            "user_id": -1,
            "ip_address": "127.0.0.1",
        })
    };

    let accepted = local_authoring_request(
        &client,
        address,
        local_site_id,
        true,
        1,
        page_params(local_site_id),
    )
    .await;
    assert!(
        accepted.get("result").is_some(),
        "the trusted local-site marker should bind the local actor: {accepted}"
    );

    let mirror_denied = local_authoring_request(
        &client,
        address,
        mirror_site_id,
        true,
        2,
        page_params(mirror_site_id),
    )
    .await;
    assert!(
        mirror_denied.get("error").is_some(),
        "the local actor must not bind to the mirror request site: {mirror_denied}"
    );

    let cross_site_denied = local_authoring_request(
        &client,
        address,
        mirror_site_id,
        true,
        3,
        page_params(local_site_id),
    )
    .await;
    assert!(
        cross_site_denied.get("error").is_some(),
        "a cross-site target must not bind the local actor: {cross_site_denied}"
    );

    let missing_marker_denied = local_authoring_request(
        &client,
        address,
        local_site_id,
        false,
        4,
        page_params(local_site_id),
    )
    .await;
    assert!(
        missing_marker_denied.get("error").is_some(),
        "ordinary anonymous requests must remain unauthenticated: {missing_marker_denied}"
    );

    handle
        .stop()
        .expect("local-authoring RPC server should stop");
    handle.stopped().await;
}

#[tokio::test]
async fn production_rpc_stack_dispatches_registered_method() {
    let response = rpc_request("echo", json!({"value": 42})).await;
    assert_eq!(response["result"], json!({"value": 42}));
    assert_eq!(response["id"], 1);
}

#[tokio::test]
async fn production_rpc_stack_converts_endpoint_errors() {
    let response = rpc_request("error", json!([])).await;
    assert!(response.get("result").is_none());
    assert_eq!(response["error"]["code"], ErrorType::BadRequest.code());
    assert_eq!(
        response["error"]["message"],
        ErrorType::BadRequest.summary(),
    );
    assert!(
        response["error"]["data"]["call_trace"]
            .as_str()
            .is_some_and(|trace| trace.contains("always fails")),
    );
}

#[tokio::test]
async fn production_rpc_stack_registers_page_watchers() {
    let response =
        rpc_request("page_watchers", json!({"site_id": 42, "page_id": 42})).await;

    assert_eq!(response["error"]["code"], ErrorType::PageNotFound.code());
    assert_ne!(response["error"]["code"], -32601);
}

#[tokio::test]
async fn production_rpc_stack_registers_page_who_rated() {
    let response =
        rpc_request("page_who_rated", json!({"site_id": 42, "page_id": 42})).await;

    assert_eq!(
        response["error"]["code"],
        ErrorType::PermissionDenied.code()
    );
    assert_ne!(response["error"]["code"], -32601);
}

async fn rpc_request_with_session(
    session_token: &str,
    method: &str,
    params: Value,
) -> Value {
    let state = build_server_state_without_workers(
        Config::integration_testing(),
        Secrets::load(),
    )
    .await
    .expect("Unable to set up server state");
    let (address, handle) =
        build_server_at(state, SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
            .await
            .expect("Unable to start RPC server");

    let response = reqwest::Client::new()
        .post(format!("http://{address}"))
        .bearer_auth(
            env::var("DEEPWELL_RPC_TOKEN").expect("test RPC token must be configured"),
        )
        .header("X-Deepwell-Site-Id", "42")
        .header("X-Deepwell-Page", "category:page")
        .header("X-Deepwell-Session-Token", session_token)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        }))
        .send()
        .await
        .expect("RPC request should complete")
        .json()
        .await
        .expect("RPC response should be JSON");

    handle.stop().expect("RPC server should stop");
    handle.stopped().await;
    response
}

/// A request that fails for an internal reason must roll back every write it
/// performed, even though a terminal MFA rejection is allowed to commit. The
/// production RPC wrapper owns that decision, so the boundary is proved here
/// rather than through a direct endpoint call.
#[tokio::test]
async fn production_rpc_stack_rolls_back_internal_failure_writes() {
    let state = build_server_state_without_workers(
        Config::integration_testing(),
        Secrets::load(),
    )
    .await
    .expect("Unable to set up server state");

    // A platform-staff session authorizes the privileged creation path that
    // inserts a known_user placeholder before its remaining validation runs.
    let session_token = {
        let txn = state
            .database
            .begin()
            .await
            .expect("Unable to start session transaction");
        let ctx = ServiceContext::new(&state, &txn);
        let token = SessionService::create(
            &ctx,
            CreateSession {
                user_id: ADMIN_USER_ID,
                ip_address: IpAddr::V4(Ipv4Addr::LOCALHOST),
                user_agent: "rpc-boundary-rollback-probe".to_owned(),
                restricted: false,
            },
        )
        .await
        .expect("admin session should be created");
        txn.commit()
            .await
            .expect("session transaction should commit");
        token
    };

    // Unique per run so a committed placeholder from a previous run cannot make
    // the probe take the "already exists" branch before its validation.
    let probe_id = 1_000_000_000_i64
        + (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .subsec_nanos() as i64)
            .rem_euclid(500_000_000);

    let response = rpc_request_with_session(
        &session_token,
        "user_create",
        json!({
            "user_type": "regular",
            "name": "rpc-boundary-rollback-probe",
            "email": "rpc-boundary-rollback-probe@example.com",
            "locales": ["en", "en"],
            "password": "password",
            "bypass_filter": true,
            "bypass_email_verification": true,
            "override_user_id": probe_id,
            "ip_address": "192.0.2.1",
        }),
    )
    .await;

    assert!(
        response.get("result").is_none(),
        "duplicate locales must be rejected: {response}",
    );
    assert_eq!(
        response["error"]["code"],
        ErrorType::BadRequest.code(),
        "the probe must reach locale validation after its placeholder write: {response}",
    );

    let txn = state
        .database
        .begin()
        .await
        .expect("Unable to start verification transaction");
    let placeholder = KnownUser::find_by_id(probe_id)
        .one(&txn)
        .await
        .expect("known_user lookup should succeed");
    txn.rollback()
        .await
        .expect("verification transaction should roll back");

    assert!(
        placeholder.is_none(),
        "an internal failure must roll back the known_user placeholder insert",
    );

    // Best-effort cleanup so repeated runs in the shared task-owned database
    // never observe a leftover placeholder or probe session.
    state
        .database
        .execute_unprepared(&format!(
            "DELETE FROM known_user WHERE user_id = {probe_id}"
        ))
        .await
        .ok();
    state
        .database
        .execute_unprepared(&format!(
            "DELETE FROM session WHERE session_token = '{}'",
            session_token.replace('\'', "''"),
        ))
        .await
        .ok();
}

/// The terminal MFA rejection is the one error path that must commit: the
/// restricted session's failed-attempt counter has to survive the rejected
/// request. The production RPC wrapper owns that decision, so the boundary is
/// proved here rather than through a direct endpoint call.
#[tokio::test]
async fn production_rpc_stack_commits_terminal_mfa_rejection_state() {
    let state = build_server_state_without_workers(
        Config::integration_testing(),
        Secrets::load(),
    )
    .await
    .expect("Unable to set up server state");

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    let name = format!("Rpc Boundary Mfa {suffix}");
    let email = format!("rpc-boundary-mfa-{suffix}@example.com");
    let password = "rpc-boundary-mfa-password";

    let created = rpc_request(
        "user_create",
        json!({
            "user_type": "regular",
            "name": name,
            "email": email,
            "locales": ["en"],
            "password": password,
            "ip_address": "192.0.2.1",
        }),
    )
    .await;
    let user_id = created["result"]["user_id"]
        .as_i64()
        .unwrap_or_else(|| panic!("user_create should return a user id: {created}"));

    let login = rpc_request(
        "login",
        json!({
            "name_or_email": name,
            "password": password,
            "ip_address": "192.0.2.1",
            "user_agent": "rpc-boundary-mfa",
        }),
    )
    .await;
    assert_eq!(
        login["result"]["needs_mfa"],
        json!(false),
        "first login should not require MFA: {login}",
    );
    let session_token = login["result"]["session_token"]
        .as_str()
        .expect("login should return a session token")
        .to_owned();

    let setup = rpc_request(
        "mfa_setup",
        json!({
            "user_id": user_id,
            "session_token": session_token,
            "ip_address": "192.0.2.1",
        }),
    )
    .await;
    let secret = setup["result"]["totp_secret"]
        .as_str()
        .unwrap_or_else(|| panic!("mfa_setup should return a TOTP secret: {setup}"))
        .to_owned();

    let mfa_login = rpc_request(
        "login",
        json!({
            "name_or_email": name,
            "password": password,
            "ip_address": "192.0.2.1",
            "user_agent": "rpc-boundary-mfa",
        }),
    )
    .await;
    assert_eq!(
        mfa_login["result"]["needs_mfa"],
        json!(true),
        "second login should require MFA: {mfa_login}",
    );
    let restricted_token = mfa_login["result"]["session_token"]
        .as_str()
        .expect("MFA login should return a restricted session token")
        .to_owned();

    // Build a code that is certainly wrong for the current time step.
    let secret_bytes = BASE32_NOPAD
        .decode(secret.as_bytes())
        .expect("generated TOTP secret should be valid base32");
    let totp = TOTP::builder()
        .secret(secret_bytes)
        .algorithm(TotpAlgorithm::SHA256)
        .digits(state.config.totp_digits)
        .time_step(state.config.totp_time_step)
        .build()
        .expect("TOTP builder should accept Deepwell configuration");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_secs()
        .checked_add_signed(state.config.totp_time_skew)
        .expect("configured TOTP time offset should produce a valid timestamp");
    let valid_code = totp.generate_at(timestamp).to_string();
    let modulus = 10_u32.pow(state.config.totp_digits);
    let wrong_code = (valid_code
        .parse::<u32>()
        .expect("TOTP code should be numeric")
        + 1)
    .checked_rem(modulus)
    .expect("TOTP modulus should be nonzero")
    .to_string();

    let rejected = rpc_request(
        "mfa_verify",
        json!({
            "session_token": restricted_token,
            "totp_or_code": wrong_code,
            "ip_address": "192.0.2.1",
            "user_agent": "rpc-boundary-mfa",
        }),
    )
    .await;
    assert_eq!(
        rejected["error"]["code"],
        ErrorType::InvalidAuthentication.code(),
        "a wrong TOTP must be rejected: {rejected}",
    );

    let txn = state
        .database
        .begin()
        .await
        .expect("Unable to start verification transaction");
    let session = SessionTable::find_by_id(restricted_token.clone())
        .one(&txn)
        .await
        .expect("session lookup should succeed");
    txn.rollback()
        .await
        .expect("verification transaction should roll back");

    assert_eq!(
        session
            .expect("restricted session should exist")
            .mfa_failed_attempts,
        1,
        "a terminal MFA rejection must persist the failed-attempt counter",
    );

    // Best-effort cleanup for repeated runs in the shared task-owned database.
    for statement in [
        format!("DELETE FROM session WHERE user_id = {user_id}"),
        format!("DELETE FROM \"user\" WHERE user_id = {user_id}"),
        format!("DELETE FROM known_user WHERE user_id = {user_id}"),
    ] {
        state.database.execute_unprepared(&statement).await.ok();
    }
}
