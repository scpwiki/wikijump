/*
 * services/error.rs
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

use crate::locales::LocalizationTranslateError;
use cuid::CuidError;
use filemagic::FileMagicError;
use s3::error::S3Error;
use sea_orm::error::DbErr;
use thiserror::Error as ThisError;
use tide::{Error as TideError, StatusCode};

pub use std::error::Error as StdError;

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, Error>;

/// Wrapper error for possible failure modes from service methods.
///
/// This has a method to convert to a correct HTTP status,
/// facilitated by `PostTransactionToApiResponse`.
#[derive(ThisError, Debug)]
pub enum Error {
    #[error("CUID generation error: {0}")]
    Cuid(#[from] CuidError),

    #[error("Cryptography error: {0}")]
    Cryptography(argon2::password_hash::Error),

    #[error("Database error: {0}")]
    Database(DbErr),

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

    #[error("Web server error: HTTP {}", .0.status() as u16)]
    Web(TideError),

    #[error("Invalid enum serialization value")]
    InvalidEnumValue,

    #[error("A request to a remote service returned an error")]
    RemoteOperationFailed,

    #[error("Attempting to perform a wikitext parse and render has timed out")]
    RenderTimeout,

    #[error("The user cannot rename as they do not have enough name change tokens")]
    InsufficientNameChanges,

    #[error("Invalid username, password, or TOTP code")]
    InvalidAuthentication,

    #[error("The request is in some way malformed or incorrect")]
    BadRequest,

    #[error("The request conflicts with data already present")]
    Conflict,

    #[error("The requested data exists, when it was expected to be missing")]
    Exists,

    #[error("The requested data was not found")]
    NotFound,

    #[error("Cannot hide the wikitext for the latest page revision")]
    CannotHideLatestRevision,
}

impl Error {
    pub fn into_tide_error(self) -> TideError {
        match self {
            Error::Cuid(inner) => TideError::new(StatusCode::InternalServerError, inner),
            Error::Cryptography(_) => {
                TideError::from_str(StatusCode::InternalServerError, "")
            }
            Error::Database(inner) => {
                TideError::new(StatusCode::InternalServerError, inner)
            }
            Error::Magic(inner) => TideError::new(StatusCode::InternalServerError, inner),
            Error::Localization(inner) => TideError::new(StatusCode::NotFound, inner),
            Error::Otp(inner) => TideError::new(StatusCode::InternalServerError, inner),
            Error::Serde(inner) => TideError::new(StatusCode::InternalServerError, inner),
            Error::S3(inner) => TideError::new(StatusCode::InternalServerError, inner),
            Error::Web(inner) => inner,
            Error::InvalidEnumValue => {
                TideError::from_str(StatusCode::InternalServerError, "")
            }
            Error::RemoteOperationFailed | Error::RenderTimeout => {
                TideError::from_str(StatusCode::InternalServerError, "")
            }
            Error::InsufficientNameChanges => {
                TideError::from_str(StatusCode::PaymentRequired, "")
            }
            Error::InvalidAuthentication => {
                TideError::from_str(StatusCode::Forbidden, "")
            }
            Error::BadRequest => TideError::from_str(StatusCode::BadRequest, ""),
            Error::Exists | Error::Conflict => {
                TideError::from_str(StatusCode::Conflict, "")
            }
            Error::NotFound => TideError::from_str(StatusCode::NotFound, ""),
            Error::CannotHideLatestRevision => {
                TideError::from_str(StatusCode::BadRequest, "")
            }
        }
    }
}

// Error conversion implementations
//
// Required if the value doesn't implement StdError,
// or we want custom conversions.

impl From<argon2::password_hash::Error> for Error {
    #[inline]
    fn from(error: argon2::password_hash::Error) -> Error {
        Error::Cryptography(error)
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

impl From<TideError> for Error {
    #[inline]
    fn from(error: TideError) -> Error {
        Error::Web(error)
    }
}

/// Trait to easily convert the result of transactions to `ApiResponse`s.
pub trait PostTransactionToApiResponse<T> {
    fn to_api(self) -> StdResult<T, TideError>;
}

impl<T> PostTransactionToApiResponse<T> for Result<T> {
    #[inline]
    fn to_api(self) -> StdResult<T, TideError> {
        self.map_err(Error::into_tide_error)
    }
}
