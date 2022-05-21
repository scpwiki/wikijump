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

use crate::utils::assert_is_csprng;
use data_encoding::BASE32_NOPAD;
use rand::distributions::{Alphanumeric, DistString};
use rand::{thread_rng, Rng};
use std::iter;

pub const RECOVERY_CODE_COUNT: usize = 12;
pub const RECOVERY_CODE_LENGTH: usize = 8;

#[derive(Debug)]
pub struct MfaSecrets {
    pub totp_secret: String,
    pub recovery_codes: Vec<String>,
}

impl MfaSecrets {
    pub fn generate() -> Self {
        let mut rng = thread_rng();
        assert_is_csprng(&rng);

        let totp_secret = {
            let mut buffer = [0; 32];
            rng.fill(&mut buffer);
            BASE32_NOPAD.encode(&buffer)
        };

        let recovery_codes = iter::repeat(())
            .take(RECOVERY_CODE_COUNT)
            .map(|_| {
                let mut code = Alphanumeric.sample_string(&mut rng, RECOVERY_CODE_LENGTH);
                code.insert(RECOVERY_CODE_LENGTH / 2, '-'); // for readability
                code
            })
            .collect();

        // TODO convert recovery codes to passwords since we only need to check if they're the
        //      same, not the value itself

        MfaSecrets {
            totp_secret,
            recovery_codes,
        }
    }
}
