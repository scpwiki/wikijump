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
use crate::error::prelude::{Error, ErrorType, OptionExt, Result, ResultExt};
use crate::locales::Localizations;
use crate::models::session::Model as SessionModel;
use crate::runtime::ServerState;
use crate::services::blob::MimeAnalyzer;
use crate::services::job::{Job, JobService};
use crate::services::page_revision::{PageRevisionService, RerenderType};
use crate::services::permission::{PermissionCache, PermissionService};
use crate::services::public_cache::PublicContentCache;
use crate::types::{PageId, Permission, Reference, RerenderDepth};
use exn::ErrorExt;
use redis::aio::MultiplexedConnection as RedisMultiplexedConnection;
use rsmq_async::Rsmq;
use s3::bucket::Bucket;
use sea_orm::{DatabaseTransaction, TransactionTrait};
use std::collections::{HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
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
            mfa_failed_attempts: 0,
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
    commit_authentication_rejection: AtomicBool,
}

#[derive(Debug)]
pub(crate) enum PostCommitAction {
    PermissionUser {
        site_id: i64,
        user_id: i64,
    },
    PermissionSite {
        site_id: i64,
    },
    PublicContentSite {
        site_id: i64,
    },
    RerenderPage {
        id: PageId,
        depth: RerenderDepth,
        r#type: RerenderType,
    },
    RerenderPageImmediate {
        id: PageId,
        depth: RerenderDepth,
        r#type: RerenderType,
    },
    DeleteTextBlockObjects {
        filenames: Vec<String>,
    },
}

impl PostCommitAction {
    pub(crate) fn is_page_rerender(&self) -> bool {
        matches!(
            self,
            Self::RerenderPage { .. } | Self::RerenderPageImmediate { .. },
        )
    }
}

