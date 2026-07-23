/*
 * tests/common/runner.rs
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

//! Helper functions and macros for running individual test cases.

use deepwell::api::{ServerState, build_server_state_without_workers};
use deepwell::config::{Config, Secrets};
use deepwell::services::{RequestContext, ServiceContext};
use sea_orm::{DatabaseTransaction, TransactionTrait};
use self_cell::self_cell;

#[derive(Debug)]
pub struct TestRunnerRequestContext {
    state: ServerState,
    transaction: Option<DatabaseTransaction>,
    request_ctx: RequestContext,
}

impl TestRunnerRequestContext {
    pub async fn new() -> Self {
        let secrets = Secrets::load();
        let config = Config::integration_testing();

        let state = build_server_state_without_workers(config, secrets)
            .await
            .expect("Unable to set up server state");

        let txn = state
            .database
            .begin()
            .await
            .expect("Unable to start database transaction");

        TestRunnerRequestContext {
            state,
            transaction: Some(txn),
            request_ctx: RequestContext::default(),
        }
    }

    fn transaction(&self) -> &DatabaseTransaction {
        // Only should be unset in Drop
        self.transaction.as_ref().expect("Should never be None")
    }

    #[inline]
    fn build_service_context<'txn>(&'txn self) -> ServiceContext<'txn> {
        ServiceContext::new(&self.state, self.transaction())
            .with_request(self.request_ctx.clone())
    }
}

impl Drop for TestRunnerRequestContext {
    fn drop(&mut self) {
        // DatabaseTransaction rolls back on drop. Tests that need synchronous
        // rollback before inspecting external state must call `teardown`.
        self.transaction.take();
    }
}

self_cell!(
    pub struct TestRunner {
        owner: TestRunnerRequestContext,

        #[covariant]
        dependent: ServiceContext,
    }

    impl {Debug}
);

impl TestRunner {
    pub async fn setup() -> Self {
        let request_ctx = TestRunnerRequestContext::new().await;
        Self::new(request_ctx, TestRunnerRequestContext::build_service_context)
    }

    #[inline]
    #[allow(unused)]
    pub fn state(&self) -> &ServerState {
        &self.borrow_owner().state
    }

    #[inline]
    #[allow(unused)]
    pub fn config(&self) -> &Config {
        &self.state().config
    }

    #[inline]
    #[allow(unused)]
    pub fn context<'a>(&'a self) -> &'a ServiceContext<'a> {
        self.borrow_dependent()
    }

    #[allow(unused)]
    pub fn set_request_context(&mut self, req_ctx: RequestContext) {
        self.with_dependent_mut(|_owner, ctx| ctx.set_request_for_test(req_ctx));
    }

    #[allow(unused)]
    pub async fn teardown(self) {
        let mut owner = self.into_owner();
        if let Some(transaction) = owner.transaction.take() {
            transaction
                .rollback()
                .await
                .expect("Unable to roll back transaction");
        }
    }
}
