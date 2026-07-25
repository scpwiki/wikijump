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
use deepwell::constants::{ADMIN_USER_ID, SAMPLE_USER_ID, SYSTEM_USER_ID};
use deepwell::hash::k12_hash;
use deepwell::services::{RequestContext, TextService};
use deepwell::types::Reference;
use sea_orm::{ConnectionTrait, Statement, Value};
use serde_json::json;

/// Reassigns a page's creating revision to another user.
///
/// Page creation is permission-checked against the request actor, so a fixture
/// needing two distinct authors sets the stored author directly rather than
/// granting create rights to a second account.
async fn set_page_creating_user(runner: &TestRunner, page_id: i64, user_id: i64) {
    let transaction = runner.context().transaction();
    let statement = Statement::from_string(
        transaction.get_database_backend(),
        format!(
            "UPDATE page_revision SET user_id = {user_id} \
             WHERE page_id = {page_id} AND revision_number = 0",
        ),
    );

    transaction
        .execute(statement)
        .await
        .expect("failed to set deterministic page author");
}

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
    let created = run_endpoint!(
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

    // The live G37 target reported two revisions after one edit, so the title
    // change here leaves the saved source, and its %%size%%, untouched.
    run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": TARGET_SLUG,
            "last_revision_id": created.revision_id,
            "revision_comments": "retitle the AJAX ListPages target",
            "user_id": ADMIN_USER_ID,
            "title": "AJAX ListPages Target Revised",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("retitling the target should create a second revision");

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
        "created_by_unix",
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
        output.body.contains(
            r#"class="set created_by_unix"><span class="name"> created_by_unix </span><span class="value"> administrator </span>"#,
        ),
        "AJAX ListPages should emit the creator account unix name rather than the display name: {}",
        output.body,
    );
    assert!(
        output.body.contains(
            r#"class="set revisions"><span class="name"> revisions </span><span class="value"> 2 </span>"#,
        ),
        "AJAX ListPages should count the created and revised page's stored revisions: {}",
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

#[tokio::test]
async fn child_listpages_expands_site_domain_and_parent_fullname() {
    const PARENT_SLUG: &str = "component:offset-timeline-parity";
    const FIRST_CHILD_SLUG: &str = "fragment:offset-timeline-parity-0";
    const SECOND_CHILD_SLUG: &str = "fragment:offset-timeline-parity-1";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    macro_rules! create_page {
        ($slug:expr, $wikitext:expr $(,)?) => {{
            runner.set_request_context(RequestContext {
                session: None,
                user_id: Some(ADMIN_USER_ID),
                site_id: Some(site_id),
                page_reference: Some(Reference::Slug($slug.to_owned().into())),
            });
            run_endpoint!(
                runner,
                page_create,
                json!({
                    "site_id": site_id,
                    "wikitext": $wikitext,
                    "title": $slug,
                    "alt_title": null,
                    "slug": $slug,
                    "layout": "wikidot",
                    "revision_comments": "offset timeline navigation parity fixture",
                    "user_id": ADMIN_USER_ID,
                    "bypass_filter": true,
                    "ip_address": common::IP_ADDRESS,
                }),
            )
        }};
    }

    let parent = create_page!(PARENT_SLUG, "Placeholder body");
    create_page!(FIRST_CHILD_SLUG, "First offset");
    create_page!(SECOND_CHILD_SLUG, "Second offset");

    for child in [FIRST_CHILD_SLUG, SECOND_CHILD_SLUG] {
        runner.set_request_context(RequestContext {
            session: None,
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site_id),
            page_reference: Some(Reference::Slug(child.to_owned().into())),
        });
        run_endpoint!(
            runner,
            parent_set,
            json!({
                "site_id": site_id,
                "parent": PARENT_SLUG,
                "child": child,
            }),
        )
        .expect("parent relationship should be created");
    }

    // The live capture of component:offset-timeline builds each offset link from
    // %%site_domain%% plus %%parent_fullname%%, so the module is installed after
    // the children exist and are linked.
    let source = concat!(
        "[[module ListPages parent=\".\" category=\"fragment\" ",
        "order=\"created_at\" separate=\"no\"]]\n",
        "https://%%site_domain%%/%%parent_fullname%%/offset/%%title%%\n",
        "[[/module]]",
    );
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(PARENT_SLUG.into())),
    });
    run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": PARENT_SLUG,
            "last_revision_id": parent.revision_id,
            "revision_comments": "install offset timeline navigation module",
            "user_id": ADMIN_USER_ID,
            "wikitext": source,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("installing the navigation module should create a revision");

    let page = deepwell::endpoints::all::page_get(
        runner.context(),
        common::make_params(json!({
            "site_id": site_id,
            "page": PARENT_SLUG,
            "details": {
                "compiled": true
            },
        })),
    )
    .await
    .expect("navigation parity page_get should succeed")
    .expect("navigation parity page_get should return page data");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    for child in [FIRST_CHILD_SLUG, SECOND_CHILD_SLUG] {
        assert!(
            html.contains(&format!(
                "https://scp-wiki.wikidot.com/{PARENT_SLUG}/offset/{child}"
            )),
            "row {child} should build its offset link from the site domain and parent full name:\n{html}",
        );
    }
    assert!(
        !html.contains("%%site_domain%%") && !html.contains("%%parent_fullname%%"),
        "resolved navigation variables should not leak into the rendered body:\n{html}",
    );
}

