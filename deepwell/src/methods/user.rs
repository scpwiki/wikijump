/*
 * methods/user.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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
use crate::models::sea_orm_active_enums::UserType;
use crate::models::user::Model as UserModel;
use crate::services::user::{
    CreateUser, UpdateUser, UserIdentityOutput, UserInfoOutput, UserProfileOutput,
};
use crate::web::{UserDetails, UserDetailsQuery};

pub async fn user_create(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    tide::log::info!("Creating new regular user");
    let input: CreateUser = req.body_json().await?;
    let output = UserService::create(&ctx, UserType::Regular, input)
        .await
        .to_api()?;

    let body = Body::from_json(&output)?;
    txn.commit().await?;

    let response = Response::builder(StatusCode::Created).body(body).into();
    Ok(response)
}

pub async fn user_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let UserDetailsQuery { detail } = req.query()?;
    let reference = Reference::try_from(&req)?;
    tide::log::info!("Getting user {:?} (details {})", reference, detail.name());

    let user = UserService::get(&ctx, reference).await.to_api()?;
    txn.commit().await?;
    build_user_response(&user, detail, StatusCode::Ok)
}

pub async fn user_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: UpdateUser = req.body_json().await?;
    let reference = Reference::try_from(&req)?;
    tide::log::info!("Updating user {:?}", reference);

    UserService::update(&ctx, reference, input).await.to_api()?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn user_delete(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let UserDetailsQuery { detail } = req.query()?;
    let reference = Reference::try_from(&req)?;
    tide::log::info!("Deleting user {:?} (details {})", reference, detail.name());

    let user = UserService::delete(&ctx, reference).await.to_api()?;
    txn.commit().await?;
    build_user_response(&user, detail, StatusCode::Ok)
}

pub async fn user_add_name_change(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let reference = Reference::try_from(&req)?;
    tide::log::info!("Adding user name change token to {:?}", reference);

    let name_changes = UserService::add_name_change_token(&ctx, reference)
        .await
        .to_api()?;

    let body = Body::from_json(&name_changes)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    txn.commit().await?;
    Ok(response)
}

fn build_user_response(
    user: &UserModel,
    user_detail: UserDetails,
    status: StatusCode,
) -> ApiResponse {
    // TODO: allow dumping the entire user model (internal API only)
    let body = match user_detail {
        UserDetails::Identity => Body::from_json(&UserIdentityOutput::from(user))?,
        UserDetails::Info => Body::from_json(&UserInfoOutput::from(user))?,
        UserDetails::Profile => Body::from_json(&UserProfileOutput::from(user))?,
    };
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
