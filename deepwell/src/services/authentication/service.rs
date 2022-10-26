/*
 * services/authentication/service.rs
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
use crate::models::user::Model as UserModel;
use crate::services::{MfaService, PasswordService};

#[derive(Debug)]
pub struct AuthenticationService;

impl AuthenticationService {
    /// Verifies the passed credentials for the user to determine if they are valid.
    /// If so, the user is cleared to log in.
    pub async fn verify_user(
        ctx: &ServiceContext<'_>,
        user: &UserModel,
        AuthenticateUser { password, totp }: AuthenticateUser,
    ) -> Result<()> {
        tide::log::info!("Attempting to authenticate user ID {}", user.user_id);

        // Verify password
        PasswordService::verify(&password, &user.password).await?;

        // Verify MFA
        match (totp, user.multi_factor_secret.is_some()) {
            // Provided TOTP, validate
            (Some(value), true) => {
                match value.parse() {
                    // If the value is a positive integer, treat it as a TOTP
                    Ok(totp) => MfaService::verify(ctx, user, totp).await?,

                    // Otherwise treat it as a recovery code string
                    //
                    // We don't need to validate it for length because
                    // we want consistent time checks on recovery codes anyways.
                    Err(_) => MfaService::verify_recovery(ctx, user, &value).await?,
                }
            }

            // MFA is required but not provided
            (Some(_), false) => {
                tide::log::warn!("User requires MFA but TOTP was not provided");
                return Err(Error::InvalidAuthentication);
            }

            // MFA is disabled, don't check
            (None, false) => (),

            // MFA is disabled but provided anyways
            (None, true) => {
                tide::log::warn!("User doesn't require MFA but TOTP was provided");
                return Err(Error::InvalidAuthentication);
            }
        }

        Ok(())
    }
}
