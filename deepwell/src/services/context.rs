/*
 * services/context.rs
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

use crate::api::ServerState;
use crate::config::Config;
use crate::error::prelude::*;
use crate::locales::Localizations;
use crate::models::session::Model as SessionModel;
use crate::services::blob::MimeAnalyzer;
use crate::services::permission::{PermissionCache, PermissionService};
use crate::types::{Permission, Reference, Resource};
use redis::aio::MultiplexedConnection as RedisMultiplexedConnection;
use rsmq_async::Rsmq;
use s3::bucket::Bucket;
use sea_orm::DatabaseTransaction;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::OnceCell;

/// Per-request context derived from HTTP headers by the middleware layer.
#[derive(Debug, Clone, Default)]
pub struct RequestContext {
    pub session: Option<SessionModel>,
    pub user_id: Option<i64>,
    pub site_id: Option<i64>,
    pub page_reference: Option<Reference<'static>>,
}

impl RequestContext {
    #[inline]
    pub fn user_session(&self) -> Result<&SessionModel> {
        self.session.as_ref().ok_or_raise(|| {
            Error::new(
                "User session not present in request context",
                ErrorType::Request,
            )
        })
    }

    #[inline]
    pub fn user_id(&self) -> Result<i64> {
        self.user_id.ok_or_raise(|| {
            Error::new("user ID not present in request context", ErrorType::Request)
        })
    }

    #[inline]
    pub fn site_id(&self) -> Result<i64> {
        self.site_id.ok_or_raise(|| {
            Error::new("site ID not present in request context", ErrorType::Request)
        })
    }

    #[inline]
    pub fn page_reference(&self) -> Result<&Reference<'_>> {
        self.page_reference.as_ref().ok_or_raise(|| {
            Error::new(
                "Page reference not present in request context",
                ErrorType::Request,
            )
        })
    }
}

#[derive(Debug)]
pub struct ServiceContext<'txn> {
    state: ServerState,
    transaction: &'txn DatabaseTransaction,
    request_ctx: RequestContext,
    permission_cache: PermissionCache,
}

impl<'txn> ServiceContext<'txn> {
    // NOTE: It is the responsibility of the caller to manage commit / rollback
    //       for transactions.
    //
    //       For our endpoints, this is managed in the wrapper macro in api.rs
    pub fn new(state: &ServerState, transaction: &'txn DatabaseTransaction) -> Self {
        ServiceContext {
            state: Arc::clone(state),
            transaction,
            request_ctx: RequestContext::default(),
            permission_cache: PermissionCache::new(),
        }
    }

    #[inline]
    pub fn with_request(self, request_ctx: RequestContext) -> Self {
        Self {
            request_ctx,
            ..self
        }
    }

    #[inline]
    /// Internal method to update the request context, for use in testing only.
    pub fn set_request_for_test(&mut self, request_ctx: RequestContext) {
        self.request_ctx = request_ctx;

        // Clear cached permissions since the user context has changed.
        self.permission_cache.clear();
    }

    // Getters
    #[inline]
    pub fn state(&self) -> ServerState {
        Arc::clone(&self.state)
    }

    #[inline]
    pub fn config(&self) -> &Config {
        &self.state.config
    }

    #[inline]
    pub fn redis(&self) -> RedisMultiplexedConnection {
        RedisMultiplexedConnection::clone(&self.state.redis)
    }

    #[inline]
    pub fn rsmq(&self) -> Rsmq {
        Rsmq::clone(&self.state.rsmq)
    }

    #[inline]
    pub fn localization(&self) -> &Localizations {
        &self.state.localizations
    }

    #[inline]
    pub fn mime(&self) -> &MimeAnalyzer {
        &self.state.mime_analyzer
    }

    #[inline]
    pub fn s3_files_bucket(&self) -> &Bucket {
        &self.state.s3_files_bucket
    }

    #[inline]
    pub fn s3_tblocks_bucket(&self) -> &Bucket {
        &self.state.s3_tblocks_bucket
    }

    #[inline]
    pub fn transaction(&self) -> &'txn DatabaseTransaction {
        self.transaction
    }

    #[inline]
    pub fn request(&self) -> &RequestContext {
        &self.request_ctx
    }

    #[inline]
    pub fn permission_cache(&self) -> &PermissionCache {
        &self.permission_cache
    }
}
