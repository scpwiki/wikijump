/*
 * deepwell.rs
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

use crate::error::{Result, TextBlockErrorReason};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::rpc_params;
use serde::Deserialize;
use std::num::NonZeroU16;
use std::time::Duration;

const JSONRPC_MAX_REQUEST: u32 = 16 * 1024;
const JSONRPC_TIMEOUT: Duration = Duration::from_secs(5);

/// Macro to create `ObjectParams` instances.
/// This is the object equivalent to `rpc_params!`, which creates `ArrayParams` instances.
macro_rules! rpc_object {
    ($($key:expr => $value:expr,)+) => { rpc_object!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {{
        use jsonrpsee::core::params::ObjectParams;

        let mut params = ObjectParams::new();
        $(
            if let Err(error) = params.insert($key, $value) {
                panic!("Parameter `{}` cannot be serialized: {:?}", stringify!($key), error);
            }
        )*
        params
    }};
}

#[derive(Debug)]
pub struct Deepwell {
    client: HttpClient,
}

impl Deepwell {
    pub fn connect(deepwell_url: &str) -> Result<Self> {
        let client = HttpClient::builder()
            .max_request_size(JSONRPC_MAX_REQUEST)
            .request_timeout(JSONRPC_TIMEOUT)
            .build(deepwell_url)?;

        Ok(Deepwell { client })
    }

    /// Attempt to ping DEEPWELL, panicking if connecting failed.
    pub async fn check(&self) {
        self.ping().await.expect("Unable to connect to DEEPWELL");
    }

    pub async fn ping(&self) -> Result<()> {
        let response: String = self.client.request("ping", rpc_params![]).await?;
        assert!(!response.is_empty());
        debug!("Successfully pinged DEEPWELL");
        Ok(())
    }

    // Getters

    pub async fn get_site_domain(&self, site_id: i64) -> Result<String> {
        let params = rpc_params![site_id];
        let domain: String = self.client.request("site_domain", params).await?;
        Ok(domain)
    }

    pub async fn get_page(
        &self,
        site_id: i64,
        page_slug: &str,
    ) -> Result<Option<PageData>> {
        let params = rpc_object! {
            "site_id" => site_id,
            "page" => page_slug,
            "wikitext" => false,
            "compiled" => false,
        };

        let page_data: Option<PageData> = self.client.request("page_get", params).await?;
        Ok(page_data)
    }

    pub async fn get_file(
        &self,
        site_id: i64,
        page_id: i64,
        filename: &str,
    ) -> Result<Option<FileData>> {
        let params = rpc_object! {
            "site_id" => site_id,
            "page_id" => page_id,
            "file" => filename,
            "data" => false,
        };

        let file_data: Option<FileData> = self.client.request("file_get", params).await?;
        Ok(file_data)
    }

    pub async fn get_text_block_index(
        &self,
        site_id: i64,
        page_id: i64,
        block_type: TextBlockType,
        block_id: TextBlockId<'_>,
        session_token: Option<&str>,
    ) -> Result<Option<TextBlockIndex>> {
        let (index, name) = match block_id {
            TextBlockId::Index(index) => (Some(index.get()), None),
            TextBlockId::Name(name) => (None, Some(name)),
        };
        let params = rpc_object! {
            "site_id" => site_id,
            "page_id" => page_id,
            "block_type" => block_type.value(),
            "index" => index,
            "name" => name,
            "session_token" => session_token,
        };

        let block_info: Option<TextBlockIndex> =
            self.client.request("text_block_get_index", params).await?;

        Ok(block_info)
    }

    // Basic errors

    pub async fn basic_error_missing_site_slug(
        &self,
        locales: &[String],
        site_slug: &str,
    ) -> Result<BasicErrorHtml> {
        let params = rpc_object! {
            "locales" => locales,
            "site_slug" => site_slug,
        };

        let html: BasicErrorHtml = self
            .client
            .request("basic_error_missing_site_slug", params)
            .await?;

        Ok(html)
    }

    pub async fn basic_error_missing_custom_domain(
        &self,
        locales: &[String],
        domain: &str,
    ) -> Result<BasicErrorHtml> {
        let params = rpc_object! {
            "locales" => locales,
            "domain" => domain,
        };

        let html: BasicErrorHtml = self
            .client
            .request("basic_error_missing_custom_domain", params)
            .await?;

        Ok(html)
    }

    pub async fn basic_error_missing_page_slug(
        &self,
        locales: &[String],
        site_id: i64,
        page_slug: &str,
    ) -> Result<BasicErrorHtml> {
        let params = rpc_object! {
            "locales" => locales,
            "site_id" => site_id,
            "page_slug" => page_slug,
        };

        let html: BasicErrorHtml = self
            .client
            .request("basic_error_missing_page_slug", params)
            .await?;

        Ok(html)
    }

    pub async fn basic_error_page_fetch(
        &self,
        locales: &[String],
        site_id: i64,
        page_slug: &str,
    ) -> Result<BasicErrorHtml> {
        let params = rpc_object! {
            "locales" => locales,
            "site_id" => site_id,
            "page_slug" => page_slug,
        };

        let html: BasicErrorHtml = self
            .client
            .request("basic_error_page_fetch", params)
            .await?;

        Ok(html)
    }

    pub async fn basic_error_missing_file_name(
        &self,
        locales: &[String],
        site_id: i64,
        page_slug: &str,
        filename: &str,
    ) -> Result<BasicErrorHtml> {
        let params = rpc_object! {
            "locales" => locales,
            "site_id" => site_id,
            "page_slug" => page_slug,
            "filename" => filename,
        };

        let html: BasicErrorHtml = self
            .client
            .request("basic_error_missing_file_name", params)
            .await?;

        Ok(html)
    }

    pub async fn basic_error_file_fetch(
        &self,
        locales: &[String],
        site_id: i64,
        page_slug: &str,
        filename: &str,
    ) -> Result<BasicErrorHtml> {
        let params = rpc_object! {
            "locales" => locales,
            "site_id" => site_id,
            "page_slug" => page_slug,
            "filename" => filename,
        };

        let html: BasicErrorHtml = self
            .client
            .request("basic_error_file_fetch", params)
            .await?;

        Ok(html)
    }

    pub async fn basic_error_text_block(
        &self,
        locales: &[String],
        site_id: i64,
        index: &str,
        block_type: TextBlockType,
        reason: TextBlockErrorReason,
    ) -> Result<BasicErrorHtml> {
        let params = rpc_object! {
            "locales" => locales,
            "site_id" => site_id,
            "index" => index,
            "block_type" => block_type.value(),
            "reason" => reason.value(),
        };

        let html: BasicErrorHtml = self
            .client
            .request("basic_error_text_block", params)
            .await?;

        Ok(html)
    }

    pub async fn basic_error_file_root(
        &self,
        locales: &[String],
    ) -> Result<BasicErrorHtml> {
        let params = rpc_object! {
            "locales" => locales,
        };

        let html: BasicErrorHtml =
            self.client.request("basic_error_file_root", params).await?;

        Ok(html)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct PageData {
    pub page_id: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FileData {
    pub file_id: i64,
    pub mime: String,
    pub size: i64,
    pub s3_hash: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TextBlockIndex {
    pub index: NonZeroU16,
    pub s3_filename: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BasicErrorHtml {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Copy, Clone)]
pub enum TextBlockType {
    Code,
    Html,
}

#[derive(Debug, Copy, Clone)]
pub enum TextBlockId<'a> {
    Index(NonZeroU16),
    Name(&'a str),
}

impl TextBlockType {
    #[inline]
    pub fn value(self) -> &'static str {
        match self {
            TextBlockType::Code => "code",
            TextBlockType::Html => "html",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::body::Bytes;
    use axum::extract::State;
    use axum::http::header;
    use axum::response::{IntoResponse, Response};
    use axum::routing::post;
    use serde_json::{Value, json};
    use std::sync::{Arc, Mutex};
    use tokio::net::TcpListener;

    type Requests = Arc<Mutex<Vec<Value>>>;

    async fn rpc_handler(State(requests): State<Requests>, body: Bytes) -> Response {
        let request: Value = serde_json::from_slice(&body).unwrap();
        let method = request["method"].as_str().unwrap();
        let id = request["id"].clone();
        let result = match method {
            "ping" => json!("pong"),
            "site_domain" => json!("scp-wiki.wikijump.local"),
            "page_get" => json!({ "page_id": 123 }),
            "file_get" => json!({
                "file_id": 7,
                "mime": "text/plain",
                "size": 42,
                "s3_hash": "abc123",
            }),
            "text_block_get_index" => json!({
                "index": 2,
                "s3_filename": "blocks/2.html",
            }),
            "basic_error_missing_site_slug"
            | "basic_error_missing_custom_domain"
            | "basic_error_missing_page_slug"
            | "basic_error_page_fetch"
            | "basic_error_missing_file_name"
            | "basic_error_file_fetch"
            | "basic_error_text_block"
            | "basic_error_file_root" => json!({
                "title": format!("title:{method}"),
                "body": format!("body:{method}"),
            }),
            other => panic!("unexpected JSON-RPC method: {other}"),
        };

        requests.lock().unwrap().push(request);
        let body = json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": id,
        })
        .to_string();

        ([(header::CONTENT_TYPE, "application/json")], body).into_response()
    }

    async fn spawn_rpc_server() -> (String, Requests) {
        let requests = Requests::default();
        let app = Router::new()
            .route("/", post(rpc_handler))
            .with_state(Arc::clone(&requests));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{address}"), requests)
    }

    fn requests_by_method(requests: &Requests, method: &str) -> Vec<Value> {
        requests
            .lock()
            .unwrap()
            .iter()
            .filter(|request| request["method"] == method)
            .cloned()
            .collect()
    }

    #[test]
    fn text_block_type_values_match_deepwell_contract() {
        assert_eq!(TextBlockType::Code.value(), "code");
        assert_eq!(TextBlockType::Html.value(), "html");
    }

    #[test]
    fn invalid_deepwell_url_is_rejected() {
        assert!(Deepwell::connect("not a url").is_err());
    }

    #[tokio::test]
    async fn deepwell_getters_send_expected_json_rpc_requests() {
        let (url, requests) = spawn_rpc_server().await;
        let deepwell = Deepwell::connect(&url).unwrap();

        deepwell.ping().await.unwrap();
        assert_eq!(
            deepwell.get_site_domain(42).await.unwrap(),
            "scp-wiki.wikijump.local",
        );
        assert_eq!(
            deepwell
                .get_page(42, "scp-173")
                .await
                .unwrap()
                .unwrap()
                .page_id,
            123
        );

        let file = deepwell
            .get_file(42, 123, "image.png")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(file.file_id, 7);
        assert_eq!(file.mime, "text/plain");
        assert_eq!(file.size, 42);
        assert_eq!(file.s3_hash, "abc123");

        let block = deepwell
            .get_text_block_index(
                42,
                123,
                TextBlockType::Html,
                TextBlockId::Index(NonZeroU16::new(2).unwrap()),
                Some("session-token"),
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(block.index, NonZeroU16::new(2).unwrap());
        assert_eq!(block.s3_filename, "blocks/2.html");

        let named_block = deepwell
            .get_text_block_index(
                42,
                123,
                TextBlockType::Code,
                TextBlockId::Name("example"),
                None,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(named_block.index, NonZeroU16::new(2).unwrap());

        let page_get = requests_by_method(&requests, "page_get");
        assert_eq!(page_get[0]["params"]["site_id"], 42);
        assert_eq!(page_get[0]["params"]["page"], "scp-173");
        assert_eq!(page_get[0]["params"]["wikitext"], false);
        assert_eq!(page_get[0]["params"]["compiled"], false);

        let file_get = requests_by_method(&requests, "file_get");
        assert_eq!(file_get[0]["params"]["site_id"], 42);
        assert_eq!(file_get[0]["params"]["page_id"], 123);
        assert_eq!(file_get[0]["params"]["file"], "image.png");
        assert_eq!(file_get[0]["params"]["data"], false);

        let block_gets = requests_by_method(&requests, "text_block_get_index");
        assert_eq!(block_gets[0]["params"]["block_type"], "html");
        assert_eq!(block_gets[0]["params"]["index"], 2);
        assert!(block_gets[0]["params"]["name"].is_null());
        assert_eq!(block_gets[0]["params"]["session_token"], "session-token");
        assert_eq!(block_gets[1]["params"]["block_type"], "code");
        assert!(block_gets[1]["params"]["index"].is_null());
        assert_eq!(block_gets[1]["params"]["name"], "example");
        assert!(block_gets[1]["params"]["session_token"].is_null());
    }

    #[tokio::test]
    async fn deepwell_basic_error_methods_send_locales_and_context() {
        let (url, requests) = spawn_rpc_server().await;
        let deepwell = Deepwell::connect(&url).unwrap();
        let locales = vec!["ja".to_string(), "en".to_string()];

        assert_eq!(
            deepwell
                .basic_error_missing_site_slug(&locales, "scp-wiki")
                .await
                .unwrap()
                .title,
            "title:basic_error_missing_site_slug",
        );
        assert_eq!(
            deepwell
                .basic_error_missing_custom_domain(&locales, "example.com")
                .await
                .unwrap()
                .title,
            "title:basic_error_missing_custom_domain",
        );
        assert_eq!(
            deepwell
                .basic_error_missing_page_slug(&locales, 42, "scp-173")
                .await
                .unwrap()
                .title,
            "title:basic_error_missing_page_slug",
        );
        assert_eq!(
            deepwell
                .basic_error_page_fetch(&locales, 42, "scp-173")
                .await
                .unwrap()
                .title,
            "title:basic_error_page_fetch",
        );
        assert_eq!(
            deepwell
                .basic_error_missing_file_name(&locales, 42, "scp-173", "image.png")
                .await
                .unwrap()
                .title,
            "title:basic_error_missing_file_name",
        );
        assert_eq!(
            deepwell
                .basic_error_file_fetch(&locales, 42, "scp-173", "image.png")
                .await
                .unwrap()
                .title,
            "title:basic_error_file_fetch",
        );
        let text_block = deepwell
            .basic_error_text_block(
                &locales,
                42,
                "2",
                TextBlockType::Html,
                TextBlockErrorReason::Missing,
            )
            .await
            .unwrap();
        assert_eq!(text_block.title, "title:basic_error_text_block");
        assert_eq!(text_block.body, "body:basic_error_text_block");
        assert_eq!(
            deepwell
                .basic_error_file_root(&locales)
                .await
                .unwrap()
                .title,
            "title:basic_error_file_root",
        );

        let site_slug = requests_by_method(&requests, "basic_error_missing_site_slug");
        assert_eq!(site_slug[0]["params"]["locales"], json!(["ja", "en"]));
        assert_eq!(site_slug[0]["params"]["site_slug"], "scp-wiki");

        let custom_domain =
            requests_by_method(&requests, "basic_error_missing_custom_domain");
        assert_eq!(custom_domain[0]["params"]["domain"], "example.com");

        let text_block_requests = requests_by_method(&requests, "basic_error_text_block");
        assert_eq!(text_block_requests[0]["params"]["site_id"], 42);
        assert_eq!(text_block_requests[0]["params"]["index"], "2");
        assert_eq!(text_block_requests[0]["params"]["block_type"], "html");
        assert_eq!(text_block_requests[0]["params"]["reason"], "missing");
    }
}
