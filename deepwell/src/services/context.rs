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

use crate::config::Config;
use crate::error::prelude::*;
use crate::locales::Localizations;
use crate::models::session::Model as SessionModel;
use crate::runtime::ServerState;
use crate::services::blob::MimeAnalyzer;
use crate::services::permission::{PermissionCache, PermissionService};
use crate::services::public_cache::PublicContentCache;
use crate::types::{Permission, Reference};
use exn::ErrorExt;
use redis::aio::MultiplexedConnection as RedisMultiplexedConnection;
use rsmq_async::Rsmq;
use s3::bucket::Bucket;
use sea_orm::DatabaseTransaction;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Reference;
    use std::borrow::Cow;
    use time::OffsetDateTime;

    fn session_model() -> SessionModel {
        SessionModel {
            session_token: str!("token"),
            user_id: 42,
            created_at: OffsetDateTime::UNIX_EPOCH,
            expires_at: OffsetDateTime::UNIX_EPOCH,
            ip_address: str!("127.0.0.1"),
            user_agent: str!("test"),
            restricted: false,
        }
    }

    #[test]
    fn request_context_accessors_return_present_values() {
        let context = RequestContext {
            session: Some(session_model()),
            user_id: Some(42),
            site_id: Some(7),
            page_reference: Some(Reference::Slug(Cow::Borrowed("page"))),
        };

        assert_eq!(context.user_session().unwrap().session_token, "token");
        assert_eq!(context.user_id().unwrap(), 42);
        assert_eq!(context.site_id().unwrap(), 7);
        assert_eq!(
            context.page_reference().unwrap(),
            &Reference::Slug(Cow::Borrowed("page")),
        );
    }

    #[test]
    fn request_context_accessors_reject_absent_values() {
        let context = RequestContext::default();

        assert!(context.user_session().is_err());
        assert!(context.user_id().is_err());
        assert!(context.site_id().is_err());
        assert!(context.page_reference().is_err());
    }
}

#[derive(Debug)]
pub struct ServiceContext<'txn> {
    state: ServerState,
    transaction: &'txn DatabaseTransaction,
    request_ctx: RequestContext,
    user_permissions: OnceCell<HashSet<Permission<'static>>>,
    post_commit_actions: Mutex<Vec<PostCommitAction>>,
}

#[derive(Debug)]
pub(crate) enum PostCommitAction {
    PermissionUser { site_id: i64, user_id: i64 },
    PermissionSite { site_id: i64 },
    PublicContentSite { site_id: i64 },
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
            user_permissions: OnceCell::new(),
            post_commit_actions: Mutex::new(Vec::new()),
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
        self.user_permissions = OnceCell::new();
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

    pub fn defer_permission_cache_invalidate_user(
        &self,
        site_id: i64,
        user_id: i64,
    ) -> Result<()> {
        let mut actions = self.post_commit_actions.lock().map_err(|_| {
            Error::new(
                "failed to queue permission cache invalidation",
                ErrorType::Permission,
            )
            .raise()
        })?;
        actions.push(PostCommitAction::PermissionUser { site_id, user_id });
        Ok(())
    }

    pub fn defer_permission_cache_invalidate_site(&self, site_id: i64) -> Result<()> {
        let mut actions = self.post_commit_actions.lock().map_err(|_| {
            Error::new(
                "failed to queue permission cache invalidation",
                ErrorType::Permission,
            )
            .raise()
        })?;
        actions.push(PostCommitAction::PermissionSite { site_id });
        Ok(())
    }

    pub fn defer_public_content_cache_invalidate_site(&self, site_id: i64) -> Result<()> {
        let mut actions = self.post_commit_actions.lock().map_err(|_| {
            Error::new(
                "failed to queue public content cache invalidation",
                ErrorType::Page,
            )
            .raise()
        })?;
        actions.push(PostCommitAction::PublicContentSite { site_id });
        Ok(())
    }

    pub(crate) fn drain_post_commit_actions(&self) -> Result<Vec<PostCommitAction>> {
        let mut actions = self.post_commit_actions.lock().map_err(|_| {
            Error::new(
                "failed to drain queued post-commit actions",
                ErrorType::Permission,
            )
            .raise()
        })?;
        Ok(std::mem::take(&mut *actions))
    }

    pub(crate) async fn run_post_commit_actions_for_state(
        state: &ServerState,
        actions: Vec<PostCommitAction>,
    ) -> Result<()> {
        for action in actions {
            match action {
                PostCommitAction::PermissionUser { site_id, user_id } => {
                    PermissionCache::invalidate_user_for_state(state, site_id, user_id)
                        .await?;
                }
                PostCommitAction::PermissionSite { site_id } => {
                    PermissionCache::invalidate_site_for_state(state, site_id).await?;
                }
                PostCommitAction::PublicContentSite { site_id } => {
                    PublicContentCache::invalidate_site_for_state(state, site_id).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn run_post_commit_actions(&self) -> Result<()> {
        let actions = self.drain_post_commit_actions()?;
        Self::run_post_commit_actions_for_state(&self.state, actions).await
    }

    #[inline]
    pub fn request(&self) -> &RequestContext {
        &self.request_ctx
    }

    pub async fn user_permissions(&self) -> Result<&HashSet<Permission<'static>>> {
        // Lazily fetch and cache user permissions for the duration of the request.
        self.user_permissions
            .get_or_try_init(|| async {
                let user_id = self.request_ctx.user_id;
                let site_id = self.request_ctx.site_id()?;
                let page_reference = self.request_ctx.page_reference.clone();

                PermissionService::get_permissions_for_user(
                    self,
                    user_id,
                    site_id,
                    page_reference,
                )
                .await
                .or_raise(|| {
                    Error::new("failed to fetch user permissions", ErrorType::Permission)
                })
            })
            .await
    }

    pub async fn user_has_permission(&self, permission: Permission<'_>) -> Result<bool> {
        let user_id = self.request_ctx.user_id;
        let site_id = self.request_ctx.site_id()?;
        let make_error =
            || Error::new("failed to check user permissions", ErrorType::Permission);

        let perms = self.user_permissions().await.or_raise(make_error)?;
        PermissionService::permission_in_set_helper(
            self,
            user_id,
            perms,
            site_id,
            self.request_ctx.page_reference.is_some(),
            permission,
        )
        .await
        .or_raise(make_error)
    }
}
