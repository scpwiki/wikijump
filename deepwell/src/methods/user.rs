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
use crate::models::user::Model as UserModel;
use crate::models::user_alias::Model as UserAliasModel;
use crate::services::user::{
    CreateUser, GetUser, GetUserOutput, UpdateUser, UpdateUserBody,
};
use crate::web::ProvidedValue;

pub async fn user_create(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    tide::log::info!("Creating new regular user");
    let input: CreateUser = req.body_json().await?;
    let output = UserService::create(&ctx, input).await.to_api()?;

    let body = Body::from_json(&output)?;
    txn.commit().await?;

    let response = Response::builder(StatusCode::Created).body(body).into();
    Ok(response)
}

pub async fn user_get(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let GetUser { user: reference } = req.body_json().await?;
    tide::log::info!("Getting user {:?}", reference);

    let user = UserService::get(&ctx, reference).await.to_api()?;
    let aliases = UserAliasService::get_all(&ctx, user.user_id)
        .await
        .to_api()?;

    txn.commit().await?;
    build_user_response(user, aliases, StatusCode::Ok)
}

pub async fn user_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let UpdateUser {
        user: reference,
        body,
    } = req.body_json().await?;

    tide::log::info!("Updating user {:?}", reference);

    UserService::update(&ctx, reference, body).await.to_api()?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn user_delete(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let GetUser { user: reference } = req.body_json().await?;
    tide::log::info!("Deleting user {:?}", reference);

    UserService::delete(&ctx, reference).await.to_api()?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

// Separate route because a JSON-encoded byte list is very inefficient.
pub async fn user_avatar_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let GetUser { user: reference } = req.query()?;
    let bytes = req.body_bytes().await?;

    let avatar = if bytes.is_empty() {
        // An empty body means delete the avatar
        tide::log::info!("Remove avatar for user {reference:?}");
        None
    } else {
        // Upload file contents from body
        tide::log::info!("Uploading avatar for user {reference:?}");
        Some(bytes)
    };

    UserService::update(
        &ctx,
        reference,
        UpdateUserBody {
            avatar: ProvidedValue::Set(avatar),
            ..Default::default()
        },
    )
    .await
    .to_api()?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn user_add_name_change(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let GetUser { user: reference } = req.body_json().await?;
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
    user: UserModel,
    aliases: Vec<UserAliasModel>,
    status: StatusCode,
) -> ApiResponse {
    let output = GetUserOutput { user, aliases };
    let body = Body::from_json(&output)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
