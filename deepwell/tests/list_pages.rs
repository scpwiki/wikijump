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
use deepwell::models::file;
use deepwell::services::blob::EMPTY_BLOB_HASH;
use deepwell::services::file_revision::CreateFirstFileRevision;
use deepwell::services::{
    FileRevisionService, RenderService, RequestContext, TextService,
};
use deepwell::types::Reference;
use sea_orm::{ActiveModelTrait, ConnectionTrait, Set, Statement, Value};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn listpages_retained_wikidot_unclosed_head_live_preview_boundaries() {
    // Independent anonymous Wikidot PagePreviewModule captures from
    // listpages-campaign-generated-live-preview.jsonl (2026-07-30):
    // lpgen-0140 (complete opener without closing module) executes its
    // empty/default query but leaves the row marker outside the module;
    // lpgen-0141 (incomplete opener itself) stays literal and never executes.
    // Do not treat this test as an owner of scanner.rs:1912-1913 until a
    // targeted mutation replay proves that those predicates affect it.
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");

    let complete = RenderService::render_wikidot_page_preview(
        runner.context(),
        site.site.site_id,
        "Wikidot retained ListPages unclosed complete head",
        "[[module ListPages category=\"fragment\"]]\n%%title%%".to_owned(),
    )
    .await
    .expect("the evidenced complete but unclosed opener should render")
    .html_output
    .body;
    assert!(
        complete.contains("list-pages-box") && complete.contains("<p>%%title%%</p>"),
        "an unclosed but complete opening executes while its row remains outside:\n{complete}"
    );
    assert!(
        !complete.contains("[[module ListPages"),
        "the executed opening must not leak as literal text:\n{complete}"
    );

    let incomplete = RenderService::render_wikidot_page_preview(
        runner.context(),
        site.site.site_id,
        "Wikidot retained ListPages incomplete head",
        "[[module ListPages category=\"fragment\"\n%%title%%".to_owned(),
    )
    .await
    .expect("the evidenced incomplete opener should remain authored text")
    .html_output
    .body;
    assert!(
        incomplete.contains("[[module ListPages category=&quot;fragment&quot;")
            && incomplete.contains("%%title%%")
            && !incomplete.contains("list-pages-box"),
        "an incomplete opener must remain literal rather than execute:\n{incomplete}"
    );

    runner.teardown().await;
}

#[tokio::test]
async fn block_modules_render_directly_inside_header_cells_and_list_items() {
    // Anonymous Wikidot PagePreviewModule captures (scp-wiki, 2026-10-11,
    // issue #2348): ListPages and Rate output inside [[hcell]] and [[li]] is a
    // direct child of <th>/<li>, exactly like the [[cell]] (<td>) control. The
    // internal compat placeholder must never reach the rendered body.
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");

    let list_pages = "[[module ListPages category=\"fragment\"]]\n%%title%%\n[[/module]]";
    let rate = "[[module Rate]]";
    for (module, rendered_box) in [
        (list_pages, r#"<div class="list-pages-box">"#),
        (rate, r#"<div class="page-rate-widget-box">"#),
    ] {
        for (source, parent) in [
            (
                format!(
                    "[[table]]\n[[row]]\n[[hcell]]\n{module}\n[[/hcell]]\n[[/row]]\n[[/table]]"
                ),
                "<th>",
            ),
            (
                format!(
                    "[[table]]\n[[row]]\n[[hcell class=\"namerow\" colspan=\"2\"]]\n{module}\n[[/hcell]]\n[[/row]]\n[[/table]]"
                ),
                r#"<th class="namerow" colspan="2">"#,
            ),
            (
                format!(
                    "[[table]]\n[[row]]\n[[cell]]\n{module}\n[[/cell]]\n[[/row]]\n[[/table]]"
                ),
                "<td>",
            ),
            (
                format!("[[ul]]\n[[li]]\n{module}\n[[/li]]\n[[/ul]]"),
                "<li>",
            ),
            (
                format!("[[ol]]\n[[li]]\n{module}\n[[/li]]\n[[/ol]]"),
                "<li>",
            ),
        ] {
            let body = RenderService::render_wikidot_page_preview(
                runner.context(),
                site.site.site_id,
                "block module container probe",
                source.clone(),
            )
            .await
            .expect("the block module container preview should render")
            .html_output
            .body;

            assert!(
                body.contains(&format!("{parent}{rendered_box}")),
                "{source}\n{body}"
            );
            assert!(
                !body.contains("WIKIJUMPWIKIDOTCOMPAT"),
                "the compat placeholder leaked:\n{source}\n{body}"
            );
        }
    }

    runner.teardown().await;
}

#[tokio::test]
async fn syntax_preview_preserves_delayed_listpages_literal() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");

    let body = RenderService::render_wikidot_syntax_preview(
        runner.context(),
        site.site.site_id,
        "syntax-only delayed ListPages",
        "[[module ListPages category=\"*\" perPage=\"1\"]]\n* %%title_linked%%\n[[/module]]"
            .to_owned(),
    )
    .await
    .expect("syntax-only preview should render")
    .html_output
    .body;

    assert!(
        !body.contains("list-pages-box"),
        "syntax-only preview must not execute ListPages: {body}"
    );
    assert_eq!(
        body,
        "<p>[[module ListPages category=&quot;*&quot; perPage=&quot;1&quot;]]</p><ul>\n<li>%%title_linked%%</li>\n</ul><p>[[/module]]</p>",
    );
    let endpoint = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site.site.site_id,
            "title": "syntax-only delayed ListPages endpoint",
            "wikitext": "[[module ListPages category=\"*\" perPage=\"1\"]]\n* %%title_linked%%\n[[/module]]",
            "syntax_only": true,
        }),
    );
    assert_eq!(endpoint.body, body);
    assert!(
        body.contains("[[module ListPages"),
        "syntax-only preview must preserve the delayed opener: {body}"
    );
}

#[tokio::test]
async fn syntax_preview_preserves_other_delayed_fixture_shapes() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    for (title, source, expected) in [
        (
            "syntax-only CountPages",
            "[[module CountPages category=\"*\"]]",
            "<p>[[module CountPages category=&quot;*&quot;]]</p>",
        ),
        (
            "syntax-only unknown module",
            "[[module UnknownOracleModule]]preserved body[[/module]]",
            "<p>[[module UnknownOracleModule]]preserved body[[/module]]</p>",
        ),
        (
            "syntax-only conditional",
            "[[if module UnknownOracleModule]]preserved conditional[[/if]]",
            "<p>[[if module UnknownOracleModule]]preserved conditional[[/if]]</p>",
        ),
    ] {
        let body = RenderService::render_wikidot_syntax_preview(
            runner.context(),
            site.site.site_id,
            title,
            source.to_owned(),
        )
        .await
        .expect("syntax-only delayed fixture should render")
        .html_output
        .body;
        assert_eq!(body, expected);
    }
}

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
        .execute_raw(statement)
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
        .execute_raw(statement)
        .await
        .expect("failed to set deterministic page creation timestamp");
}

async fn set_page_updated_at(runner: &TestRunner, page_id: i64, updated_at: &str) {
    let transaction = runner.context().transaction();
    let statement = Statement::from_string(
        transaction.get_database_backend(),
        format!(
            "UPDATE \"page\" SET updated_at = TIMESTAMPTZ '{updated_at}' WHERE page_id = {page_id}",
        ),
    );

    transaction
        .execute_raw(statement)
        .await
        .expect("failed to set deterministic page update timestamp");
}

async fn set_page_category(
    runner: &TestRunner,
    page_id: i64,
    site_id: i64,
    category_slug: &str,
) {
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "UPDATE page SET page_category_id = (
                SELECT category_id
                FROM page_category
                WHERE site_id = $1 AND slug = $2
            ) WHERE page_id = $3",
            [
                Value::from(site_id),
                Value::from(category_slug.to_owned()),
                Value::from(page_id),
            ],
        ))
        .await
        .expect("failed to set deterministic page category");
}

async fn create_listpages_file_fixture(
    runner: &TestRunner,
    site_id: i64,
    page_id: i64,
    name: &str,
    mime: &str,
) {
    let file = file::ActiveModel {
        name: Set(name.to_owned()),
        site_id: Set(site_id),
        page_id: Set(page_id),
        ..Default::default()
    }
    .insert(runner.context().transaction())
    .await
    .expect("ListPages file fixture should be inserted");
    FileRevisionService::create_first(
        runner.context(),
        CreateFirstFileRevision {
            site_id,
            page_id,
            file_id: file.file_id,
            user_id: ADMIN_USER_ID,
            name: name.to_owned(),
            s3_hash: EMPTY_BLOB_HASH,
            size: 0,
            mime: mime.to_owned(),
            content_type: None,
            blob_created: false,
            revision_comments: "create ListPages file fixture".to_owned(),
        },
    )
    .await
    .expect("ListPages file revision fixture should be created");
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
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "INSERT INTO page_vote (from_wikidot, page_id, user_id, value) VALUES (false, $1, $2, 1135)",
            [Value::from(target.page_id), Value::from(ADMIN_USER_ID)],
        ))
        .await
        .expect("deterministic legacy aggregate should be stored");

    let source = r#"Before
[[module ListPages name="Great Hippo Exact Name Target 3034"]]
%%title_linked%%
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
        html.contains(
            r#"<a href="/great-hippo-exact-name-target-3034">Great Hippo Exact Name Target 3034</a>"#,
        ),
        "the delayed linked-title row should bind its typed page link:\n{html}",
    );
    assert!(
        !html.contains("[[module ListPages"),
        "compiled output should not leak raw ListPages markup:\n{html}",
    );
    assert!(
        !html.contains("%%created_at%%") && !html.contains("%%rating%%"),
        "compiled output should not leak ListPages variables:\n{html}",
    );
    assert!(
        !html.contains(r#"style="cursor: help; display: inline;""#),
        "fresh Wikidot preview evidence has no invented inline ODate style:\n{html}",
    );
    assert!(
        !html.contains("data-wikijump-compat-date"),
        "the internal ODate trust marker must not leak from delayed rows:\n{html}",
    );
}

#[tokio::test]
async fn literal_owned_registered_listpages_openers_remain_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (case_id, slug, source, expected_open, expected_close) in [
        (
            "escaped-inline",
            "literal-listpages-escaped-inline",
            "\\[[module ListPages limit=\"1\"]]body[[/module]]",
            r#"\[[module ListPages limit=&quot;1&quot;]]"#,
            "[[/module]]",
        ),
        (
            "escaped-own-line",
            "literal-listpages-escaped-own-line",
            "BEFORE\n\\[[module ListPages limit=\"1\"]]body[[/module]]\nAFTER",
            r#"\[[module ListPages limit=&quot;1&quot;]]"#,
            "[[/module]]",
        ),
        (
            "inline-raw",
            "literal-listpages-inline-raw",
            "@@[[module ListPages limit=\"1\"]]\n[[/module]]@@",
            r#"@@[[module ListPages limit=&quot;1&quot;]]"#,
            "[[/module]]@@",
        ),
    ] {
        let preview = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            case_id,
            source.to_owned(),
        )
        .await
        .expect("literal-owned ListPages source should render in preview")
        .html_output
        .body;

        assert!(
            preview.contains(expected_open) && preview.contains(expected_close),
            "{case_id} source must remain visible in preview: {preview}",
        );
        assert!(
            !preview.contains("list-pages-box")
                && !preview.contains("list-pages-item")
                && !preview.contains("TODO: module ListPages"),
            "{case_id} must not dispatch ListPages in preview: {preview}",
        );

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
                "wikitext": source,
                "title": case_id,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "literal-owner ListPages regression",
                "user_id": ADMIN_USER_ID,
                "bypass_filter": true,
                "ip_address": common::IP_ADDRESS,
            }),
        );
        let saved = run_endpoint!(
            runner,
            page_get,
            json!({
                "site_id": site_id,
                "page": slug,
                "details": {"compiled": true},
            }),
        )
        .expect("saved literal-owner page should exist")
        .compiled_body_html
        .expect("saved literal-owner page should have compiled HTML");
        assert!(
            saved.contains(expected_open) && saved.contains(expected_close),
            "{case_id} source must remain visible after save: {saved}",
        );
        assert!(
            !saved.contains("list-pages-box")
                && !saved.contains("list-pages-item")
                && !saved.contains("TODO: module ListPages"),
            "{case_id} must not dispatch ListPages after save: {saved}",
        );
    }

    let unescaped = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unescaped ListPages preview",
        "[[module ListPages limit=\"1\"]]body[[/module]]".to_owned(),
    )
    .await
    .expect("unescaped ListPages source should render")
    .html_output
    .body;
    assert!(unescaped.contains("list-pages-box"), "{unescaped}");
}

#[tokio::test]
async fn listpages_updated_at_preview_keeps_server_date_spaces_breakable() {
    const TARGET_SLUG: &str = "listpages-date-space-target-20260802";

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
            "wikitext": "Date-space target",
            "title": "ListPages date-space target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "ListPages date-space fixture",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    set_page_updated_at(&runner, target.page_id, "2026-07-29T02:21:00Z").await;

    let html = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages date-space preview",
        format!(
            concat!(
                "[[module ListPages name=\"{}\" limit=\"1\"]]\n",
                "||~ Today's date is: ||\n",
                "||= %%updated_at|%d %B %Y%% ||\n",
                "[[/module]]",
            ),
            TARGET_SLUG,
        ),
    )
    .await
    .expect("ListPages updated_at preview should render")
    .html_output
    .body;

    assert!(
        html.contains("29 Jul 2026 02:21") && !html.contains("29 Jul 2026\u{00a0}02:21"),
        "the server-rendered ODate text must keep ordinary breakable spaces:\n{html}",
    );
}

#[tokio::test]
async fn listpages_title_order_is_case_insensitive() {
    const CATEGORY: &str = "listpages-title-order";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (slug, title) in [
        ("listpages-title-order:3law", "3law Japanese preview"),
        (
            "listpages-title-order:3law-7ow",
            "3law-7ow Japanese preview",
        ),
        (
            "listpages-title-order:3lawcracked",
            "3lawcracked Japanese preview",
        ),
        (
            "listpages-title-order:3law-pride",
            "3law-pride Japanese preview",
        ),
        ("listpages-title-order:aces", "Aces"),
        ("listpages-title-order:anon", "Anon"),
        ("listpages-title-order:a-problematic", "A-problematic"),
        ("listpages-title-order:ar", "Ar"),
        ("listpages-title-order:creating-pages", "Creating Pages"),
        ("listpages-title-order:css-themes", "CSS Themes"),
        ("listpages-title-order:template", "_template"),
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
                "wikitext": "ListPages title-order fixture.",
                "title": title,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "ListPages case-insensitive title-order fixture",
                "user_id": ADMIN_USER_ID,
                "bypass_filter": true,
                "ip_address": common::IP_ADDRESS,
            }),
        );
    }

    let html = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages title order",
        format!(
            "[[module ListPages category=\"{CATEGORY}\" order=\"title\" \
             separate=\"no\"]]\n%%title%%\n[[/module]]",
        ),
    )
    .await
    .expect("ListPages title-order preview should render")
    .html_output
    .body;

    let ordered_titles = [
        "3law Japanese preview",
        "3law-7ow Japanese preview",
        "3lawcracked Japanese preview",
        "3law-pride Japanese preview",
        "Aces",
        "Anon",
        "A-problematic",
        "Ar",
        "Creating Pages",
        "CSS Themes",
        "_template",
    ];
    let positions = ordered_titles.map(|title| {
        html.find(title)
            .unwrap_or_else(|| panic!("{title} row should render:\n{html}"))
    });
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "live Wikidot title order is case-insensitive and ignores punctuation for its primary collation key:\n{html}",
    );
}

#[tokio::test]
async fn linked_listpages_values_keep_authored_row_block_syntax() {
    const TARGET_SLUG: &str = "listpages-linked-block-row-target";

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
            "wikitext": "ListPages linked block row target",
            "title": "ListPages Linked Block Row Target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "ListPages linked block row fixture",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages linked block row",
        format!(
            concat!(
                "[[module ListPages name=\"{}\" separate=\"no\"]]\n",
                "[[div class=\"card-block\"]]\n",
                "%%title_linked%%\n",
                "[[/div]]\n",
                "[[/module]]",
            ),
            TARGET_SLUG
        ),
    )
    .await
    .expect("a linked value inside authored ListPages block syntax should render")
    .html_output
    .body;

    assert!(
        preview.contains(r#"<div class="card-block">"#)
            && preview.contains(&format!(r#"href="/{TARGET_SLUG}""#))
            && !preview.contains("[[div"),
        "typed linked slots must not make their authored row block syntax literal:\n{preview}",
    );
}

#[tokio::test]
async fn delayed_listpages_rows_keep_wikidot_whitespace_boundaries() {
    const TARGET_SLUG: &str = "listpages-delayed-whitespace-target";

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
            "wikitext": "ListPages delayed whitespace target",
            "title": "ListPages Delayed Whitespace Target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "ListPages delayed whitespace fixture",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages delayed whitespace row",
        format!(
            concat!(
                "[[module ListPages name=\"{}\" separate=\"no\"]]\n",
                "ROW %%updated_at%%\n",
                "     \n",
                "[[=]]\n",
                "CENTER\n",
                "[[/=]]\n",
                "> = ##brown|**%%updated_at%%**##\n",
                "En date: [[span style=\"color: rgb(255, 230, 0);\"]] JAUNE [[/span]].\n",
                "[[/module]]",
            ),
            TARGET_SLUG,
        ),
    )
    .await
    .expect("a delayed row with runtime scalars and legacy whitespace should render")
    .html_output
    .body;

    assert!(
        !preview.contains("<br>"),
        "a spaces-only source line must not become an active break after scalar binding:\n{preview}",
    );
    assert!(
        preview.contains(r#"<div style="text-align: center;"><p>CENTER</p></div>"#),
        "the centered block after a delayed scalar should retain its block boundary:\n{preview}",
    );
    assert!(
        preview.contains(r#"<p style="text-align: center;">"#)
            && !preview.contains("<p>= ")
            && !preview.contains("<p> = "),
        "the centered quote marker must not become visible row text:\n{preview}",
    );
    assert!(
        preview.contains(r#">JAUNE</span> .</p>"#),
        "an authored space before the span closer belongs outside the closed span:\n{preview}",
    );

    let nbsp_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages delayed NBSP row",
        format!(
            concat!(
                "[[module ListPages name=\"{}\" separate=\"no\"]]\n",
                "HEAD %%updated_at%%\n",
                "\u{00a0}\n",
                "TAIL\n",
                "[[/module]]",
            ),
            TARGET_SLUG,
        ),
    )
    .await
    .expect("a delayed row should preserve an NBSP-only source line")
    .html_output
    .body;
    assert!(
        nbsp_preview.contains('\u{00a0}'),
        "an NBSP-only delayed source line remains authored content:\n{nbsp_preview}",
    );
    assert!(
        !nbsp_preview.contains("<p>&nbsp;</p>"),
        "the NBSP-only line stays in the surrounding delayed paragraph:\n{nbsp_preview}",
    );

    let static_quote_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages static quote row",
        format!(
            concat!(
                r#"[[module ListPages name=\"{}\" category=\"*\" limit="1"]]"#,
                "\n> = ##brown|**DATE**##\n",
                "[[/module]]",
            ),
            TARGET_SLUG,
        ),
    )
    .await
    .expect("the live-observed legacy opener and static quote row should render")
    .html_output
    .body;

    assert!(
        static_quote_preview.contains(r#"<blockquote><p style="text-align: center;">"#)
            && !static_quote_preview.contains("<p>= ")
            && !static_quote_preview.contains("<p> = "),
        "a static centered quote row must not expose its alignment marker:\n{static_quote_preview}",
    );
}

#[tokio::test]
async fn delayed_listpages_rows_keep_authored_html_blocks_literal_in_preview() {
    const TARGET_SLUG: &str = "listpages-delayed-html-block-target";

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
            "wikitext": "ListPages delayed HTML block target",
            "title": "ListPages Delayed HTML Block Target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "ListPages delayed HTML block fixture",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages delayed HTML block row",
        format!(
            concat!(
                "[[module ListPages name=\"{}\" separate=\"no\"]]\n",
                "%%title%%\n",
                "[[html]]<strong>ROW_HTML_PAYLOAD</strong>[[/html]]\n",
                "[[/module]]",
            ),
            TARGET_SLUG,
        ),
    )
    .await
    .expect("an authored HTML block in a delayed ListPages row should render")
    .html_output
    .body;

    assert!(
        preview.contains("[[html]]")
            && preview.contains("[[/html]]")
            && preview.contains("ListPages Delayed HTML Block Target")
            && preview.contains("&lt;strong&gt;ROW_HTML_PAYLOAD&lt;/strong&gt;")
            && !preview.contains("%%title%%")
            && !preview.contains("[[module ListPages")
            && !preview.contains("html-block-iframe"),
        "PagePreview must keep delayed ListPages HTML blocks literal:\n{preview}",
    );
}

#[tokio::test]
async fn listpages_section_zero_html_opener_keeps_preview_literal_and_saved_html_executable()
 {
    const SOURCE_SLUG: &str = "listpages-section-zero-html-opener";
    const SOURCE: &str = concat!(
        "[[module ListPages fullname=\"scp-002\" separate=\"no\" wrapper=\"no\"]]\n",
        "[[%%content{0}%%html]]<strong>GENERATED_HTML</strong>[[%%content{0}%%/html]]\n",
        "[[/module]]",
    );

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages section-zero HTML opener preview",
        SOURCE.to_owned(),
    )
    .await
    .expect("section-zero HTML opener preview should render")
    .html_output
    .body;
    assert!(
        preview.contains("[[html]]&lt;strong&gt;GENERATED_HTML&lt;/strong&gt;[[/html]]")
            && !preview.contains("html-block-iframe")
            && !preview.contains("%%content{0}%%"),
        "PagePreview must keep generated section-zero HTML literal:
{preview}",
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
            "wikitext": SOURCE,
            "title": "ListPages section-zero HTML opener",
            "alt_title": null,
            "slug": SOURCE_SLUG,
            "layout": "wikidot",
            "revision_comments": "section-zero generated HTML regression",
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
            "details": {"compiled": true},
        })),
    )
    .await
    .expect("saved section-zero page_get should succeed")
    .expect("saved section-zero page should exist");
    let saved = page
        .compiled_body_html
        .expect("saved section-zero page should have compiled HTML");
    assert!(
        saved.contains(r#"class="html-block-iframe""#)
            && !saved.contains("[[html]]")
            && !saved.contains("GENERATED_HTML"),
        "saved pages must execute generated section-zero HTML through the iframe boundary:
{saved}",
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
async fn listpages_category_local_names_match_page_identities_and_wildcard_boundaries() {
    const TARGET_SLUG: &str = "component:listpages-category-name-block";
    const SECONDARY_SLUG: &str = "component:listpages-category-name-base";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (slug, title) in [
        (TARGET_SLUG, "ListPages category name block"),
        (SECONDARY_SLUG, "ListPages category name base"),
    ] {
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
                "wikitext": "Category-local ListPages target",
                "title": title,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "category-local ListPages selector fixture",
                "user_id": ADMIN_USER_ID,
                "bypass_filter": true,
                "ip_address": common::IP_ADDRESS,
            }),
        );
        set_page_category(&runner, page.page_id, site_id, "component").await;
    }

    for (attributes, expected_rows, absent_rows) in [
        (
            r#"category="component" name="component:listpages-category-name-block""#,
            &[TARGET_SLUG][..],
            &[SECONDARY_SLUG][..],
        ),
        (
            r#"category="+Component" name=" listpages-category-name-block ""#,
            &[TARGET_SLUG][..],
            &[SECONDARY_SLUG][..],
        ),
        (
            r#"category="component" name="listpages-category-name?block""#,
            &[TARGET_SLUG][..],
            &[SECONDARY_SLUG][..],
        ),
        (
            r#"category="component" name="listpages-category-name-*""#,
            &[SECONDARY_SLUG, TARGET_SLUG][..],
            &[][..],
        ),
        (
            r#"category="component" name="*block""#,
            &[][..],
            &[TARGET_SLUG, SECONDARY_SLUG][..],
        ),
    ] {
        let source = format!(
            "[[module ListPages {attributes} order=\"name\" separate=\"no\"]]\nROW %%fullname%%\n[[/module]]",
        );
        let html = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "Unsaved preview",
            source,
        )
        .await
        .expect("category-local ListPages preview should render")
        .html_output
        .body;

        for expected in expected_rows {
            assert!(
                html.contains(&format!("ROW {expected}")),
                "{attributes:?} should select {expected}:\n{html}",
            );
        }
        for absent in absent_rows {
            assert!(
                !html.contains(&format!("ROW {absent}")),
                "{attributes:?} must not select {absent}:\n{html}",
            );
        }
    }

    runner.teardown().await;
}

