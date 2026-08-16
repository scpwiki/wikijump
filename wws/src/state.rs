/*
 * state.rs
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

use crate::cache::Cache;
use crate::config::Secrets;
use crate::deepwell::{Deepwell, FileData, PageData, UserData};
use crate::error::{
    BasicError, FallbackError, ResponseResult, Result, build_basic_error_response,
};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use s3::bucket::Bucket;
use std::sync::Arc;
use std::time::Duration;

const BUCKET_REQUEST_TIMEOUT: Duration = Duration::from_millis(200);

pub type ServerState = Arc<ServerStateInner>;

#[derive(Debug)]
pub struct ServerStateInner {
    pub deepwell: Deepwell,
    pub cache: Cache,
    pub s3_files_bucket: Box<Bucket>,
    pub s3_tblocks_bucket: Box<Bucket>,
}

pub async fn build_server_state(
    check_deepwell: bool,
    Secrets {
        deepwell_url,
        redis_url,
        s3_files_bucket,
        s3_tblocks_bucket,
        s3_region,
        s3_credentials,
        s3_path_style,
    }: Secrets,
) -> Result<ServerState> {
    let deepwell = Deepwell::connect(&deepwell_url)?;
    if check_deepwell {
        deepwell.check().await;
    }

    let cache = Cache::connect(&redis_url)?;

    let (s3_files_bucket, s3_tblocks_bucket) = {
        let mut files_bucket =
            Bucket::new(&s3_files_bucket, s3_region.clone(), s3_credentials.clone())?;

        let mut tblocks_bucket = Bucket::new(
            &s3_tblocks_bucket,
            s3_region.clone(),
            s3_credentials.clone(),
        )?;

        if s3_path_style {
            files_bucket = files_bucket.with_path_style();
            tblocks_bucket = tblocks_bucket.with_path_style();
        }

        files_bucket.request_timeout = Some(BUCKET_REQUEST_TIMEOUT);
        tblocks_bucket.request_timeout = Some(BUCKET_REQUEST_TIMEOUT);
        (files_bucket, tblocks_bucket)
    };

    Ok(Arc::new(ServerStateInner {
        deepwell,
        cache,
        s3_files_bucket,
        s3_tblocks_bucket,
    }))
}

impl ServerStateInner {
    // Contains implementations for the common pattern of "check the cache,
    // if not present, get it from DEEPWELL and populate it".

    pub async fn get_site_domain(&self, site_id: i64) -> Result<String> {
        match self.cache.get_site_domain(site_id).await? {
            Some(preferred_domain) => Ok(preferred_domain),
            None => {
                let preferred_domain = self.deepwell.get_site_domain(site_id).await?;
                self.cache
                    .set_site_domain(site_id, &preferred_domain)
                    .await?;

                Ok(preferred_domain)
            }
        }
    }

    pub async fn get_site_domain_or_response(
        &self,
        site_id: i64,
    ) -> ResponseResult<String> {
        match self.get_site_domain(site_id).await {
            Ok(domain) => Ok(domain),
            Err(error) => {
                // XF-1003
                error!(
                    site_id = site_id,
                    "Could not fetch preferred site domain: {error}",
                );
                Err(FallbackError::RedirectMain.into_response())
            }
        }
    }

    pub async fn get_page(&self, site_id: i64, page_slug: &str) -> Result<Option<i64>> {
        match self.cache.get_page(site_id, page_slug).await? {
            Some(page_id) => Ok(Some(page_id)),
            None => match self.deepwell.get_page(site_id, page_slug).await? {
                None => Ok(None),
                Some(PageData { page_id, .. }) => {
                    self.cache.set_page(site_id, page_slug, page_id).await?;
                    Ok(Some(page_id))
                }
            },
        }
    }

    pub async fn get_page_or_response(
        &self,
        headers: &HeaderMap,
        site_id: i64,
        page_slug: &str,
    ) -> ResponseResult<i64> {
        match self.get_page(site_id, page_slug).await {
            Ok(Some(page_id)) => Ok(page_id),
            Ok(None) => {
                error!(
                    site_id = site_id,
                    page_slug = page_slug,
                    "Cannot complete request, no such page",
                );

                let response = build_basic_error_response(
                    self,
                    headers,
                    BasicError::PageSlug { site_id, page_slug },
                )
                .await;

                Err(response)
            }
            Err(error) => {
                error!(
                    site_id = site_id,
                    page_slug = page_slug,
                    "Cannot get page info: {error}",
                );

                let response = build_basic_error_response(
                    self,
                    headers,
                    BasicError::PageFetch { site_id, page_slug },
                )
                .await;

                Err(response)
            }
        }
    }

    pub async fn get_file(
        &self,
        site_id: i64,
        page_id: i64,
        filename: &str,
    ) -> Result<Option<FileData>> {
        match self.cache.get_file(site_id, page_id, filename).await? {
            Some(data) => Ok(Some(data)),
            None => match self.deepwell.get_file(site_id, page_id, filename).await? {
                None => Ok(None),
                Some(data) => {
                    self.cache
                        .set_file(site_id, page_id, filename, &data)
                        .await?;

                    Ok(Some(data))
                }
            },
        }
    }

    pub async fn get_file_or_response(
        &self,
        headers: &HeaderMap,
        site_id: i64,
        page_id: i64,
        page_slug: &str,
        filename: &str,
    ) -> ResponseResult<FileData> {
        match self.get_file(site_id, page_id, filename).await {
            Ok(Some(file_info)) => Ok(file_info),
            Ok(None) => {
                error!(
                    site_id = site_id,
                    page_id = page_id,
                    filename = filename,
                    "Cannot complete request, none with filename",
                );

                let response = build_basic_error_response(
                    self,
                    headers,
                    BasicError::FileName {
                        site_id,
                        page_slug,
                        filename,
                    },
                )
                .await;

                Err(response)
            }
            Err(error) => {
                error!(
                    site_id = site_id,
                    page_id = page_id,
                    filename = filename,
                    "Cannot get file info: {error}",
                );

                let response = build_basic_error_response(
                    self,
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

    pub async fn get_avatar(&self, user_id: i64) -> Result<Option<String>> {
        match self.cache.get_avatar(user_id).await? {
            Some(avatar_s3_hash) => Ok(Some(avatar_s3_hash)),
            None => match self.deepwell.get_user(user_id).await? {
                None => Ok(None),
                Some(UserData { avatar_s3_hash }) => {
                    let s3_hash: String = avatar_s3_hash
                        .iter()
                        .map(|b| format!("{:02x}", b).to_string())
                        .collect();
                    self.cache.set_avatar(user_id, &s3_hash).await?;
                    Ok(Some(s3_hash))
                }
            },
        }
    }

    pub async fn get_avatar_or_response(
        &self,
        headers: &HeaderMap,
        user_id: i64,
    ) -> ResponseResult<String> {
        match self.get_avatar(user_id).await {
            Ok(Some(avatar_s3_hash)) => Ok(avatar_s3_hash),
            Ok(None) => {
                error!(
                    user_id = user_id,
                    "Cannot complete request, no such user or no avatar set",
                );

                let response = build_basic_error_response(
                    self,
                    headers,
                    BasicError::UserAvatar { user_id },
                )
                .await;

                Err(response)
            }
            Err(error) => {
                error!(user_id = user_id, "Cannot get user info: {error}",);

                let response = build_basic_error_response(
                    self,
                    headers,
                    BasicError::UserFetch { user_id },
                )
                .await;

                Err(response)
            }
        }
    }
}
