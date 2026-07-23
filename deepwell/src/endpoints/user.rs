/*
 * endpoints/user.rs
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
use crate::constants::ADMIN_USER_ID;
use crate::models::user::Model as UserModel;
use crate::models::wikidot_user::Entity as WikidotUser;
use crate::services::MutationAuthorization;
use crate::services::user::{
    CreateUser, CreateUserOutput, GetUser, GetUserOutput, UpdateUser,
};
use crate::types::{AliasType, Reference, UserType};
use sea_orm::EntityTrait;

pub async fn user_create(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<CreateUserOutput> {
    info!("Creating new regular user");
    let input: CreateUser = parse!(params, User);

    let privileged_creation = input.user_type != UserType::Regular
        || input.bypass_filter
        || input.bypass_email_verification
        || input.override_user_id.is_some();
    if privileged_creation {
        MutationAuthorization::require_platform_staff(ctx, "privileged user creation")?;
    }

    UserService::create(ctx, input)
        .await
        .or_raise(|| Error::new("failed to create user", ErrorType::User))
}

pub async fn user_import(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<CreateUserOutput> {
    let input: CreateUser = parse!(params, User);
    let Some(user_id) = input.override_user_id else {
        return Err(Error::new(
            "Wikidot user import requires override_user_id",
            ErrorType::BadRequest,
        )
        .into());
    };

    let wikidot_user_id = i32::try_from(user_id).map_err(|_| {
        Error::new(
            format!("Wikidot user ID {user_id} is outside the importable range"),
            ErrorType::BadRequest,
        )
    })?;

    MutationAuthorization::require_platform_staff(ctx, "Wikidot user import")?;

    let make_error = || {
        Error::new(
            format!("failed to import Wikidot user ID {user_id}"),
            ErrorType::User,
        )
    };

    match WikidotUser::find_by_id(wikidot_user_id)
        .one(ctx.transaction())
        .await
        .or_raise(make_error)?
    {
        Some(user) if !user.is_deleted => {
            info!("Importing Wikidot user ID {user_id} into a Wikijump account");
            UserService::import_wikidot(ctx, input)
                .await
                .or_raise(make_error)
        }
        Some(_) => Err(Error::new(
            format!("cannot import deleted Wikidot user ID {user_id}"),
            ErrorType::User,
        )
        .into()),
        None => Err(Error::new(
            format!("cannot import missing Wikidot user ID {user_id}"),
            ErrorType::User,
        )
        .into()),
    }
}

pub async fn user_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<GetUserOutput>> {
    let GetUser { user: reference } = parse!(params, User);

    let make_error = || Error::new("failed to get user", ErrorType::User);
    let user = UserService::get_optional(ctx, reference)
        .await
        .or_raise(make_error)?;

    match user {
        None => Ok(None),
        Some(user) => {
            let aliases = AliasService::get_all(ctx, AliasType::User, user.user_id)
                .await
                .or_raise(make_error)?;

            Ok(Some(GetUserOutput { user, aliases }))
        }
    }
}

pub async fn user_edit(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<UserModel> {
    let UpdateUser {
        user: reference,
        ip_address,
        body,
    } = parse!(params, User);

    let target = UserService::get(ctx, reference)
        .await
        .or_raise(|| Error::new("failed to authorize user update", ErrorType::User))?;
    let actor_user_id = MutationAuthorization::require_self_or_platform_staff(
        ctx,
        target.user_id,
        "update this user",
    )?;
    if actor_user_id != ADMIN_USER_ID
        && (body.email_verified.is_set() || body.bypass_filter)
    {
        return Err(Error::new(
            "only platform staff may set user verification or filter bypass fields",
            ErrorType::PermissionDenied,
        )
        .into());
    }

    info!("Updating user ID {}", target.user_id);
    UserService::update(ctx, Reference::Id(target.user_id), ip_address, body)
        .await
        .or_raise(|| Error::new("failed to update user", ErrorType::User))
}

pub async fn user_delete(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<UserModel> {
    let GetUser { user: reference } = parse!(params, User);
    let target = UserService::get(ctx, reference)
        .await
        .or_raise(|| Error::new("failed to authorize user deletion", ErrorType::User))?;
    MutationAuthorization::require_self_or_platform_staff(
        ctx,
        target.user_id,
        "delete this user",
    )?;

    info!("Deleting user ID {}", target.user_id);
    UserService::delete(ctx, Reference::Id(target.user_id))
        .await
        .or_raise(|| Error::new("failed to delete user", ErrorType::User))
}

pub async fn user_add_name_change(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<i16> {
    let GetUser { user: reference } = parse!(params, User);
    let make_error = || Error::new("failed to add name change to user", ErrorType::User);

    MutationAuthorization::require_platform_staff(
        ctx,
        "granting a user name-change token",
    )?;

    info!("Adding user name change token to {reference:?}");
    let user = UserService::get(ctx, reference)
        .await
        .or_raise(make_error)?;

    UserService::add_name_change_token(ctx, &user)
        .await
        .or_raise(make_error)
}
