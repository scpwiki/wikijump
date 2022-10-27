/*
 * methods/auth.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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
use crate::services::authentication::{AuthenticateUser, AuthenticationService};
use crate::services::MfaService;

pub async fn auth_login(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: AuthenticateUser = req.body_json().await?;
    let reference = Reference::try_from(&req)?;
    let user = UserService::get(&ctx, reference).await.to_api()?;

    AuthenticationService::auth_user(&ctx, &user, input)
        .await
        .to_api()?;

    // TODO session creation
    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn auth_logout(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    // TODO session closure
    let _ = ctx;
    todo!()
}

pub async fn auth_mfa_setup(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let reference = Reference::try_from(&req)?;
    let user = UserService::get(&ctx, reference).await.to_api()?;
    let output = MfaService::setup(&ctx, &user).await.to_api()?;

    let body = Body::from_json(&output)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    txn.commit().await?;
    Ok(response)
}

pub async fn auth_mfa_disable(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let user_id = 4; // TODO get from session ID
    MfaService::disable(&ctx, user_id).await.to_api()?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn auth_mfa_reset_recovery(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let user_id = 4; // TODO get from session ID
    let user = UserService::get(&ctx, Reference::Id(user_id))
        .await
        .to_api()?;

    let output = MfaService::reset_recovery_codes(&ctx, &user)
        .await
        .to_api()?;

    let body = Body::from_json(&output)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    txn.commit().await?;
    Ok(response)
}
