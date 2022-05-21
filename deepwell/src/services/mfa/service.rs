/*
 * services/mfa/service.rs
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

/// The amount of time to give for each TOTP.
///
/// We use 30 seconds because this is standard with helpers
/// such as Google Authenticator and Authy.
///
/// It balances between giving the user enough time to enter a code,
/// but short enough to make bruteforcing values impractical.
const TIME_STEP: u64 = 30;

#[derive(Debug)]
pub struct MfaService;

impl MfaService {
    pub async fn setup(ctx: &ServiceContext<'_>) -> Result<()> {
        let secrets = MfaSecrets::generate();

        todo!()
    }

    /// Verifies if the TOTP passed for this user is valid.
    ///
    /// # Returns
    /// Nothing on success, yields an `InvalidAuthentication` error on failure.
    pub async fn verify(
        ctx: &ServiceContext<'_>,
        user_id: i64,
        entered_totp: u32,
    ) -> Result<()> {
        let secret: String = todo!(); // TODO fetch from database. if none, return InvalidAuthentication
        let skew = todo!();
        let actual_totp = otp::make_totp(&secret, TIME_STEP, skew)?;

        if actual_totp == entered_totp {
            Ok(())
        } else {
            Err(Error::InvalidAuthentication)
        }
    }

    /// Verifies if the recovery code for this user is valid.
    ///
    /// If it is, then the code is removed from the user's list
    /// of valid codes before returning success.
    ///
    /// # Returns
    /// Nothing on success, yields an `InvalidAuthentication` error on failure.
    pub async fn verify_recovery(
        ctx: &ServiceContext<'_>,
        user_id: i64,
        recovery_code: &str,
    ) -> Result<()> {
        todo!()
    }
}
