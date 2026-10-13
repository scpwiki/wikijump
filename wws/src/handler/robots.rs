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
use axum::http::header::{self, HOST, HeaderMap};
use axum::http::status::StatusCode;
use axum::response::Response;

const ROBOTS_TXT_ALLOW_ALL: &str = "User-agent: *\nAllow: /\n";

pub async fn handle_robots_txt(headers: HeaderMap) -> Response {
    // The main site publishes a sitemap index at `/sitemap.xml` (Wikidot
    // route shape), so its crawler policy points at that index. The host
    // comes from the request itself, never a hard-coded domain. Without a
    // host, or on the files target which serves no sitemap, the policy stays
    // a plain allow-all.
    let body = match get_target_server(&headers) {
        TargetServer::Main => match headers.get(HOST).and_then(|value| value.to_str().ok())
        {
            Some(host) => {
                format!("{ROBOTS_TXT_ALLOW_ALL}Sitemap: https://{host}/sitemap.xml\n")
            }
            None => ROBOTS_TXT_ALLOW_ALL.to_owned(),
        },
        TargetServer::Files => ROBOTS_TXT_ALLOW_ALL.to_owned(),
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

    fn main_headers_with_host(host: &'static str) -> HeaderMap {
        let mut headers = target_headers("main");
        headers.insert(HOST, HeaderValue::from_static(host));
        headers
    }

    #[tokio::test]
    async fn robots_txt_main_target_advertises_host_sitemap_index() {
        let response = handle_robots_txt(main_headers_with_host("scp-wiki.example"))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[CONTENT_TYPE],
            "text/plain; charset=utf-8",
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            body.as_ref(),
            b"User-agent: *\nAllow: /\nSitemap: https://scp-wiki.example/sitemap.xml\n",
        );
    }

    #[tokio::test]
    async fn robots_txt_files_target_ignores_host_header() {
        let mut headers = target_headers("files");
        headers.insert(HOST, HeaderValue::from_static("files.example"));

        let response = handle_robots_txt(headers).await.into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body.as_ref(), b"User-agent: *\nAllow: /\n");
    }
}