#[tokio::test]
async fn listpages_multi_zero_page_size_keeps_once_only_output_without_rows() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");

    let html = RenderService::render_wikidot_page_preview(
        runner.context(),
        site.site.site_id,
        "Unsaved preview",
        r#"[[module ListPages category="*" perPage="00" separate="no" prependLine="PRE" appendLine="POST"]]
ROW %%fullname%%
[[/module]]"#
            .to_owned(),
    )
    .await
    .expect("multi-zero perPage should render without dividing by zero")
    .html_output
    .body;

    assert!(
        html.contains("PRE"),
        "prependLine should remain visible:\n{html}"
    );
    assert!(
        html.contains("POST"),
        "appendLine should remain visible:\n{html}"
    );
    assert!(
        !html.contains("ROW ")
            && !html.contains(r#"class="pager""#)
            && !html.contains("[[module ListPages"),
        "zero page size must not render rows, a pager, or raw source:\n{html}",
    );

    runner.teardown().await;
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
            "tags": ["visible-listpages-tag", "_hidden-listpages-tag"],
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
            "module_body": format!("[[div class=\"page\"]]\n{module_body}\n[[/div]]"),
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
            r#"<span class="set fullname"><span class="name">fullname</span> <span class="value">{TARGET_SLUG}</span></span>"#
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
            r#"class="set size"><span class="name">size</span> <span class="value">3</span>"#,
        ),
        "AJAX ListPages should count normalized saved-source Unicode scalar values: {}",
        output.body,
    );
    assert!(
        output.body.contains(
            r#"class="set created_by_unix"><span class="name">created_by_unix</span> <span class="value">administrator</span>"#,
        ),
        "AJAX ListPages should emit the creator account unix name rather than the display name: {}",
        output.body,
    );
    assert!(
        output.body.contains(
            r#"class="set revisions"><span class="name">revisions</span> <span class="value">2</span>"#,
        ),
        "AJAX ListPages should count the created and revised page's stored revisions: {}",
        output.body,
    );
    assert!(
        output.body.contains(r#"class="set category"><span class="name">category</span> <span class="value">_default</span>"#),
        "AJAX ListPages should substitute the matched page category: {}",
        output.body,
    );
    assert!(
        output.body.contains(
            r#"class="set tags"><span class="name">tags</span> <span class="value">visible-listpages-tag</span>"#,
        ),
        "AJAX ListPages should emit only visible tags through %%tags%%: {}",
        output.body,
    );
    assert!(
        output.body.contains(
            r#"class="set _tags"><span class="name">_tags</span> <span class="value">_hidden-listpages-tag</span>"#,
        ),
        "AJAX ListPages should emit only hidden tags through %%_tags%%: {}",
        output.body,
    );
    assert!(
        output
            .body
            .contains(r#"<span class="value"><span class="odate time_"#)
            && !output.body.contains("&lt;span class=&quot;odate"),
        "AJAX ListPages should emit generated date markup as nested HTML: {}",
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
        .execute_raw(statement)
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
    execute_sql(
        runner,
        r#"
        ALTER TABLE wikidot_page_snapshot
        ADD COLUMN IF NOT EXISTS wikidot_size BIGINT
            CHECK (wikidot_size >= 0)
        "#,
    )
    .await;
}

#[tokio::test]
async fn imported_wikidot_size_controls_size_order_and_substitution() {
    const CATEGORY: &str = "listpages-imported-size";
    const LARGE_SLUG: &str = "listpages-imported-size:live-large";
    const SMALL_SLUG: &str = "listpages-imported-size:live-small";
    const IMPORT_RUN_ID: i64 = 7_700_002;

    let mut runner = TestRunner::setup().await;
    ensure_wikidot_import_snapshot_tables(&runner).await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let mut created_pages = Vec::new();
    for (slug, wikitext, title) in [
        (LARGE_SLUG, "x", "Imported Live Large"),
        (
            SMALL_SLUG,
            "this local source is deliberately much longer",
            "Imported Live Small",
        ),
    ] {
        runner.set_request_context(RequestContext {
            session: None,
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site_id),
            page_reference: Some(Reference::Slug(slug.into())),
        });
        created_pages.push(run_endpoint!(
            runner,
            page_create,
            json!({
                "site_id": site_id,
                "wikitext": wikitext,
                "title": title,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "imported Wikidot size compatibility fixture",
                "user_id": ADMIN_USER_ID,
                "bypass_filter": true,
                "ip_address": common::IP_ADDRESS,
            }),
        ));
    }
    let large = &created_pages[0];
    let small = &created_pages[1];

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
                'size-fixture',
                'sandbox-for-codex',
                decode(repeat('33', 32), 'hex'),
                2,
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
                last_import_run_id,
                wikidot_size
            ) VALUES
            (
                {},
                'size-fixture',
                'sandbox-for-codex',
                '22222222-2222-4222-8222-222222222222',
                '{LARGE_SLUG}',
                TIMESTAMPTZ '2026-05-26T00:00:00Z',
                TIMESTAMPTZ '2026-05-26T00:00:00Z',
                1,
                0,
                NULL,
                NULL,
                'Imported Live Large',
                NULL,
                0,
                NULL,
                NULL,
                decode(repeat('44', 32), 'hex'),
                decode(repeat('55', 32), 'hex'),
                '{{"fullname":"{LARGE_SLUG}","size":100}}'::jsonb,
                {IMPORT_RUN_ID},
                100
            ),
            (
                {},
                'size-fixture',
                'sandbox-for-codex',
                '33333333-3333-4333-8333-333333333333',
                '{SMALL_SLUG}',
                TIMESTAMPTZ '2026-05-26T00:00:01Z',
                TIMESTAMPTZ '2026-05-26T00:00:01Z',
                1,
                0,
                NULL,
                NULL,
                'Imported Live Small',
                NULL,
                0,
                NULL,
                NULL,
                decode(repeat('66', 32), 'hex'),
                decode(repeat('77', 32), 'hex'),
                '{{"fullname":"{SMALL_SLUG}","size":2}}'::jsonb,
                {IMPORT_RUN_ID},
                2
            )
            "#,
            large.page_id, small.page_id,
        ),
    )
    .await;

    let html = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Imported size preview",
        format!(
            "[[module ListPages category=\"{CATEGORY}\" order=\"size desc\" \
             separate=\"no\"]]\n%%name%%|%%size%%\n[[/module]]",
        ),
    )
    .await
    .expect("imported size ListPages preview should render")
    .html_output
    .body;

    let large_row = html
        .find("live-large|100")
        .unwrap_or_else(|| panic!("the imported large size should render:\n{html}"));
    let small_row = html
        .find("live-small|2")
        .unwrap_or_else(|| panic!("the imported small size should render:\n{html}"));
    assert!(
        large_row < small_row,
        "Wikidot's captured page size must control both ordering and %%size%% \
         even when the normalized local source lengths imply the opposite:\n{html}",
    );

    runner.teardown().await;
}

#[tokio::test]
async fn imported_wikidot_revision_count_controls_revisions_order_and_substitution() {
    const CATEGORY: &str = "listpages-imported-revisions";
    const LARGE_SLUG: &str = "listpages-imported-revisions:z-live-large";
    const TIED_SLUG: &str = "listpages-imported-revisions:a-live-tied";
    const SMALL_SLUG: &str = "listpages-imported-revisions:a-live-small";
    const IMPORT_RUN_ID: i64 = 7_700_003;

    let mut runner = TestRunner::setup().await;
    ensure_wikidot_import_snapshot_tables(&runner).await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let mut created_pages = Vec::new();
    for (slug, title) in [
        (LARGE_SLUG, "Imported Live Revision Large"),
        (TIED_SLUG, "Imported Live Revision Tied"),
        (SMALL_SLUG, "Imported Live Revision Small"),
    ] {
        runner.set_request_context(RequestContext {
            session: None,
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site_id),
            page_reference: Some(Reference::Slug(slug.into())),
        });
        created_pages.push(run_endpoint!(
            runner,
            page_create,
            json!({
                "site_id": site_id,
                "wikitext": "one normalized local revision",
                "title": title,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "imported Wikidot revision-count compatibility fixture",
                "user_id": ADMIN_USER_ID,
                "bypass_filter": true,
                "ip_address": common::IP_ADDRESS,
            }),
        ));
    }
    let large = &created_pages[0];
    let tied = &created_pages[1];
    let small = &created_pages[2];

    execute_sql(
        &runner,
        &format!(
            "UPDATE page SET from_wikidot = true WHERE page_id IN ({}, {}, {})",
            large.page_id, tied.page_id, small.page_id,
        ),
    )
    .await;
    execute_sql(
        &runner,
        &format!(
            "UPDATE page_revision SET from_wikidot = true WHERE revision_id IN ({}, {}, {})",
            large.revision_id, tied.revision_id, small.revision_id,
        ),
    )
    .await;

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
                'revision-count-fixture',
                'sandbox-for-codex',
                decode(repeat('88', 32), 'hex'),
                3,
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
            ) VALUES
            (
                {},
                'revision-count-fixture',
                'sandbox-for-codex',
                '44444444-4444-4444-8444-444444444444',
                '{LARGE_SLUG}',
                TIMESTAMPTZ '2026-05-26T00:00:00Z',
                TIMESTAMPTZ '2026-05-26T00:00:00Z',
                53,
                0,
                NULL,
                NULL,
                'Imported Live Revision Large',
                NULL,
                0,
                NULL,
                NULL,
                decode(repeat('99', 32), 'hex'),
                decode(repeat('aa', 32), 'hex'),
                '{{"fullname":"{LARGE_SLUG}","page_id":200,"revision_count":53}}'::jsonb,
                {IMPORT_RUN_ID}
            ),
            (
                {},
                'revision-count-fixture',
                'sandbox-for-codex',
                '55555555-5555-4555-8555-555555555555',
                '{TIED_SLUG}',
                TIMESTAMPTZ '2026-05-26T00:00:01Z',
                TIMESTAMPTZ '2026-05-26T00:00:01Z',
                53,
                0,
                NULL,
                NULL,
                'Imported Live Revision Tied',
                NULL,
                0,
                NULL,
                NULL,
                decode(repeat('bb', 32), 'hex'),
                decode(repeat('cc', 32), 'hex'),
                '{{"fullname":"{TIED_SLUG}","page_id":100,"revision_count":53}}'::jsonb,
                {IMPORT_RUN_ID}
            ),
            (
                {},
                'revision-count-fixture',
                'sandbox-for-codex',
                '66666666-6666-4666-8666-666666666666',
                '{SMALL_SLUG}',
                TIMESTAMPTZ '2026-05-26T00:00:02Z',
                TIMESTAMPTZ '2026-05-26T00:00:02Z',
                2,
                0,
                NULL,
                NULL,
                'Imported Live Revision Small',
                NULL,
                0,
                NULL,
                NULL,
                decode(repeat('dd', 32), 'hex'),
                decode(repeat('ee', 32), 'hex'),
                '{{"fullname":"{SMALL_SLUG}","page_id":50,"revision_count":2}}'::jsonb,
                {IMPORT_RUN_ID}
            )
            "#,
            large.page_id, tied.page_id, small.page_id,
        ),
    )
    .await;

    let html = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Imported revision count preview",
        format!(
            "[[module ListPages category=\"{CATEGORY}\" order=\"revisions desc\" \
             separate=\"no\"]]\n%%name%%|%%revisions%%\n[[/module]]",
        ),
    )
    .await
    .expect("imported revision-count ListPages preview should render")
    .html_output
    .body;

    let large_row = html.find("z-live-large|53").unwrap_or_else(|| {
        panic!("the imported large revision count should render:\n{html}")
    });
    let small_row = html.find("a-live-small|2").unwrap_or_else(|| {
        panic!("the imported small revision count should render:\n{html}")
    });
    let tied_row = html.find("a-live-tied|53").unwrap_or_else(|| {
        panic!("the tied imported revision count should render:\n{html}")
    });
    assert!(
        large_row < tied_row && tied_row < small_row,
        "Wikidot's captured revision count must control both ordering and \
         %%revisions%%, with live page identity breaking equal-count ties in \
         the same direction, even when normalized local histories tie:\n{html}",
    );

    runner.teardown().await;
}

