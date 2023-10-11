/*
 * endpoints/auth.rs
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
use crate::services::authentication::{
    AuthenticateUserOutput, AuthenticationService, LoginUser, LoginUserMfa,
    LoginUserOutput, MultiFactorAuthenticateUser,
};
use crate::services::mfa::MultiFactorConfigure;
use crate::services::session::{
    CreateSession, GetOtherSessions, GetOtherSessionsOutput, InvalidateOtherSessions,
    RenewSession,
};
use crate::services::user::GetUser;
use crate::services::Error;

pub async fn auth_login(
    state: ServerState,
    params: Params<'static>,
) -> Result<LoginUserOutput> {
    let txn = state.database.begin().await?;
    let ctx = ServiceContext::from_raw(&state, &txn);

    let LoginUser {
        authenticate,
        ip_address,
        user_agent,
    } = params.parse()?;

    // Don't allow empty passwords.
    //
    // They are never valid, and are potentially indicative of the user
    // entering the password in the name field instead, which we do
    // *not* want to be logging.
    if authenticate.password.is_empty() {
        tide::log::error!("User submitted empty password in auth request");
        return Err(Error::EmptyPassword);
    }

    // All authentication issue should return the same error.
    //
    // If anything went wrong, only allow a generic backend failure
    // to avoid leaking internal state. However since we are an internal
    // API
    //
    // The only three possible responses to this method should be:
    // * success
    // * invalid authentication
    // * server error
    let result = AuthenticationService::auth_password(&ctx, authenticate).await;
    let AuthenticateUserOutput { needs_mfa, user_id } = match result {
        Ok(output) => output,
        Err(mut error) => {
            if matches!(error, Error::InvalidAuthentication) {
                tide::log::error!("Unexpected error during user authentication: {error}");
                error = Error::InternalServerError;
            }

            return Err(error);
        }
    };

    let login_complete = !needs_mfa;
    tide::log::info!(
        "Password authentication for user ID {user_id} succeeded (login complete: {login_complete})",
    );

    let session_token = SessionService::create(
        &ctx,
        CreateSession {
            user_id,
            ip_address,
            user_agent,
            restricted: !login_complete,
        },
    )
    .await?;

    txn.commit().await?;
    Ok(LoginUserOutput {
        session_token,
        needs_mfa,
    })
}

/// Gets the information associated with a particular session token.
///
/// This is how framerail determines the user ID this user is acting as,
/// among other information.
pub async fn auth_session_retrieve(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let session_token = req.body_string().await?;
    let session = SessionService::get(&ctx, &session_token).await?;

    let body = Body::from_json(&session)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    txn.commit().await?;
    Ok(response)
}

pub async fn auth_session_renew(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);
    let input: RenewSession = req.body_json().await?;

    let new_session_token = SessionService::renew(&ctx, input).await?;

    let body = Body::from_string(new_session_token);
    let response = Response::builder(StatusCode::Ok).body(body).into();
    txn.commit().await?;
    Ok(response)
}

pub async fn auth_session_retrieve_others(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let GetOtherSessions {
        user_id,
        session_token,
    } = req.body_json().await?;

    // Produce output struct, which extracts the current session and
    // places it in its own location.
    let output = {
        let mut sessions = SessionService::get_all(&ctx, user_id).await?;
        let current = match sessions
            .iter()
            .position(|session| session.session_token == session_token)
        {
            Some(index) => sessions.remove(index),
            None => {
                tide::log::error!("Cannot find own session token in list of all sessions, must be invalid");
                return Ok(Response::new(StatusCode::NotFound));
            }
        };

        GetOtherSessionsOutput {
            current,
            others: sessions,
        }
    };

    let body = Body::from_json(&output)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    txn.commit().await?;
    Ok(response)
}

pub async fn auth_session_invalidate_others(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);
    let InvalidateOtherSessions {
        session_token,
        user_id,
    } = req.body_json().await?;

    let invalidated =
        SessionService::invalidate_others(&ctx, &session_token, user_id).await?;

    let body = Body::from_json(&invalidated)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    txn.commit().await?;
    Ok(response)
}

pub async fn auth_logout(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let session_token = req.body_string().await?;
    SessionService::invalidate(&ctx, session_token).await?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn auth_mfa_verify(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let LoginUserMfa {
        session_token,
        totp_or_code,
        ip_address,
        user_agent,
    } = req.body_json().await?;

    tide::log::info!(
        "Verifying user's MFA for login (temporary session token {session_token})"
    );

    let user = AuthenticationService::auth_mfa(
        &ctx,
        MultiFactorAuthenticateUser {
            session_token: &session_token,
            totp_or_code: &totp_or_code,
        },
    )
    .await?;

    let new_session_token = SessionService::renew(
        &ctx,
        RenewSession {
            old_session_token: session_token,
            user_id: user.user_id,
            ip_address,
            user_agent,
        },
    )
    .await?;

    let body = Body::from_string(new_session_token);
    let response = Response::builder(StatusCode::Ok).body(body).into();
    txn.commit().await?;
    Ok(response)
}

pub async fn auth_mfa_setup(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let GetUser { user: reference } = req.body_json().await?;
    let user = UserService::get(&ctx, reference).await?;
    let output = MfaService::setup(&ctx, &user).await?;

    let body = Body::from_json(&output)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    txn.commit().await?;
    Ok(response)
}

pub async fn auth_mfa_disable(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let MultiFactorConfigure {
        user_id,
        session_token,
    } = req.body_json().await?;

    let user = SessionService::get_user(&ctx, &session_token, false).await?;

    if user.user_id != user_id {
        tide::log::error!(
            "Passed user ID ({}) does not match session token ({})",
            user_id,
            user.user_id,
        );
        return Ok(Response::new(StatusCode::Forbidden));
    }

    MfaService::disable(&ctx, user.user_id).await?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn auth_mfa_reset_recovery(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let MultiFactorConfigure {
        user_id,
        session_token,
    } = req.body_json().await?;

    let user = SessionService::get_user(&ctx, &session_token, false).await?;

    if user.user_id != user_id {
        tide::log::error!(
            "Passed user ID ({}) does not match session token ({})",
            user_id,
            user.user_id,
        );
        return Ok(Response::new(StatusCode::Forbidden));
    }

    let output = MfaService::reset_recovery_codes(&ctx, &user).await?;

    let body = Body::from_json(&output)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    txn.commit().await?;
    Ok(response)
}
