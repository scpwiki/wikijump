/*
 * services/authentication/service.rs
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
use crate::models::user::{self, Entity as User, Model as UserModel};
use crate::services::{MfaService, PasswordService, SessionService};

#[derive(Debug)]
pub struct AuthenticationService;

impl AuthenticationService {
    /// Verifies the passed credentials for a user.
    /// If so, they are cleared to log in (or perform some other sensitive action).
    pub async fn auth_password(
        ctx: &ServiceContext,
        AuthenticateUser {
            name_or_email,
            password,
        }: AuthenticateUser,
    ) -> Result<AuthenticateUserOutput> {
        let auth = Self::get_user_auth(ctx, &name_or_email).await?;
        PasswordService::verify(ctx, &password, &auth.password_hash).await?;

        // User not found, return authentication failure
        if !auth.valid {
            return Err(Error::InvalidAuthentication);
        }

        Ok(AuthenticateUserOutput {
            needs_mfa: auth.multi_factor_secret.is_some(),
            user_id: auth.user_id,
        })
    }

    /// Verifies the TOTP code for a user, after they have logged in.
    ///
    /// # Returns
    /// The user model for the authenticated session.
    pub async fn auth_mfa(
        ctx: &ServiceContext,
        MultiFactorAuthenticateUser {
            session_token,
            totp_or_code,
        }: MultiFactorAuthenticateUser<'_>,
    ) -> Result<UserModel> {
        // Get associated user model from the session
        //
        // Requires the session is restricted, meaning they are
        // in the middle of logging in still
        let user = SessionService::get_user(ctx, session_token, true).await?;

        // Process input, verifying depending on type
        match totp_or_code.parse() {
            // If the value is a positive integer, treat it as a TOTP
            Ok(totp) => MfaService::verify(ctx, &user, totp).await?,

            // Otherwise treat it as a recovery code string
            //
            // We don't need to validate it for length because
            // we want consistent time checks on recovery codes anyways.
            Err(_) => MfaService::verify_recovery(ctx, &user, totp_or_code).await?,
        }

        Ok(user)
    }

    /// Gets user information from the database, or return a dummy.
    ///
    /// To avoid timing attacks, all aspects of authentication (finding the user,
    /// verifying their password, etc.) should take approximately the same amount
    /// of time.
    ///
    /// As such, if the user requested does not actually exist, we should pull a
    /// fake dummy user, perform redundant authentication checks against them before
    /// finally returning failure.
    ///
    /// Similarly, the only error that should be returned is a generic authentication error.
    async fn get_user_auth(
        ctx: &ServiceContext,
        name_or_email: &str,
    ) -> Result<UserAuthInfo> {
        info!("Looking for user matching name or email '{name_or_email}'");

        #[derive(Debug)]
        struct Row {
            user_id: i64,
            password: String,
            multi_factor_secret: Option<String>,
        }

        let mutex = ctx.sqlx_transaction();
        let mut txn = mutex.lock().await;

        let result = sqlx::query_as!(
            Row,
            r#"
            SELECT user_id, password, multi_factor_secret
            FROM "user"
            WHERE name = $1
            OR    slug = $1
            OR    email = $1
            "#,
            name_or_email
        )
        .fetch_optional(&mut **txn)
        .await?;

        match result {
            // Found user, return real auth information
            Some(Row {
                user_id,
                password,
                multi_factor_secret,
            }) => Ok(UserAuthInfo {
                user_id,
                password,
                multi_factor_secret,
                valid: true,
            }),

            // Didn't find user, return fake auth information
            // Checking should proceed as normal to avoid timing attacks
            None => Ok(UserAuthInfo::invalid()),
        }
    }
}
