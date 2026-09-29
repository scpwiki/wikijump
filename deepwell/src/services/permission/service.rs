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
use crate::services::ServiceContext;
use crate::services::audit::{AuditEvent, AuditService};
use crate::services::permission::cache::PermissionCacheKey;
use crate::services::permission::resolvers::resolve_category_slug;
use crate::services::permission::{PermissionCache, resolve_category_reference};
use crate::services::role::{
    GetRolePermissionsInput, GetUserRolesInput, GetUserVirtualRolesInput, RoleService,
    UpdateRolePermissionsInput,
};
use crate::types::{Action, Permission, Reference, Resource};
use futures::future::try_join_all;
use std::borrow::Cow;
use std::collections::HashSet;
use std::hash::Hash;
use std::net::IpAddr;

#[derive(Debug)]
pub struct PermissionService;

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
        let role_id = match role_reference {
            Reference::Id(id) => id,
            Reference::Slug(_) => {
                RoleService::get(ctx, site_id, role_reference)
                    .await
                    .or_raise(|| {
                        Error::new("failed to get role for permissions", ErrorType::Role)
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
        let mut permissions = Self::get_permissions_for_role_helper(ctx, role_id)
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

    pub async fn check_user_can(
        ctx: &ServiceContext<'_>,
        input: CheckPermissionInput,
    ) -> Result<bool> {
        // Capture the target-derived values before passing the target by value.
        let resource_type = input.target.resource_type();
        let resource_category_id = input.target.category_id();

        let make_error = || {
            Error::new(
                format!("failed to get permissions for user {:?}", input.user_id),
                ErrorType::Permission,
            )
        };

        // Get cached or fetch effective permission set for this target resource
        let permission_cache_key = PermissionCacheKey {
            user_id: input.user_id,
            site_id: input.site_id,
            target: input.target,
        };
        let permissions = ctx
            .permission_cache()
            .get_or_fetch(ctx, permission_cache_key)
            .await?;

        // Does this category have permissions scoped to it?
        let has_scoped_permissions = match (input.site_id, resource_category_id) {
            (Some(site_id), Some(category_id)) => Self::check_category_scoped(
                ctx,
                site_id,
                resource_type,
                category_id,
                input.action,
            )
            .await
            .or_raise(make_error)?,
            _ => false,
        };

        let has_permission = if has_scoped_permissions {
            permissions.contains(&Permission {
                resource_type,
                resource_category: Some(Reference::Id(resource_category_id.unwrap())),
                action: input.action,
            })
        } else {
            // If category does not have scoped permissions, fallback to _default
            permissions.contains(&Permission {
                resource_type,
                resource_category: None,
                action: input.action,
            })
        };

        Ok(has_permission)
    }

    /// Fetches permissions for `role_id`.
    /// This is a separate function that returns Vec to preserve ordering.
    async fn get_permissions_for_role_helper(
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
    pub(crate) async fn permissions_as_set(
        ctx: &ServiceContext<'_>,
        role_id: i64,
    ) -> Result<HashSet<Permission<'static>>> {
        Ok(Self::get_permissions_for_role_helper(ctx, role_id)
            .await?
            .into_iter()
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
