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

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::structs::PermissionTarget;
use crate::error::prelude::*;
use crate::models::prelude::RolePermission;
use crate::models::role_permission;
use crate::services::ServiceContext;
use crate::services::role::RoleService;
use crate::types::{Permission, Reference};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PermissionCacheKey {
    pub user_id: Option<i64>,
    pub site_id: Option<i64>,
    pub target: PermissionTarget,
}

/// Request-local cache of computed permission sets.
/// Only relevant for the duration of a single request.
#[derive(Debug, Default)]
pub struct PermissionCache {
    cache: RwLock<HashMap<PermissionCacheKey, HashSet<Permission<'static>>>>,
}

impl PermissionCache {
    pub fn new() -> PermissionCache {
        PermissionCache::default()
    }

    fn read(
        &self,
    ) -> RwLockReadGuard<'_, HashMap<PermissionCacheKey, HashSet<Permission<'static>>>>
    {
        self.cache.read().unwrap_or_else(|err| err.into_inner())
    }

    fn write(
        &self,
    ) -> RwLockWriteGuard<'_, HashMap<PermissionCacheKey, HashSet<Permission<'static>>>>
    {
        self.cache.write().unwrap_or_else(|err| err.into_inner())
    }

    pub fn insert(
        &self,
        key: PermissionCacheKey,
        permissions: HashSet<Permission<'static>>,
    ) {
        self.write().insert(key, permissions);
    }

    pub fn clear(&self) {
        self.write().clear();
    }

    pub async fn get_or_fetch(
        &self,
        ctx: &ServiceContext<'_>,
        key: PermissionCacheKey,
    ) -> Result<HashSet<Permission<'static>>> {
        if let Some(permissions) = self.read().get(&key).cloned() {
            return Ok(permissions);
        }

        let make_error = || {
            Error::new(
                "failed to fetch permissions for user and target",
                ErrorType::Permission,
            )
        };

        // Get all applicable role IDs for the user and target resource
        let role_ids: HashSet<i64> = RoleService::get_applicable_roles_for_target(
            ctx,
            key.user_id,
            key.site_id,
            &key.target,
        )
        .await?
        .into_iter()
        .map(|role| role.role_id)
        .collect();

        if role_ids.is_empty() {
            return Ok(HashSet::new());
        }

        // Load all permissions for the applicable roles
        let permissions = RolePermission::find()
            .filter(role_permission::Column::RoleId.is_in(role_ids))
            .all(ctx.transaction())
            .await
            .or_raise(make_error)?
            .into_iter()
            .map(|p| Permission {
                resource_type: p.resource_type,
                resource_category: p.resource_category_id.map(Reference::Id),
                action: p.action,
            })
            .collect::<HashSet<_>>();

        self.insert(key, permissions.clone());
        Ok(permissions)
    }
}