#[tokio::test]
async fn imported_revision_count_adds_local_lifecycle_revisions_for_order_and_substitution()
 {
    const CATEGORY: &str = "listpages-imported-revision-lifecycle";
    const TARGET_SLUG: &str = "listpages-imported-revision-lifecycle:imported-target";
    const MOVED_SLUG: &str =
        "listpages-imported-revision-lifecycle:imported-target-moved";
    const PEER_SLUG: &str = "listpages-imported-revision-lifecycle:imported-peer";
    const LOCAL_SLUG: &str = "listpages-imported-revision-lifecycle:local-page";
    const IMPORT_RUN_ID: i64 = 7_700_004;

    let mut runner = TestRunner::setup().await;
    ensure_wikidot_import_snapshot_tables(&runner).await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let mut pages = Vec::new();
    for (slug, title) in [
        (TARGET_SLUG, "Imported revision lifecycle target"),
        (PEER_SLUG, "Imported revision lifecycle peer"),
        (LOCAL_SLUG, "Local revision lifecycle control"),
    ] {
        runner.set_request_context(RequestContext {
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site_id),
            page_reference: Some(Reference::Slug(slug.into())),
            ..Default::default()
        });
        pages.push(run_endpoint!(
            runner,
            page_create,
            json!({
                "site_id": site_id,
                "wikitext": format!("{title} body"),
                "title": title,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "create revision lifecycle fixture",
                "user_id": ADMIN_USER_ID,
                "bypass_filter": true,
                "ip_address": common::IP_ADDRESS,
            }),
        ));
    }
    let target = &pages[0];
    let peer = &pages[1];
    let local = &pages[2];

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(local.page_id)),
        ..Default::default()
    });
    let local_edit = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": local.page_id,
            "last_revision_id": local.revision_id,
            "revision_comments": "second native revision",
            "user_id": ADMIN_USER_ID,
            "title": "Local revision lifecycle control edited",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("native control edit should create a revision");
    assert!(local_edit.revision_id > local.revision_id);

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
                'revision-lifecycle-fixture',
                'sandbox-for-codex',
                decode(repeat('12', 32), 'hex'),
                2,
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
            "UPDATE page SET from_wikidot = true WHERE page_id IN ({}, {})",
            target.page_id, peer.page_id,
        ),
    )
    .await;
    execute_sql(
        &runner,
        &format!(
            "UPDATE page_revision SET from_wikidot = true WHERE revision_id IN ({}, {})",
            target.revision_id, peer.revision_id,
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
            ) VALUES
            (
                {},
                'revision-lifecycle-fixture',
                'sandbox-for-codex',
                '77777777-7777-4777-8777-777777777777',
                '{TARGET_SLUG}',
                TIMESTAMPTZ '2026-08-13T00:00:00Z',
                TIMESTAMPTZ '2026-08-13T00:00:00Z',
                53,
                0,
                NULL,
                NULL,
                'Imported revision lifecycle target',
                NULL,
                0,
                NULL,
                NULL,
                decode(repeat('23', 32), 'hex'),
                decode(repeat('34', 32), 'hex'),
                '{{"fullname":"{TARGET_SLUG}","page_id":200,"revision_count":53}}'::jsonb,
                {IMPORT_RUN_ID}
            ),
            (
                {},
                'revision-lifecycle-fixture',
                'sandbox-for-codex',
                '88888888-8888-4888-8888-888888888888',
                '{PEER_SLUG}',
                TIMESTAMPTZ '2026-08-13T00:00:01Z',
                TIMESTAMPTZ '2026-08-13T00:00:01Z',
                53,
                0,
                NULL,
                NULL,
                'Imported revision lifecycle peer',
                NULL,
                0,
                NULL,
                NULL,
                decode(repeat('45', 32), 'hex'),
                decode(repeat('56', 32), 'hex'),
                '{{"fullname":"{PEER_SLUG}","page_id":100,"revision_count":53}}'::jsonb,
                {IMPORT_RUN_ID}
            )
            "#,
            target.page_id, peer.page_id,
        ),
    )
    .await;

    async fn render_rows(runner: &TestRunner, site_id: i64, category: &str) -> String {
        run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": "Imported revision lifecycle preview",
                "wikitext": format!(
                    "[[module ListPages category=\"{category}\" order=\"revisions desc\" separate=\"no\"]]\n%%name%%|%%revisions%%\n[[/module]]"
                ),
                "syntax_only": false,
            }),
        )
        .body
    }

    let initial = render_rows(&runner, site_id, CATEGORY).await;
    for expected in ["imported-target|53", "imported-peer|53", "local-page|2"] {
        assert!(
            initial.contains(expected),
            "baseline revision row {expected:?} should render:\n{initial}",
        );
    }

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(target.page_id)),
        ..Default::default()
    });
    let edited = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": target.page_id,
            "last_revision_id": target.revision_id,
            "revision_comments": "local edit after import",
            "user_id": ADMIN_USER_ID,
            "title": "Imported revision lifecycle target edited",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("imported target edit should create a revision");
    let after_edit = render_rows(&runner, site_id, CATEGORY).await;
    let edited_row = after_edit
        .find("imported-target|54")
        .unwrap_or_else(|| panic!("edited imported count should render:\n{after_edit}"));
    let peer_row = after_edit.find("imported-peer|53").unwrap_or_else(|| {
        panic!("untouched imported count should render:\n{after_edit}")
    });
    assert!(
        edited_row < peer_row,
        "the effective count must control ordering and substitution together:\n{after_edit}",
    );
    assert!(after_edit.contains("local-page|2"), "{after_edit}");

    let moved = run_endpoint!(
        runner,
        page_move,
        json!({
            "site_id": site_id,
            "page": target.page_id,
            "new_slug": MOVED_SLUG,
            "last_revision_id": edited.revision_id,
            "revision_comments": "move imported target after import",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let after_move = render_rows(&runner, site_id, CATEGORY).await;
    assert!(
        after_move.contains("imported-target-moved|55"),
        "move should add one local revision to the imported baseline:\n{after_move}",
    );
    assert!(after_move.contains("local-page|2"), "{after_move}");

    run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": target.page_id,
            "last_revision_id": moved.revision_id,
            "revision_comments": "delete imported target after import",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let while_deleted = render_rows(&runner, site_id, CATEGORY).await;
    assert!(
        !while_deleted.contains("imported-target"),
        "deleted pages must remain absent from ListPages:\n{while_deleted}",
    );
    assert!(
        while_deleted.contains("imported-peer|53"),
        "{while_deleted}"
    );
    assert!(while_deleted.contains("local-page|2"), "{while_deleted}");

    run_endpoint!(
        runner,
        page_restore,
        json!({
            "site_id": site_id,
            "page_id": target.page_id,
            "revision_comments": "restore imported target after import",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let after_restore = render_rows(&runner, site_id, CATEGORY).await;
    assert!(
        after_restore.contains("imported-target-moved|57"),
        "edit, move, delete, and restore should add four local revisions:\n{after_restore}",
    );
    assert!(
        after_restore.contains("imported-peer|53"),
        "{after_restore}"
    );
    assert!(after_restore.contains("local-page|2"), "{after_restore}");

    execute_sql(
        &runner,
        "ALTER TABLE wikidot_page_snapshot DROP CONSTRAINT \
         wikidot_page_snapshot_source_revision_count_check",
    )
    .await;
    execute_sql(
        &runner,
        &format!(
            "UPDATE wikidot_page_snapshot SET source_revision_count = -1 WHERE page_id = {}",
            target.page_id,
        ),
    )
    .await;
    let unavailable = render_rows(&runner, site_id, CATEGORY).await;
    assert!(
        unavailable.contains("%%revisions%%")
            || unavailable.contains("[[module ListPages")
            || unavailable.contains("module ListPages"),
        "a negative imported baseline must preserve the original ListPages module:\n{unavailable}",
    );
    assert!(
        !unavailable.contains("imported-target-moved|-1"),
        "an unavailable imported baseline must not render as a revision count:\n{unavailable}",
    );
    for direction in ["asc", "desc"] {
        let ordering_only = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": "Unavailable imported revision ordering preview",
                "wikitext": format!(
                    "[[module ListPages category=\"{CATEGORY}\" order=\"revisions {direction}\" separate=\"no\"]]\nORDERING_ONLY=%%name%%\n[[/module]]"
                ),
                "syntax_only": false,
            }),
        )
        .body;
        assert!(
            ordering_only.contains("ORDERING_ONLY=%%name%%")
                || ordering_only.contains("[[module ListPages")
                || ordering_only.contains("module ListPages"),
            "{direction} revision ordering must preserve the module when a selected count is unavailable:\n{ordering_only}",
        );
        assert!(
            !ordering_only.contains("ORDERING_ONLY=imported-target-moved"),
            "{direction} revision ordering must not continue with a NULL count:\n{ordering_only}",
        );
    }

    runner.teardown().await;
}

#[tokio::test]
async fn imported_revision_count_current_page_only_adds_local_revisions() {
    const SLUG: &str = "listpages-imported-current-revisions:target";
    const IMPORT_RUN_ID: i64 = 7_700_005;

    let mut runner = TestRunner::setup().await;
    ensure_wikidot_import_snapshot_tables(&runner).await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(SLUG.into())),
        ..Default::default()
    });
    let imported = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "Imported current-page revision baseline",
            "title": "Imported current-page revision target",
            "alt_title": null,
            "slug": SLUG,
            "layout": "wikidot",
            "revision_comments": "create imported current-page fixture",
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
                'current-revision-count-fixture',
                'sandbox-for-codex',
                decode(repeat('67', 32), 'hex'),
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
            "UPDATE page SET from_wikidot = true WHERE page_id = {}",
            imported.page_id,
        ),
    )
    .await;
    execute_sql(
        &runner,
        &format!(
            "UPDATE page_revision SET from_wikidot = true WHERE revision_id = {}",
            imported.revision_id,
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
                'current-revision-count-fixture',
                'sandbox-for-codex',
                '99999999-9999-4999-8999-999999999999',
                '{SLUG}',
                TIMESTAMPTZ '2026-08-13T00:00:00Z',
                TIMESTAMPTZ '2026-08-13T00:00:00Z',
                53,
                0,
                NULL,
                NULL,
                'Imported current-page revision target',
                NULL,
                0,
                NULL,
                NULL,
                decode(repeat('78', 32), 'hex'),
                decode(repeat('89', 32), 'hex'),
                '{{"fullname":"{SLUG}","revision_count":53}}'::jsonb,
                {IMPORT_RUN_ID}
            )
            "#,
            imported.page_id,
        ),
    )
    .await;

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(imported.page_id)),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": imported.page_id,
            "last_revision_id": imported.revision_id,
            "revision_comments": "add current-page-only ListPages after import",
            "user_id": ADMIN_USER_ID,
            "wikitext": "[[module ListPages name=\"=\" separate=\"no\"]]\nCURRENT_REVISIONS=%%revisions%%\n[[/module]]",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("local edit should create the current-page-only module revision");

    let saved = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": imported.page_id,
        }),
    )
    .expect("saved imported current-page fixture should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": saved.page_category_id,
            "page_id": imported.page_id,
        }),
    );
    let html = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": imported.page_id,
            "details": {"compiled": true},
        }),
    )
    .expect("rerendered imported current-page fixture should exist")
    .compiled_body_html
    .expect("rerendered imported current-page fixture should have compiled HTML");
    assert!(
        html.contains("CURRENT_REVISIONS=54"),
        "current-page-only ListPages must add the local edit to the imported baseline:\n{html}",
    );
    assert!(!html.contains("%%revisions%%"), "{html}");

    runner.teardown().await;
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
        "SOLO %%name%%\n",
        "[[/module]]\n",
        "[[module ListPages category=\"_default\" tags=\"-missing-live-probe-tag -\" ",
        "name=\"no-tags-selector-*\" separate=\"no\"]]\n",
        "COMPOUND %%name%%\n",
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
        html.contains(&format!("SOLO {UNTAGGED_SLUG}")),
        "the untagged page should match tags=\"-\":\n{html}",
    );
    assert!(
        !html.contains(&format!("SOLO {TAGGED_SLUG}")),
        "a tagged page must not be returned by tags=\"-\":\n{html}",
    );
    assert!(
        html.contains(&format!("COMPOUND {UNTAGGED_SLUG}"))
            && html.contains(&format!("COMPOUND {TAGGED_SLUG}")),
        "a standalone minus must be ignored when another tag token is present:\n{html}",
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
            .execute_raw(Statement::from_sql_and_values(
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
async fn preview_link_to_dot_drops_the_selector_without_a_current_page_identity() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let html = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved ListPages link_to current-page preview",
        concat!(
            "[[module ListPages category=\"*\" name=\"component:image-block\" ",
            "link_to=\".\" separate=\"no\"]]\n",
            "ROW|%%fullname%%\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("an unresolved preview link_to current-page selector should render")
    .html_output
    .body;

    assert!(
        html.contains("ROW|component:image-block"),
        "live Wikidot drops link_to=\".\" when PagePreview has no persisted \
         current page identity instead of querying a made-up preview slug:\n{html}",
    );
}

#[tokio::test]
async fn preview_link_to_uses_direct_links_and_reports_a_missing_target() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let html = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages direct-link boundary preview",
        concat!(
            "[[module ListPages category=\"*\" name=\"scp-002\" link_to=\"component:license-box\" separate=\"no\" wrapper=\"no\"]]\n",
            "DIRECT|%%fullname%%|\n",
            "[[/module]]\n",
            "[[module ListPages category=\"*\" name=\"scp-002\" link_to=\"component:image-block\" separate=\"no\" wrapper=\"no\"]]\n",
            "INCLUDE|%%fullname%%|\n",
            "[[/module]]\n",
            "[[module ListPages category=\"*\" name=\"scp-002\" link_to=\"scp-002\" separate=\"no\" wrapper=\"no\"]]\n",
            "SELF|%%fullname%%|\n",
            "[[/module]]\n",
            "[[module ListPages category=\"*\" name=\"scp-002\" link_to=\"definitely-missing-link-target\" separate=\"no\" wrapper=\"no\"]]\n",
            "MISSING|%%fullname%%|\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("the direct-link compatibility matrix should render")
    .html_output
    .body;

    assert!(
        html.contains("DIRECT|scp-002|"),
        "a direct page link must satisfy link_to:\n{html}",
    );
    assert!(
        !html.contains("INCLUDE|scp-002|"),
        "an include-only dependency must not satisfy link_to:\n{html}",
    );
    assert!(
        !html.contains("SELF|scp-002|"),
        "a current-page self-reference must not satisfy link_to:\n{html}",
    );
    assert!(
        html.contains(
            r#"<div class="error-block">Linked page definitely-missing-link-target does not exist</div>"#,
        ),
        "a missing link target must produce Wikidot's exact error:\n{html}",
    );
}

#[tokio::test]
async fn preview_unknown_bare_and_valueless_head_tokens_remain_inert() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for token in [
        "unknown",
        "unknown=",
        "site",
        "site=",
        "camelCase",
        "UPPER=",
        "a_b",
        "a-b=",
        "unknown= name=\"component:image-block\"",
        "name=\"component:image-block\" unknown=",
    ] {
        let source = format!(
            concat!(
                "[[module ListPages category=\"*\" name=\"component:image-block\" ",
                "separate=\"no\" wrapper=\"no\" {}]]\n",
                "BEGIN|%%fullname%%|END\n",
                "[[/module]]",
            ),
            token
        );
        let html = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "ListPages inert unknown head token",
            source,
        )
        .await
        .unwrap_or_else(|error| panic!("unknown token {token:?} should render: {error}"))
        .html_output
        .body;

        assert!(
            html.contains("BEGIN|component:image-block|END")
                && !html.contains("TODO: module ListPages")
                && !html.contains("[[module ListPages"),
            "unknown token {token:?} must remain inert without rolling back the module:\n{html}",
        );
    }
}

#[tokio::test]
async fn preview_parent_modes_do_not_invent_a_current_page_relation() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let html = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages parent modes without a current page",
        concat!(
            "[[module ListPages category=\"*\" name=\"component:image-block\" parent=\"-=\" separate=\"no\" wrapper=\"no\"]]\n",
            "DIFFERENT|%%fullname%%|\n",
            "[[/module]]\n",
            "[[module ListPages category=\"*\" name=\"component:image-block\" parent=\"=\" separate=\"no\" wrapper=\"no\"]]\n",
            "SAME|%%fullname%%|\n",
            "[[/module]]\n",
            "[[module ListPages category=\"*\" name=\"component:image-block\" parent=\"@URL|-=\" separate=\"no\" wrapper=\"no\"]]\n",
            "FALLBACK|%%fullname%%|\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("parent modes without a current page should render")
    .html_output
    .body;

    assert!(
        html.contains("DIFFERENT|component:image-block|")
            && html.contains("FALLBACK|component:image-block|"),
        "parent=\"-=\" must be a no-op without a current page, including URL fallback:\n{html}",
    );
    assert!(
        !html.contains("SAME|component:image-block|"),
        "parent=\"=\" must not invent a current-parent relation:\n{html}",
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
async fn listpages_total_counts_matches_beyond_the_query_window() {
    const TARGET_SLUG: &str = "total-over-query-window-base";
    const EXTRA_MATCHES: i32 = 260;

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
            "wikitext": "Total-over-window fixture",
            "title": "Total over query window base",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "total-over-window ListPages fixture",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let transaction = runner.context().transaction();
    let inserted = transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "INSERT INTO page \
             (created_at, from_wikidot, site_id, page_category_id, slug, layout) \
             SELECT NOW() + series * INTERVAL '1 second', source.from_wikidot, \
                    source.site_id, source.page_category_id, \
                    'total-over-query-window-' || series, source.layout \
             FROM page AS source \
             CROSS JOIN generate_series(1, $3) AS series \
             WHERE source.site_id = $1 AND source.slug = $2 \
                   AND source.deleted_at IS NULL",
            [
                Value::from(site_id),
                Value::from(TARGET_SLUG.to_owned()),
                Value::from(EXTRA_MATCHES),
            ],
        ))
        .await
        .expect("total-over-window fixture rows should be inserted");
    assert_eq!(inserted.rows_affected(), EXTRA_MATCHES as u64);

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages total over query window",
        concat!(
            "[[module ListPages category=\"_default\" ",
            "name=\"total-over-query-window-*\" perPage=\"1\" separate=\"no\"]]\n",
            "TOTAL %%total%%\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("total-over-window ListPages preview should render");
    let body = &preview.html_output.body;

    assert!(
        body.contains("TOTAL 261"),
        "the one rendered row should report all matches beyond the 250-row query window:\n{}",
        body,
    );
    assert!(
        !body.contains("[[module ListPages"),
        "an exact total should execute instead of preserving the source module:\n{}",
        body,
    );
}

#[tokio::test]
async fn listpages_uses_limit_as_total_and_defaults_pagination_to_twenty() {
    const PREFIX: &str = "listpages-default-pager-fixture";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for index in 0..21 {
        let slug = format!("{PREFIX}-{index:02}");
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
                "wikitext": "Default pager fixture",
                "title": slug,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "default ListPages pager fixture",
                "user_id": ADMIN_USER_ID,
                "bypass_filter": true,
                "ip_address": common::IP_ADDRESS,
            }),
        );
    }

    for (attributes, expected_rows, expected_pager) in [
        ("", 20, Some("page 1 of 2")),
        (r#"limit="5""#, 5, None),
        (r#"limit="21" perPage="7""#, 7, Some("page 1 of 3")),
        (r#"limit="21" perPage="999999999""#, 21, None),
        (r#"limit="" perPage="""#, 20, Some("page 1 of 2")),
    ] {
        let preview = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "Default pager preview",
            format!(
                "[[module ListPages category=\"_default\" name=\"{PREFIX}-*\" order=\"name\" separate=\"no\" {attributes}]]\nROW %%index%% %%name%%\n[[/module]]",
            ),
        )
        .await
        .expect("ListPages pagination preview should render")
        .html_output
        .body;
        assert_eq!(
            preview.matches("ROW ").count(),
            expected_rows,
            "unexpected rendered page size for {attributes:?}:\n{preview}",
        );
        if let Some(expected_pager) = expected_pager {
            assert!(
                preview.contains(expected_pager),
                "the live-compatible pager should render for {attributes:?}:\n{preview}",
            );
            assert!(
                preview
                    .contains(r#"<div class="pager"><span class="pager-no">page 1 of "#,)
                    && preview.contains(r#"href="/ajax-module-connector.php/p/2""#,),
                "preview pagers must have direct span children and Ajax-module hrefs for {attributes:?}:\n{preview}",
            );
            assert!(
                !preview.contains(r#"<div class="pager"><p>"#),
                "preview pagers must not gain an FTML paragraph wrapper:\n{preview}",
            );
        } else {
            assert!(
                !preview.contains("class=\"pager\""),
                "no pager should render for {attributes:?}:\n{preview}",
            );
        }
    }
}

/// Anonymous Wikidot PagePreview evidence, 2026-07-30:
/// `pager-dom-b16b76666.json`, case `scout-pager-dom-wrapper-no`.
#[tokio::test]
async fn listpages_unwrapped_separate_row_is_adjacent_to_its_pager() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let body = RenderService::render_wikidot_page_preview(
        runner.context(),
        site.site.site_id,
        "scout-pager-dom-wrapper-no",
        concat!(
            "[[module ListPages category=\"*\" name=\"component:image-block*\" ",
            "order=\"name\" limit=\"2\" perPage=\"1\" wrapper=\"no\"]]\n",
            "%%fullname%%|\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("unwrapped ListPages pager preview should render")
    .html_output
    .body;

    assert_eq!(
        body,
        concat!(
            r#"<div class="list-pages-item"><p>component:image-block|</p></div>"#,
            r#"<div class="pager"><span class="pager-no">page 1 of 2</span>"#,
            r#"<span class="current">1</span>"#,
            r#"<span class="target"><a href="/ajax-module-connector.php/p/2">2</a></span>"#,
            r#"<span class="target"><a href="/ajax-module-connector.php/p/2">next »</a></span>"#,
            "</div>\n",
        ),
    );
}

/// Anonymous Wikidot PagePreview evidence, 2026-07-29:
/// `cn:wikidot-module-tech:L363:B11414`.
///
/// A sectioned, unwrapped module keeps its generated list in the surrounding
/// block stream.  Wrapping the `<ul>` in an FTML paragraph changes the live
/// DOM and makes the sectioned template unusable in ordinary page markup.
#[tokio::test]
async fn listpages_unwrapped_sectioned_block_template_stays_outside_paragraph() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let body = RenderService::render_wikidot_page_preview(
        runner.context(),
        site.site.site_id,
        "wikidot-module-tech-sectioned-block",
        concat!(
            "[[module ListPages separate=\"no\" wrapper=\"no\" limit=\"5\"]]\n",
            "[[head]]\n",
            "[[ul id=\"u-myList\"]]\n",
            "[[/head]]\n",
            "[[body]]\n",
            "[[li class=\"list-item\"]]%%title_linked%% by (%%created_by%%)[[/li]]\n",
            "[[/body]]\n",
            "[[foot]]\n",
            "[[/ul]]\n",
            "[[/foot]]\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("sectioned unwrapped ListPages preview should render")
    .html_output
    .body;

    assert!(
        body.contains(r#"<ul id="u-myList">"#),
        "the sectioned template should emit its list: {body}"
    );
    assert!(
        !body.contains("<p><ul"),
        "a block section must not acquire an FTML paragraph wrapper: {body}"
    );

    runner.teardown().await;
}

/// Anonymous Wikidot PagePreview evidence, 2026-07-30:
/// `section-grammar-live.jsonl`, case `scout-sections-perm-hbf`.
#[tokio::test]
async fn listpages_combined_inline_sections_share_one_paragraph() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let body = RenderService::render_wikidot_page_preview(
        runner.context(),
        site.site.site_id,
        "scout-sections-perm-hbf",
        concat!(
            "[[module ListPages fullname=\"scp-002\" separate=\"no\" wrapper=\"no\"]]\n",
            "[[head]]H[[/head]]\n",
            "[[body]]B=%%title%%[[/body]]\n",
            "[[foot]]F[[/foot]]\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("combined inline ListPages sections should render")
    .html_output
    .body;

    assert_eq!(body, "<p>H<br>\nB=SCP-002<br>\nF</p>");

    runner.teardown().await;
}

/// Anonymous Wikidot PagePreview evidence, 2026-08-09:
/// `issue-1010-generated-heading-foot-boundary-scp-wiki-live.jsonl`, case
/// `scout-generated-heading-foot-section-level-two`.
#[tokio::test]
async fn listpages_own_line_head_starts_a_separate_paragraph() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let body = RenderService::render_wikidot_page_preview(
        runner.context(),
        site.site.site_id,
        "scout-generated-heading-foot-section",
        concat!(
            "[[module ListPages fullname=\"scp-002\" separate=\"no\" wrapper=\"no\"]]\n",
            "[[head]]\n",
            "HEAD\n",
            "[[/head]]\n",
            "[[body]]\n",
            "ROW=%%fullname%%\n",
            "[[/body]]\n",
            "[[foot]]\n",
            "+ FOOT\n",
            "[[/foot]]\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("own-line ListPages sections should render")
    .html_output
    .body;

    assert_eq!(
        body,
        "<p>HEAD</p><p>ROW=scp-002</p><h1><span>FOOT</span></h1>",
    );

    runner.teardown().await;
}

/// Anonymous Wikidot PagePreview evidence, 2026-07-29:
/// `cn:wanderers:enter-the-library:L388:B15100`.
///
/// A combined (`separate="no"`) wrapper keeps an inline generated body in
/// the wrapper's own flow. FTML must not manufacture a paragraph around the
/// inline link, even though the ListPages box itself remains a block.
#[tokio::test]
async fn listpages_combined_wrapper_keeps_inline_rows_outside_paragraph() {
    const TARGET_SLUG: &str = "listpages-inline-row-target-20260802";

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
            "wikitext": "inline ListPages row target",
            "title": "Inline ListPages row target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "combined inline ListPages row fixture",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let body = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "combined inline ListPages row source",
        concat!(
            "[[module ListPages name=\"listpages-inline-row-target-20260802\" separate=\"no\"]]",
            "[[a_ href=\"/%%fullname%%\" class=\"book\"]]",
            "[[span]]%%name%%[[/span]]",
            "[[/a]]",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("combined inline ListPages preview should render")
    .html_output
    .body;

    assert!(
        body.contains(
            r#"<div class="list-pages-box"><a class="book" href="/listpages-inline-row-target-20260802">"#,
        ),
        "the combined row should stay directly inside the ListPages box: {body}",
    );
    assert!(
        !body.contains(r#"<div class="list-pages-box"><p><a"#),
        "the combined inline row must not acquire an FTML paragraph: {body}",
    );

    runner.teardown().await;
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

#[tokio::test]
async fn unsaved_preview_runs_site_queries_without_inventing_a_current_page() {
    const TARGET_SLUG: &str = "listpages-preview-context-target";

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
            "wikitext": "[[>]]",
            "title": "ListPages Preview Context Target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "ListPages preview context fixture",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let static_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\"]]\nROW %%fullname%%\n[[/module]]",
        ),
    )
    .await
    .expect("site-scoped ListPages should render in an unsaved preview")
    .html_output
    .body;
    assert!(
        static_preview.contains(&format!("ROW {TARGET_SLUG}")),
        "an unsaved preview should still query its site:\n{static_preview}",
    );

    let empty_body_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        "[[module ListPages]]\n\n[[/module]]".to_owned(),
    )
    .await
    .expect("an empty ListPages body should use the default row template")
    .html_output
    .body;
    assert!(
        empty_body_preview.contains("ListPages Preview Context Target")
            && empty_body_preview.contains("[[/module]]"),
        "Wikidot executes an empty ListPages body with its default row template \
         while leaving the raw closer literal:\n{empty_body_preview}",
    );

    let bare_own_line_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved bare ListPages preview",
        "[[module ListPages]]".to_owned(),
    )
    .await
    .expect("a bare own-line ListPages invocation should execute")
    .html_output
    .body;
    let target_row_start = bare_own_line_preview
        .find(&format!(
            r#"<div class="list-pages-item"><h1><span><a href="/{TARGET_SLUG}">"#,
        ))
        .expect("the target row should be present");
    let target_row_tail = &bare_own_line_preview[target_row_start..];
    let target_row_end = target_row_tail[1..]
        .find(r#"<div class="list-pages-item">"#)
        .map_or(target_row_tail.len(), |offset| offset + 1);
    let target_row = &target_row_tail[..target_row_end];
    assert!(
        bare_own_line_preview.contains(r#"<div class="list-pages-box">"#)
            && bare_own_line_preview.contains("ListPages Preview Context Target")
            && bare_own_line_preview.contains("by Administrator")
            && target_row.contains(r#"<div style="text-align: right;">"#)
            && target_row.ends_with("</div></div>")
            && !bare_own_line_preview.contains("[[module ListPages]]"),
        "the exact empty-head invocation must use the default query and row template:\n{bare_own_line_preview}",
    );

    let html_encoded_script_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        concat!(
            "[[module ListPages category=&quot;fragment&quot; ",
            "limit=&quot;1&quot; order=&quot;name&quot;]]",
            "</p><p>ENCODED-BODY=%%title%%</p><p>[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("an HTML-encoded script fragment should retain the live module boundary")
    .html_output
    .body;
    assert!(
        html_encoded_script_preview.contains("ListPages Preview Context Target")
            && html_encoded_script_preview.contains("by Administrator")
            && !html_encoded_script_preview.contains("ENCODED-BODY=")
            && !html_encoded_script_preview.contains("[[/module]]"),
        "Wikidot drops the HTML-encoded script tail and executes the default \
         ListPages row template:\n{html_encoded_script_preview}",
    );

    let inline_raw_documentation_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        format!(
            concat!(
                "[[module ListPages]]@@. @@[[html]]@@ DOC PROSE\n",
                "more @@[[html]]@@ prose\n",
                "@@[[/module]]\n",
                "[[module ListPages name=\"{}\"]]",
                "LATER=%%title%%[[/module]]",
            ),
            TARGET_SLUG,
        ),
    )
    .await
    .expect("an inline-raw documentation tail should not swallow later modules")
    .html_output
    .body;
    assert!(
        inline_raw_documentation_preview.contains("[[module ListPages]]")
            && inline_raw_documentation_preview.contains("DOC PROSE")
            && inline_raw_documentation_preview
                .contains("LATER=ListPages Preview Context Target")
            && !inline_raw_documentation_preview.contains("TODO: module ListPages"),
        "Wikidot keeps the fake opener literal and executes the later module:\n\
         {inline_raw_documentation_preview}",
    );

    let sticky_note_documentation_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        concat!(
            "[[module ListPages]]@@\n",
            "@@[[div class=\"seed\"]]DOC@@[[/div]]@@\n",
            "@@[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("the sticky-note documentation opener should remain literal")
    .html_output
    .body;
    assert!(
        sticky_note_documentation_preview
            .contains("[[module ListPages]] [[div class=&quot;seed&quot;]]"),
        "Wikidot retains one collapsed raw-marker space after the documented \
         opener:\n{sticky_note_documentation_preview}",
    );

    let raw_footnote_boundary_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        concat!(
            "[[module ListPages range=\".\" wrapper=\"no\" separate=\"no\"]]",
            "@@[[footnote]] **range=\".\"** NOTE [[/footnote]]\n",
            "@@----@@\n",
            "@@%%content{2}%%@@\n",
            "@@----@@\n",
            "@@[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("the raw leading footnote boundary should render")
    .html_output
    .body;
    assert!(
        raw_footnote_boundary_preview
            .starts_with(r#"<p><span style="white-space: pre-wrap;">----</span>"#)
            && raw_footnote_boundary_preview.contains("%%content{2}%%")
            && raw_footnote_boundary_preview.contains("@@[[/module]]")
            && !raw_footnote_boundary_preview.contains("footnoteref")
            && !raw_footnote_boundary_preview.contains("footnotes-footer")
            && !raw_footnote_boundary_preview.contains("NOTE"),
        "Wikidot consumes the leading raw footnote with the ListPages opener but \
         leaves the later raw template literal:\n{raw_footnote_boundary_preview}",
    );

    let raw_footnote_boundary_with_limit_preview =
        RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "Unsaved preview",
            concat!(
                "[[module ListPages range=\".\" wrapper=\"no\" separate=\"no\" ",
                "limit=\"1\"]]",
                "@@[[footnote]] **range=\".\"** NOTE [[/footnote]]\n",
                "@@----@@\n",
                "@@%%content{2}%%@@\n",
                "@@----@@\n",
                "@@[[/module]]",
            )
            .to_owned(),
        )
        .await
        .expect("a semantically equivalent raw footnote boundary should render")
        .html_output
        .body;
    assert!(
        raw_footnote_boundary_with_limit_preview
            .starts_with(r#"<p><span style="white-space: pre-wrap;">----</span>"#)
            && raw_footnote_boundary_with_limit_preview.contains("%%content{2}%%")
            && raw_footnote_boundary_with_limit_preview.contains("@@[[/module]]")
            && !raw_footnote_boundary_with_limit_preview.contains("footnoteref")
            && !raw_footnote_boundary_with_limit_preview.contains("footnotes-footer")
            && !raw_footnote_boundary_with_limit_preview.contains("NOTE"),
        "the raw footnote boundary should follow parsed current-page semantics, not an exact head:\n{raw_footnote_boundary_with_limit_preview}",
    );

    let raw_footnote_non_current_range_preview =
        RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "Unsaved preview",
            concat!(
                "[[module ListPages range=\"others\" wrapper=\"no\" separate=\"no\" ",
                "limit=\"1\"]]",
                "@@[[footnote]] **range=\"others\"** NOTE [[/footnote]]\n",
                "@@----@@\n",
                "@@%%content{2}%%@@\n",
                "@@----@@\n",
                "@@[[/module]]",
            )
            .to_owned(),
        )
        .await
        .expect("a non-current raw footnote range should keep ordinary footnotes")
        .html_output
        .body;
    assert!(
        raw_footnote_non_current_range_preview.contains("footnoteref")
            && raw_footnote_non_current_range_preview.contains("footnotes-footer")
            && raw_footnote_non_current_range_preview.contains("NOTE"),
        "only the current-page raw footnote boundary should be consumed:\n{raw_footnote_non_current_range_preview}",
    );

    let raw_collapsible_boundary_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        concat!(
            "[[module ListPages fullname=\"@@##red|definitely-missing-page##@@\" ",
            "separate=\"yes\" limit=\"250\"]]@@\n",
            "CONSUMED-DOCUMENTATION\n",
            "> @@[[module ListPages fullname=\"@@##red|example##@@\"]]@@\n",
            "[[collapsible show=\"+ Syntax\" hide=\"- Syntax\"]]\n",
            "VISIBLE-AFTER-BOUNDARY\n",
            "[[/collapsible]]",
        )
        .to_owned(),
    )
    .await
    .expect("the raw collapsible ListPages boundary should render")
    .html_output
    .body;
    assert!(
        raw_collapsible_boundary_preview
            .contains(r#"<div class="list-pages-box"></div>"#)
            && raw_collapsible_boundary_preview.contains("VISIBLE-AFTER-BOUNDARY")
            && raw_collapsible_boundary_preview.contains("[[/collapsible]]")
            && !raw_collapsible_boundary_preview.contains("CONSUMED-DOCUMENTATION")
            && !raw_collapsible_boundary_preview.contains("collapsible-block")
            && !raw_collapsible_boundary_preview.contains("TODO: module ListPages"),
        "Wikidot executes the unclosed ListPages opener, consumes through the \
         first evidenced collapsible opening, and resumes at its body:\n\
         {raw_collapsible_boundary_preview}",
    );

    let raw_footnote_head_collapsible_preview =
        RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "Unsaved preview",
            concat!(
                "[[module ListPages @@raw head [[/footnote]]",
                "CONSUMED-FOOTNOTE-PROSE-ONE\n\n",
                "CONSUMED-FOOTNOTE-PROSE-TWO\n",
                "[[collapsible show=\"CONSUMED-OPEN\" hide=\"CONSUMED-CLOSE\"]]\n",
                "VISIBLE-FOOTNOTE-BOUNDARY-BODY\n",
                "[[/collapsible]]",
            )
            .to_owned(),
        )
        .await
        .expect("the raw-footnote head collapsible boundary should render")
        .html_output
        .body;
    assert!(
        raw_footnote_head_collapsible_preview.contains("VISIBLE-FOOTNOTE-BOUNDARY-BODY")
            && raw_footnote_head_collapsible_preview.contains("[[/collapsible]]")
            && !raw_footnote_head_collapsible_preview.contains("CONSUMED-FOOTNOTE-PROSE")
            && !raw_footnote_head_collapsible_preview.contains("CONSUMED-OPEN")
            && !raw_footnote_head_collapsible_preview.contains("TODO: module ListPages"),
        "Wikidot consumes a raw-footnote ListPages opener through the first \
         collapsible opening and resumes at its body:\n\
         {raw_footnote_head_collapsible_preview}",
    );

    let raw_unclosed_preservation_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        format!(
            concat!(
                "[[module ListPages fullname=\"definitely-missing-page\" ",
                "separate=\"yes\" limit=\"250\"]]@@\n",
                "PRESERVED-DOCUMENTATION\n",
                "@@\n",
                "[[module ListPages name=\"{TARGET_SLUG}\"]]",
                "LATER=%%fullname%%[[/module]]",
            ),
            TARGET_SLUG = TARGET_SLUG,
        ),
    )
    .await
    .expect("the unsupported unclosed raw boundary should fail closed")
    .html_output
    .body;
    assert!(
        raw_unclosed_preservation_preview.contains(
            "[[module654 ListPages fullname=&quot;definitely-missing-page&quot;",
        ) && raw_unclosed_preservation_preview.contains("PRESERVED-DOCUMENTATION")
            && raw_unclosed_preservation_preview
                .contains(&format!("LATER={TARGET_SLUG}"))
            && !raw_unclosed_preservation_preview.contains("TODO: module ListPages"),
        "Wikidot preserves the unsupported opener as legacy module654 text \
         without hiding a later valid module:\n{raw_unclosed_preservation_preview}",
    );

    let trailing_at_marker_row_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        concat!(
            "[[module ListPages created_by=\"morhadow\" tags=\"es\" ",
            "order=\"rating\" limit=\"5\" separate=\"no\"@@]]@@\n",
            "@@*@@ %%title_linked%% (+%%rating%%)\n",
            "@@[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("the corpus trailing-at-marker row tail should execute")
    .html_output
    .body;
    assert_eq!(
        trailing_at_marker_row_preview.trim(),
        r#"<div class="list-pages-box"></div>"#,
        "Wikidot executes this exact raw-row tail as an empty query instead of \
         preserving its opener as module654 text:\n{trailing_at_marker_row_preview}",
    );

    let unknown_variable_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\"]]\nKNOWN=%%fullname%%|UNKNOWN=%%unsupported%%\n[[/module]]",
        ),
    )
    .await
    .expect("an unknown row variable should not reject a valid ListPages body")
    .html_output
    .body;
    assert!(
        unknown_variable_preview
            .contains(&format!("KNOWN={TARGET_SLUG}|UNKNOWN=%%unsupported%%",)),
        "unknown variables should remain literal while known variables execute:\n{unknown_variable_preview}",
    );

    let unknown_tracking_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        format!(
            concat!(
                "[[module ListPages name=\"{TARGET_SLUG}\" separate=\"no\" wrapper=\"no\"]]\n",
                "[[image https://tracker.invalid/%%fullname%%/%%unsupported%%]]\n",
                "[[/module]]",
            ),
            TARGET_SLUG = TARGET_SLUG,
        ),
    )
    .await
    .expect("unknown variables must make tracking-only templates fail closed")
    .html_output
    .body;
    assert_eq!(
        unknown_tracking_preview.trim(),
        r#"<div class="list-pages-box"></div>"#,
        "unknown tracking templates must not expose substituted page metadata or active image markup:\n{unknown_tracking_preview}",
    );

    for (label, body) in [
        (
            "hidden image",
            "[[image https://tracker.invalid/%%fullname%%/%%unsupported%%]]",
        ),
        (
            "hidden iframe",
            r#"<iframe src="https://tracker.invalid/%%fullname%%/%%unsupported%%" style="display: none"></iframe>"#,
        ),
        (
            "hidden ListUsers module",
            "[[module ListUsers users=\"%%fullname%%/%%unsupported%%\"]]\n[[/module]]",
        ),
    ] {
        let preview = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "Unsaved preview",
            format!(
                concat!(
                    "[[module ListPages name=\"{TARGET_SLUG}\" separate=\"no\" ",
                    "wrapper=\"no\"]]\n{BODY}\n[[/module]]",
                ),
                TARGET_SLUG = TARGET_SLUG,
                BODY = body,
            ),
        )
        .await
        .expect("unknown tracking shapes must fail closed")
        .html_output
        .body;
        assert_eq!(
            preview.trim(),
            r#"<div class="list-pages-box"></div>"#,
            "{label} with an unknown variable must not become active ListPages output:\n{preview}",
        );
    }

    let sectioned_unknown_tracking_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        format!(
            concat!(
                "[[module ListPages name=\"{TARGET_SLUG}\" separate=\"no\" wrapper=\"no\"]]\n",
                "[[head]]VISIBLE-HEAD[[/head]]\n",
                "[[body]]\n",
                "[[image https://tracker.invalid/%%fullname%%/%%unsupported%%]]\n",
                "[[/body]]\n",
                "[[foot]]VISIBLE-FOOT[[/foot]]\n",
                "[[/module]]",
            ),
            TARGET_SLUG = TARGET_SLUG,
        ),
    )
    .await
    .expect("section wrappers must not hide an unknown tracking body")
    .html_output
    .body;
    assert_eq!(
        sectioned_unknown_tracking_preview.trim(),
        r#"<div class="list-pages-box"></div>"#,
        "the effective section body must use the existing tracking-only policy:\n{sectioned_unknown_tracking_preview}",
    );

    let visible_authored_image_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        format!(
            concat!(
                "[[module ListPages name=\"{TARGET_SLUG}\" separate=\"no\" wrapper=\"no\"]]\n",
                "VISIBLE [[image https://example.invalid/%%fullname%%/%%unsupported%%]]\n",
                "[[/module]]",
            ),
            TARGET_SLUG = TARGET_SLUG,
        ),
    )
    .await
    .expect("visible authored images must remain outside the tracking-only policy")
    .html_output
    .body;
    assert!(
        visible_authored_image_preview.contains("VISIBLE")
            && visible_authored_image_preview.contains(TARGET_SLUG)
            && visible_authored_image_preview.contains("%%unsupported%%")
            && !visible_authored_image_preview
                .contains(r#"<div class="list-pages-box">"#),
        "ordinary visible image markup must remain active while unknown tokens stay literal:\n{visible_authored_image_preview}",
    );

    let tabbed_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        format!(
            "[[module\tListPages\tname=\"{TARGET_SLUG}\"]]\nROW %%fullname%%\n[[/module]]",
        ),
    )
    .await
    .expect("Wikidot's horizontal module spacing should be accepted")
    .html_output
    .body;
    assert!(
        tabbed_preview.contains(&format!("ROW {TARGET_SLUG}")),
        "a tab-delimited ListPages opening should execute:\n{tabbed_preview}",
    );

    let generated_heading_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        format!(
            concat!(
                "[[toc]]\n",
                "+ AUTHORED-BEFORE\n\n",
                "[[module ListPages name=\"{TARGET_SLUG}\" separate=\"no\" wrapper=\"no\" ",
                "prependLine=\"+ GENERATED-PREPEND\" appendLine=\"+ GENERATED-APPEND\"]]\n",
                "[[head]]+ GENERATED-HEAD[[/head]]\n",
                "[[body]]+ GENERATED-ROW %%fullname%%[[/body]]\n",
                "[[foot]]+ GENERATED-FOOT[[/foot]]\n",
                "[[/module]]\n\n",
                "+ AUTHORED-AFTER",
            ),
            TARGET_SLUG = TARGET_SLUG,
        ),
    )
    .await
    .expect("ListPages generated headings should render in an unsaved preview")
    .html_output
    .body;
    assert!(
        generated_heading_preview
            .contains(r#"<h1 id="toc0"><span>AUTHORED-BEFORE</span></h1>"#)
            && generated_heading_preview
                .contains(r#"<h1 id="toc1"><span>AUTHORED-AFTER</span></h1>"#),
        "generated headings must not consume authored heading IDs:\n{generated_heading_preview}",
    );
    for heading in [
        "GENERATED-PREPEND",
        "GENERATED-HEAD",
        &format!("GENERATED-ROW {TARGET_SLUG}"),
        "GENERATED-FOOT",
        "GENERATED-APPEND",
    ] {
        assert!(
            generated_heading_preview
                .contains(&format!("<h1><span>{heading}</span></h1>")),
            "ListPages heading {heading:?} should render without a TOC ID:\n{generated_heading_preview}",
        );
    }
    assert_eq!(
        generated_heading_preview.matches("href=\"#toc").count(),
        2,
        "the outer TOC must contain only the two authored headings:\n{generated_heading_preview}",
    );

    for (label, template, expected, absent) in [
        (
            "out-of-order sections",
            "[[head]]ORDERED-HEAD[[/head]][[foot]]ORDERED-FOOT[[/foot]][[body]]ORDERED-BODY=%%fullname%%[[/body]]",
            &["ORDERED-BODY=listpages-preview-context-target"][..],
            &["ORDERED-HEAD", "ORDERED-FOOT"][..],
        ),
        (
            "mixed-case sections",
            "[[Head]]CASE-HEAD[[/hEAd]][[bODy]]CASE-BODY=%%fullname%%[[/Body]][[FOot]]CASE-FOOT[[/fooT]]",
            &[
                "CASE-HEAD",
                "CASE-BODY=listpages-preview-context-target",
                "CASE-FOOT",
            ][..],
            &[][..],
        ),
        (
            "literal-owned head",
            "[!-- [[head]]LITERAL-HEAD[[/head]] --][[body]]LITERAL-BODY=%%fullname%%[[/body]][[foot]]LITERAL-FOOT[[/foot]]",
            &[
                "LITERAL-BODY=listpages-preview-context-target",
                "LITERAL-FOOT",
            ][..],
            &["LITERAL-HEAD"][..],
        ),
        (
            "once-only variables",
            "[[head]]ONCE-HEAD=%%title%%[[/head]][[body]]ONCE-BODY=%%fullname%%[[/body]][[foot]]ONCE-FOOT=%%title%%[[/foot]]",
            &[
                "ONCE-HEAD=%%title%%",
                "ONCE-BODY=listpages-preview-context-target",
                "ONCE-FOOT=%%title%%",
            ][..],
            &[][..],
        ),
        (
            "empty body fallback",
            "[[head]]EMPTY-HEAD[[/head]][[body]][[/body]][[foot]]EMPTY-FOOT[[/foot]]",
            &[
                "EMPTY-HEAD",
                "ListPages Preview Context Target",
                "EMPTY-FOOT",
            ][..],
            &[][..],
        ),
    ] {
        let preview = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "Unsaved preview",
            format!(
                "[[module ListPages name=\"{TARGET_SLUG}\" separate=\"no\" wrapper=\"no\"]]{template}[[/module]]",
            ),
        )
        .await
        .unwrap_or_else(|error| panic!("{label} should render: {error:?}"))
        .html_output
        .body;
        for marker in expected {
            assert!(
                preview.contains(marker),
                "{label} should contain {marker:?}:\n{preview}",
            );
        }
        for marker in absent {
            assert!(
                !preview.contains(marker),
                "{label} should not contain {marker:?}:\n{preview}",
            );
        }
    }

    let unclosed_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        concat!(
            "[[module ListPages name=\"definitely-missing-listpages-unclosed\"]]\n",
            "%%title%%",
        )
        .to_owned(),
    )
    .await
    .expect("a complete unclosed ListPages opening should execute")
    .html_output
    .body;
    assert!(
        unclosed_preview
            .contains(r#"<div class="list-pages-box"></div><p>%%title%%</p>"#,),
        "Wikidot executes only the unclosed opening and leaves its following text outside the module:\n{unclosed_preview}",
    );
    assert!(
        !unclosed_preview.contains("[[module ListPages"),
        "the executed opening must not remain literal:\n{unclosed_preview}",
    );

    let unclosed_css_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        format!(
            concat!(
                "[[module CSS]]\n\n",
                "[[module ListPages name=\"{TARGET_SLUG}\" separate=\"no\"]]\n",
                "CSS-BOUNDARY %%fullname%%\n",
                "[[/module]]",
            ),
            TARGET_SLUG = TARGET_SLUG,
        ),
    )
    .await
    .expect("ListPages after an unclosed root CSS module should render")
    .html_output
    .body;
    assert!(
        unclosed_css_preview.contains(&format!("CSS-BOUNDARY {TARGET_SLUG}"))
            && !unclosed_css_preview.contains("[[module CSS]]")
            && !unclosed_css_preview.contains("[[module ListPages"),
        "early ListPages expansion must preserve FTML's unclosed-CSS yield boundary:\n{unclosed_css_preview}",
    );

    for source in [
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" tags=\"==\"]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" parent=\"-\"]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" parent=\"-=\"]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" offset=\"\"]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" offset=\"-1\"]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" offset=\"2.5\"]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" offset=\"+1\"]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" pagetype=\"@URL|normal\"]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" page_type=\"all\"]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" page-type=\"all\"]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" SCORE=\">100000\"]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" Score=\">100000\"]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" rating=\"bad\" rating=\">=-100000\"]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" votes=\"bad\" votes=\">=0\"]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" rating>100000]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" score>100000]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" votes>100000]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" created_at>2100]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" createdat>2100]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" date>2100]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" parent>={TARGET_SLUG}]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" limit>=1]]\nROW %%fullname%%\n[[/module]]",
        ),
    ] {
        let preview = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "Unsaved preview",
            source.clone(),
        )
        .await
        .expect("documented tag and parent sentinels should render")
        .html_output
        .body;
        assert!(
            preview.contains(&format!("ROW {TARGET_SLUG}")),
            "the documented selector should include the matching unparented, untagged page for {source}:\n{preview}",
        );
    }

    for source in [
        format!(
            "[[module ListPages name=\"definitely-missing-page\" fullname=\"{TARGET_SLUG}\"]]\nROW %%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages fullname=\"definitely-missing-page\" name=\"{TARGET_SLUG}\"]]\nROW %%fullname%%\n[[/module]]",
        ),
        "[[module ListPages name=\"definitely-missing-page\" fullname=\"\"]]\nROW %%fullname%%\n[[/module]]"
            .to_owned(),
        "[[module ListPages fullname=\"definitely-missing-page\" name=\"\"]]\nROW %%fullname%%\n[[/module]]"
            .to_owned(),
    ] {
        let preview = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "Unsaved preview",
            source.clone(),
        )
        .await
        .expect("composed canonical and alias name selectors should render")
        .html_output
        .body;
        assert!(
            preview.contains("class=\"list-pages-box\"")
                && !preview.contains("ROW ")
                && !preview.contains("[[module ListPages"),
            "canonical and alias name selectors compose rather than overwrite for {source}:\n{preview}",
        );
    }

    for selector in [r#"rating="=""#, r#"votes="=""#] {
        let source = format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" {selector}]]\nROW %%fullname%%\n[[/module]]",
        );
        let preview = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "Unsaved preview",
            source.clone(),
        )
        .await
        .expect(
            "a current-page score selector without a current page should render empty",
        )
        .html_output
        .body;
        assert!(
            preview.contains("class=\"list-pages-box\"")
                && !preview.contains("ROW ")
                && !preview.contains("[[module ListPages"),
            "Wikidot has no current score/vote source in an unsaved preview for {selector}:\n{preview}",
        );
    }

    let huge_offset_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" offset=\"999999999\"]]\nROW %%fullname%%\n[[/module]]",
        ),
    )
    .await
    .expect("an excessive offset should render an empty result")
    .html_output
    .body;
    assert!(
        huge_offset_preview.contains("class=\"list-pages-box\"")
            && !huge_offset_preview.contains("ROW "),
        "an excessive offset should not preserve the module or query an unsafe window:\n{huge_offset_preview}",
    );

    let empty_random_content_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        concat!(
            "[[module ListPages category=\"verification-listpages-absent-random-content\" ",
            "order=\"random\" perPage=\"250\" limit=\"250\"]]\n",
            "[[div class=\"taleBlock\"]]\n%%content%%\n[[/div]]\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("an empty random content query should render atomically")
    .html_output
    .body;
    assert!(
        empty_random_content_preview.contains(r#"<div class="list-pages-box"></div>"#)
            && !empty_random_content_preview.contains("[[module ListPages")
            && !empty_random_content_preview.contains("%%content%%"),
        "an empty random content query has no content rows to charge against the safety budget:\n{empty_random_content_preview}",
    );

    let zero_row_once_only_output = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        concat!(
            "[[module ListPages category=\"verification-listpages-absent-once-output\" ",
            "separate=\"no\" prependLine=\"ZERO-PREPEND\" ",
            "appendLine=\"ZERO-APPEND\"]]\n",
            "[[head]]ZERO-HEAD[[/head]]\n",
            "[[body]]ZERO-ROW %%fullname%%[[/body]]\n",
            "[[foot]]ZERO-FOOT[[/foot]]\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("zero-row ListPages once-only output should render")
    .html_output
    .body;
    for marker in ["ZERO-PREPEND", "ZERO-HEAD", "ZERO-FOOT", "ZERO-APPEND"] {
        assert!(
            zero_row_once_only_output.contains(marker),
            "zero-row ListPages must retain its once-only output ({marker}):\n{zero_row_once_only_output}",
        );
    }
    assert!(
        !zero_row_once_only_output.contains("ZERO-ROW")
            && !zero_row_once_only_output.contains("[[module ListPages"),
        "zero-row ListPages must not invent a row or preserve its source:\n{zero_row_once_only_output}",
    );

    let zero_row_plain_line_blocks = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        concat!(
            "[[module ListPages category=\"verification-listpages-absent-line-blocks\" ",
            "separate=\"no\" wrapper=\"no\" prependLine=\"PLAIN-PRE\" ",
            "appendLine=\"PLAIN-POST\"]]\n",
            "ZERO-ROW\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("zero-row ListPages plain once-only lines should render")
    .html_output
    .body;
    assert!(
        zero_row_plain_line_blocks.contains("<p>PLAIN-PRE</p><p>PLAIN-POST</p>"),
        "live Wikidot parses zero-row prependLine and appendLine as independent paragraphs:\n{zero_row_plain_line_blocks}",
    );

    let zero_row_continued_prepend = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        concat!(
            "[[module ListPages ",
            "category=\"verification-listpages-absent-continued-prepend\" ",
            "separate=\"no\" prependLine=\"\n**Seiten mit Wertung:** _\n\"]]\n",
            "%%title_linked%%\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a continued zero-row prepend line should render")
    .html_output
    .body;
    assert!(
        zero_row_continued_prepend.contains(concat!(
            r#"<div class="list-pages-box"><strong>Seiten mit Wertung:</strong><br>"#,
            "\n<br>",
        )) && !zero_row_continued_prepend.contains("WIKIJUMPWIKIDOTCOMPATHTML"),
        "the wrapper close must remain a block boundary after a continued prepend line:\n\
         {zero_row_continued_prepend}",
    );

    let zero_row_before_escaped_inline = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        concat!(
            "[[module ListPages ",
            "category=\"verification-listpages-absent-before-escaped-inline\"]]\n",
            "ZERO-ROW\n",
            "[[/module]]\n",
            "@@ @@\n",
            "AFTER",
        )
        .to_owned(),
    )
    .await
    .expect("a zero-row wrapper before escaped inline source should render")
    .html_output
    .body;
    assert!(
        zero_row_before_escaped_inline.contains(r#"<div class="list-pages-box"></div>"#)
            && zero_row_before_escaped_inline
                .contains(r#"<span style="white-space: pre-wrap;"> </span>"#)
            && zero_row_before_escaped_inline.contains("AFTER")
            && !zero_row_before_escaped_inline.contains("WIKIJUMPWIKIDOTCOMPATHTML"),
        "the generated wrapper must remain a sibling block before escaped source:\n\
         {zero_row_before_escaped_inline}",
    );

    let zero_row_table_line_blocks = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        concat!(
            "[[module ListPages category=\"verification-listpages-absent-table-blocks\" ",
            "separate=\"no\" wrapper=\"no\" prependLine=\"||~ TABLE-PRE ||\" ",
            "appendLine=\"|| TABLE-POST ||\"]]\n",
            "ZERO-ROW\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("zero-row ListPages table once-only lines should render")
    .html_output
    .body;
    assert_eq!(
        zero_row_table_line_blocks
            .matches(r#"<table class="wiki-content-table">"#)
            .count(),
        2,
        "live Wikidot parses zero-row table once-only lines as independent tables:\n{zero_row_table_line_blocks}",
    );

    let one_row_table_line_control = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        format!(
            concat!(
                "[[module ListPages name=\"{TARGET_SLUG}\" ",
                "separate=\"no\" wrapper=\"no\" prependLine=\"||~ TABLE-PRE ||\" ",
                "appendLine=\"|| TABLE-POST ||\"]]\n",
                "|| ROW %%fullname%% ||\n",
                "[[/module]]",
            ),
            TARGET_SLUG = TARGET_SLUG,
        ),
    )
    .await
    .expect("one-row ListPages table lines should preserve their shared table")
    .html_output
    .body;
    assert_eq!(
        one_row_table_line_control
            .matches(r#"<table class="wiki-content-table">"#)
            .count(),
        1,
        "the independent block boundary must not split matching one-row table composition:\n{one_row_table_line_control}",
    );

    let empty_dynamic_module_template = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        concat!(
            "[[module ListPages category=\"verification-listpages-absent-dynamic-module\" ",
            "limit=\"1\" order=\"_date desc\" _category=\"3\" _original=\"1\" ",
            "wrapper=\"no\"]]\n",
            "[[%%content{0}%%module css]]\n.hidden { display: none; }\n",
            "[[%%content{0}%%/module]]\n%%total%%\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("an empty ListPages dynamic module template should render atomically")
    .html_output
    .body;
    assert!(
        !empty_dynamic_module_template.contains("[[module")
            && !empty_dynamic_module_template.contains("%%content")
            && !empty_dynamic_module_template.contains("%%total%%")
            && !empty_dynamic_module_template.contains(".hidden"),
        "the ListPages scanner must retain ownership through variable-controlled module tokens:\n{empty_dynamic_module_template}",
    );

    let absent_current_data_form_context = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved preview",
        concat!(
            "[[module ListPages tags=\"{$scp-tag}\" created_by=\"{$user}\" ",
            "order=\"rating desc\" limit=\"1\" separate=\"no\" wrapper=\"no\"]]\n",
            "[[#ifexpr %%total%% > 1 |  | [!-- ]]\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("an unsaved preview has no current data-form context")
    .html_output
    .body;
    assert!(
        absent_current_data_form_context.is_empty(),
        "current data-form selectors without a current page resolve to an empty unwrapped module, even when a conditional comment opener appears in its body:\n{absent_current_data_form_context}",
    );

    let absent_current_page_unsupported_template =
        RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "Unsaved preview",
            concat!(
                "[[module ListPages limit=\"@URL|0\" range=\".\" ",
                "urlAttrPrefix=\"page5\"]]\n",
                "%%unsupported%%\n[[%%content{0}%%module css]]\n",
                ".unused { display: none; }\n[[%%content{0}%%/module]]\n",
                "[[/module]]",
            )
            .to_owned(),
        )
        .await
        .expect("a zero-limit ListPages query should not inspect its row template")
        .html_output
        .body;
    assert!(
        absent_current_page_unsupported_template
            .contains(r#"<div class="list-pages-box"></div>"#)
            && !absent_current_page_unsupported_template.contains("[[module")
            && !absent_current_page_unsupported_template.contains("%%unsupported%%")
            && !absent_current_page_unsupported_template.contains(".unused"),
        "a current-page query without page identity is empty before unsupported row-template syntax matters:\n{absent_current_page_unsupported_template}",
    );

    let absent_category_precedes_dropped_url_selectors =
        RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "Unsaved preview",
            concat!(
                "[[module ListPages category=\"verification-listpages-absent-url-category\" ",
                "order=\"@URL|name desc\" tags=\"@URL|\" perPage=\"@URL|50\" ",
                "name=\"@URL\" _original=\"@URL\"]]\n",
                "UNSUPPORTED %%form_raw{content}%%\n",
                "[[/module]]",
            )
            .to_owned(),
        )
        .await
        .expect("an absent static category should prove the query empty")
        .html_output
        .body;
    assert!(
        absent_category_precedes_dropped_url_selectors
            .contains(r#"<div class="list-pages-box"></div>"#)
            && !absent_category_precedes_dropped_url_selectors.contains("[[module")
            && !absent_category_precedes_dropped_url_selectors.contains("UNSUPPORTED"),
        "an absent static category proves the result empty before dropped URL selectors or unsupported row-template variables matter:\n{absent_category_precedes_dropped_url_selectors}",
    );

    for source in [
        "[[module ListPages range=\".\"]]\nROW %%fullname%%\n[[/module]]",
        "[[module ListPages name=\"=\"]]\nROW %%fullname%%\n[[/module]]",
        "[[module ListPages tags=\"=\"]]\nROW %%fullname%%\n[[/module]]",
        "[[module ListPages parent=\".\"]]\nROW %%fullname%%\n[[/module]]",
        "[[module ListPages parent=\"=\"]]\nROW %%fullname%%\n[[/module]]",
        "[[module ListPages created_at=\"=\"]]\nROW %%fullname%%\n[[/module]]",
        "[[module ListPages updated_at=\"=\"]]\nROW %%fullname%%\n[[/module]]",
        "[[module ListPages created_by=\"=\"]]\nROW %%fullname%%\n[[/module]]",
        "[[module ListPages category=\"*\" created_by=\"-=\"]]\nROW %%fullname%%\n[[/module]]",
    ] {
        let preview = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "Unsaved preview",
            source.to_owned(),
        )
        .await
        .expect("a current-page selector should render an empty unsaved preview")
        .html_output
        .body;
        assert!(
            preview.contains("class=\"list-pages-box\"") && !preview.contains("ROW "),
            "a preview without saved page identity should produce an empty ListPages wrapper:\n{preview}",
        );
    }

    for (source, message) in [
        (
            "[[module ListPages range=\"before\"]]\nROW %%fullname%%\n[[/module]]",
            "Invalid range argument.",
        ),
        (
            "[[module ListPages range=\"after\"]]\nROW %%fullname%%\n[[/module]]",
            "Invalid range argument.",
        ),
        (
            "[[module ListPages range=\"others\"]]\nROW %%fullname%%\n[[/module]]",
            "Invalid range argument.",
        ),
        (
            "[[module ListPages range=\"@URL|others\"]]\nROW %%fullname%%\n[[/module]]",
            "Invalid range argument.",
        ),
        (
            "[[module ListPages range=\"bogus\"]]\nROW %%fullname%%\n[[/module]]",
            "Invalid range argument.",
        ),
        (
            "[[module ListPages range=\".\" range=\"others\"]]\nROW %%fullname%%\n[[/module]]",
            "Invalid range argument.",
        ),
        (
            "[[module ListPages pagetype=\"bogus\"]]\nROW %%fullname%%\n[[/module]]",
            "Invalid pagetype attribute.",
        ),
        (
            "[[module ListPages rating=\"bad\"]]\nROW %%fullname%%\n[[/module]]",
            "Invalid rating argument.",
        ),
        (
            "[[module ListPages votes=\"bad\"]]\nROW %%fullname%%\n[[/module]]",
            "Invalid votes argument.",
        ),
        (
            "[[module ListPages parent=\"definitely-missing-listpages-parent\"]]\nROW %%fullname%%\n[[/module]]",
            "Parent page definitely-missing-listpages-parent does not exist",
        ),
        (
            "[[module ListPages parent=\"definitely-missing-listpages-parent\" range=\"others\"]]\nROW %%fullname%%\n[[/module]]",
            "Parent page definitely-missing-listpages-parent does not exist",
        ),
        (
            "[[module ListPages parent=\"definitely-missing-listpages-parent\" range=\"bogus\"]]\nROW %%fullname%%\n[[/module]]",
            "Parent page definitely-missing-listpages-parent does not exist",
        ),
        (
            "[[module ListPages parent=\"definitely-missing-listpages-parent\" pagetype=\"bogus\"]]\nROW %%fullname%%\n[[/module]]",
            "Invalid pagetype attribute.",
        ),
    ] {
        let preview = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "Unsaved preview",
            source.to_owned(),
        )
        .await
        .expect("invalid ListPages arguments should render a live-compatible error")
        .html_output
        .body;
        assert!(
            preview.contains(&format!(r#"<div class="error-block">{message}</div>"#,)),
            "the exact live ListPages error should render:\n{preview}",
        );
    }
}

#[tokio::test]
async fn listpages_default_rows_expand_page_body_runtime_modules() {
    const TARGET_SLUG: &str = "listpages-default-runtime-module-target";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    runner
        .context()
        .transaction()
        .execute_raw(Statement::from_sql_and_values(
            runner.context().transaction().get_database_backend(),
            "UPDATE site SET layout = $1 WHERE site_id = $2",
            [Value::String(Some("wikidot".to_owned())), site_id.into()],
        ))
        .await
        .expect("the regression fixture should use Wikidot site layout");

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
            "wikitext": concat!(
                "ANONUN_START\n",
                "[[module AnonymousNotificationsUnsubscribe]]\n",
                "ANONUN_END\n",
                "DASH_START\n",
                "[[module Dashboard]]\n",
                "DASH_END\n",
                "USERINFO_START\n",
                "[[module UserInfo]]\n",
                "USERINFO_END\n",
                "SEARCHUSERS_START\n",
                "[[module SearchUsers]]\n",
                "SEARCHUSERS_END\n",
                "WATCHERS_START\n",
                "[[module Watchers]]\n",
                "WATCHERS_END\n",
                "WHO_START\n",
                "[[module WhoInvited]]\n",
                "WHO_END\n",
                "THEME_START\n",
                "[[module ThemePreviewer]]\n",
                "THEME_END\n",
                "MBE_START\n",
                "[[module MembershipEmailInvitation]]\n",
                "MBE_END\n",
                "MBP_START\n",
                "[[module MembershipByPassword]]\n",
                "MBP_END\n",
                "JOIN_START\n",
                "[[module Join]]\n",
                "JOIN_END",
            ),
            "title": "ListPages Default Runtime Module Target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "ListPages default runtime-module fixture",
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
    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved ListPages runtime-module preview",
        format!(r#"[[module ListPages name="{TARGET_SLUG}"]]"#),
    )
    .await
    .expect("ListPages should render the selected page's default row")
    .html_output
    .body;

    assert!(
        preview.contains("ANONUN_START")
            && preview.contains(
                r#"<div class="error-block">Invalid indentification token.</div>"#,
            )
            && preview.contains("ANONUN_END")
            && preview.contains("DASH_START")
            && preview.contains(
                r#"<div class="error-block">Not allowed. Error.</div>"#,
            )
            && preview.contains("DASH_END")
            && preview.contains("WATCHERS_START")
            && preview.contains("WATCHERS_END")
            && preview.contains("WATCHERS_START</p>\n\n\n\n\n<p>WATCHERS_END")
            && preview.contains("THEME_START")
            && preview.contains(
                r#"<div class="error-block">Preview mode error: please contact Wikidot.com for a better error message</div>"#,
            )
            && preview.contains("THEME_END")
            && preview.contains("JOIN_START")
            && preview.contains(
                r#"<div class="join-box"><a href="javascript:;" onclick="WIKIDOT.page.listeners.join(event, 'unified')">Join</a></div>"#,
            )
            && preview.contains("JOIN_END")
            && !preview.contains("WIKIJUMPWIKIDOTCOMPATHTML"),
        "a runtime module in selected page content should execute inside the ListPages row:\n{preview}",
    );
    assert!(
        !preview.contains("[[module AnonymousNotificationsUnsubscribe")
            && !preview.contains("[[module Dashboard")
            && !preview.contains("[[module Watchers")
            && !preview.contains("[[module ThemePreviewer")
            && !preview.contains("No such module"),
        "the selected page's runtime module must not reach FTML as unsupported source:\n{preview}",
    );
}

#[tokio::test]
async fn listpages_at_marker_footnote_tail_executes_default_preview() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    runner
        .context()
        .transaction()
        .execute_raw(Statement::from_sql_and_values(
            runner.context().transaction().get_database_backend(),
            "UPDATE site SET layout = $1 WHERE site_id = $2",
            [Value::String(Some("wikidot".to_owned())), site_id.into()],
        ))
        .await
        .expect("the regression fixture should use Wikidot site layout");

    let source = "[[module Listpages @@以降という認識で良い。 [[/footnote]]";
    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Malformed ListPages opener preview",
        source.to_owned(),
    )
    .await
    .expect("the evidenced malformed opener should execute default ListPages")
    .html_output
    .body;

    assert!(
        preview.contains(r#"<div class="list-pages-box">"#),
        "the malformed opener should execute the default ListPages query:\n{preview}",
    );
    assert!(
        !preview.contains(source) && !preview.contains("[[/footnote]]"),
        "the evidenced malformed opener tail should be consumed:\n{preview}",
    );
}

#[tokio::test]
async fn listpages_default_rows_render_only_the_first_paragraph() {
    const TARGET_SLUG: &str = "listpages-default-first-paragraph-target";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    runner
        .context()
        .transaction()
        .execute_raw(Statement::from_sql_and_values(
            runner.context().transaction().get_database_backend(),
            "UPDATE site SET layout = $1 WHERE site_id = $2",
            [Value::String(Some("wikidot".to_owned())), site_id.into()],
        ))
        .await
        .expect("the regression fixture should use Wikidot site layout");

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
            "wikitext": "FIRST_PARAGRAPH\n\nSECOND_PARAGRAPH",
            "title": "ListPages Default First Paragraph Target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "ListPages default first-paragraph fixture",
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
    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved ListPages first-paragraph preview",
        format!(r#"[[module ListPages name="{TARGET_SLUG}"]]"#),
    )
    .await
    .expect("default ListPages should render the selected page")
    .html_output
    .body;

    assert!(
        preview.contains("FIRST_PARAGRAPH"),
        "the default row should render the selected page's first paragraph:\n{preview}",
    );
    assert!(
        !preview.contains("SECOND_PARAGRAPH"),
        "an unsectioned PagePreview default row keeps Wikidot's first-paragraph recovery:\n{preview}",
    );

    let explicit_empty_body = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved explicit-empty-body ListPages preview",
        format!(
            concat!(
                "[[module ListPages name=\"{TARGET_SLUG}\" separate=\"no\" wrapper=\"no\"]]\n",
                "[[head]]EXPLICIT_HEAD[[/head]]\n",
                "[[body]][[/body]]\n",
                "[[foot]]EXPLICIT_FOOT[[/foot]]\n",
                "[[/module]]",
            ),
            TARGET_SLUG = TARGET_SLUG,
        ),
    )
    .await
    .expect("an explicit empty body should use the full default summary")
    .html_output
    .body;
    assert!(
        explicit_empty_body.contains("FIRST_PARAGRAPH")
            && explicit_empty_body.contains("SECOND_PARAGRAPH")
            && explicit_empty_body.contains("EXPLICIT_HEAD")
            && explicit_empty_body.contains("EXPLICIT_FOOT"),
        "an explicit body section uses the complete first content section:\n{explicit_empty_body}",
    );
    assert!(
        !explicit_empty_body.contains("<p><h1"),
        "an unwrapped default row must remain in FTML's block stream:\n{explicit_empty_body}",
    );
}

#[tokio::test]
async fn listpages_selected_content_evaluates_iftags_against_selected_page_tags() {
    const TARGET_SLUG: &str = "listpages-selected-iftags-target";

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
            "wikitext": "[[iftags +component]]SELECTED_TAG_BODY[[/iftags]]",
            "title": "ListPages Selected Iftags Target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "tags": ["component"],
            "layout": "wikidot",
            "revision_comments": "ListPages selected iftags fixture",
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
    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved ListPages selected iftags preview",
        format!(
            r#"[[module ListPages name="{TARGET_SLUG}" separate="no" wrapper="no"]]%%content%%[[/module]]"#,
        ),
    )
    .await
    .expect("selected page content should render")
    .html_output
    .body;

    assert!(
        preview.contains("SELECTED_TAG_BODY"),
        "selected-page iftags must use the selected row tags:\n{preview}",
    );
    assert!(
        !preview.contains("[[iftags") && !preview.contains("[[/iftags]]"),
        "selected-page iftags markers must not leak into rendered HTML:\n{preview}",
    );
}

#[tokio::test]
async fn listpages_default_rows_suppress_nested_rate_modules() {
    const TARGET_SLUG: &str = "listpages-default-rate-module-target";

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
            "wikitext": "[[module Rate]]\n\nRATE_START\nRATE_END",
            "title": "ListPages Default Rate Module Target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "ListPages nested Rate fixture",
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
    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved ListPages nested Rate preview",
        format!(r#"[[module ListPages name="{TARGET_SLUG}"]]"#),
    )
    .await
    .expect("ListPages should render the selected page's default row")
    .html_output
    .body;

    // Live page-preview evidence a3a16270... selects three rating-enabled
    // pages and emits their surrounding content without any Rate module DOM.
    assert!(
        preview.contains("RATE_START") && preview.contains("RATE_END"),
        "ListPages should retain content surrounding a nested Rate module:\n{preview}",
    );
    assert!(
        !preview.contains("[[module Rate")
            && !preview.contains("TODO: module Rate")
            && !preview.contains("page-rate-widget-box"),
        "Wikidot suppresses Rate modules while rendering selected page content:\n{preview}",
    );
}

#[tokio::test]
async fn listpages_default_rows_expand_secondary_page_body_modules() {
    const TARGET_SLUG: &str = "listpages-default-secondary-module-target";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    runner
        .context()
        .transaction()
        .execute_raw(Statement::from_sql_and_values(
            runner.context().transaction().get_database_backend(),
            "UPDATE site SET layout = $1 WHERE site_id = $2",
            [Value::String(Some("wikidot".to_owned())), site_id.into()],
        ))
        .await
        .expect("the regression fixture should use Wikidot site layout");

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
            "wikitext": concat!(
                "ANONUN_START\n",
                "[[module AnonymousNotificationsUnsubscribe]]\n",
                "ANONUN_END\n",
                "DASH_START\n",
                "[[module Dashboard]]\n",
                "DASH_END\n",
                "USERINFO_START\n",
                "[[module UserInfo]]\n",
                "USERINFO_END\n",
                "SEARCHUSERS_START\n",
                "[[module SearchUsers]]\n",
                "SEARCHUSERS_END\n",
                "WATCHERS_START\n",
                "[[module Watchers]]\n",
                "WATCHERS_END\n",
                "WHO_START\n",
                "[[module WhoInvited]]\n",
                "WHO_END\n",
                "THEME_START\n",
                "[[module ThemePreviewer]]\n",
                "THEME_END\n",
                "EMAIL_START\n",
                "[[module MembershipEmailInvitation]]\n",
                "EMAIL_END\n",
                "PASSWORD_START\n",
                "[[module MembershipByPassword]]\n",
                "PASSWORD_END\n",
                "INVITE_START\n",
                "[[module SendInvitations]]\n",
                "INVITE_END\n",
                "TODO_START\n",
                "[[module SimpleToDo]]\n",
                "TODO_END\n",
                "ADSENSE_START\n",
                "[[module AdSenseUnit]]\n",
                "ADSENSE_END\n",
                "FEATURED_START\n",
                "[[module FeaturedSite]]\n",
                "FEATURED_END\n",
                "JOIN_START\n",
                "[[module Join]]\n",
                "JOIN_END\n",
                "CLONE_START\n",
                "[[module Clone button=\"CLONE_BUTTON\"]]\n",
                "CLONE_END\n",
                "BACKLINKS_START\n",
                "[[module Backlinks]]\n",
                "BACKLINKS_END\n",
                "PREVIOUS_START\n",
                "[[module PreviousPage]]\n",
                "PREVIOUS_END\n",
                "NEXT_START\n",
                "[[module NextPage]]\n",
                "NEXT_END\n",
                "PETITION_START\n",
                "[[module PetitionAdmin]]\n",
                "PETITION_END\n",
                "SITEGRID_START\n",
                "[[module SiteGrid]]\n",
                "SITEGRID_END\n",
                "SOCIAL_START\n",
                "[[social reddit,facebook]]\n",
                "SOCIAL_END\n",
                "HTML_START\n",
                "[[html]]\n",
                "<div id=\"nested-html-probe\">NESTED_HTML_PAYLOAD</div>\n",
                "[[/html]]\n",
                "HTML_END",
            ),
            "title": "ListPages Default Secondary Module Target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "ListPages secondary runtime-module fixture",
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
    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved ListPages secondary-module preview",
        format!(r#"[[module ListPages name="{TARGET_SLUG}"]]"#),
    )
    .await
    .expect("ListPages should render the selected page's default row")
    .html_output
    .body;

    for marker in [
        "PASSWORD_START",
        "PASSWORD_END",
        "INVITE_START",
        "INVITE_END",
        "TODO_START",
        "TODO_END",
        "ADSENSE_START",
        "ADSENSE_END",
        "FEATURED_START",
        "FEATURED_END",
        "JOIN_START",
        "JOIN_END",
        "CLONE_START",
        "CLONE_END",
        "BACKLINKS_START",
        "BACKLINKS_END",
        "PREVIOUS_START",
        "PREVIOUS_END",
        "NEXT_START",
        "NEXT_END",
        "PETITION_START",
        "PETITION_END",
        "SITEGRID_START",
        "SITEGRID_END",
        "SOCIAL_START",
        "SOCIAL_END",
        "HTML_START",
        "HTML_END",
    ] {
        assert!(
            preview.contains(marker),
            "ListPages should preserve {marker} around nested module output:\n{preview}",
        );
    }
    assert!(
        preview.contains("Invalid indentification token.")
            && preview.contains("Inviting users has been disabled due to severe abuse.")
            && preview.contains("The SimpleTodo module must have an id.")
            && preview.contains(
                r#"<div class="join-box"><a href="javascript:;" onclick="WIKIDOT.page.listeners.join(event, 'unified')">Join</a></div>"#,
            )
            && preview.contains("You should be logged in to clone a site.")
            && preview.contains(r#"<div class="backlinks-module-box"></div>"#)
            && preview
                .matches("The ListPages module does not work recursively.")
                .count()
                == 2
            && preview.contains(r#"<div class="title">Permission error</div>"#)
            && preview.contains(
                "This tool is for use by the administrators of this site",
            )
            && preview.contains("No sites provided.")
            && preview.contains("[[module <em>FeaturedSite</em>]] No such module")
            && !preview.contains("thumbnails.wdfiles.com")
            && !preview.contains("OZONE.dialog.hovertip"),
        "selected page content should use the same secondary runtime-module handlers as a page view:\n{preview}",
    );
    for module in [
        "MembershipByPassword",
        "SendInvitations",
        "SimpleToDo",
        "AdSenseUnit",
        "FeaturedSite",
        "Join",
        "Clone",
        "Backlinks",
        "PreviousPage",
        "NextPage",
        "PetitionAdmin",
        "SiteGrid",
    ] {
        assert!(
            !preview.contains(&format!("[[module {module}"))
                && !preview.contains(&format!("TODO: module {module}")),
            "nested {module} source should be consumed inside ListPages:\n{preview}",
        );
    }
    assert!(
        preview.contains(r#"<span id="social"#)
            && preview.contains("http://reddit.com/submit?url=http%3A%2F%2Fscp-wiki.wikidot.com%2Fajax-module-connector.php")
            && preview.contains("http://www.facebook.com/share.php?u=http%3A%2F%2Fscp-wiki.wikidot.com%2Fajax-module-connector.php")
            && preview.contains(r##"var socialspan = $j("#social"##)
            && !preview.contains("[[social"),
        "selected page content should render the evidenced legacy social widget and its matching runtime script:\n{preview}",
    );
    assert!(
        preview.contains(concat!(
            "INVITE_START</p>",
            r#"<div class="error-block">Inviting users has been disabled"#,
        )) && preview.contains(concat!(
            "site admin dashboard</a>.</div>",
            "<p>INVITE_END<br>\nTODO_START</p>",
            r#"<div class="error-block">The SimpleTodo module must have an id.</div>"#,
            "<p>TODO_END",
        )),
        "selected block-valued module errors must split their surrounding paragraphs exactly once:\n{preview}",
    );
    assert!(
        preview.contains("ADSENSE_START</p>\n\n<p>ADSENSE_END"),
        "the empty AdSenseUnit module must retain Wikidot's paragraph boundary:\n{preview}",
    );
    assert!(
        preview.contains(r#"class="html-block-iframe""#)
            && preview
                .contains(r#"src="/listpages-default-secondary-module-target/html/1""#,)
            && !preview.contains(r#"src="https://example.com/""#)
            && !preview.contains("[[html]]")
            && !preview.contains("NESTED_HTML_PAYLOAD"),
        "selected saved-page HTML blocks should retain their iframe boundary in ListPages preview:\n{preview}",
    );
    assert!(
        !preview.contains("WIKIJUMPWIKIDOTCOMPATHTML"),
        "nested trusted fragments must be fully restored before the ListPages row is sealed:\n{preview}",
    );
}

#[tokio::test]
async fn listpages_selected_content_splits_terminal_embed_error_from_its_paragraph() {
    const TARGET_SLUG: &str = "listpages-terminal-embed-error-target";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    runner
        .context()
        .transaction()
        .execute_raw(Statement::from_sql_and_values(
            runner.context().transaction().get_database_backend(),
            "UPDATE site SET layout = $1 WHERE site_id = $2",
            [Value::String(Some("wikidot".to_owned())), site_id.into()],
        ))
        .await
        .expect("the regression fixture should use Wikidot site layout");

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
            "wikitext": concat!(
                "DOC_EMBEDAUDIO_BEGIN\n",
                "[[embedaudio]]\n",
                "<div id=\"doc-embedaudio-probe\">DOC_EMBEDAUDIO_PAYLOAD</div>\n",
                "[[/embedaudio]]\n",
                "DOC_EMBEDAUDIO_END",
            ),
            "title": "ListPages Terminal Embed Error Target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "ListPages terminal embed error fixture",
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
    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unsaved ListPages terminal embed preview",
        format!(r#"[[module ListPages name="{TARGET_SLUG}"]]"#),
    )
    .await
    .expect("ListPages should render the selected page's default row")
    .html_output
    .body;

    assert!(
        preview.contains(concat!(
            "DOC_EMBEDAUDIO_BEGIN<br>\n</p>",
            "<div class=\"error-block\">",
            "Sorry, no match for the embedded content.</div>",
            "<br>\nDOC_EMBEDAUDIO_END</div>",
        ),) && !preview.contains("DOC_EMBEDAUDIO_END</p>"),
        "Wikidot closes the leading paragraph before the unsupported embed and leaves its terminal text at row level:\n{preview}",
    );
}

#[tokio::test]
async fn linked_listpages_values_keep_typed_owner_boundaries_in_preview() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (label, template, expected) in [
        (
            "active page link",
            "**%%title_linked%%**",
            concat!(
                "<strong><a href=\"/component:image-block\">",
                "Standard Image Block</a></strong>",
            ),
        ),
        (
            "page legacy raw projection",
            "@@%%title_linked%%@@",
            concat!(
                "<span style=\"white-space: pre-wrap;\">",
                "[[[component:image-block | Standard Image Block]]]</span>",
            ),
        ),
        (
            "page code shell",
            "[[code]]\n%%title_linked%%\n[[/code]]",
            concat!(
                "[[code]]<br>\n",
                "<a href=\"/component:image-block\">Standard Image Block</a>",
                "<br>\n[[/code]]",
            ),
        ),
        (
            "page parser function recovery",
            "[[#if true | %%title_linked%% | NO]]",
            "[[[component:image-block] | NO]]",
        ),
        (
            "page link in outer-page tag conditional",
            "[[iftags +component]]%%title_linked%%[[/iftags]]",
            "BEGIN||END",
        ),
        (
            "active tag link",
            "**%%tags_linked%%**",
            concat!(
                "<strong><a href=\"/system:page-tags/tag/component\">",
                "component</a></strong>",
            ),
        ),
        (
            "tag external label recovery",
            "[https://example.com %%tags_linked%%]",
            concat!(
                "<a href=\"https://example.com\">",
                "[/system:page-tags/tag/component component</a>]",
            ),
        ),
        (
            "tag image link projection",
            "[[image https://example.com/x.png link=\"%%tags_linked%%\"]]",
            concat!(
                "<a href=\"/[/system:page-tags/tag/component%20component]\">",
                "<img src=\"https://example.com/x.png\"",
            ),
        ),
        (
            "tag image alt projection",
            "[[image https://example.com/x.png alt=\"%%tags_linked%%\"]]",
            concat!(
                "<img src=\"https://example.com/x.png\" class=\"image\" ",
                "alt=\"[/system:page-tags/tag/component component]\">",
            ),
        ),
        (
            "tag link in outer-page tag conditional",
            "[[iftags +component]]%%tags_linked%%[[/iftags]]",
            "BEGIN||END",
        ),
    ] {
        let preview = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "Typed ListPages preview",
            format!(
                concat!(
                    "[[module ListPages category=\"*\" ",
                    "name=\"component:image-block\" separate=\"no\" wrapper=\"no\"]]\n",
                    "BEGIN|{template}|END\n",
                    "[[/module]]",
                ),
                template = template,
            ),
        )
        .await
        .unwrap_or_else(|error| panic!("{label} should render: {error:?}"))
        .html_output
        .body;

        assert!(
            preview.contains(expected),
            "{label} should preserve its evidenced owner boundary:\n{preview}",
        );
        if label.starts_with("tag image ") {
            assert!(
                !preview.contains("<p>BEGIN|") && !preview.contains("|END</p>"),
                "generated image attributes keep Wikidot's delayed block boundary:\n{preview}",
            );
        }
        assert!(
            !preview.contains("%%title_linked%%")
                && !preview.contains("%%tags_linked%%")
                && !preview.contains("TODO: module ListPages"),
            "{label} must not leak a generated slot or module placeholder:\n{preview}",
        );
    }
}

#[tokio::test]
async fn listpages_runtime_scalar_title_stays_inert_in_delayed_row() {
    const TARGET_SLUG: &str = "listpages-runtime-scalar-title-20260807";
    const ATTACK_TITLE: &str = "**RUNTIME_SCALAR_MARKUP**";

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
            "wikitext": "Runtime scalar title target",
            "title": ATTACK_TITLE,
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "runtime scalar title security regression",
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
    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Runtime scalar title preview",
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" separate=\"no\"]]\n\
             [[row]]\n\
             TITLE=%%title%%\n\
             [[/row]]\n\
             [[/module]]",
        ),
    )
    .await
    .expect("ListPages should render the runtime-scalar title fixture")
    .html_output
    .body;

    assert!(
        preview.contains("TITLE=**RUNTIME_SCALAR_MARKUP**"),
        "a database title must remain literal text after delayed binding:\n{preview}",
    );
    assert!(
        !preview.contains("<strong>RUNTIME_SCALAR_MARKUP</strong>"),
        "a runtime scalar must not acquire authored markup authority:\n{preview}",
    );
    assert!(
        !preview.contains("TODO: module ListPages") && !preview.contains("<iframe"),
        "a runtime scalar must not create a network-capable iframe or fall back to a module placeholder:\n{preview}",
    );
}

#[tokio::test]
async fn delayed_listpages_unicode_space_div_owner_stays_utf8_aligned() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Delayed ListPages Unicode-space div preview",
        concat!(
            "[[module ListPages limit=\"1\"]]\n",
            "[[\u{2003}div]]%%title_linked%%[[/div]]\n",
            "<br>\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("Unicode-spaced delayed div ownership must not panic")
    .html_output
    .body;

    assert!(
        !preview.contains("%%title_linked%%")
            && !preview.contains("TODO: module ListPages")
            && preview.contains("<br>"),
        "the delayed row should bind its page link and preserve following content:\n{preview}",
    );
}

#[tokio::test]
async fn linked_listpages_values_keep_the_runtime_wrapper_outside_list_mode() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Typed ListPages wrapper",
        concat!(
            "[[module ListPages category=\"*\" ",
            "name=\"component:image-block\" separate=\"no\"]]\n",
            "**%%title_linked%%**\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a linked ListPages row with the default wrapper should render")
    .html_output
    .body;

    assert!(
        preview.contains(concat!(
            "<div class=\"list-pages-box\">",
            "<p><strong><a href=\"/component:image-block\">",
            "Standard Image Block</a></strong></p>",
        )) && preview.trim_end().ends_with("</div>")
            && preview.matches(r#"<div class="list-pages-box">"#).count() == 1
            && preview.matches("</div>").count() == 1,
        "the runtime wrapper must not make the delayed List-mode row literal:\n{preview}",
    );
    assert!(
        !preview.contains("[[div class=") && !preview.contains("[[/div]]"),
        "the generated wrapper must not leak as literal source:\n{preview}",
    );
}

#[tokio::test]
async fn static_listpages_numbered_rows_remain_generated_block_html() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages numbered rows",
        concat!(
            "[[module ListPages category=\"*\" ",
            "name=\"component:image-block\" separate=\"no\"]]\n",
            "# [[[first-target|First row]]] _\n",
            "[[span class=\"page_tags\"]] [[/span]]\n",
            "# [[[*https://example.com/second-target|Second row]]] _\n",
            "[[span class=\"page_tags\"]]Second detail[[/span]]\n",
            "# [[[third-target|Third row]]] _\n",
            "[[span class=\"page_tags\"]]\u{3000}[[/span]]\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("static numbered ListPages rows should render")
    .html_output
    .body;

    assert!(
        preview.contains(r#"<div class="list-pages-box"><ol>"#)
            && preview.contains(concat!(
                r#"<li><a class="newpage" href="/first-target">First row</a><br>"#,
                "\n</li>",
            ))
            && !preview.contains(r#"<span class="page_tags"> </span>"#)
            && preview.contains(concat!(
                r#"<li><a target="_blank" href="https://example.com/second-target">"#,
                "Second row</a><br>",
                "\n",
                r#"<span class="page_tags">Second detail</span></li>"#,
            ))
            && preview.contains(concat!(
                r#"<li><a class="newpage" href="/third-target">Third row</a><br>"#,
                "\n",
                r#"<span class="page_tags">　</span></li>"#,
            ))
            && preview.matches("<ol>").count() == 1
            && preview.contains("</ol>")
            && !preview.contains("&lt;ol")
            && !preview.contains("data-wikijump-compat-listpages"),
        "generated numbered rows must cross the trusted block boundary intact:\n{preview}",
    );
}

#[tokio::test]
async fn static_listpages_numbered_rows_retain_inline_footnotes() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages numbered-row footnote",
        concat!(
            "[[module ListPages category=\"*\" ",
            "name=\"component:image-block\" separate=\"no\"]]\n",
            "# Row[[footnote]]Detail -- //Author.//[[/footnote]]\n",
            "[[footnoteblock]]\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a static numbered ListPages row with a footnote should render")
    .html_output
    .body;

    assert!(
        preview.contains(concat!(
            "<ol>\n<li>Row<sup class=\"footnoteref\">",
            "<a id=\"footnoteref-1\" href=\"javascript:;\" ",
        )) && preview.contains(r#"<div class="footnote-footer" id="footnote-1">"#,)
            && preview.contains("<em>Author.</em>")
            && !preview.contains("[[footnote]]")
            && !preview.contains("[[/footnote]]"),
        "generated numbered rows must leave inline footnotes to FTML List mode:\n{preview}",
    );
}

#[tokio::test]
async fn linked_listpages_values_keep_separate_row_containers_outside_list_mode() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Typed ListPages row container",
        concat!(
            "[[module ListPages category=\"*\" ",
            "name=\"component:image-block\"]]\n",
            "**%%title_linked%%**\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a linked ListPages row with separate containers should render")
    .html_output
    .body;

    assert!(
        preview.contains(concat!(
            "<div class=\"list-pages-box\">",
            "<div class=\"list-pages-item\">",
            "<p><strong><a href=\"/component:image-block\">",
            "Standard Image Block</a></strong></p>",
        )) && preview.trim_end().ends_with("</div>")
            && preview.matches(r#"<div class="list-pages-box">"#).count() == 1
            && preview.matches(r#"<div class="list-pages-item">"#).count() == 1
            && preview.matches("</div>").count() == 2,
        "fixed runtime containers must wrap the FTML-rendered row:\n{preview}",
    );
    assert!(
        !preview.contains("[[div class=") && !preview.contains("[[/div]]"),
        "no generated container may leak as literal source:\n{preview}",
    );
}

#[tokio::test]
async fn listpages_separate_row_does_not_append_wrapper_indent_to_unwrapped_text() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages unwrapped row tail",
        concat!(
            "[[module ListPages category=\"*\" ",
            "name=\"component:image-block\"]]\n",
            "[[=]]\n",
            "> //centered//\n",
            "[[/=]]\n",
            "after\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a ListPages row with an unwrapped trailing line should render")
    .html_output
    .body;

    assert!(
        preview.contains(concat!(
            "<div class=\"list-pages-box\">",
            "<div class=\"list-pages-item\">",
            "<div style=\"text-align: center;\">",
            "<blockquote><p><em>centered</em></p></blockquote>",
            "</div><br>\n",
            "after</div></div>",
        )),
        "the generated row close must not become part of the unwrapped text node:\n{preview}",
    );
}

#[tokio::test]
async fn zero_row_listpages_expands_generated_missing_includes() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Generated ListPages include",
        concat!(
            "[[module ListPages category=\"definitely-missing-listpages-category\" ",
            "separate=\"no\" prependLine=\"[[include ",
            "component:definitely-missing-listpages-include]]\" ",
            "appendLine=\"AFTER-INCLUDE\"]]\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a generated missing include should render its normal error")
    .html_output
    .body;

    assert!(
        preview.contains(concat!(
            "Included page &quot;component:definitely-missing-listpages-include&quot; ",
            "does not exist (",
            "<a href=\"/component:definitely-missing-listpages-include/edit/true\">",
            "create it now</a>)",
        )),
        "ListPages output must re-enter runtime include expansion:\n{preview}",
    );
    assert!(preview.contains("AFTER-INCLUDE"), "{preview}");
    assert!(
        !preview.contains("[[include"),
        "a generated include must not leak as literal source:\n{preview}",
    );
}

#[tokio::test]
async fn one_row_listpages_expands_generated_missing_includes() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Generated ListPages include with a row",
        concat!(
            "[[module ListPages category=\"*\" name=\"component:image-block\" ",
            "separate=\"no\" ",
            "prependLine=\"[[include ",
            "component:definitely-missing-listpages-include]]\" ",
            "appendLine=\"AFTER-INCLUDE\" rss=\"Generated row feed\"]]\n",
            "%%title_linked%%\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a generated missing include before a ListPages row should render")
    .html_output
    .body;

    assert!(
        preview.contains(concat!(
            "Included page &quot;component:definitely-missing-listpages-include&quot; ",
            "does not exist (",
            "<a href=\"/component:definitely-missing-listpages-include/edit/true\">",
            "create it now</a>)",
        )),
        "a generated missing include must still expand when ListPages emits rows:\n{preview}",
    );
    assert!(
        preview.contains("Standard Image Block") && !preview.contains("[[include"),
        "the generated row should remain present without leaking include syntax:\n{preview}",
    );
    assert!(
        preview.contains("AFTER-INCLUDE</p>") && !preview.contains("AFTER-INCLUDE<br"),
        "row-bearing once-only output must not invent a trailing line break:\n{preview}",
    );
    assert!(
        preview.contains(r#"<div class="feedinfo">"#)
            && preview.contains(">RSS feed</a>")
            && !preview.contains("data-wikijump-compat-listpages-feed")
            && !preview.contains("data-wikijump-authored-compat-listpages-feed"),
        "generated feed HTML must retain its trusted runtime owner through include expansion:\n{preview}",
    );
}

#[tokio::test]
async fn zero_row_listpages_still_renders_its_wikidot_rss_link() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Zero-row ListPages RSS link",
        concat!(
            "[[module ListPages perPage=\"1\" range=\".\" ",
            "rss=\"Most Recently Translated\"]]\n",
            "NO MATCH\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a zero-row ListPages feed should render")
    .html_output
    .body;

    assert!(
        preview.contains(r#"<div class="feedinfo">"#)
            // Wikidot emits the same path on its own site origin; the saved
            // HTML keeps it origin-relative so the link stays on the mirror.
            && preview.contains(concat!(
                r#"href="/feed/pages/category/_default/"#,
                "range/./limit/1/t/Most+Recently+Translated",
            ))
            && preview.contains(">RSS feed</a>")
            && !preview.contains("NO MATCH"),
        "zero selected rows must not suppress the independent RSS link:\n{preview}",
    );
}

#[tokio::test]
async fn listpages_row_keeps_content_across_repeated_empty_raw_boundaries() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages repeated empty raw boundaries",
        concat!(
            "[[module ListPages limit=\"1\" category=\"*\" ",
            "order=\"updated_at desc\" NOTA: o fim do module está na parte de ",
            "baixo da página. Também, se você tentar importar um component de ",
            "outra wiki dentro do module, que tudo entro do module desaparecerá.]]\n",
            "ALPHA %%updated_at|%Y/%m/%d%%",
            "[[footnote]]ALPHA NOTE[[/footnote]]\n\n",
            "@@@@\n",
            "@@@@\n",
            "BRAVO\n\n",
            "@@@@\n",
            "@@@@\n",
            "[[=]]\n",
            "[[div class=\"addendum\"]]\n",
            "CHARLIE\n",
            "[[/div]]\n",
            "[[/=]]\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("the complete legacy ListPages row should render")
    .html_output
    .body;

    for marker in ["ALPHA", "ALPHA NOTE", "BRAVO", "CHARLIE"] {
        assert!(
            preview.contains(marker),
            "the row prefix or suffix disappeared at an empty raw boundary ({marker}):\n\
             {preview}",
        );
    }
}

#[tokio::test]
async fn delayed_listpages_row_keeps_trailing_feed_as_generated_block_html() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Generated ListPages feed after delayed row",
        concat!(
            "[[module ListPages category=\"*\" name=\"component:image-block\" ",
            "separate=\"no\" rss=\"Delayed row feed\"]]\n",
            "%%created_by%%\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a generated feed after a delayed author row should render")
    .html_output
    .body;

    assert!(
        preview.contains(r#"<div class="feedinfo">"#)
            && preview.contains(">RSS feed</a>")
            && !preview.contains("&lt;div class=&quot;feedinfo&quot;")
            && !preview.contains("data-wikijump-compat-listpages-feed"),
        "the trailing feed must remain registered block HTML through delayed parsing:\n\
         {preview}",
    );
}

#[tokio::test]
async fn delayed_listpages_inline_span_row_keeps_trailing_feed_as_generated_block_html() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Generated ListPages feed after a delayed inline-span row",
        concat!(
            "[[module ListPages category=\"*\" name=\"component:image-block\" ",
            "separate=\"no\" rss=\"Delayed inline row feed\"]]\n",
            "[[span]]%%title_linked%% [[user %%created_by%%]] ",
            "%%created_at%%[[/span]]\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a generated feed after a delayed inline-span row should render")
    .html_output
    .body;

    assert!(
        preview.contains(r#"<div class="feedinfo">"#)
            && preview.contains(">RSS feed</a>")
            && !preview.contains("WIKIJUMPWIKIDOTCOMPATHTML"),
        "the trailing feed marker must not be stranded inside the inline row paragraph:\n\
         {preview}",
    );
}

#[tokio::test]
async fn delayed_listpages_table_keeps_trailing_feed_as_generated_block_html() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Generated ListPages feed after delayed table",
        concat!(
            "[[module ListPages category=\"*\" name=\"component:image-block\" ",
            "order=\"rating desc\" separate=\"no\" rss=\"Delayed table feed\" ",
            "prependLine=\"||~ Page ||~ Rating ||\"]]\n",
            "|| %%title_linked%% || %%rating%% ||\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a generated feed after a delayed table row should render")
    .html_output
    .body;

    assert!(
        preview.contains(r#"<table class="wiki-content-table">"#)
            && preview.contains(r#"<div class="feedinfo">"#)
            && preview.contains(">RSS feed</a>")
            && !preview.contains("&lt;div class=&quot;feedinfo&quot;")
            && !preview.contains("data-wikijump-compat-listpages-feed"),
        "the trailing feed must cross the delayed table parse as trusted block HTML:\n\
         {preview}",
    );
}

#[tokio::test]
async fn listpages_rating_vote_scalars_resolve_authored_expr_envelopes() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages runtime scalar expression",
        concat!(
            "[[module ListPages category=\"*\" name=\"component:image-block\" ",
            "separate=\"no\"]]\n",
            "%%title_linked%% ",
            "{{[%%rating%%](+[[#expr (%%rating_votes%%+%%rating%%)/2]]/",
            "-[[#expr (%%rating_votes%%-%%rating%%)/2]])}} ",
            "comments=%%comments%% ",
            "rating-branch=[[#ifexpr %%rating%% < 20 | LOW | HIGH]] ",
            "size-color=##[[#ifexpr %%size%% > 0 | green | red]]|%%size%%## ",
            "percent=[[#expr (%%rating_votes%%+%%rating%%)/2/",
            "%%rating_votes%%*10000%10001/100]]\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a ListPages expression over rating scalars should render")
    .html_output
    .body;

    assert!(
        preview.contains("<tt>[0](+0/-0)</tt>")
            && preview.contains("comments=0")
            && preview.contains("rating-branch=LOW")
            && preview.contains(r#"size-color=<span style="color: green">"#)
            && preview.contains("percent=run-time error: division by zero")
            && !preview.contains("[[#expr")
            && !preview.contains("[[#ifexpr"),
        "FTML parser functions should resolve after runtime scalar substitution:\n\
         {preview}",
    );
}

#[tokio::test]
async fn listpages_created_by_id_comment_gates_resolve_after_row_substitution() {
    const TARGET_SLUG: &str = "listpages-created-by-id-comment-gate-target";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site.site.site_id),
        page_reference: Some(Reference::Slug(TARGET_SLUG.into())),
    });
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site.site_id,
            "wikitext": "Comment gate target",
            "title": "ListPages created-by-id comment gate target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "create deterministic comment gate target",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site.site.site_id,
        "ListPages created-by-id comment gates",
        concat!(
            "[[module ListPages name=\"listpages-created-by-id-comment-gate-target\"]]\n",
            "[[#ifexpr %%created_by_id%% < 1486450 |  | [!-- ]]\n",
            "LOW\n",
            "[!-- --]\n",
            "[[#ifexpr %%created_by_id%% > 6000000 |  | [!-- ]]\n",
            "HIGH\n",
            "[!-- --]\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("created-by-id conditional ListPages preview should render")
    .html_output
    .body;

    assert!(
        preview.contains("LOW") || preview.contains("HIGH"),
        "one selected branch should remain visible after row substitution:\n{preview}",
    );
    assert!(
        !preview.contains("%%created_by_id%%")
            && !preview.contains("[[#ifexpr")
            && !preview.contains("[!--")
            && !preview.contains("[!—"),
        "generated comment gates and row variables must not leak into preview HTML:\n{preview}",
    );

    runner.teardown().await;
}

#[tokio::test]
async fn listpages_empty_runtime_monospace_shells_collapse_authored_spaces() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages empty runtime monospace shells",
        concat!(
            "[[module ListPages category=\"*\" name=\"component:image-block\" ",
            "separate=\"no\"]]\n",
            "**%%title_linked%%**\n",
            "##grey|**{{[+%%rating%%](+[[#expr ",
            "(%%rating_votes%%+%%rating%%)/2]]/-[[#expr ",
            "(%%rating_votes%%-%%rating%%)/2]])}}** **{{%%comments%%}}**",
            "[[size 0.9em]]A {{**%%commented_by%%**}} ",
            "{{**%%commented_at|%y-%m-%d %H:%M%%**}} B[[/size]]##\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("empty ListPages runtime monospace shells should render")
    .html_output
    .body;

    assert!(
        preview.contains(r#"<span style="font-size:0.9em;">A B</span>"#)
            && !preview.contains("<tt></tt>")
            && !preview.contains("A   B"),
        "suppressed runtime shells must not preserve one space per removed owner:\n\
         {preview}",
    );
}

#[tokio::test]
async fn generated_include_keeps_html_only_row_fragments_pending() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Generated ListPages include with an HTML-only row",
        concat!(
            "[[module ListPages category=\"*\" name=\"component:image-block\" ",
            "separate=\"no\" prependLine=\"[[include ",
            "component:definitely-missing-listpages-include]]\"]]\n",
            "%%title%% %%created_at%%\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a generated date should survive include expansion without a typed link")
    .html_output
    .body;

    assert!(
        preview.contains("Standard Image Block")
            && preview.contains(r#"<span class="odate time_"#)
            && !preview.contains("WIKIJUMPWIKIDOTCOMPATHTML"),
        "HTML-only delayed row fragments must survive generated include expansion:\n{preview}",
    );
}

#[tokio::test]
async fn dynamic_listpages_embed_opener_keeps_a_safe_name_only_iframe() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages dynamic embed opener",
        concat!(
            "[[module ListPages category=\"*\" ",
            "name=\"component:image-block\" separate=\"no\"]]\n",
            "[[%%content{0}%%embed]]\n",
            "<iframe name=\"isJPExist\"></iframe>\n",
            "[[/embed]]\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a dynamically opened ListPages embed should render")
    .html_output
    .body;

    assert!(
        preview.contains(r#"<p><iframe name="isJPExist"></iframe></p>"#)
            && !preview.contains("[[embed]]")
            && !preview.contains("[[/embed]]")
            && !preview.contains("&lt;iframe"),
        "the exact no-src iframe must cross the existing embed allowlist boundary:\n\
         {preview}",
    );
}

#[tokio::test]
async fn row_bearing_generated_include_keeps_the_pager_block_boundary() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Generated ListPages include with pager",
        concat!(
            "[[module ListPages category=\"*\" limit=\"2\" perPage=\"1\" ",
            "separate=\"no\" prependLine=\"[[include ",
            "component:definitely-missing-listpages-include]]\" ",
            "appendLine=\"AFTER-INCLUDE\"]]\n",
            "%%title_linked%%\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a generated pager after a typed ListPages row should render")
    .html_output
    .body;

    assert!(
        preview.contains(r#"<div class="pager">"#)
            && preview.contains("AFTER-INCLUDE</p>")
            && !preview.contains("WIKIJUMPWIKIDOTCOMPATHTML"),
        "the pager must remain a trusted sibling block after row syntax:\n{preview}",
    );
}

#[tokio::test]
async fn unresolved_data_form_selectors_keep_zero_row_once_only_lines() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unresolved ListPages form selector",
        concat!(
            "[[module ListPages category=\"*\" created_by=\"{$nombre}\" ",
            "separate=\"no\" prependLine=\"||~ Article ||~ Created ||\"]]\n",
            "|| %%title_linked%% || %%created_at%% ||\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("an unresolved form selector should still render once-only lines")
    .html_output
    .body;

    assert!(
        preview.contains(concat!(
            "<div class=\"list-pages-box\">",
            "<table class=\"wiki-content-table\">\n",
            "<tr>\n<th>Article</th>\n<th>Created</th>\n</tr>",
            "\n</table>",
        )) && preview.trim_end().ends_with("</div>")
            && preview.matches(r#"<div class="list-pages-box">"#).count() == 1
            && preview.matches("<table").count() == 1
            && preview.matches("</table>").count() == 1
            && preview.matches("</div>").count() == 1,
        "live Wikidot emits prependLine even when the unresolved selector yields no rows:\n{preview}",
    );
    assert!(
        !preview.contains("%%title_linked%%") && !preview.contains("{$nombre}"),
        "the zero-row module must not leak its template or selector:\n{preview}",
    );
}

#[tokio::test]
async fn listpages_generated_footnotes_remain_inside_the_runtime_wrapper() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages generated footnote boundary",
        concat!(
            "[[module ListPages name=\"definitely-missing-listpages-footnote\" ",
            "separate=\"no\" prependLine=\"HEAD[[footnote]]NOTE[[/footnote]]\"]]\n",
            "%%title%%\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a generated ListPages footnote should render")
    .html_output
    .body;

    let wrapper_start = preview
        .find(r#"<div class="list-pages-box">"#)
        .expect("the default runtime wrapper should be present");
    let footnotes_start = preview
        .find(r#"<div class="footnotes-footer">"#)
        .expect("the generated footnote footer should be present");
    let first_wrapper_close = preview[wrapper_start..]
        .find("</div>")
        .map(|offset| wrapper_start + offset)
        .expect("the runtime wrapper should close");
    assert!(
        wrapper_start < footnotes_start && footnotes_start < first_wrapper_close,
        "the generated footer must remain inside the ListPages wrapper:\n{preview}",
    );
}

#[tokio::test]
async fn listpages_default_row_keeps_its_footnote_footer_inside_the_item() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let target_slug = format!("listpages-footnote-item-{}", Uuid::new_v4().as_simple(),);

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(target_slug.clone().into())),
    });
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "ROW[[footnote]]NOTE[[/footnote]]",
            "title": "ListPages footnote item boundary",
            "alt_title": null,
            "slug": target_slug.clone(),
            "layout": "wikidot",
            "revision_comments": "ListPages footnote item boundary regression",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages footnote item boundary",
        format!(r#"[[module ListPages name="{target_slug}"]]"#),
    )
    .await
    .expect("a selected page footnote should render")
    .html_output
    .body;

    assert!(
        preview.contains(concat!("</sup></p>", r#"<div class="footnotes-footer">"#,))
            && !preview.contains(concat!(
                "</sup></p></div>",
                r#"<div class="footnotes-footer">"#,
            )),
        "the selected row's footnote footer must remain inside list-pages-item:\n{preview}",
    );
}

#[tokio::test]
async fn linked_listpages_slot_ranges_do_not_bind_marker_shaped_metadata() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let target_slug = format!(
        "listpages-linked-slot-collision-{}",
        Uuid::new_v4().as_simple(),
    );

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(target_slug.clone().into())),
    });
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "Marker-shaped metadata collision target",
            "title": "%%title_linked%%",
            "alt_title": null,
            "slug": target_slug.clone(),
            "layout": "wikidot",
            "revision_comments": "typed ListPages slot collision regression",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Typed ListPages slot collision",
        format!(
            concat!(
                "[[module ListPages name=\"{target_slug}\" ",
                "separate=\"no\" wrapper=\"no\"]]\n",
                "%%title%%|%%title_linked%%|%%title%%\n",
                "[[/module]]",
            ),
            target_slug = target_slug,
        ),
    )
    .await
    .expect("marker-shaped metadata should render")
    .html_output
    .body;

    assert!(
        preview.contains(&format!(
            "%%title_linked%%|<a href=\"/{target_slug}\">%%title_linked%%</a>|%%title_linked%%",
        )),
        "only the out-of-band generated slot range may acquire link authority:\n{preview}",
    );
}

#[tokio::test]
async fn random_listpages_reuses_the_same_idle_cached_order_for_one_invocation() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: None,
    });

    let module_body =
        format!("LANE5-RANDOM-{} %%fullname%%", Uuid::new_v4().as_simple(),);
    let request = || {
        json!({
            "site_id": site_id,
            "module_body": module_body.clone(),
            "parameters": {
                "category": "_default",
                "order": "random",
                "limit": "10",
                "perPage": "10",
                "separate": "no",
                "wrapper": "no"
            }
        })
    };
    let first = run_endpoint!(runner, wikidot_list_pages_module, request());
    let second = run_endpoint!(runner, wikidot_list_pages_module, request());

    assert_eq!(
        second.body, first.body,
        "live Wikidot renews a one-minute idle cache for the complete random ListPages invocation",
    );
}

#[tokio::test]
async fn wikidot_ajax_listpages_p_parameter_selects_the_rendered_page() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: None,
    });

    let output = run_endpoint!(
        runner,
        wikidot_list_pages_module,
        json!({
            "site_id": site_id,
            "module_body": "LANE5-PAGER %%fullname%%",
            "parameters": {
                "category": "_default",
                "order": "name",
                "limit": "6",
                "perPage": "2",
                "p": "2",
                "separate": "no",
                "wrapper": "no"
            }
        }),
    );

    assert!(
        output.body.contains("page 2 of 3")
            && output.body.contains(r#"<span class="current">2</span>"#),
        "Wikidot AMC's p parameter must route through the ordinary ListPages pager: {}",
        output.body,
    );
    assert!(
        output
            .body
            .contains(r#"<div class="pager"><span class="pager-no">"#)
            && output
                .body
                .contains(r#"href="/ajax-module-connector.php/p/1""#),
        "AMC pagers must have direct span children and remain on the Ajax route: {}",
        output.body,
    );
    assert!(!output.body.contains(r#"<div class="pager"><p>"#));
}

#[tokio::test]
async fn listpages_first_image_token_disappears_when_the_selected_page_has_no_image() {
    const TARGET_SLUG: &str = "listpages-first-image-empty-target";

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
            "wikitext": "Target without attachments",
            "title": "ListPages first-image empty target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "ListPages :first image regression fixture",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages first image",
        format!(
            concat!(
                "[[module ListPages name=\"{}\" separate=\"yes\"]]\n",
                "[[image :first]]\n",
                "ROW|%%fullname%%\n",
                "[[/module]]",
            ),
            TARGET_SLUG,
        ),
    )
    .await
    .expect("ListPages :first image source should render")
    .html_output
    .body;

    assert!(
        preview.contains(&format!("ROW|{TARGET_SLUG}")),
        "the selected row should still render:\n{preview}",
    );
    assert!(
        !preview.contains(":first")
            && !preview.contains("local--files//")
            && !preview.contains("local--resized-images//"),
        "Wikidot drops :first when the selected page has no attached image:\n{preview}",
    );
}

#[tokio::test]
async fn listpages_first_image_token_uses_the_selected_pages_first_image_attachment() {
    const TARGET_SLUG: &str = "listpages-first-image-attached-target";

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
            "wikitext": "Target with attachments",
            "title": "ListPages first-image attached target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "ListPages :first image attachment fixture",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    create_listpages_file_fixture(
        &runner,
        site_id,
        target.page_id,
        "notes.txt",
        "text/plain",
    )
    .await;
    create_listpages_file_fixture(
        &runner,
        site_id,
        target.page_id,
        "first image.png",
        "image/png",
    )
    .await;

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages first image",
        format!(
            concat!(
                "[[module ListPages name=\"{}\" separate=\"yes\"]]\n",
                "[[image :first alt=\"Selected preview\"]]\n",
                "ROW|%%fullname%%\n",
                "[[/module]]",
            ),
            TARGET_SLUG,
        ),
    )
    .await
    .expect("ListPages :first image source should render")
    .html_output
    .body;

    assert!(
        preview.contains(&format!("/local--files/{TARGET_SLUG}/first%20image.png"))
            && preview.contains(r#"alt="Selected preview""#),
        "the image token should bind to the first image attachment, not the first non-image file:\n{preview}",
    );
    assert!(
        !preview.contains("notes.txt") && !preview.contains(":first"),
        "the selected row must not leak the non-image attachment or pseudo target:\n{preview}",
    );
}

#[tokio::test]
async fn listpages_legacy_comparisons_and_unresolved_url_selectors_execute_in_preview() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (source, marker) in [
        (
            concat!(
                "[[module ListPages created_at<\"1900.1.1\" updated_at>=\"1900.1.1\" ",
                "separate=\"no\" prependLine=\"LEGACY-COMPARISON-EXECUTED\"]]\n",
                "ROW %%fullname%%\n",
                "[[/module]]",
            ),
            "LEGACY-COMPARISON-EXECUTED",
        ),
        (
            concat!(
                "[[module ListPages created_by=\"@URL\" parent=\"@URL\" tags=\"@URL\" ",
                "pagetype=\"@URL\" rating=\"@URL\" votes=\"@URL\" ",
                "created_at=\"@URL\" updated_at=\"@URL\" link_to=\"@URL\" ",
                "name=\"@URL\" order=\"@URL\" reverse=\"@URL\" limit=\"1\" ",
                "separate=\"no\" prependLine=\"URL-SELECTORS-EXECUTED\"]]\n",
                "ROW %%fullname%%\n",
                "[[/module]]",
            ),
            "URL-SELECTORS-EXECUTED",
        ),
        (
            concat!(
                "[[module ListPages name=\"*\" category=\"-nav -system -forum -admin\" ",
                "tags=\"-管理 -\" order=\"created_at desc desc\" limit=\"1\" ",
                "offset=\"@URL|0\" separate=\"no\" ",
                "prependLine=\"DUPLICATE-ORDER-DIRECTION-EXECUTED\"]]\n",
                "ROW %%fullname%%\n",
                "[[/module]]",
            ),
            "DUPLICATE-ORDER-DIRECTION-EXECUTED",
        ),
        (
            concat!(
                "[[module ListPages separate=\"no\" tags=\"@URL\" created_at=\"@URL\" ",
                "updated_at=\"@URL\" created_by=\"@URL\" rating=\"@URL\" ",
                "offset=\"@URL|0\" perPage=\"1\"　limit=\"1\" ",
                "order=\"@URL|created_at desc\" category=\"*\" ",
                "prependLine=\"FULL-WIDTH-SPACE-EXECUTED\"]]\n",
                "ROW %%fullname%%\n",
                "[[/module]]",
            ),
            "FULL-WIDTH-SPACE-EXECUTED",
        ),
        (
            concat!(
                "[[module ListPages order=\"random\" perPage=\"250\" limit=\"250\" ",
                "separate=\"no\" prependLine=\"BOUNDED-RANDOM-HEAD-EXECUTED\"]]\n",
                "ROW %%fullname%%\n",
                "[[/module]]",
            ),
            "BOUNDED-RANDOM-HEAD-EXECUTED",
        ),
    ] {
        let preview = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "ListPages legacy selector preview",
            source.to_owned(),
        )
        .await
        .expect("the live-compatible ListPages selector should render")
        .html_output
        .body;
        assert!(preview.contains(marker), "{source:?}:\n{preview}");
        assert!(
            !preview.contains("[[module ListPages")
                && !preview.contains("TODO: module ListPages"),
            "the selector must execute rather than leak its authored module: {source:?}:\n{preview}",
        );
    }

    for bare_head in ["属性...", "任意属性...", "???"] {
        let preview = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "ListPages inert bare head preview",
            format!("[[module ListPages {bare_head}]]\n模块主体\n[[/module]]"),
        )
        .await
        .expect("a live-evidenced inert ListPages head should render")
        .html_output
        .body;
        assert!(
            preview.contains(r#"<div class="list-pages-box">"#)
                && !preview.contains("[[module ListPages")
                && preview.contains("模块主体"),
            "an inert bare head executes Wikidot's default ListPages query and custom row template ({bare_head:?}):\n{preview}",
        );
    }

    for prose_tail in [
        "NOTE: arbitrary words can move freely.",
        "NOTA: conteúdo muda livremente.",
        "NOTE: arbitrary [[span]] words.",
        "NOTE: arbitrary \"quoted words\".",
    ] {
        let preview = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "ListPages inert prose tail preview",
            format!(
                concat!(
                    "[[module ListPages category=\"*\" limit=\"1\" ",
                    "order=\"name\" {}]]\n",
                    "INERT-PROSE-CUSTOM-ROW|%%fullname%%\n",
                    "[[/module]]",
                ),
                prose_tail,
            ),
        )
        .await
        .expect("a live-evidenced inert ListPages prose tail should render")
        .html_output
        .body;
        assert!(
            preview.contains("INERT-PROSE-CUSTOM-ROW")
                && !preview.contains("[[module ListPages"),
            "the supported assignments and custom row must execute despite an inert prose tail ({prose_tail:?}):\n{preview}",
        );
    }

    for malformed_tail in [
        r#"mystery="alpha]]"#,
        r#"alternate="bravo-42]]"#,
        r#"fullname="main:about]]"#,
        r#"mystery="alpha   ]]"#,
        r#"mystery="alpha]"#,
        r#"mystery="alpha] ]"#,
    ] {
        let preview = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "ListPages final unclosed quote preview",
            format!(
                concat!(
                    "[[module ListPages category=\"*\" limit=\"1\" order=\"name\" ",
                    "{}\n",
                    "UNTERMINATED-HEAD-CUSTOM-ROW|%%fullname%%\n",
                    "[[/module]]",
                ),
                malformed_tail,
            ),
        )
        .await
        .expect("a live-evidenced final unclosed quote should render")
        .html_output
        .body;
        assert!(
            preview.contains(r#"<div class="list-pages-box">"#)
                && !preview.contains("UNTERMINATED-HEAD-CUSTOM-ROW")
                && !preview.contains("[[module ListPages"),
            "the final unclosed quote must consume the authored row and use Wikidot's default template ({malformed_tail:?}):\n{preview}",
        );
    }

    let missing_parent = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages parent fallback preview",
        concat!(
            "[[module ListPages parent=\"@URL|Definitely-Missing-ListPages-Parent\" ",
            "range=\"others\"]]\n",
            "ROW %%fullname%%\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("a missing parent fallback should render Wikidot's error")
    .html_output
    .body;
    assert!(
        missing_parent.contains(concat!(
            r#"<div class="error-block">Parent page "#,
            r#"definitely-missing-listpages-parent does not exist</div>"#,
        ),),
        "the normalized missing-parent error must precede invalid range validation:\n{missing_parent}",
    );
    assert!(!missing_parent.contains("Invalid range argument."));

    let inert_at_marker_module = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages inert at-marker module preview",
        concat!(
            "[[module ListPages limit=\"@URL|0\" range=\".\" ",
            "urlAttrPrefix=\"page2\"]]\n",
            "|content=[[module rate]]@@@@@@@@\n",
            "LISTPAGES-INERT-MODULE-BODY\n",
            "[[/module]]\n",
            "LISTPAGES-INERT-MODULE-AFTER",
        )
        .to_owned(),
    )
    .await
    .expect("an at-marker-suffixed module token should not steal the ListPages close")
    .html_output
    .body;
    assert!(
        inert_at_marker_module.contains("LISTPAGES-INERT-MODULE-AFTER")
            && !inert_at_marker_module.contains("LISTPAGES-INERT-MODULE-BODY")
            && !inert_at_marker_module.contains("[[module rate]]"),
        "the exact live-owned ListPages body must remain atomic:\n{inert_at_marker_module}",
    );
}

