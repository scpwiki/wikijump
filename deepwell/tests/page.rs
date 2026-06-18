/*
 * tests/page.rs
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
use deepwell::error::prelude::*;
use deepwell::services::page_query::{
    CategoriesSelector, DateSelector, FoundPageFields, IncludedCategories,
    OrderBySelector, OrderProperty, PageParentSelector, PageQuery, PageQueryService,
    PageTypeSelector, PaginationSelector, RangeSelector, TagCondition,
};
use deepwell::services::{PageService, RequestContext};
use deepwell::types::{PageRevisionType, Reference};
use serde_json::json;
use std::borrow::Cow;
use time::OffsetDateTime;

#[tokio::test]
async fn basic_edit() {
    let mut runner = TestRunner::setup().await;

    const SITE_SLUG: &str = "test";
    const PAGE_SLUG: &str = "my-page";

    // Get site

    let output = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("Seeded site not found");

    let site_id = output.site.site_id;
    assert_eq!(output.site.slug, SITE_SLUG, "Site slug doesn't match");

    // Set request context to populate params for the internal permission check.
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(PAGE_SLUG.into())),
    });

    // Create page

    let output = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "これは私のページの内容。 📄",
            "title": "五反田駅",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": null,
            "revision_comments": "作った",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let page_id = output.page_id;
    let revision_id = output.revision_id;
    assert_eq!(output.slug, PAGE_SLUG);
    assert!(output.parser_errors.is_empty());

    // Get page (by slug)

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
        }),
    )
    .expect("Cannot find page");
    assert_eq!(page.site_id, site_id);
    assert_eq!(page.page_id, page_id);
    assert_eq!(page.slug, PAGE_SLUG);
    assert_eq!(page.revision_id, revision_id);
    assert_eq!(page.revision_number, 0);
    assert_eq!(page.revision_type, PageRevisionType::Create);
    assert_eq!(page.revision_user_id, ADMIN_USER_ID);
    assert_eq!(page.page_category_slug, "_default");

    // Edit page contents (by slug)

    let output = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
            "last_revision_id": revision_id,
            "revision_comments": "もっと",
            "user_id": ADMIN_USER_ID,
            "wikitext": "これは私のページ！",
            "alt_title": "PAGE",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("No revision created");
    assert_eq!(output.revision_number, 1);
    assert!(output.revision_id > revision_id);
    let revision_id = output.revision_id;
    let parser_errors = output
        .parser_errors
        .expect("No parser errors list with wikitext change");
    assert!(parser_errors.is_empty());

    // Edit page contents (by ID). Browser form submissions carry the page in
    // the endpoint params, not Deepwell request-context page headers.
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: None,
        page_reference: None,
    });

    let output = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": page_id,
            "last_revision_id": revision_id,
            "revision_comments": "",
            "user_id": ADMIN_USER_ID,
            "title": "ようこそ",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("No revision created");
    assert_eq!(output.revision_number, 2);
    assert!(output.revision_id > revision_id);
    let revision_id = output.revision_id;

    // Edit with no changes

    let output = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": page_id,
            "last_revision_id": revision_id,
            "revision_comments": "nothing",
            "user_id": ADMIN_USER_ID,
            "title": "ようこそ",
            "wikitext": "これは私のページ！",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(
        output.is_none(),
        "Revision created when there were no changes"
    );

    // Get page (by ID)

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": page_id,
        }),
    )
    .expect("Cannot find page");
    assert_eq!(page.site_id, site_id);
    assert_eq!(page.page_id, page_id);
    assert_eq!(page.slug, PAGE_SLUG);
    assert_eq!(page.revision_id, revision_id);
    assert_eq!(page.revision_number, 2);
    assert_eq!(page.revision_type, PageRevisionType::Regular);
    assert_eq!(page.revision_user_id, ADMIN_USER_ID);
    assert_eq!(page.page_category_slug, "_default");
}

#[tokio::test]
async fn wikidot_site_include_uses_local_dependency_page_for_site_qualified_include() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");

    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site.site_id,
            "wikitext": "[[module CSS]]\n@import url(https://scp-wiki.wdfiles.com/local--code/theme%3Abasalt/3)\n[[/module]]\n",
            "title": "Basalt Theme",
            "alt_title": null,
            "slug": "theme:codex-include-fallback",
            "layout": "wikidot",
            "revision_comments": "create local theme dependency",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site.site_id,
            "wikitext": "[[include :scp-wiki:theme:codex-include-fallback | hidetitle=a]]\nbody\n",
            "title": "Include Consumer",
            "alt_title": null,
            "slug": "include-consumer",
            "layout": "wikidot",
            "revision_comments": "create include consumer",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site.site.site_id,
            "page": "include-consumer",
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("include consumer should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("theme%3Abasalt/3"),
        "compiled page should include CSS from the local theme dependency: {html}"
    );
    assert!(
        html.contains("body"),
        "compiled page should retain the consumer page body"
    );
    assert!(
        html.contains("calc(var(--side-bar-width, 17rem) * -1)"),
        "compiled Basalt page should include Wikidot shell sidebar compatibility CSS: {html}"
    );
}

#[tokio::test]
async fn listpages_fixture_subset_renders_titles_slugs_order_and_tag_filter() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &runner,
        site_id,
        "fixture-listpages-unit-parent-root",
        "Fixture Parent Root",
        "Fixture Parent Root marker.",
    )
    .await;

    let target_a_revision = create_listpages_test_page(
        &runner,
        site_id,
        "fixture-listpages-unit-target-a",
        "Fixture ListPages Target Alpha",
        "Fixture ListPages Target Alpha marker.",
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        "fixture-listpages-unit-target-a",
        target_a_revision,
        &["verification", "verification-list-unit"],
    )
    .await;
    set_listpages_test_parent(
        &runner,
        site_id,
        "fixture-listpages-unit-target-a",
        "fixture-listpages-unit-parent-root",
    )
    .await;

    let target_b_revision = create_listpages_test_page(
        &runner,
        site_id,
        "fixture-listpages-unit-target-b",
        "Fixture ListPages Target Beta",
        "Fixture ListPages Target Beta marker.",
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        "fixture-listpages-unit-target-b",
        target_b_revision,
        &["verification", "verification-list-unit"],
    )
    .await;
    set_listpages_test_parent(
        &runner,
        site_id,
        "fixture-listpages-unit-target-b",
        "fixture-listpages-unit-parent-root",
    )
    .await;

    let target_c_revision = create_listpages_test_page(
        &runner,
        site_id,
        "fixture-listpages-unit-target-c",
        "Fixture ListPages Target Gamma",
        "Fixture ListPages Target Gamma marker.",
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        "fixture-listpages-unit-target-c",
        target_c_revision,
        &["verification", "verification-list-unit"],
    )
    .await;
    set_listpages_test_parent(
        &runner,
        site_id,
        "fixture-listpages-unit-target-c",
        "fixture-listpages-unit-parent-root",
    )
    .await;

    let excluded_revision = create_listpages_test_page(
        &runner,
        site_id,
        "fixture-listpages-unit-excluded",
        "Fixture ListPages Excluded",
        "Fixture ListPages Excluded marker. This text must not appear in the ListPages index.",
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        "fixture-listpages-unit-excluded",
        excluded_revision,
        &["verification", "verification-excluded"],
    )
    .await;

    create_listpages_test_page(
        &runner,
        site_id,
        "fixture-listpages-unit-index",
        "Fixture ListPages Index",
        "ListPages start marker.\n\n[[module ListPages tags=\"+verification-list-unit\" limit=\"10\" order=\"name\"]]\n* %%title%% :: %%slug%%\n[[/module]]\n\nListPages end marker.",
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "fixture-listpages-unit-index",
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("ListPages index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    for expected in [
        "ListPages start marker",
        "Fixture ListPages Target Alpha",
        "Fixture ListPages Target Beta",
        "Fixture ListPages Target Gamma",
        "fixture-listpages-unit-target-a",
        "fixture-listpages-unit-target-b",
        "fixture-listpages-unit-target-c",
        "ListPages end marker",
    ] {
        assert!(
            html.contains(expected),
            "compiled ListPages fixture should contain {expected:?}:\n{html}"
        );
    }

    for forbidden in [
        "Fixture ListPages Excluded",
        "fixture-listpages-unit-excluded",
        "%%title%%",
        "%%slug%%",
        "[[module ListPages",
    ] {
        assert!(
            !html.contains(forbidden),
            "compiled ListPages fixture should not contain {forbidden:?}:\n{html}"
        );
    }

    let target_a = html
        .find("fixture-listpages-unit-target-a")
        .expect("target A slug should render");
    let target_b = html
        .find("fixture-listpages-unit-target-b")
        .expect("target B slug should render");
    let target_c = html
        .find("fixture-listpages-unit-target-c")
        .expect("target C slug should render");
    assert!(
        target_a < target_b && target_b < target_c,
        "target slugs should render in order a, b, c:\n{html}"
    );
}

async fn create_listpages_test_page(
    runner: &TestRunner,
    site_id: i64,
    slug: &str,
    title: &str,
    wikitext: &str,
) -> i64 {
    let output = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": wikitext,
            "title": title,
            "alt_title": null,
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "create ListPages test page",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    output.revision_id
}

async fn set_listpages_test_tags(
    runner: &mut TestRunner,
    site_id: i64,
    slug: &str,
    last_revision_id: i64,
    tags: &[&str],
) {
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(std::borrow::Cow::Owned(slug.to_owned()))),
    });

    let output = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": slug,
            "last_revision_id": last_revision_id,
            "revision_comments": "set ListPages test tags",
            "user_id": ADMIN_USER_ID,
            "tags": tags,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("tag edit should create a revision");
    let parser_errors = output
        .parser_errors
        .expect("tag edit should return parser errors");
    assert!(parser_errors.is_empty());
}

async fn set_listpages_test_parent(
    runner: &TestRunner,
    site_id: i64,
    slug: &str,
    parent: &str,
) {
    run_endpoint!(
        runner,
        parent_update,
        json!({
            "site_id": site_id,
            "child": slug,
            "add": [parent],
            "remove": null,
        }),
    );
}

async fn render_listpages_test_fixture(
    runner: &mut TestRunner,
    site_id: i64,
    slug_prefix: &str,
    tag: &str,
    module_head: &str,
    body: &str,
) -> String {
    let parent_slug = format!("{slug_prefix}-parent-root");
    let target_a_slug = format!("{slug_prefix}-target-a");
    let target_b_slug = format!("{slug_prefix}-target-b");
    let target_c_slug = format!("{slug_prefix}-target-c");
    let excluded_slug = format!("{slug_prefix}-excluded");
    let index_slug = format!("{slug_prefix}-index");

    create_listpages_test_page(
        runner,
        site_id,
        &parent_slug,
        "Fixture Parent Root",
        "Fixture Parent Root marker.",
    )
    .await;

    for (slug, title, source) in [
        (
            target_a_slug.as_str(),
            "Fixture ListPages Target Alpha",
            "Fixture ListPages Target Alpha marker.",
        ),
        (
            target_b_slug.as_str(),
            "Fixture ListPages Target Beta",
            "Fixture ListPages Target Beta marker.",
        ),
        (
            target_c_slug.as_str(),
            "Fixture ListPages Target Gamma",
            "Fixture ListPages Target Gamma marker.",
        ),
    ] {
        let revision =
            create_listpages_test_page(runner, site_id, slug, title, source).await;
        set_listpages_test_tags(runner, site_id, slug, revision, &["verification", tag])
            .await;
        set_listpages_test_parent(runner, site_id, slug, &parent_slug).await;
    }

    let excluded_revision = create_listpages_test_page(
        runner,
        site_id,
        &excluded_slug,
        "Fixture ListPages Excluded",
        "Fixture ListPages Excluded marker.",
    )
    .await;
    set_listpages_test_tags(
        runner,
        site_id,
        &excluded_slug,
        excluded_revision,
        &["verification", "verification-excluded"],
    )
    .await;

    create_listpages_test_page(
        runner,
        site_id,
        &index_slug,
        "Fixture ListPages Index",
        &format!(
            "ListPages start marker.\n\n[[module ListPages {module_head}]]\n{body}\n[[/module]]\n\nListPages end marker."
        ),
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": index_slug,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("ListPages index should exist");

    page.compiled_body_html
        .expect("compiled body should be included in page_get details")
}

#[tokio::test]
async fn listpages_limit_two_caps_ordered_results() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let html = render_listpages_test_fixture(
        &mut runner,
        site.site.site_id,
        "fixture-listpages-limit",
        "verification-list-limit",
        r#"tags="+verification-list-limit" limit="2" order="name""#,
        "* %%title%% :: %%slug%%",
    )
    .await;

    for expected in [
        "Fixture ListPages Target Alpha",
        "Fixture ListPages Target Beta",
        "fixture-listpages-limit-target-a",
        "fixture-listpages-limit-target-b",
    ] {
        assert!(
            html.contains(expected),
            "limit=2 ListPages fixture should contain {expected:?}:\n{html}"
        );
    }

    for forbidden in [
        "Fixture ListPages Target Gamma",
        "fixture-listpages-limit-target-c",
        "Fixture ListPages Excluded",
        "fixture-listpages-limit-excluded",
        "%%title%%",
        "%%slug%%",
        "[[module ListPages",
    ] {
        assert!(
            !html.contains(forbidden),
            "limit=2 ListPages fixture should not contain {forbidden:?}:\n{html}"
        );
    }

    let target_a = html
        .find("fixture-listpages-limit-target-a")
        .expect("target A slug should render");
    let target_b = html
        .find("fixture-listpages-limit-target-b")
        .expect("target B slug should render");
    assert!(
        target_a < target_b,
        "limit=2 target slugs should render in order a, b:\n{html}"
    );
}

#[tokio::test]
async fn listpages_created_at_order_renders_results() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let html = render_listpages_test_fixture(
        &mut runner,
        site.site.site_id,
        "fixture-listpages-created-at",
        "verification-list-created-at",
        r#"tags="+verification-list-created-at" limit="3" order="created_at""#,
        "* %%title%% :: %%slug%%",
    )
    .await;

    for expected in [
        "Fixture ListPages Target Alpha",
        "Fixture ListPages Target Beta",
        "Fixture ListPages Target Gamma",
        "fixture-listpages-created-at-target-a",
        "fixture-listpages-created-at-target-b",
        "fixture-listpages-created-at-target-c",
    ] {
        assert!(
            html.contains(expected),
            "created_at ListPages fixture should contain {expected:?}:\n{html}",
        );
    }

    assert!(
        !html.contains("[[module ListPages") && !html.contains("%%title%%"),
        "created_at ListPages fixture should render instead of remaining raw:\n{html}",
    );
}

#[tokio::test]
async fn page_query_orders_by_page_slug_without_category_prefix() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-page-slug-order";

    for (slug, title) in [
        ("zcategory:alpha", "Page slug order alpha"),
        ("acategory:beta", "Page slug order beta"),
        ("mcategory:gamma", "Page slug order gamma"),
    ] {
        let revision = create_listpages_test_page(
            &runner,
            site_id,
            slug,
            title,
            "Page slug order marker.",
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision, &[tag]).await;
    }

    let all_tags = [Cow::Borrowed(tag)];
    let pages = PageQueryService::find(
        runner.context(),
        PageQuery {
            current_page_id: 0,
            current_site_id: site_id,
            queried_site_id: Some(site_id),
            page_type: PageTypeSelector::All,
            categories: CategoriesSelector {
                included_categories: IncludedCategories::All,
                excluded_categories: &[],
            },
            tags: TagCondition {
                any_present: &[],
                all_present: &all_tags,
                none_present: &[],
            },
            page_parent: PageParentSelector::NoParent,
            contains_outgoing_links: &[],
            creation_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            update_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            author: &[],
            score: &[],
            votes: &[],
            offset: 0,
            range: RangeSelector::Current,
            name: None,
            slug: None,
            data_form_fields: &[],
            order: Some(OrderBySelector {
                property: OrderProperty::PageSlug,
                ascending: true,
            }),
            pagination: PaginationSelector {
                limit: Some(10),
                ..Default::default()
            },
            variables: &[],
            fields: FoundPageFields {
                slug: true,
                ..Default::default()
            },
        },
    )
    .await
    .expect("page slug order query should not fail");

    let slugs = pages
        .pages
        .into_iter()
        .map(|row| row.slug.expect("slug field should be requested"))
        .collect::<Vec<_>>();

    assert_eq!(
        slugs,
        ["zcategory:alpha", "acategory:beta", "mcategory:gamma"],
        "PageSlug order should sort by page slug, not by full category-qualified slug",
    );
}

#[tokio::test]
async fn page_query_score_order_returns_results() {
    let runner = TestRunner::setup().await;
    PageQueryService::find(
        runner.context(),
        PageQuery {
            current_page_id: 0,
            current_site_id: 0,
            queried_site_id: Some(0),
            page_type: PageTypeSelector::All,
            categories: CategoriesSelector {
                included_categories: IncludedCategories::All,
                excluded_categories: &[],
            },
            tags: TagCondition {
                any_present: &[],
                all_present: &[],
                none_present: &[],
            },
            page_parent: PageParentSelector::NoParent,
            contains_outgoing_links: &[],
            creation_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            update_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            author: &[],
            score: &[],
            votes: &[],
            offset: 0,
            range: RangeSelector::Current,
            name: None,
            slug: None,
            data_form_fields: &[],
            order: Some(OrderBySelector {
                property: OrderProperty::Score,
                ascending: true,
            }),
            pagination: PaginationSelector::default(),
            variables: &[],
            fields: FoundPageFields::default(),
        },
    )
    .await
    .expect("score ordering should not fail");
}

#[tokio::test]
async fn page_undo_reverses_selected_revision_and_preserves_later_changes() {
    let mut runner = TestRunner::setup().await;

    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;
    let slug = "undo-target";

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
            "wikitext": "Body version 0",
            "title": "Title version 0",
            "alt_title": null,
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "create undo target",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let title_edit = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": slug,
            "last_revision_id": created.revision_id,
            "revision_comments": "change title",
            "user_id": ADMIN_USER_ID,
            "title": "Title version 1",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("title edit should create a revision");

    let body_edit = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": slug,
            "last_revision_id": title_edit.revision_id,
            "revision_comments": "change body",
            "user_id": ADMIN_USER_ID,
            "wikitext": "Body version 2",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("body edit should create a revision");

    let undo = PageService::undo(runner.context(), site_id, created.page_id, 1)
        .await
        .expect("undo should reverse the title edit");

    assert!(undo.revision_id > body_edit.revision_id);
    assert_eq!(undo.revision_number, 3);

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": slug,
            "details": {
                "wikitext": true
            },
        }),
    )
    .expect("page should exist after undo");

    assert_eq!(page.revision_type, PageRevisionType::Undo);
    assert_eq!(page.revision_number, 3);
    assert_eq!(page.title, "Title version 0");
    assert_eq!(
        page.wikitext.as_deref(),
        Some("Body version 2"),
        "undo should preserve the later unrelated body edit",
    );
}

#[tokio::test]
async fn listpages_deferred_forms_remain_unsupported() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");

    for (slug_suffix, module_head, body, raw_indicator) in [
        (
            "unknown-variable",
            r#"tags="+verification-list-negative-unknown-variable" limit="10" order="name""#,
            "* %%unsupported_variable%%",
            "%%unsupported_variable%%",
        ),
        (
            "range-others",
            r#"tags="+verification-list-negative-range-others" limit="10" order="name" range="others""#,
            "* %%title%% :: %%slug%%",
            "range=&quot;others&quot;",
        ),
        (
            "range-before",
            r#"tags="+verification-list-negative-range-before" limit="10" order="name" range="before""#,
            "* %%title%% :: %%slug%%",
            "range=&quot;before&quot;",
        ),
        (
            "per-page",
            r#"tags="+verification-list-negative-per-page" limit="10" per_page="1" order="name""#,
            "* %%title%% :: %%slug%%",
            "per_page=&quot;1&quot;",
        ),
        (
            "created-by-arg",
            r#"tags="+verification-list-negative-created-by-arg" limit="10" created_by="=" order="name""#,
            "* %%title%% :: %%slug%%",
            "created_by=&quot;=&quot;",
        ),
        (
            "created-by-variable",
            r#"tags="+verification-list-negative-created-by-variable" limit="10" order="name""#,
            "* %%created_by%%",
            "%%created_by%%",
        ),
        (
            "rating-variable",
            r#"tags="+verification-list-negative-rating-variable" limit="10" order="name""#,
            "* %%rating%%",
            "%%rating%%",
        ),
        (
            "tag-exclusion",
            r#"tag="-verification-list-negative-tag-exclusion" limit="10" order="name""#,
            "* %%title%% :: %%slug%%",
            r#"tag=&quot;-verification-list-negative-tag-exclusion&quot;"#,
        ),
    ] {
        let slug_prefix = format!("fixture-listpages-negative-{slug_suffix}");
        let tag = format!("verification-list-negative-{slug_suffix}");
        let html = render_listpages_test_fixture(
            &mut runner,
            site.site.site_id,
            &slug_prefix,
            &tag,
            module_head,
            body,
        )
        .await;

        assert!(
            html.contains(raw_indicator)
                || html.contains("[[module ListPages")
                || html.contains("module ListPages"),
            "unsupported ListPages case {slug_suffix} should remain raw/degraded rather than silently accepted:\n{html}"
        );
        assert!(
            !html.contains(&format!(
                "Fixture ListPages Target Alpha :: {slug_prefix}-target-a"
            )),
            "unsupported ListPages case {slug_suffix} must not silently render accepted title/slug rows:\n{html}"
        );
    }
}

#[tokio::test]
async fn basic_move() {
    let mut runner = TestRunner::setup().await;

    const SITE_SLUG: &str = "test";
    const PAGE_SLUG_1: &str = "alpha";
    const PAGE_SLUG_2: &str = "beta";

    // Get site

    let output = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("Seeded site not found");

    let site_id = output.site.site_id;
    assert_eq!(output.site.slug, SITE_SLUG, "Site slug doesn't match");

    // Set request context to populate params for the internal permission check.
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(PAGE_SLUG_1.into())),
    });

    // Create page

    let output = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "PAGE APPLE",
            "title": "Alpha 1",
            "alt_title": null,
            "slug": PAGE_SLUG_1,
            "layout": null,
            "revision_comments": "Created page",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let page_id = output.page_id;
    let revision_id = output.revision_id;
    assert_eq!(output.slug, PAGE_SLUG_1);
    assert!(output.parser_errors.is_empty());

    // Page edit (success)

    let output = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": page_id,
            "last_revision_id": revision_id,
            "revision_comments": "Edited page 1",
            "user_id": ADMIN_USER_ID,
            "title": "List of Things",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("No revision created");
    assert_eq!(output.revision_number, 1);
    assert!(output.revision_id > revision_id);
    let revision_id = output.revision_id;

    // Move page

    let output = run_endpoint!(
        runner,
        page_move,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG_1,
            "new_slug": PAGE_SLUG_2,
            "last_revision_id": revision_id,
            "revision_comments": "move",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(output.revision_number, 2);
    assert!(output.revision_id > revision_id);
    let revision_id = output.revision_id;

    // Get page (by ID)

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": page_id,
        }),
    )
    .expect("Cannot find page");
    assert_eq!(page.site_id, site_id);
    assert_eq!(page.page_id, page_id);
    assert_eq!(page.slug, PAGE_SLUG_2);
    assert_eq!(page.revision_id, revision_id);
    assert_eq!(page.revision_number, 2);
    assert_eq!(page.revision_type, PageRevisionType::Move);
    assert_eq!(page.revision_user_id, ADMIN_USER_ID);
    assert_eq!(page.page_category_slug, "_default");

    // Page edit (failure)

    let error = run_endpoint_err!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG_1,
            "last_revision_id": revision_id,
            "revision_comments": "Update title",
            "user_id": ADMIN_USER_ID,
            "title": "Beta 2",
            "wikitext": "PAGE BANANA",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PageNotFound);

    // Page edit (success)
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(PAGE_SLUG_2.into())),
    });

    let output = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG_2,
            "last_revision_id": revision_id,
            "revision_comments": "Update title",
            "user_id": ADMIN_USER_ID,
            "title": "Beta 2",
            "wikitext": "PAGE BANANA",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("No revision created");
    assert_eq!(output.revision_number, 3);
    assert!(output.revision_id > revision_id);
}

// TODO add more cases here
// e.g. create page in non-default category, move to a new category
//      create page, edit, delete, edit (fail), restore, edit (success), restore (fail)
//      create two pages, edit, make sure revision numbers are consistent
//      create page, have a variety of different edits, list revisions and check info
//      create page, edit with outdated revision, revision for another page, negative revision
//      create page, get with details (each permutation), check values are correct
//      create page, add revisions, then go back and hide revision data, then request that data (should be omitted)
//      etc.
