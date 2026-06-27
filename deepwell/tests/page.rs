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
use deepwell::models::page::{self, Entity as PageTable};
use deepwell::models::page_revision::Entity as PageRevisionTable;
use deepwell::services::RequestContext;
use deepwell::services::page_query::{
    CategoriesSelector, DateSelector, FoundPageFields, IncludedCategories,
    OrderBySelector, OrderProperty, PageParentSelector, PageQuery, PageQueryService,
    PageTypeSelector, PaginationSelector, RangeSelector, TagCondition,
};
use deepwell::types::{PageRevisionType, Reference};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set,
};
use serde_json::json;
use std::borrow::Cow;
use time::{Duration, OffsetDateTime};

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

    // Edit page contents (by ID)

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
            "wikitext": "[[include :scp-wiki:theme:codex-include-fallback | hidetitle=a]]\nInclude consumer body marker.\n",
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
        html.contains("Include consumer body marker."),
        "compiled page should retain the consumer page body"
    );
    assert!(
        html.contains("#side-bar") && html.contains("display: none !important"),
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

#[tokio::test]
async fn first_revision_current_page_listpages_uses_render_page_info() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": concat!(
                "Before first revision ListPages.\n\n",
                "[[module ListPages range=\".\"]]\n",
                "**%%title%%** :: %%fullname%%\n",
                "[[/module]]\n\n",
                "After first revision ListPages."
            ),
            "title": "Fixture First Revision Current Page",
            "alt_title": null,
            "slug": "fixture-first-revision-current-page-listpages",
            "layout": "wikidot",
            "revision_comments": "create page with current-page ListPages",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "fixture-first-revision-current-page-listpages",
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("first-revision ListPages page should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    for expected in [
        "Before first revision ListPages.",
        "Fixture First Revision Current Page",
        "fixture-first-revision-current-page-listpages",
        "After first revision ListPages.",
    ] {
        assert!(
            html.contains(expected),
            "compiled first-revision ListPages page should contain {expected:?}:\n{html}"
        );
    }

    for forbidden in ["[[module ListPages", "%%title%%", "%%fullname%%"] {
        assert!(
            !html.contains(forbidden),
            "compiled first-revision ListPages page should not contain {forbidden:?}:\n{html}"
        );
    }
}

#[tokio::test]
async fn included_author_tool_coauthored_branch_renders_named_page_box() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let target_revision = create_listpages_test_page(
        &runner,
        site_id,
        "fixture-coauthored-target",
        "Fixture Coauthored Target",
        "Fixture Coauthored Target marker.",
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        "fixture-coauthored-target",
        target_revision,
        &["verification", "verification-coauthored-target"],
    )
    .await;

    create_listpages_test_page(
        &runner,
        site_id,
        "component:coauthored-listpages-emitter",
        "Fixture Author Tool Component",
        concat!(
            "[!-- {$inc-coauthored}\n\n",
            "[[module Listpages fullname=\"{$name}\" category=\"*\"]]\n",
            "[[div class=\"content-box {$shadow}\"]]\n",
            "++ **%%title_linked%%** (//feat.// {$feat})\n",
            "[[div class=\"content-section\"]]\n",
            "------\n",
            "**Rating:** +%%rating%%\n",
            "[[/div]]\n",
            "[[div class=\"content-section\"]]\n",
            "------\n",
            "[[div class=\"translations\"]]\n",
            "[[collapsible show=\"+ Translations\" hide=\"- Translations\"]]\n",
            "[[div class=\"scpnet-interwiki-wrapper interwiki-stylable\"]]\n",
            "[[embed]]\n",
            "<iframe src=\"//interwiki.scpwiki.com/interwikiFrame.html?lang=en&community=scp&pagename=%%fullname%%\" allowtransparency=\"true\" class=\"html-block-iframe scpnet-interwiki-frame\"></iframe>\n",
            "[[/embed]]\n",
            "[[/div]]\n",
            "[[/collapsible]]\n",
            "[[/div]]\n",
            "[[/div]]\n",
            "[[/div]]\n",
            "[[/module]]\n\n",
            "[!----]\n",
        ),
    )
    .await;

    create_listpages_test_page(
        &runner,
        site_id,
        "fixture-coauthored-index",
        "Fixture Coauthored Index",
        concat!(
            "Before coauthored include.\n\n",
            "[[include component:coauthored-listpages-emitter |inc-coauthored= --]\n",
            "|name=fixture-coauthored-target\n",
            "|feat=Collaborator\n",
            "|language=en\n",
            "]]\n\n",
            "After coauthored include.",
        ),
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "fixture-coauthored-index",
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("coauthored ListPages index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    for expected in [
        "Before coauthored include.",
        "Fixture Coauthored Target",
        "Collaborator",
        "content-box",
        "content-section",
        "scpnet-interwiki-frame",
        "After coauthored include.",
    ] {
        assert!(
            html.contains(expected),
            "compiled coauthored author-tool fixture should contain {expected:?}:\n{html}"
        );
    }

    for forbidden in [
        "[[module Listpages",
        "[[module ListPages",
        "%%title_linked%%",
        "%%fullname%%",
        "{$shadow}",
    ] {
        assert!(
            !html.contains(forbidden),
            "compiled coauthored author-tool fixture should not contain {forbidden:?}:\n{html}"
        );
    }
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
    render_listpages_test_fixture_with_targets(
        runner,
        site_id,
        slug_prefix,
        tag,
        module_head,
        body,
        &[
            (
                "target-a",
                "Fixture ListPages Target Alpha",
                "Fixture ListPages Target Alpha marker.",
            ),
            (
                "target-b",
                "Fixture ListPages Target Beta",
                "Fixture ListPages Target Beta marker.",
            ),
            (
                "target-c",
                "Fixture ListPages Target Gamma",
                "Fixture ListPages Target Gamma marker.",
            ),
        ],
    )
    .await
}

