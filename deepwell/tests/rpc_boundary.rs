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

use deepwell::api::{build_server_at, build_server_state_without_workers};
use deepwell::config::{Config, Secrets};
use deepwell::error::ErrorType;
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

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
