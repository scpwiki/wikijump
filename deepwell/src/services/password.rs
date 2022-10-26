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
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand::thread_rng;

#[derive(Debug)]
pub struct PasswordService;

impl PasswordService {
    pub fn new_hash(password: &str) -> Result<String> {
        let mut rng = thread_rng();
        assert_is_csprng(&rng);

        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut rng);
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        Ok(hash)
    }
}
