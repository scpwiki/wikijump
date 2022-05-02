/*
 * test/suite/request.rs
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

use crate::api::ApiServer;
use serde::Serialize;
use serde_json::Value as JsonValue;
use tide::convert::DeserializeOwned;
use tide::http::{Method, Request, Url};
use tide::{Body, Response, Result, StatusCode};

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
