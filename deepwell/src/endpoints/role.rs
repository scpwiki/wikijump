/*
 * endpoints/role.rs
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
use crate::models::role::Model as RoleModel;
use crate::models::user_role;
use crate::services::permission::{
    CheckPermissionContext, DecoratedPermission, PermissionService,
};
use crate::services::role::{
    CreateRoleInput, DeleteRoleInput, GetRoleInput, GetRolePermissionsInput,
    GetUserRolesInput, GrantUserRoleInput, InternalCreateRoleInput,
    InternalReparentRoleInput, ListSiteRolesInput, ReparentRoleInput,
    RevokeUserRoleInput, RoleService, UpdateRoleInput, UpdateRolePermissionsInput,
};
use crate::types::{Action, Permission, Resource};

async fn require_role_permission(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    action: Action,
) -> Result<i64> {
    let actor_user_id = ctx.request().user_id().or_raise(|| {
        Error::new(
            "user does not have permission to manage roles",
            ErrorType::PermissionDenied,
        )
    })?;

    let can_manage = PermissionService::check_user_can(
        ctx,
        &CheckPermissionContext {
            user_id: Some(actor_user_id),
            site_id,
            page_reference: None,
        },
        Permission {
            resource_type: Resource::Role,
            resource_category: None,
            action,
        },
    )
    .await
    .or_raise(|| Error::new("failed to check role permission", ErrorType::Role))?;

    if !can_manage {
        return Err(Error::new(
            "user does not have permission to manage roles",
            ErrorType::PermissionDenied,
        )
        .into());
    }

    Ok(actor_user_id)
}

pub async fn list_site_roles(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<RoleModel>> {
    let ListSiteRolesInput { site_id } = parse!(params, Role);
    RoleService::get_all_roles_for_site(ctx, site_id)
        .await
        .or_raise(|| Error::new("failed to list roles", ErrorType::Role))
}

pub async fn role_create(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<RoleModel> {
    let input: CreateRoleInput = parse!(params, Role);
    let actor_user_id = require_role_permission(ctx, input.site_id, Action::Edit).await?;

    info!(
        "Creating role '{}' in site ID {}",
        input.name, input.site_id
    );

    // Translating to internal input, as there are some fields that are not meant to be set by users
    let translated = InternalCreateRoleInput {
        site_id: input.site_id,
        name: input.name,
        description: input.description,
        is_virtual: false,
        parent_role_id: Some(input.parent_role_id),
        creating_user_id: actor_user_id,
        ip_address: input.ip_address,
    };

    RoleService::create(ctx, translated)
        .await
        .or_raise(|| Error::new("failed to create role", ErrorType::Role))
}

pub async fn role_delete(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<RoleModel> {
    let mut input: DeleteRoleInput = parse!(params, Role);
    input.deleting_user_id =
        require_role_permission(ctx, input.site_id, Action::Edit).await?;

    info!(
        "Deleting role ID {} in site ID {}",
        input.role_id, input.site_id
    );

    RoleService::delete(ctx, input)
        .await
        .or_raise(|| Error::new("failed to delete role", ErrorType::Role))
}

pub async fn role_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<RoleModel> {
    let input: GetRoleInput = parse!(params, Role);
    RoleService::get(ctx, input.site_id, input.role_reference)
        .await
        .or_raise(|| Error::new("role not found", ErrorType::Role))
}

pub async fn role_reparent(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let input: ReparentRoleInput = parse!(params, Role);
    let actor_user_id = require_role_permission(ctx, input.site_id, Action::Edit).await?;

    info!(
        "Reparenting role ID {} to new parent {} in site ID {}",
        input.role_id, input.new_parent_id, input.site_id
    );

    // Translating to internal input, as there are some fields that are not meant to be set by users
    let translated = InternalReparentRoleInput {
        site_id: input.site_id,
        role_id: input.role_id,
        new_parent_id: Some(input.new_parent_id),
        reparenting_user_id: actor_user_id,
        ip_address: input.ip_address,
    };

    RoleService::reparent_role(ctx, translated)
        .await
        .or_raise(|| Error::new("failed to reparent role", ErrorType::Role))
}

pub async fn role_update_info(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<RoleModel> {
    let mut input: UpdateRoleInput = parse!(params, Role);
    input.updating_user_id =
        require_role_permission(ctx, input.site_id, Action::Edit).await?;

    info!(
        "Updating role {:?} in site ID {}",
        input.role_id, input.site_id
    );

    RoleService::update(ctx, input)
        .await
        .or_raise(|| Error::new("failed to update role", ErrorType::Role))
}

pub async fn role_update_permissions(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let mut input: UpdateRolePermissionsInput = parse!(params, Role);
    input.updating_user_id =
        require_role_permission(ctx, input.site_id, Action::Edit).await?;

    info!(
        "Updating permissions for role {:?} in site ID {}",
        input.role_reference, input.site_id
    );

    PermissionService::update_permissions_for_role(ctx, input)
        .await
        .or_raise(|| Error::new("failed to update role permissions", ErrorType::Role))
}

pub async fn get_role_permissions(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<Permission<'static>>> {
    let input: GetRolePermissionsInput = parse!(params, Role);
    PermissionService::get_permissions_for_role(ctx, input)
        .await
        .or_raise(|| Error::new("failed to get role permissions", ErrorType::Role))
}

pub async fn get_decorated_role_permissions(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<DecoratedPermission<'static>>> {
    let input: GetRolePermissionsInput = parse!(params, Role);
    PermissionService::get_decorated_permissions_for_role(ctx, input)
        .await
        .or_raise(|| Error::new("failed to get role permissions", ErrorType::Role))
}

pub async fn grant_role_to_user(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<user_role::Model> {
    let mut input: GrantUserRoleInput = parse!(params, Role);
    input.assigning_user_id =
        require_role_permission(ctx, input.site_id, Action::Assign).await?;

    info!(
        "Granting role ID {} to user ID {} in site ID {}",
        input.role_id, input.user_id, input.site_id
    );

    RoleService::grant_role_to_user(ctx, input)
        .await
        .or_raise(|| Error::new("failed to grant role to user", ErrorType::Role))
}

pub async fn get_user_roles(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<RoleModel>> {
    let input: GetUserRolesInput = parse!(params, Role);
    RoleService::get_all_roles_for_user_and_site(ctx, input)
        .await
        .or_raise(|| Error::new("failed to get user roles", ErrorType::Role))
}

pub async fn revoke_role_from_user(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<user_role::Model> {
    let mut input: RevokeUserRoleInput = parse!(params, Role);
    input.revoking_user_id =
        require_role_permission(ctx, input.site_id, Action::Assign).await?;

    info!(
        "Revoking role ID {} from user ID {} in site ID {}",
        input.role_id, input.user_id, input.site_id
    );

    RoleService::revoke_role_from_user(ctx, input)
        .await
        .or_raise(|| Error::new("failed to revoke role from user", ErrorType::Role))
}
