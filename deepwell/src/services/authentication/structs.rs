/*
 * services/authentication/structs.rs
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

use crate::models::user::Model as UserModel;
use std::net::IpAddr;

#[derive(Deserialize, Debug, Clone)]
pub struct AuthenticateUser {
    pub name_or_email: String,
    pub password: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct AuthenticateUserOutput {
    pub needs_mfa: bool,
    pub user_id: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LoginUser {
    pub ip_address: IpAddr,
    pub user_agent: String,

    #[serde(flatten)]
    pub authenticate: AuthenticateUser,
}

#[derive(Serialize, Debug, Clone)]
pub struct LoginUserOutput {
    pub session_token: String,
    pub needs_mfa: bool,
}

#[derive(Debug, Clone)]
pub struct MultiFactorAuthenticateUser<'a> {
    pub session_token: &'a str,
    pub totp_or_code: &'a str,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LoginUserMfa {
    pub session_token: String,
    pub totp_or_code: String,
    pub ip_address: IpAddr,
    pub user_agent: String,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::UserType;
    use time::OffsetDateTime;

    fn user_model() -> UserModel {
        UserModel {
            user_id: 123,
            user_type: UserType::Regular,
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: None,
            deleted_at: None,
            name: str!("Example User"),
            slug: str!("example-user"),
            name_changes_left: 2,
            last_name_change_added_at: OffsetDateTime::UNIX_EPOCH,
            last_renamed_at: None,
            email: str!("user@example.com"),
            email_verified_at: None,
            email_validation_info: None,
            email_validation_at: None,
            password: str!("stored-password-hash"),
            multi_factor_secret: Some(str!("mfa-secret")),
            multi_factor_recovery_codes: None,
            locales: vec![str!("en")],
            avatar_s3_hash: None,
            real_name: None,
            gender: None,
            birthday: None,
            location: None,
            biography: None,
            website: None,
            user_page: None,
        }
    }

    #[test]
    fn user_auth_info_keeps_valid_user_credentials() {
        let auth = UserAuthInfo::valid(user_model());

        assert_eq!(auth.user_id, 123);
        assert_eq!(auth.password_hash, "stored-password-hash");
        assert_eq!(auth.multi_factor_secret.as_deref(), Some("mfa-secret"));
        assert!(auth.valid);
    }

    #[test]
    fn invalid_user_auth_info_uses_dummy_hash() {
        let auth = UserAuthInfo::invalid();

        assert_eq!(auth.user_id, 0);
        assert_eq!(auth.password_hash, INVALID_PASSWORD_HASH);
        assert_eq!(auth.multi_factor_secret, None);
        assert!(!auth.valid);
    }
}
