/*
 * fetch.rs
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

use crate::deepwell::FileData;
use crate::error::{BasicError, ResponseResult, build_basic_error_response};
use crate::range::ByteRange;
use crate::state::ServerState;
use axum::body::Body;
use axum::http::header::HeaderMap;
use s3::request::request_trait::ResponseDataStream;
use wikidot_normalize::normalize;

pub async fn fetch_file_info(
    state: &ServerState,
    headers: &HeaderMap,
    site_id: i64,
    page_slug: &mut String,
    filename: &str,
) -> ResponseResult<FileData> {
    normalize(page_slug);

    let page_id = state
        .get_page_or_response(headers, site_id, page_slug)
        .await?;

    state
        .get_file_or_response(headers, site_id, page_id, page_slug, filename)
        .await
}

pub async fn fetch_full_body(
    state: &ServerState,
    headers: &HeaderMap,
    site_id: i64,
    file_info: &FileData,
    page_slug: &str,
    filename: &str,
) -> ResponseResult<Body> {
    match state
        .s3_files_bucket
        .get_object_stream(&file_info.s3_hash)
        .await
    {
        Ok(ResponseDataStream { bytes, status_code }) => {
            if status_code != 200 {
                error!(
                    site_id = site_id,
                    page_slug = page_slug,
                    filename = filename,
                    s3_hash = &file_info.s3_hash,
                    status_code = status_code,
                    "S3 get_object_stream returned unexpected status",
                );

                let response = build_basic_error_response(
                    state,
                    headers,
                    BasicError::FileFetch {
                        site_id,
                        page_slug,
                        filename,
                    },
                )
                .await;

                return Err(response);
            }

            Ok(Body::from_stream(bytes))
        }
        Err(error) => {
            // NOTE: If the error here is 404 we still return 500.
            //
            //       If we have a file record for a file, then the
            //       corresponding blob *should* exist.
            //
            //       If it doesn't, the data invariant is not being met,
            //       which is an unexpected error.
            error!(
                site_id = site_id,
                page_slug = page_slug,
                filename = filename,
                s3_hash = &file_info.s3_hash,
                "Cannot get blob data: {error}",
            );

            let response = build_basic_error_response(
                state,
                headers,
                BasicError::FileFetch {
                    site_id,
                    page_slug,
                    filename,
                },
            )
            .await;

            Err(response)
        }
    }
}

// Fetch a single byte range as a stream by cloning the bucket and
// injecting an HTTP Range header, so we never buffer the range in memory
pub async fn fetch_range_stream(
    state: &ServerState,
    file_info: &FileData,
    range: ByteRange,
) -> Result<Body, s3::error::S3Error> {
    let mut bucket = (*state.s3_files_bucket).clone();
    bucket.add_header("range", &format!("bytes={}-{}", range.start, range.end));
    let ResponseDataStream { bytes, status_code } =
        bucket.get_object_stream(&file_info.s3_hash).await?;

    if status_code != 206 {
        error!(
            s3_hash = &file_info.s3_hash,
            status_code = status_code,
            "S3 range stream returned unexpected status (expected 206)",
        );
        return Err(s3::error::S3Error::HttpFailWithBody(
            status_code,
            format!("expected 206, got {status_code}"),
        ));
    }

    Ok(Body::from_stream(bytes))
}

// Fetch a single byte range into memory (used for multipart assembly)
pub async fn fetch_range_bytes(
    state: &ServerState,
    file_info: &FileData,
    range: ByteRange,
) -> Result<Vec<u8>, s3::error::S3Error> {
    let resp = state
        .s3_files_bucket
        .get_object_range(&file_info.s3_hash, range.start, Some(range.end))
        .await?;

    if resp.status_code() != 206 {
        error!(
            s3_hash = &file_info.s3_hash,
            status_code = resp.status_code(),
            "S3 range get returned unexpected status (expected 206)",
        );
        return Err(s3::error::S3Error::HttpFailWithBody(
            resp.status_code(),
            format!("expected 206, got {}", resp.status_code()),
        ));
    }

    Ok(resp.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Secrets;
    use crate::state::build_server_state;
    use axum::body;
    use axum::http::HeaderMap;
    use s3::creds::Credentials;
    use s3::region::Region;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread::{self, JoinHandle};
    use std::time::Duration;

    struct S3Server {
        endpoint: String,
        requests: Arc<Mutex<Vec<String>>>,
        handle: JoinHandle<()>,
    }

    impl S3Server {
        fn requests(&self) -> Vec<String> {
            self.requests.lock().unwrap().clone()
        }

        fn join(self) {
            self.handle.join().unwrap();
        }
    }

    fn spawn_s3_server(responses: Vec<(u16, &'static [u8])>) -> S3Server {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let server_requests = Arc::clone(&requests);

        let handle = thread::spawn(move || {
            for (status, body) in responses {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();

                let mut raw = Vec::new();
                let mut chunk = [0_u8; 1024];
                loop {
                    let read = stream.read(&mut chunk).unwrap();
                    if read == 0 {
                        break;
                    }
                    raw.extend_from_slice(&chunk[..read]);
                    if raw.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }

                server_requests
                    .lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&raw).into_owned());

                let response = format!(
                    concat!(
                        "HTTP/1.1 {status} OK\r\n",
                        "Content-Length: {length}\r\n",
                        "Content-Type: application/octet-stream\r\n",
                        "ETag: \"test-etag\"\r\n",
                        "Connection: close\r\n",
                        "\r\n",
                    ),
                    status = status,
                    length = body.len(),
                );
                stream.write_all(response.as_bytes()).unwrap();
                stream.write_all(body).unwrap();
            }
        });

        S3Server {
            endpoint,
            requests,
            handle,
        }
    }

    fn file_data() -> FileData {
        FileData {
            file_id: 1,
            mime: str!("text/plain"),
            size: 6,
            s3_hash: str!("sha512-hash"),
        }
    }

    async fn test_state(endpoint: &str) -> ServerState {
        let mut state = build_server_state(
            false,
            Secrets {
                deepwell_url: str!("http://127.0.0.1:2747"),
                redis_url: str!("redis://127.0.0.1/"),
                s3_files_bucket: str!("files"),
                s3_tblocks_bucket: str!("text-blocks"),
                s3_region: Region::Custom {
                    region: str!("test"),
                    endpoint: endpoint.to_string(),
                },
                s3_credentials: Credentials::new(
                    Some("access-key"),
                    Some("secret-key"),
                    None,
                    None,
                    None,
                )
                .unwrap(),
                s3_path_style: true,
            },
        )
        .await
        .unwrap();
        disable_s3_proxies(&mut state);
        state
    }

    fn disable_s3_proxies(state: &mut ServerState) {
        let state = Arc::get_mut(state).expect("test state should have one owner");
        let files_bucket = state
            .s3_files_bucket
            .set_proxy(reqwest::Proxy::custom(|_| None::<reqwest::Url>))
            .unwrap();
        let tblocks_bucket = state
            .s3_tblocks_bucket
            .set_proxy(reqwest::Proxy::custom(|_| None::<reqwest::Url>))
            .unwrap();
        *state.s3_files_bucket = files_bucket;
        *state.s3_tblocks_bucket = tblocks_bucket;
    }

    #[tokio::test]
    async fn fetch_full_body_streams_successful_s3_object() {
        let server = spawn_s3_server(vec![(200, b"abcdef")]);
        let state = test_state(&server.endpoint).await;
        let headers = HeaderMap::new();

        let body =
            fetch_full_body(&state, &headers, 10, &file_data(), "page", "file.txt")
                .await
                .unwrap();

        let body = body::to_bytes(body, usize::MAX).await.unwrap();
        assert_eq!(&body[..], b"abcdef");
        let requests = server.requests();
        assert!(requests[0].starts_with("GET /files/sha512-hash "));
        server.join();
    }

    #[tokio::test]
    async fn fetch_range_stream_sets_range_header_and_requires_partial_content() {
        let server = spawn_s3_server(vec![(206, b"bcd")]);
        let state = test_state(&server.endpoint).await;

        let body =
            fetch_range_stream(&state, &file_data(), ByteRange { start: 1, end: 3 })
                .await
                .unwrap();

        let body = body::to_bytes(body, usize::MAX).await.unwrap();
        assert_eq!(&body[..], b"bcd");
        let requests = server.requests();
        assert!(
            requests[0]
                .to_ascii_lowercase()
                .contains("range: bytes=1-3"),
        );
        server.join();
    }

    #[tokio::test]
    async fn fetch_range_stream_rejects_non_partial_response() {
        let server = spawn_s3_server(vec![(200, b"abcdef")]);
        let state = test_state(&server.endpoint).await;

        let error =
            fetch_range_stream(&state, &file_data(), ByteRange { start: 1, end: 3 })
                .await
                .unwrap_err();

        assert!(format!("{error}").contains("expected 206, got 200"));
        server.join();
    }

    #[tokio::test]
    async fn fetch_range_bytes_returns_partial_body() {
        let server = spawn_s3_server(vec![(206, b"cde")]);
        let state = test_state(&server.endpoint).await;

        let bytes =
            fetch_range_bytes(&state, &file_data(), ByteRange { start: 2, end: 4 })
                .await
                .unwrap();

        assert_eq!(bytes, b"cde");
        let requests = server.requests();
        assert!(
            requests[0]
                .to_ascii_lowercase()
                .contains("range: bytes=2-4"),
        );
        server.join();
    }

    #[tokio::test]
    async fn fetch_range_bytes_rejects_non_partial_response() {
        let server = spawn_s3_server(vec![(200, b"abcdef")]);
        let state = test_state(&server.endpoint).await;

        let error =
            fetch_range_bytes(&state, &file_data(), ByteRange { start: 2, end: 4 })
                .await
                .unwrap_err();

        assert!(format!("{error}").contains("expected 206, got 200"));
        server.join();
    }
}