#[tokio::test]
async fn no_tags_listpages_selects_only_untagged_pages() {
    const UNTAGGED_SLUG: &str = "no-tags-selector-untagged";
    const TAGGED_SLUG: &str = "no-tags-selector-tagged";
    const SOURCE_SLUG: &str = "no-tags-selector-source";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (slug, tags) in [(UNTAGGED_SLUG, vec![]), (TAGGED_SLUG, vec!["fixture"])] {
        runner.set_request_context(RequestContext {
            session: None,
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site_id),
            page_reference: Some(Reference::Slug(slug.into())),
        });
        run_endpoint!(
            runner,
            page_create,
            json!({
                "site_id": site_id,
                "wikitext": "No-tags selector fixture",
                "title": slug,
                "alt_title": null,
                "slug": slug,
                "tags": tags,
                "layout": "wikidot",
                "revision_comments": "no-tags ListPages selector fixture",
                "user_id": ADMIN_USER_ID,
                "bypass_filter": true,
                "ip_address": common::IP_ADDRESS,
            }),
        );
    }

    let source = concat!(
        "[[module ListPages category=\"_default\" tags=\"-\" ",
        "name=\"no-tags-selector-*\" separate=\"no\"]]\n",
        "ROW %%name%%\n",
        "[[/module]]",
    );
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
            "title": "No-tags ListPages selector",
            "alt_title": null,
            "slug": SOURCE_SLUG,
            "layout": "wikidot",
            "revision_comments": "no-tags ListPages selector smoke test",
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
    .expect("no-tags selector page_get should succeed")
    .expect("no-tags selector page_get should return page data");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains(UNTAGGED_SLUG),
        "the untagged page should match tags=\"-\":\n{html}",
    );
    assert!(
        !html.contains(TAGGED_SLUG),
        "a tagged page must not be returned by tags=\"-\":\n{html}",
    );
}

