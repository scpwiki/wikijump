/*
 * tests/blob.rs
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
use deepwell::models::blob_blacklist::Entity as BlobBlacklistTable;
use deepwell::services::RequestContext;
use sea_orm::EntityTrait;
use serde_json::json;

const TEST_BLOB_HASH: &str = "11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111";

#[tokio::test]
async fn blob_hard_delete_requires_admin_request_context() {
    let runner = TestRunner::setup().await;

    let preview_error = run_endpoint_err!(
        runner,
        blob_hard_delete_preview,
        json!({ "s3_hash": TEST_BLOB_HASH }),
    );
    assert_contains_error!(preview_error, ErrorType::PermissionDenied);

    let confirm_error = run_endpoint_err!(
        runner,
        blob_hard_delete_confirm,
        json!({ "s3_hash": TEST_BLOB_HASH, "user_id": ADMIN_USER_ID }),
    );
    assert_contains_error!(confirm_error, ErrorType::PermissionDenied);
}

#[tokio::test]
async fn blob_hard_delete_uses_admin_request_actor() {
    let mut runner = TestRunner::setup().await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });

    let preview = run_endpoint!(
        runner,
        blob_hard_delete_preview,
        json!({ "s3_hash": TEST_BLOB_HASH }),
    );
    assert_eq!(preview.total_revisions, 0);
    assert_eq!(preview.total_files, 0);

    let confirm = run_endpoint!(
        runner,
        blob_hard_delete_confirm,
        json!({ "s3_hash": TEST_BLOB_HASH, "user_id": 12345 }),
    );
    assert_eq!(confirm.total_revisions, 0);
    assert_eq!(confirm.total_files, 0);

    let hash = hex::decode(TEST_BLOB_HASH).expect("valid test blob hash");
    let blacklist = BlobBlacklistTable::find_by_id(hash)
        .one(runner.context().transaction())
        .await
        .expect("blob blacklist lookup should succeed")
        .expect("hard delete should blacklist the blob hash");
    assert_eq!(blacklist.created_by, ADMIN_USER_ID);
}

#[tokio::test]
async fn blob_blacklist_mutations_require_admin_request_context() {
    let mut runner = TestRunner::setup().await;

    let add_error = run_endpoint_err!(
        runner,
        blob_blacklist_add,
        json!({ "s3_hash": TEST_BLOB_HASH, "user_id": ADMIN_USER_ID }),
    );
    assert_contains_error!(add_error, ErrorType::PermissionDenied);

    let remove_error = run_endpoint_err!(
        runner,
        blob_blacklist_remove,
        json!({ "s3_hash": TEST_BLOB_HASH }),
    );
    assert_contains_error!(remove_error, ErrorType::PermissionDenied);

    runner.set_request_context(RequestContext {
        user_id: Some(12345),
        ..Default::default()
    });
    let non_admin_add_error = run_endpoint_err!(
        runner,
        blob_blacklist_add,
        json!({ "s3_hash": TEST_BLOB_HASH }),
    );
    assert_contains_error!(non_admin_add_error, ErrorType::PermissionDenied);

    let non_admin_remove_error = run_endpoint_err!(
        runner,
        blob_blacklist_remove,
        json!({ "s3_hash": TEST_BLOB_HASH }),
    );
    assert_contains_error!(non_admin_remove_error, ErrorType::PermissionDenied);
}

#[tokio::test]
async fn blob_blacklist_uses_admin_request_actor() {
    let mut runner = TestRunner::setup().await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });

    run_endpoint!(
        runner,
        blob_blacklist_add,
        json!({ "s3_hash": TEST_BLOB_HASH, "user_id": 12345 }),
    );

    let hash = hex::decode(TEST_BLOB_HASH).expect("valid test blob hash");
    let blacklist = BlobBlacklistTable::find_by_id(hash.clone())
        .one(runner.context().transaction())
        .await
        .expect("blob blacklist lookup should succeed")
        .expect("admin request should blacklist the blob hash");
    assert_eq!(blacklist.created_by, ADMIN_USER_ID);

    run_endpoint!(
        runner,
        blob_blacklist_remove,
        json!({ "s3_hash": TEST_BLOB_HASH }),
    );
    let removed = BlobBlacklistTable::find_by_id(hash)
        .one(runner.context().transaction())
        .await
        .expect("blob blacklist lookup should succeed");
    assert!(removed.is_none());
}
