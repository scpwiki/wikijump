/*
 * cache.rs
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

//! Manages cached data in Redis.
//!
//! Whenever you make changes to this module, make sure that the code is
//! compatible with DEEPWELL's Redis code.

use crate::deepwell::FileData;
use crate::error::Result;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;

macro_rules! get_connection {
    ($client:expr) => {
        $client.get_multiplexed_async_connection().await?
    };
}

/// Helper macro to build redis keys.
///
/// This way we avoid issues with inconsistent key names,
/// forgetting to update its values or other footguns.
macro_rules! redis_key {
    (site_domain => $site_id:expr $(,)?) => {
        format!("site_domain:{}", $site_id)
    };
    (page_slug => $site_id:expr, $page_slug:expr $(,)?) => {
        format!("page_slug:{}:{}", $site_id, $page_slug)
    };
    (file_name => $site_id:expr, $page_id:expr, $filename:expr $(,)?) => {
        format!("file_name:{}:{}:{}", $site_id, $page_id, $filename)
    };
}

macro_rules! set {
    ($conn:expr, $key:expr, $value:expr $(,)?) => {
        $conn.set::<_, _, ()>($key, $value).await?
    };
}

macro_rules! hset {
    ($conn:expr, $key:expr, $field:expr, $value:expr $(,)?) => {
        $conn.hset::<_, _, _, ()>(&$key, $field, $value).await?
    };
}

macro_rules! hdel {
    ($conn:expr, $key:expr, $field:expr $(,)?) => {
        $conn.hdel::<_, _, ()>(&$key, $field).await?
    };
}

#[derive(Debug)]
pub struct Cache {
    client: redis::Client,
}

impl Cache {
    /// Connect to the Redis cluster.
    pub fn connect(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Cache { client })
    }

    /// Gets the preferred domain for a given site.
    pub async fn get_site_domain(&self, site_id: i64) -> Result<Option<String>> {
        let mut conn = get_connection!(self.client);
        let key = redis_key!(site_domain => site_id);
        let value = conn.get(key).await?;
        Ok(value)
    }

    pub async fn set_site_domain(
        &self,
        site_id: i64,
        preferred_domain: &str,
    ) -> Result<()> {
        let mut conn = get_connection!(self.client);
        let key = redis_key!(site_domain => site_id);
        set!(conn, key, preferred_domain);
        Ok(())
    }

    /// Gets the page ID for a site ID and page slug pair.
    pub async fn get_page(&self, site_id: i64, page_slug: &str) -> Result<Option<i64>> {
        let mut conn = get_connection!(self.client);
        let key = redis_key!(page_slug => site_id, page_slug);
        let value = conn.get(key).await?;
        Ok(value)
    }

    pub async fn set_page(
        &self,
        site_id: i64,
        page_slug: &str,
        page_id: i64,
    ) -> Result<()> {
        let mut conn = get_connection!(self.client);
        let key = redis_key!(page_slug => site_id, page_slug);
        set!(conn, key, page_id);
        Ok(())
    }

    /// Gets the file ID for a site ID, page ID, and filename triplet.
    pub async fn get_file(
        &self,
        site_id: i64,
        page_id: i64,
        filename: &str,
    ) -> Result<Option<FileData>> {
        let mut conn = get_connection!(self.client);
        let key = redis_key!(file_name => site_id, page_id, filename);
        let fields = &["id", "mime", "size", "s3_hash"];
        let values = conn.hmget(&key, fields).await?;
        match values {
            // Ideally, all of these should be non-null, if it's a cache hit.
            (Some(file_id), Some(mime), Some(size), Some(s3_hash)) => {
                Ok(Some(FileData {
                    file_id,
                    mime,
                    size,
                    s3_hash,
                }))
            }

            // Cache miss
            (None, None, None, None) => Ok(None),

            // Some fields are set and others aren't. Let's clear all them out.
            _ => {
                clear_inconsistent_fields(&mut conn, &key, fields).await?;
                Ok(None)
            }
        }
    }

    pub async fn set_file(
        &self,
        site_id: i64,
        page_id: i64,
        filename: &str,
        data: &FileData,
    ) -> Result<()> {
        let mut conn = get_connection!(self.client);
        let key = redis_key!(file_name => site_id, page_id, filename);
        hset!(conn, key, "id", data.file_id);
        hset!(conn, key, "mime", &data.mime);
        hset!(conn, key, "size", data.size);
        hset!(conn, key, "s3_hash", &data.s3_hash);
        Ok(())
    }
}

async fn clear_inconsistent_fields(
    conn: &mut MultiplexedConnection,
    key: &str,
    fields: &[&str],
) -> Result<()> {
    warn!(key = key, "Inconsistent cache data, deleting");
    hdel!(conn, key, fields);
    Ok(())
}