#[tokio::test]
async fn listpages_empty_assignment_keeps_later_arguments_and_executes_in_preview() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site.site.site_id,
        "ListPages empty assignment preview",
        concat!(
            "[[module ListPages category=\"*\" limit=\"1\" order=\"name\" prependLine=]]\n",
            "EMPTY-ASSIGNMENT-ROW|%%fullname%%\n",
            "[[/module]]",
        )
        .to_owned(),
    )
    .await
    .expect("the live-evidenced empty assignment should execute")
    .html_output
    .body;

    assert!(
        preview.contains(r#"<div class="list-pages-box">"#),
        "the ListPages wrapper should render:\n{preview}",
    );
    assert_eq!(
        preview.matches("EMPTY-ASSIGNMENT-ROW|").count(),
        1,
        "recognized limit and the custom row should still apply:\n{preview}",
    );
    assert!(
        !preview.contains("[[module ListPages")
            && !preview.contains("TODO: module ListPages")
            && !preview.contains("prependLine="),
        "the inert assignment must not leak into rendered output:\n{preview}",
    );
}

#[tokio::test]
async fn listpages_date_html_keeps_surrounding_inline_wikidot_markup() {
    const TARGET_SLUG: &str = "listpages-inline-date-markup-target";

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
            "wikitext": "ListPages inline date target",
            "title": "ListPages inline date target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "ListPages inline date markup regression",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages inline date markup",
        format!(
            concat!(
                "[[module ListPages name=\"{}\" limit=\"1\"]]\n",
                "++++ **##red|__Update:%%created_at|%Y/%m/%d%%__##**\n",
                "[[/module]]",
            ),
            TARGET_SLUG,
        ),
    )
    .await
    .expect("a date variable inside nested inline markup should render")
    .html_output
    .body;

    assert!(
        preview.contains(
            r#"<span style="text-decoration: underline;">Update:<span class="odate "#,
        ),
        "the typed date fragment must not sever its surrounding underline:\n{preview}",
    );
}