#[tokio::test]
async fn rating_order_listpages_sorts_by_descending_score() {
    const HIGH_SLUG: &str = "rating-order-high";
    const MID_SLUG: &str = "rating-order-mid";
    const LOW_SLUG: &str = "rating-order-low";
    const SOURCE_SLUG: &str = "rating-order-source";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (slug, rating) in [(LOW_SLUG, 3), (HIGH_SLUG, 129), (MID_SLUG, 49)] {
        runner.set_request_context(RequestContext {
            session: None,
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site_id),
            page_reference: Some(Reference::Slug(slug.into())),
        });
        let page = run_endpoint!(
            runner,
            page_create,
            json!({
                "site_id": site_id,
                "wikitext": "Rating order fixture",
                "title": slug,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "rating order ListPages fixture",
                "user_id": ADMIN_USER_ID,
                "bypass_filter": true,
                "ip_address": common::IP_ADDRESS,
            }),
        );

        let transaction = runner.context().transaction();
        transaction
            .execute(Statement::from_sql_and_values(
                transaction.get_database_backend(),
                "INSERT INTO page_vote (from_wikidot, page_id, user_id, value) VALUES (false, $1, $2, $3)",
                [
                    Value::from(page.page_id),
                    Value::from(ADMIN_USER_ID),
                    Value::from(rating),
                ],
            ))
            .await
            .expect("deterministic legacy aggregate should be stored");
    }

    let source = concat!(
        "[[module ListPages category=\"_default\" name=\"rating-order-*\" ",
        "order=\"rating desc\" separate=\"no\"]]\n",
        "ROW %%name%%\n",
        "[[/module]]",
    );
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
            "title": "Rating order ListPages",
            "alt_title": null,
            "slug": SOURCE_SLUG,
            "layout": "wikidot",
            "revision_comments": "rating order ListPages smoke test",
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
    .expect("rating order page_get should succeed")
    .expect("rating order page_get should return page data");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    let high = html
        .find(HIGH_SLUG)
        .expect("the highest rated row should render");
    let mid = html
        .find(MID_SLUG)
        .expect("the middle rated row should render");
    let low = html
        .find(LOW_SLUG)
        .expect("the lowest rated row should render");
    assert!(
        high < mid && mid < low,
        "rows should descend by rating:\n{html}",
    );
    assert!(
        !html.contains("[[module ListPages"),
        "a rating-ordered module should render rather than stay literal:\n{html}",
    );
}

#[tokio::test]
async fn link_to_listpages_selects_only_linking_pages() {
    const TARGET_SLUG: &str = "link-to-target";
    const LINKING_SLUG: &str = "link-to-linking";
    const UNRELATED_SLUG: &str = "link-to-unrelated";
    const SOURCE_SLUG: &str = "link-to-source";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (slug, wikitext) in [
        (TARGET_SLUG, "The link target".to_owned()),
        (
            LINKING_SLUG,
            format!("See [[[{TARGET_SLUG}]]] for details."),
        ),
        (UNRELATED_SLUG, "No internal links here.".to_owned()),
    ] {
        runner.set_request_context(RequestContext {
            session: None,
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site_id),
            page_reference: Some(Reference::Slug(slug.into())),
        });
        run_endpoint!(
            runner,
            page_create,
            json!({
                "site_id": site_id,
                "wikitext": wikitext,
                "title": slug,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "link_to ListPages fixture",
                "user_id": ADMIN_USER_ID,
                "bypass_filter": true,
                "ip_address": common::IP_ADDRESS,
            }),
        );
    }

    let source = format!(
        concat!(
            "[[module ListPages category=\"_default\" link_to=\"{}\" ",
            "separate=\"no\"]]\n",
            "ROW %%name%%\n",
            "[[/module]]",
        ),
        TARGET_SLUG,
    );
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
            "title": "link_to ListPages",
            "alt_title": null,
            "slug": SOURCE_SLUG,
            "layout": "wikidot",
            "revision_comments": "link_to ListPages smoke test",
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
    .expect("link_to page_get should succeed")
    .expect("link_to page_get should return page data");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains(&format!("ROW {LINKING_SLUG}")),
        "the linking page should match link_to:\n{html}",
    );
    assert!(
        !html.contains(&format!("ROW {UNRELATED_SLUG}")),
        "a page without the link must not be returned by link_to:\n{html}",
    );
}

