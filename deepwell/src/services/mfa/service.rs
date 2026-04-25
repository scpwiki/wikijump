/*
 * services/mfa/service.rs
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
use crate::{
    models::user::Model as UserModel,
    services::{
        PasswordService, UserService,
        audit::{AuditEvent, AuditService, UpdateMfaOperation},
    },
    types::UserType,
};
use sea_orm::ActiveValue;
use std::net::IpAddr;
use subtle::ConstantTimeEq;

#[derive(Debug)]
pub struct MfaService;

impl MfaService {
    /// Initializes MFA for a user.
    ///
    /// Fails if MFA is already configured.
    pub async fn setup(
        ctx: &ServiceContext<'_>,
        user: &UserModel,
        ip_address: IpAddr,
    ) -> Result<MultiFactorSetupOutput> {
        info!("Setting up MFA for user ID {}", user.user_id);

        let make_error = || {
            Error::new(
                format!(
                    "failed to set up MFA for user '{}' (ID {})",
                    user.slug, user.user_id,
                ),
                ErrorType::UserMfa,
            )
        };

        // Only regular accounts can have MFA
        if user.user_type != UserType::Regular {
            error!("Only regular users may have MFA");
            bail!(Error::new(
                format!(
                    "cannot setup MFA for user '{}' (ID {}), only permitted for regular users",
                    user.slug, user.user_id
                ),
                ErrorType::BadRequest,
            ));
        }

        // Ensure MFA is not yet set up
        if user.multi_factor_secret.is_some()
            || user.multi_factor_recovery_codes.is_some()
        {
            error!("User already has MFA set up");
            bail!(Error::new(
                format!(
                    "cannot setup MFA for user '{}' (ID {}) because it is already set up",
                    user.slug, user.user_id
                ),
                ErrorType::UserMfaExists,
            ));
        }

        // Securely generate and store secrets
        debug!("Generating MFA secrets for user ID {}", user.user_id);
        let totp_secret = generate_totp_secret();
        let recovery = RecoveryCodes::generate(ctx.config()).or_raise(make_error)?;

        debug!("Committing MFA secrets for user ID {}", user.user_id);
        UserService::set_mfa_secrets(
            ctx,
            user.user_id,
            ActiveValue::Set(Some(totp_secret.clone())),
            ActiveValue::Set(Some(recovery.recovery_codes_hashed)),
        )
        .await
        .or_raise(make_error)?;

        // Audit log
        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::UserUpdateMfa {
                user_id: user.user_id,
                operation: UpdateMfaOperation::Setup,
            },
        )
        .await
        .or_raise(make_error)?;

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
        ip_address: IpAddr,
    ) -> Result<MultiFactorResetOutput> {
        info!("Resetting MFA recovery codes for user ID {}", user.user_id);

        let make_error = || {
            Error::new(
                format!(
                    "failed to reset recovery codes MFA for user '{}' (ID {})",
                    user.slug, user.user_id,
                ),
                ErrorType::UserMfa,
            )
        };

        // Ensure MFA is set up
        if user.multi_factor_secret.is_none()
            || user.multi_factor_recovery_codes.is_none()
        {
            error!("User does not have MFA set up");
            bail!(Error::new(
                format!(
                    "cannot reset MFA recovery codes for user '{}' (ID {}) because they do not have MFA set up",
                    user.slug, user.user_id
                ),
                ErrorType::BadRequest,
            ));
        }

        // Securely generate and store secrets
        debug!("Generating recovery codes for user ID {}", user.user_id);
        let recovery = RecoveryCodes::generate(ctx.config()).or_raise(make_error)?;

        debug!("Committing recovery codes for user ID {}", user.user_id);
        UserService::set_mfa_secrets(
            ctx,
            user.user_id,
            ActiveValue::NotSet,
            ActiveValue::Set(Some(recovery.recovery_codes_hashed)),
        )
        .await
        .or_raise(make_error)?;

        // Audit log
        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::UserUpdateMfa {
                user_id: user.user_id,
                operation: UpdateMfaOperation::ResetRecoveryCodes,
            },
        )
        .await
        .or_raise(make_error)?;

        // Return to user for their storage
        Ok(MultiFactorResetOutput {
            recovery_codes: recovery.recovery_codes,
        })
    }

    /// Disables MFA for a user.
    ///
    /// After this is run, the user does not need MFA to sign in,
    /// and has no recovery codes or TOTP secret.
    pub async fn disable(
        ctx: &ServiceContext<'_>,
        user_id: i64,
        ip_address: IpAddr,
    ) -> Result<()> {
        info!("Tearing down MFA for user ID {user_id}");

        let make_error = || {
            Error::new(
                format!("failed to disable MFA for user ID {}", user_id,),
                ErrorType::UserMfa,
            )
        };

        UserService::set_mfa_secrets(
            ctx,
            user_id,
            ActiveValue::Set(None),
            ActiveValue::Set(None),
        )
        .await
        .or_raise(make_error)?;

        // Audit log
        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::UserUpdateMfa {
                user_id,
                operation: UpdateMfaOperation::Disable,
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(())
    }

    /// Verifies if the TOTP passed for this user is valid.
    ///
    /// # Returns
    /// Nothing on success, yields an `InvalidAuthentication` error on failure.
    pub async fn verify(
        ctx: &ServiceContext<'_>,
        user: &UserModel,
        entered_totp: u32,
    ) -> Result<()> {
        info!("Verifying TOTP code for user ID {}", user.user_id);

        let make_error = || {
            Error::new(
                format!(
                    "failed to verify MFA for user '{}' (ID {})",
                    user.slug, user.user_id,
                ),
                ErrorType::UserMfa,
            )
        };

        let secret = match &user.multi_factor_secret {
            Some(secret) => secret,
            None => {
                error!("User has no MFA secret, cannot verify TOTP");
                bail!(Error::new(
                    format!(
                        "cannot verify MFA for user '{}' (ID {}) because it is not set up",
                        user.slug, user.user_id,
                    ),
                    ErrorType::InvalidAuthentication,
                ));
            }
        };

        let actual_totp = rust_otp::make_totp(
            secret,
            ctx.config().totp_time_step,
            ctx.config().totp_time_skew,
        )
        .or_raise(make_error)?;

        // Constant-time comparison
        if actual_totp.ct_eq(&entered_totp).into() {
            return Ok(());
        }

        bail!(Error::new(
            format!(
                "cannot verify MFA for user '{}' (ID {})",
                user.slug, user.user_id,
            ),
            ErrorType::InvalidAuthentication,
        ));
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
        info!("Verifying recovery code for user ID {}", user.user_id);

        let recovery_code_hashes = match &user.multi_factor_recovery_codes {
            Some(codes) => codes,
            None => {
                error!("User has no MFA recovery codes, but wants to verify recovery");
                bail!(Error::new(
                    format!(
                        "cannot verify MFA recovery code for user '{}' (ID {}) because there are no codes set up",
                        user.slug, user.user_id,
                    ),
                    ErrorType::InvalidAuthentication,
                ));
            }
        };

        // Constant-time, check all the recovery codes even when we know we have a match.
        let mut matched = None;
        for recovery_code_hash in recovery_code_hashes {
            if PasswordService::verify_sleep(
                ctx,
                recovery_code,
                recovery_code_hash,
                false,
            )
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
                PasswordService::failure_sleep(ctx.config()).await;
                bail!(Error::new(
                    format!(
                        "cannot verify MFA recovery code for user '{}' (ID {})",
                        user.slug, user.user_id,
                    ),
                    ErrorType::InvalidAuthentication,
                ));
            }
        }
    }
}