/// Who committed the transaction whose post-commit actions are running.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum PostCommitRerenderTrust {
    /// An ordinary request or job.
    Public,

    /// The trusted corpus render finalizer.
    CorpusFinalizer,
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
            commit_authentication_rejection: AtomicBool::new(false),
        }
    }

    #[inline]
    pub fn with_request(mut self, request_ctx: RequestContext) -> Self {
        self.request_ctx = request_ctx;
        self.user_permissions = OnceCell::new();
        self
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

    /// Requests that the current transaction be committed before returning an
    /// expected authentication rejection to the caller.
    ///
    /// This is intentionally limited to the terminal MFA rejection path, where
    /// committing preserves the restricted session's failed-attempt counter.
    /// Internal failures must leave this unset so the normal rollback path is used.
    pub(crate) fn commit_expected_authentication_rejection(&self) {
        self.commit_authentication_rejection
            .store(true, Ordering::Relaxed);
    }

    pub(crate) fn should_commit_authentication_rejection(&self) -> bool {
        self.commit_authentication_rejection.load(Ordering::Relaxed)
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
        if actions.iter().any(|action| {
            matches!(
                action,
                PostCommitAction::PublicContentSite {
                    site_id: queued_site_id,
                } if *queued_site_id == site_id
            )
        }) {
            return Ok(());
        }
        let insert_at = actions
            .iter()
            .position(|action| matches!(action, PostCommitAction::RerenderPage { .. }))
            .unwrap_or(actions.len());
        actions.insert(insert_at, PostCommitAction::PublicContentSite { site_id });
        Ok(())
    }

    pub(crate) fn defer_rerender_page(
        &self,
        id: PageId,
        depth: RerenderDepth,
    ) -> Result<()> {
        self.defer_rerender(id, depth, RerenderType::Full)
    }

    pub(crate) fn defer_rerender_page_immediate(
        &self,
        id: PageId,
        depth: RerenderDepth,
    ) -> Result<()> {
        let mut actions = self.post_commit_actions.lock().map_err(|_| {
            Error::new("failed to queue immediate page rerender", ErrorType::Page).raise()
        })?;
        if actions.iter().any(|action| {
            matches!(
                action,
                PostCommitAction::RerenderPageImmediate {
                    id: queued_id,
                    depth: queued_depth,
                    r#type: RerenderType::Full,
                } if queued_id.site_id == id.site_id
                    && queued_id.category_id == id.category_id
                    && queued_id.page_id == id.page_id
                    && *queued_depth == depth
            )
        }) {
            return Ok(());
        }
        actions.push(PostCommitAction::RerenderPageImmediate {
            id,
            depth,
            r#type: RerenderType::Full,
        });
        Ok(())
    }

    pub(crate) fn defer_rerender_navigation_page(
        &self,
        id: PageId,
        depth: RerenderDepth,
    ) -> Result<()> {
        self.defer_rerender(id, depth, RerenderType::NavigationOnly)
    }

    fn defer_rerender(
        &self,
        id: PageId,
        depth: RerenderDepth,
        r#type: RerenderType,
    ) -> Result<()> {
        let mut actions = self.post_commit_actions.lock().map_err(|_| {
            Error::new("failed to queue page rerender", ErrorType::Page).raise()
        })?;
        if actions.iter().any(|action| {
            matches!(
                action,
                PostCommitAction::RerenderPage {
                    id: queued_id,
                    depth: queued_depth,
                    r#type: queued_type,
                } if queued_id.site_id == id.site_id
                    && queued_id.category_id == id.category_id
                    && queued_id.page_id == id.page_id
                    && *queued_depth == depth
                    && *queued_type == r#type
            )
        }) {
            return Ok(());
        }
        actions.push(PostCommitAction::RerenderPage { id, depth, r#type });
        Ok(())
    }

    pub(crate) fn defer_text_block_cleanup(&self, filenames: Vec<String>) -> Result<()> {
        if filenames.is_empty() {
            return Ok(());
        }
        let mut actions = self.post_commit_actions.lock().map_err(|_| {
            Error::new(
                "failed to queue replaced text block cleanup",
                ErrorType::TextBlock,
            )
            .raise()
        })?;
        actions.push(PostCommitAction::DeleteTextBlockObjects { filenames });
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
        Self::run_post_commit_actions_with_rerender_trust(
            state,
            actions,
            PostCommitRerenderTrust::Public,
        )
        .await
    }

    /// Runs the post-commit actions of a corpus render-finalizer commit.
    ///
    /// Immediate dependent rerenders keep the finalizer's corpus include
    /// ceiling for dependents imported from Wikidot.
    pub(crate) async fn run_post_commit_actions_for_corpus_finalizer(
        state: &ServerState,
        actions: Vec<PostCommitAction>,
    ) -> Result<()> {
        Self::run_post_commit_actions_with_rerender_trust(
            state,
            actions,
            PostCommitRerenderTrust::CorpusFinalizer,
        )
        .await
    }

    async fn run_post_commit_actions_with_rerender_trust(
        state: &ServerState,
        actions: Vec<PostCommitAction>,
        rerender_trust: PostCommitRerenderTrust,
    ) -> Result<()> {
        let mut rsmq = Rsmq::clone(&state.rsmq);
        let mut actions = VecDeque::from(actions);
        while let Some(action) = actions.pop_front() {
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
                PostCommitAction::RerenderPage { id, depth, r#type } => {
                    JobService::queue_job_inner(
                        &mut rsmq,
                        &Job::RerenderPage { id, depth, r#type },
                        None,
                    )
                    .await?;
                }
                PostCommitAction::RerenderPageImmediate { id, depth, r#type } => {
                    let make_error = || {
                        Error::new(
                            format!(
                                "failed to run immediate rerender for page ID {}",
                                id.page_id
                            ),
                            ErrorType::Page,
                        )
                    };
                    let transaction =
                        state.database.begin().await.or_raise(make_error)?;
                    let ctx = ServiceContext::new(state, &transaction);
                    match rerender_trust {
                        PostCommitRerenderTrust::Public => {
                            PageRevisionService::rerender(&ctx, id, depth, r#type).await
                        }
                        PostCommitRerenderTrust::CorpusFinalizer => {
                            PageRevisionService::rerender_dependent_for_corpus_finalizer(
                                &ctx, id, depth, r#type,
                            )
                            .await
                        }
                    }
                    .or_raise(make_error)?;
                    let followups =
                        ctx.drain_post_commit_actions().or_raise(make_error)?;
                    transaction.commit().await.or_raise(make_error)?;
                    actions.extend(followups);
                }
                PostCommitAction::DeleteTextBlockObjects { filenames } => {
                    for filename in filenames {
                        state
                            .s3_tblocks_bucket
                            .delete_object(&filename)
                            .await
                            .or_raise(|| {
                                Error::new(
                                    format!(
                                        "failed to delete replaced S3 text block '{filename}'"
                                    ),
                                    ErrorType::TextBlock,
                                )
                            })?;
                    }
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
