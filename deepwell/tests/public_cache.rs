/*
 * tests/public_cache.rs
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

#![allow(unused_imports)]

#[macro_use]
mod common;

use self::common::TestRunner;
use deepwell::services::public_cache::{
    PUBLIC_CONTENT_CACHE_INVALIDATION_CHANNEL, PublicContentCache,
};
use futures::StreamExt;
use redis::AsyncCommands;
use std::{env, time::Duration};

async fn subscribed_invalidation_stream() -> redis::aio::PubSub {
    let redis_url =
        env::var("REDIS_URL").expect("REDIS_URL must be set for integration tests");
    let client = redis::Client::open(redis_url).expect("failed to build Redis client");
    let mut pubsub = client
        .get_async_pubsub()
        .await
        .expect("failed to open Redis pub/sub connection");
    pubsub
        .subscribe(PUBLIC_CONTENT_CACHE_INVALIDATION_CHANNEL)
        .await
        .expect("failed to subscribe to public content cache invalidations");
    pubsub
}

#[tokio::test]
async fn public_content_cache_fence_defaults_to_zero_and_increments() {
    let runner = TestRunner::setup().await;
    let ctx = runner.context();
    let site_id = 9_900_001;
    let key = PublicContentCache::site_version_key(site_id);
    let mut redis = ctx.redis();
    let _: usize = redis.del(&key).await.expect("failed to clear test key");

    assert_eq!(
        PublicContentCache::cache_fence(ctx, site_id)
            .await
            .expect("failed to read missing public content cache fence"),
        "0"
    );

    PublicContentCache::invalidate_site(ctx, site_id)
        .await
        .expect("failed to increment public content cache fence");
    assert_eq!(
        PublicContentCache::cache_fence(ctx, site_id)
            .await
            .expect("failed to read incremented public content cache fence"),
        "1"
    );

    PublicContentCache::invalidate_site(ctx, site_id)
        .await
        .expect("failed to increment public content cache fence again");
    assert_eq!(
        PublicContentCache::cache_fence(ctx, site_id)
            .await
            .expect("failed to read incremented public content cache fence"),
        "2"
    );

    let _: usize = redis.del(&key).await.expect("failed to clean test key");
}

#[tokio::test]
async fn public_content_cache_invalidation_can_be_deferred_until_post_commit() {
    let runner = TestRunner::setup().await;
    let ctx = runner.context();
    let site_id = 9_900_002;
    let key = PublicContentCache::site_version_key(site_id);
    let mut redis = ctx.redis();
    let _: usize = redis.del(&key).await.expect("failed to clear test key");

    ctx.defer_public_content_cache_invalidate_site(site_id)
        .expect("failed to defer public content cache invalidation");

    assert_eq!(
        PublicContentCache::cache_fence(ctx, site_id)
            .await
            .expect("failed to read deferred public content cache fence"),
        "0"
    );

    ctx.run_post_commit_actions()
        .await
        .expect("failed to run deferred public content cache invalidation");
    assert_eq!(
        PublicContentCache::cache_fence(ctx, site_id)
            .await
            .expect("failed to read invalidated public content cache fence"),
        "1"
    );

    let _: usize = redis.del(&key).await.expect("failed to clean test key");
}

#[tokio::test]
async fn public_content_cache_invalidation_publishes_incremented_fence() {
    let runner = TestRunner::setup().await;
    let ctx = runner.context();
    let site_id = 9_900_003;
    let key = PublicContentCache::site_version_key(site_id);
    let mut redis = ctx.redis();
    let _: usize = redis.del(&key).await.expect("failed to clear test key");
    let mut pubsub = subscribed_invalidation_stream().await;

    PublicContentCache::invalidate_site(ctx, site_id)
        .await
        .expect("failed to increment public content cache fence");

    let message =
        tokio::time::timeout(Duration::from_secs(2), pubsub.on_message().next())
            .await
            .expect("timed out waiting for public content invalidation")
            .expect("pub/sub stream ended unexpectedly");
    let payload: String = message
        .get_payload()
        .expect("failed to read public content invalidation payload");

    assert_eq!(
        payload,
        format!(r#"{{"type":"public-content","site_id":{site_id},"version":"1"}}"#)
    );

    let _: usize = redis.del(&key).await.expect("failed to clean test key");
}
