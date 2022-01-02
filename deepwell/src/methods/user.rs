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
use crate::services::user::{
    CreateUser, UpdateUser, UserIdentityOutput, UserInfoOutput, UserProfileOutput,
};
use crate::web::{UserDetails, UserDetailsQuery};

pub async fn user_create(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let input: CreateUser = req.body_json().await?;
    let output = req.user(&txn).create(input).await.to_api()?;
    let body = Body::from_json(&output)?;
    txn.commit().await?;
    Ok(body.into())
}

pub async fn user_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let reference = ItemReference::try_from(&req)?;
    let user = req.user(&txn).get(reference).await.to_api()?;
    txn.commit().await?;
    let UserDetailsQuery { detail } = req.query()?;
    build_user_response(&user, detail, StatusCode::Ok)
}

pub async fn user_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let input: UpdateUser = req.body_json().await?;
    let reference = ItemReference::try_from(&req)?;
    let user = req.user(&txn).update(reference, input).await.to_api()?;
    txn.commit().await?;
    build_user_response(&user, UserDetails::default(), StatusCode::Created)
}

pub async fn user_delete(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let reference = ItemReference::try_from(&req)?;
    let user = req.user(&txn).delete(reference).await.to_api()?;
    txn.commit().await?;
    build_user_response(&user, UserDetails::default(), StatusCode::Ok)
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
