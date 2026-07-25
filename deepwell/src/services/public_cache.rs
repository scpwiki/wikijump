/*
 * services/public_cache.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
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

use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::runtime::ServerState;
use crate::services::ServiceContext;
use redis::{AsyncCommands, Script};

const PUBLIC_CONTENT_CACHE_PREFIX: &str = "deepwell:public-content:site";
pub const PUBLIC_CONTENT_CACHE_INVALIDATION_CHANNEL: &str =
    "wikijump:article-response-fence-invalidation:v1";

#[derive(Debug, Clone, Copy)]
pub struct PublicContentCache;

impl PublicContentCache {
    pub fn site_version_key(site_id: i64) -> String {
        format!("{PUBLIC_CONTENT_CACHE_PREFIX}:{site_id}:version")
    }

    pub async fn cache_fence(ctx: &ServiceContext<'_>, site_id: i64) -> Result<String> {
        let key = Self::site_version_key(site_id);
        let mut redis = ctx.redis();
        let version: Option<String> = redis.get(&key).await.or_raise(|| {
            Error::new(
                "public content cache fence read error",
                ErrorType::RedisQuery,
            )
        })?;

        Ok(version.unwrap_or_else(|| "0".to_owned()))
    }

    pub async fn invalidate_site(ctx: &ServiceContext<'_>, site_id: i64) -> Result<()> {
        Self::invalidate_site_for_state(&ctx.state(), site_id).await
    }

    pub async fn invalidate_site_for_state(
        state: &ServerState,
        site_id: i64,
    ) -> Result<()> {
        let key = Self::site_version_key(site_id);
        let mut redis = state.redis.clone();
        let _: i64 = Script::new(
            r#"
            local version = redis.call('INCR', KEYS[1])
            local payload = '{"type":"public-content","site_id":' .. ARGV[2] .. ',"version":"' .. version .. '"}'
            redis.call('PUBLISH', ARGV[1], payload)
            return version
            "#,
        )
        .key(&key)
        .arg(PUBLIC_CONTENT_CACHE_INVALIDATION_CHANNEL)
        .arg(site_id)
        .invoke_async(&mut redis)
        .await
        .or_raise(|| {
            Error::new(
                "public content cache fence invalidation error",
                ErrorType::RedisQuery,
            )
        })?;

        Ok(())
    }
}