#[tokio::test]
async fn listpages_resolves_crossing_link_conditionals_after_row_substitution() {
    const TARGET_SLUG: &str = "listpages-crossing-link-conditional-target";

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
            "wikitext": "Crossing conditional content",
            "title": "ListPages crossing link conditional target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "ListPages crossing link conditional regression",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let source = format!(
        concat!(
            "[[module ListPages name=\"{target_slug}\" limit=\"1\" perPage=\"1\"]]\n",
            "[[#ifexpr %%rating_votes%%-%%rating%%>0 | ",
            "[[a class=\"point\" href=\"/%%name%%\" | %%content{{0}}%% ]]",
            "[[#ifexpr %%rating_votes%%-%%rating%%>0 | ] | %%content{{0}}%% ]]",
            "[[#ifexpr %%rating_votes%%-%%rating%%>0 | ] | %%content{{0}}%% ]]",
            "[[#ifexpr %%rating_votes%%-%%rating%%>0 | ",
            "[[/a | %%content{{0}}%% ]]",
            "[[#ifexpr %%rating_votes%%-%%rating%%>0 | ] | %%content{{0}}%% ]]",
            "[[#ifexpr %%rating_votes%%-%%rating%%>0 | ] | %%content{{0}}%% ]]\n",
            "[[/module]]",
        ),
        target_slug = TARGET_SLUG,
    );
    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages crossing conditional preview",
        source,
    )
    .await
    .expect("the crossing conditional ListPages row should render")
    .html_output
    .body;

    assert!(
        !preview.contains("[[#ifexpr")
            && !preview.contains("[[a")
            && !preview.contains("[[/a")
            && !preview.contains("Crossing conditional content"),
        "the false post-substitution branches must discard their crossing links and row content:\n{preview}",
    );
    assert_eq!(
        preview.matches(r#"<div class="list-pages-item">"#).count(),
        1,
        "the selected row should remain structurally present:\n{preview}",
    );
}

