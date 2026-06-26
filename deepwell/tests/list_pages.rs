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
use deepwell::constants::ADMIN_USER_ID;
use deepwell::services::RequestContext;
use deepwell::types::Reference;
use sea_orm::{ConnectionTrait, Statement};
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

    let vote = run_endpoint!(
        runner,
        vote_set,
        json!({
            "page_id": target.page_id,
            "user_id": ADMIN_USER_ID,
            "value": 1135,
        }),
    )
    .expect("deterministic smoke-test vote should be accepted");
    assert_eq!(vote.value, 1135);

    let source = r#"Before
[[module ListPages name="great-hippo-exact-name-target-3034"]]
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
        html.contains("16 May 2017 16:02"),
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
