/*
 * tests/list_pages.rs
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

#[macro_use]
mod common;

use self::common::TestRunner;
use deepwell::constants::{ADMIN_USER_ID, SYSTEM_USER_ID};
use deepwell::hash::k12_hash;
use deepwell::services::{RequestContext, TextService};
use deepwell::types::Reference;
use sea_orm::{ConnectionTrait, Statement, Value};
use serde_json::json;

async fn set_page_created_at(runner: &TestRunner, page_id: i64, created_at: &str) {
    let transaction = runner.context().transaction();
    let statement = Statement::from_string(
        transaction.get_database_backend(),
        format!(
            "UPDATE \"page\" SET created_at = TIMESTAMPTZ '{created_at}' WHERE page_id = {page_id}",
        ),
    );

    transaction
        .execute(statement)
        .await
        .expect("failed to set deterministic page creation timestamp");
}

#[tokio::test]
async fn exact_name_listpages_expands_created_at_and_rating() {
    const TARGET_SLUG: &str = "great-hippo-exact-name-target-3034";
    const SOURCE_SLUG: &str = "great-hippo-exact-name-smoke";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(TARGET_SLUG.into())),
    });
    let target = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "Target page body",
            "title": "Great Hippo Exact Name Target 3034",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "target for exact-name ListPages smoke test",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    set_page_created_at(&runner, target.page_id, "2017-05-16T16:02:00Z").await;

    let transaction = runner.context().transaction();
    transaction
        .execute(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "INSERT INTO page_vote (from_wikidot, page_id, user_id, value) VALUES (false, $1, $2, 1135)",
            [Value::from(target.page_id), Value::from(ADMIN_USER_ID)],
        ))
        .await
        .expect("deterministic legacy aggregate should be stored");

    let source = r#"Before
[[module ListPages name="Great Hippo Exact Name Target 3034"]]
%%created_at%% +%%rating%%
**[##grey|%%created_at%%##] [##green|+%%rating%%##]**
[[/module]]
After"#;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(SOURCE_SLUG.into())),
    });
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": source,
            "title": "Great Hippo exact name smoke",
            "alt_title": null,
            "slug": SOURCE_SLUG,
            "layout": "wikidot",
            "revision_comments": "exact-name ListPages smoke test",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let page = deepwell::endpoints::all::page_get(
        runner.context(),
        common::make_params(json!({
            "site_id": site_id,
            "page": SOURCE_SLUG,
            "details": {
                "compiled": true
            },
        })),
    )
    .await
    .expect("exact-name source page_get should succeed")
    .expect("exact-name source page_get should return page data");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("17 May 2017, 01:02"),
        "exact-name ListPages should expand created_at in Wikidot date format:\n{html}",
    );
    assert!(
        html.contains("+1135"),
        "exact-name ListPages should expand rating while preserving the template plus sign:\n{html}",
    );
    assert!(
        !html.contains("[[module ListPages"),
        "compiled output should not leak raw ListPages markup:\n{html}",
    );
    assert!(
        !html.contains("%%created_at%%") && !html.contains("%%rating%%"),
        "compiled output should not leak ListPages variables:\n{html}",
    );
}

#[tokio::test]
async fn exact_name_listpages_missing_page_renders_no_row() {
    const SOURCE_SLUG: &str = "great-hippo-missing-name-smoke";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let source = r#"Before
[[module ListPages name="SCP-DOES-NOT-EXIST"]]
MISSING ROW [%%created_at%%] [+%%rating%%]
[[/module]]
After"#;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(SOURCE_SLUG.into())),
    });
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": source,
            "title": "Great Hippo missing exact name smoke",
            "alt_title": null,
            "slug": SOURCE_SLUG,
            "layout": "wikidot",
            "revision_comments": "missing exact-name ListPages smoke test",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let page = deepwell::endpoints::all::page_get(
        runner.context(),
        common::make_params(json!({
            "site_id": site_id,
            "page": SOURCE_SLUG,
            "details": {
                "compiled": true
            },
        })),
    )
    .await
    .expect("missing exact-name source page_get should succeed")
    .expect("missing exact-name source page_get should return page data");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("Before"),
        "compiled output should keep prefix: {html}"
    );
    assert!(
        html.contains("After"),
        "compiled output should keep suffix: {html}"
    );
    assert!(
        !html.contains("MISSING ROW")
            && !html.contains("[[module ListPages")
            && !html.contains("%%created_at%%")
            && !html.contains("%%rating%%"),
        "missing exact-name ListPages should render zero rows without leaking metadata or raw module markup:\n{html}",
    );
}

#[tokio::test]
async fn wikidot_ajax_listpages_returns_unwrapped_client_rows() {
    const TARGET_SLUG: &str = "wikidot-ajax-listpages-target";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(TARGET_SLUG.into())),
    });
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "😀e\u{301}",
            "title": "AJAX ListPages Target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "AJAX ListPages compatibility smoke test",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: None,
    });
    let module_body = [
        "fullname",
        "category",
        "name",
        "title",
        "created_at",
        "created_by_linked",
        "updated_at",
        "updated_by_linked",
        "commented_at",
        "commented_by_linked",
        "parent_fullname",
        "comments",
        "size",
        "children",
        "rating",
        "rating_votes",
        "rating_percent",
        "revisions",
        "tags",
        "_tags",
    ]
    .into_iter()
    .map(|field| {
        format!(
            "[[span class=\"set {field}\"]][[span class=\"name\"]] {field} [[/span]][[span class=\"value\"]] %%{field}%% [[/span]][[/span]]"
        )
    })
    .collect::<String>();
    let output = run_endpoint!(
        runner,
        wikidot_list_pages_module,
        json!({
            "site_id": site_id,
            "module_body": format!("[[div class=\"page\"]]{module_body}[[/div]]"),
            "parameters": {
                "pagetype": "*",
                "category": "_default",
                "name": TARGET_SLUG,
                "order": "created_at desc",
                "offset": "0",
                "perPage": "250",
                "separate": "no",
                "wrapper": "no"
            }
        }),
    );

    assert!(
        output.body.contains(r#"class="page""#),
        "AJAX ListPages should retain the client-owned row wrapper: {}",
        output.body,
    );
    assert!(
        output.body.contains(&format!(
            r#"<span class="set fullname"><span class="name"> fullname </span><span class="value"> {TARGET_SLUG} </span></span>"#
        )),
        "AJAX ListPages should retain each client set name and value in one record: {}",
        output.body,
    );
    assert!(
        output.body.contains(TARGET_SLUG)
            && output.body.contains("AJAX ListPages Target"),
        "AJAX ListPages should substitute target page metadata: {}",
        output.body,
    );
    assert!(
        output.body.contains(
            r#"class="set size"><span class="name"> size </span><span class="value"> 3 </span>"#,
        ),
        "AJAX ListPages should count normalized saved-source Unicode scalar values: {}",
        output.body,
    );
    assert!(
        output.body.contains(r#"class="set category"><span class="name"> category </span><span class="value"> _default </span>"#),
        "AJAX ListPages should substitute the matched page category: {}",
        output.body,
    );
    assert!(
        !output.body.contains("list-pages-box")
            && !output.body.contains("list-pages-item")
            && !output.body.contains("[[module ListPages")
            && !output.body.contains("%%fullname%%"),
        "AJAX ListPages should honor wrapper=no and separate=no without leaking raw markers: {}",
        output.body,
    );
    let transient_hash = k12_hash(output.body.as_bytes());
    let transient_text_exists = TextService::exists(runner.context(), &transient_hash)
        .await
        .expect("text lookup should succeed");
    assert!(
        !transient_text_exists,
        "AJAX ListPages output should remain transient and avoid compiled text storage",
    );
}

#[tokio::test]
async fn countpages_static_filter_direct_fragment_renders_zero_without_raw_markers() {
    const SOURCE_SLUG: &str = "activity-marker-countpages-direct-smoke";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let source = r#"[[module CountPages tags="{$tag} -hub -artwork -artist" wrapper="no"]]
[[div_ class="activity-container [[#ifexpr %%total%% >= 60 | large-c | not-large-c ]] " data-number="%%total%%"]]
[[span class="large-marker"]]large canon[[/span]]
[[/div]]
[[/module]]"#;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(SOURCE_SLUG.into())),
    });
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": source,
            "title": "Activity Marker CountPages Direct Smoke",
            "alt_title": null,
            "slug": SOURCE_SLUG,
            "layout": "wikidot",
            "revision_comments": "direct fragment CountPages smoke test",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let page = deepwell::endpoints::all::page_get(
        runner.context(),
        common::make_params(json!({
            "site_id": site_id,
            "page": SOURCE_SLUG,
            "details": {
                "compiled": true
            },
        })),
    )
    .await
    .expect("CountPages source page_get should succeed")
    .expect("CountPages source page_get should return page data");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("activity-container not-large-c"),
        "CountPages direct fragment shape should resolve numeric ifexpr to not-large-c:\n{html}",
    );
    assert!(
        html.contains(r#"data-number="0""#),
        "CountPages direct fragment shape should substitute %%total%% as 0:\n{html}",
    );
    assert!(
        !html.contains("[[module CountPages")
            && !html.contains("[[/module]]")
            && !html.contains("%%total%%")
            && !html.contains("[[#ifexpr"),
        "compiled output should not leak raw CountPages markers:\n{html}",
    );
}

async fn execute_sql(runner: &TestRunner, sql: &str) {
    let transaction = runner.context().transaction();
    let statement =
        Statement::from_string(transaction.get_database_backend(), sql.to_owned());
    transaction
        .execute(statement)
        .await
        .expect("failed to execute test SQL");
}

async fn ensure_wikidot_import_snapshot_tables(runner: &TestRunner) {
    execute_sql(
        runner,
        r#"
        CREATE TABLE IF NOT EXISTS wikidot_corpus_import_run (
            import_run_id BIGSERIAL PRIMARY KEY,
            site_id BIGINT NOT NULL REFERENCES site(site_id),
            source_branch TEXT NOT NULL,
            source_site TEXT NOT NULL,
            manifest_sha256 BYTEA NOT NULL CHECK (octet_length(manifest_sha256) = 32),
            manifest_row_count BIGINT NOT NULL CHECK (manifest_row_count >= 0),
            complete_inventory BOOLEAN NOT NULL,
            state TEXT NOT NULL CHECK (state IN ('planning', 'running', 'metadata_done', 'rendering', 'done', 'failed')),
            started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            finished_at TIMESTAMPTZ,
            summary JSONB NOT NULL DEFAULT '{}'::JSONB
        )
        "#,
    )
    .await;
    execute_sql(
        runner,
        r#"
        CREATE TABLE IF NOT EXISTS wikidot_page_snapshot (
            page_id BIGINT PRIMARY KEY REFERENCES page(page_id) ON DELETE CASCADE,
            source_branch TEXT NOT NULL,
            source_site TEXT NOT NULL,
            source_entity_id UUID NOT NULL,
            source_fullname TEXT NOT NULL,
            source_created_at TIMESTAMPTZ NOT NULL,
            source_updated_at TIMESTAMPTZ NOT NULL,
            source_revision_count INTEGER NOT NULL CHECK (source_revision_count >= 0),
            imported_rating BIGINT NOT NULL,
            created_by_name TEXT,
            updated_by_name TEXT,
            title_shown TEXT,
            parent_fullname TEXT,
            comments INTEGER NOT NULL CHECK (comments >= 0),
            commented_at TIMESTAMPTZ,
            commented_by_name TEXT,
            source_sha256 BYTEA NOT NULL CHECK (octet_length(source_sha256) = 32),
            meta_sha256 BYTEA NOT NULL CHECK (octet_length(meta_sha256) = 32),
            meta_json JSONB NOT NULL,
            last_import_run_id BIGINT NOT NULL REFERENCES wikidot_corpus_import_run(import_run_id),
            imported_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(source_site, source_entity_id),
            UNIQUE(source_site, source_fullname)
        )
        "#,
    )
    .await;
}

#[tokio::test]
async fn imported_rating_baseline_adds_only_local_votes() {
    const TARGET_SLUG: &str = "scp-173-imported-rating-smoke";
    const IMPORT_RUN_ID: i64 = 7_700_001;

    let mut runner = TestRunner::setup().await;
    ensure_wikidot_import_snapshot_tables(&runner).await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(TARGET_SLUG.into())),
    });
    let target = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "Imported rating target",
            "title": "Imported Rating Target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "target for imported rating smoke test",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    execute_sql(
        &runner,
        &format!(
            r#"
            INSERT INTO wikidot_corpus_import_run (
                import_run_id,
                site_id,
                source_branch,
                source_site,
                manifest_sha256,
                manifest_row_count,
                complete_inventory,
                state,
                summary
            ) VALUES (
                {IMPORT_RUN_ID},
                {site_id},
                'en',
                'scp-wiki',
                decode(repeat('00', 32), 'hex'),
                1,
                false,
                'metadata_done',
                '{{}}'::jsonb
            )
            "#,
        ),
    )
    .await;
    execute_sql(
        &runner,
        &format!(
            r#"
            INSERT INTO wikidot_page_snapshot (
                page_id,
                source_branch,
                source_site,
                source_entity_id,
                source_fullname,
                source_created_at,
                source_updated_at,
                source_revision_count,
                imported_rating,
                created_by_name,
                updated_by_name,
                title_shown,
                parent_fullname,
                comments,
                commented_at,
                commented_by_name,
                source_sha256,
                meta_sha256,
                meta_json,
                last_import_run_id
            ) VALUES (
                {},
                'en',
                'scp-wiki',
                '11111111-1111-4111-8111-111111111111',
                '{TARGET_SLUG}',
                TIMESTAMPTZ '2008-07-25T20:49:21Z',
                TIMESTAMPTZ '2025-04-02T12:17:27Z',
                57,
                10634,
                NULL,
                'ParallelPotatoes',
                'SCP-173',
                NULL,
                2026,
                TIMESTAMPTZ '2026-04-13T11:29:27Z',
                'Ekaterina Komisch',
                decode(repeat('11', 32), 'hex'),
                decode(repeat('22', 32), 'hex'),
                '{{"fullname":"scp-173"}}'::jsonb,
                {IMPORT_RUN_ID}
            )
            "#,
            target.page_id,
        ),
    )
    .await;
    execute_sql(
        &runner,
        &format!(
            r#"
            INSERT INTO page_vote (from_wikidot, page_id, user_id, value)
            VALUES (true, {}, {SYSTEM_USER_ID}, 9999)
            "#,
            target.page_id,
        ),
    )
    .await;

    let baseline = run_endpoint!(
        runner,
        page_get_score,
        json!({"site_id": site_id, "page": TARGET_SLUG}),
    );
    assert_eq!(
        baseline.score,
        deepwell::services::score::ScoreValue::Integer(10634)
    );

    let vote = run_endpoint!(
        runner,
        vote_set,
        json!({
            "page_id": target.page_id,
            "user_id": ADMIN_USER_ID,
            "value": 1,
        }),
    )
    .expect("local vote should be accepted");
    assert_eq!(vote.value, 1);

    let score = run_endpoint!(
        runner,
        page_get_score,
        json!({"site_id": site_id, "page": TARGET_SLUG}),
    );
    assert_eq!(
        score.score,
        deepwell::services::score::ScoreValue::Integer(10635)
    );
}