#[tokio::test]
async fn listpages_module_heads_accept_live_legacy_boundaries() {
    const TARGET_SLUG: &str = "listpages-head-boundary-target";

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
            "wikitext": "ListPages head-boundary target",
            "title": "ListPages Head Boundary Target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "ListPages head-boundary fixture",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let ordinary_head = format!(r#"name="{TARGET_SLUG}" limit="1" order="name""#);
    let sources = [
        format!("[[module ListPages {ordinary_head}]]]\nROW|%%fullname%%\n[[/module]]"),
        format!("[[module ListPages {ordinary_head}]]]]\nROW|%%fullname%%\n[[/module]]"),
        format!(
            "[[module ListPages\nname=\"{TARGET_SLUG}\"\nlimit=\"1\"\norder=\"name\"\n]]\nROW|%%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" limit=\"1\" order=\"name\n\"]]\nROW|%%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages | name=\"{TARGET_SLUG}\" limit=\"1\" order=\"name\"]]\nROW|%%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages size name=\"{TARGET_SLUG}\" limit=\"1\" order=\"name\"]]\nROW|%%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" limit=\"1\" order=\"name\" prependLine=]]\nROW|%%fullname%%\n[[/module]]",
        ),
        format!(
            "[[module ListPages name=\"{TARGET_SLUG}\" limit=\"1\" order=\"name\"@@]]\nROW|%%fullname%%\n[[/module]]",
        ),
    ];

    for source in sources {
        let preview = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "ListPages head boundary",
            source.clone(),
        )
        .await
        .expect("a live-compatible ListPages head boundary should render")
        .html_output
        .body;
        assert!(
            preview.contains(&format!("ROW|{TARGET_SLUG}")),
            "the ListPages opening should execute and select its exact row for {source:?}:\n{preview}",
        );
        assert!(
            !preview.contains("[[module ListPages")
                && !preview.contains("%%fullname%%")
                && !preview.contains("<p>]</p>"),
            "the recovered head must not leak source or surplus right brackets for {source:?}:\n{preview}",
        );
    }

    // Anonymous PagePreview boundary probe (2026-08-01): a final unmatched
    // double quote is executable, but Wikidot consumes the authored row and
    // renders its default template rather than treating that row as custom
    // ListPages body content. Keep this malformed-head behavior distinct from
    // the complete and surplus-bracket cases above.
    let dangling_quote_source = format!(
        "[[module ListPages name=\"{TARGET_SLUG}\" limit=\"1\" order=\"评分：]\nROW|%%fullname%%\n[[/module]]",
    );
    let dangling_quote_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages dangling quote boundary",
        dangling_quote_source,
    )
    .await
    .expect("the live-compatible dangling quote boundary should render")
    .html_output
    .body;
    assert!(
        dangling_quote_preview.contains(r#"<div class="list-pages-box">"#)
            && !dangling_quote_preview.contains(&format!("ROW|{TARGET_SLUG}"))
            && !dangling_quote_preview.contains("[[module ListPages")
            && !dangling_quote_preview.contains("%%fullname%%")
            && !dangling_quote_preview.contains("[[/module]]"),
        "an unmatched final quote must consume the authored row and close cleanly with Wikidot's default template:\n{dangling_quote_preview}",
    );

    let argumentless_eof = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Argumentless ListPages opener",
        "[[Module Listpages]]".to_owned(),
    )
    .await
    .expect("the live-compatible argumentless opener should execute at EOF")
    .html_output
    .body;
    assert!(
        argumentless_eof.contains(r#"<div class="list-pages-box">"#)
            && !argumentless_eof.contains("[[Module Listpages]]")
            && !argumentless_eof.contains("TODO: module ListPages"),
        "the complete argumentless opener must execute instead of remaining literal:\n{argumentless_eof}",
    );

    for (source, expected_heading) in [
        (
            concat!(
                "[[module ListPages created_by=\"Dr Nordsee\" category=\"*\" ",
                "pagetype=\"*\" separate=\"no\" tags=\"-admin -_sys +übersetzt\" ",
                "order=prependLine=\"||~ Übersetzung ||~ Veröffentlichungsdatum ||~ Momentane Bewertung ||\"]]]\n",
                "|| %%title_linked%% || %%created_at%% || %%rating%% ||\n",
                "[[/module]]",
            ),
            "Veröffentlichungsdatum",
        ),
        (
            concat!(
                "[[module ListPages created_by=created_by=\"s d locke\" ",
                "order=\"rating desc\" separate=\"no\" tags=\"tale\" perPage=\"250\" ",
                "prependLine=\"||~ Title ||~ Rating ||~ Comments ||~ Created ||~ Edited ||\"]]\n",
                "|| %%title_linked%% || %%rating%% || %%comments%% || %%created_at%% || %%updated_at%% ||\n",
                "[[/module]]",
            ),
            "Comments",
        ),
        (
            concat!(
                "[[module ListPages rating=\">60\" order=\"rating desc\" category=\"*\" ",
                "separate=\"false\" limit=\"200\" perPage=\"35\" date=\"@URL|2023\"",
                "[!-- UPDATE THIS TO CURRENT YEAR --] ",
                "prependLine=\"||~ タイトル||~ 評価||~ 著者||~ 作成日||\"]]\n",
                "|| %%title_linked%% || %%rating%% || %%created_by_linked%% || %%created_at%% ||\n",
                "[[/module]]",
            ),
            "作成日",
        ),
    ] {
        let preview = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "ListPages permissive legacy head",
            source.to_owned(),
        )
        .await
        .expect("an evidenced permissive ListPages head should render")
        .html_output
        .body;
        assert!(
            preview.contains(expected_heading)
                && !preview.contains("[[module ListPages")
                && !preview.contains("TODO: module ListPages"),
            "the permissive legacy head must recover its later prependLine argument for {source:?}:\n{preview}",
        );
    }

    let unclosed_head_before_raw_closer = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages unclosed legacy head",
        format!(
            concat!(
                "[[module ListPages separate=\"no\" limit=\"250\" perPage=\"250\" ",
                "tags=\"group-of-interest-form, beyond +_entropy-, -scp, -story,\" ",
                "order=\"title\"\n\n",
                "[[/module]]\nAFTER_MALFORMED\n",
                "[[module ListPages name=\"{}\"]]",
                "SECOND|%%fullname%%[[/module]]",
            ),
            TARGET_SLUG,
        ),
    )
    .await
    .expect("a live-compatible unclosed ListPages head should render")
    .html_output
    .body;
    assert!(
        !unclosed_head_before_raw_closer.contains("[[module ListPages")
            && !unclosed_head_before_raw_closer.contains("[[/module]]")
            && unclosed_head_before_raw_closer.contains("AFTER_MALFORMED")
            && unclosed_head_before_raw_closer.contains(&format!("SECOND|{TARGET_SLUG}")),
        "an unclosed head ending before a raw closer must consume only its own module and leave the following module independent:\n{unclosed_head_before_raw_closer}",
    );

    // Frozen anonymous PagePreview case `listpages-unterminated-multiline-
    // before-later-module`: the later quote inside apparent module syntax
    // completes the outer name value, so the apparent inner module is not
    // independently rendered and its authored row stays consumed by the
    // outer zero-result query.
    let later_quote_source = concat!(
        "FIRST\n",
        "[[module ListPages name=\"unterminated\n",
        "[[module ListPages category=\"*\" limit=\"1\" order=\"name\"]]\n",
        "SECOND|%%fullname%%\n",
        "[[/module]]\n",
        "LAST",
    );
    let later_quote_preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "ListPages multiline head completed by later quote",
        later_quote_source.to_owned(),
    )
    .await
    .expect("the frozen later-quote ListPages head should render")
    .html_output
    .body;
    assert_eq!(
        later_quote_preview
            .matches(r#"<div class="list-pages-box">"#)
            .count(),
        1,
        "the recovered outer opener must render exactly once:\n{later_quote_preview}",
    );
    assert!(
        later_quote_preview.contains("FIRST")
            && later_quote_preview.contains("LAST")
            && !later_quote_preview.contains("SECOND|")
            && !later_quote_preview.contains(r#"<div class="list-pages-item">"#)
            && !later_quote_preview.contains("[[module ListPages")
            && !later_quote_preview.contains("%%fullname%%"),
        "the apparent inner opener and its row must remain owned by the outer head:\n{later_quote_preview}",
    );
}

#[tokio::test]
async fn empty_listpages_after_inline_content_keeps_its_block_boundary() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let source = concat!(
        "[[module ListPages name=\"definitely-missing-listpages-page\"]]\n",
        "* %%title_linked%%\n",
        "[[/module]]\n\n",
        "**Authored heading:**\n",
        "[[module ListPages name=\"also-definitely-missing-listpages-page\"]]\n",
        "* %%title_linked%%\n",
        "[[/module]]",
    );

    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Empty ListPages block boundary",
        source.to_owned(),
    )
    .await
    .expect("two zero-row ListPages modules should render")
    .html_output
    .body;

    assert_eq!(
        preview.matches(r#"<div class="list-pages-box">"#).count(),
        2,
        "each zero-row module must restore its own trusted block wrapper:\n{preview}",
    );
    assert!(
        !preview.contains("WIKIJUMPWIKIDOTCOMPATHTML")
            && !preview.contains(r#"<p><div class="list-pages-box">"#),
        "a later ListPages block must not remain inside the authored paragraph:\n{preview}",
    );
}

#[tokio::test]
async fn listpages_respects_corpus_literal_context_ownership() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let monospace = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Literal ListPages monospace",
        "{{[[Module Listpages]]}}".to_owned(),
    )
    .await
    .expect("an inline-monospace ListPages example should render as literal text")
    .html_output
    .body;
    assert!(
        monospace.contains("[[Module Listpages]]")
            && !monospace.contains("list-pages-box")
            && !monospace.contains("TODO: module ListPages"),
        "inline monospace must own the module-shaped text:\n{monospace}",
    );

    let preview_html_block = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "HTML-owned ListPages",
        concat!(
            "[[html]]\n",
            "[[module ListPages category=\"_default\" limit=\"2\"]]\n",
            "%%name%%\n",
            "[[/module]]\n",
            "[[/html]]",
        )
        .to_owned(),
    )
    .await
    .expect("an HTML-owned ListPages example should render as preview text")
    .html_output
    .body;
    assert!(
        preview_html_block.contains("[[html]]")
            && preview_html_block.contains("[[module ListPages")
            && !preview_html_block.contains("list-pages-box")
            && !preview_html_block.contains("TODO: module ListPages"),
        "page preview keeps HTML blocks literal, including nested ListPages syntax:\n{preview_html_block}",
    );

    for (opener, closer, visible_after_close) in [
        ("[!--", "--]", "COMMENT CLOSED AFTER TWO HYPHENS"),
        ("[!----", "---]", "COMMENT CLOSED AFTER THREE HYPHENS"),
        ("[!--", "----]", "COMMENT CLOSED AFTER FOUR HYPHENS"),
        ("[!--", "-----]", "COMMENT CLOSED AFTER FIVE HYPHENS"),
        ("[!--", "------]", "COMMENT CLOSED AFTER SIX HYPHENS"),
        ("[!--", "-------]", "COMMENT CLOSED AFTER SEVEN HYPHENS"),
        ("[!--", "--------]", "COMMENT CLOSED AFTER EIGHT HYPHENS"),
    ] {
        let comment = format!(
            "{opener}\n\
             temporary hidden region\n\
             [[module ListPages order=\"updated_at\" category=\"*\" \
             perPage=\"200\" separate=\"false\"]]\n\
             %%title_linked%%\n\
             [[/module]]\n\
             {closer}\n\
             {visible_after_close}",
        );
        let hidden = RenderService::render_wikidot_page_preview(
            runner.context(),
            site_id,
            "Comment-owned ListPages",
            comment,
        )
        .await
        .expect("a comment-owned ListPages example should remain hidden")
        .html_output
        .body;
        assert!(
            hidden.contains(visible_after_close)
                && !hidden.contains("temporary hidden region")
                && !hidden.contains("list-pages-box")
                && !hidden.contains("TODO: module ListPages")
                && !hidden.contains("[[module ListPages"),
            "the complete Wikidot comment must own the module-shaped text and close before trailing content:\n{hidden}",
        );
    }

    let one_hyphen_non_closer = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Single-hyphen comment non-closer",
        concat!(
            "[!--\n",
            "temporary hidden region\n",
            "-]\n",
            "ONE HYPHEN DOES NOT CLOSE\n",
            "--]\n",
            "CANONICAL CLOSER ENDS COMMENT",
        )
        .to_owned(),
    )
    .await
    .expect("a single-hyphen pseudo-closer should remain inside the comment")
    .html_output
    .body;
    assert!(
        one_hyphen_non_closer.contains("CANONICAL CLOSER ENDS COMMENT")
            && !one_hyphen_non_closer.contains("temporary hidden region")
            && !one_hyphen_non_closer.contains("ONE HYPHEN DOES NOT CLOSE"),
        "a single-hyphen pseudo-closer must not release comment-owned text before a valid closer:\n{one_hyphen_non_closer}",
    );

    let unclosed_comment = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unclosed comment ownership",
        concat!(
            "[!--\n",
            "temporary hidden region\n",
            "UNCLOSED COMMENT REACHES EOF",
        )
        .to_owned(),
    )
    .await
    .expect("an unclosed comment should preserve ordinary source literally")
    .html_output
    .body;
    assert!(
        unclosed_comment.contains("[!—")
            && unclosed_comment.contains("temporary hidden region")
            && unclosed_comment.contains("UNCLOSED COMMENT REACHES EOF"),
        "an unclosed comment must preserve ordinary source literally through EOF:\n{unclosed_comment}",
    );

    let unmatched_closers = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Unmatched comment closers",
        "raw ---]\nraw ----]".to_owned(),
    )
    .await
    .expect("comment-shaped closers outside a comment should remain visible")
    .html_output
    .body;
    assert!(
        unmatched_closers.contains("raw —-]") && unmatched_closers.contains("raw ——]"),
        "outside a comment, three- and four-hyphen runs retain ordinary typography and a visible bracket:\n{unmatched_closers}",
    );
}
