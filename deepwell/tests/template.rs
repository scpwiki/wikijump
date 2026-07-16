/*
 * tests/template.rs
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
use deepwell::services::page::{CreatePageOutput, GetPageOutput};
use deepwell::types::Reference;
use serde_json::json;
use std::borrow::Cow;

fn set_page_actor(runner: &mut TestRunner, site_id: i64, slug: &str) {
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Owned(slug.to_owned()))),
    });
}

async fn test_site_id(runner: &TestRunner) -> i64 {
    run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site
        .site_id
}

async fn create_page(
    runner: &mut TestRunner,
    site_id: i64,
    slug: &str,
    wikitext: &str,
) -> CreatePageOutput {
    set_page_actor(runner, site_id, slug);
    let output = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": wikitext,
            "title": slug,
            "alt_title": null,
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "category template contract fixture",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(
        output.parser_errors.is_empty(),
        "unexpected parser errors: {:?}",
        output.parser_errors,
    );
    output
}

async fn get_page(runner: &TestRunner, site_id: i64, slug: &str) -> GetPageOutput {
    run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": slug,
            "details": {
                "wikitext": true,
                "compiled_html": true,
            },
        }),
    )
    .expect("fixture page should exist")
}

fn compiled_html(page: &GetPageOutput) -> &str {
    page.compiled_body_html
        .as_deref()
        .expect("compiled HTML should be requested")
}

fn assert_in_order(haystack: &str, expected: &[&str]) {
    let mut start = 0;
    for needle in expected {
        let offset = haystack[start..].find(needle).unwrap_or_else(|| {
            panic!("missing marker {needle:?} in compiled HTML:\n{haystack}")
        });
        start += offset + needle.len();
    }
}

async fn rerender_page(
    runner: &mut TestRunner,
    site_id: i64,
    category_id: i64,
    page_id: i64,
    slug: &str,
) {
    set_page_actor(runner, site_id, slug);
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": category_id,
            "page_id": page_id,
        }),
    );
}

#[tokio::test]
async fn exact_category_templates_wrap_render_input_but_not_stored_source() {
    const ROOT_PAGE: &str = "template-contract-root-page";
    const NAMED_WITHOUT_TEMPLATE: &str = "template-contract-plain:page";
    const NAMED_TEMPLATE: &str = "template-contract-exact:_template";
    const NAMED_PAGE: &str = "template-contract-exact:page";

    let mut runner = TestRunner::setup().await;
    let site_id = test_site_id(&runner).await;

    let global_template = create_page(
        &mut runner,
        site_id,
        "_template",
        "GLOBAL_BEFORE\n%%content%%\nGLOBAL_AFTER",
    )
    .await;
    let root = create_page(&mut runner, site_id, ROOT_PAGE, "ROOT_BODY").await;
    let plain =
        create_page(&mut runner, site_id, NAMED_WITHOUT_TEMPLATE, "PLAIN_BODY").await;
    let category_template = create_page(
        &mut runner,
        site_id,
        NAMED_TEMPLATE,
        "CATEGORY_V1_BEFORE\n%%content%%\nCATEGORY_V1_AFTER",
    )
    .await;
    let named = create_page(&mut runner, site_id, NAMED_PAGE, "NAMED_BODY").await;

    let root_page = get_page(&runner, site_id, ROOT_PAGE).await;
    assert_eq!(root_page.page_id, root.page_id);
    assert_eq!(root_page.wikitext.as_deref(), Some("ROOT_BODY"));
    assert_in_order(
        compiled_html(&root_page),
        &["GLOBAL_BEFORE", "ROOT_BODY", "GLOBAL_AFTER"],
    );

    let plain_page = get_page(&runner, site_id, NAMED_WITHOUT_TEMPLATE).await;
    assert_eq!(plain_page.page_id, plain.page_id);
    assert_eq!(plain_page.wikitext.as_deref(), Some("PLAIN_BODY"));
    assert!(compiled_html(&plain_page).contains("PLAIN_BODY"));
    assert!(!compiled_html(&plain_page).contains("GLOBAL_BEFORE"));

    let named_page = get_page(&runner, site_id, NAMED_PAGE).await;
    assert_eq!(named_page.page_id, named.page_id);
    assert_eq!(named_page.wikitext.as_deref(), Some("NAMED_BODY"));
    assert_in_order(
        compiled_html(&named_page),
        &["CATEGORY_V1_BEFORE", "NAMED_BODY", "CATEGORY_V1_AFTER"],
    );
    assert!(!compiled_html(&named_page).contains("GLOBAL_BEFORE"));

    let global_template_page = get_page(&runner, site_id, "_template").await;
    assert_eq!(global_template_page.page_id, global_template.page_id);
    assert_eq!(
        global_template_page.wikitext.as_deref(),
        Some("GLOBAL_BEFORE\n%%content%%\nGLOBAL_AFTER"),
    );
    assert!(!compiled_html(&global_template_page).contains("ROOT_BODY"));

    let category_template_page = get_page(&runner, site_id, NAMED_TEMPLATE).await;
    assert_eq!(category_template_page.page_id, category_template.page_id);
    assert_eq!(
        category_template_page.wikitext.as_deref(),
        Some("CATEGORY_V1_BEFORE\n%%content%%\nCATEGORY_V1_AFTER"),
    );
    assert!(!compiled_html(&category_template_page).contains("GLOBAL_BEFORE"));
}

#[tokio::test]
async fn template_edit_and_deletion_change_existing_page_on_rerender() {
    const TEMPLATE: &str = "template-contract-update:_template";
    const PAGE: &str = "template-contract-update:page";

    let mut runner = TestRunner::setup().await;
    let site_id = test_site_id(&runner).await;
    let template = create_page(
        &mut runner,
        site_id,
        TEMPLATE,
        "CATEGORY_V1_BEFORE\n%%content%%\nCATEGORY_V1_AFTER",
    )
    .await;
    let page = create_page(&mut runner, site_id, PAGE, "UNCHANGED_BODY").await;

    set_page_actor(&mut runner, site_id, TEMPLATE);
    let edited = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": TEMPLATE,
            "last_revision_id": template.revision_id,
            "revision_comments": "update category template contract fixture",
            "user_id": ADMIN_USER_ID,
            "wikitext": "CATEGORY_V2_BEFORE\n%%content%%\nCATEGORY_V2_AFTER",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("template edit should create a revision");

    let page_before_rerender = get_page(&runner, site_id, PAGE).await;
    rerender_page(
        &mut runner,
        site_id,
        page_before_rerender.page_category_id,
        page.page_id,
        PAGE,
    )
    .await;
    let page_after_edit = get_page(&runner, site_id, PAGE).await;
    assert_eq!(page_after_edit.wikitext.as_deref(), Some("UNCHANGED_BODY"));
    assert_in_order(
        compiled_html(&page_after_edit),
        &["CATEGORY_V2_BEFORE", "UNCHANGED_BODY", "CATEGORY_V2_AFTER"],
    );
    assert!(!compiled_html(&page_after_edit).contains("CATEGORY_V1_BEFORE"));

    set_page_actor(&mut runner, site_id, TEMPLATE);
    run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": TEMPLATE,
            "last_revision_id": edited.revision_id,
            "revision_comments": "delete category template contract fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    rerender_page(
        &mut runner,
        site_id,
        page_after_edit.page_category_id,
        page.page_id,
        PAGE,
    )
    .await;
    let page_after_delete = get_page(&runner, site_id, PAGE).await;
    assert_eq!(
        page_after_delete.wikitext.as_deref(),
        Some("UNCHANGED_BODY")
    );
    assert!(compiled_html(&page_after_delete).contains("UNCHANGED_BODY"));
    assert!(!compiled_html(&page_after_delete).contains("CATEGORY_V2_BEFORE"));
}