async fn render_listpages_test_fixture_with_targets(
    runner: &mut TestRunner,
    site_id: i64,
    slug_prefix: &str,
    tag: &str,
    module_head: &str,
    body: &str,
    targets: &[(&str, &str, &str)],
) -> String {
    let parent_slug = format!("{slug_prefix}-parent-root");
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

    for (index, &(slug_suffix, title, source)) in targets.iter().enumerate() {
        let slug = format!("{slug_prefix}-{slug_suffix}");
        let revision =
            create_listpages_test_page(runner, site_id, &slug, title, source).await;
        set_listpages_test_created_at(
            runner,
            site_id,
            &slug,
            OffsetDateTime::UNIX_EPOCH + Duration::seconds(index as i64 + 1),
        )
        .await;
        set_listpages_test_tags(runner, site_id, &slug, revision, &["verification", tag])
            .await;
        set_listpages_test_parent(runner, site_id, &slug, &parent_slug).await;
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

async fn set_listpages_test_created_at(
    runner: &TestRunner,
    site_id: i64,
    slug: &str,
    created_at: OffsetDateTime,
) {
    let page = PageTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page::Column::SiteId.eq(site_id))
                .add(page::Column::Slug.eq(slug)),
        )
        .one(runner.context().transaction())
        .await
        .expect("created_at test page lookup should not fail")
        .expect("created_at test page should exist");
    let mut model = page.into_active_model();
    model.created_at = Set(created_at);
    model
        .update(runner.context().transaction())
        .await
        .expect("created_at test page update should not fail");
}

async fn set_listpages_test_revision_number(
    runner: &TestRunner,
    revision_id: i64,
    revision_number: i32,
) {
    let revision = PageRevisionTable::find_by_id(revision_id)
        .one(runner.context().transaction())
        .await
        .expect("revision-number test lookup should not fail")
        .expect("revision-number test revision should exist");
    let mut model = revision.into_active_model();
    model.revision_number = Set(revision_number);
    model
        .update(runner.context().transaction())
        .await
        .expect("revision-number test update should not fail");
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
    let html = render_listpages_test_fixture_with_targets(
        &mut runner,
        site.site.site_id,
        "fixture-listpages-created-at",
        "verification-list-created-at",
        r#"tags="+verification-list-created-at" limit="3" order="created_at""#,
        "* %%title%% :: %%slug%%",
        &[
            (
                "target-z",
                "Fixture ListPages Created Zulu",
                "Fixture ListPages Created Zulu marker.",
            ),
            (
                "target-a",
                "Fixture ListPages Created Alpha",
                "Fixture ListPages Created Alpha marker.",
            ),
            (
                "target-m",
                "Fixture ListPages Created Middle",
                "Fixture ListPages Created Middle marker.",
            ),
        ],
    )
    .await;

    for expected in [
        "Fixture ListPages Created Zulu",
        "Fixture ListPages Created Alpha",
        "Fixture ListPages Created Middle",
        "fixture-listpages-created-at-target-z",
        "fixture-listpages-created-at-target-a",
        "fixture-listpages-created-at-target-m",
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

    let positions = [
        "fixture-listpages-created-at-target-z",
        "fixture-listpages-created-at-target-a",
        "fixture-listpages-created-at-target-m",
    ]
    .map(|slug| {
        html.find(slug).unwrap_or_else(|| {
            panic!("created_at ListPages fixture should contain {slug:?}")
        })
    });
    assert!(
        positions[0] < positions[1] && positions[1] < positions[2],
        "created_at ListPages fixture should render in creation order, not lexical slug/title order:\n{html}"
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
async fn page_query_created_by_uses_earliest_available_revision() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-page-query-created-by-earliest";
    let slug = "fixture-page-query-created-by-earliest";

    let revision_id = create_listpages_test_page(
        &runner,
        site_id,
        slug,
        "Fixture PageQuery CreatedBy Earliest",
        "Fixture PageQuery CreatedBy Earliest marker.",
    )
    .await;
    set_listpages_test_revision_number(&runner, revision_id, 42).await;
    set_listpages_test_tags(&mut runner, site_id, slug, revision_id, &[tag]).await;

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
            page_parent: PageParentSelector::All,
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
                created_by: true,
                ..Default::default()
            },
        },
    )
    .await
    .expect("created_by query should not fail");

    let page = pages
        .pages
        .iter()
        .find(|page| page.slug.as_deref() == Some(slug))
        .expect("created_by query should include the fixture page");
    assert_eq!(
        page.created_by,
        Some(ADMIN_USER_ID),
        "created_by should come from the earliest available revision, even when it is not revision 0",
    );

    let author_filter = [Cow::Owned(ADMIN_USER_ID.to_string())];
    let filtered_pages = PageQueryService::find(
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
            page_parent: PageParentSelector::All,
            contains_outgoing_links: &[],
            creation_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            update_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            author: &author_filter,
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
                created_by: true,
                ..Default::default()
            },
        },
    )
    .await
    .expect("author-filtered query should not fail");
    assert!(
        filtered_pages
            .pages
            .iter()
            .any(|page| page.slug.as_deref() == Some(slug)),
        "author filter should use the same earliest-available revision semantics as created_by",
    );
}

