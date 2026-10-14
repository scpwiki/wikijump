/*
 * tests/render_ahead.rs
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

//! Committed render-ahead coverage.
//!
//! A renderer generation switch renders stale saved pages into
//! `page_revision_render_ahead` while the previous build serves, and the next
//! build promotes the rows when it starts. These tests pin that the ahead run
//! leaves the served revision untouched, that promotion replaces it, and that
//! a revision rerendered by the serving build in between is not overwritten
//! with output computed from older inputs.

use deepwell::api::{
    ServerState, build_server_state_without_workers_with_job_queue_namespace,
    promote_render_ahead, run_render_ahead,
};
use deepwell::config::{Config, Secrets};
use deepwell::constants::ADMIN_USER_ID;
use deepwell::services::page::{CreatePage, PageService};
use deepwell::services::{PageRevisionService, ServiceContext, SiteService, TextService};
use deepwell::types::Reference;
use redis::AsyncCommands;
use sea_orm::{ConnectionTrait, Statement, TransactionTrait, Value};
use serde_json::Value as JsonValue;
use std::borrow::Cow;
use std::env;
use std::net::{IpAddr, Ipv4Addr};
use uuid::Uuid;

const TEST_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 0, 2, 84));
const STALE_GENERATOR: &str = "ftml v0.0.0; deepwell-render/v0";
const STALE_BODY: &str = "RENDER_AHEAD_STALE_BODY";

struct Fixture {
    state: ServerState,
    namespace: String,
    site_id: i64,
    page_id: i64,
    marker: String,
}

#[derive(Debug, PartialEq, Eq)]
struct ServedRevision {
    generator: String,
    updated_at: Option<String>,
    body: String,
}

impl Fixture {
    /// Commit a page, then make its saved output stale as a renderer
    /// generation bump does: an old generator and an old body.
    async fn new(label: &str) -> Self {
        let namespace = format!("rsmq-render-ahead-{label}-{}", Uuid::new_v4().simple());
        let state = build_server_state_without_workers_with_job_queue_namespace(
            Config::integration_testing(),
            Secrets::load(),
            &namespace,
        )
        .await
        .expect("isolated render-ahead state should build");
        let marker = format!("RENDER_AHEAD_SOURCE_{}", Uuid::new_v4().simple());

        let txn = state
            .database
            .begin()
            .await
            .expect("fixture transaction should begin");
        let (site_id, page_id) = {
            let ctx = ServiceContext::new(&state, &txn);
            let site_id = SiteService::get(&ctx, Reference::Slug(Cow::Borrowed("test")))
                .await
                .expect("seeded test site should exist")
                .site_id;
            let page_id = PageService::create(
                &ctx,
                CreatePage {
                    site_id,
                    wikitext: marker.clone(),
                    title: String::from("Render ahead fixture"),
                    alt_title: None,
                    tags: Vec::new(),
                    slug: format!("render-ahead-{label}-{}", Uuid::new_v4().simple()),
                    layout: None,
                    revision_comments: String::from("render-ahead fixture"),
                    user_id: ADMIN_USER_ID,
                    bypass_filter: true,
                    ip_address: TEST_IP,
                },
            )
            .await
            .expect("fixture page should be created")
            .page_id;
            let stale_body = TextService::create(&ctx, STALE_BODY.to_owned())
                .await
                .expect("stale body should be stored");
            let revision = PageRevisionService::get_latest(&ctx, site_id, page_id)
                .await
                .expect("fixture revision should exist");
            txn.execute_raw(Statement::from_sql_and_values(
                txn.get_database_backend(),
                "UPDATE page_revision SET compiled_generator = $1, compiled_body_html_hash = $2 WHERE revision_id = $3",
                [
                    Value::from(STALE_GENERATOR),
                    Value::from(stale_body.to_vec()),
                    Value::from(revision.revision_id),
                ],
            ))
            .await
            .expect("fixture revision should be made stale");
            (site_id, page_id)
        };
        txn.commit()
            .await
            .expect("fixture transaction should commit");

        Self {
            state,
            namespace,
            site_id,
            page_id,
            marker,
        }
    }

    async fn render_ahead(&self, dry_run: bool) -> JsonValue {
        let page_ids = self.page_id.to_string();
        run_render_ahead(&self.state, |name| match name {
            "DEEPWELL_RENDER_AHEAD_PAGE_IDS" => Some(page_ids.clone()),
            "DEEPWELL_RENDER_AHEAD_CONCURRENCY" => Some(String::from("1")),
            "DEEPWELL_RENDER_AHEAD_DRY_RUN" => Some(dry_run.to_string()),
            _ => None,
        })
        .await
        .expect("render-ahead should run")
    }

    async fn served(&self) -> ServedRevision {
        let txn = self
            .state
            .database
            .begin()
            .await
            .expect("inspection transaction should begin");
        let ctx = ServiceContext::new(&self.state, &txn);
        let revision = PageRevisionService::get_latest(&ctx, self.site_id, self.page_id)
            .await
            .expect("fixture revision should exist");
        let body = TextService::get(&ctx, &revision.compiled_body_html_hash)
            .await
            .expect("served body should exist");
        ServedRevision {
            generator: revision.compiled_generator,
            updated_at: revision.updated_at.map(|value| value.to_string()),
            body,
        }
    }

    async fn ahead_rows(&self) -> i64 {
        self.state
            .database
            .query_one_raw(Statement::from_sql_and_values(
                self.state.database.get_database_backend(),
                "SELECT COUNT(*) AS rows FROM page_revision_render_ahead WHERE page_id = $1",
                [Value::from(self.page_id)],
            ))
            .await
            .expect("ahead rows should be counted")
            .expect("count row should be returned")
            .try_get("", "rows")
            .expect("count should be an integer")
    }

    /// Stand in for the serving build rerendering the revision after the
    /// ahead row's inputs were read.
    async fn touch_served_revision(&self) {
        self.state
            .database
            .execute_raw(Statement::from_sql_and_values(
                self.state.database.get_database_backend(),
                "UPDATE page_revision SET updated_at = COALESCE(updated_at, NOW()) + INTERVAL '1 second' WHERE revision_id = (SELECT latest_revision_id FROM page WHERE page_id = $1)",
                [Value::from(self.page_id)],
            ))
            .await
            .expect("served revision should be touched");
    }

    async fn cleanup(self) {
        let redis_url =
            env::var("REDIS_URL").expect("REDIS_URL must be set for integration tests");
        let client =
            redis::Client::open(redis_url).expect("failed to construct Redis client");
        let mut connection = client
            .get_multiplexed_async_connection()
            .await
            .expect("failed to connect to Redis for cleanup");
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(format!("{}:*", self.namespace))
            .query_async(&mut connection)
            .await
            .expect("failed to list test Redis keys");
        if !keys.is_empty() {
            let _: usize = connection
                .del(keys)
                .await
                .expect("failed to delete test Redis keys");
        }
    }
}

fn count(summary: &JsonValue, field: &str) -> i64 {
    summary[field]
        .as_i64()
        .unwrap_or_else(|| panic!("summary field {field} should be a count: {summary}"))
}

#[tokio::test]
async fn render_ahead_is_served_only_after_promotion() {
    let fixture = Fixture::new("promote").await;
    let before = fixture.served().await;
    assert_eq!(before.generator, STALE_GENERATOR);
    assert_eq!(before.body, STALE_BODY);

    let summary = fixture.render_ahead(false).await;
    assert_eq!(count(&summary, "candidates"), 1, "{summary}");
    assert_eq!(count(&summary, "rendered"), 1, "{summary}");
    assert_eq!(count(&summary, "failed"), 0, "{summary}");
    assert_eq!(count(&summary, "stale_without_valid_ahead"), 0, "{summary}");
    assert_eq!(
        fixture.served().await,
        before,
        "the serving build's revision must not change before promotion",
    );
    assert_eq!(fixture.ahead_rows().await, 1);

    let rerun = fixture.render_ahead(false).await;
    assert_eq!(
        count(&rerun, "candidates"),
        0,
        "a page with a valid ahead row is not rendered again: {rerun}",
    );

    let promotion = promote_render_ahead(&fixture.state, Some(&[fixture.page_id]))
        .await
        .expect("promotion should run");
    assert_eq!(promotion.promoted, 1, "{promotion:?}");
    assert_eq!(promotion.discarded, 0, "{promotion:?}");
    assert_eq!(promotion.site_ids, vec![fixture.site_id]);

    let after = fixture.served().await;
    assert_eq!(after.generator, promotion.compiled_generator);
    assert_ne!(after.generator, STALE_GENERATOR);
    assert!(
        after.body.contains(&fixture.marker),
        "promotion must serve the ahead render: {}",
        after.body,
    );
    assert_eq!(fixture.ahead_rows().await, 0, "promoted rows are removed");

    let repeat = promote_render_ahead(&fixture.state, Some(&[fixture.page_id]))
        .await
        .expect("a repeated promotion should run");
    assert_eq!(repeat.promoted, 0, "promotion is idempotent: {repeat:?}");

    fixture.cleanup().await;
}

#[tokio::test]
async fn render_ahead_rows_do_not_replace_a_revision_rerendered_since() {
    let fixture = Fixture::new("superseded").await;
    let summary = fixture.render_ahead(false).await;
    assert_eq!(count(&summary, "rendered"), 1, "{summary}");

    fixture.touch_served_revision().await;
    let promotion = promote_render_ahead(&fixture.state, Some(&[fixture.page_id]))
        .await
        .expect("promotion should run");
    assert_eq!(promotion.promoted, 0, "{promotion:?}");
    assert_eq!(promotion.discarded, 1, "{promotion:?}");
    let served = fixture.served().await;
    assert_eq!(served.generator, STALE_GENERATOR);
    assert_eq!(served.body, STALE_BODY);

    let retry = fixture.render_ahead(false).await;
    assert_eq!(
        count(&retry, "candidates"),
        1,
        "the superseded page is rendered again: {retry}",
    );
    let promotion = promote_render_ahead(&fixture.state, Some(&[fixture.page_id]))
        .await
        .expect("promotion should run");
    assert_eq!(promotion.promoted, 1, "{promotion:?}");

    fixture.cleanup().await;
}

#[tokio::test]
async fn render_ahead_dry_run_renders_nothing() {
    let fixture = Fixture::new("dry-run").await;
    let summary = fixture.render_ahead(true).await;
    assert_eq!(count(&summary, "candidates"), 1, "{summary}");
    assert_eq!(count(&summary, "rendered"), 0, "{summary}");
    assert_eq!(count(&summary, "stale_without_valid_ahead"), 1, "{summary}");
    assert_eq!(fixture.ahead_rows().await, 0);

    fixture.cleanup().await;
}
