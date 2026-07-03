/*
 * error/basic.rs
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

//! See the `BasicErrorService` in DEEPWELL for information.

use crate::deepwell::TextBlockType;
use crate::error::FallbackError;
use crate::language::parse_accept_language;
use crate::state::ServerStateInner;
use axum::body::Body;
use axum::http::header::{self, HeaderMap};
use axum::http::status::StatusCode;
use axum::response::{IntoResponse, Response};
use paste::paste;

pub use crate::deepwell::BasicErrorHtml;

#[derive(Debug, Copy, Clone)]
pub enum BasicError<'a> {
    SiteSlug {
        site_slug: &'a str,
    },
    SiteCustom {
        host: &'a str,
    },
    PageSlug {
        site_id: i64,
        page_slug: &'a str,
    },
    PageFetch {
        site_id: i64,
        page_slug: &'a str,
    },
    FileName {
        site_id: i64,
        page_slug: &'a str,
        filename: &'a str,
    },
    FileFetch {
        site_id: i64,
        page_slug: &'a str,
        filename: &'a str,
    },
    TextBlock {
        site_id: i64,
        index: &'a str,
        block_type: TextBlockType,
        reason: TextBlockErrorReason,
    },
    FileRoot,
}

#[derive(Debug)]
pub struct BasicErrorOutput {
    pub title: String,
    pub body: String,
    pub status: StatusCode,
}

impl BasicErrorOutput {
    fn into_response(self) -> Response {
        let BasicErrorOutput {
            title,
            body,
            status,
        } = self;

        // SAFETY: Both string fields here come from DEEPWELL,
        //         which in turn come from Fluent translation lines.
        //         As such, they can be trusted to not contain malicious HTML.

        const HTML_START: &str = r#"<html><head><meta name="viewport" content="width=device-width, initial-scale=1.0"/><title>"#;
        const HTML_MIDDLE: &str = "</title></head><body><article>";
        const HTML_END: &str = "</article></body></html>\n";

        let html = format!("{HTML_START}{title}{HTML_MIDDLE}{body}{HTML_END}");
        Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(html))
            .expect("Unable to convert response data")
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum TextBlockErrorReason {
    /// This hosted text block does not exist.
    Missing,

    /// The URL to this hosted text block is invalid.
    Invalid,

    /// The server was unable to retrieve this hosted text block.
    Fetch,
}

impl TextBlockErrorReason {
    #[inline]
    pub fn value(self) -> &'static str {
        // These must match the values in the Fluent files.
        match self {
            TextBlockErrorReason::Missing => "missing",
            TextBlockErrorReason::Invalid => "invalid",
            TextBlockErrorReason::Fetch => "fetch",
        }
    }
}

pub async fn build_basic_error_response(
    // NOTE: We need to accept the inner struct specifically here, since there are
    //       some places in state.rs itself where we need to call this function.
    state: &ServerStateInner,
    headers: &HeaderMap,
    basic_error: BasicError<'_>,
) -> Response {
    // Get a list of preferred locales from the Accept-Language header.
    let locales = parse_accept_language(headers);

    // Build the appropriate error case

    macro_rules! deepwell_fetch {
        ($method:ident => $status_code:ident $(,)?) => {
            deepwell_fetch!($method, => $status_code)
        };
        ($method:ident, $($arg:expr),* => $status_code:ident $(,)?) => {
            deepwell_fetch!($method, $($arg),* ; StatusCode::$status_code)
        };
        ($method:ident, $($arg:expr),* ; $status_code:expr $(,)?) => {{
            paste! {
                let result = state.deepwell.[<basic_error_ $method>](&locales, $($arg),*).await;
            }

            match result {
                Ok(BasicErrorHtml { title, body }) => {
                    BasicErrorOutput { title, body, status: $status_code }
                }
                Err(error) => {
                    // XF-1001
                    error!(
                        "Unable to get basic error for {}: {}",
                        stringify!($method),
                        error,
                    );
                    return FallbackError::BasicErrorFetch.into_response();
                }
            }
        }};
    }

    let output = match basic_error {
        BasicError::SiteSlug { site_slug } => {
            deepwell_fetch!(missing_site_slug, site_slug => NOT_FOUND)
        }
        BasicError::SiteCustom { host } => {
            deepwell_fetch!(missing_custom_domain, host => NOT_FOUND)
        }
        BasicError::PageSlug { site_id, page_slug } => {
            deepwell_fetch!(missing_page_slug, site_id, page_slug => NOT_FOUND)
        }
        BasicError::PageFetch { site_id, page_slug } => {
            deepwell_fetch!(page_fetch, site_id, page_slug => INTERNAL_SERVER_ERROR)
        }
        BasicError::FileName {
            site_id,
            page_slug,
            filename,
        } => {
            deepwell_fetch!(missing_file_name, site_id, page_slug, filename => NOT_FOUND)
        }
        BasicError::FileFetch {
            site_id,
            page_slug,
            filename,
        } => {
            deepwell_fetch!(file_fetch, site_id, page_slug, filename => INTERNAL_SERVER_ERROR)
        }
        BasicError::TextBlock {
            site_id,
            index,
            block_type,
            reason,
        } => {
            let status_code = match reason {
                TextBlockErrorReason::Missing => StatusCode::NOT_FOUND,
                TextBlockErrorReason::Invalid => StatusCode::BAD_REQUEST,
                TextBlockErrorReason::Fetch => StatusCode::INTERNAL_SERVER_ERROR,
            };

            deepwell_fetch!(text_block, site_id, index, block_type, reason; status_code)
        }
        BasicError::FileRoot => {
            deepwell_fetch!(file_root => BAD_REQUEST)
        }
    };

    output.into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::Cache;
    use crate::deepwell::Deepwell;
    use axum::Router;
    use axum::body;
    use axum::body::Bytes;
    use axum::extract::State;
    use axum::http::header::ACCEPT_LANGUAGE;
    use axum::routing::post;
    use s3::bucket::Bucket;
    use s3::creds::Credentials;
    use s3::region::Region;
    use serde_json::{Value, json};
    use std::sync::{Arc, Mutex};
    use tokio::net::TcpListener;

    type Requests = Arc<Mutex<Vec<Value>>>;

    #[derive(Clone)]
    struct RpcState {
        requests: Requests,
        fail: bool,
    }

    async fn rpc_handler(State(state): State<RpcState>, body: Bytes) -> Response {
        let request: Value = serde_json::from_slice(&body).unwrap();
        let method = request["method"].as_str().unwrap().to_owned();
        let id = request["id"].clone();
        state.requests.lock().unwrap().push(request);

        let body = if state.fail {
            json!({
                "jsonrpc": "2.0",
                "error": { "code": -32000, "message": "basic error unavailable" },
                "id": id,
            })
        } else {
            json!({
                "jsonrpc": "2.0",
                "result": {
                    "title": format!("title:{method}"),
                    "body": format!("body:{method}"),
                },
                "id": id,
            })
        }
        .to_string();

        ([(header::CONTENT_TYPE, "application/json")], body).into_response()
    }

    async fn spawn_rpc_server(fail: bool) -> (String, Requests) {
        let requests = Requests::default();
        let app = Router::new()
            .route("/", post(rpc_handler))
            .with_state(RpcState {
                requests: Arc::clone(&requests),
                fail,
            });
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{address}"), requests)
    }

    fn test_bucket(name: &str) -> Box<Bucket> {
        Bucket::new(
            name,
            Region::Custom {
                region: str!("test"),
                endpoint: str!("http://127.0.0.1:9000"),
            },
            Credentials::new(Some("access-key"), Some("secret-key"), None, None, None)
                .unwrap(),
        )
        .unwrap()
        .with_path_style()
    }

    fn test_state(deepwell_url: &str) -> ServerStateInner {
        ServerStateInner {
            deepwell: Deepwell::connect(deepwell_url).unwrap(),
            cache: Cache::connect("redis://127.0.0.1/").unwrap(),
            s3_files_bucket: test_bucket("files"),
            s3_tblocks_bucket: test_bucket("text-blocks"),
        }
    }

    async fn assert_basic_response(response: Response, status: StatusCode, method: &str) {
        assert_eq!(response.status(), status);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8",
        );

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains(&format!("<title>title:{method}</title>")));
        assert!(body.contains(&format!("body:{method}")));
    }

    #[test]
    fn text_block_error_reasons_match_deepwell_fluent_values() {
        assert_eq!(TextBlockErrorReason::Missing.value(), "missing");
        assert_eq!(TextBlockErrorReason::Invalid.value(), "invalid");
        assert_eq!(TextBlockErrorReason::Fetch.value(), "fetch");
    }

    #[tokio::test]
    async fn basic_error_output_builds_html_response() {
        let response = BasicErrorOutput {
            title: "Missing page".to_string(),
            body: "<p>No such page.</p>".to_string(),
            status: StatusCode::NOT_FOUND,
        }
        .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8",
        );

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(
            &body[..],
            br#"<html><head><meta name="viewport" content="width=device-width, initial-scale=1.0"/><title>Missing page</title></head><body><article><p>No such page.</p></article></body></html>
"#,
        );
    }

    #[tokio::test]
    async fn build_basic_error_response_fetches_localized_deepwell_errors() {
        let (url, requests) = spawn_rpc_server(false).await;
        let state = test_state(&url);
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT_LANGUAGE, "ja, en;q=0.5".parse().unwrap());

        assert_basic_response(
            build_basic_error_response(
                &state,
                &headers,
                BasicError::SiteSlug {
                    site_slug: "scp-wiki",
                },
            )
            .await,
            StatusCode::NOT_FOUND,
            "basic_error_missing_site_slug",
        )
        .await;
        assert_basic_response(
            build_basic_error_response(
                &state,
                &headers,
                BasicError::SiteCustom {
                    host: "example.com",
                },
            )
            .await,
            StatusCode::NOT_FOUND,
            "basic_error_missing_custom_domain",
        )
        .await;
        assert_basic_response(
            build_basic_error_response(
                &state,
                &headers,
                BasicError::PageSlug {
                    site_id: 42,
                    page_slug: "scp-173",
                },
            )
            .await,
            StatusCode::NOT_FOUND,
            "basic_error_missing_page_slug",
        )
        .await;
        assert_basic_response(
            build_basic_error_response(
                &state,
                &headers,
                BasicError::PageFetch {
                    site_id: 42,
                    page_slug: "scp-173",
                },
            )
            .await,
            StatusCode::INTERNAL_SERVER_ERROR,
            "basic_error_page_fetch",
        )
        .await;
        assert_basic_response(
            build_basic_error_response(
                &state,
                &headers,
                BasicError::FileName {
                    site_id: 42,
                    page_slug: "scp-173",
                    filename: "image.png",
                },
            )
            .await,
            StatusCode::NOT_FOUND,
            "basic_error_missing_file_name",
        )
        .await;
        assert_basic_response(
            build_basic_error_response(
                &state,
                &headers,
                BasicError::FileFetch {
                    site_id: 42,
                    page_slug: "scp-173",
                    filename: "image.png",
                },
            )
            .await,
            StatusCode::INTERNAL_SERVER_ERROR,
            "basic_error_file_fetch",
        )
        .await;
        for (reason, status) in [
            (TextBlockErrorReason::Missing, StatusCode::NOT_FOUND),
            (TextBlockErrorReason::Invalid, StatusCode::BAD_REQUEST),
            (
                TextBlockErrorReason::Fetch,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        ] {
            assert_basic_response(
                build_basic_error_response(
                    &state,
                    &headers,
                    BasicError::TextBlock {
                        site_id: 42,
                        index: "2",
                        block_type: TextBlockType::Html,
                        reason,
                    },
                )
                .await,
                status,
                "basic_error_text_block",
            )
            .await;
        }
        assert_basic_response(
            build_basic_error_response(&state, &headers, BasicError::FileRoot).await,
            StatusCode::BAD_REQUEST,
            "basic_error_file_root",
        )
        .await;

        let requests = requests.lock().unwrap();
        assert_eq!(
            requests[0]["params"]["locales"],
            json!(["ja", "en"]),
            "Accept-Language should be forwarded to DEEPWELL",
        );
        let text_block_requests = requests
            .iter()
            .filter(|request| request["method"] == "basic_error_text_block")
            .collect::<Vec<_>>();
        assert_eq!(text_block_requests[0]["params"]["reason"], "missing");
        assert_eq!(text_block_requests[1]["params"]["reason"], "invalid");
        assert_eq!(text_block_requests[2]["params"]["reason"], "fetch");
    }

    #[tokio::test]
    async fn build_basic_error_response_falls_back_when_deepwell_errors() {
        let (url, _requests) = spawn_rpc_server(true).await;
        let state = test_state(&url);
        let response =
            build_basic_error_response(&state, &HeaderMap::new(), BasicError::FileRoot)
                .await;

        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"ERROR XF-1001");
    }
}
