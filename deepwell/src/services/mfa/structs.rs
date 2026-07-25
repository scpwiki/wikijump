/*
 * services/mfa/structs.rs
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
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::PasswordService;
use crate::utils::assert_is_csprng;
use data_encoding::BASE32_NOPAD;
use rand::RngExt;
use rand::distr::{Alphanumeric, SampleString};
use std::iter;
use std::net::IpAddr;

pub fn generate_totp_secret() -> String {
    let mut rng = rand::rng();
    assert_is_csprng(&rng);

    // TOTP secret is any sufficiently-long random base32 string
    {
        let mut buffer = [0; 32];
        rng.fill(&mut buffer);
        BASE32_NOPAD.encode(&buffer)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct RecoveryCodes {
    pub recovery_codes: Vec<String>,
    pub recovery_codes_hashed: Vec<String>,
}

impl RecoveryCodes {
    pub fn generate(config: &Config) -> Result<Self> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to generate new MFA recovery codes ({} codes each {} bytes long each)",
                    config.recovery_code_count, config.recovery_code_length,
                ),
                ErrorType::UserMfa,
            )
        };

        let mut rng = rand::rng();
        assert_is_csprng(&rng);

        // Recovery codes are any randomly-generated codes which the application
        // accepts as a one-time code to bypass MFA.
        let recovery_codes = iter::repeat_n((), config.recovery_code_count)
            .map(|_| {
                let mut code =
                    Alphanumeric.sample_string(&mut rng, config.recovery_code_length);

                code.insert(config.recovery_code_length / 2, '-'); // for readability
                code
            })
            .collect::<Vec<String>>();

        // Since we only need to check if the recovery code is *correct*, not what it is,
        // we can hash them just like passwords.
        // We use argon2, the same as recommended for passwords.
        let recovery_codes_hashed = {
            let mut hashes = Vec::new();

            for code in &recovery_codes {
                debug!("Hashing recovery code");
                let hash = PasswordService::new_hash(code).or_raise(make_error)?;
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

#[derive(Deserialize, Debug, Clone)]
pub struct MultiFactorConfigure {
    pub user_id: i64,
    pub session_token: String,
    pub ip_address: IpAddr,
}

#[derive(Serialize, Debug, Clone)]
pub struct MultiFactorSetupOutput {
    pub totp_secret: String,
    pub recovery_codes: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct MultiFactorResetOutput {
    pub recovery_codes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    #[test]
    fn totp_secret_is_base32_encoded_random_bytes() {
        let secret = generate_totp_secret();
        let decoded = BASE32_NOPAD.decode(secret.as_bytes()).unwrap();

        assert_eq!(decoded.len(), 32);
        assert!(
            secret
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        );
    }

    #[test]
    fn recovery_codes_are_returned_with_matching_argon_hashes() {
        let config = Config::integration_testing();
        let codes = RecoveryCodes::generate(&config).unwrap();

        assert_eq!(codes.recovery_codes.len(), config.recovery_code_count);
        assert_eq!(
            codes.recovery_codes_hashed.len(),
            config.recovery_code_count
        );

        for (code, hash) in codes
            .recovery_codes
            .iter()
            .zip(codes.recovery_codes_hashed.iter())
        {
            assert_eq!(code.len(), config.recovery_code_length + 1);
            assert_eq!(code.as_bytes()[config.recovery_code_length / 2], b'-');

            let parsed_hash = PasswordHash::new(hash).unwrap();
            Argon2::default()
                .verify_password(code.as_bytes(), &parsed_hash)
                .unwrap();
        }
    }
}
