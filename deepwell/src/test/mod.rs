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
    pub use super::{setup, RequestBuilderExt};
    pub use serde_json::{json, Value as JsonValue};
    pub use tide::{Body, Result, StatusCode};
    pub use tide_testing::TideTestingExt;

    use serde::Serialize;

    pub const WWW_SITE_ID: i64 = 1;
    pub const EN_TEMPLATE_SITE_ID: i64 = 2;

    pub const ADMIN_USER_ID: i64 = 1;
    pub const AUTOMATIC_USER_ID: i64 = 2;
    pub const ANONYMOUS_USER_ID: i64 = 3;
    pub const REGULAR_USER_ID: i64 = 4;

    #[inline]
    pub fn create_body<T: Serialize>(data: T) -> Body {
        Body::from_json(&data).expect("Unable to create JSON body")
    }
}

mod misc;
mod page;

use crate::api::{self, ApiServer};
use crate::config::Config;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use surf::RequestBuilder;
use tide::StatusCode;

pub async fn setup() -> ApiServer {
    // The Default impl is different in the test environment
    let config = Config::load();

    // Build API server
    crate::setup(&config)
        .await
        .expect("Unable to run API setup");
    api::build_server(config)
        .await
        .expect("Unable to build API server")
}

#[async_trait]
pub trait RequestBuilderExt {
    async fn recv_bytes_status(self) -> surf::Result<(Vec<u8>, StatusCode)>;
    async fn recv_string_status(self) -> surf::Result<(String, StatusCode)>;
    async fn recv_json_status<T: DeserializeOwned>(self)
        -> surf::Result<(T, StatusCode)>;
}

#[async_trait]
impl RequestBuilderExt for RequestBuilder {
    async fn recv_bytes_status(self) -> surf::Result<(Vec<u8>, StatusCode)> {
        let mut response = self.send().await?;
        let status_code = response.status();
        let body = response.body_bytes().await?;
        Ok((body, status_code))
    }

    async fn recv_string_status(self) -> surf::Result<(String, StatusCode)> {
        let mut response = self.send().await?;
        let status_code = response.status();
        let body = response.body_string().await?;
        Ok((body, status_code))
    }

    async fn recv_json_status<T: DeserializeOwned>(
        self,
    ) -> surf::Result<(T, StatusCode)> {
        let mut response = self.send().await?;
        let status_code = response.status();
        let body = response.body_json::<T>().await?;
        Ok((body, status_code))
    }
}
