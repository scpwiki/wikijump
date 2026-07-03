/*
 * handler/basic_error.rs
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

use super::{
    HEADER_BASIC_ERROR, HEADER_FILENAME, HEADER_PAGE_SLUG, get_header, get_site_id,
    get_site_slug,
};
use crate::error::{BasicError, FallbackError, build_basic_error_response};
use crate::state::ServerState;
use axum::extract::{Path, State};
use axum::http::header::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum_extra::TypedHeader;
use headers::Host;
use std::result::Result as StdResult;

fn get_page_slug(headers: &HeaderMap) -> &str {
    get_header(
        headers,
        HEADER_PAGE_SLUG,
        "No page slug header in request",
        "Page slug header is not UTF-8",
    )
}

fn get_filename(headers: &HeaderMap) -> &str {
    get_header(
        headers,
        HEADER_FILENAME,
        "No filename header in request",
        "Filename header is not UTF-8",
    )
}

fn basic_error_from_request<'a>(
    host: &'a str,
    error_code: &str,
    headers: &'a HeaderMap,
) -> StdResult<BasicError<'a>, FallbackError> {
    // This header can only be set internally, so let's check it before
    // returning any error information.
    if headers.get(HEADER_BASIC_ERROR).is_none() {
        // XF-1002
        return Err(FallbackError::BasicErrorDirect);
    }

    // Build the appropriate BasicError enum case
    let input = match error_code {
        // Required headers:
        // - x-wikijump-site-slug
        "site-slug" => {
            let site_slug = get_site_slug(headers);
            BasicError::SiteSlug { site_slug }
        }
        // No required headers
        "site-custom" => BasicError::SiteCustom { host },
        // Required headers:
        // - x-wikijump-page-slug
        "page-slug" => {
            let site_id = get_site_id(headers);
            let page_slug = get_page_slug(headers);
            BasicError::PageSlug { site_id, page_slug }
        }
        // Required headers:
        // - x-wikijump-page-slug
        "page-fetch" => {
            let site_id = get_site_id(headers);
            let page_slug = get_page_slug(headers);
            BasicError::PageFetch { site_id, page_slug }
        }
        // Required headers:
        // - x-wikijump-page-slug
        // - x-wikijump-filename
        "file-name" => {
            let site_id = get_site_id(headers);
            let page_slug = get_page_slug(headers);
            let filename = get_filename(headers);
            BasicError::FileName {
                site_id,
                page_slug,
                filename,
            }
        }
        // Required headers:
        // - x-wikijump-page-slug
        // - x-wikijump-filename
        "file-fetch" => {
            let site_id = get_site_id(headers);
            let page_slug = get_page_slug(headers);
            let filename = get_filename(headers);
            BasicError::FileFetch {
                site_id,
                page_slug,
                filename,
            }
        }
        // No required headers
        "file-root" => BasicError::FileRoot,
        // Invalid
        _ => {
            // XF-1000
            error!("Invalid basic error code: {error_code}");
            return Err(FallbackError::BasicErrorCode);
        }
    };

    Ok(input)
}

pub async fn handle_basic_error(
    State(state): State<ServerState>,
    TypedHeader(host_info): TypedHeader<Host>,
    Path(error_code): Path<String>,
    headers: HeaderMap,
) -> Response {
    info!(error_code = error_code, "Returning basic error response");

    let input =
        match basic_error_from_request(host_info.hostname(), &error_code, &headers) {
            Ok(input) => input,
            Err(error) => return error.into_response(),
        };

    build_basic_error_response(&state, &headers, input).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::{HEADER_SITE_ID, HEADER_SITE_SLUG};
    use axum::http::HeaderValue;

    fn internal_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(HEADER_BASIC_ERROR, HeaderValue::from_static("1"));
        headers.insert(HEADER_SITE_ID, HeaderValue::from_static("42"));
        headers.insert(HEADER_SITE_SLUG, HeaderValue::from_static("scp-wiki"));
        headers.insert(HEADER_PAGE_SLUG, HeaderValue::from_static("scp-173"));
        headers.insert(HEADER_FILENAME, HeaderValue::from_static("image.png"));
        headers
    }

    #[test]
    fn rejects_direct_basic_error_access() {
        let headers = HeaderMap::new();

        assert_eq!(
            basic_error_from_request("example.com", "file-root", &headers).unwrap_err(),
            FallbackError::BasicErrorDirect,
        );
    }

    #[test]
    fn rejects_unknown_basic_error_code() {
        let headers = internal_headers();

        assert_eq!(
            basic_error_from_request("example.com", "unknown", &headers).unwrap_err(),
            FallbackError::BasicErrorCode,
        );
    }

    #[test]
    fn maps_site_error_codes_to_basic_error_inputs() {
        let headers = internal_headers();

        match basic_error_from_request("example.com", "site-slug", &headers).unwrap() {
            BasicError::SiteSlug { site_slug } => assert_eq!(site_slug, "scp-wiki"),
            other => panic!("unexpected input: {other:?}"),
        }

        match basic_error_from_request("example.com", "site-custom", &headers).unwrap() {
            BasicError::SiteCustom { host } => assert_eq!(host, "example.com"),
            other => panic!("unexpected input: {other:?}"),
        }
    }

    #[test]
    fn maps_page_error_codes_to_basic_error_inputs() {
        let headers = internal_headers();

        match basic_error_from_request("example.com", "page-slug", &headers).unwrap() {
            BasicError::PageSlug { site_id, page_slug } => {
                assert_eq!(site_id, 42);
                assert_eq!(page_slug, "scp-173");
            }
            other => panic!("unexpected input: {other:?}"),
        }

        match basic_error_from_request("example.com", "page-fetch", &headers).unwrap() {
            BasicError::PageFetch { site_id, page_slug } => {
                assert_eq!(site_id, 42);
                assert_eq!(page_slug, "scp-173");
            }
            other => panic!("unexpected input: {other:?}"),
        }
    }

    #[test]
    fn maps_file_error_codes_to_basic_error_inputs() {
        let headers = internal_headers();

        match basic_error_from_request("example.com", "file-name", &headers).unwrap() {
            BasicError::FileName {
                site_id,
                page_slug,
                filename,
            } => {
                assert_eq!(site_id, 42);
                assert_eq!(page_slug, "scp-173");
                assert_eq!(filename, "image.png");
            }
            other => panic!("unexpected input: {other:?}"),
        }

        match basic_error_from_request("example.com", "file-fetch", &headers).unwrap() {
            BasicError::FileFetch {
                site_id,
                page_slug,
                filename,
            } => {
                assert_eq!(site_id, 42);
                assert_eq!(page_slug, "scp-173");
                assert_eq!(filename, "image.png");
            }
            other => panic!("unexpected input: {other:?}"),
        }
    }

    #[test]
    fn maps_file_root_error_code_to_basic_error_input() {
        let headers = internal_headers();

        assert!(matches!(
            basic_error_from_request("example.com", "file-root", &headers).unwrap(),
            BasicError::FileRoot,
        ));
    }
}
