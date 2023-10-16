/*
 * services/error.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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

use crate::locales::LocalizationTranslateError;
use filemagic::FileMagicError;
use jsonrpsee::types::error::ErrorObjectOwned;
use reqwest::Error as ReqwestError;
use s3::error::S3Error;
use sea_orm::{error::DbErr, TransactionError};
use thiserror::Error as ThisError;
use unic_langid::LanguageIdentifierError;

pub use std::error::Error as StdError;

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, Error>;

/// Wrapper error for possible failure modes from service methods.
#[derive(ThisError, Debug)]
pub enum Error {
    // Error passed straight to ErrorObjectOwned without conversion
    #[error("{0}")]
    Raw(#[from] ErrorObjectOwned),

    #[error("Cryptography error: {0}")]
    Cryptography(argon2::password_hash::Error),

    #[error("Database error: {0}")]
    Database(DbErr),

    #[error("Invalid locale: {0}")]
    Locale(#[from] LanguageIdentifierError),

    #[error("Localization error: {0}")]
    Localization(#[from] LocalizationTranslateError),

    #[error("Magic library error: {0}")]
    Magic(#[from] FileMagicError),

    #[error("One-time password error: {0}")]
    Otp(#[from] otp::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("S3 error: {0}")]
    S3(#[from] S3Error),

    // See also RemoteOperationFailed.
    #[error("Web request error: {0}")]
    WebRequest(#[from] ReqwestError),

    #[error("Invalid enum serialization value")]
    InvalidEnumValue,

    #[error("Inconsistency found in checked data")]
    Inconsistent,

    #[error("A request to a remote service returned an error")]
    RemoteOperationFailed,

    #[error("Attempting to perform a wikitext parse and render has timed out")]
    RenderTimeout,

    #[error("The user cannot rename as they do not have enough name change tokens")]
    InsufficientNameChanges,

    #[error("Invalid username, password, or TOTP code")]
    InvalidAuthentication,

    #[error("User ID {session_user_id} associated with session does not match active user ID {active_user_id}")]
    SessionUserId {
        active_user_id: i64,
        session_user_id: i64,
    },

    #[error("A password is required")]
    EmptyPassword,

    #[error("The user's email is disallowed")]
    DisallowedEmail,

    #[error("The user's email is invalid")]
    InvalidEmail,

    #[error("The request is in some way malformed or incorrect")]
    BadRequest,

    #[error("The server ran into an unspecified or unknown error")]
    InternalServerError,

    #[error("The request conflicts with data already present")]
    Conflict,

    #[error("The requested data exists, when it was expected to be missing")]
    Exists,

    #[error("The requested data was not found")]
    NotFound,

    #[error("The request violates a configured content filter")]
    FilterViolation,

    #[error("Cannot hide the wikitext for the latest page revision")]
    CannotHideLatestRevision,

    #[error("Cannot perform this action because you are blocked by the user")]
    UserBlockedUser,

    #[error("Cannot perform this action because you are blocked by the site")]
    SiteBlockedUser,

    #[error("The rate limit for an external API has been reached")]
    RateLimited,
}

// Error conversion implementations
//
// Required if the value doesn't implement StdError,
// or we want custom conversions.

impl From<argon2::password_hash::Error> for Error {
    #[inline]
    fn from(error: argon2::password_hash::Error) -> Error {
        match error {
            argon2::password_hash::Error::Password => Error::InvalidAuthentication,
            _ => Error::Cryptography(error),
        }
    }
}

impl From<DbErr> for Error {
    fn from(error: DbErr) -> Error {
        match error {
            DbErr::RecordNotFound(_) => Error::NotFound,
            _ => Error::Database(error),
        }
    }
}

// End-conversion for methods
//
// This is used to convert our ServiceError type into the RPC error type.

impl From<Error> for ErrorObjectOwned {
    fn from(error: Error) -> ErrorObjectOwned {
        // XXX
        todo!()
    }
}

// Helper function for unwrapping two layers of third party crate error wrapper types.

pub fn into_rpc_error(error: TransactionError<ErrorObjectOwned>) -> ErrorObjectOwned {
    match error {
        TransactionError::Connection(error) => Error::Database(error).into(),
        TransactionError::Transaction(error) => error,
    }
}
