/*
 * handler/well_known.rs
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

//! Handling for the `.well-known` special discovery path.
//!
//! Many different standard paths are served here, and each
//! should be implemented as a separate handler.

use super::get_target_server;
use axum::body::Body;
use axum::http::header::{self, HeaderMap};
use axum::http::status::StatusCode;
use axum::response::Response;

const WELL_KNOWN_NOT_CONFIGURED: &str =
    "No .well-known resource is configured for this WWS target.\n";

pub async fn handle_well_known(headers: HeaderMap) -> Response {
    let _target_server = get_target_server(&headers);

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(WELL_KNOWN_NOT_CONFIGURED))
        .expect("Unable to convert .well-known response data")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::HEADER_TARGET_SERVER;
    use axum::body::to_bytes;
    use axum::http::header::{CONTENT_TYPE, HeaderValue};
    use axum::response::IntoResponse;

    fn target_headers(target: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(HEADER_TARGET_SERVER, HeaderValue::from_static(target));
        headers
    }

    #[tokio::test]
    async fn well_known_main_target_returns_plain_not_found() {
        let response = handle_well_known(target_headers("main"))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response.headers()[CONTENT_TYPE],
            "text/plain; charset=utf-8",
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            body.as_ref(),
            b"No .well-known resource is configured for this WWS target.\n",
        );
    }

    #[tokio::test]
    async fn well_known_files_target_returns_plain_not_found() {
        let response = handle_well_known(target_headers("files"))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            body.as_ref(),
            b"No .well-known resource is configured for this WWS target.\n",
        );
    }
}
