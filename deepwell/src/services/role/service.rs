/*
 * services/role/service.rs
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
use crate::endpoints::user;
use crate::error::{Error, ErrorType};
use crate::models::prelude::Page;
use crate::models::role::{self, Entity as Role, Model as RoleModel};
use crate::models::role_permission::{
    self, Entity as RolePermission, Model as RolePermissionModel,
};
use crate::models::user_role::{Entity as UserRole, Model as UserRoleModel};
use crate::models::{page, user_role};
use crate::services::audit::{AuditEvent, AuditService};
use crate::services::permission::{
    CheckPermissionContext, PermissionService, resolve_category_reference,
};
use crate::services::relation::{
    GetPageAttributions, GetSiteBan, GetSiteMember, SiteMemberAccepted,
};
use crate::services::role::SystemRole;
use crate::services::{PageService, RelationService, ServiceContext};
use crate::types::{Action, Permission, Reference, Resource};
use crate::utils::{now, trim_default};
use sea_orm::prelude::Expr;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::net::IpAddr;
use std::str::FromStr;

#[derive(Debug)]
pub struct RoleService;

#[allow(dead_code)] // Temp
impl RoleService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        InternalCreateRoleInput {
            site_id,
            name,
            description,
            is_virtual,
            parent_role_id,
            creating_user_id,
            ip_address,
        }: InternalCreateRoleInput,
    ) -> Result<RoleModel> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!("failed to create role '{}' in site ID {}", name, site_id,),
                ErrorType::Role,
            )
        };

        // Validate parent role (if specified) exists
        if let Some(parent_id) = parent_role_id {
            Self::assert_exists(ctx, site_id, parent_id.into())
                .await
                .or_raise(make_error)?;
        }

        // Insert role
        let now = now();

        let model = role::ActiveModel {
            site_id: Set(site_id),
            name: Set(name.clone()),
            description: Set(description.clone().unwrap_or_default()),
            is_virtual: Set(is_virtual),
            created_at: Set(now),
            parent_role_id: Set(parent_role_id),
            ..Default::default()
        };

        let created_role = model.insert(txn).await.or_raise(make_error)?;

        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::RoleCreate {
                site_id: created_role.site_id,
                role_id: created_role.role_id,
                creating_user_id,
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(created_role)
    }

    pub async fn update(
        ctx: &ServiceContext<'_>,
        UpdateRoleInput {
            site_id,
            role_id,
            name,
            description,
            updating_user_id,
            ip_address,
        }: UpdateRoleInput,
    ) -> Result<RoleModel> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!(
                    "failed to update role ID {}, changed by user ID {}",
                    role_id, updating_user_id,
                ),
                ErrorType::Role,
            )
        };

        // Validate that the role belongs to the site
        Self::assert_exists(ctx, site_id, role_id.into())
            .await
            .or_raise(make_error)?;

        let mut model = role::ActiveModel {
            role_id: Set(role_id),
            updated_at: Set(Some(now())),
            ..Default::default()
        };

        // Update fields
        if let Maybe::Set(name_val) = &name {
            model.name = Set(name_val.clone());
        }

        if let Maybe::Set(description_val) = &description {
            model.description = Set(description_val.clone());
        }

        let updated_role = model.update(txn).await.or_raise(make_error)?;

        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::RoleUpdate {
                role_id,
                name: name.to_option().map(String::as_str),
                description: description.to_option().map(String::as_str),
                updating_user_id,
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(updated_role)
    }

    pub async fn delete(
        ctx: &ServiceContext<'_>,
        DeleteRoleInput {
            site_id,
            role_id,
            deleting_user_id,
            reparent_children,
            ip_address,
        }: DeleteRoleInput,
    ) -> Result<RoleModel> {
        let txn = ctx.transaction();

        let role = Self::get(ctx, site_id, role_id.into())
            .await
            .or_raise(|| Error::new("failed to delete role", ErrorType::Role))?;

        let make_error = || {
            Error::new(
                format!(
                    "failed to delete role ID {} by user ID {}",
                    role.role_id, deleting_user_id,
                ),
                ErrorType::Role,
            )
        };

        // Check for child roles
        let child_roles = Role::find()
            .filter(
                Condition::all()
                    .add(role::Column::ParentRoleId.eq(role.role_id))
                    .add(role::Column::DeletedAt.is_null()),
            )
            .all(txn)
            .await
            .or_raise(make_error)?;

        if !child_roles.is_empty() {
            if reparent_children {
                if let Some(parent_id) = role.parent_role_id {
                    // Reparent child roles to the deleted role's parent
                    Role::update_many()
                        .col_expr(
                            role::Column::ParentRoleId,
                            Expr::value(Some(parent_id)),
                        )
                        .filter(
                            Condition::all()
                                .add(role::Column::ParentRoleId.eq(role.role_id))
                                .add(role::Column::DeletedAt.is_null()),
                        )
                        .exec(txn)
                        .await
                        .or_raise(make_error)?;
                } else {
                    // Cannot reparent to null, bail with error
                    // TODO: Figure out what to do with top-level role, if we are not having a "root" role.
                    bail!(Error::new(
                        format!(
                            "cannot delete top-level role ID {} because it has child roles and reparenting is not possible",
                            role.role_id,
                        ),
                        ErrorType::DeleteRoleWithChildren,
                    ));
                }
            } else {
                bail!(Error::new(
                    format!(
                        "cannot delete role ID {} because it has child roles",
                        role.role_id,
                    ),
                    ErrorType::DeleteRoleWithChildren,
                ));
            }
        }

        // Remove this role from all users who actively have it
        UserRole::update_many()
            .col_expr(user_role::Column::DeletedAt, Expr::value(Some(now())))
            .filter(
                Condition::all()
                    .add(user_role::Column::RoleId.eq(role.role_id))
                    .add(user_role::Column::DeletedAt.is_null()),
            )
            .exec(txn)
            .await
            .or_raise(|| {
                Error::new("failed to remove role from users", ErrorType::Role)
            })?;

        let deleted_role = role::ActiveModel {
            role_id: Set(role.role_id),
            deleted_at: Set(Some(now())),
            ..Default::default()
        }
        .update(txn)
        .await
        .or_raise(make_error)?;

        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::RoleDelete {
                role_id: role.role_id,
                deleting_user_id,
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(deleted_role)
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
    ) -> Result<Option<RoleModel>> {
        let txn = ctx.transaction();

        let make_error = || Error::new("failed to get role", ErrorType::Role);

        let role = {
            let condition = match reference {
                Reference::Id(id) => Condition::from(role::Column::RoleId.eq(id)),
                Reference::Slug(slug) => {
                    // Get role by role name
                    Condition::all()
                        .add(role::Column::Name.eq(slug))
                        .add(role::Column::DeletedAt.is_null())
                }
            };

            Role::find()
                .filter(
                    Condition::all()
                        .add(condition)
                        .add(role::Column::SiteId.eq(site_id)),
                )
                .one(txn)
                .await
                .or_raise(make_error)?
        };

        Ok(role)
    }

    #[inline]
    pub async fn get(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
    ) -> Result<RoleModel> {
        find_or_error!(Self::get_optional(ctx, site_id, reference), "role", Role)
    }

    #[inline]
    pub async fn assert_exists(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
    ) -> Result<()> {
        let _ = Self::get(ctx, site_id, reference).await?;
        Ok(())
    }

    pub async fn grant_role_to_user(
        ctx: &ServiceContext<'_>,
        GrantUserRoleInput {
            user_id,
            role_id,
            site_id,
            assigning_user_id,
            expires_at,
            ip_address,
        }: GrantUserRoleInput,
    ) -> Result<UserRoleModel> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!("failed to grant role ID {} to user ID {}", role_id, user_id,),
                ErrorType::GrantUserRole,
            )
        };

        crate::services::RelationService::check_site_ban(
            ctx,
            crate::services::relation::GetSiteBan { site_id, user_id },
            "receive a site role",
        )
        .await
        .or_raise(make_error)?;

        let role = Self::get(ctx, site_id, role_id.into()).await?;

        let user_role = user_role::ActiveModel {
            user_id: Set(user_id),
            role_id: Set(role.role_id),
            site_id: Set(role.site_id),
            assigned_at: Set(now()),
            assigned_by: Set(assigning_user_id),
            expires_at: Set(expires_at),
            ..Default::default()
        }
        .insert(txn)
        .await
        .or_raise(make_error)?;

        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::GrantUserRole {
                user_id,
                role_id,
                assigning_user_id,
                expires_at,
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(user_role)
    }

    pub async fn revoke_role_from_user(
        ctx: &ServiceContext<'_>,
        RevokeUserRoleInput {
            user_id,
            role_id,
            site_id: _,
            revoking_user_id,
            ip_address,
        }: RevokeUserRoleInput,
    ) -> Result<UserRoleModel> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!(
                    "failed to revoke role ID {} from user ID {}",
                    role_id, user_id,
                ),
                ErrorType::RevokeUserRole,
            )
        };

        let deleted_user_role = user_role::ActiveModel {
            user_id: Set(user_id),
            role_id: Set(role_id),
            deleted_at: Set(Some(now())),
            ..Default::default()
        }
        .update(txn)
        .await
        .or_raise(make_error)?;

        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::RevokeUserRole {
                user_id,
                role_id,
                revoking_user_id,
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(deleted_user_role)
    }

    pub async fn get_all_roles_for_user_and_site(
        ctx: &ServiceContext<'_>,
        input: GetUserRolesInput<'_>,
    ) -> Result<Vec<RoleModel>> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!(
                    "failed to get roles for user ID {:?} in site ID {}",
                    input.user_id, input.site_id
                ),
                ErrorType::Role,
            )
        };

        let mut roles = match input.user_id {
            Some(id) => Role::find()
                .join(JoinType::InnerJoin, role::Relation::UserRole.def())
                .filter(
                    Condition::all()
                        .add(user_role::Column::UserId.eq(id))
                        .add(role::Column::SiteId.eq(input.site_id))
                        .add(role::Column::DeletedAt.is_null())
                        .add(user_role::Column::DeletedAt.is_null())
                        .add(
                            Condition::any()
                                .add(user_role::Column::ExpiresAt.is_null())
                                .add(user_role::Column::ExpiresAt.gt(now())),
                        ),
                )
                .all(txn)
                .await
                .or_raise(make_error)?,
            None => Vec::new(),
        };

        let virtual_roles = Self::get_virtual_roles_for_user(ctx, &input)
            .await
            .or_raise(make_error)?;

        roles.extend(virtual_roles);

        info!(
            "User ID {:?} has these roles in site ID {}: {:?}",
            input.user_id,
            input.site_id,
            roles.iter().map(|r| &r.name).collect::<Vec<_>>()
        );

        Ok(roles)
    }

    pub async fn reparent_role(
        ctx: &ServiceContext<'_>,
        InternalReparentRoleInput {
            site_id,
            role_id,
            new_parent_id,
            reparenting_user_id,
            ip_address,
        }: InternalReparentRoleInput,
    ) -> Result<()> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!(
                    "failed to reparent role ID {} under parent ID {:?} in site ID {}",
                    role_id, new_parent_id, site_id,
                ),
                ErrorType::Role,
            )
        };

        // Perform validations
        if let Some(parent_id) = new_parent_id {
            // Validate that parent role is not self
            if parent_id == role_id {
                bail!(Error::new(
                    format!("role ID {} cannot be its own parent", role_id),
                    ErrorType::CyclicRoleViolation {
                        role_id,
                        parent_role_id: parent_id
                    },
                ));
            }

            // Validate that the new parent role exists
            Self::assert_exists(ctx, site_id, parent_id.into())
                .await
                .or_raise(make_error)?;

            let is_proper_subset =
                Self::validate_child_role_subset_of_parent(ctx, role_id, parent_id)
                    .await
                    .or_raise(make_error)?;

            // Hacky solution to avoid running the expensive cycle check.
            // If the new parent has more permissions than the child, it cannot be a descendant of the child, so we can skip the cycle check.
            // This assumes that the role tree is valid and constraints are respected.
            if !is_proper_subset {
                Self::validate_reparent_no_cycle(ctx, site_id, role_id, parent_id)
                    .await
                    .or_raise(make_error)?;
            }
        }

        role::ActiveModel {
            role_id: Set(role_id),
            parent_role_id: Set(new_parent_id),
            updated_at: Set(Some(now())),
            ..Default::default()
        }
        .update(txn)
        .await
        .or_raise(make_error)?;

        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::RoleReparent {
                role_id,
                new_parent_id,
                reparenting_user_id,
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(())
    }

    /// Validates that the permissions of the child role are a subset of the permissions of the parent role.
    /// Throws an error if constraint is violated.
    ///
    /// Returns true if the child role has strictly fewer permissions than the parent role (a proper subset),
    /// and false if the child role has the same permissions as the parent role.
    async fn validate_child_role_subset_of_parent(
        ctx: &ServiceContext<'_>,
        child_role_id: i64,
        parent_role_id: i64,
    ) -> Result<bool> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to validate permissions for child role ID {} and parent role ID {}",
                    child_role_id, parent_role_id,
                ),
                ErrorType::Role,
            )
        };

        let child_permissions = PermissionService::permissions_as_set(ctx, child_role_id)
            .await
            .or_raise(make_error)?;

        let parent_permissions =
            PermissionService::permissions_as_set(ctx, parent_role_id)
                .await
                .or_raise(make_error)?;

        if !child_permissions.is_subset(&parent_permissions) {
            bail!(Error::new(
                format!(
                    "cannot reparent role ID {} under role ID {} because the child role has permissions that the parent role does not have",
                    child_role_id, parent_role_id,
                ),
                ErrorType::RoleHierarchyViolation {
                    role_id: child_role_id,
                    parent_role_id,
                },
            ));
        }

        Ok(child_permissions.len() < parent_permissions.len())
    }

    /// Checks that (re)parenting `role_id` under `parent_id` would not introduce a cycle.
    ///
    /// Validate that the parent role is not already a descendant of the child role.
    async fn validate_reparent_no_cycle(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        role_id: i64,
        parent_id: i64,
    ) -> Result<()> {
        if parent_id == role_id {
            bail!(Error::new(
                format!("role ID {} cannot be its own parent", role_id),
                ErrorType::CyclicRoleViolation {
                    role_id,
                    parent_role_id: parent_id,
                },
            ));
        }

        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!(
                    "failed to check reparent cycle for role ID {} in site ID {}",
                    role_id, site_id,
                ),
                ErrorType::Role,
            )
        };

        // Load all active roles for the site to build the parent map in memory.
        // TODO: Figure out a more efficient way to do this
        let roles: Vec<RoleModel> = Role::find()
            .filter(
                Condition::all()
                    .add(role::Column::SiteId.eq(site_id))
                    .add(role::Column::DeletedAt.is_null()),
            )
            .all(txn)
            .await
            .or_raise(make_error)?;

        // Hashmap for quick lookup.
        // role_id -> parent_role_id
        let parent_map: HashMap<i64, i64> = roles
            .iter()
            .filter_map(|r| r.parent_role_id.map(|pid| (r.role_id, pid)))
            .collect();

        // Walk the ancestor chain of parent_id upward.
        // If we reach role_id, the reparent would create a cycle.
        let mut current = parent_id;
        loop {
            match parent_map.get(&current) {
                Some(&pid) if pid == role_id => {
                    bail!(Error::new(
                        format!(
                            "reparenting role ID {} under role ID {} would create a cycle",
                            role_id, parent_id,
                        ),
                        ErrorType::CyclicRoleViolation {
                            role_id,
                            parent_role_id: parent_id,
                        },
                    ));
                }
                Some(&pid) => current = pid,
                None => break,
            }
        }

        Ok(())
    }

    pub async fn get_all_roles_for_site(
        ctx: &ServiceContext<'_>,
        site_id: i64,
    ) -> Result<Vec<RoleModel>> {
        let txn = ctx.transaction();

        let make_error = || Error::new("failed to get roles for site", ErrorType::Role);

        let roles = Role::find()
            .filter(
                Condition::all()
                    .add(role::Column::SiteId.eq(site_id))
                    .add(role::Column::DeletedAt.is_null()),
            )
            .all(txn)
            .await
            .or_raise(make_error)?;

        Ok(roles)
    }

    pub async fn get_virtual_roles_for_user(
        ctx: &ServiceContext<'_>,
        input: &GetUserRolesInput<'_>,
    ) -> Result<Vec<RoleModel>> {
        let txn = ctx.transaction();

        let make_error = || Error::new("failed to apply virtual roles", ErrorType::Role);

        let virtual_roles = Role::find()
            .filter(
                Condition::all()
                    .add(role::Column::SiteId.eq(input.site_id))
                    .add(role::Column::IsVirtual.eq(true)), // Virtual roles are never deleted, so we don't need to check DeletedAt
            )
            .all(txn)
            .await
            .or_raise(make_error)?;

        let virtual_role_name_map = virtual_roles
            .into_iter()
            .map(|role| (role.name.clone(), role))
            .collect::<std::collections::HashMap<_, _>>();

        // Compute user state flags
        let is_logged_in = input.user_id.is_some();
        let is_member = if is_logged_in {
            let membership = RelationService::get_optional_site_member(
                ctx,
                GetSiteMember {
                    site_id: input.site_id,
                    user_id: input.user_id.unwrap(),
                },
            )
            .await
            .or_raise(make_error)?;

            membership.is_some()
        } else {
            false
        };
        let is_page_author = if is_member && let Some(page_ref) = &input.page_reference {
            let attributions = RelationService::get_page_attributions(
                ctx,
                GetPageAttributions {
                    site_id: input.site_id,
                    page: page_ref.clone(),
                },
            )
            .await
            .or_raise(make_error)?;
            attributions
                .iter()
                .any(|attr| attr.user_id == input.user_id.unwrap())
        } else {
            false
        };
        let is_banned = if is_logged_in {
            RelationService::site_ban_exists(
                ctx,
                GetSiteBan {
                    site_id: input.site_id,
                    user_id: input.user_id.unwrap(),
                },
            )
            .await
            .or_raise(make_error)?
        } else {
            false
        };

        // Collect virtual roles to apply based on flags
        let mut applied_virtual_roles = Vec::with_capacity(4); // At most 4 virtual roles at a time
        if is_logged_in {
            applied_virtual_roles.push(SystemRole::Registered);
            if is_banned {
                // If user is banned, skip any other site roles
                applied_virtual_roles.push(SystemRole::Banned);
            } else {
                if is_member {
                    applied_virtual_roles.push(SystemRole::Member);
                } else {
                    applied_virtual_roles.push(SystemRole::Guest);
                }
                if is_page_author {
                    applied_virtual_roles.push(SystemRole::PageAuthor);
                }
            }
        } else {
            applied_virtual_roles.push(SystemRole::Anonymous);
            applied_virtual_roles.push(SystemRole::Guest);
        }
        applied_virtual_roles.push(SystemRole::Everyone);

        info!(
            "Applying these virtual roles for user ID {:?} in site ID {}: {:?}",
            input.user_id, input.site_id, applied_virtual_roles
        );

        // Build the final list of applicable roles
        let applicable_virtual_roles = applied_virtual_roles
            .into_iter()
            .filter_map(|role| virtual_role_name_map.get(role.into()).cloned())
            .collect();

        Ok(applicable_virtual_roles)
    }
}
