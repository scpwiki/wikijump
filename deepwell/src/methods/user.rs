/*
 * methods/user.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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
use crate::models::users::Model as UserModel;
use crate::services::user::{CreateUser, UpdateUser};

pub async fn user_create(mut req: ApiRequest) -> ApiResponse {
    let input: CreateUser = req.body_json().await?;
    let output = req.user().create(input).await?;
    let body = Body::from_json(&output)?;
    Ok(body.into())
}

pub async fn user_get(req: ApiRequest) -> ApiResponse {
    let reference = ItemReference::try_from(&req)?;
    let user = req.user().get(reference).await?;
    build_user_response(&user, StatusCode::Ok)
}

pub async fn user_put(mut req: ApiRequest) -> ApiResponse {
    let input: UpdateUser = req.body_json().await?;
    let reference = ItemReference::try_from(&req)?;
    let user = req.user().update(reference, input).await?;
    build_user_response(&user, StatusCode::Created)
}

pub async fn user_delete(req: ApiRequest) -> ApiResponse {
    let reference = ItemReference::try_from(&req)?;
    let user = req.user().delete(reference).await?;
    build_user_response(&user, StatusCode::Ok)
}

fn build_user_response(user: &UserModel, status: StatusCode) -> ApiResponse {
    // This includes fields like the password hash.
    //
    // For now this is fine, but depending on what
    // we want the usage of the API to be, we may
    // want to filter out fields.
    let body = Body::from_json(user)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
