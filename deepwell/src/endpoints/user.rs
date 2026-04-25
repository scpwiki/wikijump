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
use crate::{
    models::user::Model as UserModel,
    services::user::{CreateUser, CreateUserOutput, GetUser, GetUserOutput, UpdateUser},
    types::AliasType,
};

pub async fn user_create(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<CreateUserOutput> {
    info!("Creating new regular user");
    let input: CreateUser = parse!(params, User);

    UserService::create(ctx, input)
        .await
        .or_raise(|| Error::new("failed to create user", ErrorType::User))
}

pub async fn user_import(
    _ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<CreateUserOutput> {
    // TODO implement importing user from Wikidot
    todo!()
}

pub async fn user_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<GetUserOutput>> {
    let GetUser { user: reference } = parse!(params, User);
    info!("Getting user {reference:?}");

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

    info!("Updating user {reference:?}");
    UserService::update(ctx, reference, ip_address, body)
        .await
        .or_raise(|| Error::new("failed to update user", ErrorType::User))
}

pub async fn user_delete(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<UserModel> {
    let GetUser { user: reference } = parse!(params, User);
    info!("Deleting user {reference:?}");
    UserService::delete(ctx, reference)
        .await
        .or_raise(|| Error::new("failed to delete user", ErrorType::User))
}

pub async fn user_add_name_change(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<i16> {
    let GetUser { user: reference } = parse!(params, User);
    let make_error = || Error::new("failed to add name change to user", ErrorType::User);

    info!("Adding user name change token to {reference:?}");
    let user = UserService::get(ctx, reference)
        .await
        .or_raise(make_error)?;

    UserService::add_name_change_token(ctx, &user)
        .await
        .or_raise(make_error)
}
