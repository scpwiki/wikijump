/*
 * tests/page/page_modules.rs
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

//! Page-selection and navigation runtime module integration tests.
//!
//! Extracted from `tests/page.rs` to keep the integration-test crate root
//! focused. These cases exercise the page-oriented runtime modules (Pages,
//! ChildPages, NextPreviousPage, PagesByTag, ListPages URL/pagination/AJAX/
//! parent/tag/range/metric/RSS/feed behavior, Backlinks, OrphanedPages,
//! WantedPages, NewPage, Categories, and PageTree) through the public JSONRPC
//! API. Shared crate-root imports and helpers are reused via `super::*`.

use super::*;
use ftml::parsing::{ParseErrorKind, Token};

/// Live capture (sandbox-for-codex, 2026-07-25): `[[module Pages]]` emits
/// `div.list-pages-box` with 20 title-linked rows per page and a Wikidot pager.
/// `/p/999` clamps to the final page.
#[tokio::test]
async fn pages_module_renders_the_site_index_and_clamps_pagination() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let holder_slug = "fixture-pages-holder";
    let private_category = "fixture-pages-private";

    make_listpages_test_category_admin_only(&runner, site_id, private_category).await;

    for (slug, title) in [
        ("fixture-pages-zulu", "!! Fixture Pages Zulu"),
        ("fixture-pages-aardvark", "!! Fixture Pages aardvark"),
        ("fixture-pages-private:hidden", "!! Fixture Pages Private"),
    ] {
        create_listpages_test_page(
            &mut runner,
            site_id,
            slug,
            title,
            "Pages module fixture.",
        )
        .await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        holder_slug,
        "!! Fixture Pages Holder",
        concat!(
            "PAGES_START\n\n",
            "[[module Pages]]\n\n",
            "PAGES_END\n\n",
            "AUTHORED_DOTS:x....x\n\n",
            "LITERAL_START\n\n",
            "[[module Pages limit=\"5\"]]\n\n",
            "LITERAL_END",
        ),
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": holder_slug,
            "details": {"compiled": true},
        }),
    )
    .expect("Pages holder should exist after creation");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    for expected in [
        "PAGES_START",
        r#"<div class="list-pages-box">"#,
        r#"<div class="list-pages-item">"#,
        r#"<a href="/fixture-pages-aardvark">!! Fixture Pages aardvark</a>"#,
        r#"<a href="/fixture-pages-holder">!! Fixture Pages Holder</a>"#,
        r#"<a href="/fixture-pages-zulu">!! Fixture Pages Zulu</a>"#,
        r#"<div class="pager">"#,
        "PAGES_END",
    ] {
        assert!(html.contains(expected), "missing {expected}: {html}");
    }
    let aardvark = html.find("fixture-pages-aardvark").expect("aardvark row");
    let holder_row = html.find("fixture-pages-holder").expect("holder row");
    let zulu = html.find("fixture-pages-zulu").expect("zulu row");
    assert!(
        aardvark < holder_row && holder_row < zulu,
        "rows must follow case-insensitive title order: {html}",
    );
    assert!(
        !html.contains("fixture-pages-private"),
        "an anonymously hidden category must not contribute a row: {html}",
    );
    let limited_section = html
        .split_once("LITERAL_START")
        .expect("limited Pages section should start")
        .1
        .split_once("LITERAL_END")
        .expect("limited Pages section should end")
        .0;
    assert_eq!(
        limited_section
            .matches(r#"class="list-pages-item""#)
            .count(),
        5,
        "limit=5 is documented and live-evidenced, so it should render five rows: {html}",
    );
    assert!(
        !html.contains("[[module Pages"),
        "all Pages invocations in this fixture should execute: {html}",
    );
    assert!(
        html.contains("AUTHORED_DOTS:x….x"),
        "authored prose must receive Wikidot typography before module output: {html}",
    );

    let template_category = "fixture-pages-template";
    CategoryService::get_or_create(runner.context(), site_id, template_category)
        .await
        .expect("Pages template category should be created");
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-pages-template:_template",
        "Fixture Pages Template",
        "TEMPLATE_START\n\n[[module Pages]]\n\n%%content%%\n\nTEMPLATE_END",
    )
    .await;
    let template_holder_slug = "fixture-pages-template:holder";
    create_listpages_test_page(
        &mut runner,
        site_id,
        template_holder_slug,
        "Fixture Pages Template Holder",
        "TEMPLATE_CONTENT",
    )
    .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-pages-new",
        "!! Fixture Pages Newly Created",
        "Created after the Pages holder.",
    )
    .await;
    let refreshed_view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": holder_slug, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let refreshed_page = match refreshed_view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found Pages view, got {other:?}"),
    };
    assert!(
        refreshed_page.contains("fixture-pages-new"),
        "a bare page view must reflect pages created after its stored render: {refreshed_page}",
    );
    let refreshed_template_view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": template_holder_slug, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let refreshed_template_page = match refreshed_template_view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found templated Pages view, got {other:?}"),
    };
    assert!(
        refreshed_template_page.contains("fixture-pages-new")
            && refreshed_template_page.contains("TEMPLATE_CONTENT"),
        "a templated bare page view must refresh its Pages index: {refreshed_template_page}",
    );

    let second_view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": holder_slug, "extra": "/p/2"},
            "locales": ["en-US", "en"],
        }),
    );
    let second_page = match second_view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found Pages view, got {other:?}"),
    };
    let second_page_index = second_page
        .split_once("PAGES_START")
        .expect("second page index should start")
        .1
        .split_once("PAGES_END")
        .expect("second page index should end")
        .0;
    assert!(
        second_page_index.contains("page 2 of ")
            && !second_page_index.contains("fixture-pages-aardvark"),
        "the real /p/2 index view must rerender the second slice: {second_page}",
    );

    let last_view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": holder_slug, "extra": "/p/999"},
            "locales": ["en-US", "en"],
        }),
    );
    let last_page = match last_view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found Pages view, got {other:?}"),
    };

    assert!(
        last_page.contains("« previous") && !last_page.contains("next »"),
        "an out-of-range request must clamp to the final page: {last_page}",
    );
    let last_page_index = last_page
        .split_once("PAGES_START")
        .expect("last page index should start")
        .1
        .split_once("PAGES_END")
        .expect("last page index should end")
        .0;
    assert!(
        !last_page_index.contains("fixture-pages-aardvark"),
        "the final page must not repeat the first page: {last_page}",
    );
}

/// Live capture (sandbox-for-codex, 2026-07-28):
/// `[[module Pages category="..."]]` filters to that category, `details="true"`
/// switches rows to a details table, `preview="true"` is ignored, `limit="0"`
/// yields an empty list, and unknown arguments are ignored.
#[tokio::test]
async fn pages_module_renders_documented_arguments_and_live_fallbacks() {
    fn section<'a>(html: &'a str, start: &str, end: &str) -> &'a str {
        html.split_once(start)
            .unwrap_or_else(|| panic!("missing section start {start:?}"))
            .1
            .split_once(end)
            .unwrap_or_else(|| panic!("missing section end {end:?}"))
            .0
    }

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let category = "fixture-pages-arguments";
    let other_category = "fixture-pages-arguments-other";
    let empty_category = "fixture-pages-arguments-empty";

    CategoryService::get_or_create(runner.context(), site_id, category)
        .await
        .expect("Pages argument category should be created");
    CategoryService::get_or_create(runner.context(), site_id, other_category)
        .await
        .expect("Pages argument other category should be created");
    CategoryService::get_or_create(runner.context(), site_id, empty_category)
        .await
        .expect("Pages argument empty category should be created");

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-pages-arguments:alpha",
        "AAA Fixture Pages Arguments Alpha",
        "Alpha source preview must not render.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-pages-arguments:zulu",
        "ZZZ Fixture Pages Arguments Zulu",
        "Zulu source preview must not render.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-pages-arguments-other:bravo",
        "MMM Fixture Pages Arguments Bravo",
        "Other category marker must not render.",
    )
    .await;

    let holder_slug = "fixture-pages-arguments-holder";
    create_listpages_test_page(
        &mut runner,
        site_id,
        holder_slug,
        "Fixture Pages Arguments Holder",
        &format!(
            concat!(
                "CATEGORY_START\n\n",
                "[[module Pages category=\"{category}\"]]\n\n",
                "CATEGORY_END\n\n",
                "DETAILS_START\n\n",
                "[[module Pages category=\"{category}\" details=\"true\"]]\n\n",
                "DETAILS_END\n\n",
                "PREVIEW_START\n\n",
                "[[module Pages category=\"{category}\" preview=\"true\"]]\n\n",
                "PREVIEW_END\n\n",
                "TITLE_DESC_LIMIT_START\n\n",
                "[[module Pages category=\"{category}\" order=\"titleDesc\" limit=\"1\"]]\n\n",
                "TITLE_DESC_LIMIT_END\n\n",
                "DATE_CREATED_DESC_START\n\n",
                "[[module Pages category=\"{category}\" order=\"dateCreatedDesc\" limit=\"2\"]]\n\n",
                "DATE_CREATED_DESC_END\n\n",
                "EMPTY_CATEGORY_START\n\n",
                "[[module Pages category=\"{empty_category}\"]]\n\n",
                "EMPTY_CATEGORY_END\n\n",
                "LIMIT_ZERO_START\n\n",
                "[[module Pages category=\"{category}\" limit=\"0\"]]\n\n",
                "LIMIT_ZERO_END\n\n",
                "UNKNOWN_ARGUMENT_START\n\n",
                "[[module Pages category=\"{category}\" frob=\"x\"]]\n\n",
                "UNKNOWN_ARGUMENT_END",
            ),
            category = category,
            empty_category = empty_category,
        ),
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": holder_slug,
            "details": {"compiled": true},
        }),
    )
    .expect("Pages argument holder should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    let category_section = section(&html, "CATEGORY_START", "CATEGORY_END");
    assert!(
        category_section.contains("/fixture-pages-arguments:alpha")
            && category_section.contains("/fixture-pages-arguments:zulu"),
        "category-filtered Pages output should include same-category rows:\n{html}",
    );
    assert!(
        !category_section.contains("fixture-pages-arguments-other:bravo"),
        "category-filtered Pages output must exclude other categories:\n{html}",
    );

    let details_section = section(&html, "DETAILS_START", "DETAILS_END");
    for expected in [
        r#"<td class="title"><a href="/fixture-pages-arguments:alpha">AAA Fixture Pages Arguments Alpha</a></td>"#,
        r#"<td class="last-mod-by">"#,
        r#"<td class="revision-no">rev. "#,
        r#"<td class="last-mod-date">"#,
    ] {
        assert!(
            details_section.contains(expected),
            "details=true should render Wikidot's details table cell {expected:?}:\n{html}",
        );
    }

    let preview_section = section(&html, "PREVIEW_START", "PREVIEW_END");
    assert!(
        preview_section.contains("/fixture-pages-arguments:alpha")
            && !preview_section.contains("Alpha source preview must not render"),
        "preview=true is ignored by live Wikidot and should not render source previews:\n{html}",
    );

    let title_desc_limit =
        section(&html, "TITLE_DESC_LIMIT_START", "TITLE_DESC_LIMIT_END");
    assert!(
        title_desc_limit.contains("/fixture-pages-arguments:zulu")
            && !title_desc_limit.contains("/fixture-pages-arguments:alpha"),
        "titleDesc with limit=1 should keep only the descending-title first row:\n{html}",
    );

    let created_desc = section(&html, "DATE_CREATED_DESC_START", "DATE_CREATED_DESC_END");
    let zulu = created_desc
        .find("/fixture-pages-arguments:zulu")
        .expect("dateCreatedDesc should include zulu");
    let alpha = created_desc
        .find("/fixture-pages-arguments:alpha")
        .expect("dateCreatedDesc should include alpha");
    assert!(
        zulu < alpha,
        "dateCreatedDesc should place the later-created zulu row before alpha:\n{html}",
    );

    for (start, end, label) in [
        (
            "EMPTY_CATEGORY_START",
            "EMPTY_CATEGORY_END",
            "empty category",
        ),
        ("LIMIT_ZERO_START", "LIMIT_ZERO_END", "limit=0"),
    ] {
        let section = section(&html, start, end);
        assert!(
            section.contains(r#"<div class="list-pages-box">"#)
                && !section.contains(r#"class="list-pages-item""#),
            "{label} should render an empty list-pages-box:\n{html}",
        );
    }

    let unknown_argument =
        section(&html, "UNKNOWN_ARGUMENT_START", "UNKNOWN_ARGUMENT_END");
    assert!(
        unknown_argument.contains("/fixture-pages-arguments:alpha")
            && unknown_argument.contains("/fixture-pages-arguments:zulu"),
        "live Wikidot ignores unknown Pages arguments while applying recognized ones:\n{html}",
    );

    assert!(
        !html.contains("[[module Pages"),
        "all captured Pages argument shapes should execute, not remain literal:\n{html}",
    );
}

/// Live capture (sandbox-for-codex, 2026-07-28): `[[module ChildPages]]`
/// emits `div.child-pages-block > ul` for the current page's children, sorted
/// alphabetically by title. The live module includes child pages from any
/// category, includes underscore-prefixed hidden pages, ignores unknown
/// arguments, and emits no wrapper at all for an empty child set.
#[tokio::test]
async fn childpages_module_renders_live_child_list_and_empty_state() {
    fn section<'a>(html: &'a str, start: &str, end: &str) -> &'a str {
        html.split_once(start)
            .unwrap_or_else(|| panic!("missing section start {start:?}"))
            .1
            .split_once(end)
            .unwrap_or_else(|| panic!("missing section end {end:?}"))
            .0
    }

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let parent_slug = "fixture-childpages:parent";
    let empty_slug = "fixture-childpages:empty";
    let private_category = "fixture-childpages-private";

    CategoryService::get_or_create(runner.context(), site_id, "fixture-childpages")
        .await
        .expect("ChildPages fixture category should be created");
    CategoryService::get_or_create(runner.context(), site_id, "fixture-childpages-other")
        .await
        .expect("ChildPages fixture other category should be created");
    make_listpages_test_category_admin_only(&runner, site_id, private_category).await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        parent_slug,
        "Fixture ChildPages Parent",
        concat!(
            "CHILDPAGES_START\n\n",
            "[[module ChildPages]]\n\n",
            "CHILDPAGES_END\n\n",
            "UNKNOWN_START\n\n",
            "[[module ChildPages foo=\"bar\"]]\n\n",
            "UNKNOWN_END",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        empty_slug,
        "Fixture ChildPages Empty",
        "EMPTY_START\n\n[[module ChildPages]]\n\nEMPTY_END",
    )
    .await;

    for (slug, title) in [
        ("fixture-childpages:zulu", "Zulu ChildPages Child"),
        ("fixture-childpages:alpha", "alpha ChildPages Child"),
        (
            "fixture-childpages-other:bravo",
            "Bravo Other Category Child",
        ),
        ("fixture-childpages:_hidden", "Hidden ChildPages Child"),
        (
            "fixture-childpages-private:secret",
            "Secret ChildPages Child",
        ),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, "Child body.")
            .await;
        set_listpages_test_parent(&mut runner, site_id, slug, parent_slug).await;
    }

    let parent = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": parent_slug}),
    )
    .expect("ChildPages parent holder should exist before rerender");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": parent.page_category_id,
            "page_id": parent.page_id,
        }),
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": parent_slug,
            "details": {"compiled": true},
        }),
    )
    .expect("ChildPages parent holder should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    let childpages_section = section(&html, "CHILDPAGES_START", "CHILDPAGES_END");
    for expected in [
        r#"<div class="child-pages-block">"#,
        "<ul>",
        r#"<li><a href="/fixture-childpages:alpha">alpha ChildPages Child</a></li>"#,
        r#"<li><a href="/fixture-childpages-other:bravo">Bravo Other Category Child</a></li>"#,
        r#"<li><a href="/fixture-childpages:_hidden">Hidden ChildPages Child</a></li>"#,
        r#"<li><a href="/fixture-childpages:zulu">Zulu ChildPages Child</a></li>"#,
    ] {
        assert!(
            childpages_section.contains(expected),
            "ChildPages output missing {expected:?}:\n{html}",
        );
    }
    assert!(
        !childpages_section.contains("fixture-childpages-private:secret"),
        "ChildPages must honor anonymous view permissions:\n{html}",
    );
    let alpha = childpages_section
        .find("fixture-childpages:alpha")
        .expect("alpha child row");
    let bravo = childpages_section
        .find("fixture-childpages-other:bravo")
        .expect("bravo child row");
    let hidden = childpages_section
        .find("fixture-childpages:_hidden")
        .expect("hidden child row");
    let zulu = childpages_section
        .find("fixture-childpages:zulu")
        .expect("zulu child row");
    assert!(
        alpha < bravo && bravo < hidden && hidden < zulu,
        "ChildPages rows must follow live title order:\n{html}",
    );

    let unknown_section = section(&html, "UNKNOWN_START", "UNKNOWN_END");
    assert!(
        unknown_section.contains(r#"<div class="child-pages-block">"#)
            && unknown_section.contains("fixture-childpages:alpha")
            && !unknown_section.contains("[[module ChildPages"),
        "live Wikidot ignores unknown ChildPages arguments while rendering children:\n{html}",
    );

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-childpages:late",
        "Late ChildPages Child",
        "Late child body.",
    )
    .await;
    set_listpages_test_parent(
        &mut runner,
        site_id,
        "fixture-childpages:late",
        parent_slug,
    )
    .await;
    let runtime_view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": parent_slug, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let runtime_html = match runtime_view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found ChildPages page view, got {other:?}"),
    };
    assert!(
        runtime_html.contains(
            r#"<li><a href="/fixture-childpages:late">Late ChildPages Child</a></li>"#,
        ),
        "ChildPages page views must evaluate against current parent state without a saved-page rerender:\n{runtime_html}",
    );

    let empty = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": empty_slug,
            "details": {"compiled": true},
        }),
    )
    .expect("ChildPages empty holder should exist");
    let empty_html = empty
        .compiled_body_html
        .expect("compiled body should be included in page_get details");
    let empty_section = section(&empty_html, "EMPTY_START", "EMPTY_END");
    assert!(
        !empty_section.contains("child-pages-block")
            && !empty_section.contains("[[module ChildPages"),
        "empty ChildPages renders no wrapper or literal module:\n{empty_html}",
    );
}

/// Live anonymous PagePreview capture (2026-07-31), case IDs
/// `module-own-line-nextpage` and `module-own-line-previouspage`: without a
/// current page, both modules emit `<div class="error-block">Invalid range
/// argument.</div>`.
#[tokio::test]
async fn nextpreviouspage_preview_without_page_context_renders_live_range_error() {
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
    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "NextPreviousPage no-context preview",
            "wikitext": concat!(
                "NEXT_START\n[[module NextPage]]\nNEXT_END\n",
                "PREVIOUS_START\n[[module PreviousPage]]\nPREVIOUS_END",
            ),
        }),
    );

    assert_eq!(
        preview.body.matches("Invalid range argument.").count(),
        2,
        "live PagePreview returns the same range error for NextPage and PreviousPage without a current page:\n{}",
        preview.body,
    );
    assert_eq!(
        preview.body.matches(r#"<div class="error-block">"#).count(),
        2,
        "each no-context module should render one live error block:\n{}",
        preview.body,
    );
    assert!(
        preview.body.contains("NEXT_START")
            && preview.body.contains("NEXT_END")
            && preview.body.contains("PREVIOUS_START")
            && preview.body.contains("PREVIOUS_END"),
        "range errors should remain in authored source order:\n{}",
        preview.body,
    );
    assert!(
        !preview.body.contains("No such module")
            && !preview.body.contains("[[module NextPage")
            && !preview.body.contains("[[module PreviousPage"),
        "recognized no-context modules must not fall through to unknown-module output:\n{}",
        preview.body,
    );
}

/// Live capture (sandbox-for-codex, 2026-07-28): `NextPage` and
/// `PreviousPage` use the ListPages wrapper and template variables, default
/// to creation-date adjacency, and have a legacy title-mode quirk where
/// `PreviousPage by="title"` returns the current page itself.
#[tokio::test]
async fn nextpreviouspage_module_renders_live_selection_templates_and_runtime_updates() {
    fn section<'a>(html: &'a str, start: &str, end: &str) -> &'a str {
        html.split_once(start)
            .unwrap_or_else(|| panic!("missing section start {start:?}"))
            .1
            .split_once(end)
            .unwrap_or_else(|| panic!("missing section end {end:?}"))
            .0
    }

    fn assert_inline_module_diagnostics(parser_errors: &[ftml::parsing::ParseError]) {
        let expected = [
            (Token::Identifier, ParseErrorKind::RuleFailed),
            (Token::LeftBlock, ParseErrorKind::NoRulesMatch),
            (Token::RightBlock, ParseErrorKind::NoRulesMatch),
        ];
        assert_eq!(parser_errors.len(), expected.len() * 4);
        for group in parser_errors.chunks_exact(expected.len()) {
            assert!(
                group
                    .iter()
                    .zip(expected)
                    .all(|(error, (token, kind))| error.token() == token
                        && error.kind() == kind),
                "each intentionally literal inline module should retain its three parser diagnostics: {parser_errors:?}",
            );
        }
    }

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let category = "fixture-nextpreviouspage";
    let required_tag = "fixture-nextpreviouspage-required";
    let shared_tag = "fixture-nextpreviouspage-shared";
    const IMPORT_RUN_ID: i64 = 944_006;
    const CREATOR_ID: i64 = 10_382_659;
    const CREATOR_NAME: &str = "voted-fated-smuggler";
    const CREATOR_SLUG: &str = "voted-fated-smuggler";

    CategoryService::get_or_create(runner.context(), site_id, category)
        .await
        .expect("NextPreviousPage fixture category should be created");

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-nextpreviouspage:alpha",
        "Alpha NextPreviousPage",
        "Alpha body.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-nextpreviouspage:bravo",
        "Bravo NextPreviousPage",
        "Bravo body.",
    )
    .await;
    let bravo_page_id =
        listpages_test_page_id(&runner, site_id, "fixture-nextpreviouspage:bravo").await;
    create_listpages_test_import_run(&runner, site_id, IMPORT_RUN_ID, 1).await;
    mark_imported_page_with_author_snapshot(
        &runner,
        site_id,
        IMPORT_RUN_ID,
        (
            bravo_page_id,
            "fixture-nextpreviouspage:bravo",
            944_006,
            CREATOR_NAME,
        ),
    )
    .await;
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "UPDATE wikidot_page_snapshot \
             SET meta_json = jsonb_build_object( \
                 'created_by_id', $1::bigint, \
                 'created_by_unix', $2::text \
             ) \
             WHERE page_id = $3",
            [
                Value::from(CREATOR_ID),
                Value::from(CREATOR_SLUG),
                Value::from(bravo_page_id),
            ],
        ))
        .await
        .expect("NextPreviousPage imported author provenance should be attached");
    let holder_slug = "fixture-nextpreviouspage:charlie";
    let holder_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        holder_slug,
        "Charlie NextPreviousPage",
        "placeholder",
    )
    .await;
    let delta_slug = "fixture-nextpreviouspage:delta";
    let delta_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        delta_slug,
        "Delta NextPreviousPage",
        "Delta body.",
    )
    .await;
    let delta_revision = set_listpages_test_tags(
        &mut runner,
        site_id,
        delta_slug,
        delta_revision,
        &[required_tag, shared_tag],
    )
    .await;

    let holder_source = format!(
        concat!(
            "NEXTPREV_START\n\n",
            "TITLE_PREV_START\n",
            "[[module PreviousPage category=\"{category}\" by=\"title\"]]\n",
            "PREV_TITLE=%%linked_title%%|%%title%%|%%name%%|%%fullname%%\n",
            "[[/module]]\n",
            "TITLE_PREV_END\n\n",
            "TITLE_NEXT_START\n",
            "[[module NextPage category=\"{category}\" by=\"title\"]]\n",
            "NEXT_TITLE=%%linked_title%%|%%title%%|%%name%%|%%fullname%%\n",
            "[[/module]]\n",
            "TITLE_NEXT_END\n\n",
            "DATE_PREV_START\n",
            "[[module PreviousPage category=\"{category}\"]]\n",
            "PREV_DATE=%%linked_title%%|%%title%%|%%name%%|%%fullname%%\n",
            "[[/module]]\n",
            "DATE_PREV_END\n\n",
            "DATE_NEXT_START\n",
            "[[module NextPage category=\"{category}\"]]\n",
            "NEXT_DATE=%%linked_title%%|%%title%%|%%name%%|%%fullname%%\n",
            "[[/module]]\n",
            "DATE_NEXT_END\n\n",
            "UNKNOWN_START\n",
            "[[module NextPage category=\"{category}\" by=\"title\" foo=\"bar\"]]\n",
            "UNKNOWN=%%linked_title%%\n",
            "[[/module]]\n",
            "UNKNOWN_END\n\n",
            "TAG_REQUIRED_START\n",
            "[[module NextPage category=\"{category}\" by=\"title\" tags=\"+{required_tag}\"]]\n",
            "TAG_REQUIRED=%%linked_title%%|%%fullname%%\n",
            "[[/module]]\n",
            "TAG_REQUIRED_END\n\n",
            "TAG_EQUALS_START\n",
            "[[module NextPage category=\"{category}\" by=\"title\" tags=\"=\"]]\n",
            "TAG_EQUALS=%%linked_title%%|%%fullname%%\n",
            "[[/module]]\n",
            "TAG_EQUALS_END\n\n",
            "DEFAULT_START\n",
            "[[module PreviousPage category=\"{category}\"]]\n",
            "DEFAULT_END\n\n",
            "INLINE_START\n",
            "start-[[module NextPage]]-middle\n",
            "start-[[module PreviousPage]]-middle\n",
            "prefix-[[module NextPage]]-suffix\n",
            "  [[module PreviousPage]]\n",
            "INLINE_END\n\n",
            "NEXTPREV_END",
        ),
        category = category,
        required_tag = required_tag,
    );
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(holder_slug)),
    );
    let edited_holder = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": holder_slug,
            "last_revision_id": holder_revision,
            "wikitext": holder_source,
            "revision_comments": "set NextPreviousPage fixture source",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("NextPreviousPage holder edit should succeed");
    let parser_errors = edited_holder
        .parser_errors
        .as_ref()
        .expect("NextPreviousPage source edit should return parser diagnostics");
    assert_inline_module_diagnostics(parser_errors);
    let holder_revision = edited_holder.revision_id;
    let tagged_holder = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": holder_slug,
            "last_revision_id": holder_revision,
            "revision_comments": "set NextPreviousPage fixture tags",
            "user_id": ADMIN_USER_ID,
            "tags": [shared_tag],
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("NextPreviousPage tag edit should create a revision");
    let parser_errors = tagged_holder
        .parser_errors
        .expect("NextPreviousPage tag edit should return parser diagnostics");
    assert_inline_module_diagnostics(&parser_errors);

    let last_source = format!(
        concat!(
            "LAST_NEXT_START\n",
            "[[module NextPage category=\"{category}\" by=\"title\"]]\n",
            "LAST_NEXT=%%linked_title%%\n",
            "[[/module]]\n",
            "LAST_NEXT_END",
        ),
        category = category,
    );
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(delta_slug)),
    );
    run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": delta_slug,
            "last_revision_id": delta_revision,
            "wikitext": last_source,
            "revision_comments": "set NextPreviousPage last-page fixture source",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("NextPreviousPage last page edit should succeed");

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": holder_slug,
            "details": {"compiled": true},
        }),
    )
    .expect("NextPreviousPage holder should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    let title_prev = section(&html, "TITLE_PREV_START", "TITLE_PREV_END");
    assert!(
        title_prev.contains(r#"<div class="list-pages-box">"#)
            && title_prev.contains(r#"<div class="list-pages-item">"#)
            && title_prev.contains(
                r#"PREV_TITLE=<a href="/fixture-nextpreviouspage:charlie">Charlie NextPreviousPage</a>|Charlie NextPreviousPage|charlie|fixture-nextpreviouspage:charlie"#,
            ),
        "PreviousPage by=title must preserve live's inclusive current-page quirk and ListPages wrapper:\n{html}",
    );

    let title_next = section(&html, "TITLE_NEXT_START", "TITLE_NEXT_END");
    assert!(
        title_next.contains(
            r#"NEXT_TITLE=<a href="/fixture-nextpreviouspage:delta">Delta NextPreviousPage</a>|Delta NextPreviousPage|delta|fixture-nextpreviouspage:delta"#,
        ),
        "NextPage by=title should select the next title row:\n{html}",
    );

    let date_prev = section(&html, "DATE_PREV_START", "DATE_PREV_END");
    let date_next = section(&html, "DATE_NEXT_START", "DATE_NEXT_END");
    assert!(
        date_prev.contains("/fixture-nextpreviouspage:bravo")
            && date_next.contains("/fixture-nextpreviouspage:delta"),
        "date-mode NextPreviousPage should select creation-date neighbors:\n{html}",
    );

    let unknown = section(&html, "UNKNOWN_START", "UNKNOWN_END");
    assert!(
        unknown.contains(r#"UNKNOWN=<a href="/fixture-nextpreviouspage:delta">Delta NextPreviousPage</a>"#)
            && !unknown.contains("[[module NextPage"),
        "live Wikidot ignores unknown NextPage arguments while applying recognized ones:\n{html}",
    );

    let tag_required = section(&html, "TAG_REQUIRED_START", "TAG_REQUIRED_END");
    let tag_equals = section(&html, "TAG_EQUALS_START", "TAG_EQUALS_END");
    assert!(
        tag_required.contains("/fixture-nextpreviouspage:delta")
            && tag_equals.contains("/fixture-nextpreviouspage:delta"),
        "NextPreviousPage tag selectors should filter candidates while using the current page as the position anchor:\n{html}",
    );
    let default = section(&html, "DEFAULT_START", "DEFAULT_END");
    let expected_creator_href =
        format!(r#"href="http://www.wikidot.com/user:info/{CREATOR_SLUG}""#);
    let expected_avatar_src = format!(
        "src=\"http://www.wikidot.com/avatar.php?userid={CREATOR_ID}&amp;amp;size=small&amp;amp;timestamp="
    );
    let expected_karma_style = format!(
        r#"style="background-image:url(http://www.wikidot.com/userkarma.php?u={CREATOR_ID})""#
    );
    for expected in [
        r#"<div class="list-pages-box">"#,
        r#"<div class="list-pages-item">"#,
        r#"<h1><span><a href="/fixture-nextpreviouspage:bravo">Bravo NextPreviousPage</a></span></h1>"#,
        r#"<p>by <span class="printuser avatarhover">"#,
        expected_creator_href.as_str(),
        expected_avatar_src.as_str(),
        expected_karma_style.as_str(),
        r#"<span class="odate time_"#,
        "Bravo body.",
    ] {
        assert!(
            default.contains(expected),
            "bare PreviousPage must preserve the shared ListPages default title, local author, date, and body DOM: {expected:?}\n{default}",
        );
    }
    assert!(
        !default.contains("data-wikijump-compat-"),
        "generated PreviousPage DOM must not expose internal compatibility markers:\n{default}",
    );
    let inline = section(&html, "INLINE_START", "INLINE_END");
    assert!(
        inline.contains("start-[[module NextPage]]-middle")
            && inline.contains("start-[[module PreviousPage]]-middle")
            && inline.contains("prefix-[[module NextPage]]-suffix")
            && inline.contains("[[module PreviousPage]]"),
        "inline NextPage and PreviousPage invocations must remain literal:\n{inline}",
    );
    assert_eq!(html.matches("[[module NextPage]]").count(), 2);
    assert_eq!(html.matches("[[module PreviousPage]]").count(), 2);

    let last = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": delta_slug,
            "details": {"compiled": true},
        }),
    )
    .expect("NextPreviousPage last page should exist");
    let last_html = last
        .compiled_body_html
        .expect("compiled body should be included in page_get details");
    let last_next = section(&last_html, "LAST_NEXT_START", "LAST_NEXT_END");
    assert!(
        last_next.contains(r#"<div class="list-pages-box">"#)
            && !last_next.contains(r#"class="list-pages-item""#)
            && !last_next.contains("[[module NextPage"),
        "an empty NextPage result should render an empty list-pages-box:\n{last_html}",
    );

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-nextpreviouspage:cyan",
        "Cyan Runtime NextPreviousPage",
        "Cyan body.",
    )
    .await;
    let runtime_view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": holder_slug, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let runtime_html = match runtime_view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found NextPreviousPage page view, got {other:?}"),
    };
    let runtime_title_next = section(&runtime_html, "TITLE_NEXT_START", "TITLE_NEXT_END");
    assert!(
        runtime_title_next.contains("/fixture-nextpreviouspage:cyan")
            && !runtime_title_next.contains("/fixture-nextpreviouspage:delta"),
        "NextPreviousPage page views must evaluate against current title state without a saved-page rerender:\n{runtime_html}",
    );
}

async fn make_nextprevious_test_category_member_only(
    runner: &TestRunner,
    site_id: i64,
    category_slug: &str,
    user_id: i64,
) {
    let category_id =
        CategoryService::get_or_create(runner.context(), site_id, category_slug)
            .await
            .expect("private NextPreviousPage category should be created")
            .category_id;
    let role = RoleService::create(
        runner.context(),
        InternalCreateRoleInput {
            site_id,
            name: format!("{category_slug}-viewer"),
            description: None,
            is_virtual: false,
            parent_role_id: None,
            creating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("private NextPreviousPage role should be created");
    PermissionService::update_permissions_for_role(
        runner.context(),
        UpdateRolePermissionsInput {
            site_id,
            role_reference: Reference::Id(role.role_id),
            new_permissions: vec![Permission {
                resource_type: Resource::Page,
                resource_category: Some(Reference::Id(category_id)),
                action: Action::View,
            }],
            cascade_removals: false,
            updating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("private NextPreviousPage role permissions should be updated");
    RoleService::grant_role_to_user(
        runner.context(),
        GrantUserRoleInput {
            site_id,
            user_id,
            role_id: role.role_id,
            assigning_user_id: SYSTEM_USER_ID,
            expires_at: None,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("member should receive the private NextPreviousPage role");
}

/// Live capture (sandbox-for-codex, 2026-09-14): when two candidate pages
/// share a title, `NextPage by="title"` selects the first-created page even
/// when its fullname sorts later, `PreviousPage by="title"` keeps its
/// inclusive current-row quirk, repeated reads are stable, and the authorized
/// PagePreview request resolves the same selection as the saved render.
#[tokio::test]
async fn nextpreviouspage_title_tie_uses_creation_order_and_previous_stays_inclusive() {
    fn section<'a>(html: &'a str, start: &str, end: &str) -> &'a str {
        html.split_once(start)
            .unwrap_or_else(|| panic!("missing NextPreviousPage section start {start:?}"))
            .1
            .split_once(end)
            .unwrap_or_else(|| panic!("missing NextPreviousPage section end {end:?}"))
            .0
    }

    const CATEGORY: &str = "fixture-npp-title-tie";
    const HOLDER_SLUG: &str = "fixture-npp-title-tie:mm-holder";
    const PREV_HOLDER_SLUG: &str = "fixture-npp-title-tie:oo-holder";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (slug, title, created_seconds) in [
        (HOLDER_SLUG, "MM tie holder", 1_i64),
        ("fixture-npp-title-tie:zz-tie", "NN tie", 2_i64),
        ("fixture-npp-title-tie:aa-tie", "NN tie", 3_i64),
        (PREV_HOLDER_SLUG, "OO prev tie holder", 4_i64),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, "Tie body.").await;
        set_listpages_test_created_at(
            &runner,
            site_id,
            slug,
            OffsetDateTime::UNIX_EPOCH + Duration::seconds(created_seconds),
        )
        .await;
    }

    let source = format!(
        concat!(
            "TIE_PREV_START\n",
            "[[module PreviousPage category=\"{category}\" by=\"title\"]]\n",
            "PREV=%%fullname%%|%%title%%\n",
            "[[/module]]\n",
            "TIE_PREV_END\n",
            "TIE_NEXT_START\n",
            "[[module NextPage category=\"{category}\" by=\"title\"]]\n",
            "NEXT=%%fullname%%|%%title%%\n",
            "[[/module]]\n",
            "TIE_NEXT_END",
        ),
        category = CATEGORY,
    );
    let holder = PageTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page::Column::SiteId.eq(site_id))
                .add(page::Column::Slug.eq(HOLDER_SLUG)),
        )
        .one(runner.context().transaction())
        .await
        .expect("NextPreviousPage tie holder lookup should succeed")
        .expect("NextPreviousPage tie holder should exist");
    let page_info = PageInfo {
        page: Cow::Borrowed(HOLDER_SLUG),
        category: Some(Cow::Borrowed(CATEGORY)),
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("MM tie holder"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let render = |viewer_user_id| {
        RenderService::render_page_for_viewer(
            runner.context(),
            source.clone(),
            &page_info,
            Layout::Wikidot,
            PageId {
                site_id,
                category_id: holder.page_category_id,
                page_id: holder.page_id,
            },
            viewer_user_id,
            UrlArguments::default(),
        )
    };

    let anonymous = render(None)
        .await
        .expect("NextPreviousPage tie render should succeed")
        .html_output
        .body;
    let tie_prev = section(&anonymous, "TIE_PREV_START", "TIE_PREV_END");
    assert!(
        tie_prev.contains(&format!("PREV={HOLDER_SLUG}|MM tie holder")),
        "PreviousPage by=title must keep the inclusive current-row quirk:\n{anonymous}",
    );
    let tie_next = section(&anonymous, "TIE_NEXT_START", "TIE_NEXT_END");
    assert!(
        tie_next.contains("NEXT=fixture-npp-title-tie:zz-tie|NN tie"),
        "NextPage by=title must select the first-created title tie even though its fullname sorts later:\n{anonymous}",
    );

    let repeated = render(None)
        .await
        .expect("repeated NextPreviousPage tie render should succeed")
        .html_output
        .body;
    assert_eq!(
        repeated, anonymous,
        "title-tie selection must be stable across repeated reads",
    );

    let prev_holder = PageTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page::Column::SiteId.eq(site_id))
                .add(page::Column::Slug.eq(PREV_HOLDER_SLUG)),
        )
        .one(runner.context().transaction())
        .await
        .expect("NextPreviousPage previous tie holder lookup should succeed")
        .expect("NextPreviousPage previous tie holder should exist");
    let prev_page_info = PageInfo {
        page: Cow::Borrowed(PREV_HOLDER_SLUG),
        category: Some(Cow::Borrowed(CATEGORY)),
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("OO prev tie holder"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let prev_holder_html = RenderService::render_page_for_viewer(
        runner.context(),
        source.clone(),
        &prev_page_info,
        Layout::Wikidot,
        PageId {
            site_id,
            category_id: prev_holder.page_category_id,
            page_id: prev_holder.page_id,
        },
        None,
        UrlArguments::default(),
    )
    .await
    .expect("NextPreviousPage previous tie holder render should succeed")
    .html_output
    .body;
    let prev_holder_prev = section(&prev_holder_html, "TIE_PREV_START", "TIE_PREV_END");
    assert!(
        prev_holder_prev.contains(&format!("PREV={PREV_HOLDER_SLUG}|OO prev tie holder")),
        "PreviousPage by=title from the later holder must still return itself:\n{prev_holder_html}",
    );

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(holder.page_id)),
    });
    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "MM tie holder",
            "wikitext": source,
        }),
    );
    let preview_next = section(&preview.body, "TIE_NEXT_START", "TIE_NEXT_END");
    assert!(
        preview_next.contains("NEXT=fixture-npp-title-tie:zz-tie|NN tie"),
        "the authorized PagePreview must resolve the same title-tie selection as the saved render:\n{}",
        preview.body,
    );
}

/// Live capture (sandbox-for-codex, 2026-09-14): `NextPage` and
/// `PreviousPage` name a private adjacent page in their module output for
/// anonymous, non-member, and authorized readers, even though a direct read
/// of that page still enforces the category view permission.
#[tokio::test]
async fn nextpreviouspage_names_private_adjacent_pages_for_every_viewer() {
    fn section<'a>(html: &'a str, start: &str, end: &str) -> &'a str {
        html.split_once(start)
            .unwrap_or_else(|| panic!("missing NextPreviousPage section start {start:?}"))
            .1
            .split_once(end)
            .unwrap_or_else(|| panic!("missing NextPreviousPage section end {end:?}"))
            .0
    }

    const PUBLIC_CATEGORY: &str = "fixture-npp-adj";
    const PRIVATE_CATEGORY: &str = "fixture-npp-adj-priv";
    const HOLDER_SLUG: &str = "fixture-npp-adj:holder";
    const PRIVATE_PREV_SLUG: &str = "fixture-npp-adj-priv:priv-prev";
    const PRIVATE_NEXT_SLUG: &str = "fixture-npp-adj-priv:priv-next";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    make_nextprevious_test_category_member_only(
        &runner,
        site_id,
        PRIVATE_CATEGORY,
        SAMPLE_USER_ID,
    )
    .await;

    for (slug, title, created_seconds) in [
        ("fixture-npp-adj:pub-prev", "AA public previous", 1_i64),
        (PRIVATE_PREV_SLUG, "BB private previous", 2_i64),
        (HOLDER_SLUG, "CC holder", 3_i64),
        (PRIVATE_NEXT_SLUG, "DD private next", 4_i64),
        ("fixture-npp-adj:pub-next", "EE public next", 5_i64),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, "Adjacent body.")
            .await;
        set_listpages_test_created_at(
            &runner,
            site_id,
            slug,
            OffsetDateTime::UNIX_EPOCH + Duration::seconds(created_seconds),
        )
        .await;
    }

    let private_prev = PageTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page::Column::SiteId.eq(site_id))
                .add(page::Column::Slug.eq(PRIVATE_PREV_SLUG)),
        )
        .one(runner.context().transaction())
        .await
        .expect("private adjacent page lookup should succeed")
        .expect("private adjacent page should exist");
    for (viewer_user_id, expected) in [(None, false), (Some(SAMPLE_USER_ID), true)] {
        let can_view = PermissionService::check_user_can(
            runner.context(),
            &CheckPermissionContext {
                user_id: viewer_user_id,
                site_id,
                page_reference: Some(Reference::Id(private_prev.page_id)),
            },
            Permission {
                resource_type: Resource::Page,
                resource_category: Some(Reference::Id(private_prev.page_category_id)),
                action: Action::View,
            },
        )
        .await
        .expect("private adjacent page permission check should succeed");
        assert_eq!(
            can_view, expected,
            "direct private-page reads must keep the category view permission boundary",
        );
    }

    let source = format!(
        concat!(
            "ADJ_PREV_START\n",
            "[[module PreviousPage category=\"{categories}\"]]\n",
            "PREV=%%fullname%%|%%title%%\n",
            "[[/module]]\n",
            "ADJ_PREV_END\n",
            "ADJ_NEXT_START\n",
            "[[module NextPage category=\"{categories}\"]]\n",
            "NEXT=%%fullname%%|%%title%%\n",
            "[[/module]]\n",
            "ADJ_NEXT_END",
        ),
        categories = format!("{PUBLIC_CATEGORY},{PRIVATE_CATEGORY}"),
    );
    let holder = PageTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page::Column::SiteId.eq(site_id))
                .add(page::Column::Slug.eq(HOLDER_SLUG)),
        )
        .one(runner.context().transaction())
        .await
        .expect("private adjacency holder lookup should succeed")
        .expect("private adjacency holder should exist");
    let page_info = PageInfo {
        page: Cow::Borrowed(HOLDER_SLUG),
        category: Some(Cow::Borrowed(PUBLIC_CATEGORY)),
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("CC holder"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let render = |viewer_user_id| {
        RenderService::render_page_for_viewer(
            runner.context(),
            source.clone(),
            &page_info,
            Layout::Wikidot,
            PageId {
                site_id,
                category_id: holder.page_category_id,
                page_id: holder.page_id,
            },
            viewer_user_id,
            UrlArguments::default(),
        )
    };

    for viewer_user_id in [None, Some(SAMPLE_USER_ID)] {
        let html = render(viewer_user_id)
            .await
            .expect("private adjacency render should succeed")
            .html_output
            .body;
        let previous = section(&html, "ADJ_PREV_START", "ADJ_PREV_END");
        assert!(
            previous.contains(&format!("PREV={PRIVATE_PREV_SLUG}|BB private previous")),
            "PreviousPage must name the private adjacent page for every observed viewer:\n{html}",
        );
        let next = section(&html, "ADJ_NEXT_START", "ADJ_NEXT_END");
        assert!(
            next.contains(&format!("NEXT={PRIVATE_NEXT_SLUG}|DD private next")),
            "NextPage must name the private adjacent page for every observed viewer:\n{html}",
        );
    }
}

/// Live capture (sandbox-for-codex, 2026-07-25): `[[module PagesByTag tag="x"]]`
/// emits an anchor, an `h2`, and `div#tagged-pages-list.pages-list` holding one
/// `pages-list-item` per tagged page, ordered case-insensitively by title. A tag
/// with no pages still emits the heading and an empty list; 55 tagged pages
/// emitted all 55 rows and no pager.
#[tokio::test]
async fn pages_by_tag_module_renders_tagged_pages_ordered_by_title() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "fixture-pbt-tag";
    let holder_slug = "fixture-pbt-holder";

    // Titles chosen so byte ordering and case-insensitive ordering disagree:
    // live Wikidot put "aardvark probe" before "Zulu Probe".
    for (slug, title) in [
        ("fixture-pbt-zulu", "Zulu Probe"),
        ("fixture-pbt-aardvark", "aardvark probe"),
        ("fixture-pbt-mango", "Mango Probe"),
    ] {
        let revision_id = create_listpages_test_page(
            &mut runner,
            site_id,
            slug,
            title,
            "Tagged probe body.",
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision_id, &[tag]).await;
    }
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-pbt-untagged",
        "Fixture PBT Untagged",
        "This page carries no probe tag.",
    )
    .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        holder_slug,
        "Fixture PBT Holder",
        &format!("PBT_START\n\n[[module PagesByTag tag=\"{tag}\"]]\n\nPBT_END"),
    )
    .await;

    let holder = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": holder_slug}),
    )
    .expect("PagesByTag holder should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": holder.page_category_id,
            "page_id": holder.page_id,
        }),
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": holder_slug,
            "details": {"compiled": true},
        }),
    )
    .expect("PagesByTag holder should exist after rerender");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    for expected in [
        "PBT_START",
        r#"<a name="pages"></a>"#,
        &format!("<h2>List of pages tagged with <em>{tag}</em>:</h2>"),
        r#"<div class="pages-list" id="tagged-pages-list">"#,
        r#"<a href="/fixture-pbt-aardvark">aardvark probe</a>"#,
        r#"<a href="/fixture-pbt-mango">Mango Probe</a>"#,
        r#"<a href="/fixture-pbt-zulu">Zulu Probe</a>"#,
        "PBT_END",
    ] {
        assert!(html.contains(expected), "missing {expected}: {html}");
    }

    assert!(
        !html.contains("fixture-pbt-untagged"),
        "untagged page must not appear: {html}",
    );
    assert!(
        !html.contains(r#"<div class="pager">"#),
        "live emitted no pager for this module: {html}",
    );

    let aardvark = html.find("fixture-pbt-aardvark").expect("aardvark row");
    let mango = html.find("fixture-pbt-mango").expect("mango row");
    let zulu = html.find("fixture-pbt-zulu").expect("zulu row");
    assert!(
        aardvark < mango && mango < zulu,
        "rows must follow case-insensitive title order: {html}",
    );
}

#[tokio::test]
async fn pages_by_tag_saved_page_views_follow_current_tagged_page_state() {
    async fn load_public_view(
        runner: &TestRunner,
        site_id: i64,
        holder_slug: &str,
    ) -> String {
        match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": null,
                "route": {"slug": holder_slug, "extra": ""},
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                compiled_body_html, ..
            } => compiled_body_html,
            other => panic!("expected found PagesByTag page view, got {other:?}"),
        }
    }

    fn assert_only_alpha(html: &str, alpha_slug: &str, beta_slug: &str) {
        let expected =
            format!(r#"<a href="/{alpha_slug}">Alpha Current PagesByTag Match</a>"#,);
        assert!(
            html.contains(&expected),
            "the current matching alpha row should be selected:\n{html}",
        );
        assert!(
            !html.contains(beta_slug) && html.matches("pages-list-item").count() == 1,
            "nonmatching pages must not change the selected row:\n{html}",
        );
    }

    fn assert_empty(html: &str, alpha_slug: &str, beta_slug: &str) {
        assert!(
            !html.contains(alpha_slug)
                && !html.contains(beta_slug)
                && !html.contains("pages-list-item"),
            "the current selector should have no matching rows:\n{html}",
        );
    }

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "fixture-pbt-current-tag";
    let other_tag = "fixture-pbt-current-other-tag";
    let category = "fixture-pbt-current";
    let holder_slug = "fixture-pbt-current-holder";
    let alpha_slug = "fixture-pbt-current:alpha";
    let beta_slug = "fixture-pbt-current:beta";

    let beta_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        beta_slug,
        "Beta Nonmatching PagesByTag Page",
        "Beta body.",
    )
    .await;
    let mut beta_revision = set_listpages_test_tags(
        &mut runner,
        site_id,
        beta_slug,
        beta_revision,
        &[other_tag],
    )
    .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        holder_slug,
        "Fixture Current PagesByTag Holder",
        &format!(r#"[[module PagesByTag tag="{tag}" category="{category}"]]"#),
    )
    .await;
    let initial = load_public_view(&runner, site_id, holder_slug).await;
    assert_empty(&initial, alpha_slug, beta_slug);
    let stored_before_matching_page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": holder_slug,
            "details": {"compiled": true},
        }),
    )
    .expect("PagesByTag holder should have stored compiled details")
    .compiled_body_html
    .expect("PagesByTag holder should have stored compiled HTML");
    assert_empty(&stored_before_matching_page, alpha_slug, beta_slug);

    let alpha_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        alpha_slug,
        "Alpha Current PagesByTag Match",
        "Alpha body.",
    )
    .await;
    let mut alpha_revision =
        set_listpages_test_tags(&mut runner, site_id, alpha_slug, alpha_revision, &[tag])
            .await;
    let alpha = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": alpha_slug}),
    )
    .expect("matching PagesByTag page should exist");
    let beta = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": beta_slug}),
    )
    .expect("nonmatching PagesByTag page should exist");

    let stored_after_matching_page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": holder_slug,
            "details": {"compiled": true},
        }),
    )
    .expect("PagesByTag holder should retain stored compiled details")
    .compiled_body_html
    .expect("PagesByTag holder should retain stored compiled HTML");
    assert_eq!(
        stored_after_matching_page, stored_before_matching_page,
        "matching page mutations must not rewrite the holder's save-time compilation",
    );
    assert_empty(&stored_after_matching_page, alpha_slug, beta_slug);

    let after_alpha = load_public_view(&runner, site_id, holder_slug).await;
    assert_only_alpha(&after_alpha, alpha_slug, beta_slug);

    beta_revision = set_listpages_test_tags(
        &mut runner,
        site_id,
        beta_slug,
        beta_revision,
        &[other_tag, "fixture-pbt-current-mutated"],
    )
    .await;
    let after_nonmatching_tag_edit =
        load_public_view(&runner, site_id, holder_slug).await;
    assert_only_alpha(&after_nonmatching_tag_edit, alpha_slug, beta_slug);

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(beta.page_id),
    );
    let moved_beta_slug = "fixture-pbt-current-other:beta";
    run_endpoint!(
        runner,
        page_move,
        json!({
            "site_id": site_id,
            "page": beta.page_id,
            "new_slug": moved_beta_slug,
            "last_revision_id": beta_revision,
            "revision_comments": "move nonmatching PagesByTag fixture page",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let after_nonmatching_move = load_public_view(&runner, site_id, holder_slug).await;
    assert_only_alpha(&after_nonmatching_move, alpha_slug, moved_beta_slug);

    alpha_revision =
        set_listpages_test_tags(&mut runner, site_id, alpha_slug, alpha_revision, &[])
            .await;
    let after_tag_removal = load_public_view(&runner, site_id, holder_slug).await;
    assert_empty(&after_tag_removal, alpha_slug, moved_beta_slug);

    alpha_revision =
        set_listpages_test_tags(&mut runner, site_id, alpha_slug, alpha_revision, &[tag])
            .await;
    let after_tag_restore = load_public_view(&runner, site_id, holder_slug).await;
    assert_only_alpha(&after_tag_restore, alpha_slug, moved_beta_slug);

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(alpha.page_id),
    );
    let moved_alpha_slug = "fixture-pbt-current-other:alpha";
    let moved_alpha = run_endpoint!(
        runner,
        page_move,
        json!({
            "site_id": site_id,
            "page": alpha.page_id,
            "new_slug": moved_alpha_slug,
            "last_revision_id": alpha_revision,
            "revision_comments": "move matching PagesByTag fixture out of category",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    alpha_revision = moved_alpha.revision_id;
    let after_matching_move = load_public_view(&runner, site_id, holder_slug).await;
    assert_empty(&after_matching_move, moved_alpha_slug, moved_beta_slug);

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(alpha.page_id),
    );
    let restored_alpha = run_endpoint!(
        runner,
        page_move,
        json!({
            "site_id": site_id,
            "page": alpha.page_id,
            "new_slug": alpha_slug,
            "last_revision_id": alpha_revision,
            "revision_comments": "restore matching PagesByTag fixture category",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    alpha_revision = restored_alpha.revision_id;
    let after_category_restore = load_public_view(&runner, site_id, holder_slug).await;
    assert_only_alpha(&after_category_restore, alpha_slug, moved_beta_slug);

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(alpha.page_id),
    );
    run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": alpha.page_id,
            "last_revision_id": alpha_revision,
            "revision_comments": "delete matching PagesByTag fixture page",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let after_delete = load_public_view(&runner, site_id, holder_slug).await;
    assert_empty(&after_delete, alpha_slug, moved_beta_slug);
}

/// A tag matching nothing still renders the heading and an empty list, while
/// argument forms with no live capture stay literal.
#[tokio::test]
async fn pages_by_tag_module_renders_empty_list_and_preserves_unevidenced_forms() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let holder_slug = "fixture-pbt-edge-holder";

    create_listpages_test_page(
        &mut runner,
        site_id,
        holder_slug,
        "Fixture PBT Edge Holder",
        concat!(
            "EMPTY_START\n\n",
            "[[module PagesByTag tag=\"fixture-pbt-nothing-has-this\"]]\n\n",
            "EMPTY_END\n\n",
            "LITERAL_START\n\n",
            "[[module PagesByTag tag=\"a\" limit=\"5\"]]\n\n",
            "LITERAL_END\n\n",
            "NOARG_START\n\n",
            "[[module PagesByTag]]\n\n",
            "NOARG_END\n\n",
            "CODE_START\n\n",
            "[[code]]\n[[module PagesByTag tag=\"a\"]]\n[[/code]]\n\n",
            "CODE_END",
        ),
    )
    .await;

    let holder = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": holder_slug}),
    )
    .expect("PagesByTag edge holder should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": holder.page_category_id,
            "page_id": holder.page_id,
        }),
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": holder_slug,
            "details": {"compiled": true},
        }),
    )
    .expect("PagesByTag edge holder should exist after rerender");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains(
            "<h2>List of pages tagged with <em>fixture-pbt-nothing-has-this</em>:</h2>",
        ),
        "empty tag still renders its heading: {html}",
    );
    assert!(
        !html.contains("pages-list-item"),
        "no tagged page exists, so no row may render: {html}",
    );
    assert!(
        html.contains("[[module PagesByTag tag=&quot;a&quot; limit=&quot;5&quot;]]"),
        "an unevidenced argument form must stay literal: {html}",
    );
    // Live renders nothing for a tagless module unless a URL argument supplies
    // the tag. A stored revision render answers no request, so it has none.
    let noarg = &html[html.find("NOARG_START").expect("noarg marker")
        ..html.find("NOARG_END").expect("noarg end marker")];
    assert!(
        !noarg.contains("tagged-pages-list") && !noarg.contains("PagesByTag"),
        "tagless module renders nothing: {noarg}",
    );
    let code = &html[html.find("CODE_START").expect("code marker")
        ..html.find("CODE_END").expect("code end marker")];
    assert!(
        code.contains("[[module PagesByTag tag=&quot;a&quot;]]")
            && !code.contains("tagged-pages-list"),
        "PagesByTag text in a code block must remain literal: {code}",
    );
}

/// A `/tag/<value>` URL argument supplies the tag for a module that carries
/// none, and a module that names its own tag ignores the URL.
///
/// Captured live on `sandbox-for-codex` on 2026-07-25 against a holder page
/// whose source is exactly `[[module PagesByTag]]`.
#[tokio::test]
async fn pages_by_tag_module_reads_the_url_tag_argument() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let url_tag = "fixture-pbt-url-tag";
    let module_tag = "fixture-pbt-module-tag";
    let holder_slug = "fixture-pbt-url-holder";

    for (slug, title, tag) in [
        ("fixture-pbt-url-page", "URL Argument Probe", url_tag),
        (
            "fixture-pbt-module-page",
            "Module Argument Probe",
            module_tag,
        ),
    ] {
        let revision_id =
            create_listpages_test_page(&mut runner, site_id, slug, title, "Probe body.")
                .await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision_id, &[tag]).await;
    }

    let page = create_listpages_test_page(
        &mut runner,
        site_id,
        holder_slug,
        "Fixture PBT URL Holder",
        "placeholder",
    )
    .await;
    let _ = page;
    let holder = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": holder_slug}),
    )
    .expect("PagesByTag URL holder should exist");

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(holder_slug))),
    });
    let page_info = PageInfo {
        page: Cow::Borrowed(holder_slug),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("Fixture PBT URL Holder"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let page_id = PageId {
        site_id,
        category_id: holder.page_category_id,
        page_id: holder.page_id,
    };

    let render = async |wikitext: &str, url_tag: Option<&str>| {
        RenderService::render_page(
            runner.context(),
            wikitext.to_owned(),
            &page_info,
            Layout::Wikidot,
            page_id,
            UrlArguments {
                tag: url_tag,
                ..UrlArguments::default()
            },
        )
        .await
        .expect("PagesByTag render should succeed")
        .html_output
        .body
    };

    let tagless = "PBT_START\n\n[[module PagesByTag]]\n\nPBT_END";
    let html = render(tagless, Some(url_tag)).await;
    assert!(
        html.contains(&format!(
            "<h2>List of pages tagged with <em>{url_tag}</em>:</h2>"
        )),
        "the URL tag supplies the heading: {html}",
    );
    assert!(
        html.contains(r#"<a href="/fixture-pbt-url-page">URL Argument Probe</a>"#),
        "the URL tag selects its tagged page: {html}",
    );
    assert!(
        !html.contains("fixture-pbt-module-page"),
        "a page carrying a different tag must not appear: {html}",
    );

    // The module's own argument answers the question, so the URL cannot
    // change which tag it lists.
    let self_tagged =
        format!("PBT_START\n\n[[module PagesByTag tag=\"{module_tag}\"]]\n\nPBT_END");
    let html = render(&self_tagged, Some(url_tag)).await;
    assert!(
        html.contains(&format!(
            "<h2>List of pages tagged with <em>{module_tag}</em>:</h2>"
        )),
        "the module argument wins over the URL: {html}",
    );
    assert!(
        !html.contains("fixture-pbt-url-page"),
        "the URL tag must not leak into a module that names its own: {html}",
    );

    // Live renders `<em></em>` and an empty list for `/holder/tag` and
    // `/holder/tag/`, which is distinct from omitting the argument.
    let html = render(tagless, Some("")).await;
    assert!(
        html.contains("<h2>List of pages tagged with <em></em>:</h2>"),
        "an empty URL tag still renders its heading: {html}",
    );
    assert!(
        !html.contains("pages-list-item"),
        "no page carries an empty tag, so no row may render: {html}",
    );

    let html = render(tagless, None).await;
    assert!(
        !html.contains("tagged-pages-list"),
        "with neither argument the module renders nothing: {html}",
    );
}

#[tokio::test]
async fn pages_by_tag_module_filters_module_and_url_categories() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "fixture-pbt-category-tag";
    let category_a = "fixture-pbt-category-a";
    let category_b = "fixture-pbt-category-b";
    let holder_slug = "fixture-pbt-category-holder";

    for (slug, title, tags) in [
        (
            format!("{category_a}:alpha"),
            "Alpha PBT Category Probe",
            vec![tag],
        ),
        (
            format!("{category_b}:bravo"),
            "Bravo PBT Category Probe",
            vec![tag],
        ),
        (
            format!("{category_a}:untagged"),
            "Untagged PBT Category Probe",
            Vec::new(),
        ),
    ] {
        let revision_id =
            create_listpages_test_page(&mut runner, site_id, &slug, title, "Probe body.")
                .await;
        if !tags.is_empty() {
            set_listpages_test_tags(&mut runner, site_id, &slug, revision_id, &tags)
                .await;
        }
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        holder_slug,
        "Fixture PBT Category Holder",
        "placeholder",
    )
    .await;
    let holder = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": holder_slug}),
    )
    .expect("PagesByTag category holder should exist");

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(holder_slug))),
    });
    let page_info = PageInfo {
        page: Cow::Borrowed(holder_slug),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("Fixture PBT Category Holder"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let page_id = PageId {
        site_id,
        category_id: holder.page_category_id,
        page_id: holder.page_id,
    };

    let render = async |wikitext: &str, url: UrlArguments<'_>| {
        RenderService::render_page(
            runner.context(),
            wikitext.to_owned(),
            &page_info,
            Layout::Wikidot,
            page_id,
            url,
        )
        .await
        .expect("PagesByTag category render should succeed")
        .html_output
        .body
    };

    let module_category =
        format!(r#"[[module PagesByTag tag="{tag}" category="{category_a}"]]"#,);
    let html = render(&module_category, UrlArguments::default()).await;
    assert!(
        html.contains(&format!(
            "List of pages tagged with <em>{tag}</em> from category <em>{category_a}</em>:"
        )),
        "a module category should appear in the live heading: {html}",
    );
    assert!(
        html.contains(
            r#"<a href="/fixture-pbt-category-a:alpha">Alpha PBT Category Probe</a>"#
        ) && !html.contains("fixture-pbt-category-b:bravo")
            && !html.contains("fixture-pbt-category-a:untagged"),
        "a module category should restrict the tag listing to that category only: {html}",
    );
    assert!(
        html.contains(&format!(
            r#"<span style="float: right">(<a href="/{holder_slug}/tag/{tag}">show from all categories</a>)</span>"#
        )),
        "a category-filtered PagesByTag render should include Wikidot's all-categories link: {html}",
    );

    let tag_with_url_category = format!(r#"[[module PagesByTag tag="{tag}"]]"#);
    let html = render(
        &tag_with_url_category,
        UrlArguments {
            category: Some(category_b),
            ..UrlArguments::default()
        },
    )
    .await;
    assert!(
        html.contains(&format!("from category <em>{category_b}</em>:"))
            && html.contains(
                r#"<a href="/fixture-pbt-category-b:bravo">Bravo PBT Category Probe</a>"#
            )
            && !html.contains("fixture-pbt-category-a:alpha"),
        "a URL category should restrict a PagesByTag module that omits category: {html}",
    );

    let category_with_url_tag =
        format!(r#"[[module PagesByTag category="{category_a}"]]"#);
    let html = render(
        &category_with_url_tag,
        UrlArguments {
            tag: Some(tag),
            ..UrlArguments::default()
        },
    )
    .await;
    assert!(
        html.contains(&format!(
            "List of pages tagged with <em>{tag}</em> from category <em>{category_a}</em>:"
        ))
            && html.contains(r#"<a href="/fixture-pbt-category-a:alpha">Alpha PBT Category Probe</a>"#)
            && !html.contains("fixture-pbt-category-b:bravo"),
        "a URL tag should compose with a module category: {html}",
    );
}

#[tokio::test]
async fn list_pages_url_tag_selector_reads_the_url_tag_argument() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let url_tag = "fixture-lp-url-tag";
    let fallback_tag = "fixture-lp-fallback-tag";
    let holder_slug = "fixture-lp-url-holder";

    for (slug, title, tag) in [
        ("fixture-lp-url-page", "LP URL Probe", url_tag),
        (
            "fixture-lp-fallback-page",
            "LP Fallback Probe",
            fallback_tag,
        ),
    ] {
        let revision_id =
            create_listpages_test_page(&mut runner, site_id, slug, title, "Probe body.")
                .await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision_id, &[tag]).await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        holder_slug,
        "Fixture LP URL Holder",
        "placeholder",
    )
    .await;
    let holder = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": holder_slug}),
    )
    .expect("ListPages URL holder should exist");

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(holder_slug))),
    });
    let page_info = PageInfo {
        page: Cow::Borrowed(holder_slug),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("Fixture LP URL Holder"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let page_id = PageId {
        site_id,
        category_id: holder.page_category_id,
        page_id: holder.page_id,
    };

    let render = async |wikitext: &str, url_tag: Option<&str>| {
        RenderService::render_page(
            runner.context(),
            wikitext.to_owned(),
            &page_info,
            Layout::Wikidot,
            page_id,
            UrlArguments {
                tag: url_tag,
                ..UrlArguments::default()
            },
        )
        .await
        .expect("ListPages render should succeed")
        .html_output
        .body
    };

    let bare = concat!(
        "[[module ListPages tags=\"@URL\" name=\"fixture-lp-*\" separate=\"no\"]]\n",
        "ROW %%name%%\n",
        "[[/module]]",
    );
    let html = render(bare, Some(url_tag)).await;
    assert!(
        html.contains("fixture-lp-url-page"),
        "the URL tag selects its tagged page: {html}",
    );
    assert!(
        !html.contains("fixture-lp-fallback-page"),
        "a page carrying a different tag must not appear: {html}",
    );

    // Live drops an unresolved `@URL` rather than matching nothing, so the
    // query widens to every page the remaining selectors allow.
    for absent in [None, Some("")] {
        let html = render(bare, absent).await;
        assert!(
            html.contains("fixture-lp-url-page")
                && html.contains("fixture-lp-fallback-page"),
            "an unresolved @URL widens instead of matching nothing: {html}",
        );
    }

    // A fallback answers only when the URL supplies nothing.
    let with_fallback = format!(
        concat!(
            "[[module ListPages tags=\"@URL|{fallback}\" name=\"fixture-lp-*\" ",
            "separate=\"no\"]]\nROW %%name%%\n[[/module]]",
        ),
        fallback = fallback_tag,
    );
    let html = render(&with_fallback, None).await;
    assert!(
        html.contains("fixture-lp-fallback-page")
            && !html.contains("fixture-lp-url-page"),
        "without a URL tag the fallback selects: {html}",
    );

    let html = render(&with_fallback, Some(url_tag)).await;
    assert!(
        html.contains("fixture-lp-url-page")
            && !html.contains("fixture-lp-fallback-page"),
        "the URL tag beats the fallback: {html}",
    );
}

#[tokio::test]
async fn list_pages_url_page_number_composes_with_the_module_offset() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let holder_slug = "fixture-lp-page-holder";
    let tag = "fixture-lp-page-tag";

    // Titles sort into a known order so the rendered slice is checkable.
    for title in ["Alpha", "Bravo", "Charlie", "Delta", "Echo"] {
        let slug = format!("fixture-lp-page-{}", title.to_lowercase());
        let revision_id =
            create_listpages_test_page(&mut runner, site_id, &slug, title, "Row body.")
                .await;
        set_listpages_test_tags(&mut runner, site_id, &slug, revision_id, &[tag]).await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        holder_slug,
        "Fixture LP Page Holder",
        "placeholder",
    )
    .await;
    let holder = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": holder_slug}),
    )
    .expect("ListPages pagination holder should exist");

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(holder_slug))),
    });
    let page_info = PageInfo {
        page: Cow::Borrowed(holder_slug),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("Fixture LP Page Holder"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let page_id = PageId {
        site_id,
        category_id: holder.page_category_id,
        page_id: holder.page_id,
    };

    let render = async |wikitext: &str, url_page: Option<u32>| {
        RenderService::render_page(
            runner.context(),
            wikitext.to_owned(),
            &page_info,
            Layout::Wikidot,
            page_id,
            UrlArguments {
                page: url_page,
                ..UrlArguments::default()
            },
        )
        .await
        .expect("ListPages pagination render should succeed")
        .html_output
        .body
    };

    // `offset="1"` drops Alpha, leaving four rows paginated two at a time.
    let source = format!(
        concat!(
            "[[module ListPages tags=\"{tag}\" name=\"fixture-lp-page-*\" ",
            "order=\"title\" separate=\"no\" perPage=\"2\" offset=\"1\"]]\n",
            "ROW %%title%%\n[[/module]]",
        ),
        tag = tag,
    );

    let rows = |html: &str| {
        ["Alpha", "Bravo", "Charlie", "Delta", "Echo"]
            .into_iter()
            .filter(|title| html.contains(&format!("ROW {title}")))
            .collect::<Vec<_>>()
    };

    let first = render(&source, None).await;
    assert_eq!(
        rows(&first),
        vec!["Bravo", "Charlie"],
        "the offset skips Alpha and the first page holds two rows: {first}",
    );
    let explicit_first = render(&source, Some(1)).await;
    assert_eq!(
        rows(&explicit_first),
        vec!["Bravo", "Charlie"],
        "/p/1 renders the same page as no argument: {explicit_first}",
    );

    // Page two skips `offset + perPage`, not `offset` alone.
    let second = render(&source, Some(2)).await;
    assert_eq!(
        rows(&second),
        vec!["Delta", "Echo"],
        "/p/2 continues after the offset: {second}",
    );

    // Four rows at two per page is two pages, counted after the offset.
    assert!(
        second.contains("page 2 of 2"),
        "the pager counts pages after the offset: {second}",
    );

    // Live clamps a number past the end to the last page.
    for beyond in [3, 99] {
        let clamped = render(&source, Some(beyond)).await;
        assert_eq!(
            rows(&clamped),
            vec!["Delta", "Echo"],
            "/p/{beyond} clamps to the last page: {clamped}",
        );
    }
}

#[test]
fn list_pages_saved_view_preserves_live_pagination_path_shapes() {
    // The fixture intentionally exercises five ListPages passes and a very
    // large offset. Keep the test's async poll stack separate from the test
    // harness's 2 MiB stack so the compatibility assertion measures rendering,
    // not the harness's stack budget.
    std::thread::Builder::new()
        .name("list-pages-pagination-test".to_owned())
        .stack_size(4 * 1024 * 1024)
        .spawn(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("ListPages pagination test runtime should build")
                .block_on(
                    list_pages_saved_view_preserves_live_pagination_path_shapes_impl(),
                );
        })
        .expect("ListPages pagination test thread should spawn")
        .join()
        .expect("ListPages pagination test thread should complete");
}

async fn setup_list_pages_saved_view_path_fixture() -> (TestRunner, i64) {
    const HOLDER: &str = "fixture-lp-path-holder";
    const A_PREFIX: &str = "fixture-lp-path-a";
    const B_PREFIX: &str = "fixture-lp-path-b";
    const OFFSET_HUGE_PREFIX: &str = "fixture-lp-path-offset-huge";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for index in 1..=5 {
        let slug = format!("{A_PREFIX}-{index}");
        create_listpages_test_page(&mut runner, site_id, &slug, &slug, "A row").await;
    }
    for index in 1..=4 {
        let slug = format!("{B_PREFIX}-{index}");
        create_listpages_test_page(&mut runner, site_id, &slug, &slug, "B row").await;
    }
    for index in 1..=23 {
        let slug = format!("{OFFSET_HUGE_PREFIX}-{index:02}");
        create_listpages_test_page(&mut runner, site_id, &slug, &slug, "Offset row")
            .await;
    }
    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER,
        "ListPages path holder",
        &format!(
            concat!(
                "A_START\n",
                "[[module ListPages name=\"{A_PREFIX}-*\" order=\"name\" separate=\"no\" perPage=\"2\"]]\n",
                "A%%index%%:%%name%%|\n",
                "[[/module]]\n",
                "A_END\n",
                "B_START\n",
                "[[module ListPages name=\"{B_PREFIX}-*\" order=\"name\" separate=\"no\" perPage=\"2\"]]\n",
                "B%%index%%:%%name%%|\n",
                "[[/module]]\n",
                "B_END\n",
                "PA_START\n",
                "[[module ListPages name=\"{A_PREFIX}-*\" order=\"name\" separate=\"no\" perPage=\"2\" urlAttrPrefix=\"a\"]]\n",
                "PA%%index%%:%%name%%|\n",
                "[[/module]]\n",
                "PA_END\n",
                "PB_START\n",
                "[[module ListPages name=\"{B_PREFIX}-*\" order=\"name\" separate=\"no\" perPage=\"2\" urlAttrPrefix=\"b\"]]\n",
                "PB%%index%%:%%name%%|\n",
                "[[/module]]\n",
                "PB_END\n",
                "OH_START\n",
                "[[module ListPages name=\"{OFFSET_HUGE_PREFIX}-*\" order=\"name\" separate=\"no\" offset=\"999999999\"]]\n",
                "OH%%index%%:%%name%%|\n",
                "[[/module]]\n",
                "OH_END",
            ),
            A_PREFIX = A_PREFIX,
            B_PREFIX = B_PREFIX,
            OFFSET_HUGE_PREFIX = OFFSET_HUGE_PREFIX,
        ),
    )
    .await;

    (runner, site_id)
}

async fn list_pages_saved_view_preserves_live_pagination_path_shapes_impl() {
    const HOLDER: &str = "fixture-lp-path-holder";
    const A_PREFIX: &str = "fixture-lp-path-a";
    const B_PREFIX: &str = "fixture-lp-path-b";
    const OFFSET_HUGE_PREFIX: &str = "fixture-lp-path-offset-huge";

    let (runner, site_id) = setup_list_pages_saved_view_path_fixture().await;

    let view = async |extra: &str| match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": HOLDER, "extra": extra},
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected a found ListPages path view, got {other:?}"),
    };
    fn section<'a>(html: &'a str, marker: &str) -> &'a str {
        let start_marker = format!("{marker}_START");
        let end_marker = format!("{marker}_END");
        let start = html
            .find(&start_marker)
            .expect("pagination fixture section should start");
        let end = html[start..]
            .find(&end_marker)
            .map(|offset| start + offset)
            .expect("pagination fixture section should end");
        &html[start..end]
    }
    let assert_row = |html: &str, marker: &str, row: &str| {
        let section = section(html, marker);
        assert!(
            section.contains(row),
            "{marker} should contain {row}:\n{section}",
        );
    };

    let first = view("").await;
    assert_row(&first, "A", &format!("A1:{A_PREFIX}-1|"));
    assert_row(&first, "B", &format!("B1:{B_PREFIX}-1|"));

    let explicit_first = view("/p/1").await;
    assert_row(&explicit_first, "A", &format!("A1:{A_PREFIX}-1|"));
    assert_row(&explicit_first, "B", &format!("B1:{B_PREFIX}-1|"));

    let second = view("/p/2").await;
    assert_row(&second, "A", &format!("A3:{A_PREFIX}-3|"));
    assert_row(&second, "B", &format!("B3:{B_PREFIX}-3|"));
    assert_row(&second, "PA", &format!("PA1:{A_PREFIX}-1|"));
    assert!(
        section(&second, "PA").contains(&format!("href=\"/{HOLDER}/p/2/a_p/2\"",)),
        "{second}",
    );

    let prefixed = view("/a_p/2").await;
    assert_row(&prefixed, "A", &format!("A1:{A_PREFIX}-1|"));
    assert_row(&prefixed, "PA", &format!("PA3:{A_PREFIX}-3|"));
    assert_row(&prefixed, "PB", &format!("PB1:{B_PREFIX}-1|"));
    assert!(
        section(&prefixed, "A").contains(&format!("href=\"/{HOLDER}/a_p/2/p/2\"",)),
        "{prefixed}",
    );

    let zero = view("/p/0").await;
    assert_row(&zero, "A", &format!("A1:{A_PREFIX}-1|"));
    assert!(
        section(&zero, "A").contains(&format!("href=\"/{HOLDER}/p/2\""))
            && !section(&zero, "A").contains("/p/0/p/2"),
        "{zero}",
    );

    for malformed in ["/p/-1", "/p/nope", "/p/abc", "/p/2.5", "/p"] {
        let html = view(malformed).await;
        assert_row(&html, "A", &format!("A1:{A_PREFIX}-1|"));
        assert!(
            section(&html, "A").contains(&format!("href=\"/{HOLDER}{malformed}/p/2\"",)),
            "{malformed} should be preserved before the appended pager argument:\n{html}",
        );
    }

    let repeated = view("/p/2/p/3").await;
    assert_row(&repeated, "A", &format!("A5:{A_PREFIX}-5|"));
    assert!(
        section(&repeated, "A").contains(&format!("href=\"/{HOLDER}/p/1/p/3\"",)),
        "live replaces the first numeric pager argument while selecting the last:\n{repeated}",
    );

    let beyond = view("/p/999").await;
    assert_row(&beyond, "A", &format!("A5:{A_PREFIX}-5|"));
    assert_row(&beyond, "B", &format!("B3:{B_PREFIX}-3|"));

    let huge = view("/p/999999999").await;
    assert_row(&huge, "A", &format!("A5:{A_PREFIX}-5|"));
    assert_row(&huge, "B", &format!("B3:{B_PREFIX}-3|"));

    let prefixed_b = view("/b_p/2").await;
    assert_row(&prefixed_b, "A", &format!("A1:{A_PREFIX}-1|"));
    assert_row(&prefixed_b, "PA", &format!("PA1:{A_PREFIX}-1|"));
    assert_row(&prefixed_b, "PB", &format!("PB3:{B_PREFIX}-3|"));

    let offset_huge_first = view("").await;
    assert!(
        !section(&offset_huge_first, "OH").contains(OFFSET_HUGE_PREFIX),
        "live renders no huge-offset rows on the initial page:\n{offset_huge_first}",
    );

    let offset_huge_second = view("/p/2").await;
    for index in 20..=23 {
        assert_row(
            &offset_huge_second,
            "OH",
            &format!("OH{index}:{OFFSET_HUGE_PREFIX}-{index:02}|"),
        );
    }
}

#[tokio::test]
async fn list_pages_saved_view_preserves_argument_order_around_the_pager() {
    const HOLDER: &str = "fixture-lp-argument-order-holder";
    const ITEM_PREFIX: &str = "fixture-lp-argument-order-item";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for index in 1..=5 {
        let slug = format!("{ITEM_PREFIX}-{index}");
        create_listpages_test_page(&mut runner, site_id, &slug, &slug, "Path row").await;
    }
    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER,
        "ListPages argument order holder",
        &format!(
            concat!(
                "[[module ListPages name=\"{ITEM_PREFIX}-*\" order=\"name\" separate=\"no\" perPage=\"2\"]]\n",
                "ROW%%index%%:%%name%%|\n",
                "[[/module]]",
            ),
            ITEM_PREFIX = ITEM_PREFIX,
        ),
    )
    .await;

    let view = async |extra: &str| match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": HOLDER, "extra": extra},
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected a found argument-order view, got {other:?}"),
    };

    let reverse_order = view("/p/2/tag/alpha").await;
    assert!(
        reverse_order.contains(&format!("ROW3:{ITEM_PREFIX}-3|"))
            && reverse_order.contains(&format!("href=\"/{HOLDER}/p/1/tag/alpha\""))
            && reverse_order.contains(r#"<div class="pager"><span class="pager-no">"#),
        "the pager replaces a numeric page argument in place without reordering later arguments:\n{reverse_order}",
    );
    assert!(
        !reverse_order.contains(r#"<div class="pager"><p>"#),
        "saved-page pagers must keep their span children direct:\n{reverse_order}",
    );

    let category_path = view("/category/fragment/p/2").await;
    assert!(
        category_path.contains(&format!("ROW3:{ITEM_PREFIX}-3|"))
            && category_path
                .contains(&format!("href=\"/{HOLDER}/category/fragment/p/1\"",)),
        "non-page arguments remain before the replaced page argument:\n{category_path}",
    );
}

#[tokio::test]
async fn list_pages_prefixed_url_arguments_address_only_the_matching_module() {
    const HOLDER: &str = "fixture-lp-prefixed-url-holder";
    const ITEM_PREFIX: &str = "fixture-lp-prefixed-url-item";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for index in 1..=3 {
        let slug = format!("{ITEM_PREFIX}-{index}");
        create_listpages_test_page(&mut runner, site_id, &slug, &slug, "URL row").await;
    }
    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER,
        "ListPages prefixed URL holder",
        &format!(
            concat!(
                "P2_START\n",
                "[[module ListPages name=\"{ITEM_PREFIX}-*\" order=\"name\" separate=\"no\" limit=\"@URL|0\" urlAttrPrefix=\"page2\"]]\n",
                "P2%%index%%:%%name%%|\n",
                "[[/module]]\n",
                "P2_END\n",
                "P3_START\n",
                "[[module ListPages name=\"{ITEM_PREFIX}-*\" order=\"name\" separate=\"no\" limit=\"@URL|0\" urlAttrPrefix=\"page3\"]]\n",
                "P3%%index%%:%%name%%|\n",
                "[[/module]]\n",
                "P3_END",
            ),
            ITEM_PREFIX = ITEM_PREFIX,
        ),
    )
    .await;

    let output = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {
                "slug": HOLDER,
                "extra": "/page2_limit/1/page3_limit/2",
            },
            "locales": ["en-US", "en"],
        }),
    );
    let html = match output {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected a found prefixed URL view, got {other:?}"),
    };
    let section = |marker: &str| {
        let start_marker = format!("{marker}_START");
        let end_marker = format!("{marker}_END");
        let start = html
            .find(&start_marker)
            .expect("prefixed URL fixture section should start");
        let end = html[start..]
            .find(&end_marker)
            .map(|offset| start + offset)
            .expect("prefixed URL fixture section should end");
        &html[start..end]
    };

    assert!(
        section("P2").contains(&format!("P21:{ITEM_PREFIX}-1|"))
            && !section("P2").contains(&format!("{ITEM_PREFIX}-2|")),
        "page2_limit applies only to the page2-prefixed module:\n{html}",
    );
    assert!(
        section("P3").contains(&format!("P31:{ITEM_PREFIX}-1|"))
            && section("P3").contains(&format!("P32:{ITEM_PREFIX}-2|"))
            && !section("P3").contains(&format!("{ITEM_PREFIX}-3|")),
        "page3_limit applies only to the page3-prefixed module:\n{html}",
    );
}

#[tokio::test]
async fn list_pages_url_offset_selector_reads_the_request_route() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let holder_slug = "fixture-lp-offset-holder";
    let tag = "fixture-lp-offset-tag";

    for title in ["Alpha", "Bravo", "Charlie"] {
        let slug = format!("fixture-lp-offset-{}", title.to_lowercase());
        let revision_id =
            create_listpages_test_page(&mut runner, site_id, &slug, title, "Row body.")
                .await;
        set_listpages_test_tags(&mut runner, site_id, &slug, revision_id, &[tag]).await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        holder_slug,
        "Fixture LP Offset Holder",
        &format!(
            concat!(
                "[[module ListPages tags=\"{tag}\" name=\"fixture-lp-offset-*\" ",
                "order=\"title\" separate=\"no\" limit=\"2\" offset=\"@URL|0\"]]\n",
                "ROW %%title%%\n[[/module]]",
            ),
            tag = tag,
        ),
    )
    .await;

    let view = async |extra: &str| {
        let output = run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": null,
                "route": {"slug": holder_slug, "extra": extra},
                "locales": ["en-US", "en"],
            }),
        );
        match output {
            GetPageViewOutput::Found {
                compiled_body_html, ..
            } => compiled_body_html,
            other => panic!("expected found ListPages offset view, got {other:?}"),
        }
    };

    let rows = |html: &str| {
        ["Alpha", "Bravo", "Charlie"]
            .into_iter()
            .filter(|title| html.contains(&format!("ROW {title}")))
            .collect::<Vec<_>>()
    };

    let base = view("").await;
    assert_eq!(rows(&base), vec!["Alpha", "Bravo"], "{base}");

    let offset = view("/offset/1").await;
    assert_eq!(rows(&offset), vec!["Bravo", "Charlie"], "{offset}");

    for invalid in ["/offset/not-an-integer"] {
        let fallback = view(invalid).await;
        assert_eq!(
            rows(&fallback),
            vec!["Alpha", "Bravo"],
            "{invalid} must use the authored fallback: {fallback}",
        );
    }

    let paginated_holder_slug = "fixture-lp-offset-paginated-holder";
    create_listpages_test_page(
        &mut runner,
        site_id,
        paginated_holder_slug,
        "Fixture LP Offset Paginated Holder",
        &format!(
            concat!(
                "[[module ListPages tags=\"{tag}\" name=\"fixture-lp-offset-*\" ",
                "order=\"title\" separate=\"no\" perPage=\"1\" offset=\"@URL|0\"]]\n",
                "ROW %%title%%\n[[/module]]",
            ),
            tag = tag,
        ),
    )
    .await;

    let paginated_view = async |extra: &str| {
        let output = run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": null,
                "route": {"slug": paginated_holder_slug, "extra": extra},
                "locales": ["en-US", "en"],
            }),
        );
        match output {
            GetPageViewOutput::Found {
                compiled_body_html, ..
            } => compiled_body_html,
            other => {
                panic!("expected found paginated ListPages offset view, got {other:?}")
            }
        }
    };
    let offset_with_page = paginated_view("/offset/1/p/2").await;
    assert_eq!(
        rows(&offset_with_page),
        vec!["Charlie"],
        "offset and pagination compose through the request route: {offset_with_page}",
    );
}

#[tokio::test]
async fn ajax_listpages_recursively_renders_balanced_nested_modules() {
    const OUTER_SLUG: &str = "fixture-ajax-listpages-nested-outer";
    const INNER_SLUG: &str = "fixture-ajax-listpages-nested-inner";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        OUTER_SLUG,
        "Fixture Nested ListPages Outer",
        "Outer row source.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INNER_SLUG,
        "Fixture Nested ListPages Inner",
        "Inner row source.",
    )
    .await;

    let output = run_endpoint!(
        runner,
        wikidot_list_pages_module,
        json!({
            "site_id": site_id,
            "module_body": format!(
                "OUTER_BEFORE %%fullname%%\n\
                 [[module ListPages name=\"{}\" limit=\"1\"]]\n\
                 INNER_ROW %%fullname%%\n\
                 [[/module]]\n\
                 OUTER_AFTER %%fullname%%",
                INNER_SLUG,
            ),
            "parameters": {
                "name": OUTER_SLUG,
                "limit": "1",
            },
        }),
    );

    assert_eq!(
        output
            .body
            .matches(r#"<div class="list-pages-box">"#)
            .count(),
        2,
        "live Wikidot recursively renders both ListPages wrappers: {}",
        output.body,
    );
    for marker in ["OUTER_BEFORE", "INNER_ROW", "OUTER_AFTER"] {
        assert!(
            output.body.contains(marker),
            "nested ListPages output should contain {marker}: {}",
            output.body,
        );
    }
    assert_eq!(
        output.body.matches(OUTER_SLUG).count(),
        3,
        "the outer row substitutes variables before the revealed inner module executes: {}",
        output.body,
    );
    assert!(
        !output.body.contains(INNER_SLUG)
            && !output.body.contains("[[module ListPages")
            && !output.body.contains("[[/module]]"),
        "the inner query should execute without leaking its source or row identity: {}",
        output.body,
    );

    let mut deeply_nested_body = "DEPTH_LIMIT".to_owned();
    for _ in 0..8 {
        deeply_nested_body = format!(
            "[[module ListPages name=\"{INNER_SLUG}\" limit=\"1\"]]\n\
             {deeply_nested_body}\n\
             [[/module]]",
        );
    }
    let depth_limited = run_endpoint!(
        runner,
        wikidot_list_pages_module,
        json!({
            "site_id": site_id,
            "module_body": deeply_nested_body,
            "parameters": {
                "name": OUTER_SLUG,
                "limit": "1",
            },
        }),
    );
    assert_eq!(
        depth_limited
            .body
            .matches(r#"<div class="list-pages-box">"#)
            .count(),
        9,
        "the root pass plus eight nested passes should execute: {}",
        depth_limited.body,
    );

    let too_deep_body = format!(
        "[[module ListPages name=\"{INNER_SLUG}\" limit=\"1\"]]\n\
         {deeply_nested_body}\n\
         [[/module]]",
    );
    let too_deep = run_endpoint!(
        runner,
        wikidot_list_pages_module,
        json!({
            "site_id": site_id,
            "module_body": too_deep_body,
            "parameters": {
                "name": OUTER_SLUG,
                "limit": "1",
            },
        }),
    );
    assert_eq!(
        too_deep
            .body
            .matches(r#"<div class="list-pages-box">"#)
            .count(),
        9,
        "a ninth nested pass should not execute: {}",
        too_deep.body,
    );
    assert_eq!(
        too_deep.body.matches("TODO: module ListPages").count(),
        1,
        "the first module beyond the depth limit should use the unsupported-module diagnostic: {}",
        too_deep.body,
    );

    let many_nested_modules = (0..64)
        .map(|index| {
            format!(
                "[[module ListPages name=\"{INNER_SLUG}\" limit=\"1\"]]\n\
                 CAPPED_{index}\n\
                 [[/module]]",
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let module_limited = run_endpoint!(
        runner,
        wikidot_list_pages_module,
        json!({
            "site_id": site_id,
            "module_body": many_nested_modules,
            "parameters": {
                "name": OUTER_SLUG,
                "limit": "1",
            },
        }),
    );
    assert_eq!(
        module_limited
            .body
            .matches(r#"<div class="list-pages-box">"#)
            .count(),
        65,
        "the root plus 64 revealed sibling modules should execute: {}",
        module_limited.body,
    );

    let too_many_modules = format!(
        "{many_nested_modules}\n\
         [[module ListPages name=\"{INNER_SLUG}\" limit=\"1\"]]\n\
         CAPPED_64\n\
         [[/module]]",
    );
    let too_many = run_endpoint!(
        runner,
        wikidot_list_pages_module,
        json!({
            "site_id": site_id,
            "module_body": too_many_modules,
            "parameters": {
                "name": OUTER_SLUG,
                "limit": "1",
            },
        }),
    );
    assert_eq!(
        too_many
            .body
            .matches(r#"<div class="list-pages-box">"#)
            .count(),
        1,
        "a revealed pass above the module cap should not execute any sibling: {}",
        too_many.body,
    );
    assert_eq!(
        too_many.body.matches("TODO: module ListPages").count(),
        65,
        "all modules above the cap should use the unsupported-module diagnostic: {}",
        too_many.body,
    );
}

#[tokio::test]
async fn ajax_listpages_renders_unbalanced_module_markers_literally() {
    const OUTER_SLUG: &str = "fixture-ajax-listpages-malformed-body";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    create_listpages_test_page(
        &mut runner,
        site_id,
        OUTER_SLUG,
        "Fixture Malformed ListPages Body",
        "Outer row source.",
    )
    .await;

    for (module_body, expected_html) in [
        (
            "BEFORE [[/module]] AFTER",
            "<p>BEFORE [[/module]] AFTER</p>",
        ),
        (
            "BEFORE [[module ListPages limit=\"1\"]] UNCLOSED",
            "<p>BEFORE [[module ListPages limit=&quot;1&quot;]] UNCLOSED</p>",
        ),
    ] {
        let output = run_endpoint!(
            runner,
            wikidot_list_pages_module,
            json!({
                "site_id": site_id,
                "module_body": module_body,
                "parameters": {
                    "name": OUTER_SLUG,
                    "limit": "1",
                },
            }),
        );
        assert_eq!(
            output
                .body
                .matches(r#"<div class="list-pages-box">"#)
                .count(),
            1,
            "the selected outer row should still render: {}",
            output.body,
        );
        assert!(
            output.body.contains(expected_html) && !output.body.contains("@@"),
            "the unbalanced marker should be visible literal text: {}",
            output.body,
        );
    }
}

#[tokio::test]
async fn list_pages_url_category_selector_reads_the_url_category_argument() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let holder_slug = "fixture-lp-cat-holder";

    // One page in the default category and one in a named category, so a
    // resolved selector can be told apart from a dropped one.
    for (slug, title) in [
        ("fixture-lp-cat-default", "Cat Default Probe"),
        ("fixturelpcat:zoned", "Cat Zoned Probe"),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, "Probe body.")
            .await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        holder_slug,
        "Fixture LP Category Holder",
        "placeholder",
    )
    .await;
    let holder = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": holder_slug}),
    )
    .expect("ListPages category holder should exist");

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(holder_slug))),
    });
    let page_info = PageInfo {
        page: Cow::Borrowed(holder_slug),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("Fixture LP Category Holder"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let page_id = PageId {
        site_id,
        category_id: holder.page_category_id,
        page_id: holder.page_id,
    };

    let render = async |wikitext: &str, url_category: Option<&str>| {
        RenderService::render_page(
            runner.context(),
            wikitext.to_owned(),
            &page_info,
            Layout::Wikidot,
            page_id,
            UrlArguments {
                category: url_category,
                ..UrlArguments::default()
            },
        )
        .await
        .expect("ListPages category render should succeed")
        .html_output
        .body
    };

    let source = concat!(
        "[[module ListPages category=\"@URL\" name=\"*\" separate=\"no\" ",
        "limit=\"50\"]]\nROW %%title%%\n[[/module]]",
    );

    let zoned = render(source, Some("fixturelpcat")).await;
    assert!(
        zoned.contains("ROW Cat Zoned Probe"),
        "the URL category selects its own category: {zoned}",
    );
    assert!(
        !zoned.contains("ROW Cat Default Probe"),
        "a page outside the named category must not appear: {zoned}",
    );

    // Live drops the constraint here, which means the module's own default
    // category rather than every category.
    for absent in [None, Some("")] {
        let dropped = render(source, absent).await;
        assert!(
            dropped.contains("ROW Cat Default Probe"),
            "an unresolved @URL falls back to the default category: {dropped}",
        );
        assert!(
            !dropped.contains("ROW Cat Zoned Probe"),
            "dropping the selector must not widen across categories: {dropped}",
        );
    }
}

#[tokio::test]
async fn page_backlinks_view_filters_before_exposing_titles_and_slugs() {
    const TARGET_SLUG: &str = "fixture-page-backlinks-view-target";
    const PRIVATE_CATEGORY: &str = "fixture-page-backlinks-view-private";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;

    let target_revision_id = create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Backlinks View Target",
        "Backlinks view target body",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-page-backlinks-view-beta",
        "Beta Linker",
        &format!("[[[{TARGET_SLUG}]]]"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-page-backlinks-view-alpha",
        "Alpha Linker",
        &format!("[[[{TARGET_SLUG}]]]"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-page-backlinks-view-private",
        "Private Linker",
        &format!("[[[{TARGET_SLUG}]]]"),
    )
    .await;
    set_listpages_test_category_slug(
        &runner,
        site_id,
        "fixture-page-backlinks-view-private",
        PRIVATE_CATEGORY,
    )
    .await;
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("backlinks view permission cache should be invalidated");
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(TARGET_SLUG))),
        ..Default::default()
    });

    let backlinks = run_endpoint!(
        runner,
        page_backlinks_view,
        json!({"site_id": site_id, "page": TARGET_SLUG}),
    );
    assert_eq!(
        backlinks
            .iter()
            .map(|page| (page.slug.as_str(), page.title.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("fixture-page-backlinks-view-alpha", "Alpha Linker"),
            ("fixture-page-backlinks-view-beta", "Beta Linker"),
        ],
    );
    let serialized =
        serde_json::to_value(&backlinks).expect("public backlinks view should serialize");
    assert!(!serialized.to_string().contains("page_id"));
    assert!(!serialized.to_string().contains("Private Linker"));

    let private_category_id = PageCategoryTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page_category::Column::SiteId.eq(site_id))
                .add(page_category::Column::Slug.eq(PRIVATE_CATEGORY)),
        )
        .one(runner.context().transaction())
        .await
        .expect("private backlinks category lookup should succeed")
        .expect("private backlinks category should exist")
        .category_id;
    let target_page_id = PageTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page::Column::SiteId.eq(site_id))
                .add(page::Column::Slug.eq(TARGET_SLUG)),
        )
        .one(runner.context().transaction())
        .await
        .expect("backlinks target lookup should succeed")
        .expect("backlinks target should exist")
        .page_id;
    let transaction = runner.context().transaction();
    let inserted_private_connections = transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            concat!(
                "WITH pages AS (",
                "INSERT INTO page (site_id, page_category_id, slug, layout) ",
                "SELECT $1, $2, $3 || series, 'wikidot' ",
                "FROM generate_series(1, 501) AS series ",
                "RETURNING page_id, slug), revisions AS (",
                "INSERT INTO page_revision (revision_type, created_at, updated_at, ",
                "revision_number, page_id, site_id, user_id, from_wikidot, changes, ",
                "wikitext_hash, compiled_body_html_hash, compiled_body_styles_hash, ",
                "compiled_top_bar_html_hash, compiled_side_bar_html_hash, compiled_at, ",
                "compiled_generator, comments, hidden, title, alt_title, slug, tags) ",
                "SELECT source.revision_type, source.created_at, source.updated_at, ",
                "source.revision_number, pages.page_id, source.site_id, source.user_id, ",
                "source.from_wikidot, source.changes, source.wikitext_hash, ",
                "source.compiled_body_html_hash, source.compiled_body_styles_hash, ",
                "source.compiled_top_bar_html_hash, source.compiled_side_bar_html_hash, ",
                "source.compiled_at, source.compiled_generator, source.comments, ",
                "source.hidden, source.title, source.alt_title, pages.slug, source.tags ",
                "FROM pages CROSS JOIN page_revision source ",
                "WHERE source.revision_id = $4 RETURNING page_id) ",
                "INSERT INTO page_connection ",
                "(from_page_id, to_page_id, connection_type, count) ",
                "SELECT revisions.page_id, $5, 'link', 1 FROM revisions",
            ),
            [
                Value::from(site_id),
                Value::from(private_category_id),
                Value::from("fixture-page-backlinks-view-private-overflow-"),
                Value::from(target_revision_id),
                Value::from(target_page_id),
            ],
        ))
        .await
        .expect("private backlinks overflow connections should insert");
    assert_eq!(inserted_private_connections.rows_affected(), 501);

    let updated_private_pages_result = transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            concat!(
                "UPDATE page SET latest_revision_id = page_revision.revision_id ",
                "FROM page_revision WHERE page.page_id = page_revision.page_id ",
                "AND page.site_id = $1 AND page.slug LIKE $2 || '%'",
            ),
            [
                Value::from(site_id),
                Value::from("fixture-page-backlinks-view-private-overflow-"),
            ],
        ))
        .await
        .expect("private backlinks overflow pages should receive revisions");
    assert_eq!(updated_private_pages_result.rows_affected(), 501);
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("private backlinks overflow permissions should be invalidated");
    let hidden_overflow = run_endpoint!(
        runner,
        page_backlinks_view,
        json!({"site_id": site_id, "page": TARGET_SLUG}),
    );
    assert_eq!(hidden_overflow, backlinks);

    set_listpages_test_category_slug(&runner, site_id, TARGET_SLUG, PRIVATE_CATEGORY)
        .await;
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("target permission cache should be invalidated");
    let error = run_endpoint_err!(
        runner,
        page_backlinks_view,
        json!({"site_id": site_id, "page": TARGET_SLUG}),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);
}

#[tokio::test]
async fn site_tools_orphaned_pages_filters_before_exposing_ordered_rows() {
    const PREFIX: &str = "fixture-site-tools-orphaned-";
    const PRIVATE_CATEGORY: &str = "fixture-site-tools-orphaned-private";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;

    for (slug, title) in [
        ("fixture-site-tools-orphaned-zulu", "Zulu Orphan"),
        ("fixture-site-tools-orphaned-alpha", "alpha Orphan"),
        ("fixture-site-tools-orphaned-linked", "Linked Page"),
        ("fixture-site-tools-orphaned-private", "Private Orphan"),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, "orphan body")
            .await;
    }
    create_listpages_test_page(
        &mut runner,
        site_id,
        "site-tools-orphaned-linker",
        "Orphaned Linker",
        "[[[fixture-site-tools-orphaned-linked]]]",
    )
    .await;
    set_listpages_test_category_slug(
        &runner,
        site_id,
        "fixture-site-tools-orphaned-private",
        PRIVATE_CATEGORY,
    )
    .await;
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("site tools permission cache should be invalidated");
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });

    let rows = run_endpoint!(
        runner,
        site_tools_orphaned_pages,
        json!({"site_id": site_id}),
    );
    let fixture_rows = rows
        .iter()
        .filter(|row| row.slug.starts_with(PREFIX))
        .map(|row| (row.slug.as_str(), row.title.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        fixture_rows,
        vec![
            ("fixture-site-tools-orphaned-alpha", "alpha Orphan"),
            ("fixture-site-tools-orphaned-zulu", "Zulu Orphan"),
        ],
    );
    let serialized =
        serde_json::to_value(&rows).expect("public orphaned rows should serialize");
    assert!(!serialized.to_string().contains("page_id"));
    assert!(!serialized.to_string().contains("Private Orphan"));
}

#[tokio::test]
async fn site_tools_orphaned_pages_ignores_deleted_linkers_without_changing_link_rules() {
    const DELETED_TARGET_SLUG: &str =
        "fixture-site-tools-orphaned-lifecycle-deleted-target";
    const ACTIVE_TARGET_SLUG: &str =
        "fixture-site-tools-orphaned-lifecycle-active-target";
    const SELF_LINK_SLUG: &str = "fixture-site-tools-orphaned-lifecycle-self-link";
    const INCLUDE_TARGET_SLUG: &str =
        "fixture-site-tools-orphaned-lifecycle-include-target";
    const DELETED_LINKER_SLUG: &str = "site-tools-orphaned-lifecycle-deleted-linker";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scpaiueouiuiuiui"}))
        .expect("editable local authoring site should exist");
    let site_id = site.site.site_id;

    for (slug, title, wikitext) in [
        (
            DELETED_TARGET_SLUG,
            "Deleted Linker Target",
            "target linked only from a page that will be deleted",
        ),
        (
            ACTIVE_TARGET_SLUG,
            "Active Linker Target",
            "target linked from a live page",
        ),
        (
            SELF_LINK_SLUG,
            "Self Link Target",
            &format!("[[[{SELF_LINK_SLUG}]]]"),
        ),
        (
            INCLUDE_TARGET_SLUG,
            "Include-only Target",
            "target included but not linked",
        ),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, wikitext).await;
    }

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(DELETED_LINKER_SLUG)),
    );
    let deleted_linker = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": format!("[[[{DELETED_TARGET_SLUG}]]]"),
            "title": "Deleted OrphanedPages Linker",
            "alt_title": null,
            "slug": DELETED_LINKER_SLUG,
            "layout": "wikidot",
            "revision_comments": "create deleted OrphanedPages linker",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    create_listpages_test_page(
        &mut runner,
        site_id,
        "site-tools-orphaned-lifecycle-active-linker",
        "Active OrphanedPages Linker",
        &format!("[[[{ACTIVE_TARGET_SLUG}]]]"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "site-tools-orphaned-lifecycle-include-source",
        "OrphanedPages Include Source",
        &format!("[[include {INCLUDE_TARGET_SLUG}]]"),
    )
    .await;
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
    let orphan_slugs = |rows: &[deepwell::services::view::SiteToolsPageView]| {
        rows.iter()
            .map(|row| row.slug.clone())
            .collect::<BTreeSet<_>>()
    };
    let before_delete = run_endpoint!(
        runner,
        site_tools_orphaned_pages,
        json!({"site_id": site_id}),
    );
    let before_delete = orphan_slugs(&before_delete);
    assert!(!before_delete.contains(DELETED_TARGET_SLUG));
    assert!(!before_delete.contains(ACTIVE_TARGET_SLUG));
    assert!(before_delete.contains(SELF_LINK_SLUG));
    assert!(before_delete.contains(INCLUDE_TARGET_SLUG));

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(deleted_linker.page_id),
    );
    run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": deleted_linker.page_id,
            "last_revision_id": deleted_linker.revision_id,
            "revision_comments": "delete OrphanedPages linker",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
    let after_delete = run_endpoint!(
        runner,
        site_tools_orphaned_pages,
        json!({"site_id": site_id}),
    );
    let after_delete = orphan_slugs(&after_delete);
    assert!(after_delete.contains(DELETED_TARGET_SLUG));
    assert!(!after_delete.contains(ACTIVE_TARGET_SLUG));
    assert!(after_delete.contains(SELF_LINK_SLUG));
    assert!(after_delete.contains(INCLUDE_TARGET_SLUG));
}

#[tokio::test]
async fn site_tools_wanted_pages_filters_sources_before_grouping_ordered_targets() {
    const PREFIX: &str = "fixture-site-tools-wanted-missing-";
    const PRIVATE_CATEGORY: &str = "fixture-site-tools-wanted-private";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-site-tools-wanted-zulu-source",
        "Zulu Source",
        "[[[fixture-site-tools-wanted-missing-zulu]]]",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-site-tools-wanted-alpha-source",
        "alpha Source",
        "[[[fixture-site-tools-wanted-missing-alpha]]]",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-site-tools-wanted-beta-source",
        "Beta Source",
        "[[[fixture-site-tools-wanted-missing-alpha]]]",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-site-tools-wanted-missing-existing",
        "Existing Target",
        "existing target body",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "site-tools-wanted-existing-source",
        "Existing Target Source",
        "[[[fixture-site-tools-wanted-missing-existing]]]",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-site-tools-wanted-private-source",
        "Private Source",
        "[[[fixture-site-tools-wanted-missing-private]]]",
    )
    .await;
    set_listpages_test_category_slug(
        &runner,
        site_id,
        "fixture-site-tools-wanted-private-source",
        PRIVATE_CATEGORY,
    )
    .await;
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("site tools permission cache should be invalidated");
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });

    let targets =
        run_endpoint!(runner, site_tools_wanted_pages, json!({"site_id": site_id}),);
    let fixture_targets = targets
        .iter()
        .filter(|target| target.slug.starts_with(PREFIX))
        .map(|target| {
            (
                target.slug.as_str(),
                target
                    .sources
                    .iter()
                    .map(|source| (source.slug.as_str(), source.title.as_str()))
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        fixture_targets,
        vec![
            (
                "fixture-site-tools-wanted-missing-alpha",
                vec![
                    ("fixture-site-tools-wanted-alpha-source", "alpha Source"),
                    ("fixture-site-tools-wanted-beta-source", "Beta Source"),
                ],
            ),
            (
                "fixture-site-tools-wanted-missing-zulu",
                vec![("fixture-site-tools-wanted-zulu-source", "Zulu Source")],
            ),
        ],
    );
    let serialized =
        serde_json::to_value(&targets).expect("public wanted rows should serialize");
    assert!(!serialized.to_string().contains("page_id"));
    assert!(!serialized.to_string().contains("Private Source"));
}

#[tokio::test]
async fn backlinks_module_renders_current_page_incoming_links() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let target_slug = "fixture-backlinks-current-target";

    create_listpages_test_page(
        &mut runner,
        site_id,
        target_slug,
        "Fixture Backlinks Current Target",
        concat!(
            "BF_DEFAULT_START\n[[module Backlinks]]\nBF_DEFAULT_END\n",
            "BF_INLINE_START\nstart-[[module Backlinks]]-middle\nBF_INLINE_END",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-backlinks-linker-alpha",
        "Fixture Backlinks Linker Alpha",
        &format!("[[[{target_slug}|alpha target link]]]"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-backlinks-linker-beta",
        "Fixture Backlinks Linker Beta",
        &format!("[[[{target_slug}|beta target link]]]"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-backlinks-excluded",
        "Fixture Backlinks Excluded",
        "This page does not link to the backlinks target.",
    )
    .await;

    let target = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": target_slug,
        }),
    )
    .expect("Backlinks target should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": target.page_category_id,
            "page_id": target.page_id,
        }),
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": target_slug,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("Backlinks target should exist after rerender");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    for expected in [
        "BF_DEFAULT_START",
        "<div class=\"backlinks-module-box\">\n\t\t\t<ul>",
        r#"<a href="/fixture-backlinks-linker-alpha">Fixture Backlinks Linker Alpha</a>"#,
        r#"<a href="/fixture-backlinks-linker-beta">Fixture Backlinks Linker Beta</a>"#,
        "BF_DEFAULT_END",
        "BF_INLINE_START",
        "start-[[module Backlinks]]-middle",
        "BF_INLINE_END",
    ] {
        assert!(
            html.contains(expected),
            "Backlinks module output should contain {expected:?}:\n{html}"
        );
    }

    let expected_live_rows = concat!(
        "<div class=\"backlinks-module-box\">\n\t\t\t<ul>\n",
        "\t\t\t\t\t\t\t<li>\n\t\t\t\t\t\t<a href=\"/fixture-backlinks-linker-alpha\">Fixture Backlinks Linker Alpha</a>\n\t\t\t\t\t</li>\n",
        "\t\t\t\t\t\t\t<li>\n\t\t\t\t\t\t<a href=\"/fixture-backlinks-linker-beta\">Fixture Backlinks Linker Beta</a>\n\t\t\t\t\t</li>\n",
        "\t\t\t\t\t</ul>\n\t</div>",
    );
    assert!(
        html.contains(expected_live_rows),
        "Backlinks rows must retain the observed live DOM shape:\n{html}"
    );

    for forbidden in [
        "TODO: module Backlinks",
        "data-wikijump-compat-backlinks",
        "Fixture Backlinks Excluded",
        "fixture-backlinks-excluded",
    ] {
        assert!(
            !html.contains(forbidden),
            "Backlinks module output should not contain {forbidden:?}:\n{html}"
        );
    }

    let alpha = html
        .find("Fixture Backlinks Linker Alpha")
        .expect("alpha backlink should render");
    let beta = html
        .find("Fixture Backlinks Linker Beta")
        .expect("beta backlink should render");
    assert!(
        alpha < beta,
        "Backlinks should render in title order:\n{html}"
    );
    assert_eq!(
        html.matches("[[module Backlinks]]").count(),
        1,
        "only the live inline literal should retain a Backlinks opener:\n{html}",
    );

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-backlinks-linker-gamma",
        "Fixture Backlinks Linker Gamma",
        &format!("[[[{target_slug}|gamma target link]]]"),
    )
    .await;
    let runtime_view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": target_slug, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let runtime_html = match runtime_view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected a found Backlinks page view, got {other:?}"),
    };
    assert!(
        runtime_html.contains("Fixture Backlinks Linker Gamma"),
        "Backlinks must query the current link graph on the next page view:\n{runtime_html}",
    );
    assert!(runtime_html.contains("start-[[module Backlinks]]-middle"));
}

#[tokio::test]
async fn backlinks_module_forgets_removed_links_and_collapses_duplicate_linkers() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let target_slug = "fixture-backlinks-unlink-target";
    create_listpages_test_page(
        &mut runner,
        site_id,
        target_slug,
        "Fixture Backlinks Unlink Target",
        "UNLINK_START\n[[module Backlinks]]\nUNLINK_END",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-backlinks-linker-delta",
        "Fixture Backlinks Linker Delta",
        &format!(
            "[[[{target_slug}|first target link]]] middle [[[{target_slug}|second target link]]]"
        ),
    )
    .await;
    let echo_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-backlinks-linker-echo",
        "Fixture Backlinks Linker Echo",
        &format!("[[[{target_slug}|echo target link]]]"),
    )
    .await;

    let target = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": target_slug}),
    )
    .expect("Backlinks unlink target should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": target.page_category_id,
            "page_id": target.page_id,
        }),
    );
    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": target_slug,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("Backlinks target should exist after rerender");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");
    assert_eq!(
        html.matches("Fixture Backlinks Linker Delta").count(),
        1,
        "a double-linking source must collapse to one backlink row:\n{html}",
    );
    assert!(
        html.contains("Fixture Backlinks Linker Echo"),
        "both linkers should render before the unlink edit:\n{html}",
    );

    run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": "fixture-backlinks-linker-echo",
            "last_revision_id": echo_revision,
            "revision_comments": "remove backlink for unlink freshness",
            "user_id": ADMIN_USER_ID,
            "wikitext": "This page no longer links to the backlinks target.",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("unlink edit should create a revision");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": target.page_category_id,
            "page_id": target.page_id,
        }),
    );
    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": target_slug,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("Backlinks target should exist after unlink rerender");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");
    assert!(
        !html.contains("Fixture Backlinks Linker Echo"),
        "a removed inbound link must disappear on the next render:\n{html}",
    );
    assert!(
        html.contains("Fixture Backlinks Linker Delta"),
        "the remaining linker must survive the unlink edit:\n{html}",
    );
}

#[tokio::test]
async fn backlinks_module_page_preview_renders_current_page_incoming_links() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let target_slug = "fixture-backlinks-preview-target";
    let target_source = "BF_PREVIEW_START\n[[module Backlinks]]\nBF_PREVIEW_END";

    create_listpages_test_page(
        &mut runner,
        site_id,
        target_slug,
        "Fixture Backlinks Preview Target",
        target_source,
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-backlinks-preview-linker-alpha",
        "Fixture Backlinks Preview Linker Alpha",
        &format!("[[[{target_slug}|preview target link]]]"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-backlinks-preview-linker-beta",
        "Fixture Backlinks Preview Linker Beta",
        &format!("[[[{target_slug}|preview target link]]]"),
    )
    .await;

    let target = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": target_slug}),
    )
    .expect("Backlinks preview target should exist");
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(target.page_id)),
        ..Default::default()
    });

    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "Fixture Backlinks Preview Target",
            "wikitext": target_source,
        }),
    );
    let expected_live_rows = concat!(
        "<div class=\"backlinks-module-box\">\n\t\t\t<ul>\n",
        "\t\t\t\t\t\t\t<li>\n",
        "\t\t\t\t\t\t<a href=\"/fixture-backlinks-preview-linker-alpha\">Fixture Backlinks Preview Linker Alpha</a>\n",
        "\t\t\t\t\t</li>\n",
        "\t\t\t\t\t\t\t<li>\n",
        "\t\t\t\t\t\t<a href=\"/fixture-backlinks-preview-linker-beta\">Fixture Backlinks Preview Linker Beta</a>\n",
        "\t\t\t\t\t</li>\n",
        "\t\t\t\t\t</ul>\n\t</div>",
    );
    assert!(
        preview.body.contains(expected_live_rows),
        "PagePreview must render the observed populated Backlinks DOM for its explicit page context:\n{}",
        preview.body,
    );
}

#[tokio::test]
async fn backlinks_page_preview_controls_identity_visibility_and_scan_boundaries() {
    const SOURCE: &str = "[[module Backlinks]]";
    const EMPTY_BOX: &str = "\n<div class=\"backlinks-module-box\">\n</div>\n";
    const PRIVATE_CATEGORY: &str = "fixture-backlinks-preview-private";
    const SCAN_SOURCE_PREFIX: &str = "fixture-backlinks-preview-scan-source-";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let target_slug = "fixture-backlinks-preview-boundary-target";
    create_listpages_test_page(
        &mut runner,
        site_id,
        target_slug,
        "Backlinks Preview Boundary Target",
        SOURCE,
    )
    .await;
    let target = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": target_slug}),
    )
    .expect("Backlinks preview target should exist");

    let escaped_revision_id = create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-backlinks-preview-escape-source",
        "Backlinks Preview Escape Source",
        &format!("[[[{target_slug}|target link]]]"),
    )
    .await;
    let escaped_revision = PageRevisionTable::find_by_id(escaped_revision_id)
        .one(runner.context().transaction())
        .await
        .expect("escaped source revision lookup should succeed")
        .expect("escaped source revision should exist");
    let escaped_page = PageTable::find_by_id(escaped_revision.page_id)
        .one(runner.context().transaction())
        .await
        .expect("escaped source page lookup should succeed")
        .expect("escaped source page should exist");
    let mut escaped_revision = escaped_revision.into_active_model();
    escaped_revision.title = Set("Backlinks <Escape> & \"Title\"".to_owned());
    escaped_revision.slug = Set("fixture-backlinks-escape-&\"".to_owned());
    escaped_revision
        .update(runner.context().transaction())
        .await
        .expect("escaped source revision should update");
    let mut escaped_page = escaped_page.into_active_model();
    escaped_page.slug = Set("fixture-backlinks-escape-&\"".to_owned());
    escaped_page
        .update(runner.context().transaction())
        .await
        .expect("escaped source page should update");

    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-backlinks-preview-private-source",
        "Backlinks Private Source",
        &format!("[[[{target_slug}|target link]]]"),
    )
    .await;
    set_listpages_test_category_slug(
        &runner,
        site_id,
        "fixture-backlinks-preview-private-source",
        PRIVATE_CATEGORY,
    )
    .await;

    let hidden_revision_id = create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-backlinks-preview-hidden-source",
        "Backlinks Hidden Source",
        &format!("[[[{target_slug}|target link]]]"),
    )
    .await;
    let hidden_revision = PageRevisionTable::find_by_id(hidden_revision_id)
        .one(runner.context().transaction())
        .await
        .expect("hidden source revision lookup should succeed")
        .expect("hidden source revision should exist");
    let mut hidden_revision = hidden_revision.into_active_model();
    hidden_revision.hidden = Set(vec!["title".to_owned(), "slug".to_owned()]);
    hidden_revision
        .update(runner.context().transaction())
        .await
        .expect("hidden source revision should update");

    let deleted_revision_id = create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-backlinks-preview-deleted-source",
        "Backlinks Deleted Source",
        &format!("[[[{target_slug}|target link]]]"),
    )
    .await;
    let deleted_revision = PageRevisionTable::find_by_id(deleted_revision_id)
        .one(runner.context().transaction())
        .await
        .expect("deleted source revision lookup should succeed")
        .expect("deleted source revision should exist");
    let deleted_page = PageTable::find_by_id(deleted_revision.page_id)
        .one(runner.context().transaction())
        .await
        .expect("deleted source page lookup should succeed")
        .expect("deleted source page should exist");
    let mut deleted_page = deleted_page.into_active_model();
    deleted_page.deleted_at = Set(Some(OffsetDateTime::now_utc()));
    deleted_page
        .update(runner.context().transaction())
        .await
        .expect("deleted source page should update");

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(target.page_id)),
        ..Default::default()
    });
    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({"site_id": site_id, "title": "unrelated renamed preview", "wikitext": SOURCE}),
    );
    assert!(preview.body.contains("backlinks-module-box"));
    assert!(
        preview
            .body
            .contains("fixture-backlinks-escape-&amp;&quot;")
    );
    assert!(
        preview
            .body
            .contains("Backlinks &lt;Escape&gt; &amp; \"Title\"")
    );
    for forbidden in [
        "Backlinks Private Source",
        "Backlinks Hidden Source",
        "fixture-backlinks-preview-hidden-source",
        "Backlinks Deleted Source",
    ] {
        assert!(
            !preview.body.contains(forbidden),
            "Backlinks leaked {forbidden:?}: {}",
            preview.body
        );
    }

    let empty_slug = "fixture-backlinks-preview-empty-target";
    create_listpages_test_page(&mut runner, site_id, empty_slug, "Empty Target", SOURCE)
        .await;
    let empty_target = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": empty_slug}),
    )
    .expect("empty Backlinks target should exist");
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(empty_target.page_id)),
        ..Default::default()
    });
    let empty = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({"site_id": site_id, "title": "empty", "wikitext": SOURCE}),
    );
    assert_eq!(empty.body, EMPTY_BOX);

    // PageTree and ChildPages resolve no current-page identity in
    // PagePreviewModule, so an identity-bearing and an identity-free preview
    // must stay byte-identical for them.
    let identity_free_source = "[[module PageTree]]\n[[module ChildPages]]";
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: None,
        ..Default::default()
    });
    let identity_free = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({"site_id": site_id, "title": "identity-free", "wikitext": identity_free_source}),
    );
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(target.page_id)),
        ..Default::default()
    });
    let identified = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({"site_id": site_id, "title": "identified", "wikitext": identity_free_source}),
    );
    assert_eq!(identified.body, identity_free.body);

    // Live capture (sandbox-for-codex, 2026-09-14) retained in
    // install/local/wikidot-verification/artifacts/ratings-actor-tie-live-20260914.json:
    // an authorized `edit/PagePreviewModule` request carrying
    // page_unix_name/pageId resolves the same NextPage/PreviousPage selection
    // as the saved render. `Q811_AJAX_CONTEXT` /
    // `Q811_PRIVATE_ADJACENT_AUTHORIZED` ajax_prevholder_a has body sha256
    // fa2843f4986bf0dd2cdad922cfede74c26886291d424589453efbbeadde98e77,
    // matching saved fragment q811_prevholder_a
    // 69fa37a7c9fb89c2b3f097947aec594f9f00ebc5a897afd08f941e4a9d349cc3, and
    // `Q1040_AJAX_CONTEXT` / `Q1040_PRIVATE_ADJACENT_AUTHORIZED`
    // ajax_nextholder_a has body sha256
    // 781be0b4b2dee0512d4b55d488fceb685b42271d51a8aaa7d5637c27ed56fdaa,
    // matching saved fragment q1040_nextholder_a
    // 41ed9ecb09de58c068899f92620c4fcf5f3884ed84025031c2a28b73bb21f9a3. The
    // identity-omitted control `Q1040_AJAX_CONTEXT` ajax_no_context_A renders
    // `<div class="error-block">Invalid range argument.</div>` (body sha256
    // 73bb54ab1bfe5b99fb3e3399c26582837c1ad4232eb8782569056590e9f7eea7).
    fn section<'a>(html: &'a str, start: &str, end: &str) -> &'a str {
        html.split_once(start)
            .unwrap_or_else(|| panic!("missing NextPreviousPage preview start {start:?}"))
            .1
            .split_once(end)
            .unwrap_or_else(|| panic!("missing NextPreviousPage preview end {end:?}"))
            .0
    }

    let next_previous_source = concat!(
        "NEXT_PREVIEW_START\n",
        "[[module NextPage]]\n",
        "NEXT=%%fullname%%|%%title%%\n",
        "[[/module]]\n",
        "NEXT_PREVIEW_END\n",
        "PREV_PREVIEW_START\n",
        "[[module PreviousPage]]\n",
        "PREV=%%fullname%%|%%title%%\n",
        "[[/module]]\n",
        "PREV_PREVIEW_END",
    );
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: None,
        ..Default::default()
    });
    let identity_free_next_previous = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "identity-free",
            "wikitext": next_previous_source,
        }),
    );
    assert!(
        identity_free_next_previous
            .body
            .contains("Invalid range argument.")
            && !identity_free_next_previous.body.contains("NEXT=")
            && !identity_free_next_previous.body.contains("PREV="),
        "an identity-free preview cannot resolve NextPage/PreviousPage:\n{}",
        identity_free_next_previous.body,
    );

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(target.page_id)),
        ..Default::default()
    });
    let identified_next_previous = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": target.title,
            "wikitext": next_previous_source,
        }),
    );
    let target_page = PageTable::find_by_id(target.page_id)
        .one(runner.context().transaction())
        .await
        .expect("NextPage preview target lookup should succeed")
        .expect("NextPage preview target should exist");
    let target_info = PageInfo {
        page: Cow::Borrowed(target_slug),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Owned(target.title.clone()),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: target
            .tags
            .iter()
            .map(|tag| Cow::Borrowed(tag.as_str()))
            .collect(),
        language: Cow::Borrowed("en"),
    };
    let saved_next_previous = RenderService::render_page_for_viewer(
        runner.context(),
        next_previous_source.to_owned(),
        &target_info,
        Layout::Wikidot,
        PageId {
            site_id,
            category_id: target_page.page_category_id,
            page_id: target.page_id,
        },
        None,
        UrlArguments::default(),
    )
    .await
    .expect("saved NextPage/PreviousPage render should succeed")
    .html_output
    .body;
    let identified_next = section(
        &identified_next_previous.body,
        "NEXT_PREVIEW_START",
        "NEXT_PREVIEW_END",
    );
    let saved_next = section(
        &saved_next_previous,
        "NEXT_PREVIEW_START",
        "NEXT_PREVIEW_END",
    );
    assert!(
        identified_next.contains("NEXT="),
        "the identified preview must resolve the NextPage selection:\n{}",
        identified_next_previous.body,
    );
    assert_eq!(
        identified_next, saved_next,
        "the identified preview must resolve the same NextPage selection as the saved render:\n{}",
        identified_next_previous.body,
    );
    let identified_prev = section(
        &identified_next_previous.body,
        "PREV_PREVIEW_START",
        "PREV_PREVIEW_END",
    );
    let saved_prev = section(
        &saved_next_previous,
        "PREV_PREVIEW_START",
        "PREV_PREVIEW_END",
    );
    assert!(
        identified_prev.contains("PREV="),
        "the identified preview must resolve the PreviousPage selection:\n{}",
        identified_next_previous.body,
    );
    assert_eq!(
        identified_prev, saved_prev,
        "the identified preview must resolve the same PreviousPage selection as the saved render:\n{}",
        identified_next_previous.body,
    );

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: None,
        ..Default::default()
    });
    let absent = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({"site_id": site_id, "title": "absent", "wikitext": SOURCE}),
    );
    assert_eq!(absent.body, EMPTY_BOX);

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(i64::MAX)),
        ..Default::default()
    });
    let missing = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({"site_id": site_id, "title": "missing", "wikitext": SOURCE}),
    );
    assert_eq!(missing.body, EMPTY_BOX);

    let stale_slug = "fixture-backlinks-preview-stale-target";
    create_listpages_test_page(&mut runner, site_id, stale_slug, "Stale Target", SOURCE)
        .await;
    let stale = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": stale_slug}),
    )
    .expect("stale Backlinks target should exist");
    let stale_page = PageTable::find_by_id(stale.page_id)
        .one(runner.context().transaction())
        .await
        .expect("stale Backlinks target lookup should succeed")
        .expect("stale Backlinks target lookup should return a page");
    let mut stale_page = stale_page.into_active_model();
    stale_page.deleted_at = Set(Some(OffsetDateTime::now_utc()));
    stale_page
        .update(runner.context().transaction())
        .await
        .expect("stale Backlinks target should update");
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(stale.page_id)),
        ..Default::default()
    });
    let stale_preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({"site_id": site_id, "title": "stale", "wikitext": SOURCE}),
    );
    assert_eq!(stale_preview.body, EMPTY_BOX);

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(target.page_id)),
        ..Default::default()
    });
    let syntax_only = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "syntax-only",
            "wikitext": SOURCE,
            "syntax_only": true,
        }),
    );
    assert!(syntax_only.body.contains(SOURCE));
    let other_site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("second seeded site should exist");
    let other_page_id = create_listpages_test_page(
        &mut runner,
        other_site.site.site_id,
        "fixture-backlinks-preview-other-site-target",
        "Other Site Target",
        SOURCE,
    )
    .await;
    let other_page_id = PageRevisionTable::find_by_id(other_page_id)
        .one(runner.context().transaction())
        .await
        .expect("other-site revision lookup should succeed")
        .expect("other-site revision should exist")
        .page_id;
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(other_page_id)),
        ..Default::default()
    });
    let mismatched = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({"site_id": site_id, "title": "mismatched", "wikitext": SOURCE}),
    );
    assert_eq!(mismatched.body, EMPTY_BOX);

    let template_revision_id = create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-backlinks-preview-scan-template",
        "Backlinks Scan Template",
        &format!("[[[{target_slug}|target link]]]"),
    )
    .await;
    let target_category_id = target.page_category_id;
    let transaction = runner.context().transaction();
    let inserted_connections = transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            concat!(
                "WITH pages AS (",
                "INSERT INTO page (site_id, page_category_id, slug, layout) ",
                "SELECT $1, $2, $3 || series, 'wikidot' ",
                "FROM generate_series(1, 501) AS series ",
                "RETURNING page_id, slug), revisions AS (",
                "INSERT INTO page_revision (revision_type, created_at, updated_at, ",
                "revision_number, page_id, site_id, user_id, from_wikidot, changes, ",
                "wikitext_hash, compiled_body_html_hash, compiled_body_styles_hash, ",
                "compiled_top_bar_html_hash, compiled_side_bar_html_hash, compiled_at, ",
                "compiled_generator, comments, hidden, title, alt_title, slug, tags) ",
                "SELECT source.revision_type, source.created_at, source.updated_at, ",
                "source.revision_number, pages.page_id, source.site_id, source.user_id, ",
                "source.from_wikidot, source.changes, source.wikitext_hash, ",
                "source.compiled_body_html_hash, source.compiled_body_styles_hash, ",
                "source.compiled_top_bar_html_hash, source.compiled_side_bar_html_hash, ",
                "source.compiled_at, source.compiled_generator, source.comments, ",
                "source.hidden, source.title, source.alt_title, pages.slug, source.tags ",
                "FROM pages CROSS JOIN page_revision source ",
                "WHERE source.revision_id = $4 RETURNING revision_id, page_id) ",
                "INSERT INTO page_connection ",
                "(from_page_id, to_page_id, connection_type, count) ",
                "SELECT revisions.page_id, $5, 'link', 1 FROM revisions",
            ),
            [
                Value::from(site_id),
                Value::from(target_category_id),
                Value::from(SCAN_SOURCE_PREFIX),
                Value::from(template_revision_id),
                Value::from(target.page_id),
            ],
        ))
        .await
        .expect("Backlinks scan-boundary sources should be inserted");
    assert_eq!(inserted_connections.rows_affected(), 501);
    let updated_pages_result = transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            concat!(
                "UPDATE page SET latest_revision_id = page_revision.revision_id ",
                "FROM page_revision WHERE page.page_id = page_revision.page_id ",
                "AND page.site_id = $1 AND page.slug LIKE $2 || '%'",
            ),
            [Value::from(site_id), Value::from(SCAN_SOURCE_PREFIX)],
        ))
        .await
        .expect("Backlinks scan-boundary pages should receive their revisions");
    assert_eq!(updated_pages_result.rows_affected(), 501);
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(target.page_id)),
        ..Default::default()
    });
    let saturated = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({"site_id": site_id, "title": "saturated", "wikitext": SOURCE}),
    );
    assert!(!saturated.body.contains("backlinks-module-box"));
}

#[tokio::test]
async fn backlinks_module_ignores_arguments_like_live_wikidot() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let target_slug = "fixture-backlinks-unsupported-target";

    create_listpages_test_page(
        &mut runner,
        site_id,
        target_slug,
        "Fixture Backlinks Argument Target",
        concat!(
            "BACKLINKS_PAGE_ARG_START\n",
            "[[module Backlinks page=\"start\"]]\n",
            "BACKLINKS_PAGE_ARG_END\n",
            "BACKLINKS_UNKNOWN_ARG_START\n",
            "[[module Backlinks foo=\"bar\"]]\n",
            "BACKLINKS_UNKNOWN_ARG_END",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-backlinks-unsupported-linker",
        "Fixture Backlinks Argument Linker",
        &format!("[[[{target_slug}|argument target link]]]"),
    )
    .await;

    let target = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": target_slug,
        }),
    )
    .expect("unsupported Backlinks target should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": target.page_category_id,
            "page_id": target.page_id,
        }),
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": target_slug,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("unsupported Backlinks target should exist after rerender");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(!html.contains("TODO: module Backlinks"));
    assert!(!html.contains("[[module Backlinks"));
    assert_eq!(
        html.matches("Fixture Backlinks Argument Linker").count(),
        2,
        "live Wikidot ignores Backlinks arguments and renders current-page backlinks for both modules:\n{html}"
    );
}

#[tokio::test]
async fn orphanedpages_module_lists_pages_without_incoming_internal_links() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    const HOLDER_SLUG: &str = "fixture-orphanedpages-holder";
    const ORPHAN_SLUG: &str = "fixture-orphanedpages-orphan";
    const LINKED_TARGET_SLUG: &str = "fixture-orphanedpages-linked-target";
    const LINKER_SLUG: &str = "fixture-orphanedpages-linker";

    create_listpages_test_page(
        &mut runner,
        site_id,
        ORPHAN_SLUG,
        "Fixture OrphanedPages Orphan",
        "This page has no incoming internal links.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        LINKED_TARGET_SLUG,
        "Fixture OrphanedPages Linked Target",
        "This page has an incoming internal link.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        LINKER_SLUG,
        "Fixture OrphanedPages Linker",
        &format!("[[[{LINKED_TARGET_SLUG}|linked target]]]"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER_SLUG,
        "Fixture OrphanedPages Holder",
        "ORPHANED_START\n[[module OrphanedPages]]\nORPHANED_END",
    )
    .await;

    let holder = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": HOLDER_SLUG,
        }),
    )
    .expect("OrphanedPages holder should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": holder.page_category_id,
            "page_id": holder.page_id,
        }),
    );

    let html = load_listpages_test_compiled_html(&runner, site_id, HOLDER_SLUG).await;

    for expected in [
        "ORPHANED_START",
        "<h1>List of orphaned pages</h1>",
        concat!(
            r#"<a href="/fixture-orphanedpages-orphan">Fixture OrphanedPages Orphan</a> "#,
            r#"<span style="color: #999">(fixture-orphanedpages-orphan)</span>"#,
            "\n\t\t<br/>",
        ),
        "ORPHANED_END",
    ] {
        assert!(
            html.contains(expected),
            "OrphanedPages output should contain {expected:?}:\n{html}"
        );
    }

    for forbidden in [
        "TODO: module OrphanedPages",
        "[[module OrphanedPages",
        "Fixture OrphanedPages Linked Target",
        "fixture-orphanedpages-linked-target",
    ] {
        assert!(
            !html.contains(forbidden),
            "OrphanedPages output should not contain {forbidden:?}:\n{html}"
        );
    }
}

#[tokio::test]
async fn orphanedpages_module_refreshes_after_linker_soft_delete() {
    const HOLDER_SLUG: &str = "fixture-orphanedpages-lifecycle-holder";
    const DELETED_TARGET_SLUG: &str = "fixture-orphanedpages-lifecycle-deleted-target";
    const ACTIVE_TARGET_SLUG: &str = "fixture-orphanedpages-lifecycle-active-target";
    const SELF_LINK_SLUG: &str = "fixture-orphanedpages-lifecycle-self-link";
    const INCLUDE_TARGET_SLUG: &str = "fixture-orphanedpages-lifecycle-include-target";
    const DELETED_LINKER_SLUG: &str = "orphanedpages-lifecycle-deleted-linker";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scpaiueouiuiuiui"}))
        .expect("editable local authoring site should exist");
    let site_id = site.site.site_id;

    for (slug, title, wikitext) in [
        (
            DELETED_TARGET_SLUG,
            "Deleted Module Linker Target",
            "linked only from a page that will be deleted",
        ),
        (
            ACTIVE_TARGET_SLUG,
            "Active Module Linker Target",
            "linked from a live page",
        ),
        (
            SELF_LINK_SLUG,
            "Self-linked Module Target",
            &format!("[[[{SELF_LINK_SLUG}]]]"),
        ),
        (
            INCLUDE_TARGET_SLUG,
            "Include-only Module Target",
            "included but not linked",
        ),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, wikitext).await;
    }

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(DELETED_LINKER_SLUG)),
    );
    let deleted_linker = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": format!("[[[{DELETED_TARGET_SLUG}]]]"),
            "title": "Deleted OrphanedPages Module Linker",
            "alt_title": null,
            "slug": DELETED_LINKER_SLUG,
            "layout": "wikidot",
            "revision_comments": "create deleted OrphanedPages module linker",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    create_listpages_test_page(
        &mut runner,
        site_id,
        "orphanedpages-lifecycle-active-linker",
        "Active OrphanedPages Module Linker",
        &format!("[[[{ACTIVE_TARGET_SLUG}]]]"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "orphanedpages-lifecycle-include-source",
        "OrphanedPages Module Include Source",
        &format!("[[include {INCLUDE_TARGET_SLUG}]]"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER_SLUG,
        "OrphanedPages Lifecycle Holder",
        "[[module OrphanedPages]]",
    )
    .await;

    let before_delete = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": HOLDER_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let before_delete = match before_delete {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected a found OrphanedPages view, got {other:?}"),
    };
    assert!(!before_delete.contains(DELETED_TARGET_SLUG));
    assert!(!before_delete.contains(ACTIVE_TARGET_SLUG));
    assert!(before_delete.contains(SELF_LINK_SLUG));
    assert!(before_delete.contains(INCLUDE_TARGET_SLUG));

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(deleted_linker.page_id),
    );
    run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": deleted_linker.page_id,
            "last_revision_id": deleted_linker.revision_id,
            "revision_comments": "delete OrphanedPages module linker",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let stored_after_delete =
        load_listpages_test_compiled_html(&runner, site_id, HOLDER_SLUG).await;
    assert!(
        !stored_after_delete.contains(DELETED_TARGET_SLUG),
        "the saved artifact predates the link-graph mutation"
    );

    let after_delete = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": HOLDER_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let after_delete = match after_delete {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected a found OrphanedPages view, got {other:?}"),
    };
    assert!(after_delete.contains(DELETED_TARGET_SLUG));
    assert!(!after_delete.contains(ACTIVE_TARGET_SLUG));
    assert!(after_delete.contains(SELF_LINK_SLUG));
    assert!(after_delete.contains(INCLUDE_TARGET_SLUG));
}

#[tokio::test]
async fn wantedpages_module_groups_missing_internal_links_by_target() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    const HOLDER_SLUG: &str = "fixture-wantedpages-holder";
    const EXISTING_TARGET_SLUG: &str = "fixture-wantedpages-existing-target";
    const MISSING_TARGET_SLUG: &str = "0000-fixture-wantedpages-missing-target";
    const LINKER_A_SLUG: &str = "fixture-wantedpages-linker-a";
    const LINKER_B_SLUG: &str = "fixture-wantedpages-linker-b";

    create_listpages_test_page(
        &mut runner,
        site_id,
        EXISTING_TARGET_SLUG,
        "Fixture WantedPages Existing Target",
        "Existing target.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        LINKER_A_SLUG,
        "AAA Fixture WantedPages Linker A",
        &format!(
            "[[[{EXISTING_TARGET_SLUG}|existing]]]\n[[[{MISSING_TARGET_SLUG}|missing]]]"
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        LINKER_B_SLUG,
        "AAB Fixture WantedPages Linker B",
        &format!("[[[{MISSING_TARGET_SLUG}|missing again]]]"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER_SLUG,
        "AAC Fixture WantedPages Holder",
        "WANTED_START\n[[module WantedPages]]\nWANTED_END",
    )
    .await;

    let holder = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": HOLDER_SLUG,
        }),
    )
    .expect("WantedPages holder should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": holder.page_category_id,
            "page_id": holder.page_id,
        }),
    );

    let html = load_listpages_test_compiled_html(&runner, site_id, HOLDER_SLUG).await;

    for expected in [
        "WANTED_START",
        r#"<div class="wanted-pages-module">"#,
        r#"<table class="form grid" style="margin: 1em auto;">"#,
        "Linked from",
        "Linked to (wanted page name)",
        r#"<a href="/fixture-wantedpages-linker-a">AAA Fixture WantedPages Linker A</a><br/>"#,
        r#"<a href="/fixture-wantedpages-linker-b">AAB Fixture WantedPages Linker B</a><br/>"#,
        r#"<a href="/0000-fixture-wantedpages-missing-target" class="newpage">0000-fixture-wantedpages-missing-target</a>"#,
        "WANTED_END",
    ] {
        assert!(
            html.contains(expected),
            "WantedPages output should contain {expected:?}:\n{html}"
        );
    }

    for forbidden in [
        "TODO: module WantedPages",
        "[[module WantedPages",
        "fixture-wantedpages-existing-target\" class=\"newpage\"",
    ] {
        assert!(
            !html.contains(forbidden),
            "WantedPages output should not contain {forbidden:?}:\n{html}"
        );
    }

    let linker_a = html
        .find("AAA Fixture WantedPages Linker A")
        .expect("first wanted source should render");
    let linker_b = html
        .find("AAB Fixture WantedPages Linker B")
        .expect("second wanted source should render");
    let missing = html
        .find("0000-fixture-wantedpages-missing-target")
        .expect("wanted target should render");
    assert!(
        linker_a < linker_b && linker_b < missing,
        "WantedPages should group source pages before the wanted target:\n{html}"
    );
}

#[tokio::test]
async fn newpage_module_resolves_existing_templates_in_rendered_pages() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    const TEMPLATE_A_SLUG: &str = "template:fixture-newpage-template-a";
    const TEMPLATE_B_SLUG: &str = "template:fixture-newpage-template-b";
    const HOLDER_SLUG: &str = "fixture-newpage-template-holder";
    const MISSING_HOLDER_SLUG: &str = "fixture-newpage-missing-template-holder";

    create_listpages_test_page(
        &mut runner,
        site_id,
        TEMPLATE_A_SLUG,
        "Fixture NewPage Template A",
        "Template A source",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        TEMPLATE_B_SLUG,
        "Fixture NewPage Template B",
        "Template B source",
    )
    .await;
    let template_a_id = listpages_test_page_id(&runner, site_id, TEMPLATE_A_SLUG).await;
    let template_b_id = listpages_test_page_id(&runner, site_id, TEMPLATE_B_SLUG).await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER_SLUG,
        "Fixture NewPage Template Holder",
        &format!(
            "NEWPAGE_START\n[[module NewPage template=\"{TEMPLATE_A_SLUG},{TEMPLATE_B_SLUG}\" category=\"probe\"]]\nNEWPAGE_END"
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        MISSING_HOLDER_SLUG,
        "Fixture NewPage Missing Template Holder",
        "MISSING_START\n[[module NewPage template=\"template:fixture-newpage-template-missing\"]]\nMISSING_END",
    )
    .await;

    for slug in [HOLDER_SLUG, MISSING_HOLDER_SLUG] {
        let holder = run_endpoint!(
            runner,
            page_get,
            json!({
                "site_id": site_id,
                "page": slug,
            }),
        )
        .expect("NewPage holder should exist");
        run_endpoint!(
            runner,
            page_rerender,
            json!({
                "site_id": site_id,
                "category_id": holder.page_category_id,
                "page_id": holder.page_id,
            }),
        );
    }

    let html = load_listpages_test_compiled_html(&runner, site_id, HOLDER_SLUG).await;
    let expected_fragments = vec![
        "NEWPAGE_START".to_owned(),
        r#"<div class="new-page-box" style="text-align: center; margin: 1em 0;">"#
            .to_owned(),
        r#"<form action="dummy.html" method="get" onsubmit="WIKIDOT.modules.NewPageHelperModule.listeners.create(event);">"#
            .to_owned(),
        r#"<input class="text" name="pageName" type="text" size="30" maxlength="128" style="margin: 1px" aria-label="Name of the new page"/>"#
            .to_owned(),
        r#"<select name="template" style="margin: 1px">"#.to_owned(),
        r#"<option value="" selected="selected">-- Select a template --</option>"#
            .to_owned(),
        format!(r#"<option value="{template_a_id}">Fixture NewPage Template A</option>"#),
        format!(r#"<option value="{template_b_id}">Fixture NewPage Template B</option>"#),
        r#"<input type="submit" class="button" value="Create page" style="margin: 1px;"/>"#
            .to_owned(),
        r#"<input type="hidden" name="categoryName" value="probe"/>"#.to_owned(),
        "NEWPAGE_END".to_owned(),
    ];
    for expected in expected_fragments {
        assert!(
            html.contains(&expected),
            "NewPage template output should contain {expected:?}:\n{html}"
        );
    }
    assert!(
        !html.contains("[[module NewPage"),
        "NewPage template output should not leak the raw module:\n{html}"
    );

    let missing_html =
        load_listpages_test_compiled_html(&runner, site_id, MISSING_HOLDER_SLUG).await;
    assert!(
        missing_html.contains(
            r#"<div class="error-block">Template "template:fixture-newpage-template-missing" can not be found.</div>"#
        ),
        "missing NewPage template should render Wikidot's error block:\n{missing_html}"
    );
}

#[tokio::test]
async fn saved_newpage_views_resolve_templates_from_current_anonymous_state() {
    const TEMPLATE_SLUG: &str = "template:fixture-newpage-freshness-target";
    const TEMPLATE_OLD_TITLE: &str = "Fixture NewPage Freshness Old";
    const TEMPLATE_NEW_TITLE: &str = "Fixture NewPage Freshness Current";
    const HOLDER_SLUG: &str = "fixture-newpage-freshness-holder";
    const COMPOSED_TEMPLATE_SLUG: &str = "fixture-newpage-freshness-composed:_template";
    const COMPOSED_HOLDER_SLUG: &str = "fixture-newpage-freshness-composed:holder";
    const PRIVATE_CATEGORY: &str = "fixture-newpage-freshness-private";
    const MISSING_TEMPLATE_SLUG: &str =
        "template:fixture-newpage-freshness-created-later";
    const MISSING_HOLDER_SLUG: &str = "fixture-newpage-freshness-missing-holder";

    fn assert_single_template(html: &str, page_id: i64) {
        assert!(
            html.contains(&format!(
                r#"<input type="hidden" name="template" value="{page_id}"/>"#,
            )),
            "single-template NewPage output should contain page ID {page_id}:\n{html}",
        );
        assert!(
            !html.contains("<select name=\"template\""),
            "one template should not render a selector:\n{html}",
        );
    }

    async fn load_view(
        runner: &TestRunner,
        site_id: i64,
        slug: &str,
        session_token: Option<&str>,
    ) -> String {
        match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": session_token,
                "route": {"slug": slug, "extra": ""},
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                compiled_body_html, ..
            } => compiled_body_html,
            other => panic!("expected found NewPage holder view, got {other:?}"),
        }
    }

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let template_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        TEMPLATE_SLUG,
        TEMPLATE_OLD_TITLE,
        "NewPage freshness template source.",
    )
    .await;
    let template_id = listpages_test_page_id(&runner, site_id, TEMPLATE_SLUG).await;
    let stable_template_slug = "template:fixture-newpage-freshness-stable";
    create_listpages_test_page(
        &mut runner,
        site_id,
        stable_template_slug,
        "Fixture NewPage Freshness Stable",
        "Stable NewPage template source.",
    )
    .await;

    let newpage_source = format!(
        r#"[[module NewPage template="{TEMPLATE_SLUG},{stable_template_slug}"]]"#,
    );
    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER_SLUG,
        "Fixture NewPage Freshness Holder",
        &newpage_source,
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        COMPOSED_TEMPLATE_SLUG,
        "Fixture NewPage Freshness Category Template",
        &format!("{newpage_source}\n%%content%%"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        COMPOSED_HOLDER_SLUG,
        "Fixture NewPage Freshness Composed Holder",
        "COMPOSED_NEWPAGE_CONTENT",
    )
    .await;

    let stored_holder =
        load_listpages_test_compiled_html(&runner, site_id, HOLDER_SLUG).await;
    let stored_composed =
        load_listpages_test_compiled_html(&runner, site_id, COMPOSED_HOLDER_SLUG).await;
    for stored in [&stored_holder, &stored_composed] {
        assert!(stored.contains(TEMPLATE_OLD_TITLE), "{stored}");
        assert!(!stored.contains(TEMPLATE_NEW_TITLE), "{stored}");
    }

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(template_id),
    );
    let edited_template = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": template_id,
            "last_revision_id": template_revision,
            "revision_comments": "change NewPage freshness template title",
            "user_id": ADMIN_USER_ID,
            "title": TEMPLATE_NEW_TITLE,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("NewPage template title edit should create a revision");

    for slug in [HOLDER_SLUG, COMPOSED_HOLDER_SLUG] {
        let current = load_view(&runner, site_id, slug, None).await;
        assert!(
            current.contains(TEMPLATE_NEW_TITLE) && !current.contains(TEMPLATE_OLD_TITLE),
            "saved NewPage view should use the independently edited title for {slug}:\n{current}",
        );
    }
    assert_eq!(
        load_listpages_test_compiled_html(&runner, site_id, HOLDER_SLUG).await,
        stored_holder,
        "a fresh NewPage GET must not rewrite the holder's stored compiled HTML",
    );
    assert_eq!(
        load_listpages_test_compiled_html(&runner, site_id, COMPOSED_HOLDER_SLUG).await,
        stored_composed,
        "a category-template-hosted NewPage GET must not rewrite stored compiled HTML",
    );

    let admin_session = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "NewPage anonymous template visibility test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("administrator session should be created");
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;
    set_listpages_test_category_slug(&runner, site_id, TEMPLATE_SLUG, PRIVATE_CATEGORY)
        .await;
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("NewPage visibility cache should invalidate");

    let missing_error = format!(
        r#"<div class="error-block">Template "{TEMPLATE_SLUG}" can not be found.</div>"#,
    );
    for session in [None, Some(admin_session.as_str())] {
        let hidden = load_view(&runner, site_id, HOLDER_SLUG, session).await;
        assert!(
            hidden.contains(&missing_error),
            "anonymous and authenticated views must both resolve NewPage templates with anonymous visibility:\n{hidden}",
        );
        assert!(!hidden.contains(TEMPLATE_NEW_TITLE), "{hidden}");
    }

    set_listpages_test_category_slug(&runner, site_id, TEMPLATE_SLUG, "template").await;
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("restored NewPage visibility cache should invalidate");
    let visible_again = load_view(&runner, site_id, HOLDER_SLUG, None).await;
    assert!(
        visible_again.contains(TEMPLATE_NEW_TITLE),
        "{visible_again}"
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(template_id),
    );
    run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": template_id,
            "last_revision_id": edited_template.revision_id,
            "revision_comments": "delete NewPage freshness template",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let deleted = load_view(&runner, site_id, HOLDER_SLUG, None).await;
    assert!(deleted.contains(&missing_error), "{deleted}");

    run_endpoint!(
        runner,
        page_restore,
        json!({
            "site_id": site_id,
            "page_id": template_id,
            "revision_comments": "restore NewPage freshness template",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let restored = load_view(&runner, site_id, HOLDER_SLUG, None).await;
    assert!(restored.contains(TEMPLATE_NEW_TITLE), "{restored}");

    create_listpages_test_page(
        &mut runner,
        site_id,
        MISSING_HOLDER_SLUG,
        "Fixture NewPage Late Template Holder",
        &format!(r#"[[module NewPage template="{MISSING_TEMPLATE_SLUG}"]]"#),
    )
    .await;
    let late_missing = load_view(&runner, site_id, MISSING_HOLDER_SLUG, None).await;
    assert!(
        late_missing.contains(&format!(
            r#"Template "{MISSING_TEMPLATE_SLUG}" can not be found."#,
        )),
        "{late_missing}",
    );
    create_listpages_test_page(
        &mut runner,
        site_id,
        MISSING_TEMPLATE_SLUG,
        "Fixture NewPage Created Later",
        "Late-created NewPage template source.",
    )
    .await;
    let late_template_id =
        listpages_test_page_id(&runner, site_id, MISSING_TEMPLATE_SLUG).await;
    let late_created = load_view(&runner, site_id, MISSING_HOLDER_SLUG, None).await;
    assert_single_template(&late_created, late_template_id);
    assert!(!late_created.contains("can not be found"), "{late_created}");
}

#[tokio::test]
async fn newpage_over_budget_template_list_remains_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    const HOLDER_SLUG: &str = "fixture-newpage-over-budget-holder";
    let names = (0..40)
        .map(|index| format!("template:fixture-newpage-over-budget-{index}"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(
        "NEWPAGE_OVER_BUDGET_START\n[[module NewPage template=\"{names}\"]]\n[[module NewPage button=\"after-budget\"]]\nNEWPAGE_OVER_BUDGET_END"
    );

    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER_SLUG,
        "Fixture NewPage Over Budget Holder",
        &source,
    )
    .await;
    let holder = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": HOLDER_SLUG,
        }),
    )
    .expect("over-budget NewPage holder should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": holder.page_category_id,
            "page_id": holder.page_id,
        }),
    );

    let html = load_listpages_test_compiled_html(&runner, site_id, HOLDER_SLUG).await;
    assert!(
        html.contains("NEWPAGE_OVER_BUDGET_START")
            && html.contains("[[module NewPage template=&quot;"),
        "an over-budget NewPage module must remain literal:\n{html}"
    );
    assert!(
        html.contains("fixture-newpage-over-budget-39"),
        "the complete authored module must remain visible:\n{html}"
    );
    assert_eq!(
        html.matches("class=\"new-page-box\"").count(),
        1,
        "only the later valid NewPage module should expand:\n{html}"
    );
    assert!(
        html.contains(r#"value="after-budget""#),
        "a later valid NewPage module must remain executable:\n{html}"
    );
}

#[tokio::test]
async fn newpage_modules_after_the_per_render_budget_remain_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    const HOLDER_SLUG: &str = "fixture-newpage-module-budget-holder";
    let modules = (0..=64)
        .map(|index| format!(r#"[[module NewPage button="budget-{index}"]]"#))
        .collect::<Vec<_>>()
        .join("\n");
    let source =
        format!("NEWPAGE_MODULE_BUDGET_START\n{modules}\nNEWPAGE_MODULE_BUDGET_END");

    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER_SLUG,
        "Fixture NewPage Module Budget Holder",
        &source,
    )
    .await;
    let holder = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": HOLDER_SLUG,
        }),
    )
    .expect("NewPage module budget holder should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": holder.page_category_id,
            "page_id": holder.page_id,
        }),
    );

    let html = load_listpages_test_compiled_html(&runner, site_id, HOLDER_SLUG).await;
    assert_eq!(
        html.matches("class=\"new-page-box\"").count(),
        64,
        "only the bounded NewPage prefix should expand:\n{html}"
    );
    assert!(
        html.contains("[[module NewPage button=&quot;budget-64&quot;]]"),
        "the first module after the budget must remain literal:\n{html}"
    );
}

#[tokio::test]
async fn categories_module_lists_active_categories_and_honors_include_hidden() {
    const VISIBLE_CATEGORY: &str = "fixture-categories-visible";
    const HIDDEN_CATEGORY: &str = "_fixture-categories-hidden";
    const VISIBLE_PAGE: &str = "fixture-categories-visible-page";
    const HIDDEN_PAGE: &str = "fixture-categories-hidden-page";
    const INDEX_PAGE: &str = "fixture-categories-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        VISIBLE_PAGE,
        "Fixture Categories Visible",
        "visible category page",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        HIDDEN_PAGE,
        "Fixture Categories Hidden",
        "hidden category page",
    )
    .await;

    let visible_category =
        CategoryService::get_or_create(runner.context(), site_id, VISIBLE_CATEGORY)
            .await
            .expect("visible category should be created");
    let hidden_category =
        CategoryService::get_or_create(runner.context(), site_id, HIDDEN_CATEGORY)
            .await
            .expect("hidden category should be created");
    for (page_slug, category_id) in [
        (VISIBLE_PAGE, visible_category.category_id),
        (HIDDEN_PAGE, hidden_category.category_id),
    ] {
        let page = PageTable::find()
            .filter(
                sea_orm::Condition::all()
                    .add(page::Column::SiteId.eq(site_id))
                    .add(page::Column::Slug.eq(page_slug)),
            )
            .one(runner.context().transaction())
            .await
            .expect("category fixture page lookup should not fail")
            .expect("category fixture page should exist");
        let mut page = page.into_active_model();
        page.page_category_id = Set(category_id);
        page.update(runner.context().transaction())
            .await
            .expect("category fixture page should move categories");
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_PAGE,
        "Fixture Categories Index",
        concat!(
            "CAT_DEFAULT_START\n[[module Categories]]\nCAT_DEFAULT_END\n",
            "CAT_TRUE_START\n[[module categories includeHidden=\"true\"]]\nCAT_TRUE_END\n",
            "CAT_FALSE_START\n[[module Categories includeHidden=\"false\"]]\nCAT_FALSE_END\n",
            "CAT_UPPER_ATTR_START\n[[module CATEGORIES INCLUDEHIDDEN=\"true\"]]\nCAT_UPPER_ATTR_END\n",
            "CAT_BARE_START\n[[module Categories includeHidden=true]]\nCAT_BARE_END",
        ),
    )
    .await;
    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": INDEX_PAGE,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("Categories index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    let section = |start: &str, end: &str| {
        let start = html.find(start).expect("section start should render");
        let end = html[start..]
            .find(end)
            .map(|offset| start + offset)
            .expect("section end should render");
        &html[start..end]
    };
    let default = section("CAT_DEFAULT_START", "CAT_DEFAULT_END");
    let true_quoted = section("CAT_TRUE_START", "CAT_TRUE_END");
    let false_quoted = section("CAT_FALSE_START", "CAT_FALSE_END");
    let upper_attr = section("CAT_UPPER_ATTR_START", "CAT_UPPER_ATTR_END");
    let bare = section("CAT_BARE_START", "CAT_BARE_END");

    for snippet in [default, true_quoted, false_quoted, upper_attr, bare] {
        assert!(
            snippet.contains(&format!("<h3>{VISIBLE_CATEGORY}</h3>")),
            "visible categories should render in every Categories module case:\n{snippet}"
        );
        assert!(snippet.contains("<h3>_default</h3>"));
    }
    for snippet in [true_quoted, false_quoted] {
        assert!(
            snippet.contains(&format!("<h3>{HIDDEN_CATEGORY}</h3>")),
            "live Wikidot treats any non-empty exact includeHidden=\"...\" value as enabling hidden categories:\n{snippet}"
        );
        assert!(
            snippet.contains(r#"style="display: none" id="category-pages-"#)
                && snippet.contains(r#"-options">1</div>"#),
            "includeHidden output should carry Wikidot's hidden-options marker:\n{snippet}"
        );
    }
    for snippet in [default, upper_attr, bare] {
        assert!(
            !snippet.contains(&format!("<h3>{HIDDEN_CATEGORY}</h3>")),
            "live Wikidot requires exact-case, double-quoted includeHidden to expose hidden categories:\n{snippet}"
        );
    }
    for (category, category_id) in [
        (VISIBLE_CATEGORY, visible_category.category_id),
        (HIDDEN_CATEGORY, hidden_category.category_id),
    ] {
        assert!(
            true_quoted.contains(&format!(
                concat!(
                    "<h3>{category}</h3>\n",
                    "<a id=\"category-pages-toggler-{category_id}\" href=\"javascript:;\" ",
                    "onclick=\"WIKIDOT.modules.WikiCategoriesModule.listeners.toggleListPages(event, {category_id})\">+ list pages</a>"
                ),
                category = category,
                category_id = category_id,
            )),
            "Categories should use the live Wikidot DOM for {category}:\n{html}"
        );
    }
    assert!(!html.contains("TODO: module Categories"));
    assert!(!html.contains("[[module Categories"));
}

#[tokio::test]
async fn categories_module_refreshes_inventory_after_category_pages_come_and_go() {
    const CATEGORY_AAA: &str = "fixture-categories-lifecycle-aaa";
    const CATEGORY_ZZZ: &str = "fixture-categories-lifecycle-zzz";
    const PAGE_AAA: &str = "fixture-categories-lifecycle-aaa-page";
    const PAGE_ZZZ: &str = "fixture-categories-lifecycle-zzz-page";
    const INDEX_PAGE: &str = "fixture-categories-lifecycle-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (page_slug, category_slug) in [(PAGE_AAA, CATEGORY_AAA), (PAGE_ZZZ, CATEGORY_ZZZ)]
    {
        create_listpages_test_page(
            &mut runner,
            site_id,
            page_slug,
            page_slug,
            "lifecycle category page",
        )
        .await;
        let category =
            CategoryService::get_or_create(runner.context(), site_id, category_slug)
                .await
                .expect("lifecycle category should be created");
        let page = PageTable::find()
            .filter(
                sea_orm::Condition::all()
                    .add(page::Column::SiteId.eq(site_id))
                    .add(page::Column::Slug.eq(page_slug)),
            )
            .one(runner.context().transaction())
            .await
            .expect("lifecycle page lookup should not fail")
            .expect("lifecycle page should exist");
        let mut page = page.into_active_model();
        page.page_category_id = Set(category.category_id);
        page.update(runner.context().transaction())
            .await
            .expect("lifecycle page should move categories");
    }
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_PAGE,
        "Fixture Categories Lifecycle Index",
        "LIFECYCLE_START\n[[module Categories]]\nLIFECYCLE_END",
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": INDEX_PAGE,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("Categories lifecycle index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");
    for category in [CATEGORY_AAA, CATEGORY_ZZZ] {
        assert!(
            html.contains(&format!("<h3>{category}</h3>")),
            "a newly populated category should render:\n{html}",
        );
    }
    assert!(
        html.find(CATEGORY_AAA).expect("aaa category should render")
            < html.find(CATEGORY_ZZZ).expect("zzz category should render"),
        "lifecycle categories should render in canonical order:\n{html}",
    );

    for page_slug in [PAGE_AAA, PAGE_ZZZ] {
        let page = run_endpoint!(
            runner,
            page_get,
            json!({"site_id": site_id, "page": page_slug}),
        )
        .expect("lifecycle page should exist before deletion");
        set_mutation_request_context(
            &mut runner,
            ADMIN_USER_ID,
            site_id,
            Reference::Id(page.page_id),
        );
        run_endpoint!(
            runner,
            page_delete,
            json!({
                "site_id": site_id,
                "page": page.page_id,
                "last_revision_id": page.revision_id,
                "revision_comments": "remove lifecycle category page",
                "user_id": ADMIN_USER_ID,
                "ip_address": common::IP_ADDRESS,
            }),
        );
    }
    let index = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": INDEX_PAGE}),
    )
    .expect("Categories lifecycle index should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": index.page_category_id,
            "page_id": index.page_id,
        }),
    );
    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": INDEX_PAGE,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("Categories lifecycle index should exist after deletions");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");
    for category in [CATEGORY_AAA, CATEGORY_ZZZ] {
        assert!(
            !html.contains(&format!("<h3>{category}</h3>")),
            "an emptied category must leave the inventory, not render empty:\n{html}",
        );
    }
    assert!(
        html.contains("<h3>_default</h3>"),
        "the surrounding inventory must survive the lifecycle:\n{html}",
    );
}

#[tokio::test]
async fn page_tree_module_renders_current_page_hierarchy_with_live_depth_dom() {
    const ROOT: &str = "fixture-pagetree-root";
    const ALPHA: &str = "fixture-pagetree-alpha";
    const BETA: &str = "fixture-pagetree-beta";
    const GRANDCHILD: &str = "fixture-pagetree-grandchild";
    const GREAT_GRANDCHILD: &str = "fixture-pagetree-great-grandchild";
    const PRIVATE_CHILD: &str = "fixture-pagetree-private-child";
    const RUNTIME_CHILD: &str = "fixture-pagetree-runtime-child";
    const PRIVATE_CATEGORY: &str = "fixture-pagetree-private";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        ROOT,
        "PageTree Root",
        concat!(
            "PT_DEFAULT_START\n[[module PageTree depth=\"2\"]]\nPT_DEFAULT_END\n",
            "PT_SHOW_START\n[[module PageTree showRoot=\"true\" depth=\"1\"]]\nPT_SHOW_END\n",
            "PT_CASE_START\n[[module PageTree Showroot=\"true\" Depth=\"1\"]]\nPT_CASE_END\n",
            "PT_INLINE_START\nstart-[[module PageTree]]-middle\nPT_INLINE_END",
        ),
    )
    .await;
    for (slug, title) in [
        (ALPHA, "Alpha Child"),
        (BETA, "Beta Child"),
        (GRANDCHILD, "Alpha Grandchild"),
        (GREAT_GRANDCHILD, "Alpha Great Grandchild"),
        (PRIVATE_CHILD, "Private Child"),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, "PageTree fixture")
            .await;
    }
    set_listpages_test_category_slug(&runner, site_id, PRIVATE_CHILD, PRIVATE_CATEGORY)
        .await;
    for (child, parent) in [
        (ALPHA, ROOT),
        (BETA, ROOT),
        (GRANDCHILD, ALPHA),
        (GREAT_GRANDCHILD, GRANDCHILD),
        (PRIVATE_CHILD, ROOT),
    ] {
        set_listpages_test_parent(&mut runner, site_id, child, parent).await;
    }

    let root = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": ROOT,
        }),
    )
    .expect("PageTree root should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": root.page_category_id,
            "page_id": root.page_id,
        }),
    );
    let root = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": ROOT,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("rerendered PageTree root should exist");
    let html = root
        .compiled_body_html
        .expect("compiled body should be included in page_get details");
    fn section<'a>(document: &'a str, start: &str, end: &str) -> &'a str {
        let start = document.find(start).expect("section start should render");
        let end = document[start..]
            .find(end)
            .map(|offset| start + offset)
            .expect("section end should render");
        &document[start..end]
    }

    let default = section(&html, "PT_DEFAULT_START", "PT_DEFAULT_END");
    assert!(
        default.contains(&format!(r#"<a href="/{ALPHA}">Alpha Child</a>"#)),
        "{default}",
    );
    assert!(
        default.contains(&format!(r#"<a href="/{GRANDCHILD}">Alpha Grandchild</a>"#))
    );
    assert!(default.contains(&format!(r#"<a href="/{BETA}">Beta Child</a>"#)));
    assert!(
        !default.contains(PRIVATE_CHILD),
        "PageTree must not reveal a private child to an anonymous render:\n{default}"
    );
    assert!(!default.contains("PageTree Root"));
    assert!(!default.contains("Alpha Great Grandchild"));
    assert!(
        default.find("Alpha Child") < default.find("Beta Child"),
        "siblings should preserve page_parent creation order:\n{default}"
    );

    let show_root = section(&html, "PT_SHOW_START", "PT_SHOW_END");
    assert!(show_root.contains(&format!(r#"<a href="/{ROOT}">PageTree Root</a>"#)));
    assert!(show_root.contains("Alpha Child"));
    assert!(show_root.contains("Beta Child"));
    assert!(!show_root.contains("Alpha Grandchild"));

    let case_variant = section(&html, "PT_CASE_START", "PT_CASE_END");
    assert!(!case_variant.contains("PageTree Root"));
    assert!(case_variant.contains("Alpha Great Grandchild"));
    for unsupported_wrapper in ["class=", " id=", "data-"] {
        assert!(
            !case_variant.contains(unsupported_wrapper),
            "PageTree DOM must remain plain ul, li, and a elements:\n{case_variant}"
        );
    }
    let inline = section(&html, "PT_INLINE_START", "PT_INLINE_END");
    assert!(
        inline.contains("start-[[module PageTree]]-middle"),
        "{inline}"
    );
    assert_eq!(html.matches("[[module PageTree]]").count(), 1);
    assert!(!html.contains("TODO: module PageTree"));

    create_listpages_test_page(
        &mut runner,
        site_id,
        RUNTIME_CHILD,
        "Runtime Child",
        "PageTree runtime fixture",
    )
    .await;
    set_listpages_test_parent(&mut runner, site_id, RUNTIME_CHILD, ROOT).await;
    let runtime_view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": ROOT, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let runtime_html = match runtime_view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected a found PageTree page view, got {other:?}"),
    };
    let runtime_default = section(&runtime_html, "PT_DEFAULT_START", "PT_DEFAULT_END");
    assert!(
        runtime_default
            .contains(&format!(r#"<a href="/{RUNTIME_CHILD}">Runtime Child</a>"#)),
        "PageTree must query current parent state on the next page view:\n{runtime_html}",
    );
}

#[tokio::test]
async fn page_tree_resource_bounds_fail_closed_without_partial_visible_rows() {
    const ROOT: &str = "fixture-pagetree-bound-root";
    const TEMPLATE: &str = "fixture-pagetree-bound-template";
    const NODE_PREFIX: &str = "fixture-pagetree-bound-node";
    const PRIVATE_CATEGORY: &str = "fixture-pagetree-bound-private";
    const PRIVATE_NODE: &str = "fixture-pagetree-bound-node-225";
    const LAST_UNDER_BOUND_NODE: &str = "fixture-pagetree-bound-node-224";
    const FIRST_OVER_BOUND_NODE: &str = "fixture-pagetree-bound-node-226";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        ROOT,
        "PageTree resource-bound root",
        "BOUND_START\n[[module PageTree]]\nBOUND_END",
    )
    .await;
    let template_revision_id = create_listpages_test_page(
        &mut runner,
        site_id,
        TEMPLATE,
        "PageTree bounded node",
        "PageTree resource-bound fixture",
    )
    .await;
    let root = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": ROOT,
        }),
    )
    .expect("PageTree resource-bound root should exist");
    let private_category_id =
        CategoryService::get_or_create(runner.context(), site_id, PRIVATE_CATEGORY)
            .await
            .expect("PageTree resource-bound private category should exist")
            .category_id;
    {
        let transaction = runner.context().transaction();
        transaction
            .execute_raw(Statement::from_sql_and_values(
                transaction.get_database_backend(),
                concat!(
                    "INSERT INTO page (",
                    "site_id, latest_revision_id, page_category_id, slug, layout",
                    ") SELECT $1, $2, $3, $4 || '-' || lpad(sequence::TEXT, 3, '0'), 'wikidot' ",
                    "FROM generate_series(1, 225) AS sequence",
                ),
                [
                    Value::from(site_id),
                    Value::from(template_revision_id),
                    Value::from(root.page_category_id),
                    Value::from(NODE_PREFIX),
                ],
            ))
            .await
            .expect("PageTree bounded node fixtures should be inserted");
        transaction
            .execute_raw(Statement::from_sql_and_values(
                transaction.get_database_backend(),
                "UPDATE page SET page_category_id = $1 WHERE site_id = $2 AND slug = $3",
                [
                    Value::from(private_category_id),
                    Value::from(site_id),
                    Value::from(PRIVATE_NODE),
                ],
            ))
            .await
            .expect(
                "PageTree private node fixture should be hidden from anonymous viewers",
            );
        transaction
            .execute_raw(Statement::from_sql_and_values(
                transaction.get_database_backend(),
                concat!(
                    "INSERT INTO page_parent (parent_page_id, child_page_id, created_at) ",
                    "SELECT parent.page_id, child.page_id, ",
                    "TIMESTAMPTZ '2030-01-01 00:00:00+00' ",
                    "FROM page AS parent CROSS JOIN page AS child ",
                    "WHERE parent.site_id = $1 AND child.site_id = $1 ",
                    "AND parent.slug LIKE $2 || '-%' AND child.slug LIKE $2 || '-%' ",
                    "AND parent.page_id <> child.page_id",
                ),
                [Value::from(site_id), Value::from(NODE_PREFIX)],
            ))
            .await
            .expect("PageTree bounded relation fixtures should be inserted");
        transaction
            .execute_raw(Statement::from_sql_and_values(
                transaction.get_database_backend(),
                concat!(
                    "INSERT INTO page_parent (parent_page_id, child_page_id) ",
                    "SELECT $1, page_id FROM page WHERE site_id = $2 AND slug = $3",
                ),
                [
                    Value::from(root.page_id),
                    Value::from(site_id),
                    Value::from(format!("{NODE_PREFIX}-001")),
                ],
            ))
            .await
            .expect("PageTree root relation fixture should be inserted");
    }

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
    let under_bound = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": ROOT, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let under_bound = match under_bound {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected a found bounded PageTree view, got {other:?}"),
    };
    let under_bound = under_bound
        .split_once("BOUND_START")
        .and_then(|(_, tail)| tail.split_once("BOUND_END"))
        .map(|(section, _)| section)
        .expect("bounded PageTree markers should render");
    assert!(
        under_bound.contains(LAST_UNDER_BOUND_NODE)
            && !under_bound.contains(PRIVATE_NODE),
        "PageTree should apply anonymous visibility before its relation bound:\n{under_bound}",
    );

    {
        let transaction = runner.context().transaction();
        transaction
            .execute_raw(Statement::from_sql_and_values(
                transaction.get_database_backend(),
                concat!(
                    "INSERT INTO page (",
                    "site_id, latest_revision_id, page_category_id, slug, layout",
                    ") VALUES ($1, $2, $3, $4, 'wikidot')",
                ),
                [
                    Value::from(site_id),
                    Value::from(template_revision_id),
                    Value::from(root.page_category_id),
                    Value::from(FIRST_OVER_BOUND_NODE),
                ],
            ))
            .await
            .expect("PageTree first over-bound node should be inserted");
        transaction
            .execute_raw(Statement::from_sql_and_values(
                transaction.get_database_backend(),
                concat!(
                    "INSERT INTO page_parent (parent_page_id, child_page_id, created_at) ",
                    "SELECT added.page_id, existing.page_id, ",
                    "TIMESTAMPTZ '2030-01-02 00:00:00+00' ",
                    "FROM page AS added CROSS JOIN page AS existing ",
                    "WHERE added.site_id = $1 AND added.slug = $2 ",
                    "AND existing.site_id = $1 AND existing.slug LIKE $3 || '-%' ",
                    "AND added.page_id <> existing.page_id ",
                    "UNION ALL ",
                    "SELECT existing.page_id, added.page_id, ",
                    "TIMESTAMPTZ '2030-01-02 00:00:00+00' ",
                    "FROM page AS added CROSS JOIN page AS existing ",
                    "WHERE added.site_id = $1 AND added.slug = $2 ",
                    "AND existing.site_id = $1 AND existing.slug LIKE $3 || '-%' ",
                    "AND added.page_id <> existing.page_id",
                ),
                [
                    Value::from(site_id),
                    Value::from(FIRST_OVER_BOUND_NODE),
                    Value::from(NODE_PREFIX),
                ],
            ))
            .await
            .expect("PageTree over-bound relation fixtures should be inserted");
    }

    let relation_saturated = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": ROOT, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let relation_saturated = match relation_saturated {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => {
            panic!("expected a found relation-saturated PageTree view, got {other:?}")
        }
    };
    let relation_saturated = relation_saturated
        .split_once("BOUND_START")
        .and_then(|(_, tail)| tail.split_once("BOUND_END"))
        .map(|(section, _)| section)
        .expect("relation-saturated PageTree markers should render");
    assert!(
        !relation_saturated.contains(NODE_PREFIX) && !relation_saturated.contains("<ul>"),
        "an exhausted PageTree relation bound must not return a partial visible tree:\n{relation_saturated}",
    );

    {
        let transaction = runner.context().transaction();
        transaction
            .execute_raw(Statement::from_sql_and_values(
                transaction.get_database_backend(),
                concat!(
                    "DELETE FROM page_parent AS relation USING page AS parent, page AS child ",
                    "WHERE relation.parent_page_id = parent.page_id ",
                    "AND relation.child_page_id = child.page_id ",
                    "AND (parent.slug LIKE $1 || '-%' OR child.slug LIKE $1 || '-%') ",
                    "AND NOT (relation.parent_page_id = $2 AND child.slug = $3)",
                ),
                [
                    Value::from(NODE_PREFIX),
                    Value::from(root.page_id),
                    Value::from(format!("{NODE_PREFIX}-001")),
                ],
            ))
            .await
            .expect("PageTree relation fixtures should be reduced to one visible edge");
        transaction
            .execute_raw(Statement::from_sql_and_values(
                transaction.get_database_backend(),
                concat!(
                    "INSERT INTO page (site_id, page_category_id, slug) ",
                    "SELECT $1, $2, 'fixture-pagetree-bound-unrelated-' || sequence ",
                    "FROM generate_series(1, 50001) AS sequence",
                ),
                [Value::from(site_id), Value::from(root.page_category_id)],
            ))
            .await
            .expect("PageTree over-bound page candidates should be inserted");
    }

    let page_saturated = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": ROOT, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let page_saturated = match page_saturated {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected a found page-saturated PageTree view, got {other:?}"),
    };
    let page_saturated = page_saturated
        .split_once("BOUND_START")
        .and_then(|(_, tail)| tail.split_once("BOUND_END"))
        .map(|(section, _)| section)
        .expect("page-saturated PageTree markers should render");
    assert!(
        !page_saturated.contains(NODE_PREFIX) && !page_saturated.contains("<ul>"),
        "an exhausted PageTree candidate bound must not return a partial visible tree:\n{page_saturated}",
    );
}

#[tokio::test]
async fn listpages_parent_selectors_match_the_saved_page_live_fixture() {
    const PREFIX: &str = "fixture-listpages-parent";
    const ROOT: &str = "fixture-listpages-parent-root";
    const HOLDER: &str = "fixture-listpages-parent-holder";
    const SIBLING: &str = "fixture-listpages-parent-sibling";
    const CHILD: &str = "fixture-listpages-parent-child";
    const UNRELATED: &str = "fixture-listpages-parent-unrelated";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let module = |marker: &str, parent: &str| {
        format!(
            concat!(
                "{marker}_START\n",
                "[[module ListPages name=\"{PREFIX}-*\" parent=\"{parent}\" ",
                "order=\"name\" separate=\"no\" perPage=\"250\"]]\n",
                "ROW %%name%%|\n",
                "[[/module]]\n",
                "{marker}_END",
            ),
            marker = marker,
            parent = parent,
            PREFIX = PREFIX,
        )
    };
    let holder_source = [
        module("STATIC", ROOT),
        module("SAME", "="),
        module("DIFFERENT", "-="),
        module("CHILD", "."),
        module("NONE", "-"),
    ]
    .join("\n");

    for (slug, source) in [
        (ROOT, "root"),
        (HOLDER, holder_source.as_str()),
        (SIBLING, "sibling"),
        (CHILD, "child"),
        (UNRELATED, "unrelated"),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, slug, source).await;
    }
    for (child, parent) in [(HOLDER, ROOT), (SIBLING, ROOT), (CHILD, HOLDER)] {
        set_listpages_test_parent(&mut runner, site_id, child, parent).await;
    }

    let holder = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": HOLDER}),
    )
    .expect("ListPages parent holder should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": holder.page_category_id,
            "page_id": holder.page_id,
        }),
    );
    let holder = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": HOLDER,
            "details": {"compiled": true},
        }),
    )
    .expect("rerendered ListPages parent holder should exist");
    let html = holder
        .compiled_body_html
        .expect("compiled parent fixture should be available");
    let section = |marker: &str| {
        let start_marker = format!("{marker}_START");
        let end_marker = format!("{marker}_END");
        let start = html
            .find(&start_marker)
            .expect("parent fixture section should start");
        let end = html[start..]
            .find(&end_marker)
            .map(|offset| start + offset)
            .expect("parent fixture section should end");
        &html[start..end]
    };
    let assert_rows = |marker: &str, included: &[&str], excluded: &[&str]| {
        let section = section(marker);
        for slug in included {
            assert!(
                section.contains(&format!("ROW {slug}|")),
                "{marker} should include {slug}:\n{section}",
            );
        }
        for slug in excluded {
            assert!(
                !section.contains(&format!("ROW {slug}|")),
                "{marker} should exclude {slug}:\n{section}",
            );
        }
    };

    assert_rows("STATIC", &[HOLDER, SIBLING], &[ROOT, CHILD, UNRELATED]);
    assert_rows("SAME", &[HOLDER, SIBLING], &[ROOT, CHILD, UNRELATED]);
    assert_rows("DIFFERENT", &[CHILD], &[ROOT, HOLDER, SIBLING, UNRELATED]);
    assert_rows("CHILD", &[CHILD], &[ROOT, HOLDER, SIBLING, UNRELATED]);
    assert_rows("NONE", &[ROOT, UNRELATED], &[HOLDER, SIBLING, CHILD]);
    assert!(!html.contains("[[module ListPages"), "{html}");
}

#[tokio::test]
async fn listpages_url_parent_does_not_reveal_private_parent_rows() {
    const PRIVATE_CATEGORY: &str = "fixture-listpages-url-parent-private";
    const HOLDER: &str = "fixture-listpages-url-parent-holder";
    const PRIVATE_PARENT: &str = "fixture-listpages-url-parent-private:parent";
    const PUBLIC_CHILD: &str = "fixture-listpages-url-parent-public-child";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER,
        "URL parent holder",
        "[[module ListPages parent=\"@URL\" order=\"name\" separate=\"no\"]]\nROW %%name%%\n[[/module]]",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        PRIVATE_PARENT,
        "Private URL parent",
        "private parent",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        PUBLIC_CHILD,
        "Public child of private URL parent",
        "public child",
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, PRIVATE_PARENT, PRIVATE_CATEGORY)
        .await;
    set_listpages_test_parent(&mut runner, site_id, PUBLIC_CHILD, PRIVATE_PARENT).await;

    let holder = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": HOLDER}),
    )
    .expect("URL parent holder should exist");
    let page_info = PageInfo {
        page: Cow::Borrowed(HOLDER),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("URL parent holder"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let path_arguments = [UrlArgumentPair {
        name: "parent".to_owned(),
        value: Some(PRIVATE_PARENT.to_owned()),
    }];
    let html = RenderService::render_page_for_viewer(
        runner.context(),
        "[[module ListPages parent=\"@URL\" order=\"name\" separate=\"no\"]]\nROW %%name%%\n[[/module]]".to_owned(),
        &page_info,
        Layout::Wikidot,
        PageId {
            site_id,
            category_id: holder.page_category_id,
            page_id: holder.page_id,
        },
        None,
        UrlArguments {
            path_arguments: &path_arguments,
            ..UrlArguments::default()
        },
    )
    .await
    .expect("URL parent render should succeed")
    .html_output
    .body;

    assert!(
        !html.contains(PUBLIC_CHILD),
        "a hidden URL parent must not select or reveal its public child:\n{html}"
    );
}

#[tokio::test]
async fn listpages_current_tag_selectors_match_the_saved_page_live_fixture() {
    const PREFIX: &str = "fixture-listpages-current-tags";
    const HOLDER: &str = "fixture-listpages-current-tags-holder";
    const EXACT: &str = "fixture-listpages-current-tags-exact";
    const TAG_A: &str = "fixture-listpages-current-tags-a";
    const TAG_B: &str = "fixture-listpages-current-tags-b";
    const SUPERSET: &str = "fixture-listpages-current-tags-superset";
    const NONE: &str = "fixture-listpages-current-tags-none";
    const SAME_A: &str = "verification-listpages-current-tags-a";
    const SAME_B: &str = "verification-listpages-current-tags-b";
    const SAME_C: &str = "verification-listpages-current-tags-c";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (slug, tags) in [
        (EXACT, &[SAME_A, SAME_B, "_fixture-target-hidden"][..]),
        (TAG_A, &[SAME_A][..]),
        (TAG_B, &[SAME_B][..]),
        (SUPERSET, &[SAME_A, SAME_B, SAME_C][..]),
        (NONE, &[][..]),
    ] {
        let revision_id =
            create_listpages_test_page(&mut runner, site_id, slug, slug, slug).await;
        if !tags.is_empty() {
            set_listpages_test_tags(&mut runner, site_id, slug, revision_id, tags).await;
        }
    }

    let holder_source = format!(
        concat!(
            "SAME_START\n",
            "[[module ListPages name=\"{PREFIX}-*\" tags=\"=\" order=\"name\" range=\"others\" separate=\"no\" perPage=\"250\"]]\n",
            "ROW %%name%%|\n",
            "[[/module]]\n",
            "SAME_END\n",
            "EXACT_START\n",
            "[[module ListPages name=\"{PREFIX}-*\" tags=\"==\" order=\"name\" range=\"others\" separate=\"no\" perPage=\"250\"]]\n",
            "ROW %%name%%|\n",
            "[[/module]]\n",
            "EXACT_END",
        ),
        PREFIX = PREFIX,
    );
    let holder_revision =
        create_listpages_test_page(&mut runner, site_id, HOLDER, HOLDER, &holder_source)
            .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        HOLDER,
        holder_revision,
        &[SAME_A, SAME_B, "_fixture-holder-hidden"],
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, HOLDER).await;
    let section = |marker: &str| {
        let start_marker = format!("{marker}_START");
        let end_marker = format!("{marker}_END");
        let start = html
            .find(&start_marker)
            .expect("tag fixture section should start");
        let end = html[start..]
            .find(&end_marker)
            .map(|offset| start + offset)
            .expect("tag fixture section should end");
        &html[start..end]
    };
    let assert_rows = |marker: &str, included: &[&str], excluded: &[&str]| {
        let section = section(marker);
        for slug in included {
            assert!(
                section.contains(&format!("ROW {slug}|")),
                "{marker} should include {slug}:\n{section}",
            );
        }
        for slug in excluded {
            assert!(
                !section.contains(&format!("ROW {slug}|")),
                "{marker} should exclude {slug}:\n{section}",
            );
        }
    };

    assert_rows("SAME", &[EXACT, TAG_A, TAG_B, SUPERSET], &[HOLDER, NONE]);
    assert_rows("EXACT", &[EXACT], &[HOLDER, TAG_A, TAG_B, SUPERSET, NONE]);
    assert!(!html.contains("[[module ListPages"), "{html}");
}

#[tokio::test]
async fn listpages_range_selectors_match_the_saved_page_live_fixture() {
    const PREFIX: &str = "fixture-listpages-range";
    const RANGE_A: &str = "fixture-listpages-range-a";
    const RANGE_B: &str = "fixture-listpages-range-b";
    const HOLDER: &str = "fixture-listpages-range-m";
    const RANGE_Y: &str = "fixture-listpages-range-y";
    const RANGE_Z: &str = "fixture-listpages-range-z";
    const TAG: &str = "verification-listpages-range";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for slug in [RANGE_A, RANGE_B, RANGE_Y, RANGE_Z] {
        let revision_id =
            create_listpages_test_page(&mut runner, site_id, slug, slug, slug).await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision_id, &[TAG]).await;
    }

    let holder_source = [
        ("BEFORE", "before"),
        ("AFTER", "after"),
        ("OTHERS", "others"),
        ("CURRENT", "."),
    ]
    .into_iter()
    .map(|(marker, range)| {
        format!(
            concat!(
                "{marker}_START\n",
                "[[module ListPages name=\"{PREFIX}-*\" tags=\"+{TAG}\" order=\"name\" range=\"{range}\" separate=\"no\" perPage=\"250\"]]\n",
                "ROW %%name%%|\n",
                "[[/module]]\n",
                "{marker}_END",
            ),
            marker = marker,
            range = range,
            PREFIX = PREFIX,
            TAG = TAG,
        )
    })
    .collect::<Vec<_>>()
    .join("\n");
    let holder_revision =
        create_listpages_test_page(&mut runner, site_id, HOLDER, HOLDER, &holder_source)
            .await;
    set_listpages_test_tags(&mut runner, site_id, HOLDER, holder_revision, &[TAG]).await;
    let holder = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": HOLDER}),
    )
    .expect("ListPages range holder should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": holder.page_category_id,
            "page_id": holder.page_id,
        }),
    );

    let html = load_listpages_test_compiled_html(&runner, site_id, HOLDER).await;
    let section = |marker: &str| {
        let start_marker = format!("{marker}_START");
        let end_marker = format!("{marker}_END");
        let start = html
            .find(&start_marker)
            .expect("range fixture section should start");
        let end = html[start..]
            .find(&end_marker)
            .map(|offset| start + offset)
            .expect("range fixture section should end");
        &html[start..end]
    };
    let assert_rows = |marker: &str, included: &[&str], excluded: &[&str]| {
        let section = section(marker);
        for slug in included {
            assert!(
                section.contains(&format!("ROW {slug}|")),
                "{marker} should include {slug}:\n{section}",
            );
        }
        for slug in excluded {
            assert!(
                !section.contains(&format!("ROW {slug}|")),
                "{marker} should exclude {slug}:\n{section}",
            );
        }
    };

    assert_rows("BEFORE", &[RANGE_A, RANGE_B], &[HOLDER, RANGE_Y, RANGE_Z]);
    assert_rows("AFTER", &[RANGE_Y, RANGE_Z], &[RANGE_A, RANGE_B, HOLDER]);
    assert_rows("OTHERS", &[RANGE_A, RANGE_B, RANGE_Y, RANGE_Z], &[HOLDER]);
    assert_rows("CURRENT", &[HOLDER], &[RANGE_A, RANGE_B, RANGE_Y, RANGE_Z]);
    assert!(!html.contains("[[module ListPages"), "{html}");
}

#[tokio::test]
async fn listpages_current_metric_selectors_match_the_saved_page_live_fixture() {
    const PREFIX: &str = "fixture-listpages-current-metric";
    const HOLDER: &str = "fixture-listpages-current-metric-holder";
    const TARGET_DOWN: &str = "fixture-listpages-current-metric-down";
    const TARGET_TWO: &str = "fixture-listpages-current-metric-two";
    const TARGET_UP: &str = "fixture-listpages-current-metric-up";
    const TARGET_ZERO: &str = "fixture-listpages-current-metric-zero";
    const TAG: &str = "verification-listpages-current-metric";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for slug in [TARGET_DOWN, TARGET_TWO, TARGET_UP, TARGET_ZERO] {
        let revision_id =
            create_listpages_test_page(&mut runner, site_id, slug, slug, slug).await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision_id, &[TAG]).await;
    }

    let holder_source = [
        ("RATING_CURRENT", r#"rating="=""#),
        ("RATING_NOT_ZERO", r#"rating="<>0""#),
        ("VOTES_CURRENT", r#"votes="=""#),
        ("VOTES_POSITIVE", r#"votes=">0""#),
        ("CREATED_CURRENT", r#"created_at="=""#),
        ("UPDATED_CURRENT", r#"updated_at="=""#),
    ]
    .into_iter()
    .map(|(marker, selector)| {
        format!(
            concat!(
                "{marker}_START\n",
                "[[module ListPages name=\"{PREFIX}-*\" tags=\"+{TAG}\" order=\"name\" separate=\"no\" perPage=\"250\" {selector}]]\n",
                "ROW %%name%%:%%rating%%:%%rating_votes%%|\n",
                "[[/module]]\n",
                "{marker}_END",
            ),
            marker = marker,
            selector = selector,
            PREFIX = PREFIX,
            TAG = TAG,
        )
    })
    .collect::<Vec<_>>()
    .join("\n");
    create_listpages_test_page(&mut runner, site_id, HOLDER, HOLDER, &holder_source)
        .await;
    let holder = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": HOLDER}),
    )
    .expect("ListPages metric holder should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": holder.page_category_id,
            "page_id": holder.page_id,
        }),
    );

    let html = load_listpages_test_compiled_html(&runner, site_id, HOLDER).await;
    let section = |marker: &str| {
        let start_marker = format!("{marker}_START");
        let end_marker = format!("{marker}_END");
        let start = html
            .find(&start_marker)
            .expect("metric fixture section should start");
        let end = html[start..]
            .find(&end_marker)
            .map(|offset| start + offset)
            .expect("metric fixture section should end");
        &html[start..end]
    };
    let assert_rows = |marker: &str, included: &[&str], excluded: &[&str]| {
        let section = section(marker);
        for slug in included {
            assert!(
                section.contains(&format!("ROW {slug}:0:0|")),
                "{marker} should include {slug}:\n{section}",
            );
        }
        for slug in excluded {
            assert!(
                !section.contains(&format!("ROW {slug}:0:0|")),
                "{marker} should exclude {slug}:\n{section}",
            );
        }
    };
    let all_targets = [TARGET_DOWN, TARGET_TWO, TARGET_UP, TARGET_ZERO];

    assert_rows("RATING_CURRENT", &all_targets, &[]);
    assert_rows("RATING_NOT_ZERO", &[], &all_targets);
    assert_rows("VOTES_CURRENT", &[], &all_targets);
    assert_rows("VOTES_POSITIVE", &[], &all_targets);
    assert_rows("CREATED_CURRENT", &all_targets, &[]);
    assert_rows("UPDATED_CURRENT", &all_targets, &[]);
    assert!(!html.contains("[[module ListPages"), "{html}");
}

#[tokio::test]
async fn listpages_rss_link_and_rss_only_match_live_wikidot_markup() {
    const PREFIX: &str = "fixture-listpages-rss-target";
    const HOLDER: &str = "fixture-listpages-rss-holder";
    const TAG: &str = "verification-listpages-rss";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for index in 1..=2 {
        let slug = format!("{PREFIX}-{index}");
        let revision =
            create_listpages_test_page(&mut runner, site_id, &slug, &slug, "RSS row")
                .await;
        set_listpages_test_tags(&mut runner, site_id, &slug, revision, &[TAG]).await;
    }
    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER,
        "ListPages RSS holder",
        &format!(
            concat!(
                "NORMAL_START\n",
                "[[module ListPages name=\"{PREFIX}-*\" tags=\"+{TAG}\" order=\"name\" ",
                "limit=\"1\" rss=\"Feed Title\" rssDescription=\"Description & details\" ",
                "rssHome=\"blog:_start\"]]\nROW %%name%%[[/module]]\n",
                "NORMAL_END\n",
                "ONLY_START\n",
                "[[module ListPages name=\"{PREFIX}-*\" tags=\"+{TAG}\" limit=\"1\" ",
                "rssTitle=\"Feed Title\" rssOnly=\"yes\"]]\nSHOULD_NOT_RENDER %%name%%[[/module]]\n",
                "ONLY_END\n",
                "EMPTY_START\n",
                "[[module ListPages name=\"{PREFIX}-*\" tags=\"+{TAG}\" limit=\"1\" rss=\"\"]]\n",
                "EMPTY_ROW %%name%%[[/module]]\n",
                "EMPTY_END\n",
                "PATH_START\n",
                "[[module ListPages pagetype=\"*\" category=\"* -deleted\" ",
                "tags=\"+alpha -beta\" parent=\"-\" created_by=\"Some_User\" ",
                "rating=\">=1\" offset=\"7\" range=\"others\" order=\"dateCreatedDesc\" ",
                "limit=\"5\" perPage=\"3\" rss=\"RSS4\" rssDescription=\"D\" ",
                "rssHome=\"home\" rssOnly=\"yes\"]]\nIGNORED[[/module]]\n",
                "PATH_END\n",
                "BYPASS_START\n",
                "[[module ListPages range=\"before\" votes=\">=1\" ",
                "order=\"_rank::integer desc\" rss=\"Bypass\" rssOnly=\"yes\"]]\n",
                "IGNORED[[/module]]\n",
                "BYPASS_END",
            ),
            PREFIX = PREFIX,
            TAG = TAG,
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, HOLDER).await;
    let section = |marker: &str| {
        let start_marker = format!("{marker}_START");
        let end_marker = format!("{marker}_END");
        let start = html.find(&start_marker).expect("RSS section should start");
        let end = html[start..]
            .find(&end_marker)
            .map(|offset| start + offset)
            .expect("RSS section should end");
        &html[start..end]
    };

    let normal = section("NORMAL");
    assert!(normal.contains(&format!("ROW {PREFIX}-1")), "{normal}");
    assert!(
        normal.contains(concat!(
            r#"<div class="feedinfo"><span class="rss-icon"><img "#,
            r#"src="/common--theme/base/images/feed/feed-icon-14x14.png" "#,
            r#"alt="rss icon"/></span><a href="http://scp-wiki.wikidot.com/feed/pages/"#,
            r#"category/_default/tags/%2Bverification-listpages-rss/order/name/"#,
            r#"limit/1/t/Feed+Title/d/Description+%26+details/h/blog%3A_start">"#,
            "RSS feed</a></div>",
        ),),
        "normal RSS output should use Wikidot's direct feed-info DOM and canonical URL:\n{normal}\nFULL:\n{html}",
    );

    let only = section("ONLY");
    assert!(only.contains(r#"<div class="feedinfo">"#), "{only}");
    assert!(!only.contains("SHOULD_NOT_RENDER"), "{only}");
    assert!(!only.contains("list-pages-box"), "{only}");
    assert!(!only.contains("list-pages-item"), "{only}");

    let empty = section("EMPTY");
    assert!(empty.contains(&format!("EMPTY_ROW {PREFIX}-2")), "{empty}");
    assert!(!empty.contains("feedinfo"), "{empty}");

    let path = section("PATH");
    assert!(
        path.contains(concat!(
            r#"href="http://scp-wiki.wikidot.com/feed/pages/pagetype/%2A/"#,
            "category/%2A%2C-deleted/tags/%2Balpha%2C-beta/parent/-/",
            "created_by/some_user/offset/7/rating/%3E%3D1/range/others/",
            "order/dateCreatedDesc/limit/3/t/RSS4/d/D/h/home",
            r#"">RSS feed</a>"#,
        )),
        "the complete live RSS selector path should be serialized in Wikidot's order:\n{path}",
    );
    assert!(!path.contains("IGNORED"), "{path}");

    let bypass = section("BYPASS");
    assert!(
        bypass.contains(concat!(
            r#"href="http://scp-wiki.wikidot.com/feed/pages/category/_default/"#,
            "range/before/order/_rank%3A%3Ainteger+desc/t/Bypass",
            r#"">RSS feed</a>"#,
        )),
        "RSS-only rendering should not query or reject selectors the feed URL carries or ignores:\n{bypass}",
    );
    assert!(!bypass.contains("IGNORED"), "{bypass}");
    assert!(!bypass.contains("error-block"), "{bypass}");
}

#[tokio::test]
async fn wikidot_listpages_feed_queries_newest_viewable_pages() {
    const PREFIX: &str = "fixture-listpages-feed-target";
    const TAG: &str = "verification-listpages-feed-endpoint";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (index, timestamp) in [(1, 1_700_000_001), (2, 1_700_000_002), (3, 1_700_000_003)]
    {
        let slug = format!("{PREFIX}-{index}");
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            &slug,
            &format!("Feed target {index}"),
            &format!("Feed body {index}"),
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, &slug, revision, &[TAG]).await;
        set_listpages_test_created_at(
            &runner,
            site_id,
            &slug,
            OffsetDateTime::from_unix_timestamp(timestamp)
                .expect("fixture timestamp should be valid"),
        )
        .await;
    }

    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: None,
    });
    let output = run_endpoint!(
        runner,
        wikidot_list_pages_feed,
        json!({
            "site_id": site_id,
            "pagetype": "normal",
            "category": "*",
            "tags": format!("+{TAG}"),
            "parent": null,
            "created_by": null,
            "rating": null,
            "range": null,
        }),
    );

    assert_eq!(output.items.len(), 3);
    assert_eq!(
        output
            .items
            .iter()
            .map(|item| item.slug.as_str())
            .collect::<Vec<_>>(),
        [
            format!("{PREFIX}-3"),
            format!("{PREFIX}-2"),
            format!("{PREFIX}-1"),
        ],
    );
    assert_eq!(output.items[0].title, "Feed target 3");
    assert!(
        output.items[0].body_html.contains("Feed body 3"),
        "{:?}",
        output.items[0],
    );

    let empty = run_endpoint!(
        runner,
        wikidot_list_pages_feed,
        json!({
            "site_id": site_id,
            "pagetype": null,
            "category": null,
            "tags": TAG,
            "parent": null,
            "created_by": null,
            "rating": null,
            "range": ".",
        }),
    );
    assert!(empty.items.is_empty());
}

#[tokio::test]
async fn wikidot_listpages_feed_skips_imported_pages_without_revisions() {
    const IMPORT_RUN_ID: i64 = 944_210;
    const CATEGORY: &str = "fixture-listpages-feed-incomplete";
    const HEALTHY_SLUG: &str = "fixture-listpages-feed-incomplete:healthy";
    const INCOMPLETE_SLUG: &str = "fixture-listpages-feed-incomplete:no-revision";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        HEALTHY_SLUG,
        "Healthy RSS item",
        "The valid feed item remains available.",
    )
    .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        INCOMPLETE_SLUG,
        "Incomplete imported RSS item",
        "This imported page will have no revision row.",
    )
    .await;
    create_listpages_test_import_run(&runner, site_id, IMPORT_RUN_ID, 1).await;
    let incomplete_page_id =
        listpages_test_page_id(&runner, site_id, INCOMPLETE_SLUG).await;
    mark_imported_page_with_author_snapshot(
        &runner,
        site_id,
        IMPORT_RUN_ID,
        (
            incomplete_page_id,
            INCOMPLETE_SLUG,
            IMPORT_RUN_ID as u64,
            "fixture-imported-author",
        ),
    )
    .await;
    let incomplete_page = PageTable::find_by_id(incomplete_page_id)
        .one(runner.context().transaction())
        .await
        .expect("incomplete imported RSS page lookup should not fail")
        .expect("incomplete imported RSS page should exist");
    let mut incomplete_page = incomplete_page.into_active_model();
    incomplete_page.latest_revision_id = Set(None);
    incomplete_page
        .update(runner.context().transaction())
        .await
        .expect("the incomplete imported page should have no current revision pointer");
    PageRevisionTable::delete_many()
        .filter(page_revision::Column::PageId.eq(incomplete_page_id))
        .exec(runner.context().transaction())
        .await
        .expect("the incomplete imported page revision should be removed");

    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: None,
    });
    let output = run_endpoint!(
        runner,
        wikidot_list_pages_feed,
        json!({"site_id": site_id, "category": CATEGORY}),
    );

    assert_eq!(output.items.len(), 1);
    assert_eq!(output.items[0].slug, HEALTHY_SLUG);
    assert!(
        output.items[0]
            .body_html
            .contains("The valid feed item remains available.")
    );
    assert!(
        !output.items.iter().any(|item| item.slug == INCOMPLETE_SLUG),
        "incomplete imported pages must not poison the whole RSS response",
    );
}

#[tokio::test]
async fn wikidot_listpages_feed_uses_imported_creator_identity_provenance() {
    const IMPORT_RUN_ID: i64 = 944_004;
    const TARGET_SLUG: &str = "fixture-listpages-imported-feed-target";
    const TAG: &str = "verification-listpages-imported-feed";
    const CREATOR_ID: i64 = 10_382_659;
    const CREATOR_NAME: &str = "voted-fated-smuggler";
    const CREATOR_SLUG: &str = "voted-fated-smuggler";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let revision = create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture Imported Feed Target",
        "Imported feed target.",
    )
    .await;
    set_listpages_test_tags(&mut runner, site_id, TARGET_SLUG, revision, &[TAG]).await;
    let target_id = listpages_test_page_id(&runner, site_id, TARGET_SLUG).await;
    create_listpages_test_import_run(&runner, site_id, IMPORT_RUN_ID, 1).await;
    mark_imported_page_with_author_snapshot(
        &runner,
        site_id,
        IMPORT_RUN_ID,
        (target_id, TARGET_SLUG, 944_004, CREATOR_NAME),
    )
    .await;
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "UPDATE wikidot_page_snapshot \
             SET meta_json = jsonb_build_object( \
                 'created_by_id', $1::bigint, \
                 'created_by_unix', $2::text \
             ) \
             WHERE page_id = $3",
            [
                Value::from(CREATOR_ID),
                Value::from(CREATOR_SLUG),
                Value::from(target_id),
            ],
        ))
        .await
        .expect("imported feed creator identity provenance should be attached");

    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: None,
    });
    let output = run_endpoint!(
        runner,
        wikidot_list_pages_feed,
        json!({
            "site_id": site_id,
            "pagetype": "normal",
            "category": "*",
            "tags": format!("+{TAG}"),
        }),
    );

    assert_eq!(output.items.len(), 1);
    let created_by_html = &output.items[0].created_by_html;
    for expected in [
        CREATOR_NAME.to_owned(),
        format!("http://www.wikidot.com/user:info/{CREATOR_SLUG}"),
        r#"<span class="printuser avatarhover">"#.to_owned(),
        format!(
            "http://www.wikidot.com/avatar.php?userid={CREATOR_ID}&amp;amp;size=small&amp;amp;timestamp="
        ),
        format!(
            r#"style="background-image:url(http://www.wikidot.com/userkarma.php?u={CREATOR_ID})""#
        ),
    ] {
        assert!(
            created_by_html.contains(&expected),
            "imported identity provenance must drive feed authorship ({expected}):\n{created_by_html}",
        );
    }
    assert!(
        !created_by_html.contains("onclick=")
            && !created_by_html.contains("data-wikijump-compat"),
        "live ListPages RSS author markup has no browser-only onclick or Wikijump marker:\n{created_by_html}",
    );
}
