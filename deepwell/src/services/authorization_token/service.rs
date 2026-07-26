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

use super::structs::{AuthorizedObject, CreateAuthorizationToken};
use crate::constants::ADMIN_USER_ID;
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::authorization_token::{
    self, Entity as AuthorizationToken, Model as AuthorizationTokenModel,
};
use crate::services::ServiceContext;
use crate::services::audit::{AuditEvent, AuditService};
use crate::types::ArrayLength;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
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
            ip_address,
        }: CreateAuthorizationToken,
    ) -> Result<String> {
        let creating_user_id = Self::require_platform_staff(ctx)?;
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

        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::AuthorizationTokenCreate {
                user_id: creating_user_id,
                object_type,
                description: &description,
            },
        )
        .await
        .or_raise(make_error)?;

        let txn = ctx.transaction();
        let model = authorization_token::ActiveModel {
            token_value: Set(token.clone()),
            created_by: Set(creating_user_id),
            description: Set(description),
            ..Default::default()
        };

        AuthorizationToken::insert(model)
            .exec(txn)
            .await
            .or_raise(make_error)?;

        Ok(token)
    }

    fn require_platform_staff(ctx: &ServiceContext<'_>) -> Result<i64> {
        let user_id = ctx.request().user_id().or_raise(|| {
            Error::new(
                "issuing authorization tokens requires an admin request context",
                ErrorType::PermissionDenied,
            )
        })?;

        if user_id != ADMIN_USER_ID {
            bail!(Error::new(
                "issuing authorization tokens requires an admin request context",
                ErrorType::PermissionDenied,
            ));
        }

        Ok(user_id)
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
        info!("Verifying authorization token for scope {:?}", object_type);

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
        let deleted_tokens: Vec<AuthorizationTokenModel> =
            AuthorizationToken::delete_many()
                .filter(authorization_token::Column::TokenValue.eq(token))
                .exec_with_returning(txn)
                .await
                .or_raise(make_error)?;

        let token_id = match deleted_tokens.as_slice() {
            [deleted_token] => deleted_token.token_id,
            _ => {
                error!(
                    "Authorization token consumption deleted {} rows instead of one",
                    deleted_tokens.len(),
                );
                bail!(make_error());
            }
        };

        info!("Successfully consumed authorization token row ID {token_id}");

        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::AuthorizationTokenVerify {
                object_type,
                token_id,
            },
        )
        .await
        .or_raise(make_error)?;

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
    fn test(object_type: AuthorizedObject) {
        let token = AuthorizationTokenService::generate(object_type);
        assert_eq!(token.len(), AUTHORIZATION_TOKEN_LENGTH);
        assert_eq!(first_char(&token), object_type.code());
        let regex = regex!(
            r"^[A-Z]-[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}$"
        );
        assert!(regex.is_match(&token));
    }

    test(AuthorizedObject::Site);
    test(AuthorizedObject::User);
    test(AuthorizedObject::BotUser);
}
