/*
 * handler/robots.rs
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

//! Handling for the `robots.txt` file.

use super::{TargetServer, get_target_server};
use axum::body::Body;
use axum::http::header::{self, HeaderMap};
use axum::http::status::StatusCode;
use axum::response::Response;

const ROBOTS_TXT_ALLOW_ALL: &str = "User-agent: *\nAllow: /\n";

pub async fn handle_robots_txt(headers: HeaderMap) -> Response {
    let body = match get_target_server(&headers) {
        TargetServer::Main | TargetServer::Files => ROBOTS_TXT_ALLOW_ALL,
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(body))
        .expect("Unable to convert robots.txt response data")
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
    async fn robots_txt_main_target_returns_plain_allow_all_policy() {
        let response = handle_robots_txt(target_headers("main"))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[CONTENT_TYPE],
            "text/plain; charset=utf-8",
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body.as_ref(), b"User-agent: *\nAllow: /\n");
    }

    #[tokio::test]
    async fn robots_txt_files_target_returns_plain_allow_all_policy() {
        let response = handle_robots_txt(target_headers("files"))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body.as_ref(), b"User-agent: *\nAllow: /\n");
    }
}
