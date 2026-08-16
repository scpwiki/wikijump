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
    HEADER_BASIC_ERROR, HEADER_BLOB_HASH, HEADER_FILENAME, HEADER_PAGE_SLUG,
    HEADER_USER_ID, get_header, get_site_id, get_site_slug,
};
use crate::error::{BasicError, FallbackError, build_basic_error_response};
use crate::state::ServerState;
use axum::extract::{Path, State};
use axum::http::header::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum_extra::TypedHeader;
use headers::Host;

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

fn get_blob_hash(headers: &HeaderMap) -> &str {
    get_header(
        headers,
        HEADER_BLOB_HASH,
        "No blob s3 hash header in request",
        "Blob s3 hash header is not UTF-8",
    )
}

fn get_user_id(headers: &HeaderMap) -> i64 {
    get_header(
        headers,
        HEADER_USER_ID,
        "No user ID header in request",
        "User ID header is not UTF-8",
    )
    .parse()
    .expect("User ID is not a valid integer")
}

pub async fn handle_basic_error(
    State(state): State<ServerState>,
    TypedHeader(host_info): TypedHeader<Host>,
    Path(error_code): Path<String>,
    headers: HeaderMap,
) -> Response {
    info!(error_code = error_code, "Returning basic error response");

    // This header can only be set internally, so let's check it before
    // returning any error information.
    if headers.get(HEADER_BASIC_ERROR).is_none() {
        // XF-1002
        return FallbackError::BasicErrorDirect.into_response();
    }

    // Build the appropriate BasicError enum case
    let input = match error_code.as_str() {
        // Required headers:
        // - x-wikijump-site-slug
        "site-slug" => {
            let site_slug = get_site_slug(&headers);
            BasicError::SiteSlug { site_slug }
        }
        // No required headers
        "site-custom" => BasicError::SiteCustom {
            host: host_info.hostname(),
        },
        // Required headers:
        // - x-wikijump-page-slug
        "page-slug" => {
            let site_id = get_site_id(&headers);
            let page_slug = get_page_slug(&headers);
            BasicError::PageSlug { site_id, page_slug }
        }
        // Required headers:
        // - x-wikijump-page-slug
        "page-fetch" => {
            let site_id = get_site_id(&headers);
            let page_slug = get_page_slug(&headers);
            BasicError::PageFetch { site_id, page_slug }
        }
        // Required headers:
        // - x-wikijump-page-slug
        // - x-wikijump-filename
        "file-name" => {
            let site_id = get_site_id(&headers);
            let page_slug = get_page_slug(&headers);
            let filename = get_filename(&headers);
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
            let site_id = get_site_id(&headers);
            let page_slug = get_page_slug(&headers);
            let filename = get_filename(&headers);
            BasicError::FileFetch {
                site_id,
                page_slug,
                filename,
            }
        }
        // No required headers
        "file-root" => BasicError::FileRoot,
        // Required headers:
        // - x-wikijump-s3-hash
        "blob-fetch" => BasicError::BlobFetch {
            s3_hash: get_blob_hash(&headers),
        },
        // Required headers:
        // - x-wikijump-user-id
        "user-fetch" => BasicError::UserFetch {
            user_id: get_user_id(&headers),
        },
        // Required headers:
        // - x-wikijump-user-id
        "user-avatar" => BasicError::UserAvatar {
            user_id: get_user_id(&headers),
        },
        // Invalid
        _ => {
            // XF-1000
            error!("Invalid basic error code: {error_code}");
            return FallbackError::BasicErrorCode.into_response();
        }
    };

    build_basic_error_response(&state, &headers, input).await
}
