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

use super::super::{ADMIN_USER_ID, WWW_SITE_ID};
use super::{GeneratedPage, GeneratedSite, GeneratedUser, RequestBuilder};
use crate::api::{self, ApiServer};
use crate::config::Config;
use crate::services::page::CreatePageOutput;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use serde_json::json;
use tide::{http::Method, Result, StatusCode};

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
    /// API server to run requests against.
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
        self.slug_with_prefix("test-")
    }

    pub fn slug_with_prefix(&self, prefix: &str) -> String {
        let mut slug = self.name_with_prefix(prefix);
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

    // Factory methods

    #[inline]
    pub async fn page(&self) -> Result<GeneratedPage> {
        self.page2(None, None, None).await
    }

    pub async fn page2(
        &self,
        site_id: Option<i64>,
        user_id: Option<i64>,
        slug: Option<String>,
    ) -> Result<GeneratedPage> {
        let site_id = site_id.unwrap_or(WWW_SITE_ID);
        let user_id = user_id.unwrap_or(ADMIN_USER_ID);
        let slug = slug.unwrap_or_else(|| self.slug());

        let (output, status) = self
            .post(format!("/page/{site_id}"))?
            .body_json(json!({
                "wikitext": "Page contents",
                "title": "Page title",
                "altTitle": null,
                "slug": &slug,
                "revisionComments": "[factory] Create page",
                "userId": user_id,
            }))?
            .recv_json::<CreatePageOutput>()
            .await?;

        assert_eq!(status, StatusCode::Ok, "[factory] Failed to create page");

        Ok(GeneratedPage {
            revision_id: output.revision_id,
            site_id,
            user_id,
            page_id: output.page_id,
            slug,
        })
    }

    pub async fn site(&self) -> Result<GeneratedSite> {
        let slug = self.slug_with_prefix("test-site-");

        let (site_id, status) = self
            .post(format!(
                "/site/temp/Test Site/{slug}/A temporary site for testing/en",
            ))?
            .recv_json::<i64>()
            .await?;

        assert_eq!(status, StatusCode::Ok, "[factory] Failed to create site");

        Ok(GeneratedSite { site_id, slug })
    }

    // TODO
    #[allow(dead_code)]
    pub async fn user(&self) -> Result<GeneratedUser> {
        todo!()
    }
}
