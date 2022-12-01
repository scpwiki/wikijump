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
use crate::services::authentication::{
    AuthenticateUser, AuthenticationService, MultiFactorAuthenticateUser,
};
use crate::services::session::{RenewSession, InvalidateOtherSessions, SessionInputOutput, VerifySession};
use crate::services::{Error, MfaService, SessionService};
use crate::web::UserIdQuery;

pub async fn auth_login(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);
    let input: AuthenticateUser = req.body_json().await?;

    // Don't allow empty passwords.
    //
    // They are never valid, and are potentially indicative of the user
    // entering the password in the name field instead, which we do
    // *not* want to be logging.
    if input.password.is_empty() {
        tide::log::error!("User submitted empty password in auth request");
        return Err(TideError::from_str(StatusCode::BadRequest, ""));
    }

    // All authentication issue should return the same error.
    //
    // If anything went wrong, only allow a generic backend failure
    // to avoid leaking internal state.
    //
    // The only three possible responses to this method should be:
    // * success
    // * invalid authentication
    // * server error
    let result = AuthenticationService::auth_password(&ctx, input).await;
    let output = match result {
        Ok(output) => output,
        Err(error) => {
            let status_code = match error {
                Error::InvalidAuthentication => StatusCode::Forbidden,
                _ => {
                    tide::log::error!(
                        "Unexpected error during user authentication: {error}",
                    );

                    StatusCode::InternalServerError
                }
            };

            return Err(TideError::from_str(status_code, ""));
        }
    };

    // TODO session creation
    //      look at output.needs_mfa

    let body = Body::from_json(&output)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    txn.commit().await?;
    Ok(response)
}

pub async fn auth_session_get(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);
    let UserIdQuery { user_id } = req.query()?;

    let sessions = SessionService::get_all(&ctx, user_id).await.to_api()?;

    let body = Body::from_json(&sessions)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    txn.commit().await?;
    Ok(response)
}

pub async fn auth_session_validate(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);
    let VerifySession { session_token, user_id } = req.body_json().await?;

    SessionService::verify(&ctx, &session_token, user_id).await.to_api()?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn auth_session_renew(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);
    let input: RenewSession = req.body_json().await?;

    let session_token = SessionService::renew(&ctx, input).await.to_api()?;

    let body = Body::from_json(&SessionInputOutput { session_token })?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    txn.commit().await?;
    Ok(response)
}

pub async fn auth_session_invalidate_others(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);
    let InvalidateOtherSessions { session_token, user_id } = req.body_json().await?;

    let invalidated = SessionService::invalidate_others(&ctx, &session_token, user_id).await.to_api()?;

    let body = Body::from_json(&invalidated)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    txn.commit().await?;
    Ok(response)
}

pub async fn auth_logout(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);
    let SessionInputOutput { session_token } = req.body_json().await?;

    SessionService::invalidate(&ctx, session_token).await?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn auth_mfa_verify(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);
    let input: MultiFactorAuthenticateUser = req.body_json().await?;

    AuthenticationService::auth_mfa(&ctx, input).await?;

    // TODO session recreation

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
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
