/*
 * test/mod.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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

#[allow(dead_code)]
mod prelude {
    pub use super::TestEnvironment;
    pub use async_std_test::async_test;
    pub use serde_json::{json, Value as JsonValue};
    pub use tide::{Result, StatusCode};

    pub const WWW_SITE_ID: i64 = 1;
    pub const EN_TEMPLATE_SITE_ID: i64 = 2;

    pub const ADMIN_USER_ID: i64 = 1;
    pub const AUTOMATIC_USER_ID: i64 = 2;
    pub const ANONYMOUS_USER_ID: i64 = 3;
    pub const REGULAR_USER_ID: i64 = 4;
}

mod locale;
mod misc;
mod page;

use crate::api::{self, ApiServer};
use crate::config::Config;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use serde::Serialize;
use serde_json::Value as JsonValue;
use tide::convert::DeserializeOwned;
use tide::http::{Method, Request, Url};
use tide::{Body, Response, Result, StatusCode};

macro_rules! impl_request_method {
    ($method_enum:ident, $method_name:ident) => {
        #[inline]
        #[allow(dead_code)]
        pub fn $method_name<'a, S: AsRef<str>>(
            &'a self,
            route: S,
        ) -> Result<RequestBuilder<'a>> {
            RequestBuilder::new(&self.app, Method::$method_enum, route.as_ref())
        }
    };
}

#[derive(Debug)]
pub struct TestEnvironment {
    pub app: ApiServer,
}

impl TestEnvironment {
    pub async fn setup() -> Result<Self> {
        // The Default impl is different in the test environment
        let config = Config::load();

        // Build API server
        crate::setup(&config).await?;
        let app = api::build_internal_api(config).await?;

        // Build and return
        Ok(TestEnvironment { app })
    }

    impl_request_method!(Get, get);
    impl_request_method!(Put, put);
    impl_request_method!(Post, post);
    impl_request_method!(Delete, delete);
    impl_request_method!(Head, head);
    impl_request_method!(Connect, connect);
    impl_request_method!(Options, options);
    impl_request_method!(Trace, trace);
    impl_request_method!(Patch, patch);

    #[inline]
    pub fn random_slug(&self) -> String {
        let mut slug = self.random_name_with_prefix("test-");
        slug.make_ascii_lowercase();
        slug
    }

    pub fn random_name_with_prefix(&self, prefix: &str) -> String {
        let mut slug = String::from(prefix);
        let mut rng = thread_rng();

        for _ in 0..20 {
            slug.push(rng.sample(Alphanumeric) as char);
        }

        slug
    }
}

#[derive(Debug)]
pub struct RequestBuilder<'a> {
    app: &'a ApiServer,
    request: Request,
}

impl<'a> RequestBuilder<'a> {
    pub fn new(app: &'a ApiServer, method: Method, route: &str) -> Result<Self> {
        assert!(route.starts_with('/'), "Route doesn't start with /");

        let url = Url::parse(&format!("https://test.example.com{route}"))?;
        let mut request = Request::new(method, url);
        request.insert_header("accept", "*/*");
        request.insert_header("user-agent", "deepwell/test");
        Ok(RequestBuilder { app, request })
    }

    #[allow(dead_code)]
    pub fn body_bytes<B: Into<Vec<u8>>>(mut self, bytes: B) -> Self {
        let body = Body::from_bytes(bytes.into());
        self.request.set_body(body);
        self
    }

    #[allow(dead_code)]
    pub fn body_string<S: Into<String>>(mut self, string: S) -> Self {
        let body = Body::from_string(string.into());
        self.request.set_body(body);
        self
    }

    #[allow(dead_code)]
    pub fn body_json<T: Serialize>(mut self, data: T) -> Result<Self> {
        let body = Body::from_json(&data)?;
        self.request.set_body(body);
        Ok(self)
    }

    pub async fn send(self) -> Result<Response> {
        self.app.respond(self.request).await
    }

    pub async fn recv(self) -> Result<StatusCode> {
        let response = self.send().await?;
        Ok(response.status())
    }

    pub async fn recv_string(self) -> Result<(String, StatusCode)> {
        let mut response = self.send().await?;
        let status = response.status();
        let body = response.take_body().into_string().await?;
        Ok((body, status))
    }

    pub async fn recv_json<T: DeserializeOwned>(self) -> Result<(T, StatusCode)> {
        let mut response = self.send().await?;
        let status = response.status();
        let body = response.take_body();

        // Special handling if empty, probably error
        // and a serde error is unhelpful.
        //
        // A panic here will give a more useful traceback
        // that we can follow.
        if body.is_empty().unwrap_or(true) {
            panic!(
                "Response body is empty in recv_json() (status {} {:?})",
                status, status,
            );
        }

        let output = body.into_json().await?;
        Ok((output, status))
    }

    #[allow(dead_code)]
    pub async fn recv_json_value(self) -> Result<(JsonValue, StatusCode)> {
        self.recv_json().await
    }
}
