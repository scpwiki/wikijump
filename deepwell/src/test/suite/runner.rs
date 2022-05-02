/*
 * test/suite/runner.rs
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

use super::RequestBuilder;
use crate::api::{self, ApiServer};
use crate::config::Config;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use tide::{http::Method, Result};

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
pub struct Runner {
    pub app: ApiServer,
}

impl Runner {
    pub async fn setup() -> Result<Self> {
        // The Default impl is different in the test environment
        let config = Config::load();

        // Build API server
        crate::setup(&config).await?;
        let app = api::build_internal_api(config).await?;

        // Build and return
        Ok(Runner { app })
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

    // Helper methods

    #[inline]
    pub fn slug(&self) -> String {
        let mut slug = self.name_with_prefix("test-");
        slug.make_ascii_lowercase();
        slug
    }

    pub fn name_with_prefix(&self, prefix: &str) -> String {
        let mut slug = String::from(prefix);
        let mut rng = thread_rng();

        for _ in 0..20 {
            slug.push(rng.sample(Alphanumeric) as char);
        }

        slug
    }
}
