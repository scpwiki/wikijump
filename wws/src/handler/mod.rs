/*
 * handler/mod.rs
 *
 * Wilson's Web Server - Serves a zoo of user-generated content
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

mod avatar;
mod basic_error;
mod file;
mod misc;
mod redirect;
mod robots;
mod text_block;
mod well_known;

pub use self::avatar::*;
pub use self::basic_error::*;
pub use self::file::*;
pub use self::misc::*;
pub use self::redirect::*;
pub use self::robots::*;
pub use self::text_block::*;
pub use self::well_known::*;

use axum::http::header::{HeaderMap, HeaderName};

pub const HEADER_SITE_ID: HeaderName = HeaderName::from_static("x-wikijump-site-id");
pub const HEADER_SITE_SLUG: HeaderName = HeaderName::from_static("x-wikijump-site-slug");
pub const HEADER_PAGE_SLUG: HeaderName = HeaderName::from_static("x-wikijump-page-slug");
pub const HEADER_FILENAME: HeaderName = HeaderName::from_static("x-wikijump-filename");
pub const HEADER_BLOB_HASH: HeaderName = HeaderName::from_static("x-wikijump-s3-hash");
pub const HEADER_USER_ID: HeaderName = HeaderName::from_static("x-wikijump-user-id");
pub const HEADER_TARGET_SERVER: HeaderName =
    HeaderName::from_static("x-wikijump-target-server");
pub const HEADER_BASIC_ERROR: HeaderName =
    HeaderName::from_static("x-wikijump-basic-error");

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TargetServer {
    Main,
    Files,
}

/// Helper function to extract a string value from a header.
/// This function asserts that the header does exist.
fn get_header<'a>(
    headers: &'a HeaderMap,
    header: HeaderName,
    missing_header_message: &'static str,
    not_utf8_header_message: &'static str,
) -> &'a str {
    headers
        .get(header)
        .expect(missing_header_message)
        .to_str()
        .expect(not_utf8_header_message)
}

/// Helper function to get the site ID from headers.
fn get_site_id(headers: &HeaderMap) -> i64 {
    get_header(
        headers,
        HEADER_SITE_ID,
        "No site ID header in request",
        "Site ID header is not UTF-8",
    )
    .parse()
    .expect("Site ID is not a valid integer")
}

/// Helper function to get the site slug from headers.
fn get_site_slug(headers: &HeaderMap) -> &str {
    get_header(
        headers,
        HEADER_SITE_SLUG,
        "No site slug header in request",
        "Site slug header is not UTF-8",
    )
}

/// Helper function to get which target server Caddy has told us we are.
///
/// This is either `main` or `files`, and refers to whether routes like
/// `robots.txt` are `foo.wikijump.com` or `foo.wjfiles.com`.
fn get_target_server(headers: &HeaderMap) -> TargetServer {
    let value = headers
        .get(HEADER_TARGET_SERVER)
        .expect("No target server header in request")
        .as_bytes();

    match value {
        b"main" => TargetServer::Main,
        b"files" => TargetServer::Files,
        _ => panic!("Invalid header value: {value:?}"),
    }
}
