/*
 * tests/authorization_token.rs
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

#[allow(unused_imports)]
#[macro_use]
mod common;

use self::common::TestRunner;
use deepwell::api::ServerState;
use deepwell::constants::ADMIN_USER_ID;
use deepwell::error::prelude::*;
use deepwell::models::audit_log::{
    self, Column as AuditLogColumn, Entity as AuditLogTable,
};
use deepwell::models::authorization_token::{self, Entity as AuthorizationTokenTable};
use deepwell::services::ServiceContext;
use deepwell::services::authorization_token::{
    AuthorizationTokenService, AuthorizedObject,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Barrier;
use tokio::time::timeout;
use uuid::Uuid;

async fn insert_authorization_token(state: &ServerState, token: &str) -> i32 {
    let txn = state
        .database
        .begin()
        .await
        .expect("authorization token setup transaction should begin");
    let token_id = AuthorizationTokenTable::insert(authorization_token::ActiveModel {
        token_value: Set(token.to_owned()),
        created_by: Set(ADMIN_USER_ID),
        description: Set("authorization token concurrency test".to_owned()),
        ..Default::default()
    })
    .exec(&txn)
    .await
    .expect("authorization token fixture should be inserted")
    .last_insert_id;
    txn.commit()
        .await
        .expect("authorization token setup transaction should commit");
    token_id
}

async fn verify_in_fresh_transaction(
    state: ServerState,
    token: String,
    start: Option<Arc<Barrier>>,
) -> Result<()> {
    let txn = state
        .database
        .begin()
        .await
        .expect("authorization token verification transaction should begin");

    if let Some(start) = start {
        start.wait().await;
    }

    let result = {
        let ctx = ServiceContext::new(&state, &txn);
        AuthorizationTokenService::verify(
            &ctx,
            &token,
            AuthorizedObject::BotUser,
            common::IP_ADDRESS,
        )
        .await
    };

    match result {
        Ok(()) => {
            txn.commit()
                .await
                .expect("successful authorization token verification should commit");
            Ok(())
        }
        Err(error) => {
            txn.rollback()
                .await
                .expect("failed authorization token verification should roll back");
            Err(error)
        }
    }
}

async fn verification_audit_events(
    state: &ServerState,
    token_id: i32,
) -> Vec<audit_log::Model> {
    let txn = state
        .database
        .begin()
        .await
        .expect("authorization token assertion transaction should begin");
    let events = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("authorization_token.verify"))
        .filter(AuditLogColumn::ExtraId1.eq(i64::from(token_id)))
        .all(&txn)
        .await
        .expect("authorization token verification audit lookup should succeed");
    txn.rollback()
        .await
        .expect("authorization token assertion transaction should roll back");
    events
}

async fn cleanup_authorization_token_test(state: &ServerState, token_id: i32) {
    let txn = state
        .database
        .begin()
        .await
        .expect("authorization token cleanup transaction should begin");
    AuthorizationTokenTable::delete_by_id(token_id)
        .exec(&txn)
        .await
        .expect("authorization token cleanup should succeed");
    AuditLogTable::delete_many()
        .filter(AuditLogColumn::EventType.eq("authorization_token.verify"))
        .filter(AuditLogColumn::ExtraId1.eq(i64::from(token_id)))
        .exec(&txn)
        .await
        .expect("authorization token audit cleanup should succeed");
    txn.commit()
        .await
        .expect("authorization token cleanup transaction should commit");
}

#[tokio::test]
async fn authorization_token_consumption_is_atomic_and_secret_free() {
    let runner = TestRunner::setup().await;
    let state = Arc::clone(runner.state());
    let token = format!(
        "B-{}",
        Uuid::new_v4().hyphenated().to_string().to_uppercase()
    );
    let token_id = insert_authorization_token(&state, &token).await;
    let start = Arc::new(Barrier::new(2));

    let (left, right) = timeout(Duration::from_secs(30), async {
        tokio::join!(
            verify_in_fresh_transaction(
                Arc::clone(&state),
                token.clone(),
                Some(Arc::clone(&start)),
            ),
            verify_in_fresh_transaction(
                Arc::clone(&state),
                token.clone(),
                Some(Arc::clone(&start)),
            ),
        )
    })
    .await
    .expect("concurrent authorization token verification should not deadlock");

    let reuse =
        verify_in_fresh_transaction(Arc::clone(&state), token.clone(), None).await;
    let events = verification_audit_events(&state, token_id).await;
    cleanup_authorization_token_test(&state, token_id).await;

    let concurrent_error = match (left, right) {
        (Ok(()), Err(error)) | (Err(error), Ok(())) => error,
        (left, right) => {
            panic!(
                "exactly one concurrent verification should succeed: {left:?}, {right:?}"
            )
        }
    };
    assert_contains_error!(concurrent_error, ErrorType::InvalidAuthorizationToken,);

    let reuse_error =
        reuse.expect_err("a consumed authorization token must fail sequential reuse");
    assert_contains_error!(reuse_error, ErrorType::InvalidAuthorizationToken);

    assert_eq!(
        events.len(),
        1,
        "only the successful consume should be audited"
    );
    let event = &events[0];
    assert_eq!(event.extra_string_1.as_deref(), Some("bot-user"));
    assert_eq!(event.extra_string_2, None);
    assert_ne!(event.extra_string_1.as_deref(), Some(token.as_str()));
    let serialized_event = serde_json::to_string(event)
        .expect("authorization token verification audit should serialize");
    assert!(
        !serialized_event.contains(&token),
        "serialized authorization token audit must not contain the raw token",
    );
}
