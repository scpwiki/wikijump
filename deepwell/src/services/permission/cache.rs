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

use std::{borrow::Cow, collections::HashMap};

use super::prelude::*;
use crate::{
    error::{Error, ErrorType},
    models::{prelude::RolePermission, role_permission},
    services::ServiceContext,
    types::{Action, Resource},
};
use ftml::info;
use redis::AsyncCommands;

pub const DEFAULT_CATEGORY_KEY: &str = "_default";

#[derive(Debug, Clone, Copy)]
pub struct PermissionCache;

#[allow(dead_code)]
impl PermissionCache {
    /// Build Redis cache key for a permission hash.
    fn key(site_id: i64, resource_type: Resource, action: Action) -> String {
        format!("perm:{}:{}:{}", site_id, resource_type, action)
    }

    /// Build the field key within a permission hash for a specific category.
    fn category_field(resource_category_id: Option<i64>) -> Cow<'static, str> {
        resource_category_id
            .map(|id| Cow::Owned(id.to_string()))
            .unwrap_or_else(|| Cow::Borrowed(DEFAULT_CATEGORY_KEY))
    }

    /// Check if an action should be cached.
    pub fn is_cacheable(resource_type: Resource, action: Action) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match (resource_type, action) {
            (_, Action::View) => true,
            _ => false,
        }
    }

    /// Fetch cached role IDs for a permission category.
    pub async fn query_roles_with_permission_cache(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        resource_type: Resource,
        resource_category_id: Option<i64>,
        action: Action,
    ) -> Result<Option<Vec<i64>>> {
        let mut redis = ctx.redis();

        let key = Self::key(site_id, resource_type, action);
        let field = Self::category_field(resource_category_id);
        let default_field = Self::category_field(None);

        // Fetch both category-specific and default values in one round trip
        let values: Vec<Option<String>> = redis
            .hmget(&key, &[&field, &default_field])
            .await
            .or_raise(|| {
                warn!(
                    "Failed to read permission cache key '{}' fields '{}' and '{}'",
                    key, field, default_field
                );
                Error::new("Permission cache read error", ErrorType::Permission)
            })?;

        let category_value = values[0].clone();
        let default_value = values[1].clone();

        // Prefer category-specific value, fall back to default if not found
        let role_ids_str = if category_value.is_some() {
            category_value
        } else {
            default_value
        };

        match role_ids_str {
            None => Ok(None), // Field doesn't exist = cache miss
            Some(s) => {
                let parsed = s
                    .split(',')
                    .filter_map(|part| part.trim().parse::<i64>().ok())
                    .collect();
                Ok(Some(parsed))
            }
        }
    }

    pub async fn build_permission_cache(
        ctx: &ServiceContext<'_>,
        site_id: i64,
    ) -> Result<()> {
        let txn = ctx.transaction();
        let make_error =
            || Error::new("Error building permission cache", ErrorType::Permission);

        let all_permissions = RolePermission::find()
            .filter(role_permission::Column::SiteId.eq(site_id))
            .all(txn)
            .await
            .or_raise(make_error)?;

        let mut cache_map: HashMap<(Resource, Option<i64>, Action), Vec<i64>> =
            HashMap::new();
        for perm in all_permissions {
            if !Self::is_cacheable(perm.resource_type, perm.action) {
                continue;
            }

            let key = (perm.resource_type, perm.resource_category_id, perm.action);
            cache_map.entry(key).or_default().push(perm.role_id);
        }

        let mut redis = ctx.redis();
        for ((resource_type, resource_category_id, action), role_ids) in cache_map {
            let key = Self::key(site_id, resource_type, action);
            let field = Self::category_field(resource_category_id);
            let value = role_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let _: () = redis.hset(&key, &field, value).await.or_raise(make_error)?;
        }

        Ok(())
    }

    pub async fn invalidate_site(ctx: &ServiceContext<'_>, site_id: i64) -> Result<()> {
        let mut redis = ctx.redis();
        let pattern = format!("perm:{}:*", site_id);
        let make_error = || {
            Error::new(
                format!("Failed to invalidate permission cache for site {}", site_id),
                ErrorType::Permission,
            )
        };

        let keys: Vec<String> = redis.keys(&pattern).await.or_raise(make_error)?;

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
