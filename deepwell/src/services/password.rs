/*
 * services/password.rs
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
use crate::utils::assert_is_csprng;
use argon2::{
    password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use async_std::task;
use rand::thread_rng;
use std::time::Duration;

/// Time to sleep in between failed authentication requests.
const AUTHENTICATION_FAILURE_DELAY: Duration = Duration::from_millis(100);

#[derive(Debug)]
pub struct PasswordService;

impl PasswordService {
    /// Produces a new password hash from the input string.
    ///
    /// Generates a salt securely and performs Argon-2 hashing
    /// and yields a string in PHC format.
    pub fn new_hash(password: &str) -> Result<String> {
        // Create and verify CSPRNG
        let mut rng = thread_rng();
        assert_is_csprng(&rng);

        // Create Argon-2 context, salt, and then hash the password
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut rng);
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        Ok(hash)
    }

    /// Verifies that the inputted password matches the provided password hash.
    ///
    /// The password hash is expected to be in PHC format.
    ///
    /// On failure, the method will sleep a bit to avoid brute-forcing.
    pub async fn verify(password: &str, hash: &str) -> Result<()> {
        tide::log::info!("Attempting to verify password");
        let result = Self::verify_internal(password, hash);

        match result {
            Ok(()) => Ok(()),
            Err(error) => {
                match error {
                    // Simply the wrong password
                    Error::InvalidAuthentication => {
                        tide::log::warn!("Invalid password entered, verification failed");
                    }

                    // Some kind of server error
                    _ => {
                        tide::log::error!(
                            "Unexpected error while verifying password: {error}",
                        );
                    }
                }

                // Delay a bit on failure to prevent brute-force attacks.
                task::sleep(AUTHENTICATION_FAILURE_DELAY).await;

                // Always return the same error for authentication methods,
                // to not expose internal state to an adversary.
                Err(Error::InvalidAuthentication)
            }
        }
    }

    fn verify_internal(password: &str, hash: &str) -> Result<()> {
        // Parse PHC string
        let hash = PasswordHash::new(hash)?;

        // Create Argon-2 context, then verify the password
        let argon2 = Argon2::default();
        argon2.verify_password(password.as_bytes(), &hash)?;
        Ok(())
    }
}
