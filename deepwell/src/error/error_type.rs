/*
 * error/error_type.rs
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

use crate::hash::BlobHash;
use crate::services::filter::FilterSummary;
use crate::services::view::ViewType;
use fluent::FluentError;
use fluent_syntax::parser::ParserError as FluentParserError;
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    // 1000
    ApplicationStart,
    Request,
    DatabaseSeeder,
    HealthCheck,
    Login,
    Logout,
    Site,
    SiteSettings,
    User,
    Page,
    PageRevision,
    PageLink,
    PageOutdater,
    PageVote,
    File,
    FileRevision,
    GetView(ViewType),
    Job,
    Render,

    // 1100
    ServerSetup,
    DatabaseSetup,
    RedisSetup,
    ConfigSetup,

    // 1200
    DatabaseQuery,
    RedisQuery,
    RenderTimeout,
    RateLimited,
    EmailVerification,
    DatabaseImport,
    Localization,
    Fluent(Vec<FluentError>),
    FluentParser(Vec<FluentParserError>),
    Cryptography(String),

    // 1300
    Text,
    Blob,
    Message,
    MessageDraft,
    MessageRecord,
    SiteMembership,
    PageAttribution,
    PageCategory,
    PageParent,
    PageQuery,
    UserBotOwner,
    UserMfa,
    Caddyfile,
    BasicError,
    License,
    TextBlock,
    AuditLog,
    BlueprintPage,
    Filter,
    CustomDomain,
    Alias,
    AuthorizationToken,

    // 2000
    #[allow(dead_code)]
    GeneralNotFound,
    AliasNotFound,
    RelationNotFound,
    UserNotFound,
    SiteNotFound,
    PageNotFound,
    PageCategoryNotFound,
    PageParentNotFound,
    PageRevisionNotFound,
    FileNotFound,
    FileRevisionNotFound,
    VoteNotFound,
    FilterNotFound,
    CustomDomainNotFound,
    MessageNotFound,
    MessageDraftNotFound,
    BlobNotFound,
    TextNotFound,

    // 2100
    UserExists,
    UserMfaExists,
    SiteExists,
    PageExists,
    PageSlugExists,
    FileExists,
    FilterExists,
    CustomDomainExists,

    // 3000
    InvalidAuthentication,
    InvalidSessionToken,
    CreateSession,
    Session,
    SessionUserId {
        active_user_id: i64,
        session_user_id: i64,
    },
    EmptyPassword,
    InvalidAuthorizationToken,

    // 4000
    BadRequest,

    // 4100
    UserNameTooShort,
    UserSlugEmpty,
    UserEmailEmpty,
    UserWrongType,
    InsufficientNameChanges,
    DisallowedEmail,
    InvalidEmail,

    // 4200
    SiteSlugEmpty,

    // 4300
    PageSlugEmpty,
    PageNotDeleted,
    CannotHideLatestRevision,
    NotLatestRevisionId,

    // 4400
    FileNameEmpty,
    FileNameTooLong {
        length: usize,
        maximum: usize,
    },
    FileNameInvalidCharacters,
    FileMimeEmpty,
    FileNotDeleted,

    // 5000
    LocaleInvalid {
        locale: String,
    },
    LocaleMissing {
        locale: String,
    },
    LocaleMessageMissing {
        message_key: String,
    },
    LocaleMessageValueMissing {
        message_key: String,
    },
    LocaleMessageAttributeMissing {
        message_key: String,
        attribute: String,
    },
    NoLocalesSpecified,

    // 5100
    FilterViolation {
        field: String,
        value: String,
        failed: Vec<FilterSummary>,
    },
    FilterRegexInvalid {
        regex: String,
    },
    FilterNotDeleted,

    // 5200
    BlobNotUploaded,
    BlobWrongUser,
    BlobTooBig,
    BlobSizeMismatch {
        expected: usize,
        actual: usize,
    },
    BlobBlacklisted(BlobHash),
    BlobCannotBlacklistExisting,
    BlobBackend,

    // 5300
    MessageSubjectEmpty,
    MessageSubjectTooLong,
    MessageBodyEmpty,
    MessageBodyTooLong,
    MessageNoRecipients,
    MessageTooManyRecipients,

    // 5400
    CustomDomainSubdomain,
    CustomDomainWrongSite,
    CustomDomainUsePunycode {
        domain: String,
    },
    InvalidDomainValue {
        domain: String,
    },

    // 6000
    Relation,
    UserBlockRelation,
    UserBotOwnerRelation,
    UserFollowRelation,
    SiteBanRelation,
    SiteMemberRelation,
    SiteUserRelation,
    PageAttributionRelation,
    PageStarRelation,
    PageWatchRelation,

    // 6100
    UserBlockedUser,
    SiteBannedUser,
}

impl ErrorType {
    /// Returns a unique integer code for this type of error.
    ///
    /// Errors are divided into groups:
    /// * 1000 - High-level
    ///   * 1000 - Common
    ///   * 1100 - Intermediate Setup
    ///   * 1200 - Intermediate Operations
    ///   * 1300 - Other / Uncommon
    /// * 2000 - Data-consistency
    ///   * 2000 - Not Found
    ///   * 2100 - Already Exists
    /// * 3000 - Client / Protocol Errors
    ///   * 3000 - Authentication
    ///   * 3100 - Permissions
    /// * 4000 - Client / Request Errors / Core Data Objects
    ///   * 4000 - General
    ///   * 4100 - User
    ///   * 4200 - Site
    ///   * 4300 - Page
    ///   * 4400 - File
    /// * 5000 - Client / Request Errors / Ancillary Data Objects
    ///   * 5000 - Locale
    ///   * 5100 - Filter
    ///   * 5200 - Blob
    ///   * 5300 - Message
    ///   * 5400 - Domains
    /// * 6000 - Client / Request Errors / Composite Data
    ///   * 6000 - Relation Types
    ///   * 6100 - Relation Conflicts
    pub fn code(&self) -> i32 {
        match self {
            //
            // 1000 -- High-Level
            //

            // 1000 - Common
            ErrorType::ApplicationStart => 1000,
            ErrorType::Request => 1001,
            ErrorType::DatabaseSeeder => 1002,
            ErrorType::HealthCheck => 1003,
            ErrorType::Login => 1004,
            ErrorType::Logout => 1005,
            ErrorType::Site => 1006,
            ErrorType::SiteSettings => 1007,
            ErrorType::User => 1008,
            ErrorType::Page => 1009,
            ErrorType::PageRevision => 1010,
            ErrorType::PageLink => 1011,
            ErrorType::PageOutdater => 1012,
            ErrorType::PageVote => 1013,
            ErrorType::File => 1014,
            ErrorType::FileRevision => 1015,
            ErrorType::GetView(_) => 1016,
            ErrorType::Job => 1017,
            ErrorType::Render => 1018,

            // 1100 - Intermediate Setup
            ErrorType::ServerSetup => 1100,
            ErrorType::DatabaseSetup => 1101,
            ErrorType::RedisSetup => 1102,
            ErrorType::ConfigSetup => 1103,

            // 1200 - Intermediate Operations
            ErrorType::DatabaseQuery => 1200,
            ErrorType::RedisQuery => 1201,
            ErrorType::RenderTimeout => 1202,
            ErrorType::RateLimited => 1203,
            ErrorType::EmailVerification => 1204,
            ErrorType::DatabaseImport => 1205,
            ErrorType::Localization => 1206,
            ErrorType::Fluent(_) => 1207,
            ErrorType::FluentParser(_) => 1208,
            ErrorType::Cryptography(_) => 1209,

            // 1300 - Other / Uncommon
            ErrorType::Text => 1300,
            ErrorType::Blob => 1301,
            ErrorType::Message => 1302,
            ErrorType::MessageDraft => 1303,
            ErrorType::MessageRecord => 1304,
            ErrorType::SiteMembership => 1305,
            ErrorType::PageAttribution => 1306,
            ErrorType::PageCategory => 1307,
            ErrorType::PageParent => 1308,
            ErrorType::PageQuery => 1309,
            ErrorType::UserBotOwner => 1310,
            ErrorType::UserMfa => 1311,
            ErrorType::Caddyfile => 1312,
            ErrorType::BasicError => 1313,
            ErrorType::License => 1314,
            ErrorType::TextBlock => 1315,
            ErrorType::AuditLog => 1316,
            ErrorType::BlueprintPage => 1317,
            ErrorType::Filter => 1318,
            ErrorType::CustomDomain => 1319,
            ErrorType::Alias => 1320,
            ErrorType::AuthorizationToken => 1321,

            //
            // 2000 -- Data Consistency
            //

            // 2000 - Not Found
            ErrorType::GeneralNotFound => 2000,
            ErrorType::AliasNotFound => 2001,
            ErrorType::RelationNotFound => 2002,
            ErrorType::UserNotFound => 2003,
            ErrorType::SiteNotFound => 2004,
            ErrorType::PageNotFound => 2005,
            ErrorType::PageCategoryNotFound => 2006,
            ErrorType::PageParentNotFound => 2007,
            ErrorType::PageRevisionNotFound => 2008,
            ErrorType::FileNotFound => 2009,
            ErrorType::FileRevisionNotFound => 2010,
            ErrorType::VoteNotFound => 2011,
            ErrorType::FilterNotFound => 2012,
            ErrorType::CustomDomainNotFound => 2013,
            ErrorType::MessageNotFound => 2014,
            ErrorType::MessageDraftNotFound => 2015,
            ErrorType::BlobNotFound => 2016,
            ErrorType::TextNotFound => 2017,

            // 2100 - Already Exists
            ErrorType::UserExists => 2100,
            ErrorType::UserMfaExists => 2101,
            ErrorType::SiteExists => 2102,
            ErrorType::PageExists => 2103,
            ErrorType::PageSlugExists => 2104,
            ErrorType::FileExists => 2105,
            ErrorType::FilterExists => 2106,
            ErrorType::CustomDomainExists => 2107,

            //
            // 3000 -- Client / Protocol Errors
            //

            // 3000 - Authentication
            ErrorType::InvalidAuthentication => 3000,
            ErrorType::InvalidSessionToken => 3001,
            ErrorType::CreateSession => 3002,
            ErrorType::Session => 3003,
            ErrorType::SessionUserId { .. } => 3004,
            ErrorType::EmptyPassword => 3005,
            ErrorType::InvalidAuthorizationToken => 3006,

            // 3100 - Permissions
            // TODO

            //
            // 4000, 5000, 6000 -- Client / Request Errors
            //

            //
            // 4000 -- Client / Request Errors - Core Data Objects
            //

            // 4000 - General
            //
            // Some of these requests are pretty general, unless it is a rare edge case,
            // consider adding a new error case when code to handle new fail states are
            // introduced.
            ErrorType::BadRequest => 4000,

            // 4100 - User
            ErrorType::UserNameTooShort => 4100,
            ErrorType::UserSlugEmpty => 4101,
            ErrorType::UserEmailEmpty => 4102,
            ErrorType::UserWrongType => 4103,
            ErrorType::InsufficientNameChanges => 4104,
            ErrorType::InvalidEmail => 4105,
            ErrorType::DisallowedEmail => 4106,

            // 4200 - Site
            ErrorType::SiteSlugEmpty => 4200,

            // 4300 - Page
            ErrorType::PageSlugEmpty => 4300,
            ErrorType::PageNotDeleted => 4301,
            ErrorType::CannotHideLatestRevision => 4302,
            ErrorType::NotLatestRevisionId => 4303,

            // 4400 - File
            ErrorType::FileNameEmpty => 4400,
            ErrorType::FileNameTooLong { .. } => 4401,
            ErrorType::FileNameInvalidCharacters => 4402,
            ErrorType::FileMimeEmpty => 4403,
            ErrorType::FileNotDeleted => 4404,

            //
            // 5000 -- Client / Request Errors - Ancillary Data Objects
            //

            // 5000 - Locale
            ErrorType::LocaleInvalid { .. } => 5000,
            ErrorType::LocaleMissing { .. } => 5001,
            ErrorType::LocaleMessageMissing { .. } => 5002,
            ErrorType::LocaleMessageValueMissing { .. } => 5003,
            ErrorType::LocaleMessageAttributeMissing { .. } => 5004,
            ErrorType::NoLocalesSpecified => 5005,

            // 5100 - Filter
            ErrorType::FilterViolation { .. } => 5100,
            ErrorType::FilterRegexInvalid { .. } => 5101,
            ErrorType::FilterNotDeleted => 5102,

            // 5200 - Blob
            ErrorType::BlobNotUploaded => 5200,
            ErrorType::BlobWrongUser => 5201,
            ErrorType::BlobTooBig => 5202,
            ErrorType::BlobSizeMismatch { .. } => 5204,
            ErrorType::BlobBlacklisted(_) => 5205,
            ErrorType::BlobCannotBlacklistExisting => 5206,
            ErrorType::BlobBackend => 5207,

            // 5300 - Message
            ErrorType::MessageSubjectEmpty => 5300,
            ErrorType::MessageSubjectTooLong => 5301,
            ErrorType::MessageBodyEmpty => 5302,
            ErrorType::MessageBodyTooLong => 5303,
            ErrorType::MessageNoRecipients => 5304,
            ErrorType::MessageTooManyRecipients => 5305,

            // 5400 - Domains
            ErrorType::CustomDomainWrongSite => 5400,
            ErrorType::CustomDomainSubdomain => 5401,
            ErrorType::CustomDomainUsePunycode { .. } => 5402,
            ErrorType::InvalidDomainValue { .. } => 5403,

            //
            // 6000 -- Client / Request Errors - Composite Data
            //

            // 6000 - Relation Types
            ErrorType::Relation => 6000,

            //  6010 - User
            ErrorType::UserBlockRelation => 6010,
            ErrorType::UserBotOwnerRelation => 6011,
            ErrorType::UserFollowRelation => 6012,

            //  6020 - Site
            ErrorType::SiteBanRelation => 6020,
            ErrorType::SiteMemberRelation => 6021,
            ErrorType::SiteUserRelation => 6022,

            //  6030 - Page
            ErrorType::PageAttributionRelation => 6030,
            ErrorType::PageStarRelation => 6031,
            ErrorType::PageWatchRelation => 6032,

            // 6100 - Relation Conflicts
            ErrorType::UserBlockedUser => 6100,
            ErrorType::SiteBannedUser => 6101,
        }
    }

    /// Returns a basic summary of what this error is meant to represent.
    ///
    /// This is always a `&'static str`, so this lookup is cheap and has
    /// no effect of memory consumption.
    pub fn summary(&self) -> &'static str {
        match self {
            // 1000
            ErrorType::ApplicationStart => "Application failed to start",
            ErrorType::Request => "This request returned an error",
            ErrorType::DatabaseSeeder => "Database seeding failed",
            ErrorType::HealthCheck => "Health check failed",
            ErrorType::Login => "Log in failed",
            ErrorType::Logout => "Log out failed",
            ErrorType::Site => "Site operation failed",
            ErrorType::SiteSettings => "Site settings operation failed",
            ErrorType::User => "User operation failed",
            ErrorType::Page => "Page operation failed",
            ErrorType::PageRevision => "Page revision operation failed",
            ErrorType::PageLink => "Fetching or updating page links failed",
            ErrorType::PageOutdater => "Page outdater processing failed",
            ErrorType::PageVote => "Page vote operation failed",
            ErrorType::File => "File operation failed",
            ErrorType::FileRevision => "File revision operation failed",
            ErrorType::GetView(_) => "Getting web view failed",
            ErrorType::Job => "Failed to process job from queue",
            ErrorType::Render => "Wikitext parsing and rendering failed",

            // 1100
            ErrorType::ServerSetup => "Failed to set up server internal state",
            ErrorType::DatabaseSetup => "Failed to set up the database connection",
            ErrorType::RedisSetup => "Failed to set up the Redis connection",
            ErrorType::ConfigSetup => "Failed to load application configuration",

            // 1200
            ErrorType::DatabaseQuery => "Database query failed",
            ErrorType::RedisQuery => "Redis query failed",
            ErrorType::RateLimited => "An external API has ratelimited us",
            ErrorType::RenderTimeout => "Wikitext parsing and rendering has timed out",
            ErrorType::EmailVerification => "Email verification failed",
            ErrorType::DatabaseImport => "Database import operation failed",
            ErrorType::Localization => "Localization or translation failed",
            ErrorType::Fluent(_) => "Fluent bundle error",
            ErrorType::FluentParser(_) => "Fluent parser error",
            ErrorType::Cryptography(_) => "Cryptographic operation failed",

            // 1300
            ErrorType::Text => "Failed to act on a text entry",
            ErrorType::Blob => "Failed to act on a file blob",
            ErrorType::Message => "Failed to act on a message",
            ErrorType::MessageDraft => "Failed to act on a message draft",
            ErrorType::MessageRecord => "Failed to act on a message record",
            ErrorType::SiteMembership => "Failed to act on a site membership",
            ErrorType::PageAttribution => "Failed to act on a page attribution",
            ErrorType::PageCategory => "Failed to act on a page category",
            ErrorType::PageParent => "Failed to act on a page parent",
            ErrorType::PageQuery => "Failed to perform ListPages query",
            ErrorType::UserBotOwner => "Failed to act on a user / bot owner",
            ErrorType::UserMfa => "Failed to act on a user's MFA settings",
            ErrorType::Caddyfile => "Failed to generate a Caddyfile",
            ErrorType::BasicError => "Failed to generate a basic error message",
            ErrorType::License => "Failed to determine license data",
            ErrorType::TextBlock => "Failed to act on a text block",
            ErrorType::AuditLog => "Failed to generate an audit log entry",
            ErrorType::BlueprintPage => "Failed to get or format a blueprint page",
            ErrorType::Filter => "Failed to act on a filter",
            ErrorType::CustomDomain => "Failed to act on a custom domain",
            ErrorType::Alias => "Failed to act on an object alias",
            ErrorType::AuthorizationToken => {
                "Failed to create or verify an authorization token"
            }

            // 2000
            ErrorType::GeneralNotFound => "Unspecified entity does not exist",
            ErrorType::AliasNotFound => "Alias does not exist",
            ErrorType::RelationNotFound => "Relation value does not exist",
            ErrorType::UserNotFound => "User does not exist",
            ErrorType::SiteNotFound => "Site does not exist",
            ErrorType::PageNotFound => "Page does not exist",
            ErrorType::PageCategoryNotFound => "Page category does not exist",
            ErrorType::PageParentNotFound => "Page parent does not exist",
            ErrorType::PageRevisionNotFound => "Page revision does not exist",
            ErrorType::FileNotFound => "File does not exist",
            ErrorType::FileRevisionNotFound => "File revision does not exist",
            ErrorType::VoteNotFound => "Vote does not exist",
            ErrorType::FilterNotFound => "Filter does not exist",
            ErrorType::CustomDomainNotFound => "Custom domain does not exist",
            ErrorType::MessageNotFound => "Message does not exist",
            ErrorType::MessageDraftNotFound => "Message draft does not exist",
            ErrorType::BlobNotFound => "Blob item does not exist",
            ErrorType::TextNotFound => "Text item does not exist",

            // 2100
            ErrorType::UserExists => "Cannot perform, user already exists",
            ErrorType::UserMfaExists => "Cannot set up user MFA, already set up",
            ErrorType::SiteExists => "Cannot perform, site already exists",
            ErrorType::PageExists => "Cannot perform, page already exists",
            ErrorType::PageSlugExists => "Cannot perform, page slug already exists",
            ErrorType::FileExists => "Cannot perform, file already exists",
            ErrorType::FilterExists => "Cannot perform, filter already exists",
            ErrorType::CustomDomainExists => {
                "Cannot perform, custom domain already exists"
            }

            // 3000
            ErrorType::InvalidAuthentication => {
                "Invalid username, password, or TOTP code"
            }
            ErrorType::InvalidSessionToken => {
                "Invalid session token, cannot be used for authentication"
            }
            ErrorType::CreateSession => "Unable to create new login session",
            ErrorType::Session => "Unable to perform operation involving a login session",
            ErrorType::SessionUserId { .. } => {
                "User associated with the session does not match the active user"
            }
            ErrorType::EmptyPassword => "A password was required, but not provided",
            ErrorType::InvalidAuthorizationToken => {
                "Provided authorization token was invalid"
            }

            // 4000
            ErrorType::BadRequest => "The request is in some way malformed or incorrect",

            // 4100
            ErrorType::UserNameTooShort => "User name is too short",
            ErrorType::UserSlugEmpty => "User slug cannot be empty",
            ErrorType::UserEmailEmpty => "User email cannot be empty",
            ErrorType::UserWrongType => "Wrong user type for this operation",
            ErrorType::InsufficientNameChanges => {
                "The user cannot rename as they do not have enough name change tokens"
            }
            ErrorType::DisallowedEmail => "The user's email is disallowed",
            ErrorType::InvalidEmail => "The user's email is invalid",

            // 4200
            ErrorType::SiteSlugEmpty => "Site slug cannot be empty",

            // 4300
            ErrorType::PageSlugEmpty => "Page slug cannot be empty",
            ErrorType::PageNotDeleted => "Cannot restore a non-deleted page",
            ErrorType::CannotHideLatestRevision => {
                "Cannot hide the wikitext for the latest page revision"
            }
            ErrorType::NotLatestRevisionId => {
                "Revision ID passed for this operation is not the latest"
            }

            // 4400
            ErrorType::FileNameEmpty => "File name cannot be empty",
            ErrorType::FileNameTooLong { .. } => "File name too long",
            ErrorType::FileNameInvalidCharacters => {
                "File name contains invalid characters (control chars or slashes)"
            }
            ErrorType::FileMimeEmpty => "File MIME type cannot be empty",
            ErrorType::FileNotDeleted => "Cannot restore a non-deleted file",

            // 5000
            ErrorType::LocaleInvalid { .. } => "Invalid locale name",
            ErrorType::LocaleMissing { .. } => {
                "No messages are available for this locale"
            }
            ErrorType::LocaleMessageMissing { .. } => {
                "Message key not found for this locale"
            }
            ErrorType::LocaleMessageValueMissing { .. } => {
                "Message key was found, but has no value"
            }
            ErrorType::LocaleMessageAttributeMissing { .. } => {
                "Message key was found, but does not have this attribute"
            }
            ErrorType::NoLocalesSpecified => "No locales were specified in the request",

            // 5100
            ErrorType::FilterViolation { .. } => {
                "The request violates a configured content filter"
            }
            ErrorType::FilterRegexInvalid { .. } => {
                "The proposed filter has invalid regex"
            }
            ErrorType::FilterNotDeleted => "Cannot restore a non-deleted filter",

            // 5200
            ErrorType::BlobNotUploaded => "Blob not uploaded",
            ErrorType::BlobWrongUser => "Cannot use blob uploaded by different user",
            ErrorType::BlobTooBig => "Uploaded blob is too big for this operation",
            ErrorType::BlobSizeMismatch { .. } => {
                "Uploaded blob does not match expected length"
            }
            ErrorType::BlobBlacklisted(_) => "Uploaded blob content is blacklisted",
            ErrorType::BlobCannotBlacklistExisting => {
                "Cannot blacklist a blob which is already in use, you must do a hard deletion"
            }
            ErrorType::BlobBackend => "S3 operation failed",

            // 5300
            ErrorType::MessageSubjectEmpty => "Message subject cannot be empty",
            ErrorType::MessageSubjectTooLong => "Message subject too long",
            ErrorType::MessageBodyEmpty => "Message body cannot be empty",
            ErrorType::MessageBodyTooLong => "Message body too long",
            ErrorType::MessageNoRecipients => "Message cannot have no recipients",
            ErrorType::MessageTooManyRecipients => "Message has too many recipients",

            // 5400
            ErrorType::CustomDomainSubdomain => {
                "Custom domains may not be subdomains of the Wikijump main or file domains"
            }
            ErrorType::CustomDomainWrongSite => {
                "Cannot use custom domain, as it belongs to a different site"
            }
            ErrorType::CustomDomainUsePunycode { .. } => {
                "Submitted domain values should use punycode"
            }
            ErrorType::InvalidDomainValue { .. } => {
                "Domain value contains unexpected characters"
            }

            // 6000
            ErrorType::Relation => "Cannot perform relation operation",

            //  6010 - User
            ErrorType::UserBlockRelation => "Failed to act on a user block",
            ErrorType::UserBotOwnerRelation => {
                "Failed to act on a bot user's ownership information"
            }
            ErrorType::UserFollowRelation => "Failed to act on a user follow",

            //  6020 - Site
            ErrorType::SiteBanRelation => "Failed to act on a site ban",
            ErrorType::SiteMemberRelation => "Failed to act on a site membership",
            ErrorType::SiteUserRelation => "Failed to act on a site user",

            //  6030 - Page
            ErrorType::PageAttributionRelation => {
                "Failed to act on a page attribution entry"
            }
            ErrorType::PageStarRelation => "Failed to act on a page star",
            ErrorType::PageWatchRelation => "Failed to act on a page watch",

            // 6100
            ErrorType::UserBlockedUser => {
                "Cannot perform this action because you are blocked by the user"
            }
            ErrorType::SiteBannedUser => {
                "Cannot perform this action because you are banned by the site"
            }
        }
    }

    /// Returns auxiliary data for this error.
    ///
    /// In effect, this serializes any contents of this error.
    /// For instance, if it refers to a particular user ID
    /// which caused an issue then this value would be
    /// returned in the JSON output.
    pub fn data(&self) -> JsonValue {
        use crate::hash::blob_hash_to_hex;
        use serde_json::json;
        use std::fmt::Display;

        fn misc_errors_to_json<T: Display>(errors: &[T]) -> JsonValue {
            errors
                .iter()
                .map(|e| JsonValue::from(str!(e)))
                .collect::<Vec<_>>()
                .into()
        }

        match self {
            ErrorType::GetView(view_type) => json!(view_type),
            ErrorType::Fluent(errors) => misc_errors_to_json(errors),
            ErrorType::FluentParser(errors) => misc_errors_to_json(errors),
            ErrorType::Cryptography(extra) => json!(extra),
            ErrorType::SessionUserId {
                active_user_id,
                session_user_id,
            } => json!({
                "active_user_id": active_user_id,
                "session_user_id": session_user_id,
            }),
            ErrorType::FileNameTooLong { length, maximum } => json!({
                "length": length,
                "maximum": maximum,
            }),
            ErrorType::LocaleInvalid { locale } | ErrorType::LocaleMissing { locale } => {
                json!({ "locale": locale })
            }
            ErrorType::LocaleMessageMissing { message_key }
            | ErrorType::LocaleMessageValueMissing { message_key } => json!({
                "message_key": message_key,
            }),
            ErrorType::LocaleMessageAttributeMissing {
                message_key,
                attribute,
            } => json!({
                "message_key": message_key,
                "attribute": attribute,
            }),
            ErrorType::FilterViolation {
                field,
                value,
                failed,
            } => json!({
                "field": field,
                "value": value,
                "failed": failed,
            }),
            ErrorType::FilterRegexInvalid { regex } => json!({
                "regex": regex,
            }),
            ErrorType::BlobSizeMismatch { expected, actual } => json!({
                "expected": expected,
                "actual": actual,
            }),
            ErrorType::BlobBlacklisted(bytes) => json!(*blob_hash_to_hex(bytes)),
            _ => json!(null),
        }
    }

    /// Indicates if this error type is high-level.
    ///
    /// Such errors are not useful to return to end users and exist
    /// to indicate the overall kind of operation being performed.
    pub fn is_high_level(&self) -> bool {
        matches!(self.code(), 1000..2000)
    }
}
