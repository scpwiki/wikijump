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
use super::prelude::*;
use crate::endpoints::{parent, site};
use crate::error::{Error, ErrorType};
use crate::models::prelude::{Role, RolePermission};
use crate::models::role_permission::Model as RolePermissionModel;
use crate::models::{role, role_permission, user_role};
use crate::services::audit::{AuditEvent, AuditService};
use crate::services::permission::resolvers::resolve_category_slug;
use crate::services::permission::{
    CheckPermissionContext, PermissionCache, PermissionInput, resolve_category_reference,
};
use crate::services::role::{
    GetRolePermissionsInput, GetUserRolesInput, RoleService, UpdateRolePermissionsInput,
};
use crate::services::{ServiceContext, SessionService};
use crate::types::{Action, Permission, Reference, Resource, UserPermissions};
use futures::future::try_join_all;
use rand::rand_core::le;
use std::borrow::Cow;
use std::collections::HashSet;
use std::hash::Hash;
use std::net::IpAddr;

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
        let resolved_permissions: HashSet<Permission> =
            try_join_all(new_permissions.into_iter().map(|input| async move {
                let resource_category_id = match input.resource_category {
                    Some(cat_ref) => {
                        resolve_category_reference(
                            ctx,
                            site_id,
                            input.resource_type,
                            cat_ref,
                        )
                        .await?
                    }
                    None => None,
                };
                Ok::<_, ExnError>(Permission {
                    resource: input.resource_type,
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
                role::Column::ParentRoleId
                    .eq(role.role_id)
                    .and(role::Column::SiteId.eq(site_id))
                    .and(role::Column::DeletedAt.is_null()),
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
                        child.role_id
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
                        resource_type: Set(perm.resource),
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
                        resource: p.resource_type,
                        resource_category: p.resource_category_id.map(Reference::Id),
                        action: p.action,
                    })
                    .collect(),
                new_permissions: resolved_permissions.into_iter().collect(),
            },
        )
        .await
        .or_raise(make_error)?;

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
    ) -> Result<Vec<Permission>> {
        let role_id = match role_reference {
            Reference::Id(id) => id,
            Reference::Slug(_) => {
                RoleService::get(ctx, site_id, role_reference)
                    .await
                    .or_raise(|| {
                        Error::new("Failed to get role for permissions", ErrorType::Role)
                    })?
                    .role_id
            }
        };
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
                if let Some(Reference::Id(cat_id)) = perm.resource_category {
                    perm.resource_category =
                        resolve_category_slug(ctx, site_id, perm.resource, cat_id.into())
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
    ) -> Result<Vec<DecoratedPermission>> {
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
                role::Column::ParentRoleId
                    .eq(role.role_id)
                    .and(role::Column::SiteId.eq(site_id))
                    .and(role::Column::DeletedAt.is_null()),
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
        let universe: HashSet<Permission> = Permission::ALL
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
                && let Some(Reference::Id(cat_id)) = perm.resource_category
            {
                perm.resource_category =
                    resolve_category_slug(ctx, site_id, perm.resource, cat_id.into())
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

    // Lambda to query roles that have the specified permission, optionally filtered by category
    async fn query_roles_with_permission(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        resource: Resource,
        resource_category_id: Option<i64>,
        action: Action,
    ) -> Result<Vec<i64>> {
        let txn = ctx.transaction();

        let category_condition = match resource_category_id {
            Some(id) => role_permission::Column::ResourceCategoryId.eq(id),
            None => role_permission::Column::ResourceCategoryId.is_null(),
        };

        Ok(RolePermission::find()
            .filter(role_permission::Column::SiteId.eq(site_id))
            .filter(role_permission::Column::ResourceType.eq(resource))
            .filter(category_condition)
            .filter(role_permission::Column::Action.eq(action))
            .all(txn)
            .await
            .or_raise(|| Error::new("Error querying permissions", ErrorType::Permission))?
            .into_iter()
            .map(|p| p.role_id)
            .collect::<Vec<_>>())
    }

    async fn query_roles_with_permission_db(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        resource_type: Resource,
        resource_category_id: Option<i64>,
        action: Action,
    ) -> Result<Vec<i64>> {
        let make_error =
            || Error::new("Error querying permissions", ErrorType::Permission);

        // Check first for presence of scoped permission
        let specific_roles = match resource_category_id {
            Some(category_id) => Self::query_roles_with_permission(
                ctx,
                site_id,
                resource_type,
                Some(category_id),
                action,
            )
            .await
            .or_raise(make_error)?,
            None => vec![],
        };

        let roles_with_permission = if specific_roles.is_empty() {
            // If no scoped permission, check for _default permission
            Self::query_roles_with_permission(ctx, site_id, resource_type, None, action)
                .await
                .or_raise(make_error)?
        } else {
            specific_roles
        };

        info!(
            "Queried database for permission '{}:{}:{}', found {} roles",
            resource_type,
            resource_category_id.unwrap_or(0),
            action,
            roles_with_permission.len()
        );
        Ok(roles_with_permission)
    }

    pub async fn check_user_can(
        ctx: &ServiceContext<'_>,
        perm_ctx: &CheckPermissionContext<'_>,
        input: PermissionInput<'_>,
    ) -> Result<bool> {
        let [result] = Self::batch_check_user_can(ctx, perm_ctx, [input]).await?;
        Ok(result)
    }

    /// Batch check if a user has the specified permissions.
    ///
    /// Returns an array of booleans corresponding to each permission input.
    /// Results are returned in the same order as the input array, best used with destructuring.
    pub async fn batch_check_user_can<const N: usize>(
        ctx: &ServiceContext<'_>,
        perm_ctx: &CheckPermissionContext<'_>,
        permissions: [PermissionInput<'_>; N],
    ) -> Result<[bool; N]> {
        let user_id = perm_ctx.user_id;
        let site_id = perm_ctx.site_id;
        let page_reference = perm_ctx.page_reference.clone();

        let make_error =
            || Error::new("failed to check permissions", ErrorType::Permission);

        // Get all roles for this user (includes virtual roles)
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

        // Short-circuit: no roles means no permissions.
        if role_ids.is_empty() {
            return Ok([false; N]);
        }

        // Lambda to check if user has any of the specified roles
        let user_has_any_role = |granted_roles: &[i64]| -> bool {
            granted_roles
                .iter()
                .any(|role_id| role_ids.contains(role_id))
        };

        let mut results = [false; N];

        for (
            i,
            PermissionInput {
                resource_type,
                resource_category,
                action,
            },
        ) in permissions.into_iter().enumerate()
        {
            info!(
                "Checking permission for user ID {:?} on site ID {} for resource {} of category {:?} with action {}",
                user_id, site_id, resource_type, resource_category, action,
            );

            // Resolve category reference to ID for permission checking
            let resource_category_id = match resource_category {
                Some(reference) => {
                    resolve_category_reference(ctx, site_id, resource_type, reference)
                        .await?
                }
                None => None,
            };

            // Check permission based on category specificity
            let roles_with_permission =
                if PermissionCache::is_cacheable(resource_type, action) {
                    // Try to fetch from cache
                    let cached_roles =
                        PermissionCache::query_roles_with_permission_cache(
                            ctx,
                            site_id,
                            resource_type,
                            resource_category_id,
                            action,
                        )
                        .await
                        .or_raise(make_error)?;

                    if let Some(cached) = cached_roles {
                        info!("Permission cache hit, {} roles found", cached.len());
                        cached
                    } else {
                        info!("Permission cache miss, querying database directly");

                        // If cache miss, rebuild tree async
                        PermissionCache::build_permission_cache(ctx, site_id)
                            .await
                            .or_raise(make_error)?;

                        // fall back to database query
                        Self::query_roles_with_permission_db(
                            ctx,
                            site_id,
                            resource_type,
                            resource_category_id,
                            action,
                        )
                        .await
                        .or_raise(make_error)?
                    }
                } else {
                    info!("Permission not cacheable, querying database directly");
                    // For non-cacheable permissions, query database directly
                    Self::query_roles_with_permission_db(
                        ctx,
                        site_id,
                        resource_type,
                        resource_category_id,
                        action,
                    )
                    .await
                    .or_raise(make_error)?
                };

            results[i] = user_has_any_role(&roles_with_permission);
        }

        Ok(results)
    }

    pub async fn prefetch_permission_context(
        ctx: &ServiceContext<'_>,
        perm_ctx: &PrefetchPermissionsInput<'_>,
    ) -> Result<UserPermissions> {
        let site_id = perm_ctx.site_id;

        // TODO: Add support for platform permissions (cross-site permissions)
        if site_id.is_none() {
            return Ok(UserPermissions::new());
        }

        let page_reference = perm_ctx.page_reference.clone();
        let user_id = match perm_ctx.session_token.as_deref() {
            Some("") | None => None,
            Some(token) => {
                let session = SessionService::get(ctx, token).await?;
                Some(session.user_id)
            }
        };

        let make_error = || {
            Error::new(
                format!(
                    "failed to hydrate permissions for user ID {:?} on site ID {}",
                    user_id,
                    site_id.unwrap()
                ),
                ErrorType::Permission,
            )
        };

        let role_ids: Vec<i64> = RoleService::get_all_roles_for_user_and_site(
            ctx,
            GetUserRolesInput {
                user_id,
                site_id: site_id.unwrap(),
                page_reference,
            },
        )
        .await
        .or_raise(make_error)?
        .into_iter()
        .map(|r| r.role_id)
        .collect();

        let permissions: HashSet<Permission> = RolePermission::find()
            .filter(role_permission::Column::RoleId.is_in(role_ids))
            .all(ctx.transaction())
            .await
            .or_raise(make_error)?
            .into_iter()
            .map(|p| Permission {
                resource: p.resource_type,
                resource_category: p.resource_category_id.map(Reference::Id),
                action: p.action,
            })
            .collect();

        let all_site_permissions: Vec<Permission> = RolePermission::find()
            .filter(role_permission::Column::SiteId.eq(site_id.unwrap()))
            .distinct_on([
                role_permission::Column::ResourceType,
                role_permission::Column::ResourceCategoryId,
                role_permission::Column::Action,
            ])
            .all(ctx.transaction())
            .await
            .or_raise(make_error)?
            .into_iter()
            .map(|p| Permission {
                resource: p.resource_type,
                resource_category: p.resource_category_id.map(Reference::Id),
                action: p.action,
            })
            .collect();

        let permission_map = all_site_permissions
            .into_iter()
            .map(|p| (p.clone(), permissions.contains(&p)))
            .collect();

        Ok(UserPermissions { permission_map })
    }

    pub async fn check_category_scoped(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        resource: Resource,
        resource_category_id: i64,
        action: Action,
    ) -> Result<bool> {
        let txn = ctx.transaction();
        Ok(RolePermission::find()
            .filter(role_permission::Column::SiteId.eq(site_id))
            .filter(role_permission::Column::ResourceType.eq(resource))
            .filter(role_permission::Column::ResourceCategoryId.eq(resource_category_id))
            .filter(role_permission::Column::Action.eq(action))
            .exists(txn)
            .await
            .or_raise(|| {
                Error::new("Error querying permissions", ErrorType::Permission)
            })?)
    }

    async fn fetch_permissions(
        ctx: &ServiceContext<'_>,
        role_id: i64,
    ) -> Result<Vec<Permission>> {
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
                resource: p.resource_type,
                resource_category: p.resource_category_id.map(Reference::Id),
                action: p.action,
            })
            .collect())
    }

    /// Fetches permissions for `role_id` as a set for easy comparison in hierarchy validation.
    pub async fn permissions_as_set(
        ctx: &ServiceContext<'_>,
        role_id: i64,
    ) -> Result<HashSet<Permission>> {
        Ok(Self::fetch_permissions(ctx, role_id)
            .await?
            .into_iter()
            .collect())
    }

    async fn cascade_permission_removals(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        child_role_id: i64,
        removed_permissions: &HashSet<Permission>,
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
        let to_remove: HashSet<Permission> = child_perms
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
            RolePermission::delete_many()
                .filter(role_permission::Column::RoleId.eq(child_role_id))
                .filter(role_permission::Column::ResourceType.eq(perm.resource))
                .filter(match resource_category_id {
                    Some(id) => role_permission::Column::ResourceCategoryId.eq(id),
                    None => role_permission::Column::ResourceCategoryId.is_null(),
                })
                .filter(role_permission::Column::Action.eq(perm.action))
                .exec(txn)
                .await
                .or_raise(make_error)?;
        }

        // Recursively cascade to grandchildren
        let grandchildren = Role::find()
            .filter(
                role::Column::ParentRoleId
                    .eq(child_role_id)
                    .and(role::Column::SiteId.eq(site_id))
                    .and(role::Column::DeletedAt.is_null()),
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
