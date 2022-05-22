/*
 * services/mfa/structs.rs
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
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use data_encoding::BASE32_NOPAD;
use rand::distributions::{Alphanumeric, DistString};
use rand::{thread_rng, Rng};
use std::iter;

pub const RECOVERY_CODE_COUNT: usize = 12;
pub const RECOVERY_CODE_LENGTH: usize = 8;

pub fn generate_totp_secret() -> String {
    let mut rng = thread_rng();
    assert_is_csprng(&rng);

    // TOTP secret is any sufficiently-long random base32 string
    let totp_secret = {
        let mut buffer = [0; 32];
        rng.fill(&mut buffer);
        BASE32_NOPAD.encode(&buffer)
    };

    totp_secret
}

#[derive(Debug)]
pub struct RecoveryCodes {
    pub recovery_codes: Vec<String>,
    pub recovery_codes_hashed: Vec<String>,
}

impl RecoveryCodes {
    pub fn generate() -> Result<Self> {
        let mut rng = thread_rng();
        assert_is_csprng(&rng);

        // Recovery codes are any randomly-generated codes which the application
        // accepts as a one-time code to bypass MFA.
        let recovery_codes = iter::repeat(())
            .take(RECOVERY_CODE_COUNT)
            .map(|_| {
                let mut code = Alphanumeric.sample_string(&mut rng, RECOVERY_CODE_LENGTH);
                code.insert(RECOVERY_CODE_LENGTH / 2, '-'); // for readability
                code
            })
            .collect::<Vec<String>>();

        // Since we only need to check if the recovery code is *correct*, not what it is,
        // we can hash them just like passwords.
        // We use argon2, the same as recommended for passwords.
        let recovery_codes_hashed = {
            let mut hashes = Vec::new();
            let argon2 = Argon2::default();

            for code in &recovery_codes {
                let salt = SaltString::generate(&mut rng);
                let hash = argon2.hash_password(code.as_bytes(), &salt)?.to_string();
                hashes.push(hash);
            }

            hashes
        };

        Ok(RecoveryCodes {
            recovery_codes,
            recovery_codes_hashed,
        })
    }
}
