/*
 * endpoints/user.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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
use crate::models::sea_orm_active_enums::AliasType;
use crate::models::user::Model as UserModel;
use crate::services::user::{
    CreateUser, CreateUserOutput, GetUser, GetUserOutput, UpdateUser,
};

pub async fn user_create(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<CreateUserOutput> {
    info!("Creating new regular user");
    let input: CreateUser = params.parse()?;
    UserService::create(ctx, input).await
}

pub async fn user_import(
    _ctx: &ServiceContext,
    _params: Params<'static>,
) -> Result<CreateUserOutput> {
    // TODO implement importing user from Wikidot
    todo!()
}

pub async fn user_get(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<Option<GetUserOutput>> {
    let GetUser { user: reference } = params.parse()?;
    info!("Getting user {:?}", reference);

    match UserService::get_optional(ctx, reference).await? {
        None => Ok(None),
        Some(user) => {
            let aliases =
                AliasService::get_all(ctx, AliasType::User, user.user_id).await?;

            Ok(Some(GetUserOutput { user, aliases }))
        }
    }
}

pub async fn user_edit(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<UserModel> {
    let UpdateUser {
        user: reference,
        body,
    } = params.parse()?;

    info!("Updating user {:?}", reference);
    UserService::update(ctx, reference, body).await
}

pub async fn user_delete(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<UserModel> {
    let GetUser { user: reference } = params.parse()?;
    info!("Deleting user {:?}", reference);
    UserService::delete(ctx, reference).await
}

pub async fn user_add_name_change(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<i16> {
    let GetUser { user: reference } = params.parse()?;
    info!("Adding user name change token to {:?}", reference);
    let user = UserService::get(ctx, reference).await?;
    UserService::add_name_change_token(ctx, &user).await
}
