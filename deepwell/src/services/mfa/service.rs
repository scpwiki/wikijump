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
use crate::models::user::Model as UserModel;
use crate::services::{PasswordService, UserService};
use sea_orm::ActiveValue;
use subtle::ConstantTimeEq;

/// The amount of time to give for each TOTP.
///
/// We use 30 seconds because this is standard with helpers
/// such as Google Authenticator and Authy.
///
/// It balances between giving the user enough time to enter a code,
/// but short enough to make bruteforcing values impractical.
const TIME_STEP: u64 = 30;

/// The allowed leniency value to account for clock skew.
///
/// This represents the seconds that a TOTP is offset by in
/// determining whether the authentication was accepted.
///
/// See https://github.com/TimDumol/rust-otp/blob/master/src/lib.rs#L56
const TIME_SKEW: i64 = 1;

#[derive(Debug)]
pub struct MfaService;

impl MfaService {
    /// Initializes MFA for a user.
    ///
    /// Fails if MFA is already configured.
    pub async fn setup(
        ctx: &ServiceContext<'_>,
        user: &UserModel,
    ) -> Result<MultiFactorSetupOutput> {
        tide::log::info!("Setting up MFA for user ID {}", user.user_id);

        // Ensure MFA is not yet set up
        if user.multi_factor_secret.is_some()
            || user.multi_factor_recovery_codes.is_some()
        {
            tide::log::error!("User already has MFA set up");
            return Err(Error::Conflict);
        }

        // Securely generate and store secrets
        let totp_secret = generate_totp_secret();
        let recovery = RecoveryCodes::generate()?;

        UserService::set_mfa_secrets(
            ctx,
            user.user_id,
            ActiveValue::Set(Some(totp_secret.clone())),
            ActiveValue::Set(Some(recovery.recovery_codes_hashed)),
        )
        .await?;

        // Return to user for their storage
        Ok(MultiFactorSetupOutput {
            totp_secret,
            recovery_codes: recovery.recovery_codes,
        })
    }

    /// Regenerates all / refills recovery codes for this user.
    ///
    /// All prior recovery codes are invalidated.
    pub async fn reset_recovery_codes(
        ctx: &ServiceContext<'_>,
        user: &UserModel,
    ) -> Result<MultiFactorResetOutput> {
        tide::log::info!("Resetting MFA recovery codes for user ID {}", user.user_id);

        // Ensure MFA is set up
        if user.multi_factor_secret.is_none()
            || user.multi_factor_recovery_codes.is_none()
        {
            tide::log::error!("User does not have MFA set up");
            return Err(Error::Conflict);
        }

        // Securely generate and store secrets
        let recovery = RecoveryCodes::generate()?;

        UserService::set_mfa_secrets(
            ctx,
            user.user_id,
            ActiveValue::NotSet,
            ActiveValue::Set(Some(recovery.recovery_codes_hashed)),
        )
        .await?;

        // Return to user for their storage
        Ok(MultiFactorResetOutput {
            recovery_codes: recovery.recovery_codes,
        })
    }

    /// Disables MFA for a user.
    ///
    /// After this is run, the user does not need MFA to sign in,
    /// and has no recovery codes or TOTP secret.
    pub async fn disable(ctx: &ServiceContext<'_>, user_id: i64) -> Result<()> {
        tide::log::info!("Tearing down MFA for user ID {}", user_id);

        UserService::set_mfa_secrets(
            ctx,
            user_id,
            ActiveValue::Set(None),
            ActiveValue::Set(None),
        )
        .await
    }

    /// Verifies if the TOTP passed for this user is valid.
    ///
    /// # Returns
    /// Nothing on success, yields an `InvalidAuthentication` error on failure.
    pub async fn verify(user: &UserModel, entered_totp: u32) -> Result<()> {
        tide::log::info!("Verifying TOTP code for user ID {}", user.user_id);

        let secret = match &user.multi_factor_secret {
            Some(secret) => secret,
            None => {
                tide::log::warn!("User has no MFA secret, cannot verify TOTP");
                return Err(Error::InvalidAuthentication);
            }
        };

        let actual_totp = otp::make_totp(&secret, TIME_STEP, TIME_SKEW)?;

        // Constant-time comparison
        if actual_totp.ct_eq(&entered_totp).into() {
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
        user: &UserModel,
        recovery_code: &str,
    ) -> Result<()> {
        tide::log::info!("Verifying recovery code for user ID {}", user.user_id);

        let recovery_code_hashes = match &user.multi_factor_recovery_codes {
            Some(codes) => codes,
            None => {
                tide::log::warn!(
                    "User has no MFA recovery codes, but wants to verify recovery",
                );

                return Err(Error::InvalidAuthentication);
            }
        };

        // Constant-time, check all the recovery codes even when we know we have a match.
        let mut matched = None;
        for recovery_code_hash in recovery_code_hashes {
            if PasswordService::verify_sleep(recovery_code, &recovery_code_hash, false)
                .await
                .is_ok()
            {
                matched = Some(recovery_code_hash);
            }
        }

        match matched {
            // Remove the used recovery code from the list.
            Some(hash) => {
                UserService::remove_recovery_code(ctx, user, hash).await?;
                Ok(())
            }

            // We sleep ourselves, once at the end.
            //
            // Otherwise we have variable-time recovery code checks based on whether
            // the recovery code was correct or not.
            None => {
                PasswordService::failure_sleep().await;
                Err(Error::InvalidAuthentication)
            }
        }
    }
}
