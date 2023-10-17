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
    LocaleInvalid(#[from] LanguageIdentifierError),

    #[error("No messages are available for this locale")]
    LocaleMissing,

    #[error("Message key not found for this locale")]
    LocaleMessageMissing,

    #[error("Message key was found, but has no value")]
    LocaleMessageValueMissing,

    #[error("Message key was found, but does not have this attribute")]
    LocaleMessageAttributeMissing,

    #[error("Magic library error: {0}")]
    Magic(#[from] FileMagicError),

    #[error("One-time password error: {0}")]
    Otp(#[from] otp::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("S3 service returned error: {0}")]
    S3Service(#[from] S3Error),

    #[error("S3 service failed to respond properly")]
    S3Response,

    #[error("Email verification error: {}", .0.as_ref().unwrap_or(&str!("<unspecified>")))]
    EmailVerification(Option<String>),

    #[error("Web request error: {0}")]
    WebRequest(#[from] ReqwestError),

    #[error("Invalid enum serialization value")]
    InvalidEnumValue,

    #[error("Attempting to perform a wikitext parse and render has timed out")]
    RenderTimeout,

    #[error("The user cannot rename as they do not have enough name change tokens")]
    InsufficientNameChanges,

    #[error("Invalid username, password, or TOTP code")]
    InvalidAuthentication,

    #[error("Backend error while trying to authenticate")]
    AuthenticationBackend(Box<Error>),

    #[error("Invalid session token, cannot be used for authentication")]
    InvalidSessionToken,

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

    #[error("The request violates a configured content filter")]
    FilterViolation,

    #[error("The regular expression found in the database is invalid")]
    FilterRegexInvalid(regex::Error),

    #[error("Cannot hide the wikitext for the latest page revision")]
    CannotHideLatestRevision,

    #[error("Unspecified entity not found")]
    GeneralNotFound,

    #[error("Alias does not exist")]
    AliasNotFound,

    #[error("Interaction value does not exist")]
    InteractionNotFound,

    #[error("User does not exist")]
    UserNotFound,

    #[error("Site does not exist")]
    SiteNotFound,

    #[error("Page does not exist")]
    PageNotFound,

    #[error("Page category does not exist")]
    PageCategoryNotFound,

    #[error("Page parent does not exist")]
    PageParentNotFound,

    #[error("Page revision does not exist")]
    PageRevisionNotFound,

    #[error("File does not exist")]
    FileNotFound,

    #[error("File revision does not exist")]
    FileRevisionNotFound,

    #[error("Vote does not exist")]
    VoteNotFound,

    #[error("Filter does not exist")]
    FilterNotFound,

    #[error("Custom domain does not exist")]
    CustomDomainNotFound,

    #[error("Message does not exist")]
    MessageNotFound,

    #[error("Message draft does not exist")]
    MessageDraftNotFound,

    #[error("Blob item does not exist")]
    BlobNotFound,

    #[error("Text item does not exist")]
    TextNotFound,

    #[error("Cannot perform, user already exists")]
    UserExists,

    #[error("Cannot set up user MFA, already set up")]
    UserMfaExists,

    #[error("Cannot perform, site already exists")]
    SiteExists,

    #[error("Cannot perform, page already exists")]
    PageExists,

    #[error("Cannot perform, page parent already exists")]
    PageParentExists,

    #[error("Cannot perform, file already exists")]
    FileExists,

    #[error("Cannot perform, filter already exists")]
    FilterExists,

    #[error("Cannot perform, custom domain already exists")]
    CustomDomainExists,

    #[error("Cannot perform this action because you are blocked by the user")]
    UserBlockedUser,

    #[error("Cannot perform this action because you are blocked by the site")]
    SiteBlockedUser,

    #[error("The rate limit for an external API has been reached")]
    RateLimited,
}

impl Error {
    /// Returns the code associated with this error.
    ///
    /// The JSON-RPC spec has each unique error case return its own integer error code.
    /// Some very negative codes are reserved for RPC internals, so we will only output
    /// positive values.
    ///
    /// Sort of similar to HTTP status codes, we are also dividing them into groups based
    /// generally on the kind of error it is.
    ///
    /// When an error case is removed, then its number should not be reused, just use the next
    /// available value in line.
    pub fn code(&self) -> i32 {
        match self {
            // 1000 - Miscellaneous, general errors
            //        Avoid putting stuff here, prefer other categories instead
            Error::Raw(_) => 1000,

            // 2000 - Database conflicts
            //        Missing fields
            Error::GeneralNotFound => 2000,
            Error::AliasNotFound => 2001,
            Error::InteractionNotFound => 2002,
            Error::UserNotFound => 2003,
            Error::SiteNotFound => 2004,
            Error::PageNotFound => 2005,
            Error::PageCategoryNotFound => 2006,
            Error::PageParentNotFound => 2007,
            Error::PageRevisionNotFound => 2008,
            Error::FileNotFound => 2009,
            Error::FileRevisionNotFound => 2010,
            Error::VoteNotFound => 2011,
            Error::FilterNotFound => 2012,
            Error::CustomDomainNotFound => 2013,
            Error::MessageNotFound => 2014,
            Error::MessageDraftNotFound => 2015,
            Error::BlobNotFound => 2016,
            Error::TextNotFound => 2017,

            // 2100 -- Existing fields
            Error::UserExists => 2100,
            Error::UserMfaExists => 2101,
            Error::SiteExists => 2102,
            Error::PageExists => 2103,
            Error::PageParentExists => 2104,
            Error::FileExists => 2105,
            Error::FilterExists => 2106,
            Error::CustomDomainExists => 2107,

            // 3000 - Server errors, unexpected
            Error::RateLimited => 3000,
            Error::WebRequest(_) => 3001,
            Error::AuthenticationBackend(_) => 3002,

            // 3100 -- Remote services
            Error::RenderTimeout => 3100,
            Error::EmailVerification(_) => 3101,
            Error::S3Service(_) => 3102,
            Error::S3Response => 3103,

            // 3200 -- Backend issues
            Error::Serde(_) => 3200,
            Error::Database(_) => 3201,
            Error::Cryptography(_) => 3202,
            Error::FilterRegexInvalid(_) => 3203,
            Error::Magic(_) => 3204,
            Error::Otp(_) => 3205,

            // 4000 - Client, request errors
            Error::BadRequest => 4000,
            Error::InvalidEnumValue => 4001,
            Error::FilterViolation => 4002,
            Error::InsufficientNameChanges => 4003,
            Error::CannotHideLatestRevision => 4004,

            // 4100 -- Localization
            Error::LocaleInvalid(_) => 4100,
            Error::LocaleMissing => 4101,
            Error::LocaleMessageMissing => 4102,
            Error::LocaleMessageValueMissing => 4103,
            Error::LocaleMessageAttributeMissing => 4104,

            // 4200 -- Login errors
            Error::EmptyPassword => 4200,
            Error::InvalidEmail => 4201,
            Error::DisallowedEmail => 4202,

            // 4300 -- Relationship conflicts
            Error::SiteBlockedUser => 4300,
            Error::UserBlockedUser => 4301,

            // 5000 - Authentication, permission, or role errors
            Error::InvalidAuthentication => 5000,
            Error::InvalidSessionToken => 5001,
            Error::SessionUserId { .. } => 5002,
            // TODO: permission errors (e.g. locked page, cannot apply bans)
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
        match error {
            // Password is invalid, expected error
            argon2::password_hash::Error::Password => Error::InvalidAuthentication,

            // Problem with the password hashing process
            _ => Error::Cryptography(error),
        }
    }
}

impl From<DbErr> for Error {
    fn from(error: DbErr) -> Error {
        match error {
            DbErr::RecordNotFound(_) => Error::GeneralNotFound,
            _ => Error::Database(error),
        }
    }
}

// End-conversion for methods
//
// This is used to convert our ServiceError type into the RPC error type.

impl From<Error> for ErrorObjectOwned {
    fn from(error: Error) -> ErrorObjectOwned {
        let error_code = error.code();
        let message = str!(error);
        let data = todo!(); // XXX use error
        ErrorObjectOwned::owned(error_code, message, Some(data))
    }
}

// Helper function for unwrapping two layers of third party crate error wrapper types.

pub fn into_rpc_error(error: TransactionError<ErrorObjectOwned>) -> ErrorObjectOwned {
    match error {
        TransactionError::Connection(error) => Error::Database(error).into(),
        TransactionError::Transaction(error) => error,
    }
}
