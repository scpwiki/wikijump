/*
 * services/authentication/structs.rs
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

use crate::models::user::Model as UserModel;
use std::net::IpAddr;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateUser {
    pub name_or_email: String,
    pub password: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateUserOutput {
    pub needs_mfa: bool,
    pub user_id: i64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginUser {
    pub ip_address: IpAddr,
    pub user_agent: String,

    #[serde(flatten)]
    pub authenticate: AuthenticateUser,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginUserOutput {
    pub session_token: String,
    pub needs_mfa: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MultiFactorAuthenticateUser {
    pub session_token: String,
    pub totp_or_code: String,
}

/// Password hash to compute against when a user does not exist.
///
/// It has similar settings to other passwords on Wikijump, but
/// after hashing the result is ignored (see `valid`).
///
/// This is *not* a secret, but the value isn't too important
/// anyways. The password is simply a long randomly-generated value.
pub const INVALID_PASSWORD_HASH: &str =
    "$argon2id$v=19$m=4096,t=3,p=1$UjcwSVNZd1hzUWdkc0s2bg$kxdfVniblhviREHGGy81/A";

#[derive(Debug, Clone)]
pub struct UserAuthInfo {
    pub user_id: i64,
    pub password_hash: String,
    pub multi_factor_secret: Option<String>,
    pub valid: bool,
}

impl UserAuthInfo {
    pub fn valid(user: UserModel) -> Self {
        UserAuthInfo {
            user_id: user.user_id,
            password_hash: user.password,
            multi_factor_secret: user.multi_factor_secret,
            valid: true,
        }
    }

    #[inline]
    pub fn invalid() -> Self {
        UserAuthInfo {
            user_id: 0,
            password_hash: str!(INVALID_PASSWORD_HASH),
            multi_factor_secret: None,
            valid: false,
        }
    }
}
