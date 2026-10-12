/*
 * tests/corpus_finalizer.rs
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

//! Committed corpus render-finalizer dependent coverage.
//!
//! Pass 2 rerenders an imported include source and outdates its dependents,
//! which rerender immediately after the finalizer commits. A dependent whose
//! latest revision was imported from Wikidot must render with the same corpus
//! include ceiling the finalizer used for it in pass 1, while a locally
//! authored dependent keeps the public safety limit.

use deepwell::api::{
    ServerState, build_server_state_without_workers_with_job_queue_namespace,
    run_corpus_render_finalizer,
};
use deepwell::config::{Config, Secrets};
use deepwell::constants::ADMIN_USER_ID;
use deepwell::models::page_revision::Entity as PageRevisionTable;
use deepwell::services::page::{CreatePage, PageService};
use deepwell::services::{PageRevisionService, ServiceContext, SiteService, TextService};
use deepwell::types::Reference;
use redis::AsyncCommands;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, EntityTrait, IntoActiveModel, Set, Statement,
    TransactionTrait, Value,
};
use serde_json::Value as JsonValue;
use std::borrow::Cow;
use std::env;
use std::net::{IpAddr, Ipv4Addr};
use uuid::Uuid;

const TEST_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 0, 2, 83));
const LEAF_MARKER: &str = "CORPUS_CASCADE_LEAF";

/// One include of the source plus this many leaf includes exceeds the public
/// ceiling of 256 total includes but stays within the corpus ceiling.
const DENSE_LEAF_INCLUDES: usize = 300;

async fn cleanup_namespace(redis_url: &str, namespace: &str) {
    let client =
        redis::Client::open(redis_url).expect("failed to construct Redis client");
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .expect("failed to connect to Redis for cleanup");
    let keys: Vec<String> = redis::cmd("KEYS")
        .arg(format!("{namespace}:*"))
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

struct CascadeFixture {
    state: ServerState,
    site_id: i64,
    dependent_page_id: i64,
    import_run_id: i64,
}

async fn create_page(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    slug: &str,
    wikitext: String,
) -> i64 {
    PageService::create(
        ctx,
        CreatePage {
            site_id,
            wikitext,
            title: slug.to_owned(),
            alt_title: None,
            tags: Vec::new(),
            slug: slug.to_owned(),
            layout: None,
            revision_comments: String::from("corpus finalizer cascade fixture"),
            user_id: ADMIN_USER_ID,
            bypass_filter: true,
            ip_address: TEST_IP,
        },
    )
    .await
    .expect("cascade fixture page should be created")
    .page_id
}

/// Replace a page's latest source without rendering it, as the corpus import
/// stores revisions before the finalizer renders them.
async fn store_imported_source(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_id: i64,
    wikitext: Option<String>,
    from_wikidot: bool,
) {
    let revision = PageRevisionService::get_latest(ctx, site_id, page_id)
        .await
        .expect("cascade fixture revision should exist");
    let mut model = PageRevisionTable::find_by_id(revision.revision_id)
        .one(ctx.transaction())
        .await
        .expect("cascade fixture revision lookup should succeed")
        .expect("cascade fixture revision row should exist")
        .into_active_model();
    if let Some(wikitext) = wikitext {
        let hash = TextService::create(ctx, wikitext)
            .await
            .expect("cascade fixture source should be stored");
        model.wikitext_hash = Set(hash.to_vec());
    }
    model.from_wikidot = Set(from_wikidot);
    model
        .update(ctx.transaction())
        .await
        .expect("cascade fixture revision should be updated");
}

impl CascadeFixture {
    /// Commit an imported include source, a leaf it shares with its
    /// dependent, and a dependent whose stored source is denser than the
    /// public include ceiling. The source is a rendered pass-1 item of an
    /// import run, so pass 2 selects it for its query module.
    async fn new(namespace: &str, dependent_from_wikidot: bool) -> Self {
        let run_id = Uuid::new_v4().simple().to_string();
        let mut config = Config::integration_testing();
        config.rerender_skip = vec![(10, None)];
        let state = build_server_state_without_workers_with_job_queue_namespace(
            config,
            Secrets::load(),
            namespace,
        )
        .await
        .expect("isolated corpus finalizer state should build");

        let leaf_slug = format!("component:corpus-cascade-leaf-{run_id}");
        let source_slug = format!("component:corpus-cascade-source-{run_id}");
        let dependent_slug = format!("corpus-cascade-dependent-{run_id}");
        let transaction = state
            .database
            .begin()
            .await
            .expect("cascade fixture transaction should begin");
        let (site_id, source_page_id, dependent_page_id) = {
            let ctx = ServiceContext::new(&state, &transaction);
            let site_id = SiteService::get(&ctx, Reference::Slug(Cow::Borrowed("test")))
                .await
                .expect("seeded test site should exist")
                .site_id;
            create_page(&ctx, site_id, &leaf_slug, LEAF_MARKER.to_owned()).await;
            let source_page_id = create_page(
                &ctx,
                site_id,
                &source_slug,
                String::from(
                    "[[module ListPages range=\".\"]]\nCited as %%title%%\n[[/module]]",
                ),
            )
            .await;
            let dependent_page_id = create_page(
                &ctx,
                site_id,
                &dependent_slug,
                format!("[[include {source_slug}]]\n[[include {leaf_slug}]]\n"),
            )
            .await;
            store_imported_source(&ctx, site_id, source_page_id, None, true).await;
            store_imported_source(
                &ctx,
                site_id,
                dependent_page_id,
                Some(format!(
                    "[[include {source_slug}]]\n{}",
                    format!("[[include {leaf_slug}]]\n").repeat(DENSE_LEAF_INCLUDES),
                )),
                dependent_from_wikidot,
            )
            .await;
            (site_id, source_page_id, dependent_page_id)
        };
        let backend = transaction.get_database_backend();
        let import_run_id = transaction
            .query_one_raw(Statement::from_sql_and_values(
                backend,
                "INSERT INTO wikidot_corpus_import_run(site_id, source_branch, source_site, manifest_sha256, manifest_row_count, complete_inventory, state) VALUES ($1, 'en', 'corpus-cascade', decode(repeat('ab', 32), 'hex'), 1, false, 'rendering') RETURNING import_run_id",
                [Value::from(site_id)],
            ))
            .await
            .expect("cascade import run should be inserted")
            .expect("cascade import run should be returned")
            .try_get::<i64>("", "import_run_id")
            .expect("cascade import run ID should be present");
        transaction
            .execute_raw(Statement::from_sql_and_values(
                backend,
                "INSERT INTO wikidot_corpus_import_item(import_run_id, source_entity_id, source_fullname, page_id, source_sha256, meta_sha256, state) VALUES ($1, $2::uuid, $3, $4, decode(repeat('ab', 32), 'hex'), decode(repeat('cd', 32), 'hex'), 'rendered')",
                [
                    Value::from(import_run_id),
                    Value::from(Uuid::new_v4().to_string()),
                    Value::from(source_slug),
                    Value::from(source_page_id),
                ],
            ))
            .await
            .expect("rendered cascade source item should be inserted");
        transaction
            .commit()
            .await
            .expect("cascade fixture transaction should commit");

        Self {
            state,
            site_id,
            dependent_page_id,
            import_run_id,
        }
    }

    async fn run_pass2(&self) -> JsonValue {
        let import_run_id = self.import_run_id.to_string();
        run_corpus_render_finalizer(&self.state, |name| match name {
            "DEEPWELL_RENDER_IMPORT_RUN_ID" => Some(import_run_id.clone()),
            "DEEPWELL_RENDER_PASS" => Some(String::from("pass2")),
            "DEEPWELL_RENDER_CONCURRENCY" => Some(String::from("1")),
            "DEEPWELL_RENDER_DRY_RUN" => Some(String::from("false")),
            _ => None,
        })
        .await
        .expect("pass 2 should run")
    }

    async fn dependent_leaf_count(&self) -> usize {
        let transaction = self
            .state
            .database
            .begin()
            .await
            .expect("dependent inspection transaction should begin");
        let ctx = ServiceContext::new(&self.state, &transaction);
        let revision =
            PageRevisionService::get_latest(&ctx, self.site_id, self.dependent_page_id)
                .await
                .expect("dependent revision should exist");
        let html = TextService::get(&ctx, &revision.compiled_body_html_hash)
            .await
            .expect("dependent compiled body should exist");
        html.matches(LEAF_MARKER).count()
    }
}

fn summary_count(summary: &JsonValue, field: &str) -> u64 {
    summary[field]
        .as_u64()
        .unwrap_or_else(|| panic!("summary field {field} should be a count: {summary}"))
}

#[tokio::test]
async fn corpus_pass2_rerenders_imported_dense_dependents_with_the_corpus_ceiling() {
    let redis_url =
        env::var("REDIS_URL").expect("REDIS_URL must be set for integration tests");
    let namespace = format!("rsmq-corpus-cascade-imported-{}", Uuid::new_v4().simple());
    let fixture = CascadeFixture::new(&namespace, true).await;
    assert_eq!(
        fixture.dependent_leaf_count().await,
        1,
        "the dependent should still serve its pre-import render",
    );

    let summary = fixture.run_pass2().await;
    assert_eq!(summary_count(&summary, "claimed"), 1, "{summary}");
    assert_eq!(
        summary_count(&summary, "render_failed"),
        0,
        "an imported dense dependent must not fail the source's pass 2: {summary}",
    );
    assert_eq!(summary_count(&summary, "done"), 1, "{summary}");
    assert_eq!(
        fixture.dependent_leaf_count().await,
        DENSE_LEAF_INCLUDES,
        "the post-commit rerender must render every imported include",
    );

    cleanup_namespace(&redis_url, &namespace).await;
}

#[tokio::test]
async fn corpus_pass2_keeps_the_public_ceiling_for_local_dense_dependents() {
    let redis_url =
        env::var("REDIS_URL").expect("REDIS_URL must be set for integration tests");
    let namespace = format!("rsmq-corpus-cascade-local-{}", Uuid::new_v4().simple());
    let fixture = CascadeFixture::new(&namespace, false).await;

    let summary = fixture.run_pass2().await;
    assert_eq!(summary_count(&summary, "claimed"), 1, "{summary}");
    assert_eq!(summary_count(&summary, "done"), 0, "{summary}");
    assert_eq!(summary_count(&summary, "render_failed"), 1, "{summary}");
    let error_chain = summary["items"][0]["error_chain"]
        .as_str()
        .unwrap_or_else(|| panic!("failed item should carry its error chain: {summary}"));
    assert!(
        error_chain.contains("include expansion exceeded maximum total includes 256"),
        "a locally authored dependent must keep the public include ceiling: {error_chain}",
    );
    assert_eq!(
        fixture.dependent_leaf_count().await,
        1,
        "a rejected dependent render must leave its stored output unchanged",
    );

    cleanup_namespace(&redis_url, &namespace).await;
}
