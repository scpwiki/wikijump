/*
 * services/password.rs
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

use crate::config::Config;
use crate::error::prelude::{Error, ErrorType, Result};
use crate::services::ServiceContext;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use tokio::time;

#[derive(Debug)]
pub struct PasswordService;

impl PasswordService {
    /// Produces a new password hash from the input string.
    ///
    /// Generates a salt securely and performs Argon-2 hashing
    /// and yields a string in PHC format.
    pub fn new_hash(password: &str) -> Result<String> {
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes())
            .map_err(convert_argon_error)?
            .to_string();

        Ok(hash)
    }

    /// Verifies that the inputted password matches the provided password hash.
    ///
    /// The password hash is expected to be in PHC format.
    ///
    /// # Returns
    /// Nothing on success, yields an `InvalidAuthentication` error on failure.
    /// Will sleep a bit on failure.
    pub async fn verify(
        ctx: &ServiceContext<'_>,
        password: &str,
        hash: &str,
    ) -> Result<()> {
        Self::verify_sleep(ctx, password, hash, true).await
    }

    /// Like `verify()`, but allows specifying whether sleeping should take place.
    ///
    /// Should only be used internally, when the sleeping is performed by the caller
    /// themselves on failure.
    pub async fn verify_sleep(
        ctx: &ServiceContext<'_>,
        password: &str,
        hash: &str,
        sleep: bool,
    ) -> Result<()> {
        info!("Attempting to verify password");
        let result = Self::verify_internal(password, hash);
        match result {
            Ok(()) => Ok(()),
            Err(error) => {
                match error.error_type {
                    // Simply the wrong password
                    // This is converted in services/error.rs
                    ErrorType::InvalidAuthentication => {
                        warn!("Invalid password entered, verification failed\n{error}");
                    }

                    // Some other kind of server error
                    _ => {
                        error!("Unexpected error while verifying password:\n{error}");
                    }
                }

                // Delay a bit on failure to prevent brute-force attacks.
                if sleep {
                    Self::failure_sleep(ctx.config()).await;
                }

                // Always return the same error for authentication methods,
                // to not expose internal state to an adversary.
                bail!(Error::new(
                    "failed to verify password",
                    ErrorType::InvalidAuthentication,
                ))
            }
        }
    }

    fn verify_internal(password: &str, hash: &str) -> Result<()> {
        let argon2 = Argon2::default();
        argon2
            .verify_password(password.as_bytes(), hash)
            .map_err(convert_argon_error)?;

        Ok(())
    }

    /// Sleeps for a bit after authentication failure.
    pub async fn failure_sleep(config: &Config) {
        time::sleep(config.authentication_fail_delay).await;
    }
}

fn convert_argon_error(error: argon2::password_hash::Error) -> Error {
    Error::new(
        "failed to hash password",
        ErrorType::Cryptography(str!(error)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn password_hashes_verify_original_password_and_reject_wrong_password() {
        let password = Uuid::new_v4().to_string();
        let wrong_password = Uuid::new_v4().to_string();
        let hash = PasswordService::new_hash(&password).unwrap();

        PasswordService::verify_internal(&password, &hash).unwrap();
        assert!(PasswordService::verify_internal(&wrong_password, &hash).is_err());
        assert!(PasswordService::verify_internal(&password, "not phc").is_err());
    }

    #[tokio::test]
    async fn failure_sleep_uses_configured_delay() {
        let mut config = Config::integration_testing();
        config.authentication_fail_delay = std::time::Duration::from_millis(0);

        PasswordService::failure_sleep(&config).await;
    }
}
