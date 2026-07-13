/*
 * tests/text.rs
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
use deepwell::hash::{k12_hash, text_hash_to_hex};
use deepwell::services::RequestContext;
use serde_json::json;

#[tokio::test]
async fn text() {
    let mut runner = TestRunner::setup().await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });

    // The string to use

    const TEXT_TO_STORE: &str = "Greetings fine whale! 🐳";
    const TEXT_HASH: &str = "923f4eee1cc5651277830ce7802dafe0";

    let expected_hash = k12_hash(TEXT_TO_STORE.as_bytes());
    let expected_hex_hash = text_hash_to_hex(&expected_hash);

    assert_eq!(
        TEXT_HASH,
        expected_hex_hash.as_str(),
        "Test hash value doesn't match, needs fix",
    );

    // Doesn't exist yet

    let error = run_endpoint_err!(runner, text_get, json!([TEXT_HASH]));
    assert_contains_error!(error, ErrorType::TextNotFound);

    // Insert

    let text_hash = run_endpoint!(runner, text_create, json!([TEXT_TO_STORE]));

    assert_eq!(
        text_hash_to_hex(text_hash.as_ref()),
        expected_hex_hash,
        "Actual text hash does not match expected",
    );

    // Fetch

    let text_result = run_endpoint!(runner, text_get, json!([TEXT_HASH]));
    assert_eq!(
        text_result, TEXT_TO_STORE,
        "Actual text contents does not match expected",
    );

    // Errors

    // Not a hex hash
    let error = run_endpoint_err!(
        runner,
        text_get,
        json!(["zzzzyyyyxxxxvvvvuuuuttttssssrrrr"]),
    );
    assert!(
        format!("{error:?}").contains("InvalidParams"),
        "JSONRPC InvalidParams error not returned:\n{:?}",
        error,
    );

    // Not the right length
    let error = run_endpoint_err!(runner, text_get, json!(["aaff0011"]));
    assert_contains_error!(error, ErrorType::BadRequest);
}
