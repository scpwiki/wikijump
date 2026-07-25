/*
 * services/permission/service.rs
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

use super::structs::DecoratedPermission;
use crate::error::prelude::{ExnError, Result, ResultExt};
use crate::error::{Error, ErrorType};
use crate::models::prelude::{Role, RolePermission};
use crate::models::{role, role_permission};
use crate::services::ServiceContext;
use crate::services::audit::{AuditEvent, AuditService};
use crate::services::permission::resolvers::resolve_category_slug;
use crate::services::permission::{
    CheckPermissionContext, PermissionCache, SetUserPermissionInput,
    resolve_category_reference,
};
use crate::services::relation::{GetSiteBan, RelationService};
use crate::services::role::{
    GetRolePermissionsInput, GetUserRolesInput, RoleService, UpdateRolePermissionsInput,
};
use crate::types::{Action, Permission, Reference, Resource};
use futures::future::try_join_all;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, Set};
use std::collections::HashSet;

#[derive(Debug)]
pub struct PermissionService;

#[allow(dead_code)] // TEMP
impl PermissionService {
    /// Updates the permissions for a role, replacing the existing set with the provided set.
    pub async fn update_permissions_for_role(
        ctx: &ServiceContext<'_>,
        UpdateRolePermissionsInput {
            site_id,
            role_reference: reference,
            new_permissions,
            cascade_removals,
            updating_user_id,
            ip_address,
        }: UpdateRolePermissionsInput<'_>,
    ) -> Result<()> {
        let txn = ctx.transaction();

        let role = RoleService::get(ctx, site_id, reference)
            .await
            .or_raise(|| Error::new("failed to get role", ErrorType::RoleNotFound))?;

        let make_error = || {
            Error::new(
                format!("failed to update permissions for role ID {}", role.role_id),
                ErrorType::Role,
            )
        };

        // Resolve all category references concurrently before any DB writes.
        let resolved_permissions: HashSet<Permission<'static>> =
            try_join_all(new_permissions.into_iter().map(|input| async move {
                let resource_category_id = match input.resource_category {
                    Some(cat_ref) => {
                        resolve_category_reference(
                            ctx,
                            site_id,
                            input.resource_type,
                            &cat_ref,
                        )
                        .await?
                    }
                    None => None,
                };
                Ok::<_, ExnError>(Permission {
                    resource_type: input.resource_type,
                    resource_category: resource_category_id.map(Reference::Id),
                    action: input.action,
                })
            }))
            .await
            .or_raise(make_error)?
            .into_iter()
            .collect();

        // Validate that the new permission set is a subset of the parent's permissions (if a parent exists).
        if let Some(parent_id) = role.parent_role_id {
            let parent_perms = Self::permissions_as_set(ctx, parent_id)
                .await
                .or_raise(make_error)?;
            if !resolved_permissions.is_subset(&parent_perms) {
                bail!(Error::new(
                    format!(
                        "role ID {} has permissions not present in parent role ID {}",
                        role.role_id, parent_id,
                    ),
                    ErrorType::RoleHierarchyViolation {
                        role_id: role.role_id,
                        parent_role_id: parent_id,
                    },
                ));
            }
        }

        // Validate that the new permission set is a superset of each child's permissions.
        let children = Role::find()
            .filter(
                Condition::all()
                    .add(role::Column::ParentRoleId.eq(role.role_id))
                    .add(role::Column::SiteId.eq(site_id))
                    .add(role::Column::DeletedAt.is_null()),
            )
            .all(txn)
            .await
            .or_raise(make_error)?;

        for child in &children {
            let child_perms = Self::permissions_as_set(ctx, child.role_id)
                .await
                .or_raise(make_error)?;

            if !child_perms.is_subset(&resolved_permissions) {
                if !cascade_removals {
                    bail!(Error::new(
                        format!(
                            "role ID {} has permissions not present in parent role ID {}",
                            child.role_id, role.role_id,
                        ),
                        ErrorType::RoleHierarchyViolation {
                            role_id: child.role_id,
                            parent_role_id: role.role_id,
                        },
                    ));
                } else {
                    info!(
                        "Cascading permission removals to child role ID {} to maintain hierarchy consistency",
                        child.role_id,
                    );
                    Self::cascade_permission_removals(
                        ctx,
                        site_id,
                        child.role_id,
                        &child_perms
                            .difference(&resolved_permissions)
                            .cloned()
                            .collect(),
                    )
                    .await
                    .or_raise(make_error)?;
                }
            }
        }

        // If validation passes, replace the permission set.
        let deleted_permissions = RolePermission::delete_many()
            .filter(role_permission::Column::RoleId.eq(role.role_id))
            .exec_with_returning(txn)
            .await
            .or_raise(make_error)?;

        if !resolved_permissions.is_empty() {
            let models: Vec<role_permission::ActiveModel> = resolved_permissions
                .iter()
                .map(|perm| {
                    let resource_category_id =
                        perm.resource_category.as_ref().and_then(|r| match r {
                            Reference::Id(id) => Some(*id),
                            _ => None,
                        });
                    role_permission::ActiveModel {
                        role_id: Set(role.role_id),
                        site_id: Set(site_id),
                        resource_type: Set(perm.resource_type),
                        resource_category_id: Set(resource_category_id),
                        action: Set(perm.action),
                        ..Default::default()
                    }
                })
                .collect();

            RolePermission::insert_many(models)
                .exec(txn)
                .await
                .or_raise(make_error)?;
        }

        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::UpdatePermissions {
                role_id: role.role_id,
                updating_user_id,
                old_permissions: deleted_permissions
                    .into_iter()
                    .map(|p| Permission {
                        resource_type: p.resource_type,
                        resource_category: p.resource_category_id.map(Reference::Id),
                        action: p.action,
                    })
                    .collect(),
                new_permissions: resolved_permissions.into_iter().collect(),
            },
        )
        .await
        .or_raise(make_error)?;
        PermissionCache::defer_invalidate_site(ctx, site_id).or_raise(make_error)?;

        Ok(())
    }

    /// Fetches permissions for a role
    ///
    /// Optionally returns human-readable category names.
    pub async fn get_permissions_for_role(
        ctx: &ServiceContext<'_>,
        GetRolePermissionsInput {
            site_id,
            role_reference,
            human_readable_categories,
        }: GetRolePermissionsInput<'_>,
    ) -> Result<Vec<Permission<'static>>> {
        let role_id = RoleService::get(ctx, site_id, role_reference)
            .await
            .or_raise(|| {
                Error::new("failed to get role for permissions", ErrorType::Role)
            })?
            .role_id;
        let make_error = || {
            Error::new(
                format!("failed to get permissions for role ID {}", role_id),
                ErrorType::Permission,
            )
        };
        let mut permissions = Self::fetch_permissions(ctx, role_id)
            .await
            .or_raise(make_error)?;

        if human_readable_categories {
            for perm in &mut permissions {
                if let Some(category_ref) = &perm.resource_category {
                    perm.resource_category = resolve_category_slug(
                        ctx,
                        site_id,
                        perm.resource_type,
                        category_ref,
                    )
                    .await
                    .or_raise(make_error)?
                    .map(Reference::Slug);
                }
            }
        }
        Ok(permissions)
    }

    pub async fn get_decorated_permissions_for_role(
        ctx: &ServiceContext<'_>,
        GetRolePermissionsInput {
            site_id,
            role_reference,
            human_readable_categories,
        }: GetRolePermissionsInput<'_>,
    ) -> Result<Vec<DecoratedPermission<'static>>> {
        let txn = ctx.transaction();

        let role = RoleService::get(ctx, site_id, role_reference)
            .await
            .or_raise(|| {
                Error::new(
                    "Failed to get role for decorated permissions",
                    ErrorType::Role,
                )
            })?;

        let make_error = || {
            Error::new(
                format!(
                    "failed to get decorated permissions for role ID {}",
                    role.role_id
                ),
                ErrorType::Permission,
            )
        };

        // Get permissions for the current role
        let role_permissions = Self::permissions_as_set(ctx, role.role_id)
            .await
            .or_raise(make_error)?;

        // Get permissions for the parent role (if any)
        let parent_permissions = match role.parent_role_id {
            Some(parent_id) => Some(
                Self::permissions_as_set(ctx, parent_id)
                    .await
                    .or_raise(make_error)?,
            ),
            None => None,
        };

        // Get combined permissions for child roles
        let children_roles = Role::find()
            .filter(
                Condition::all()
                    .add(role::Column::ParentRoleId.eq(role.role_id))
                    .add(role::Column::SiteId.eq(site_id))
                    .add(role::Column::DeletedAt.is_null()),
            )
            .all(txn)
            .await
            .or_raise(make_error)?;

        let mut children_permissions = HashSet::new();
        for child in &children_roles {
            let child_perms = Self::permissions_as_set(ctx, child.role_id)
                .await
                .or_raise(make_error)?;
            children_permissions.extend(child_perms);
        }

        // Considering valid hierarchy, parent permissions should encompass the permissions of all descendants
        let active_permissions = match parent_permissions.as_ref() {
            Some(parent_perms) => parent_perms.iter().cloned(),
            None => role_permissions.iter().cloned(),
        };

        // Construct universe set from base permissions and optionally scoped permissions
        let universe: HashSet<Permission<'static>> = Permission::ALL
            .iter()
            .cloned()
            .chain(active_permissions)
            .collect();

        let mut decorated = Vec::with_capacity(universe.len());

        // Decorate each permission
        for mut perm in universe {
            // The role has this permission
            let active = role_permissions.contains(&perm);

            // The role doesn't have this permission, but parent role does, so it can be added
            let addable = !active
                && parent_permissions
                    .as_ref()
                    .is_none_or(|parent| parent.contains(&perm));

            // The role has this permission, and at least one child role contains it, so it can't be removed
            let removable = active && !children_permissions.contains(&perm);

            // Remap resource category from ID to slug
            if human_readable_categories
                && let Some(category_ref) = &perm.resource_category
            {
                perm.resource_category =
                    resolve_category_slug(ctx, site_id, perm.resource_type, category_ref)
                        .await
                        .or_raise(make_error)?
                        .map(Reference::Slug);
            }

            decorated.push(DecoratedPermission {
                permission: perm,
                active,
                addable,
                removable,
            });
        }

        Ok(decorated)
    }

    pub async fn check_user_can<'a>(
        ctx: &ServiceContext<'_>,
        perm_ctx: &CheckPermissionContext<'_>,
        input: Permission<'a>,
    ) -> Result<bool> {
        let [result] = Self::batch_check_user_can(ctx, perm_ctx, [input]).await?;
        Ok(result)
    }

    /// Batch check if a user has the specified permissions.
    ///
    /// Returns an array of booleans corresponding to each permission input.
    /// Results are returned in the same order as the input array, best used with destructuring.
    pub async fn batch_check_user_can<'a, const N: usize>(
        ctx: &ServiceContext<'_>,
        perm_ctx: &CheckPermissionContext<'_>,
        permissions: [Permission<'a>; N],
    ) -> Result<[bool; N]> {
        let user_id = perm_ctx.user_id;
        let site_id = perm_ctx.site_id;
        let page_reference = perm_ctx.page_reference.clone();
        let page_scoped_roles = page_reference.is_some();

        let make_error =
            || Error::new("failed to check permissions", ErrorType::Permission);

        let user_permissions =
            Self::get_permissions_for_user(ctx, user_id, site_id, page_reference)
                .await
                .or_raise(make_error)?;

        // Short-circuit: no permissions.
        if user_permissions.is_empty() {
            return Ok([false; N]);
        }

        let mut results = [false; N];

        for (i, permission) in permissions.into_iter().enumerate() {
            results[i] = Self::permission_in_set_helper(
                ctx,
                user_id,
                &user_permissions,
                site_id,
                page_scoped_roles,
                permission,
            )
            .await
            .or_raise(make_error)?;
        }

        Ok(results)
    }

    /// Helper function to check if a permission is present in (the user's) permission set.
    pub(crate) async fn permission_in_set_helper(
        ctx: &ServiceContext<'_>,
        user_id: Option<i64>,
        user_permissions: &HashSet<Permission<'static>>,
        site_id: i64,
        page_scoped_roles: bool,
        Permission {
            resource_type: resource,
            resource_category,
            action,
        }: Permission<'_>,
    ) -> Result<bool> {
        let make_error =
            || Error::new("failed to check permission", ErrorType::Permission);

        info!(
            "Checking permission for user ID {:?} on site ID {} for resource {} of category {:?} with action {}",
            user_id, site_id, resource, resource_category, action,
        );

        // Active bans are time-dependent and may lapse without a write, so
        // cached view decisions must be bypassed while a ban is active.
        // `page_reference` can add page-specific virtual roles (for example,
        // PageAuthor). Those decisions cannot safely use the category-wide
        // permission cache key.
        let cacheable = PermissionCache::is_cacheable(resource, action)
            && !page_scoped_roles
            && !Self::active_site_ban_suppresses_cache(ctx, site_id, user_id)
                .await
                .or_raise(make_error)?;

        // Resolve category reference to ID for permission checking
        let resource_category_id = match &resource_category {
            Some(reference) => {
                resolve_category_reference(ctx, site_id, resource, reference).await?
            }
            None => None,
        };

        let cache_fence = if cacheable {
            Some(
                PermissionCache::cache_fence(ctx, site_id, user_id)
                    .await
                    .or_raise(make_error)?,
            )
        } else {
            None
        };

        if let Some(cache_fence) = &cache_fence {
            // Check if this permission has been cached
            let has_permission = PermissionCache::check_user_permission(
                ctx,
                Some(site_id),
                user_id,
                resource,
                resource_category_id,
                action,
                cache_fence,
            )
            .await
            .or_raise(make_error)?;

            // If we have a cached result, use it
            if let Some(has_permission) = has_permission {
                info!(
                    "Cache hit for user ID {:?} on site ID {} for resource {} of category {:?} with action {}",
                    user_id, site_id, resource, resource_category, action,
                );
                return Ok(has_permission);
            } else {
                info!(
                    "Cache miss for user ID {:?} on site ID {} for resource {} of category {:?} with action {}",
                    user_id, site_id, resource, resource_category, action,
                );
            }
        }

        // If permission is not cacheable, or is not cached, compute it fresh

        // Does this category have permissions scoped to it?
        let has_scoped_permissions = match resource_category_id {
            Some(category_id) => {
                Self::check_category_scoped(ctx, site_id, resource, category_id, action)
                    .await
                    .or_raise(make_error)?
            }
            None => false,
        };

        let has_permission = if has_scoped_permissions {
            user_permissions.contains(&Permission {
                resource_type: resource,
                resource_category: resource_category_id.map(Reference::Id),
                action,
            })
        } else {
            // If category does not have scoped permissions, fallback to _default
            user_permissions.contains(&Permission {
                resource_type: resource,
                resource_category: None,
                action,
            })
        };

        // Cache result if cacheable
        if let Some(cache_fence) = cache_fence {
            PermissionCache::set_user_permission_if_fence_current(
                ctx,
                SetUserPermissionInput {
                    site_id: Some(site_id),
                    user_id,
                    resource_type: resource,
                    resource_category_id,
                    action,
                    has_permission,
                },
                &cache_fence,
            )
            .await
            .or_raise(make_error)?;
        }

        Ok(has_permission)
    }

    async fn active_site_ban_suppresses_cache(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        user_id: Option<i64>,
    ) -> Result<bool> {
        if let Some(user_id) = user_id {
            RelationService::active_site_ban_exists(ctx, GetSiteBan { site_id, user_id })
                .await
        } else {
            Ok(false)
        }
    }

    /// Fetches permissions for `role_id`.
    /// This is a separate function that returns Vec to preserve ordering.
    async fn fetch_permissions(
        ctx: &ServiceContext<'_>,
        role_id: i64,
    ) -> Result<Vec<Permission<'static>>> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!("failed to get permissions for role ID {}", role_id),
                ErrorType::Role,
            )
        };
        Ok(RolePermission::find()
            .filter(role_permission::Column::RoleId.eq(role_id))
            .order_by_asc(role_permission::Column::ResourceType)
            .order_by_asc(role_permission::Column::ResourceCategoryId)
            .order_by_asc(role_permission::Column::Action)
            .all(txn)
            .await
            .or_raise(make_error)?
            .into_iter()
            .map(|p| Permission {
                resource_type: p.resource_type,
                resource_category: p.resource_category_id.map(Reference::Id),
                action: p.action,
            })
            .collect())
    }

    /// Fetches permissions for `role_id` as a set for easy comparison in hierarchy validation.
    pub async fn permissions_as_set(
        ctx: &ServiceContext<'_>,
        role_id: i64,
    ) -> Result<HashSet<Permission<'static>>> {
        Ok(Self::fetch_permissions(ctx, role_id)
            .await?
            .into_iter()
            .collect())
    }

    pub async fn get_permissions_for_user(
        ctx: &ServiceContext<'_>,
        user_id: Option<i64>,
        site_id: i64,
        page_reference: Option<Reference<'_>>,
    ) -> Result<HashSet<Permission<'static>>> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to get permissions for user {:?} in site {:?}",
                    user_id, site_id
                ),
                ErrorType::Permission,
            )
        };

        let role_ids: Vec<i64> = RoleService::get_all_roles_for_user_and_site(
            ctx,
            GetUserRolesInput {
                user_id,
                site_id,
                page_reference,
            },
        )
        .await
        .or_raise(make_error)?
        .into_iter()
        .map(|r| r.role_id)
        .collect();

        Ok(RolePermission::find()
            .filter(role_permission::Column::RoleId.is_in(role_ids))
            .all(txn)
            .await
            .or_raise(make_error)?
            .into_iter()
            .map(|p| Permission {
                resource_type: p.resource_type,
                resource_category: p.resource_category_id.map(Reference::Id),
                action: p.action,
            })
            .collect())
    }

    async fn check_category_scoped(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        resource: Resource,
        resource_category_id: i64,
        action: Action,
    ) -> Result<bool> {
        let txn = ctx.transaction();
        Ok(RolePermission::find()
            .filter(
                Condition::all()
                    .add(role_permission::Column::SiteId.eq(site_id))
                    .add(role_permission::Column::ResourceType.eq(resource))
                    .add(
                        role_permission::Column::ResourceCategoryId
                            .eq(resource_category_id),
                    )
                    .add(role_permission::Column::Action.eq(action)),
            )
            .one(txn)
            .await
            .or_raise(|| {
                Error::new("error querying permissions", ErrorType::Permission)
            })?)
        .map(|p| p.is_some())
    }

    async fn cascade_permission_removals(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        child_role_id: i64,
        removed_permissions: &HashSet<Permission<'static>>,
    ) -> Result<()> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!(
                    "failed to cascade permission removals for role ID {}",
                    child_role_id
                ),
                ErrorType::Permission,
            )
        };

        let child_perms = Self::permissions_as_set(ctx, child_role_id)
            .await
            .or_raise(make_error)?;
        let to_remove: HashSet<Permission<'static>> = child_perms
            .intersection(removed_permissions)
            .cloned()
            .collect();

        // Remove permissions from child role
        for perm in &to_remove {
            let resource_category_id =
                perm.resource_category.as_ref().and_then(|r| match r {
                    Reference::Id(id) => Some(*id),
                    _ => None,
                });
            let resource_condition = match resource_category_id {
                Some(id) => role_permission::Column::ResourceCategoryId.eq(id),
                None => role_permission::Column::ResourceCategoryId.is_null(),
            };

            RolePermission::delete_many()
                .filter(
                    Condition::all()
                        .add(role_permission::Column::RoleId.eq(child_role_id))
                        .add(role_permission::Column::ResourceType.eq(perm.resource_type))
                        .add(resource_condition)
                        .add(role_permission::Column::Action.eq(perm.action)),
                )
                .exec(txn)
                .await
                .or_raise(make_error)?;
        }

        // Recursively cascade to grandchildren
        let grandchildren = Role::find()
            .filter(
                Condition::all()
                    .add(role::Column::ParentRoleId.eq(child_role_id))
                    .add(role::Column::SiteId.eq(site_id))
                    .add(role::Column::DeletedAt.is_null()),
            )
            .all(txn)
            .await
            .or_raise(make_error)?;

        for grandchild in grandchildren {
            Box::pin(Self::cascade_permission_removals(
                ctx,
                site_id,
                grandchild.role_id,
                &to_remove,
            ))
            .await
            .or_raise(make_error)?;
        }

        Ok(())
    }
}
