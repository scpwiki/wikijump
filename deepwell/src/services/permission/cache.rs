/*
 * services/permission/cache.rs
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

use std::borrow::Cow;
use std::collections::HashMap;

use super::prelude::*;
use crate::error::{Error, ErrorType};
use crate::models::prelude::RolePermission;
use crate::models::role_permission;
use crate::runtime::ServerState;
use crate::services::ServiceContext;
use crate::types::{Action, Resource};
use ftml::info;
use redis::{AsyncCommands, AsyncIter, Script};

pub const DEFAULT_CATEGORY_KEY: &str = "_default";
pub const SITE_NOT_SET_KEY: &str = "platform";
pub const USER_NOT_SET_KEY: &str = "anonymous";
pub const PERMISSION_CACHE_TTL_SECONDS: i64 = 300;
pub const PERMISSION_CACHE_FENCE_TTL_SECONDS: i64 = PERMISSION_CACHE_TTL_SECONDS * 2;
pub const PERMISSION_CACHE_INVALIDATION_CHANNEL: &str =
    "wikijump:article-response-fence-invalidation:v1";

#[derive(Debug, Clone, Copy)]
pub struct PermissionCache;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionCacheFence {
    site_version: String,
    user_version: String,
}

impl PermissionCacheFence {
    pub fn cache_key_fragment(&self) -> String {
        format!("site={},user={}", self.site_version, self.user_version)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SetUserPermissionInput {
    pub site_id: Option<i64>,
    pub user_id: Option<i64>,
    pub resource_type: Resource,
    pub resource_category_id: Option<i64>,
    pub action: Action,
    pub has_permission: bool,
}

#[allow(dead_code)]
impl PermissionCache {
    /// Build Redis cache key prefix to lookup user permissions for a specific site.
    fn site_user_key(site_id: Option<i64>, user_id: Option<i64>) -> String {
        format!(
            "permission:site:{}:user:{}",
            site_id
                .map(|id| id.to_string())
                .unwrap_or(SITE_NOT_SET_KEY.to_owned()),
            user_id
                .map(|id| id.to_string())
                .unwrap_or(USER_NOT_SET_KEY.to_owned())
        )
    }

    /// Build Redis cache key to lookup one user permission decision.
    fn site_user_permission_key(
        site_id: Option<i64>,
        user_id: Option<i64>,
        resource: Resource,
        resource_category_id: Option<i64>,
        action: Action,
        fence: &PermissionCacheFence,
    ) -> String {
        format!(
            "{}:v:{}:{}:{}",
            Self::site_user_key(site_id, user_id),
            fence.site_version,
            fence.user_version,
            Self::permission_key(resource, resource_category_id, action),
        )
    }

    fn site_version_key(site_id: i64) -> String {
        format!("permission:site:{}:version", site_id)
    }

    fn site_user_version_key(site_id: i64, user_id: Option<i64>) -> String {
        format!("{}:version", Self::site_user_key(Some(site_id), user_id))
    }

    /// Build a hash field key for the permission
    fn permission_key(
        resource: Resource,
        resource_category_id: Option<i64>,
        action: Action,
    ) -> Cow<'static, str> {
        let category_id_str = resource_category_id
            .map(|id| id.to_string())
            .unwrap_or(DEFAULT_CATEGORY_KEY.to_owned());
        Cow::Owned(format!(
            "permission:{}:{}:{}",
            resource, category_id_str, action
        ))
    }

    /// Check if an action should be cached.
    pub fn is_cacheable(resource_type: Resource, action: Action) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match (resource_type, action) {
            (_, Action::View) => true,
            _ => false,
        }
    }

    /// Check if this user's permission has been cached, and return it.
    pub async fn check_user_permission(
        ctx: &ServiceContext<'_>,
        site_id: Option<i64>,
        user_id: Option<i64>,
        resource_type: Resource,
        resource_category_id: Option<i64>,
        action: Action,
        fence: &PermissionCacheFence,
    ) -> Result<Option<bool>> {
        let key = Self::site_user_permission_key(
            site_id,
            user_id,
            resource_type,
            resource_category_id,
            action,
            fence,
        );

        let mut redis = ctx.redis();
        let has_permission: Option<String> = redis.get(&key).await.or_raise(|| {
            warn!("Failed to read permission cache key '{}'", key);
            Error::new("permission cache read error", ErrorType::Permission)
        })?;

        Ok(has_permission.map(|val| val == "1"))
    }

    pub async fn cache_fence(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        user_id: Option<i64>,
    ) -> Result<PermissionCacheFence> {
        let site_key = Self::site_version_key(site_id);
        let user_key = Self::site_user_version_key(site_id, user_id);
        let mut redis = ctx.redis();
        let site_version: Option<String> = redis.get(&site_key).await.or_raise(|| {
            warn!("Failed to read permission cache fence key '{}'", site_key);
            Error::new("permission cache fence read error", ErrorType::Permission)
        })?;
        let user_version: Option<String> = redis.get(&user_key).await.or_raise(|| {
            warn!("Failed to read permission cache fence key '{}'", user_key,);
            Error::new("permission cache fence read error", ErrorType::Permission)
        })?;

        Ok(PermissionCacheFence {
            site_version: site_version.unwrap_or_else(|| "0".to_owned()),
            user_version: user_version.unwrap_or_else(|| "0".to_owned()),
        })
    }

    /// Set a user's permission value in the cache.
    pub async fn set_user_permission(
        ctx: &ServiceContext<'_>,
        site_id: Option<i64>,
        user_id: Option<i64>,
        resource_type: Resource,
        resource_category_id: Option<i64>,
        action: Action,
        has_permission: bool,
    ) -> Result<()> {
        let input = SetUserPermissionInput {
            site_id,
            user_id,
            resource_type,
            resource_category_id,
            action,
            has_permission,
        };
        let site_id_value = site_id.ok_or_raise(|| {
            Error::new(
                "permission cache write requires a site ID",
                ErrorType::Permission,
            )
        })?;
        let fence = Self::cache_fence(ctx, site_id_value, user_id).await?;
        Self::set_user_permission_if_fence_current(ctx, input, &fence).await?;

        Ok(())
    }

    pub async fn set_user_permission_if_fence_current(
        ctx: &ServiceContext<'_>,
        input: SetUserPermissionInput,
        fence: &PermissionCacheFence,
    ) -> Result<bool> {
        let SetUserPermissionInput {
            site_id,
            user_id,
            resource_type,
            resource_category_id,
            action,
            has_permission,
        } = input;
        let site_id_value = site_id.ok_or_raise(|| {
            Error::new(
                "permission cache write requires a site ID",
                ErrorType::Permission,
            )
        })?;
        let key = Self::site_user_permission_key(
            site_id,
            user_id,
            resource_type,
            resource_category_id,
            action,
            fence,
        );
        let site_version_key = Self::site_version_key(site_id_value);
        let user_version_key = Self::site_user_version_key(site_id_value, user_id);

        let mut redis = ctx.redis();
        let set: i64 = Script::new(
            r#"
            local site_version = redis.call('GET', KEYS[1]) or '0'
            local user_version = redis.call('GET', KEYS[2]) or '0'
            if site_version == ARGV[1] and user_version == ARGV[2] then
                redis.call('SETEX', KEYS[3], ARGV[3], ARGV[4])
                return 1
            end
            return 0
            "#,
        )
        .key(&site_version_key)
        .key(&user_version_key)
        .key(&key)
        .arg(&fence.site_version)
        .arg(&fence.user_version)
        .arg(PERMISSION_CACHE_TTL_SECONDS)
        .arg(if has_permission { "1" } else { "0" })
        .invoke_async(&mut redis)
        .await
        .or_raise(|| {
            warn!("Failed to write permission cache key '{}'", key);
            Error::new("permission cache write error", ErrorType::Permission)
        })?;

        Ok(set == 1)
    }

    /// Queue cache invalidation for a specific user on a specific site.
    ///
    /// The queued invalidation is run by the API wrapper after the enclosing
    /// database transaction commits.
    pub fn defer_invalidate_user(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        user_id: i64,
    ) -> Result<()> {
        ctx.defer_permission_cache_invalidate_user(site_id, user_id)
    }

    /// Queue cache invalidation for a specific site.
    ///
    /// The queued invalidation is run by the API wrapper after the enclosing
    /// database transaction commits.
    pub fn defer_invalidate_site(ctx: &ServiceContext<'_>, site_id: i64) -> Result<()> {
        ctx.defer_permission_cache_invalidate_site(site_id)
    }

    /// Invalidate the cache for a specific user on a specific site.
    pub async fn invalidate_user(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        user_id: i64,
    ) -> Result<()> {
        Self::invalidate_user_for_state(&ctx.state(), site_id, user_id).await
    }

    pub(crate) async fn invalidate_user_for_state(
        state: &ServerState,
        site_id: i64,
        user_id: i64,
    ) -> Result<()> {
        let mut redis = state.redis.clone();
        let pattern = format!("{}:*", Self::site_user_key(Some(site_id), Some(user_id)));
        let version_key = Self::site_user_version_key(site_id, Some(user_id));
        let make_error = || {
            Error::new(
                format!(
                    "Failed to invalidate permission cache for user {} on site {}",
                    user_id, site_id,
                ),
                ErrorType::Permission,
            )
        };

        let _: i64 = Script::new(
            r#"
            local version = redis.call('INCR', KEYS[1])
            redis.call('EXPIRE', KEYS[1], ARGV[1])
            local payload = '{"type":"user-permission","site_id":' .. ARGV[3] .. ',"user_id":' .. ARGV[4] .. ',"version":"' .. version .. '"}'
            redis.call('PUBLISH', ARGV[2], payload)
            return version
            "#,
        )
        .key(&version_key)
        .arg(PERMISSION_CACHE_FENCE_TTL_SECONDS)
        .arg(PERMISSION_CACHE_INVALIDATION_CHANNEL)
        .arg(site_id)
        .arg(user_id)
        .invoke_async(&mut redis)
        .await
        .or_raise(make_error)?;

        let mut iter: AsyncIter<String> =
            redis.scan_match(&pattern).await.or_raise(make_error)?;
        let mut keys = Vec::new();
        while let Some(key) = iter.next_item().await {
            keys.push(key.or_raise(make_error)?);
        }
        drop(iter);
        keys.retain(|key| !key.ends_with(":version"));

        if keys.is_empty() {
            debug!(
                "No permission cache entries to invalidate for user {} on site {}",
                user_id, site_id
            );
            return Ok(());
        }

        let _: usize = redis.del(keys).await.or_raise(make_error)?;
        Ok(())
    }

    /// Invalidate the cache for a specific site.
    pub async fn invalidate_site(ctx: &ServiceContext<'_>, site_id: i64) -> Result<()> {
        Self::invalidate_site_for_state(&ctx.state(), site_id).await
    }

    pub(crate) async fn invalidate_site_for_state(
        state: &ServerState,
        site_id: i64,
    ) -> Result<()> {
        let mut redis = state.redis.clone();
        let pattern = format!("permission:site:{}:*", site_id);
        let version_key = Self::site_version_key(site_id);
        let anonymous_user_version_key = Self::site_user_version_key(site_id, None);
        let make_error = || {
            Error::new(
                format!("Failed to invalidate permission cache for site {}", site_id),
                ErrorType::Permission,
            )
        };

        let _: i64 = Script::new(
            r#"
            local site_version = redis.call('INCR', KEYS[1])
            redis.call('EXPIRE', KEYS[1], ARGV[1])
            local user_version = redis.call('GET', KEYS[2]) or '0'
            local payload = '{"type":"anonymous-permission","site_id":' .. ARGV[3] .. ',"site_version":"' .. site_version .. '","user_version":"' .. user_version .. '"}'
            redis.call('PUBLISH', ARGV[2], payload)
            return site_version
            "#,
        )
        .key(&version_key)
        .key(&anonymous_user_version_key)
        .arg(PERMISSION_CACHE_FENCE_TTL_SECONDS)
        .arg(PERMISSION_CACHE_INVALIDATION_CHANNEL)
        .arg(site_id)
        .invoke_async(&mut redis)
        .await
        .or_raise(make_error)?;

        let mut iter: AsyncIter<String> =
            redis.scan_match(&pattern).await.or_raise(make_error)?;
        let mut keys = Vec::new();
        while let Some(key) = iter.next_item().await {
            keys.push(key.or_raise(make_error)?);
        }
        drop(iter);
        keys.retain(|key| !key.ends_with(":version"));

        if keys.is_empty() {
            debug!(
                "No permission cache entries to invalidate for site {}",
                site_id
            );
            return Ok(());
        }

        let _: usize = redis.del(keys).await.or_raise(make_error)?;

        Ok(())
    }
}
