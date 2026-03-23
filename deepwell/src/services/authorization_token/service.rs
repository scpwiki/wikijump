/*
 * services/authorization_token/service.rs
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
use crate::models::authorization_token::{
    self, Entity as AuthorizationToken, Model as AuthorizationTokenModel,
};
use crate::types::ArrayLength;
use std::net::IpAddr;
use uuid::Uuid;
use uuid::fmt::Hyphenated;

pub const AUTHORIZATION_TOKEN_LENGTH: usize = 38;

#[derive(Debug)]
pub struct AuthorizationTokenService;

impl AuthorizationTokenService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        CreateAuthorizationToken {
            r#type: object_type,
            description,
            creating_user_id,
            ip_address,
        }: CreateAuthorizationToken,
    ) -> Result<String> {
        let token = Self::generate(object_type);
        assert_eq!(token.len(), AUTHORIZATION_TOKEN_LENGTH);

        let make_error = || {
            Error::new(
                format!(
                    "failed to create new authorization token for scope {:?} (created by user ID {})",
                    object_type, creating_user_id,
                ),
                ErrorType::AuthorizationToken,
            )
        };

        let txn = ctx.transaction();
        let model = authorization_token::ActiveModel {
            token_value: Set(token.clone()),
            created_by: Set(creating_user_id),
            description: Set(description),
            ..Default::default()
        };

        // TODO audit log
        let _ = ip_address;

        AuthorizationToken::insert(model)
            .exec(txn)
            .await
            .or_raise(make_error)?;

        Ok(token)
    }

    fn generate(object_type: AuthorizedObject) -> String {
        type TokenBuffer = [u8; 36];
        const_assert_eq!(TokenBuffer::LENGTH, Hyphenated::LENGTH);

        let mut buffer: TokenBuffer = [0; 36];
        Uuid::new_v4().hyphenated().encode_upper(&mut buffer);
        let uuid_str = str::from_utf8(&buffer)
            .expect("UUID hyphenated formatter produced non-UTF-8 output");

        format!("{}-{}", object_type.code(), uuid_str)
    }

    /// Verifies that an authorization token is valid, consuming it.
    ///
    /// This validates a token has been properly issued for its respective
    /// scope, and following this, removes the token.
    ///
    /// If this method returns `Ok(())`, then the user may proceed with the
    /// action described by `AuthorizedObject` and `token` is no longer valid.
    pub async fn verify(
        ctx: &ServiceContext<'_>,
        token: &str,
        object_type: AuthorizedObject,
        ip_address: IpAddr,
    ) -> Result<()> {
        info!(
            "Verifying authorization token '{}' (scope {:?})",
            token, object_type,
        );

        if token.len() != AUTHORIZATION_TOKEN_LENGTH {
            bail!(Error::new(
                format!(
                    "passed authorization token has an invalid length (actual {} ≠ expected {} bytes)",
                    token.len(),
                    AUTHORIZATION_TOKEN_LENGTH,
                ),
                ErrorType::BadRequest
            ));
        }

        let make_error = || {
            Error::new(
                "failed to verify authorization token, already used or invalid",
                ErrorType::InvalidAuthorizationToken,
            )
        };

        let char_code = first_char(token);
        if object_type.code() != char_code {
            error!(
                "Authorization token has char code '{}', but this scope is '{}'",
                char_code,
                object_type.code(),
            );
            bail!(make_error());
        }

        let txn = ctx.transaction();
        let token_id: i32 = AuthorizationToken::find()
            .select_only()
            .column(authorization_token::Column::TokenId)
            .filter(authorization_token::Column::TokenValue.eq(token))
            .into_tuple()
            .one(txn)
            .await
            .or_raise(make_error)?
            .ok_or_raise(make_error)?;

        info!(
            "Successfully matched token with row ID {}, will remove and return OK",
            token_id,
        );

        AuthorizationToken::delete_by_id(token_id)
            .exec(txn)
            .await
            .or_raise(make_error)?;

        // TODO audit log
        let _ = ip_address;

        Ok(())
    }
}

/// Gets the first unicode codepoint in a string.
///
/// # Panics
/// If the string is empty.
fn first_char(string: &str) -> char {
    string.chars().next().expect("empty string")
}

#[test]
fn generate_token() {
    use regex::Regex;
    use std::sync::LazyLock;

    static REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"^[A-Z]-[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}$",
        )
        .unwrap()
    });

    fn test(object_type: AuthorizedObject) {
        let token = AuthorizationTokenService::generate(object_type);
        assert_eq!(token.len(), AUTHORIZATION_TOKEN_LENGTH);
        assert_eq!(first_char(&token), object_type.code());
        assert!(REGEX.is_match(&token));
    }

    test(AuthorizedObject::Site);
    test(AuthorizedObject::User);
    test(AuthorizedObject::BotUser);
}
