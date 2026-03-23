/*
 * endpoints/auth.rs
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
use crate::models::session::Model as SessionModel;
use crate::services::authentication::{
    AuthenticateUserOutput, AuthenticationService, LoginUser, LoginUserMfa,
    LoginUserOutput, MultiFactorAuthenticateUser,
};
use crate::services::authorization_token::AuthorizationTokenService;
use crate::services::mfa::{
    MultiFactorConfigure, MultiFactorResetOutput, MultiFactorSetupOutput,
};
use crate::services::session::{
    CreateSession, GetOtherSessions, GetOtherSessionsOutput, InvalidateOtherSessions,
    RenewSession,
};

pub async fn auth_login(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<LoginUserOutput> {
    let LoginUser {
        authenticate,
        ip_address,
        user_agent,
    } = parse!(params, Login);

    // Don't allow empty passwords.
    //
    // They are never valid, and are potentially indicative of the user
    // entering the password in the name field instead, which we do
    // *not* want to be logging.
    if authenticate.password.is_empty() {
        error!("User submitted empty password in auth request");
        bail!(Error::new(
            "password cannot be empty",
            ErrorType::EmptyPassword,
        ));
    }

    let make_error = || Error::new("failed to perform login", ErrorType::Login);

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
    let result = AuthenticationService::auth_password(ctx, authenticate).await;
    let AuthenticateUserOutput { needs_mfa, user_id } = match result {
        Ok(output) => output,
        Err(error) => match error.error_type {
            ErrorType::InvalidAuthentication => bail!(error),
            _ => bail!(error.raise(make_error())),
        },
    };

    let login_complete = !needs_mfa;
    info!(
        "Password authentication for user ID {user_id} succeeded (login complete: {login_complete})",
    );

    let session_token = SessionService::create(
        ctx,
        CreateSession {
            user_id,
            ip_address,
            user_agent,
            restricted: !login_complete,
        },
    )
    .await
    .or_raise(make_error)?;

    Ok(LoginUserOutput {
        session_token,
        needs_mfa,
    })
}

pub async fn auth_logout(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let session_token: String = parse_one!(params, Logout);

    SessionService::invalidate(ctx, session_token)
        .await
        .or_raise(|| Error::new("failed to perform logout", ErrorType::Logout))
}

/// Gets the information associated with a particular session token.
///
/// This is how framerail determines the user ID this user is acting as,
/// among other information.
pub async fn auth_session_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<SessionModel>> {
    let session_token: String = parse_one!(params);

    SessionService::get_optional(ctx, &session_token)
        .await
        .or_raise(|| Error::new("failed to get session data", ErrorType::Request))
}

pub async fn auth_session_renew(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<String> {
    let input: RenewSession = parse_one!(params);

    SessionService::renew(ctx, input)
        .await
        .or_raise(|| Error::new("failed to renew session data", ErrorType::Request))
}

pub async fn auth_session_get_others(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<GetOtherSessionsOutput> {
    let GetOtherSessions {
        user_id,
        session_token,
    } = parse!(params);

    let make_error = || Error::new("failed to get other sessions", ErrorType::Request);

    // Produce output struct, which extracts the current session and
    // places it in its own location.
    let mut sessions = SessionService::get_all(ctx, user_id)
        .await
        .or_raise(make_error)?;

    let current = match sessions
        .iter()
        .position(|session| session.session_token == session_token)
    {
        Some(index) => sessions.remove(index),
        None => {
            error!(
                "Cannot find own session token in list of all sessions, must be invalid",
            );
            bail!(Error::new(
                "failed to get session token, must be invalid",
                ErrorType::InvalidSessionToken,
            ));
        }
    };

    Ok(GetOtherSessionsOutput {
        current,
        others: sessions,
    })
}

pub async fn auth_session_invalidate_others(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<u64> {
    let InvalidateOtherSessions {
        session_token,
        user_id,
    } = parse!(params);

    SessionService::invalidate_others(ctx, &session_token, user_id)
        .await
        .or_raise(|| {
            Error::new("failed to invalidate other sessions", ErrorType::Request)
        })
}

pub async fn auth_mfa_verify(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<String> {
    let LoginUserMfa {
        session_token,
        totp_or_code,
        ip_address,
        user_agent,
    } = parse!(params, InvalidAuthentication);

    let make_error = || {
        Error::new(
            "failed to verify user MFA",
            ErrorType::InvalidAuthentication,
        )
    };

    info!("Verifying user's MFA for login (temporary session token {session_token})");

    let user = AuthenticationService::auth_mfa(
        ctx,
        MultiFactorAuthenticateUser {
            session_token: &session_token,
            totp_or_code: &totp_or_code,
        },
    )
    .await
    .or_raise(make_error)?;

    SessionService::renew(
        ctx,
        RenewSession {
            old_session_token: session_token,
            user_id: user.user_id,
            ip_address,
            user_agent,
        },
    )
    .await
    .or_raise(make_error)
}

pub async fn auth_mfa_setup(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<MultiFactorSetupOutput> {
    let MultiFactorConfigure {
        user_id,
        session_token,
        ip_address,
    } = parse!(params);

    let make_error = || Error::new("failed to set up MFA", ErrorType::Request);

    let user = SessionService::get_user_with_id(ctx, &session_token, false, user_id)
        .await
        .or_raise(make_error)?;

    MfaService::setup(ctx, &user, ip_address)
        .await
        .or_raise(make_error)
}

pub async fn auth_mfa_disable(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let MultiFactorConfigure {
        user_id,
        session_token,
        ip_address,
    } = parse!(params);

    let make_error = || Error::new("failed to disable MFA", ErrorType::Request);

    let user = SessionService::get_user_with_id(ctx, &session_token, false, user_id)
        .await
        .or_raise(make_error)?;

    MfaService::disable(ctx, user.user_id, ip_address)
        .await
        .or_raise(make_error)
}

pub async fn auth_mfa_reset_recovery(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<MultiFactorResetOutput> {
    let MultiFactorConfigure {
        user_id,
        session_token,
        ip_address,
    } = parse!(params);

    let make_error = || Error::new("failed to reset recovery codes", ErrorType::Request);

    let user = SessionService::get_user_with_id(ctx, &session_token, false, user_id)
        .await
        .or_raise(make_error)?;

    MfaService::reset_recovery_codes(ctx, &user, ip_address)
        .await
        .or_raise(make_error)
}

pub async fn auth_token_issue(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<String> {
    let input = parse!(params, AuthorizationToken);
    let make_error = || {
        Error::new(
            "failed to issue new authorization token",
            ErrorType::Request,
        )
    };

    AuthorizationTokenService::create(ctx, input)
        .await
        .or_raise(make_error)
}