#[tokio::test]
async fn listpages_total_counts_matches_beyond_the_rendered_page() {
    // Deliberately outside the selector glob below; a source page that matched
    // its own query would be counted among the results.
    const SOURCE_SLUG: &str = "total-window-holder";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for index in 0..4 {
        let slug = format!("total-beyond-page-{index}");
        runner.set_request_context(RequestContext {
            session: None,
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site_id),
            page_reference: Some(Reference::Slug(slug.clone().into())),
        });
        run_endpoint!(
            runner,
            page_create,
            json!({
                "site_id": site_id,
                "wikitext": "Total fixture",
                "title": slug,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "total ListPages fixture",
                "user_id": ADMIN_USER_ID,
                "bypass_filter": true,
                "ip_address": common::IP_ADDRESS,
            }),
        );
    }

    // Wikidot's tales-by-year renders one perPage window while %%total%%
    // reports every match, which is what lets the template number rows as
    // `%%total%% - %%index%% + 1`.
    let source = concat!(
        "[[module ListPages category=\"_default\" name=\"total-beyond-page-*\" ",
        "order=\"name\" limit=\"2\" separate=\"no\"]]\n",
        "ROW %%index%% OF %%total%%\n",
        "[[/module]]",
    );
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
            "title": "Total beyond page",
            "alt_title": null,
            "slug": SOURCE_SLUG,
            "layout": "wikidot",
            "revision_comments": "total ListPages smoke test",
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
    .expect("total page_get should succeed")
    .expect("total page_get should return page data");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("ROW 1 OF 4") && html.contains("ROW 2 OF 4"),
        "the two rendered rows should report all four matches:\n{html}",
    );
    assert!(
        !html.contains("ROW 3 OF"),
        "only the requested limit of rows should render:\n{html}",
    );
}

#[tokio::test]
async fn created_by_exclusion_omits_the_containing_pages_author() {
    const OWN_SLUG: &str = "author-exclusion-own";
    const OTHER_SLUG: &str = "author-exclusion-other";
    const SOURCE_SLUG: &str = "author-exclusion-source";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    // Both pages are created by the permitted account, then the other page's
    // stored author is reassigned, which is what the exclusion reads.
    for (slug, author) in [(OWN_SLUG, ADMIN_USER_ID), (OTHER_SLUG, SAMPLE_USER_ID)] {
        runner.set_request_context(RequestContext {
            session: None,
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site_id),
            page_reference: Some(Reference::Slug(slug.into())),
        });
        let created = run_endpoint!(
            runner,
            page_create,
            json!({
                "site_id": site_id,
                "wikitext": "Author exclusion fixture",
                "title": slug,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "author exclusion fixture",
                "user_id": ADMIN_USER_ID,
                "bypass_filter": true,
                "ip_address": common::IP_ADDRESS,
            }),
        );
        if author != ADMIN_USER_ID {
            set_page_creating_user(&runner, created.page_id, author).await;
        }
    }

    // The module lives on a page authored by ADMIN_USER_ID, so `-=` excludes
    // that author's pages and keeps the other author's.
    let source = concat!(
        "[[module ListPages category=\"_default\" name=\"author-exclusion-*\" ",
        "created_by=\"-=\" separate=\"no\"]]\n",
        "ROW %%name%%\n",
        "[[/module]]",
    );
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
            "title": "Author exclusion",
            "alt_title": null,
            "slug": SOURCE_SLUG,
            "layout": "wikidot",
            "revision_comments": "author exclusion smoke test",
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
    .expect("author exclusion page_get should succeed")
    .expect("author exclusion page_get should return page data");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains(&format!("ROW {OTHER_SLUG}")),
        "a page by another author should remain:\n{html}",
    );
    assert!(
        !html.contains(&format!("ROW {OWN_SLUG}")),
        "the excluded author's page must not be returned:\n{html}",
    );
}
