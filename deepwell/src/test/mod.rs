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
    pub use serde_json::{json, Value as JsonValue};
    pub use tide::Result;

    use serde::Serialize;

    pub const WWW_SITE_ID: i64 = 1;
    pub const EN_TEMPLATE_SITE_ID: i64 = 2;

    pub const ADMIN_USER_ID: i64 = 1;
    pub const AUTOMATIC_USER_ID: i64 = 2;
    pub const ANONYMOUS_USER_ID: i64 = 3;
    pub const REGULAR_USER_ID: i64 = 4;
}

mod misc;
mod page;

use crate::api::{self, ApiServer};
use crate::config::Config;
use serde::Serialize;
use tide::convert::DeserializeOwned;
use tide::http::{Method, Request, Url};
use tide::{Body, Response, Result};

macro_rules! impl_request_method {
    ($method_enum:ident, $method_name:ident) => {
        #[inline]
        #[allow(dead_code)]
        pub fn $method_name<'a>(&'a self, route: &str) -> Result<RequestBuilder<'a>> {
            RequestBuilder::new(&self.app, Method::$method_enum, route)
        }
    };
}

macro_rules! impl_recv_method {
    ($self:expr, $into_method:ident) => {{
        let mut response = $self.recv().await?;
        let data = response.take_body().$into_method().await?;
        Ok(data)
    }};
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
        let app = api::build_server(config).await?;

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
        let request = Request::new(method, url);
        Ok(RequestBuilder { app, request })
    }

    pub fn body_bytes<B: Into<Vec<u8>>>(mut self, bytes: B) -> Self {
        let body = Body::from_bytes(bytes.into());
        self.request.set_body(body);
        self
    }

    pub fn body_string<S: Into<String>>(mut self, string: S) -> Self {
        let body = Body::from_string(string.into());
        self.request.set_body(body);
        self
    }

    pub fn body_json(mut self, data: &impl Serialize) -> Result<Self> {
        let body = Body::from_json(data)?;
        self.request.set_body(body);
        Ok(self)
    }

    pub async fn recv(self) -> Result<Response> {
        self.app.respond(self.request).await
    }

    pub async fn recv_bytes(self) -> Result<Vec<u8>> {
        impl_recv_method!(self, into_bytes)
    }

    pub async fn recv_string(self) -> Result<String> {
        impl_recv_method!(self, into_string)
    }

    pub async fn recv_json<T: DeserializeOwned>(self) -> Result<T> {
        impl_recv_method!(self, into_json)
    }
}