#[tokio::test]
async fn page_query_score_order_returns_results() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-page-score-order";

    for (slug, title, vote) in [
        ("fixture-score-order-high", "Score Order High", 5),
        ("fixture-score-order-low", "Score Order Low", -2),
        ("fixture-score-order-zero", "Score Order Zero", 0),
    ] {
        let output = run_endpoint!(
            runner,
            page_create,
            json!({
                "site_id": site_id,
                "wikitext": "Score order marker.",
                "title": title,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "create score order test page",
                "user_id": ADMIN_USER_ID,
                "ip_address": common::IP_ADDRESS,
            }),
        );
        set_listpages_test_tags(&mut runner, site_id, slug, output.revision_id, &[tag])
            .await;
        if vote != 0 {
            run_endpoint!(
                runner,
                vote_set,
                json!({
                    "page_id": output.page_id,
                    "user_id": ADMIN_USER_ID,
                    "value": vote,
                }),
            );
        }
    }

    let all_tags = [Cow::Borrowed(tag)];
    let base_query = PageQuery {
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
            property: OrderProperty::Score,
            ascending: true,
        }),
        pagination: PaginationSelector::default(),
        variables: &[],
        fields: FoundPageFields {
            slug: true,
            score: true,
            ..Default::default()
        },
    };

    let pages = PageQueryService::find(runner.context(), base_query.clone())
        .await
        .expect("score ordering should not fail");

    let ordered = pages
        .pages
        .into_iter()
        .map(|row| {
            (
                row.slug.expect("slug field should be requested"),
                row.score.expect("score field should be requested"),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        ordered,
        [
            ("fixture-score-order-low".to_owned(), -2.0),
            ("fixture-score-order-zero".to_owned(), 0.0),
            ("fixture-score-order-high".to_owned(), 5.0),
        ],
        "score order query should return pages sorted by computed score",
    );

    let mut limited_query = base_query.clone();
    limited_query.pagination.limit = Some(2);
    let limited_pages = PageQueryService::find(runner.context(), limited_query)
        .await
        .expect("limited score ordering should not fail");

    let limited_ordered = limited_pages
        .pages
        .into_iter()
        .map(|row| {
            (
                row.slug.expect("slug field should be requested"),
                row.score.expect("score field should be requested"),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        limited_ordered,
        [
            ("fixture-score-order-low".to_owned(), -2.0),
            ("fixture-score-order-zero".to_owned(), 0.0),
        ],
        "limited score order should truncate after computed-score sorting",
    );

    let mut offset_query = base_query;
    offset_query.offset = 1;
    offset_query.pagination.limit = Some(1);
    let offset_pages = PageQueryService::find(runner.context(), offset_query)
        .await
        .expect("offset score ordering should not fail");
    let offset_ordered = offset_pages
        .pages
        .into_iter()
        .map(|row| {
            (
                row.slug.expect("slug field should be requested"),
                row.score.expect("score field should be requested"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        offset_ordered,
        [("fixture-score-order-zero".to_owned(), 0.0)],
        "score order should apply offset after computed-score sorting",
    );
}

#[tokio::test]
async fn listpages_deferred_forms_remain_unsupported() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");

    for (slug_suffix, module_head, body, raw_indicator) in [(
        "unknown-variable",
        r#"tags="+verification-list-negative-unknown-variable" limit="10" order="name""#,
        "* %%unsupported_variable%%",
        "%%unsupported_variable%%",
    )] {
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
