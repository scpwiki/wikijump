/*
 * endpoints/auth.rs
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
use crate::models::session::Model as SessionModel;
use crate::services::authentication::{
    AuthenticateUserOutput, AuthenticationService, LoginUser, LoginUserMfa,
    LoginUserOutput, MultiFactorAuthenticateUser,
};
use crate::services::mfa::{
    MultiFactorConfigure, MultiFactorResetOutput, MultiFactorSetupOutput,
};
use crate::services::session::{
    CreateSession, GetOtherSessions, GetOtherSessionsOutput, InvalidateOtherSessions,
    RenewSession,
};
use crate::services::user::GetUser;
use crate::services::Error;

pub async fn auth_login(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<LoginUserOutput> {
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
        error!("User submitted empty password in auth request");
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
    let result = AuthenticationService::auth_password(ctx, authenticate).await;
    let AuthenticateUserOutput { needs_mfa, user_id } = match result {
        Ok(output) => output,
        Err(mut error) => {
            if !matches!(error, Error::InvalidAuthentication) {
                error!("Unexpected error during user authentication: {error}");
                error = Error::AuthenticationBackend(Box::new(error));
            }

            return Err(error);
        }
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
    .await?;

    Ok(LoginUserOutput {
        session_token,
        needs_mfa,
    })
}

pub async fn auth_logout(ctx: &ServiceContext, params: Params<'static>) -> Result<()> {
    let session_token: String = params.one()?;
    SessionService::invalidate(ctx, session_token).await
}

/// Gets the information associated with a particular session token.
///
/// This is how framerail determines the user ID this user is acting as,
/// among other information.
pub async fn auth_session_get(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<Option<SessionModel>> {
    let session_token: String = params.one()?;
    SessionService::get_optional(ctx, &session_token).await
}

pub async fn auth_session_renew(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<String> {
    let input: RenewSession = params.parse()?;
    SessionService::renew(ctx, input).await
}

pub async fn auth_session_get_others(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<GetOtherSessionsOutput> {
    let GetOtherSessions {
        user_id,
        session_token,
    } = params.parse()?;

    // Produce output struct, which extracts the current session and
    // places it in its own location.
    let mut sessions = SessionService::get_all(ctx, user_id).await?;
    let current = match sessions
        .iter()
        .position(|session| session.session_token == session_token)
    {
        Some(index) => sessions.remove(index),
        None => {
            error!(
                "Cannot find own session token in list of all sessions, must be invalid",
            );
            return Err(Error::InvalidSessionToken);
        }
    };

    Ok(GetOtherSessionsOutput {
        current,
        others: sessions,
    })
}

pub async fn auth_session_invalidate_others(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<u64> {
    let InvalidateOtherSessions {
        session_token,
        user_id,
    } = params.parse()?;

    SessionService::invalidate_others(ctx, &session_token, user_id).await
}

pub async fn auth_mfa_verify(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<String> {
    let LoginUserMfa {
        session_token,
        totp_or_code,
        ip_address,
        user_agent,
    } = params.parse()?;

    info!("Verifying user's MFA for login (temporary session token {session_token})",);

    let user = AuthenticationService::auth_mfa(
        ctx,
        MultiFactorAuthenticateUser {
            session_token: &session_token,
            totp_or_code: &totp_or_code,
        },
    )
    .await?;

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
}

pub async fn auth_mfa_setup(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<MultiFactorSetupOutput> {
    let GetUser { user: reference } = params.parse()?;
    let user = UserService::get(ctx, reference).await?;
    MfaService::setup(ctx, &user).await
}

pub async fn auth_mfa_disable(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<()> {
    let MultiFactorConfigure {
        user_id,
        session_token,
    } = params.parse()?;

    let user = SessionService::get_user(ctx, &session_token, false).await?;
    if user.user_id != user_id {
        error!(
            "Passed user ID ({}) does not match session token ({})",
            user_id, user.user_id,
        );

        return Err(Error::SessionUserId {
            active_user_id: user_id,
            session_user_id: user.user_id,
        });
    }

    MfaService::disable(ctx, user.user_id).await
}

pub async fn auth_mfa_reset_recovery(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<MultiFactorResetOutput> {
    let MultiFactorConfigure {
        user_id,
        session_token,
    } = params.parse()?;

    let user = SessionService::get_user(ctx, &session_token, false).await?;
    if user.user_id != user_id {
        error!(
            "Passed user ID ({}) does not match session token ({})",
            user_id, user.user_id,
        );

        return Err(Error::SessionUserId {
            active_user_id: user_id,
            session_user_id: user.user_id,
        });
    }

    MfaService::reset_recovery_codes(ctx, &user).await
}
