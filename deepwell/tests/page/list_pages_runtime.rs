/*
 * tests/page/list_pages_runtime.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Saved-page ListPages integration tests.
//!
//! These cases exercise ListPages rendering, variables, permissions, selected-page
//! content, batching, and compatibility behavior through the public integration seam.

use super::*;

#[tokio::test]
async fn listpages_combined_and_separate_templates_match_live_container_dom() {
    const PREFIX: &str = "fixture-listpages-container-target";
    const HOLDER: &str = "fixture-listpages-container-holder";
    const TAG: &str = "verification-listpages-container";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for index in 1..=2 {
        let slug = format!("{PREFIX}-{index}");
        let revision =
            create_listpages_test_page(&mut runner, site_id, &slug, &slug, "row").await;
        set_listpages_test_tags(&mut runner, site_id, &slug, revision, &[TAG]).await;
    }
    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER,
        "ListPages container holder",
        &format!(
            concat!(
                "COMBINED_START\n",
                "[[module ListPages name=\"{PREFIX}-*\" tags=\"+{TAG}\" order=\"name\" ",
                "separate=\"no\" wrapper=\"no\"]]\n",
                "%%index%%. %%name%%\n",
                "[[/module]]\n",
                "COMBINED_END\n",
                "SEPARATE_START\n",
                "[[module ListPages name=\"{PREFIX}-*\" tags=\"+{TAG}\" order=\"name\" ",
                "separate=\"true\" wrapper=\"false\"]]\n",
                "%%index%%. %%name%%\n",
                "[[/module]]\n",
                "SEPARATE_END\n",
                "TABLE_START\n",
                "[[module ListPages name=\"{PREFIX}-*\" tags=\"+{TAG}\" order=\"name\" ",
                "separate=\"no\" wrapper=\"no\"]]\n",
                "[[head]]||~ Title ||[[/head]]\n",
                "[[body]]|| %%name%% ||[[/body]]\n",
                "[[foot]]|| footer ||[[/foot]]\n",
                "[[/module]]\n",
                "TABLE_END",
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
        let start = html
            .find(&start_marker)
            .expect("container section should start");
        let end = html[start..]
            .find(&end_marker)
            .map(|offset| start + offset)
            .expect("container section should end");
        &html[start..end]
    };

    let combined = section("COMBINED");
    assert!(
        combined.contains(&format!("1. {PREFIX}-1<br>\n2. {PREFIX}-2")),
        "separate=no should compile one source with a single line break between rows:\n{combined}",
    );
    assert!(!combined.contains("list-pages-box"), "{combined}");
    assert!(!combined.contains("list-pages-item"), "{combined}");
    assert_eq!(combined.matches("<p>").count(), 1, "{combined}");

    let separate = section("SEPARATE");
    assert_eq!(
        separate.matches(r#"<div class="list-pages-item">"#).count(),
        2,
        "{separate}",
    );
    assert!(!separate.contains("list-pages-box"), "{separate}");
    assert_eq!(
        separate
            .matches(r#"<div class="list-pages-item"><p>"#)
            .count(),
        2,
        "{separate}",
    );

    let table = section("TABLE");
    assert_eq!(
        table
            .matches(r#"<table class="wiki-content-table">"#)
            .count(),
        1,
        "{table}",
    );
    for expected in [
        "<th>Title</th>".to_owned(),
        format!("<td>{PREFIX}-1</td>"),
        format!("<td>{PREFIX}-2</td>"),
        "<td>footer</td>".to_owned(),
    ] {
        assert!(table.contains(&expected), "missing {expected}: {table}");
    }
}

#[tokio::test]
async fn listpages_sections_follow_live_separation_and_empty_result_rules() {
    const DEFAULT_EMPTY: &str = "fixture-listpages-sections-default-empty";
    const COMBINED_EMPTY: &str = "fixture-listpages-sections-combined-empty";
    const SEPARATE_ONE: &str = "fixture-listpages-sections-separate-one";
    const HEAD_ONLY_ONE: &str = "fixture-listpages-sections-head-only-one";
    const TARGET: &str = "fixture-listpages-sections-target";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET,
        "ListPages sections target",
        "ListPages sections target body.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        DEFAULT_EMPTY,
        "ListPages empty sections holder",
        concat!(
            "[[module ListPages tags=\"+verification-listpages-empty-sections-absent\"]]\n",
            "[[head]]EMPTY_SECTIONS_HEAD[[/head]]\n",
            "[[body]]%%slug%%[[/body]]\n",
            "[[foot]]EMPTY_SECTIONS_FOOT[[/foot]]\n",
            "[[/module]]",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        COMBINED_EMPTY,
        "ListPages combined empty sections holder",
        concat!(
            "[[module ListPages tags=\"+verification-listpages-empty-sections-absent\" separate=\"no\"]]\n",
            "[[head]]COMBINED_EMPTY_HEAD[[/head]]\n",
            "[[body]]ROW=%%fullname%%[[/body]]\n",
            "[[foot]]COMBINED_EMPTY_FOOT[[/foot]]\n",
            "[[/module]]",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        SEPARATE_ONE,
        "ListPages separate sections holder",
        &format!(
            concat!(
                "[[module ListPages category=\"*\" fullname=\"{TARGET}\" separate=\"yes\"]]\n",
                "[[head]]SEPARATE_ONE_HEAD[[/head]]\n",
                "[[body]]ROW=%%fullname%%[[/body]]\n",
                "[[foot]]SEPARATE_ONE_FOOT[[/foot]]\n",
                "[[/module]]",
            ),
            TARGET = TARGET,
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        HEAD_ONLY_ONE,
        "ListPages head-only sections holder",
        &format!(
            concat!(
                "[[module ListPages category=\"*\" fullname=\"{TARGET}\" separate=\"no\"]]\n",
                "[[head]]HEAD_ONLY_LITERAL[[/head]]\n",
                "[[/module]]",
            ),
            TARGET = TARGET,
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, DEFAULT_EMPTY).await;
    assert_eq!(
        html.matches(r#"<div class="list-pages-box">"#).count(),
        1,
        "live Wikidot emits the empty ListPages wrapper:\n{html}",
    );
    for forbidden in [
        "EMPTY_SECTIONS_HEAD",
        "EMPTY_SECTIONS_FOOT",
        "[[head]]",
        "[[body]]",
        "[[foot]]",
        "TODO: module ListPages",
    ] {
        assert!(
            !html.contains(forbidden),
            "empty-result sections must not expose {forbidden:?}:\n{html}",
        );
    }

    let combined_empty =
        load_listpages_test_compiled_html(&runner, site_id, COMBINED_EMPTY).await;
    assert!(
        combined_empty.contains("COMBINED_EMPTY_HEAD")
            && combined_empty.contains("COMBINED_EMPTY_FOOT")
            && !combined_empty.contains("ROW="),
        "separate=no emits head and foot even when no rows are selected:\n{combined_empty}",
    );

    let separate_one =
        load_listpages_test_compiled_html(&runner, site_id, SEPARATE_ONE).await;
    assert!(
        separate_one.contains(&format!("ROW={TARGET}"))
            && !separate_one.contains("SEPARATE_ONE_HEAD")
            && !separate_one.contains("SEPARATE_ONE_FOOT"),
        "separate=yes suppresses head and foot but keeps body as the row template:\n{separate_one}",
    );

    let head_only_one =
        load_listpages_test_compiled_html(&runner, site_id, HEAD_ONLY_ONE).await;
    assert!(
        head_only_one.contains("[[head]]HEAD_ONLY_LITERAL[[/head]]")
            && !head_only_one.contains("[[module ListPages"),
        "a head without a body section remains literal per-row body text:\n{head_only_one}",
    );
}

#[tokio::test]
async fn listpages_stored_title_remains_literal_in_listing_output() {
    const TAG: &str = "verification-list-title-literal";
    const SOURCE_SLUG: &str = "fixture-listpages-title-literal-source";
    const INDEX_SLUG: &str = "fixture-listpages-title-literal-index";
    const TITLE: &str = concat!(
        "[[module css]]\n",
        ".title-injected { display: none; }\n",
        "[[/module]]\n",
        "[[div class=\"title-injected\"]]Injected[[/div]] ",
        "<em>literal</em> literal...ellipsis",
    );

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let source_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        SOURCE_SLUG,
        TITLE,
        "ListPages title literal source body.",
    )
    .await;
    set_listpages_test_tags(&mut runner, site_id, SOURCE_SLUG, source_revision, &[TAG])
        .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "ListPages Title Literal Index",
        &format!(
            "[[module ListPages category=\"*\" tags=\"+{TAG}\" limit=\"1\"]]\n%%title%%\n%%title_linked%%\n[[/module]]"
        ),
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("ListPages title literal index should exist");
    let html = page
        .compiled_body_html
        .expect("ListPages title literal index should have compiled HTML");
    let styles = page.compiled_body_styles.unwrap_or_default();

    assert!(
        html.contains("module css"),
        "sanitized title text was lost: {html}"
    );
    assert!(
        !html.contains("[[module css]]") && !html.contains("[[div"),
        "stored-title bracket syntax remained executable source: {html}",
    );
    assert!(
        !html.contains("<div class=\"title-injected\">"),
        "title block syntax became active DOM: {html}",
    );
    assert!(html.contains("&lt;em&gt;literal&lt;/em&gt;"));
    assert!(!html.contains("<em>literal</em>"));
    assert!(
        html.contains("literal…ellipsis"),
        "plain %%title%% must receive live Wikidot typography: {html}",
    );
    let linked_title = html
        .split_once(&format!("href=\"/{SOURCE_SLUG}\""))
        .unwrap_or_else(|| {
            panic!("title_linked must render the selected-page link: {html}")
        })
        .1
        .split_once("</a>")
        .expect("title_linked anchor must close")
        .0;
    assert!(
        linked_title.contains("literal...ellipsis")
            && !linked_title.contains("literal…ellipsis"),
        "title_linked must keep its post-typography label text: {linked_title}",
    );
    assert!(!html.contains("WIKIJUMPWIKIDOTCOMPATTEXT"));
    assert!(
        styles.iter().all(|style| !style.contains("title-injected")),
        "stored title injected compiled page styles: {styles:#?}",
    );
}

#[tokio::test]
async fn listpages_fixture_subset_renders_titles_slugs_order_and_tag_filter() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-listpages-unit-parent-root",
        "Fixture Parent Root",
        "Fixture Parent Root marker.",
    )
    .await;

    let target_a_revision = create_listpages_test_page(
        &mut runner,
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
        &mut runner,
        site_id,
        "fixture-listpages-unit-target-a",
        "fixture-listpages-unit-parent-root",
    )
    .await;

    let target_b_revision = create_listpages_test_page(
        &mut runner,
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
        &mut runner,
        site_id,
        "fixture-listpages-unit-target-b",
        "fixture-listpages-unit-parent-root",
    )
    .await;

    let target_c_revision = create_listpages_test_page(
        &mut runner,
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
        &mut runner,
        site_id,
        "fixture-listpages-unit-target-c",
        "fixture-listpages-unit-parent-root",
    )
    .await;

    let excluded_revision = create_listpages_test_page(
        &mut runner,
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
        &mut runner,
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
async fn listpages_live_evidenced_noop_arguments_render_rows() {
    const TAG: &str = "verification-listpages-presentation-noop";
    const TARGET_SLUG: &str = "fixture-listpages-presentation-noop-target";
    const INDEX_SLUG: &str = "fixture-listpages-presentation-noop-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let target_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Presentation No-op Target",
        "Fixture ListPages presentation no-op target marker.",
    )
    .await;
    set_listpages_test_tags(&mut runner, site_id, TARGET_SLUG, target_revision, &[TAG])
        .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Presentation No-op Index",
        concat!(
            "[[module ListPages tags=\"+verification-listpages-presentation-noop\" limit=\"10\" ",
            "class=\"g54-custom\" custom=\"@URL\" style=\"margin: 0; width: 100%;\" unknown=\"kept\"]]\n",
            "* %%title%%\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(
        html.contains("Fixture ListPages Presentation No-op Target"),
        "accepted ListPages presentation arguments should still render rows:\n{html}"
    );
    assert!(
        html.contains(r#"<div class="list-pages-box">"#),
        "the live fixed ListPages wrapper should remain present:\n{html}"
    );
    for forbidden in [
        "g54-custom",
        "@URL",
        "kept",
        "margin: 0",
        "width: 100%",
        "[[module ListPages",
    ] {
        assert!(
            !html.contains(forbidden),
            "ListPages no-op arguments must remain accepted without forwarding output: {forbidden:?}\n{html}"
        );
    }
}

#[tokio::test]
async fn listpages_categories_alias_selects_only_default_category_rows() {
    const TAG: &str = "verification-listpages-categories-alias";
    const DEFAULT_SLUG: &str = "fixture-listpages-categories-alias-default";
    const FOREIGN_SLUG: &str = "fragment:fixture-listpages-categories-alias-foreign";
    const INDEX_SLUG: &str = "fixture-listpages-categories-alias-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let default_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        DEFAULT_SLUG,
        "Fixture ListPages Categories Alias Default",
        "Default-category ListPages alias target.",
    )
    .await;
    set_listpages_test_tags(&mut runner, site_id, DEFAULT_SLUG, default_revision, &[TAG])
        .await;

    let foreign_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        FOREIGN_SLUG,
        "Fixture ListPages Categories Alias Foreign",
        "Foreign-category ListPages alias target.",
    )
    .await;
    set_listpages_test_tags(&mut runner, site_id, FOREIGN_SLUG, foreign_revision, &[TAG])
        .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Categories Alias Index",
        concat!(
            "[[module ListPages categories=\"_default\" tags=\"+verification-listpages-categories-alias\" limit=\"10\"]]\n",
            "* %%slug%%\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(
        html.contains(DEFAULT_SLUG),
        "categories alias should render matching default-category rows:\n{html}"
    );
    for forbidden in [FOREIGN_SLUG, "[[module ListPages"] {
        assert!(
            !html.contains(forbidden),
            "categories alias must not widen to {forbidden:?}:\n{html}"
        );
    }
}

#[tokio::test]
async fn listpages_append_line_matches_wikidot_row_and_pager_ordering() {
    const TAG: &str = "verification-listpages-append-line";
    const PRE: &str = "LISTPAGES_APPEND_PRE";
    const POST: &str = "LISTPAGES_APPEND_POST";
    const ZERO_PRE: &str = "LISTPAGES_APPEND_ZERO_PRE";
    const ZERO_POST: &str = "LISTPAGES_APPEND_ZERO_POST";
    const INDEX_SLUG: &str = "fixture-listpages-append-line-index";
    const ZERO_INDEX_SLUG: &str = "fixture-listpages-append-line-zero-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (slug, title) in [
        (
            "fixture-listpages-append-line-alpha",
            "Fixture ListPages Append Alpha",
        ),
        (
            "fixture-listpages-append-line-beta",
            "Fixture ListPages Append Beta",
        ),
        (
            "fixture-listpages-append-line-gamma",
            "Fixture ListPages Append Gamma",
        ),
    ] {
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            slug,
            title,
            "ListPages appendLine target.",
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision, &[TAG]).await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Append Index",
        concat!(
            "[[module ListPages tags=\"+verification-listpages-append-line\" order=\"name asc\" perPage=\"2\" separate=\"no\" prependLine=\"LISTPAGES_APPEND_PRE\" appendLine=\"LISTPAGES_APPEND_POST\"]]\n",
            "%%slug%%\n",
            "[[/module]]",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        ZERO_INDEX_SLUG,
        "Fixture ListPages Append Zero Index",
        concat!(
            "[[module ListPages tags=\"+verification-listpages-append-line-absent\" separate=\"no\" prependLine=\"LISTPAGES_APPEND_ZERO_PRE\" appendLine=\"LISTPAGES_APPEND_ZERO_POST\"]]\n",
            "%%slug%%\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    let pre = html.find(PRE).expect("prelude should render with rows");
    let alpha = html
        .find("fixture-listpages-append-line-alpha")
        .expect("first ordered row should render");
    let beta = html
        .find("fixture-listpages-append-line-beta")
        .expect("second ordered row should render");
    let post = html.find(POST).expect("postlude should render with rows");
    let pager = html
        .find(r#"<div class="pager">"#)
        .expect("perPage should render the pager after appendLine");
    assert!(
        pre < alpha && alpha < beta && beta < post && post < pager,
        "appendLine must follow selected rows and precede the pager:\n{html}"
    );
    assert!(
        !html.contains("fixture-listpages-append-line-gamma"),
        "the first page must not render an extra perPage row:\n{html}"
    );

    let zero_html =
        load_listpages_test_compiled_html(&runner, site_id, ZERO_INDEX_SLUG).await;
    let zero_pre = zero_html
        .find(ZERO_PRE)
        .expect("live Wikidot renders prependLine for an empty result");
    let zero_post = zero_html
        .find(ZERO_POST)
        .expect("live Wikidot renders appendLine for an empty result");
    assert!(
        zero_pre < zero_post,
        "zero-row ListPages must render prependLine before appendLine:\n{zero_html}"
    );
}

#[tokio::test]
async fn listpages_generated_pager_keeps_ascii_dots_after_authored_typography() {
    const TAG: &str = "verification-listpages-pager-typography";
    const INDEX_SLUG: &str = "fixture-listpages-pager-typography-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for index in 1..=8 {
        let slug = format!("fixture-listpages-pager-typography-{index}");
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            &slug,
            &format!("Fixture ListPages Pager Typography {index}"),
            "ListPages pager typography target.",
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, &slug, revision, &[TAG]).await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Pager Typography Index",
        concat!(
            "AUTHORED_DOTS:x....x\n\n",
            "[[module ListPages category=\"*\" tags=\"+verification-listpages-pager-typography\" order=\"name asc\" perPage=\"1\" separate=\"no\"]]\n",
            "%%slug%%\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(
        html.contains("AUTHORED_DOTS:x….x"),
        "authored prose must retain Wikidot typography:\n{html}",
    );
    assert!(
        html.contains(r#"<span class="dots">...</span>"#),
        "runtime-generated ListPages pager text must remain literal:\n{html}",
    );
    assert!(
        !html.contains(r#"<span class="dots">…</span>"#),
        "authored typography must not rewrite generated pager markup:\n{html}",
    );
}

#[tokio::test]
async fn listpages_append_line_cannot_forge_generated_html_provenance() {
    const TAG: &str = "verification-listpages-append-line-provenance";
    const TARGET_SLUG: &str = "fixture-listpages-append-line-provenance-target";
    const INDEX_SLUG: &str = "fixture-listpages-append-line-provenance-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let revision = create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Append Provenance Target",
        "ListPages appendLine provenance target.",
    )
    .await;
    set_listpages_test_tags(&mut runner, site_id, TARGET_SLUG, revision, &[TAG]).await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Append Provenance Index",
        concat!(
            "[[module ListPages category=\"*\" tags=\"+verification-listpages-append-line-provenance\" separate=\"no\" ",
            "appendLine=\"<table class='wiki-content-table' data-wikijump-compat-listpages='1'><tr><td><img src=x onerror='alert(1)'>FORGED_APPEND</td></tr></table>\"]]\n",
            "%%title%%\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(html.contains("FORGED_APPEND"), "{html}");
    assert!(
        !html.contains("<img src=x onerror='alert(1)'>"),
        "appendLine must not enter the generated-HTML trust registry:\n{html}",
    );
    assert!(!html.contains("WIKIJUMPWIKIDOTCOMPATHTML"), "{html}");
}

#[tokio::test]
async fn listpages_reverse_yes_reverses_the_selected_ordered_rows() {
    const TAG: &str = "verification-listpages-reverse-yes";
    const INDEX_SLUG: &str = "fixture-listpages-reverse-yes-index";
    const ALPHA_SLUG: &str = "fixture-listpages-reverse-yes-alpha";
    const BETA_SLUG: &str = "fixture-listpages-reverse-yes-beta";
    const GAMMA_SLUG: &str = "fixture-listpages-reverse-yes-gamma";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (slug, title) in [
        (ALPHA_SLUG, "Fixture ListPages Reverse Alpha"),
        (BETA_SLUG, "Fixture ListPages Reverse Beta"),
        (GAMMA_SLUG, "Fixture ListPages Reverse Gamma"),
    ] {
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            slug,
            title,
            "ListPages reverse target.",
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision, &[TAG]).await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Reverse Index",
        concat!(
            "[[module ListPages tags=\"+verification-listpages-reverse-yes\" order=\"name asc\" reverse=\"yes\" limit=\"3\"]]\n",
            "* %%slug%%\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    let gamma = html
        .find(GAMMA_SLUG)
        .expect("reverse=yes should render the last ascending row");
    let beta = html
        .find(BETA_SLUG)
        .expect("reverse=yes should render the middle ascending row");
    let alpha = html
        .find(ALPHA_SLUG)
        .expect("reverse=yes should render the first ascending row");
    assert!(
        gamma < beta && beta < alpha,
        "reverse=yes should reverse the selected ascending rows:\n{html}"
    );
}

#[tokio::test]
async fn listpages_reverse_boolean_coercion_matches_live_wikidot() {
    const TAG: &str = "verification-listpages-reverse-booleans";
    const INDEX_SLUG: &str = "fixture-listpages-reverse-booleans-index";
    const ALPHA_SLUG: &str = "fixture-listpages-reverse-booleans-alpha";
    const BETA_SLUG: &str = "fixture-listpages-reverse-booleans-beta";
    const GAMMA_SLUG: &str = "fixture-listpages-reverse-booleans-gamma";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (slug, title) in [
        (ALPHA_SLUG, "Fixture ListPages Reverse Boolean Alpha"),
        (BETA_SLUG, "Fixture ListPages Reverse Boolean Beta"),
        (GAMMA_SLUG, "Fixture ListPages Reverse Boolean Gamma"),
    ] {
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            slug,
            title,
            "ListPages reverse boolean target.",
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision, &[TAG]).await;
    }

    let source = ["yes", "true", "no", "false", "invalid", ""]
        .into_iter()
        .map(|value| {
            let marker = if value.is_empty() { "EMPTY" } else { value };
            format!(
                "BEGIN_{marker}\n[[module ListPages tags=\"+{TAG}\" order=\"name asc\" reverse=\"{value}\" separate=\"no\" limit=\"3\"]]\n{marker}:%%slug%%|\n[[/module]]\nEND_{marker}"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Reverse Boolean Index",
        &source,
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    for (value, expected) in [
        ("yes", [GAMMA_SLUG, BETA_SLUG, ALPHA_SLUG]),
        ("true", [GAMMA_SLUG, BETA_SLUG, ALPHA_SLUG]),
        ("no", [ALPHA_SLUG, BETA_SLUG, GAMMA_SLUG]),
        ("false", [ALPHA_SLUG, BETA_SLUG, GAMMA_SLUG]),
        ("invalid", [ALPHA_SLUG, BETA_SLUG, GAMMA_SLUG]),
        ("EMPTY", [ALPHA_SLUG, BETA_SLUG, GAMMA_SLUG]),
    ] {
        let begin = html.find(&format!("BEGIN_{value}")).unwrap_or_else(|| {
            panic!("missing begin marker for reverse={value}:\n{html}")
        });
        let end = html
            .find(&format!("END_{value}"))
            .unwrap_or_else(|| panic!("missing end marker for reverse={value}:\n{html}"));
        let segment = &html[begin..end];
        let first = segment.find(expected[0]).unwrap_or_else(|| {
            panic!("missing first row for reverse={value}:\n{segment}")
        });
        let second = segment.find(expected[1]).unwrap_or_else(|| {
            panic!("missing second row for reverse={value}:\n{segment}")
        });
        let third = segment.find(expected[2]).unwrap_or_else(|| {
            panic!("missing third row for reverse={value}:\n{segment}")
        });
        assert!(
            first < second && second < third,
            "reverse={value} rendered the wrong row order:\n{segment}"
        );
        assert!(
            !segment.contains("[[module ListPages"),
            "reverse={value} must execute instead of failing closed:\n{segment}"
        );
    }
}

#[tokio::test]
async fn listpages_link_to_dot_selects_links_to_the_current_page() {
    const INDEX_SLUG: &str = "fixture-listpages-link-to-current-index";
    const LINKER_SLUG: &str = "fixture-listpages-link-to-current-linker";
    const UNRELATED_SLUG: &str = "fixture-listpages-link-to-current-unrelated";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let index_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Link To Current Index",
        "ListPages holder before its incoming link is compiled.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        LINKER_SLUG,
        "Fixture ListPages Link To Current Linker",
        &format!("[[[{INDEX_SLUG}|current holder link]]]"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        UNRELATED_SLUG,
        "Fixture ListPages Link To Current Unrelated",
        "This page does not link to the holder.",
    )
    .await;
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(INDEX_SLUG)),
    );
    run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
            "last_revision_id": index_revision,
            "revision_comments": "activate ListPages link_to current fixture",
            "user_id": ADMIN_USER_ID,
            "wikitext": concat!(
            "[[module ListPages category=\"*\" link_to=\".\" order=\"name\" separate=\"no\" wrapper=\"no\"]]\n",
            "%%fullname%%|\n",
            "[[/module]]",
            ),
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("holder edit should create a revision");

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(
        html.contains(LINKER_SLUG),
        "link_to=\".\" must select a page that links to the current holder:\n{html}"
    );
    assert!(
        !html.contains(UNRELATED_SLUG),
        "link_to=\".\" must not select a page without that outgoing link:\n{html}"
    );
    assert!(
        !html.contains("[[module ListPages"),
        "link_to=\".\" must execute through the saved-page public seam:\n{html}"
    );
}

#[tokio::test]
async fn listpages_metadata_selectors_honor_target_view_permissions() {
    const PRIVATE_CATEGORY: &str = "fixture-listpages-metadata-private";
    const PRIVATE_PARENT: &str = "fixture-listpages-metadata-private:parent";
    const PUBLIC_CHILD: &str = "fixture-listpages-metadata-public:child";
    const PUBLIC_PARENT: &str = "fixture-listpages-metadata-public:parent";
    const PUBLIC_COUNT_CHILD: &str = "fixture-listpages-metadata-public:count-child";
    const PRIVATE_COUNT_CHILD: &str = "fixture-listpages-metadata-private:count-child";
    const PUBLIC_LINK_TARGET: &str = "fixture-listpages-metadata-public-link-target";
    const PRIVATE_LINK_TARGET: &str = "fixture-listpages-metadata-private:link-target";
    const PUBLIC_LINK_SOURCE: &str = "fixture-listpages-metadata-public-link-source";
    const PRIVATE_LINK_SOURCE: &str = "fixture-listpages-metadata-private-link-source";
    const PUBLIC_INCLUDE_SOURCE: &str =
        "fixture-listpages-metadata-public-include-source";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;

    for (slug, title) in [
        (PRIVATE_PARENT, "Private ListPages metadata parent"),
        (PUBLIC_CHILD, "Public ListPages metadata child"),
        (PUBLIC_PARENT, "Public ListPages metadata parent"),
        (PUBLIC_COUNT_CHILD, "Public ListPages metadata count child"),
        (
            PRIVATE_COUNT_CHILD,
            "Private ListPages metadata count child",
        ),
        (PUBLIC_LINK_TARGET, "Public ListPages metadata link target"),
        (
            PRIVATE_LINK_TARGET,
            "Private ListPages metadata link target",
        ),
        (PUBLIC_LINK_SOURCE, "Public ListPages metadata link source"),
        (
            PRIVATE_LINK_SOURCE,
            "Private ListPages metadata link source",
        ),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, "fixture body.")
            .await;
    }
    create_listpages_test_page(
        &mut runner,
        site_id,
        PUBLIC_INCLUDE_SOURCE,
        "Public ListPages metadata include source",
        &format!("[[include {PUBLIC_LINK_TARGET}]]"),
    )
    .await;

    for slug in [PRIVATE_PARENT, PRIVATE_COUNT_CHILD, PRIVATE_LINK_TARGET] {
        set_listpages_test_category_slug(&runner, site_id, slug, PRIVATE_CATEGORY).await;
    }
    set_listpages_test_parent(&mut runner, site_id, PUBLIC_CHILD, PRIVATE_PARENT).await;
    set_listpages_test_parent(&mut runner, site_id, PUBLIC_COUNT_CHILD, PUBLIC_PARENT)
        .await;
    set_listpages_test_parent(&mut runner, site_id, PRIVATE_COUNT_CHILD, PUBLIC_PARENT)
        .await;

    for (source, target) in [
        (PUBLIC_LINK_SOURCE, PUBLIC_LINK_TARGET),
        (PRIVATE_LINK_SOURCE, PRIVATE_LINK_TARGET),
    ] {
        set_mutation_request_context(
            &mut runner,
            ADMIN_USER_ID,
            site_id,
            Reference::Slug(Cow::Borrowed(source)),
        );
        let page_id = listpages_test_page_id(&runner, site_id, source).await;
        let page = PageTable::find_by_id(page_id)
            .one(runner.context().transaction())
            .await
            .expect("ListPages metadata link source lookup should not fail")
            .expect("ListPages metadata link source should exist");
        let source_revision = page
            .latest_revision_id
            .expect("ListPages metadata link source should have a revision");
        run_endpoint!(
            runner,
            page_edit,
            json!({
                "site_id": site_id,
                "page": source,
                "last_revision_id": source_revision,
                "revision_comments": "add ListPages metadata link fixture",
                "user_id": ADMIN_USER_ID,
                "wikitext": format!("[[[{target}|linked target]]]") ,
                "ip_address": common::IP_ADDRESS,
            }),
        )
        .expect("ListPages metadata link source edit should succeed");
    }

    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: None,
    });
    let private_link_target = PageTable::find()
        .filter(page::Column::Slug.eq(PRIVATE_LINK_TARGET))
        .one(runner.context().transaction())
        .await
        .expect("private link target lookup should not fail")
        .expect("private link target should exist");
    let private_link_view = PermissionService::check_user_can(
        runner.context(),
        &CheckPermissionContext {
            user_id: None,
            site_id,
            page_reference: Some(Reference::Id(private_link_target.page_id)),
        },
        Permission {
            resource_type: Resource::Page,
            resource_category: Some(Reference::Id(private_link_target.page_category_id)),
            action: Action::View,
        },
    )
    .await
    .expect("private link target permission check should not fail");
    assert!(
        !private_link_view,
        "private link target must be denied anonymously"
    );
    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Anonymous ListPages metadata permission preview",
        format!(
            concat!(
                "[[module ListPages category=\"*\" fullname=\"{PUBLIC_CHILD}\" separate=\"no\" wrapper=\"no\"]]",
                "PARENT=%%parent_fullname%%",
                "[[/module]]\n",
                "[[module ListPages category=\"*\" fullname=\"{PUBLIC_PARENT}\" separate=\"no\" wrapper=\"no\"]]",
                "CHILDREN=%%children%%",
                "[[/module]]\n",
                "[[module ListPages category=\"*\" link_to=\"{PUBLIC_LINK_TARGET}\" separate=\"no\" wrapper=\"no\"]]",
                "PUBLIC_LINK=%%slug%%",
                "[[/module]]\n",
                "[[module ListPages category=\"*\" link_to=\"{PRIVATE_LINK_TARGET}\" separate=\"no\" wrapper=\"no\"]]",
                "PRIVATE_LINK=%%slug%%",
                "[[/module]]\n",
                "[[[{PUBLIC_LINK_TARGET}|Public link]]]\n",
                "[[[{PRIVATE_LINK_TARGET}|Private link]]]",
            ),
            PUBLIC_CHILD = PUBLIC_CHILD,
            PUBLIC_PARENT = PUBLIC_PARENT,
            PUBLIC_LINK_TARGET = PUBLIC_LINK_TARGET,
            PRIVATE_LINK_TARGET = PRIVATE_LINK_TARGET,
        ),
    )
    .await
    .expect("anonymous ListPages metadata permission preview should render");
    let html = preview.html_output.body;

    assert!(
        html.contains("PARENT=") && !html.contains(PRIVATE_PARENT),
        "a private parent fullname must not cross the ListPages permission boundary:\n{html}",
    );
    assert!(
        html.contains("CHILDREN=1") && !html.contains("CHILDREN=2"),
        "%%children%% must count only viewable direct children:\n{html}",
    );
    assert!(
        html.contains(&format!("PUBLIC_LINK={PUBLIC_LINK_SOURCE}"))
            && !html.contains(&format!("PRIVATE_LINK={PRIVATE_LINK_SOURCE}"))
            && !html.contains(PUBLIC_INCLUDE_SOURCE),
        "link_to must select public links only, without private-target or include metadata:\n{html}",
    );
    assert!(
        html.contains(&format!(
            r#"<a href="/{PUBLIC_LINK_TARGET}">Public link</a>"#
        )),
        "a viewable ordinary link should remain a normal link:\n{html}",
    );
    assert!(
        html.contains(&format!(
            r#"<a class="newpage" href="/{PRIVATE_LINK_TARGET}">Private link</a>"#
        )),
        "a private ordinary link should be indistinguishable from a missing link:\n{html}",
    );
}

#[tokio::test]
async fn listpages_total_does_not_expose_raw_private_candidate_threshold() {
    const PRIVATE_CATEGORY: &str = "fixture-listpages-total-threshold-private";
    const PUBLIC_TARGET: &str = "fixture-listpages-total-threshold-public";
    const PRIVATE_ROW_COUNT: i32 = 250;

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        PUBLIC_TARGET,
        "Public ListPages total threshold target",
        "public target",
    )
    .await;
    let private_category_id =
        CategoryService::get_or_create(runner.context(), site_id, PRIVATE_CATEGORY)
            .await
            .expect("private total threshold category should exist")
            .category_id;
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "INSERT INTO page (
                created_at, updated_at, deleted_at, from_wikidot, site_id,
                latest_revision_id, page_category_id, slug, discussion_thread_id, layout
             )
             SELECT NOW(), NULL, NULL, false, $1, NULL, $2,
                    'fixture-listpages-total-threshold-private-' || series, NULL, 'wikidot'
             FROM generate_series(1, $3) AS series",
            [
                Value::from(site_id),
                Value::from(private_category_id),
                Value::from(PRIVATE_ROW_COUNT),
            ],
        ))
        .await
        .expect("private total threshold rows should be inserted");

    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: None,
    });
    let preview = RenderService::render_wikidot_page_preview(
        runner.context(),
        site_id,
        "Anonymous ListPages total threshold preview",
        "[[module ListPages category=\"*\" name=\"fixture-listpages-total-threshold-*\" separate=\"no\" wrapper=\"no\"]]TOTAL=%%total%%[[/module]]".to_owned(),
    )
    .await
    .expect("anonymous ListPages total threshold preview should render");
    let html = preview.html_output.body;
    assert!(
        html.contains("TOTAL=1") && !html.contains("%%total%%"),
        "%%total%% must count visible rows without preserving based on raw private candidates:\n{html}",
    );
}

#[tokio::test]
async fn listpages_index_remains_absolute_after_offset() {
    const TAG: &str = "verification-listpages-offset-index";
    const INDEX_SLUG: &str = "fixture-listpages-offset-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (slug, title) in [
        (
            "fixture-listpages-offset-index-alpha",
            "Fixture ListPages Offset Index Alpha",
        ),
        (
            "fixture-listpages-offset-index-beta",
            "Fixture ListPages Offset Index Beta",
        ),
        (
            "fixture-listpages-offset-index-gamma",
            "Fixture ListPages Offset Index Gamma",
        ),
        (
            "fixture-listpages-offset-index-delta",
            "Fixture ListPages Offset Index Delta",
        ),
    ] {
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            slug,
            title,
            "ListPages offset index target.",
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision, &[TAG]).await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Offset Index",
        concat!(
            "[[module ListPages tags=\"+verification-listpages-offset-index\" order=\"name asc\" offset=\"1\" limit=\"3\"]]\n",
            "%%index%%:%%slug%%\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    let beta = html
        .find("2:fixture-listpages-offset-index-beta")
        .expect("the first offset row should keep its absolute index");
    let delta = html
        .find("3:fixture-listpages-offset-index-delta")
        .expect("the second offset row should keep its absolute index");
    let gamma = html
        .find("4:fixture-listpages-offset-index-gamma")
        .expect("the third offset row should keep its absolute index");
    assert!(
        beta < delta && delta < gamma,
        "offset ListPages rows should retain their pre-offset indexes:\n{html}"
    );
    assert!(
        !html.contains("1:fixture-listpages-offset-index-beta"),
        "the selected post-offset row must not be renumbered from one:\n{html}"
    );
}

#[tokio::test]
async fn listpages_link_uses_the_unsuffixed_wikidot_page_url() {
    const TAG: &str = "verification-listpages-link-fullname";
    const TARGET_SLUG: &str = "component:fixture-listpages-link-fullname-target";
    const INDEX_SLUG: &str = "fixture-listpages-link-fullname-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let target_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Link Fullname Target",
        "ListPages link/fullname target.",
    )
    .await;
    set_listpages_test_tags(&mut runner, site_id, TARGET_SLUG, target_revision, &[TAG])
        .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Link Fullname Index",
        concat!(
            "[[module ListPages category=\"*\" tags=\"+verification-listpages-link-fullname\" limit=\"1\"]]\n",
            "PLAIN=%%link%%\n",
            "[[[%%link%%|absolute link]]]\n",
            "[[[%%fullname%%|qualified name]]]\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(
        html.contains(&format!(">http://scp-wiki.wikidot.com/{TARGET_SLUG}</a>")),
        "plain %%link%% must expose Wikidot's exact page URL:\n{html}"
    );
    assert!(
        html.contains(&format!(
            "href=\"http://scp-wiki.wikidot.com/{TARGET_SLUG}\""
        )),
        "linked %%link%% must use Wikidot's exact page URL:\n{html}"
    );
    assert!(
        html.contains(&format!("href=\"/{TARGET_SLUG}\"")),
        "%%fullname%% must remain the category-qualified internal page name:\n{html}"
    );
    assert!(
        !html.contains("/noredirect/true"),
        "Wikidot does not append a noredirect suffix to %%link%%:\n{html}"
    );
}

#[tokio::test]
async fn listpages_date_formats_are_deferred_to_the_wikidot_client_phase() {
    const TARGET_SLUG: &str = "fixture-listpages-date-phase-target";
    const INDEX_SLUG: &str = "fixture-listpages-date-phase-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Date Phase Target",
        "Date phase target.",
    )
    .await;
    set_listpages_test_created_at(
        &runner,
        site_id,
        TARGET_SLUG,
        OffsetDateTime::from_unix_timestamp(1_216_474_620)
            .expect("live-evidenced timestamp is valid"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Date Phase Index",
        concat!(
            "[[module ListPages fullname=\"fixture-listpages-date-phase-target\" separate=\"no\" wrapper=\"no\"]]\n",
            "BARE=%%created_at%%\n",
            "YEAR=%%created_at|%Y%%\n",
            "UNKNOWN=%%created_at|x%%\n",
            "MODIFIER=%%created_at|%Y|agohover%%\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    // This helper calls saved-page `page_get`, whose ODate phase uses the
    // saved-page timezone/text contract. Anonymous PagePreview uses the
    // separate UTC/no-comma contract covered by the renderer unit tests and
    // the live PagePreview evidence.
    assert_eq!(
        html.matches(">19 Jul 2008, 22:37</span>").count(),
        4,
        "the raw response must retain default date text for every format:\n{html}",
    );
    for encoded_format in [
        "format_%25e%20%25b%20%25Y%2C%20%25H%3A%25M",
        "format_%25Y",
        "format_x",
        "format_%25Y%7Cagohover",
    ] {
        assert!(
            html.contains(encoded_format),
            "the requested client format must survive in the ODate class {encoded_format:?}:\n{html}",
        );
    }
    assert!(
        !html.contains(">2008</span>") && !html.contains(">x</span>"),
        "custom format payloads must not execute in the server response:\n{html}",
    );
}

#[tokio::test]
async fn listpages_structural_identity_and_site_variables_match_live_wikidot() {
    const PARENT_SLUG: &str = "fixture-listpages-variables-parent";
    const TARGET_SLUG: &str = "fixture-listpages-variables:target";
    const INDEX_SLUG: &str = "fixture-listpages-variables-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist")
        .site;
    let site_id = site.site_id;
    let admin = UserTable::find_by_id(ADMIN_USER_ID)
        .one(runner.context().transaction())
        .await
        .expect("test administrator lookup should succeed")
        .expect("test administrator should exist");

    create_listpages_test_page(
        &mut runner,
        site_id,
        PARENT_SLUG,
        "Fixture ListPages Variables Parent",
        "Parent fixture.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Variables Target",
        "Target fixture.",
    )
    .await;
    set_listpages_test_parent(&mut runner, site_id, TARGET_SLUG, PARENT_SLUG).await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Variables Index",
        concat!(
            "[[module ListPages category=\"*\" fullname=\"fixture-listpages-variables:target\" separate=\"no\" wrapper=\"no\"]]\n",
            "created_by_id=%%created_by_id%%\n",
            "updated_by_unix=%%updated_by_unix%%\n",
            "updated_by_id=%%updated_by_id%%\n",
            "name=%%name%%\n",
            "page_name=%%page_name%%\n",
            "fullname=%%fullname%%\n",
            "full_page_name=%%full_page_name%%\n",
            "parent_name=%%parent_name%%\n",
            "parent_category=%%parent_category%%\n",
            "parent_fullname=%%parent_fullname%%\n",
            "parent_title=%%parent_title%%\n",
            "parent_title_linked=%%parent_title_linked%%\n",
            "total_or_limit=%%total_or_limit%%\n",
            "site_title=%%site_title%%\n",
            "site_name=%%site_name%%\n",
            "site_domain=%%site_domain%%\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    for expected in [
        format!("created_by_id={ADMIN_USER_ID}"),
        format!("updated_by_unix={}", admin.slug),
        format!("updated_by_id={ADMIN_USER_ID}"),
        "name=target".to_owned(),
        "page_name=target".to_owned(),
        format!("fullname={TARGET_SLUG}"),
        format!("full_page_name={TARGET_SLUG}"),
        format!("parent_name={PARENT_SLUG}"),
        "parent_category=".to_owned(),
        format!("parent_fullname={PARENT_SLUG}"),
        "parent_title=Fixture ListPages Variables Parent".to_owned(),
        format!("href=\"/{PARENT_SLUG}\""),
        "total_or_limit=1".to_owned(),
        format!("site_title={}", site.name),
        format!("site_name={}", site.slug),
        format!("site_domain={}.wikidot.com", site.slug),
    ] {
        assert!(
            html.contains(&expected),
            "ListPages live-evidenced variable output should contain {expected:?}:\n{html}"
        );
    }
    for unresolved in [
        "%%created_by_id%%",
        "%%updated_by_unix%%",
        "%%updated_by_id%%",
        "%%page_name%%",
        "%%full_page_name%%",
        "%%parent_name%%",
        "%%parent_category%%",
        "%%parent_title%%",
        "%%parent_title_linked%%",
        "%%total_or_limit%%",
        "%%site_title%%",
        "%%site_name%%",
    ] {
        assert!(
            !html.contains(unresolved),
            "ListPages must not leave the live-evidenced variable literal: {unresolved}\n{html}"
        );
    }
}

#[tokio::test]
async fn listpages_preview_summary_and_content_aliases_match_live_wikidot() {
    const TARGET_SLUG: &str = "fixture-listpages-content-variables-target";
    const INDEX_SLUG: &str = "fixture-listpages-content-variables-index";
    const TARGET_SOURCE: &str =
        "First paragraph alpha beta.\n\nSecond paragraph final.\n\nThird paragraph.";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Content Variables Target",
        TARGET_SOURCE,
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Content Variables Index",
        concat!(
            "[[module ListPages category=\"*\" fullname=\"fixture-listpages-content-variables-target\" separate=\"no\" wrapper=\"no\"]]\n",
            "preview=%%preview%%\n",
            "preview_17=%%preview(17)%%\n",
            "summary=%%summary%%\n",
            "first_paragraph=%%first_paragraph%%\n",
            "description=%%description%%\n",
            "short=%%short%%\n",
            "text=%%text%%\n",
            "long=%%long%%\n",
            "body=%%body%%\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    for expected in [
        "<span style=\"white-space: pre-wrap;\">First paragraph alpha beta. Second paragraph final. Third paragraph.</span>",
        "<span style=\"white-space: pre-wrap;\">First paragraph...</span>",
        "summary=First paragraph alpha beta.",
        "first_paragraph=First paragraph alpha beta.",
        "description=First paragraph alpha beta.",
        "short=First paragraph alpha beta.",
        "text=First paragraph alpha beta.",
        "long=First paragraph alpha beta.",
        "body=First paragraph alpha beta.",
        "Second paragraph final.",
        "Third paragraph.",
    ] {
        assert!(
            html.contains(expected),
            "ListPages live-evidenced content variable output should contain {expected:?}:\n{html}"
        );
    }
    for unresolved in [
        "%%preview%%",
        "%%preview(17)%%",
        "%%summary%%",
        "%%first_paragraph%%",
        "%%description%%",
        "%%short%%",
        "%%text%%",
        "%%long%%",
        "%%body%%",
    ] {
        assert!(
            !html.contains(unresolved),
            "ListPages must not leave the live-evidenced content variable literal: {unresolved}\n{html}"
        );
    }
}

#[tokio::test]
async fn listpages_variable_suffixes_are_specific_to_each_variable_family() {
    const TARGET_SLUG: &str = "fixture-listpages-variable-suffix-target";
    const INDEX_SLUG: &str = "fixture-listpages-variable-suffix-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Variable Suffix Target",
        "First section.\n=====\nSecond section.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Variable Suffix Index",
        concat!(
            "[[module ListPages fullname=\"fixture-listpages-variable-suffix-target\" separate=\"no\" wrapper=\"no\"]]\n",
            "INVALID=%%title{1}%%|%%fullname(1)%%|%%link|x%%|%%site_name{1}(2)|x%%|%%rating(1)%%|%%comments{1}%%|%%index|x%%|%%total{1}%%|%%created_at{1}(2)|x%%|%%preview{17}%%\n",
            "VALID=%%title%%|%%content{2}%%|%%preview(5)%%|%%created_at|%Y%%\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    for literal in [
        "%%title{1}%%",
        "%%fullname(1)%%",
        "%%link|x%%",
        "%%site_name{1}(2)|x%%",
        "%%rating(1)%%",
        "%%comments{1}%%",
        "%%index|x%%",
        "%%total{1}%%",
        "%%created_at{1}(2)|x%%",
        "%%preview{17}%%",
    ] {
        assert!(
            html.contains(literal),
            "an unsupported suffix must preserve its complete token {literal:?}:\n{html}",
        );
    }
    for expected in [
        "Fixture ListPages Variable Suffix Target",
        "Second section.",
        "style=\"white-space: pre-wrap;\"",
        "format_%25Y",
    ] {
        assert!(
            html.contains(expected),
            "the valid variable-specific suffix should still render {expected:?}:\n{html}",
        );
    }
    for valid_token in [
        "%%title%%",
        "%%content{2}%%",
        "%%preview(5)%%",
        "%%created_at|%Y%%",
    ] {
        assert!(
            !html.contains(valid_token),
            "a supported variable-specific form must substitute {valid_token:?}:\n{html}",
        );
    }
}

#[tokio::test]
async fn listpages_preview_uses_rendered_plain_text_and_legacy_word_limits() {
    const TARGET_SLUG: &str = "fixture-listpages-rendered-preview-target";
    const INDEX_SLUG: &str = "fixture-listpages-rendered-preview-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Rendered Preview Target",
        concat!(
            "[[>]]\n",
            "[[module Rate]]\n",
            "[[/>]]\n",
            "[[div class=\"preview\"]]SCP-002 in its containment area[[/div]]\n",
            "**Item #:** SCP-002\n",
            "**Object Class:** Euclid\n",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Rendered Preview Index",
        concat!(
            "[[module ListPages fullname=\"fixture-listpages-rendered-preview-target\" separate=\"no\" wrapper=\"no\"]]\n",
            "DEFAULT=%%preview%%\n",
            "N0=%%preview(0)%%\n",
            "N1=%%preview(1)%%\n",
            "N2=%%preview(2)%%\n",
            "N5=%%preview(5)%%\n",
            "N17=%%preview(17)%%\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    for expected in [
        "DEFAULT=<span style=\"white-space: pre-wrap;\">SCP-002 in its containment area Item #: SCP-002 Object Class: Euclid</span>",
        "N0=<span style=\"white-space: pre-wrap;\">....</span>",
        "N1=<span style=\"white-space: pre-wrap;\">.....</span>",
        "N2=<span style=\"white-space: pre-wrap;\">......</span>",
        "N5=<span style=\"white-space: pre-wrap;\">...</span>",
        "N17=<span style=\"white-space: pre-wrap;\">SCP-002 in its...</span>",
    ] {
        assert!(
            html.contains(expected),
            "the rendered preview must contain the live-evidenced value {expected:?}:\n{html}",
        );
    }
    assert!(
        !html.contains("[[module") && !html.contains("[[div"),
        "preview text must not expose selected-page source syntax:\n{html}",
    );
}

#[tokio::test]
async fn listpages_summary_aliases_cover_the_first_section_but_first_paragraph_does_not()
{
    const TARGET_SLUG: &str = "fixture-listpages-summary-family-target";
    const INDEX_SLUG: &str = "fixture-listpages-summary-family-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Summary Family Target",
        concat!(
            "FIRST-PARAGRAPH\n\n",
            "SECOND-SUMMARY-MARKER\n",
            "====\n",
            "EXCLUDED-SECOND-SECTION",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Summary Family Index",
        concat!(
            "[[module ListPages fullname=\"fixture-listpages-summary-family-target\" separate=\"no\" wrapper=\"no\"]]\n",
            "SUMMARY-BEGIN %%summary%% SUMMARY-END\n",
            "FIRST-BEGIN %%first_paragraph%% FIRST-END\n",
            "DESCRIPTION-BEGIN %%description%% DESCRIPTION-END\n",
            "SHORT-BEGIN %%short%% SHORT-END\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert_eq!(
        html.matches("SECOND-SUMMARY-MARKER").count(),
        3,
        "summary, description, and short must include the second paragraph of section one:\n{html}",
    );
    let first = html
        .split_once("FIRST-BEGIN")
        .and_then(|(_, suffix)| suffix.split_once("FIRST-END"))
        .map(|(value, _)| value)
        .expect("first-paragraph sentinels should survive rendering");
    assert!(
        first.contains("FIRST-PARAGRAPH") && !first.contains("SECOND-SUMMARY-MARKER"),
        "first_paragraph must use its own paragraph boundary:\n{html}",
    );
    assert!(
        !html.contains("EXCLUDED-SECOND-SECTION"),
        "none of the summary-family variables may cross the first content-section separator:\n{html}",
    );
}

#[tokio::test]
async fn listpages_plain_content_executes_selected_page_includes_only() {
    const COMPONENT_SLUG: &str = "component:listpages-plain-content-include";
    const TARGET_SLUG: &str = "fixture-listpages-plain-content-target";
    const INDEX_SLUG: &str = "fixture-listpages-plain-content-index";
    const INCLUDE_MARKER: &str = "LISTPAGES_PLAIN_CONTENT_INCLUDE";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        COMPONENT_SLUG,
        "Fixture ListPages Plain Content Include",
        INCLUDE_MARKER,
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Plain Content Target",
        &format!("FIRST-SECTION\n[[include {COMPONENT_SLUG}]]\n=====\nSECOND-SECTION"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Plain Content Index",
        &format!(
            concat!(
                "[[module ListPages fullname=\"{TARGET_SLUG}\" separate=\"no\" wrapper=\"no\"]]\n",
                "PLAIN-BEGIN|%%content%%|PLAIN-END\n",
                "SECTION-BEGIN|%%content{{1}}%%|SECTION-END\n",
                "[[/module]]",
            ),
            TARGET_SLUG = TARGET_SLUG,
        ),
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("ListPages plain-content index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled ListPages plain-content body should be available");

    assert_eq!(
        html.matches(INCLUDE_MARKER).count(),
        1,
        "plain content must execute the selected page include exactly once:\n{html}",
    );
    assert_eq!(
        html.matches(&format!("[[include {COMPONENT_SLUG}]]"))
            .count(),
        1,
        "numbered content must preserve the selected page include literally:\n{html}",
    );
}

#[tokio::test]
async fn listpages_numbered_content_does_not_recursively_expand_selected_page_includes() {
    const INCLUDED_SLUG: &str = "fixture-listpages-content-phase-included";
    const TARGET_SLUG: &str = "fixture-listpages-content-phase-target";
    const INDEX_SLUG: &str = "fixture-listpages-content-phase-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        INCLUDED_SLUG,
        "Fixture ListPages Content Phase Included",
        "EXPANDED-SELECTED-PAGE-INCLUDE",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Content Phase Target",
        concat!(
            "[[>]]\n",
            "[[module Rate]]\n",
            "[[/>]]\n",
            "====\n",
            "SECOND SECTION\n",
            "====\n",
            "[[include fixture-listpages-content-phase-included]]",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Content Phase Index",
        concat!(
            "[[module ListPages fullname=\"fixture-listpages-content-phase-target\" separate=\"no\" wrapper=\"no\"]]\n",
            "FIRST|%%content{1}%%|FIRST-END\n",
            "BEGIN|%%content{3}%%|END\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(
        html.contains("[[include fixture-listpages-content-phase-included]]"),
        "an include-only selected section must remain at Wikidot's authored insertion phase:\n{html}",
    );
    assert!(
        !html.contains("EXPANDED-SELECTED-PAGE-INCLUDE"),
        "ListPages must not grant selected-page content recursive include authority:\n{html}",
    );
    assert!(
        html.contains("[[&gt;]]")
            && html.contains("[[module Rate]]")
            && html.contains("[[/&gt;]]"),
        "selected-page quote and module delimiters must remain at the authored insertion phase:\n{html}",
    );
}

#[tokio::test]
async fn listpages_missing_data_form_variables_stay_literal_without_blocking_rows() {
    const TARGET_SLUG: &str = "fixture-listpages-missing-form-target";
    const INDEX_SLUG: &str = "fixture-listpages-missing-form-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Missing Form Target",
        "Ordinary non-data-form page.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Missing Form Index",
        concat!(
            "[[module ListPages category=\"*\" fullname=\"fixture-listpages-missing-form-target\" separate=\"no\" wrapper=\"no\"]]\n",
            "title=%%title%%\n",
            "data=%%form_data{missing}%%\n",
            "raw=%%form_raw{missing}%%\n",
            "label=%%form_label{missing}%%\n",
            "hint=%%form_hint{missing}%%\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(html.contains("title=Fixture ListPages Missing Form Target"));
    for literal in [
        "%%form_data{missing}%%",
        "%%form_raw{missing}%%",
        "%%form_label{missing}%%",
        "%%form_hint{missing}%%",
    ] {
        assert!(
            html.contains(literal),
            "Wikidot leaves a missing data-form field variable literal: {literal}\n{html}"
        );
    }
    assert!(
        !html.contains("[[module ListPages"),
        "the row must still execute when a data-form field is missing:\n{html}"
    );
}

#[tokio::test]
async fn listpages_data_form_variables_match_live_wikidot() {
    const CATEGORY: &str = "fixture-listpages-form";
    const TEMPLATE_SLUG: &str = "fixture-listpages-form-template";
    const TARGET_SLUG: &str = "fixture-listpages-form:fixture-listpages-form-target";
    const INDEX_SLUG: &str = "fixture-listpages-form-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TEMPLATE_SLUG,
        "Fixture ListPages Form Template",
        concat!(
            "[[form]]\n",
            "fields:\n",
            "  probe:\n",
            "    label: Probe Label\n",
            "    hint: 'Probe hint #1'\n",
            "  choice:\n",
            "    label: Choice Label\n",
            "    hint: Choice hint\n",
            "    type: select\n",
            "    values:\n",
            "      alpha: Alpha Display\n",
            "      beta: Beta Display\n",
            "  empty:\n",
            "    label: Empty Label\n",
            "    hint: Empty hint\n",
            "[[/form]]\n",
            "====\n",
            "PROBE=%%form_data{probe}%%",
        ),
    )
    .await;
    let template_page_id = listpages_test_page_id(&runner, site_id, TEMPLATE_SLUG).await;
    set_listpages_test_category_template_page(
        &runner,
        site_id,
        CATEGORY,
        template_page_id,
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, TEMPLATE_SLUG, CATEGORY).await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Form Target",
        "probe: 'raw probe value'\nchoice: alpha\nempty: ''",
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, TARGET_SLUG, CATEGORY).await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Form Index",
        concat!(
            "[[module ListPages category=\"fixture-listpages-form\" name=\"fixture-listpages-form-target\" limit=\"1\" separate=\"no\" wrapper=\"no\"]]\n",
            "D=%%form_data{probe}%%|R=%%form_raw{probe}%%|L=%%form_label{probe}%%|H=%%form_hint{probe}%%|",
            "DC=%%form_data{choice}%%|RC=%%form_raw{choice}%%|LC=%%form_label{choice}%%|HC=%%form_hint{choice}%%|",
            "DE=%%form_data{empty}%%|RE=%%form_raw{empty}%%|LE=%%form_label{empty}%%|HE=%%form_hint{empty}%%|",
            "DM=%%form_data{missing}%%|RM=%%form_raw{missing}%%|LM=%%form_label{missing}%%|HM=%%form_hint{missing}%%\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(
        html.contains(concat!(
            "D=raw probe value|R=raw probe value|L=Probe Label|H=Probe hint #1|",
            "DC=Alpha Display|RC=alpha|LC=Choice Label|HC=|",
            "DE=|RE=|LE=Empty Label|HE=Empty hint|",
            "DM=|RM=|LM=|HM=",
        )),
        "ListPages data-form variables must match live Wikidot value, label, hint, select, empty, and missing-field behavior:\n{html}",
    );
    for unresolved in [
        "%%form_data{probe}%%",
        "%%form_raw{probe}%%",
        "%%form_label{probe}%%",
        "%%form_hint{probe}%%",
        "%%form_data{missing}%%",
        "%%form_raw{missing}%%",
        "%%form_label{missing}%%",
        "%%form_hint{missing}%%",
    ] {
        assert!(
            !html.contains(unresolved),
            "data-form page variables should resolve rather than remain literal: {unresolved}\n{html}"
        );
    }
}

#[tokio::test]
async fn listpages_checkbox_and_wiki_variables_match_live_wikidot() {
    const CATEGORY: &str = "fixture-listpages-checkbox-wiki";
    const TEMPLATE_SLUG: &str = "fixture-listpages-checkbox-wiki:_template";
    const TARGET_SLUG: &str = "fixture-listpages-checkbox-wiki:target";
    const INDEX_SLUG: &str = "fixture-listpages-checkbox-wiki-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TEMPLATE_SLUG,
        "Fixture ListPages Checkbox Wiki Template",
        concat!(
            "[[form]]\n",
            "fields:\n",
            "  enabled:\n",
            "    label: Enabled\n",
            "    type: checkbox\n",
            "  details:\n",
            "    label: Details\n",
            "    type: wiki\n",
            "[[/form]]",
        ),
    )
    .await;
    let template_page_id = listpages_test_page_id(&runner, site_id, TEMPLATE_SLUG).await;
    set_listpages_test_category_template_page(
        &runner,
        site_id,
        CATEGORY,
        template_page_id,
    )
    .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Checkbox Wiki Target",
        "enabled: '1'\ndetails: \"**Bold**\\n[[[fixture-listpages-checkbox-wiki-missing-start|Home]]]\"",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Checkbox Wiki Index",
        concat!(
            "[[module ListPages category=\"fixture-listpages-checkbox-wiki\" ",
            "fullname=\"fixture-listpages-checkbox-wiki:target\" ",
            "limit=\"1\" separate=\"no\" wrapper=\"no\"]]\n",
            "DATA-WIKI-BEGIN\n",
            "%%form_data{details}%%\n",
            "DATA-WIKI-END\n",
            "RAW-WIKI-BEGIN\n",
            "%%form_raw{details}%%\n",
            "RAW-WIKI-END\n",
            "CHECKBOX=%%form_data{enabled}%%|%%form_raw{enabled}%%\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert_eq!(
        html.matches("<strong>Bold</strong>").count(),
        2,
        "both form_data and form_raw must parse wiki field syntax:\n{html}",
    );
    assert_eq!(
        html.matches(
            r#"<a class="newpage" href="/fixture-listpages-checkbox-wiki-missing-start">Home</a>"#,
        )
            .count(),
        2,
        "both wiki variables must decode the stored newline and internal link:\n{html}",
    );
    assert!(
        html.contains("CHECKBOX=1|1"),
        "checkbox variables must expose the stored digit:\n{html}",
    );
    assert!(
        !html.contains(r"\n[[[start|Home]]]"),
        "stored multiline wiki scalars must be decoded before substitution:\n{html}",
    );
}

#[tokio::test]
async fn listpages_template_selectors_can_use_current_page_data_form_values() {
    const CATEGORY: &str = "fixture-listpages-current-form";
    const TEMPLATE_SLUG: &str = "fixture-listpages-current-form:_template";
    const HOLDER_SLUG: &str = "fixture-listpages-current-form:holder";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TEMPLATE_SLUG,
        "Fixture ListPages Current Form Template",
        concat!(
            "[[form]]\n",
            "fields:\n",
            "  scotland:\n",
            "    label: Scotland tour\n",
            "    type: select\n",
            "    values:\n",
            "      visit: \"Yes\"\n",
            "      novisit: \"No\"\n",
            "  kind:\n",
            "    label: Music type\n",
            "    type: select\n",
            "    values:\n",
            "      folk: Folk\n",
            "      rock: Rock\n",
            "  albums:\n",
            "    label: Albums/CDs released\n",
            "    type: select\n",
            "    values:\n",
            "      \"01\": 1\n",
            "      \"02\": 2\n",
            "      \"09\": 9\n",
            "      \"10\": 10\n",
            "[[/form]]\n",
            "\n",
            "MATCHING\n",
            "[[module ListPages category=\"fixture-listpages-current-form\" ",
            "_scotland=\"%%form_raw{scotland}%%\" _kind=\"%%form_raw{kind}%%\" ",
            "skipCurrent=\"true\" order=\"name\" separate=\"false\" wrapper=\"no\"]]\n",
            "%%page_name%%|%%form_data{kind}%%|%%form_data{scotland}%%\n",
            "[[/module]]\n",
            "ORDER\n",
            "[[module ListPages category=\"fixture-listpages-current-form\" ",
            "skipCurrent=\"true\" order=\"_albums desc\" separate=\"false\" wrapper=\"no\"]]\n",
            "%%page_name%%|%%form_data{albums}%%|%%form_raw{albums}%%\n",
            "[[/module]]\n",
            "CONTENT=%%content%%",
        ),
    )
    .await;
    let template_page_id = listpages_test_page_id(&runner, site_id, TEMPLATE_SLUG).await;
    set_listpages_test_category_template_page(
        &runner,
        site_id,
        CATEGORY,
        template_page_id,
    )
    .await;

    for (slug, title, source) in [
        (
            "fixture-listpages-current-form:folk-visit",
            "Fixture Folk Visit",
            "scotland: visit\nkind: folk\nalbums: '02'\n\nFolk visit body.",
        ),
        (
            "fixture-listpages-current-form:folk-no",
            "Fixture Folk No",
            "scotland: novisit\nkind: folk\nalbums: '09'\n\nFolk no body.",
        ),
        (
            "fixture-listpages-current-form:rock-visit",
            "Fixture Rock Visit",
            "scotland: visit\nkind: rock\nalbums: '10'\n\nRock visit body.",
        ),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, source).await;
    }
    let holder_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER_SLUG,
        "Fixture Current Form Holder",
        "scotland: visit\nkind: folk\nalbums: '01'\n\nHolder body.",
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, HOLDER_SLUG).await;
    assert!(
        html.contains("folk-visit|Folk|Yes"),
        "ListPages selectors inside a data-form category template must resolve current-page form_raw variables before querying:\n{html}",
    );
    for unexpected in ["folk-no|Folk|No", "rock-visit|Rock|Yes"] {
        assert!(
            !html.contains(unexpected),
            "current-page data-form selector should filter out {unexpected:?}:\n{html}",
        );
    }
    let rock = html
        .find("rock-visit|10|10")
        .expect("data-form order should include rock visit row");
    let folk_no = html
        .find("folk-no|9|09")
        .expect("data-form order should include folk-no row");
    let folk_visit = html
        .find("folk-visit|2|02")
        .expect("data-form order should include folk visit row");
    assert!(
        rock < folk_no && folk_no < folk_visit,
        "ListPages order=\"_field desc\" should sort by stored data-form field property while displaying form_data labels:\n{html}",
    );
    assert!(
        html.contains("Holder body."),
        "the data-form page content should still be composed through the template:\n{html}",
    );
    assert!(
        !html.contains("%%form_raw{scotland}%%") && !html.contains("[[module ListPages"),
        "resolved current-page data-form selector variables must not remain literal:\n{html}",
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(HOLDER_SLUG)),
    );
    run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": HOLDER_SLUG,
            "last_revision_id": holder_revision,
            "revision_comments": "change current data-form selector values",
            "user_id": ADMIN_USER_ID,
            "wikitext": "scotland: visit\nkind: rock\nalbums: '01'\n\nEdited holder body.",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("editing the data-form holder should create a revision");

    let edited_html =
        load_listpages_test_compiled_html(&runner, site_id, HOLDER_SLUG).await;
    assert!(
        edited_html.contains("rock-visit|Rock|Yes"),
        "revision compilation must resolve current-page data-form selectors from the edit candidate rather than the previous committed revision:\n{edited_html}",
    );
    for stale in ["folk-visit|Folk|Yes", "folk-no|Folk|No"] {
        assert!(
            !edited_html.contains(stale),
            "the edited revision must not retain a row selected by stale form values ({stale}):\n{edited_html}",
        );
    }
    assert!(
        edited_html.contains("Edited holder body."),
        "the edited candidate body and its ListPages selector must compile from the same source revision:\n{edited_html}",
    );
}

#[tokio::test]
async fn listpages_imported_creator_identity_uses_structured_corpus_provenance() {
    const IMPORT_RUN_ID: i64 = 944_001;
    const TARGET_SLUG: &str = "fixture-listpages-imported-creator-target";
    const INDEX_SLUG: &str = "fixture-listpages-imported-creator-index";
    const CREATOR_ID: i64 = 10_382_659;
    const CREATOR_NAME: &str = "voted-fated-smuggler";
    const CREATOR_SLUG: &str = "voted-fated-smuggler";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture Imported Creator Target",
        "Imported creator target.",
    )
    .await;
    let target_id = listpages_test_page_id(&runner, site_id, TARGET_SLUG).await;
    create_listpages_test_import_run(&runner, site_id, IMPORT_RUN_ID, 1).await;
    mark_imported_page_with_author_snapshot(
        &runner,
        site_id,
        IMPORT_RUN_ID,
        (target_id, TARGET_SLUG, 944_001, CREATOR_NAME),
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
        .expect("imported creator identity provenance should be attached");

    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture Imported Creator Index",
        concat!(
            "[[module ListPages category=\"*\" ",
            "fullname=\"fixture-listpages-imported-creator-target\" ",
            "separate=\"no\" wrapper=\"no\"]]\n",
            "created_by=[%%created_by%%]\n",
            "created_by_unix=[%%created_by_unix%%]\n",
            "created_by_id=[%%created_by_id%%]\n",
            "created_by_linked=[%%created_by_linked%%]\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    for expected in [
        format!("created_by=[{CREATOR_NAME}]"),
        format!("created_by_unix=[{CREATOR_SLUG}]"),
        format!("created_by_id=[{CREATOR_ID}]"),
        format!("http://www.wikidot.com/user:info/{CREATOR_SLUG}"),
        format!("WIKIDOT.page.listeners.userInfo({CREATOR_ID})"),
        format!(
            "src=\"http://www.wikidot.com/avatar.php?userid={CREATOR_ID}&amp;amp;size=small&amp;amp;timestamp="
        ),
        format!(
            r#"style="background-image:url(http://www.wikidot.com/userkarma.php?u={CREATOR_ID})""#
        ),
    ] {
        assert!(
            html.contains(&expected),
            "imported creator identity provenance must drive every lifecycle variable ({expected}):\n{html}",
        );
    }
    assert!(
        !html.contains("[[module ListPages") && !html.contains("%%created_by"),
        "a fully identified imported creator must not make ListPages fail closed:\n{html}",
    );
}

#[tokio::test]
async fn listpages_imported_empty_title_uses_wikidot_page_name_label() {
    const IMPORT_RUN_ID: i64 = 944_005;
    const TARGET_SLUG: &str = "fixtureemptytitle:dfui-a-dfuixp04";
    const INDEX_SLUG: &str = "fixture-listpages-imported-empty-title-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        TARGET_SLUG,
        "Imported empty-title target.",
    )
    .await;
    let target_id = listpages_test_page_id(&runner, site_id, TARGET_SLUG).await;
    create_listpages_test_import_run(&runner, site_id, IMPORT_RUN_ID, 1).await;
    mark_imported_page_with_author_snapshot(
        &runner,
        site_id,
        IMPORT_RUN_ID,
        (target_id, TARGET_SLUG, 944_005, "Fixture Creator"),
    )
    .await;
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "UPDATE wikidot_page_snapshot \
             SET title_shown = '', \
                 meta_json = jsonb_build_object( \
                     'title', '', \
                     'title_shown', '' \
                 ) \
             WHERE page_id = $1",
            [Value::from(target_id)],
        ))
        .await
        .expect("imported empty-title provenance should be attached");

    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture Imported Empty Title Index",
        concat!(
            "[[module ListPages category=\"*\" ",
            "fullname=\"fixtureemptytitle:dfui-a-dfuixp04\" ",
            "separate=\"no\" wrapper=\"no\"]]\n",
            "TITLE=[%%title%%]\n",
            "LINK=[%%title_linked%%]\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(
        html.contains("TITLE=[Dfui a dfuixp04]"),
        "an imported empty title should use Wikidot's page-name label:\n{html}",
    );
    assert!(
        html.contains(">Dfui a dfuixp04</a>"),
        "the linked-title label should use the same Wikidot fallback:\n{html}",
    );
    assert!(
        !html.contains("TITLE=[fixtureemptytitle:dfui-a-dfuixp04]"),
        "the import-only storage fallback must not leak as the visible title:\n{html}",
    );
}

#[tokio::test]
async fn listpages_imported_editor_identity_uses_structured_corpus_provenance() {
    const IMPORT_RUN_ID: i64 = 944_002;
    const TARGET_SLUG: &str = "fixture-listpages-imported-editor-target";
    const INDEX_SLUG: &str = "fixture-listpages-imported-editor-index";
    const EDITOR_ID: i64 = 10_382_659;
    const EDITOR_NAME: &str = "voted-fated-smuggler";
    const EDITOR_SLUG: &str = "voted-fated-smuggler";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture Imported Editor Target",
        "Imported editor target.",
    )
    .await;
    let target_id = listpages_test_page_id(&runner, site_id, TARGET_SLUG).await;
    create_listpages_test_import_run(&runner, site_id, IMPORT_RUN_ID, 1).await;
    mark_imported_page_with_author_snapshot(
        &runner,
        site_id,
        IMPORT_RUN_ID,
        (target_id, TARGET_SLUG, 944_002, "Fixture Creator"),
    )
    .await;
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "UPDATE wikidot_page_snapshot \
             SET updated_by_name = $1, \
                 meta_json = jsonb_build_object( \
                     'updated_by_id', $2::bigint, \
                     'updated_by_unix', $3::text \
                 ) \
             WHERE page_id = $4",
            [
                Value::from(EDITOR_NAME),
                Value::from(EDITOR_ID),
                Value::from(EDITOR_SLUG),
                Value::from(target_id),
            ],
        ))
        .await
        .expect("imported editor identity provenance should be attached");

    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture Imported Editor Index",
        concat!(
            "[[module ListPages category=\"*\" ",
            "fullname=\"fixture-listpages-imported-editor-target\" ",
            "separate=\"no\" wrapper=\"no\"]]\n",
            "updated_by=[%%updated_by%%]\n",
            "updated_by_unix=[%%updated_by_unix%%]\n",
            "updated_by_id=[%%updated_by_id%%]\n",
            "updated_by_linked=[%%updated_by_linked%%]\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    for expected in [
        format!("updated_by=[{EDITOR_NAME}]"),
        format!("updated_by_unix=[{EDITOR_SLUG}]"),
        format!("updated_by_id=[{EDITOR_ID}]"),
        format!("http://www.wikidot.com/user:info/{EDITOR_SLUG}"),
        format!("WIKIDOT.page.listeners.userInfo({EDITOR_ID})"),
    ] {
        assert!(
            html.contains(&expected),
            "imported editor identity provenance must drive every lifecycle variable ({expected}):\n{html}",
        );
    }
    assert!(
        !html.contains("%%updated_by"),
        "a fully identified imported editor must resolve every identity variable:\n{html}",
    );
}

#[tokio::test]
async fn listpages_imported_commenter_identity_uses_structured_corpus_provenance() {
    const IMPORT_RUN_ID: i64 = 944_003;
    const TARGET_SLUG: &str = "fixture-listpages-imported-commenter-target";
    const INDEX_SLUG: &str = "fixture-listpages-imported-commenter-index";
    const COMMENTER_ID: i64 = 10_382_659;
    const COMMENTER_NAME: &str = "voted-fated-smuggler";
    const COMMENTER_SLUG: &str = "voted-fated-smuggler";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture Imported Commenter Target",
        "Imported commenter target.",
    )
    .await;
    let target_id = listpages_test_page_id(&runner, site_id, TARGET_SLUG).await;
    create_listpages_test_import_run(&runner, site_id, IMPORT_RUN_ID, 1).await;
    mark_imported_page_with_author_snapshot(
        &runner,
        site_id,
        IMPORT_RUN_ID,
        (target_id, TARGET_SLUG, 944_003, "Fixture Creator"),
    )
    .await;
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "UPDATE wikidot_page_snapshot \
             SET comments = 1, \
                 commented_at = TIMESTAMPTZ '2026-07-28 06:36:00+00', \
                 commented_by_name = $1, \
                 meta_json = jsonb_build_object( \
                     'commented_by_id', $2::bigint, \
                     'commented_by_unix', $3::text \
                 ) \
             WHERE page_id = $4",
            [
                Value::from(COMMENTER_NAME),
                Value::from(COMMENTER_ID),
                Value::from(COMMENTER_SLUG),
                Value::from(target_id),
            ],
        ))
        .await
        .expect("imported commenter identity provenance should be attached");

    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture Imported Commenter Index",
        concat!(
            "[[module ListPages category=\"*\" ",
            "fullname=\"fixture-listpages-imported-commenter-target\" ",
            "separate=\"no\" wrapper=\"no\"]]\n",
            "comments=[%%comments%%]\n",
            "commented_by=[%%commented_by%%]\n",
            "commented_by_unix=[%%commented_by_unix%%]\n",
            "commented_by_id=[%%commented_by_id%%]\n",
            "commented_by_linked=[%%commented_by_linked%%]\n",
            "commented_at=[%%commented_at%%]\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    for expected in [
        "comments=[1]".to_owned(),
        format!("commented_by=[{COMMENTER_NAME}]"),
        format!("commented_by_unix=[{COMMENTER_SLUG}]"),
        format!("commented_by_id=[{COMMENTER_ID}]"),
        format!("http://www.wikidot.com/user:info/{COMMENTER_SLUG}"),
        format!("WIKIDOT.page.listeners.userInfo({COMMENTER_ID})"),
        "commented_at=[<span class=\"odate time_".to_owned(),
    ] {
        assert!(
            html.contains(&expected),
            "imported commenter identity provenance must drive every last-comment variable ({expected}):\n{html}",
        );
    }
    assert!(
        !html.contains("%%commented_"),
        "a fully identified imported commenter must resolve every identity variable:\n{html}",
    );
}

#[tokio::test]
async fn listpages_no_comment_variables_match_live_wikidot() {
    const TARGET_SLUG: &str = "fixture-listpages-no-comments-target";
    const INDEX_SLUG: &str = "fixture-listpages-no-comments-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages No Comments Target",
        "Ordinary page with no comments.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages No Comments Index",
        concat!(
            "[[module ListPages category=\"*\" fullname=\"fixture-listpages-no-comments-target\" separate=\"no\" wrapper=\"no\"]]\n",
            "comments=[%%comments%%]\n",
            "commented_by=[%%commented_by%%]\n",
            "commented_by_linked=[%%commented_by_linked%%]\n",
            "commented_by_unix=[%%commented_by_unix%%]\n",
            "commented_by_id=[%%commented_by_id%%]\n",
            "commented_at=[%%commented_at%%]\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(
        html.contains("comments=[0]"),
        "an ordinary no-comment page must expose a zero comment count:\n{html}"
    );
    for expected in [
        "commented_by=[]",
        "commented_by_linked=[]",
        "commented_by_unix=[]",
        "commented_by_id=[]",
        "commented_at=[]",
    ] {
        assert!(
            html.contains(expected),
            "a missing last-comment field must render empty ({expected}):\n{html}"
        );
    }
    for unresolved in [
        "%%comments%%",
        "%%commented_by%%",
        "%%commented_by_linked%%",
        "%%commented_by_unix%%",
        "%%commented_by_id%%",
        "%%commented_at%%",
        "[[module ListPages",
    ] {
        assert!(
            !html.contains(unresolved),
            "the live-evidenced no-comment row must execute fully: {unresolved}\n{html}"
        );
    }
}

#[tokio::test]
async fn listpages_last_comment_variables_match_live_wikidot() {
    const TARGET_SLUG: &str = "fixture-listpages-last-comment-target";
    const INDEX_SLUG: &str = "fixture-listpages-last-comment-index";
    const COMMENTER_ID: i64 = 10_382_659;
    const COMMENTER_NAME: &str = "voted-fated-smuggler";
    const COMMENTER_SLUG: &str = "voted-fated-smuggler";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_string(
            transaction.get_database_backend(),
            format!(
                r#"
                INSERT INTO known_user (user_id)
                VALUES ({COMMENTER_ID})
                ON CONFLICT (user_id) DO NOTHING
                "#,
            ),
        ))
        .await
        .expect("commenter known_user fixture should be inserted");
    transaction
        .execute_raw(Statement::from_string(
            transaction.get_database_backend(),
            format!(
                r#"
                INSERT INTO wikidot_user (
                    user_id,
                    created_at,
                    fetched_at,
                    is_deleted,
                    name,
                    slug,
                    karma,
                    is_pro
                ) VALUES (
                    {COMMENTER_ID},
                    TIMESTAMPTZ '2020-01-01T00:00:00Z',
                    TIMESTAMPTZ '2026-07-28T00:00:00Z',
                    FALSE,
                    '{COMMENTER_NAME}',
                    '{COMMENTER_SLUG}',
                    0,
                    FALSE
                )
                "#,
            ),
        ))
        .await
        .expect("commenter wikidot_user fixture should be inserted");
    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Last Comment Target",
        "Page with one controlled discussion comment.",
    )
    .await;
    let target_page_id = listpages_test_page_id(&runner, site_id, TARGET_SLUG).await;

    let group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "ListPages comment fixture group".to_owned(),
            description: "ListPages comment fixture group".to_owned(),
            visible: true,
            sort_index: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("ListPages comment forum group should be created");
    let category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "ListPages comment fixture category".to_owned(),
            description: "ListPages comment fixture category".to_owned(),
            sort_index: None,
            max_nest_level: Some(3),
            per_page_discussion: Some(true),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("ListPages comment forum category should be created");
    let thread = ForumThreadService::create(
        runner.context(),
        CreateForumThread {
            forum_category_id: category.forum_category_id,
            user_id: ADMIN_USER_ID,
            associated_page_id: Some(target_page_id),
            title: "ListPages last comment fixture".to_owned(),
            description: String::new(),
            sticky: false,
            from_wikidot: false,
        },
    )
    .await
    .expect("ListPages comment thread should be created");
    let post = ForumPostService::create(
        runner.context(),
        CreateForumPost {
            forum_thread_id: thread.forum_thread_id,
            parent_post_id: None,
            user_id: COMMENTER_ID,
            title: "ListPages last comment".to_owned(),
            wikitext: "Controlled last comment".to_owned(),
            comments: "create ListPages comment fixture".to_owned(),
            from_wikidot: false,
        },
    )
    .await
    .expect("ListPages comment should be created");
    assert!(post.parser_errors.is_empty());

    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Last Comment Index",
        concat!(
            "[[module ListPages category=\"*\" fullname=\"fixture-listpages-last-comment-target\" separate=\"no\" wrapper=\"no\"]]\n",
            "comments=[%%comments%%]\n",
            "commented_by=[%%commented_by%%]\n",
            "commented_by_linked=[%%commented_by_linked%%]\n",
            "commented_by_unix=[%%commented_by_unix%%]\n",
            "commented_by_id=[%%commented_by_id%%]\n",
            "commented_at=[%%commented_at%%]\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(html.contains("comments=[1]"), "{html}");
    assert!(
        html.contains(&format!("commented_by=[{COMMENTER_NAME}]")),
        "{html}"
    );
    assert!(
        html.contains(&format!("commented_by_unix=[{COMMENTER_SLUG}]")),
        "{html}"
    );
    assert!(
        html.contains(&format!("commented_by_id=[{COMMENTER_ID}]")),
        "{html}"
    );
    assert!(
        html.contains(&format!(
            "http://www.wikidot.com/user:info/{COMMENTER_SLUG}"
        )) && html.contains(&format!("WIKIDOT.page.listeners.userInfo({COMMENTER_ID})")),
        "{html}"
    );
    assert!(
        html.contains("commented_at=[<span class=\"odate time_"),
        "{html}"
    );
}

#[tokio::test]
async fn listpages_star_rating_variables_match_live_wikidot() {
    const CATEGORY: &str = "fixture-listpages-stars";
    const FOUR_SLUG: &str = "fixture-listpages-star-four";
    const HALF_SLUG: &str = "fixture-listpages-star-half";
    const ZERO_SLUG: &str = "fixture-listpages-star-zero";
    const INDEX_SLUG: &str = "fixture-listpages-star-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    set_listpages_test_category_rating_type(&runner, site_id, CATEGORY, "stars").await;

    for (slug, title) in [
        (FOUR_SLUG, "Fixture ListPages Star Four"),
        (HALF_SLUG, "Fixture ListPages Star Half"),
        (ZERO_SLUG, "Fixture ListPages Star Zero"),
    ] {
        create_listpages_test_page(
            &mut runner,
            site_id,
            slug,
            title,
            "Star rating target.",
        )
        .await;
        set_listpages_test_category_slug(&runner, site_id, slug, CATEGORY).await;
    }

    let four_id = listpages_test_page_id(&runner, site_id, FOUR_SLUG).await;
    let half_id = listpages_test_page_id(&runner, site_id, HALF_SLUG).await;
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "INSERT INTO page_vote (from_wikidot, page_id, user_id, rating_system, value) VALUES \
             (false, $1, $2, 'stars', 4), \
             (false, $1, $3, 'stars', 4), \
             (false, $4, $2, 'stars', 5), \
             (false, $4, $3, 'stars', 4)",
            [
                Value::from(four_id),
                Value::from(ADMIN_USER_ID),
                Value::from(SAMPLE_USER_ID),
                Value::from(half_id),
            ],
        ))
        .await
        .expect("star rating fixtures should be inserted");

    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Star Index",
        concat!(
            "[[module ListPages category=\"fixture-listpages-stars\" order=\"name\"]]\n",
            "%%name%% rating=[%%rating%%] votes=[%%rating_votes%%] percent=[%%rating_percent%%]\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(
        html.contains(concat!(
            "fixture-listpages-star-four rating=[",
            "<span class=\"page-rate-list-pages-start\" data-rating=\"4\">4</span>",
            "] votes=[2] percent=[80]",
        )),
        "Wikidot renders a star ListPages score as a protected span and rating_percent as rating*20:\n{html}"
    );
    assert!(
        html.contains(concat!(
            "fixture-listpages-star-half rating=[",
            "<span class=\"page-rate-list-pages-start\" data-rating=\"4.5\">4.5</span>",
            "] votes=[2] percent=[90]",
        )),
        "Wikidot preserves fractional star averages without forcing an integer:\n{html}"
    );
    assert!(
        html.contains(concat!(
            "fixture-listpages-star-zero rating=[",
            "<span class=\"page-rate-list-pages-start\" data-rating=\"0\">0</span>",
            "] votes=[0] percent=[0]",
        )),
        "Wikidot emits an explicit zero state for unrated star rows:\n{html}"
    );
    assert!(
        !html.contains("%%rating_percent%%"),
        "star rating_percent variables must not remain literal:\n{html}"
    );
}

#[tokio::test]
async fn listpages_legacy_updated_author_aliases_match_live_wikidot() {
    const TARGET_SLUG: &str = "fixture-listpages-updated-author-target";
    const INDEX_SLUG: &str = "fixture-listpages-updated-author-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let admin = UserTable::find_by_id(ADMIN_USER_ID)
        .one(runner.context().transaction())
        .await
        .expect("test administrator lookup should succeed")
        .expect("test administrator should exist");

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Updated Author Target",
        "Updated author target.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Updated Author Index",
        concat!(
            "[[module ListPages category=\"*\" fullname=\"fixture-listpages-updated-author-target\" separate=\"no\" wrapper=\"no\"]]\n",
            "updated_by=%%updated_by%%\n",
            "updated_by_linked=%%updated_by_linked%%\n",
            "author_edited=%%author_edited%%\n",
            "user_edited=%%user_edited%%\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(html.contains(&format!("updated_by={}", admin.name)));
    assert!(html.matches(&admin.name).count() >= 4, "{html}");
    for unresolved in ["%%author_edited%%", "%%user_edited%%"] {
        assert!(
            !html.contains(unresolved),
            "legacy updated-author aliases must resolve: {unresolved}\n{html}"
        );
    }
}

#[tokio::test]
async fn listpages_legacy_skip_current_and_tag_target_match_live_wikidot() {
    const PREFIX: &str = "fixture-listpages-legacy-edge";
    const TAG: &str = "verification-listpages-legacy-edge";
    const TARGET_SLUG: &str = "fixture-listpages-legacy-edge-target";
    const HOLDER_SLUG: &str = "fixture-listpages-legacy-edge-holder";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let target_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Legacy Edge Target",
        "Target.",
    )
    .await;
    set_listpages_test_tags(&mut runner, site_id, TARGET_SLUG, target_revision, &[TAG])
        .await;
    let holder_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER_SLUG,
        "Fixture ListPages Legacy Edge Holder",
        &format!(
            concat!(
                "SKIP_YES_START\n",
                "[[module ListPages category=\"*\" name=\"{PREFIX}-*\" tags=\"+{TAG}\" order=\"name\" separate=\"no\" perPage=\"250\" skipCurrent=\"yes\"]]\n",
                "%%name%%|\n",
                "[[/module]]\n",
                "SKIP_YES_END\n",
                "SKIP_TRUE_START\n",
                "[[module ListPages category=\"*\" name=\"{PREFIX}-*\" tags=\"+{TAG}\" order=\"name\" separate=\"no\" perPage=\"250\" skipCurrent=\"TrUe\"]]\n",
                "%%name%%|\n",
                "[[/module]]\n",
                "SKIP_TRUE_END\n",
                "SKIP_NO_START\n",
                "[[module ListPages category=\"*\" name=\"{PREFIX}-*\" tags=\"+{TAG}\" order=\"name\" separate=\"no\" perPage=\"250\" skipCurrent=\"no\"]]\n",
                "%%name%%|\n",
                "[[/module]]\n",
                "SKIP_NO_END\n",
                "SKIP_FALSE_START\n",
                "[[module ListPages category=\"*\" name=\"{PREFIX}-*\" tags=\"+{TAG}\" order=\"name\" separate=\"no\" perPage=\"250\" skipCurrent=\"false\"]]\n",
                "%%name%%|\n",
                "[[/module]]\n",
                "SKIP_FALSE_END\n",
                "SKIP_INVALID_START\n",
                "[[module ListPages category=\"*\" name=\"{PREFIX}-*\" tags=\"+{TAG}\" order=\"name\" separate=\"no\" perPage=\"250\" skipCurrent=\"invalid\"]]\n",
                "%%name%%|\n",
                "[[/module]]\n",
                "SKIP_INVALID_END\n",
                "SAME_TAGS_START\n",
                "[[module ListPages category=\"*\" tags=\"=\" order=\"name\" separate=\"no\" perPage=\"250\"]]\n",
                "%%name%%|\n",
                "[[/module]]\n",
                "SAME_TAGS_END\n",
                "TAG_TARGET_START\n",
                "[[module ListPages category=\"*\" fullname=\"{TARGET_SLUG}\" tagTarget=\"edge-tags\"]]\n",
                "T=%%tags%%|L=%%tags_linked%%|E=%%tags_linked|custom/tag/%%\n",
                "[[/module]]\n",
                "TAG_TARGET_END",
            ),
            PREFIX = PREFIX,
            TAG = TAG,
            TARGET_SLUG = TARGET_SLUG,
        ),
    )
    .await;
    set_listpages_test_tags(&mut runner, site_id, HOLDER_SLUG, holder_revision, &[TAG])
        .await;
    let holder = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": HOLDER_SLUG}),
    )
    .expect("legacy edge holder should exist");
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
    let section = |marker: &str| {
        let start_marker = format!("{marker}_START");
        let end_marker = format!("{marker}_END");
        let start = html
            .find(&start_marker)
            .expect("legacy edge section should start");
        let end = html[start..]
            .find(&end_marker)
            .map(|offset| start + offset)
            .expect("legacy edge section should end");
        &html[start..end]
    };

    for marker in ["SKIP_YES", "SKIP_TRUE", "SAME_TAGS"] {
        let section = section(marker);
        assert!(section.contains(TARGET_SLUG), "{marker}:\n{section}");
        assert!(!section.contains(HOLDER_SLUG), "{marker}:\n{section}");
        assert!(
            !section.contains("[[module ListPages"),
            "{marker}:\n{section}"
        );
    }
    for marker in ["SKIP_NO", "SKIP_FALSE", "SKIP_INVALID"] {
        let section = section(marker);
        assert!(section.contains(TARGET_SLUG), "{marker}:\n{section}");
        assert!(section.contains(HOLDER_SLUG), "{marker}:\n{section}");
        assert!(
            !section.contains("[[module ListPages"),
            "{marker}:\n{section}"
        );
    }
    let tag_target = section("TAG_TARGET");
    assert!(tag_target.contains(TAG), "{tag_target}");
    assert!(
        !tag_target.contains("/system:page-tags/tag/"),
        "tagTarget must replace the default generated tag path:\n{tag_target}"
    );
    assert!(
        tag_target.contains(&format!(r#"href="/edge-tags/tag/{TAG}""#)),
        "tagTarget must affect the generated %%tags_linked%% href:\n{tag_target}",
    );
    assert!(
        tag_target.contains(&format!(r#"href="/custom/tag/{TAG}""#)),
        "an explicit tags_linked prefix must take precedence over tagTarget:\n{tag_target}",
    );
    assert!(
        tag_target.contains(&format!("T={TAG}|")),
        "plain %%tags%% must remain unlinked:\n{tag_target}",
    );
    assert!(!tag_target.contains("[[module ListPages"), "{tag_target}");
}

#[tokio::test]
async fn listpages_section_summary_and_separate_lines_match_live_wikidot() {
    const TARGET_SLUG: &str = "fixture-listpages-section-summary-target";
    const INDEX_SLUG: &str = "fixture-listpages-section-summary-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Section Summary Target",
        "Summary first section.\n====\nSecond content section.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Section Summary Index",
        concat!(
            "[[module ListPages category=\"*\" fullname=\"fixture-listpages-section-summary-target\" separate=\"yes\" prependLine=\"PREPEND_SHOULD_NOT_RENDER\" appendLine=\"APPEND_SHOULD_NOT_RENDER\"]]\n",
            "summary=%%summary%%|description=%%description%%|content1=%%content{1}%%|\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(
        html.contains(
            "summary=Summary first section.|description=Summary first section.|content1=Summary first section.|"
        ),
        "summary aliases must use content section one:\n{html}"
    );
    assert!(
        !html.contains("Second content section."),
        "section-two content must not leak into summary or content{{1}}:\n{html}"
    );
    assert!(
        !html.contains("PREPEND_SHOULD_NOT_RENDER")
            && !html.contains("APPEND_SHOULD_NOT_RENDER"),
        "live Wikidot ignores prependLine and appendLine when separate is true:\n{html}"
    );
}

#[tokio::test]
async fn listpages_unclosed_empty_body_uses_the_live_default_template() {
    const TARGET_SLUG: &str = "fixture-listpages-default-template-target";
    const INDEX_SLUG: &str = "fixture-listpages-default-template-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Default Template Target",
        "Default template summary.\n\nSecond paragraph.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Default Template Index",
        "[[module ListPages category=\"*\" fullname=\"fixture-listpages-default-template-target\"]]",
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    for expected in [
        r#"<div class="list-pages-box">"#,
        r#"<div class="list-pages-item">"#,
        "<h1>",
        r#"href="/fixture-listpages-default-template-target""#,
        "Fixture ListPages Default Template Target",
        "by Administrator",
        "Default template summary.",
        r#"<span class="odate "#,
    ] {
        assert!(
            html.contains(expected),
            "default ListPages output should contain {expected:?}:\n{html}"
        );
    }
    assert!(
        html.contains("Second paragraph."),
        "the default template summary spans the complete first content section:\n{html}"
    );
    assert!(!html.contains("[[module ListPages"), "{html}");
}

#[tokio::test]
async fn listpages_default_summary_unwraps_residual_div_before_recursive_error() {
    const TARGET_SLUG: &str = "fixture-listpages-default-summary-residual-div-target";
    const INDEX_SLUG: &str = "fixture-listpages-default-summary-residual-div-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Default Summary Residual Div Target",
        concat!(
            "[[div class=\"licensebox\"]]\n",
            "Default summary prefix.\n",
            "[[module ListPages category=\"*\"]]%%title%%[[/module]]\n",
            "=====\n",
            "Outside the default summary section.",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Default Summary Residual Div Index",
        concat!(
            "[[module ListPages category=\"*\" ",
            "fullname=\"fixture-listpages-default-summary-residual-div-target\"]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    let expected = concat!(
        "[[div class=&quot;licensebox&quot;]]<br>\n",
        "Default summary prefix.",
        r#"<div class="error-block">The ListPages module does not work recursively.</div>"#,
    );
    assert!(
        html.contains(expected),
        "the runtime-owned default summary boundary must match live Wikidot:\n{html}",
    );
    assert!(
        !html.contains(&format!("<p>{expected}")),
        "the residual generated div prefix must not remain paragraph-wrapped:\n{html}",
    );
    assert!(
        html.contains("</p>\n[[div class=&quot;licensebox&quot;]]"),
        "the residual generated div prefix must retain its root flow break:\n{html}",
    );
}

#[tokio::test]
async fn listpages_code_and_html_bodies_follow_the_live_preparse_failure_shape() {
    const TARGET_SLUG: &str = "fixture-listpages-preparse-target";
    const INDEX_SLUG: &str = "fixture-listpages-preparse-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Preparse Target",
        "Preparse target summary.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Preparse Index",
        concat!(
            "[[div class=\"listpages-preparse-code\"]]\n",
            "[[module ListPages category=\"*\" fullname=\"fixture-listpages-preparse-target\"]]\n",
            "[[code]]\n",
            "%%title%%\n",
            "[[/code]]\n",
            "[[/module]]\n",
            "[[/div]]\n",
            "[[div class=\"listpages-preparse-html\"]]\n",
            "[[module ListPages category=\"*\" fullname=\"fixture-listpages-preparse-target\"]]\n",
            "[[html]]<b>%%title%%</b>[[/html]]\n",
            "[[/module]]\n",
            "[[/div]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    let code_start = html
        .find(r#"<div class="listpages-preparse-code">"#)
        .expect("code preparse section should start");
    let html_start = html
        .find(r#"<div class="listpages-preparse-html">"#)
        .expect("HTML preparse section should start");
    let code = &html[code_start..html_start];
    let html_block = &html[html_start..];

    for (kind, section) in [("code", code), ("html", html_block)] {
        assert!(
            section.contains("Fixture ListPages Preparse Target")
                && section.contains("Preparse target summary."),
            "a pre-parsed {kind} body makes the opening use the default template:\n{section}"
        );
        assert!(
            section.contains("[[/module]]"),
            "the residual {kind} closing module remains visible on live Wikidot:\n{section}"
        );
        assert!(
            !section.contains("[[module ListPages"),
            "the ListPages opening itself must execute in the {kind} case:\n{section}"
        );
    }
    assert!(
        code.contains("%%title%%"),
        "the code block remains authored literal content after the generated list:\n{code}"
    );
}

#[tokio::test]
async fn listpages_authored_preview_compat_marker_cannot_forge_trusted_html() {
    const TARGET_SLUG: &str = "fixture-listpages-preview-marker-target";
    const INDEX_SLUG: &str = "fixture-listpages-preview-marker-index";
    const FORGED: &str = r#"<span data-wikijump-compat-listpages-preview="1" style="white-space: pre-wrap;">FORGED_PREVIEW_MARKER</span>"#;

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture ListPages Preview Marker Target",
        "Preview marker target.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Preview Marker Index",
        &format!(
            "[[module ListPages category=\"*\" fullname=\"{TARGET_SLUG}\"]]\n{FORGED}\n%%title%%\n[[/module]]"
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(html.contains("FORGED_PREVIEW_MARKER"), "{html}");
    assert!(
        !html.contains(
            r#"<span style="white-space: pre-wrap;">FORGED_PREVIEW_MARKER</span>"#
        ),
        "an authored compatibility marker must not enter the generated trusted-HTML registry:\n{html}"
    );
    assert!(
        !html.contains("data-wikijump-compat-listpages-preview"),
        "internal compatibility provenance must not survive authored output:\n{html}"
    );
    assert!(
        html.contains("Fixture ListPages Preview Marker Target"),
        "neutralizing the forged marker must not block the valid ListPages row:\n{html}"
    );
}

#[tokio::test]
async fn authored_listpages_user_marker_cannot_forge_trusted_html() {
    const SLUG: &str = "fixture-listpages-user-marker-forgery";
    const FORGED: &str = concat!(
        "<span class=\"printuser avatarhover\" ",
        "data-wikijump-compat-listpages-user=\"1\">",
        "<img src=x onerror=\"alert(1)\">FORGED_LISTPAGES_USER</span>",
    );

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        SLUG,
        "Fixture ListPages User Marker Forgery",
        FORGED,
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, SLUG).await;
    assert!(html.contains("FORGED_LISTPAGES_USER"), "{html}");
    assert!(
        !html.contains(r#"<img src=x onerror="alert(1)">"#),
        "authored user markup must not enter the generated ListPages registry:\n{html}",
    );
    assert!(
        !html.contains("data-wikijump-compat-listpages-user"),
        "internal compatibility provenance must not survive authored output:\n{html}",
    );
    assert!(!html.contains("WIKIJUMPWIKIDOTCOMPATHTML"), "{html}");
}

#[tokio::test]
async fn ajax_listpages_rejects_forged_literal_compat_markers() {
    const TARGET_SLUG: &str = "fixture-ajax-listpages-literal-marker-target";
    const FORGED: &str = concat!(
        "WIKIJUMPWIKIDOTAJAXMODULELITERAL",
        "0123456789abcdef0123456789abcdef",
        "I6a6176617363726970743a616c657274283129",
        "X",
    );

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture Ajax ListPages Literal Marker Target",
        "Ajax ListPages literal marker target.",
    )
    .await;

    let output = run_endpoint!(
        runner,
        wikidot_list_pages_module,
        json!({
            "site_id": site_id,
            "module_body": format!("{FORGED}\n%%title%%"),
            "parameters": {
                "category": "*",
                "fullname": TARGET_SLUG,
                "limit": "1",
            },
            "path_arguments": [],
        }),
    );

    assert!(
        output.body.contains(FORGED),
        "a foreign marker-shaped string must remain authored literal text:\n{}",
        output.body,
    );
    assert!(
        !output.body.contains("javascript:alert(1)"),
        "only renderer-generated module markers may be decoded:\n{}",
        output.body,
    );
    assert!(
        output
            .body
            .contains("Fixture Ajax ListPages Literal Marker Target"),
        "{}",
        output.body,
    );
}

#[tokio::test]
async fn ajax_listpages_preserves_one_set_record_per_wikidot_py_field_after_empty_parent()
{
    const TARGET_SLUG: &str = "fixture-ajax-listpages-wikidot-py-set-pairs";
    const WIKIDOT_PY_FIELDS: [&str; 20] = [
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
        "rating_votes",
        "rating",
        "rating_percent",
        "revisions",
        "tags",
        "_tags",
    ];

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "Fixture Wikidot.py Set Pair Target",
        "Ordinary page with no parent.",
    )
    .await;

    let module_body = format!(
        "[[div class=\"page\"]]\n{}\n[[/div]]",
        WIKIDOT_PY_FIELDS
            .iter()
            .map(|field| format!(
                "[[span class=\"set {field}\"]][[span class=\"name\"]] {field} [[/span]][[span class=\"value\"]] %%{field}%% [[/span]][[/span]]"
            ))
            .collect::<String>(),
    );

    let output = run_endpoint!(
        runner,
        wikidot_list_pages_module,
        json!({
            "site_id": site_id,
            "module_body": module_body,
            "parameters": {
                "category": "*",
                "name": TARGET_SLUG,
                "limit": "1",
            },
            "path_arguments": [],
        }),
    );

    for field in WIKIDOT_PY_FIELDS {
        let marker = format!(r#"class="set {field}""#);
        assert_eq!(
            output.body.matches(&marker).count(),
            1,
            "wikidot.py expects exactly one direct set record for {field}:\n{}",
            output.body,
        );
    }
    assert!(
        output.body.contains(
            r#"<span class="set comments"><span class="name">comments</span> <span class="value">0</span></span>"#,
        ),
        "the field after empty parent_fullname must retain its name/value pair:\n{}",
        output.body,
    );
}

#[tokio::test]
async fn listpages_default_category_and_bare_tags_follow_wikidot_semantics() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let category_slug = "fixture-listpages-default-category";
    let tag_a = "fixture-listpages-bare-a";
    let tag_b = "fixture-listpages-bare-b";
    let target_a_slug = format!("{category_slug}:target-a");
    let target_b_slug = format!("{category_slug}:target-b");

    for (slug, title, tags) in [
        (
            target_a_slug.clone(),
            "Fixture ListPages Bare Tag Target A",
            vec![tag_a],
        ),
        (
            target_b_slug.clone(),
            "Fixture ListPages Bare Tag Target B",
            vec![tag_b],
        ),
        (
            "fixture-listpages-default-category-excluded".to_owned(),
            "Fixture ListPages Bare Tag Excluded",
            vec![tag_a],
        ),
    ] {
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            &slug,
            title,
            "Fixture ListPages bare tag target.",
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, &slug, revision, &tags).await;
    }

    let index_slug = format!("{category_slug}:index");
    create_listpages_test_page(
        &mut runner,
        site_id,
        &index_slug,
        "Fixture ListPages Default Category Index",
        &format!(
            "Default category ListPages start.\n\n[[module ListPages tags=\"{tag_a} {tag_b}\" limit=\"10\" order=\"name\"]]\n* %%title%% :: %%slug%%\n[[/module]]\n\nDefault category ListPages end."
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
    .expect("ListPages default-category index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    for expected in [
        "Default category ListPages start.",
        "Fixture ListPages Bare Tag Target A",
        "Fixture ListPages Bare Tag Target B",
        "Fixture ListPages Bare Tag Target A :: target-a",
        "Fixture ListPages Bare Tag Target B :: target-b",
        "Default category ListPages end.",
    ] {
        assert!(
            html.contains(expected),
            "ListPages should include current-category pages matching either bare tag {expected:?}:\n{html}"
        );
    }

    for forbidden in [
        "Fixture ListPages Bare Tag Excluded",
        "fixture-listpages-default-category-excluded",
        "[[module ListPages",
        "%%title%%",
        "%%slug%%",
    ] {
        assert!(
            !html.contains(forbidden),
            "ListPages should default to the current category and render body variables, but found {forbidden:?}:\n{html}"
        );
    }
}

#[tokio::test]
async fn first_revision_current_page_listpages_uses_render_page_info() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(
            "fixture-first-revision-current-page-listpages",
        )),
    );
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
async fn current_page_listpages_created_by_uses_creation_revision() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let category_slug = "fixture-listpages-created-by-revision";
    let page_slug = format!("{category_slug}:target");

    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        category_slug,
        SAMPLE_USER_ID,
        &[Action::View, Action::Create, Action::Edit],
        "sample-mutator",
    )
    .await;
    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        category_slug,
        ADMIN_USER_ID,
        &[Action::View, Action::Create, Action::Edit],
        "admin-mutator",
    )
    .await;
    set_test_user_name(&runner, SAMPLE_USER_ID, "Sample Creator").await;
    set_test_user_name(&runner, ADMIN_USER_ID, "Admin Editor").await;

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Slug(Cow::Owned(page_slug.clone())),
    );
    let created = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": concat!(
                "[[module ListPages range=\".\"]]\n",
                "CREATED_BY=%%created_by%%\n",
                "UPDATED_BY=%%updated_by%%\n",
                "[[/module]]\n",
                "[[module ListPages range=\".\" created_by=\"-=\"]]\n",
                "EXCLUDED_RANGE=%%fullname%%\n",
                "[[/module]]\n",
                "[[module ListPages name=\"=\" created_by=\"-=\"]]\n",
                "EXCLUDED_NAME=%%fullname%%\n",
                "[[/module]]\n",
                "[[module ListPages range=\".\" created_by=\"=\"]]\n",
                "INCLUDED_CURRENT=%%fullname%%\n",
                "[[/module]]"
            ),
            "title": "Fixture ListPages Created By Revision",
            "alt_title": null,
            "slug": page_slug,
            "layout": "wikidot",
            "revision_comments": "create ListPages author fixture",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(created.page_id),
    );
    let edited = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": created.page_id,
            "last_revision_id": created.revision_id,
            "revision_comments": "edit ListPages author fixture",
            "user_id": ADMIN_USER_ID,
            "wikitext": concat!(
                "[[module ListPages range=\".\"]]\n",
                "CREATED_BY=%%created_by%%\n",
                "UPDATED_BY=%%updated_by%%\n",
                "[[/module]]\n",
                "[[module ListPages range=\".\" created_by=\"-=\"]]\n",
                "EXCLUDED_RANGE=%%fullname%%\n",
                "[[/module]]\n",
                "[[module ListPages name=\"=\" created_by=\"-=\"]]\n",
                "EXCLUDED_NAME=%%fullname%%\n",
                "[[/module]]\n",
                "[[module ListPages range=\".\" created_by=\"=\"]]\n",
                "INCLUDED_CURRENT=%%fullname%%\n",
                "[[/module]]\n",
                "after edit"
            ),
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("admin edit should create a revision");
    assert_eq!(edited.revision_number, 1);

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": created.page_id,
        }),
    )
    .expect("ListPages author fixture should exist before rerender");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": page.page_category_id,
            "page_id": created.page_id,
        }),
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": created.page_id,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("ListPages author fixture should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("CREATED_BY=Sample Creator"),
        "range=. ListPages should keep the creation author after later edits:\n{html}"
    );
    assert!(
        html.contains("UPDATED_BY=Admin Editor"),
        "range=. ListPages should still use the latest revision for updated_by:\n{html}"
    );
    assert!(
        !html.contains("CREATED_BY=Admin Editor"),
        "range=. ListPages must not use the latest editor as created_by:\n{html}"
    );
    assert!(
        html.contains(&format!("INCLUDED_CURRENT={page_slug}")),
        "created_by=\"=\" should retain the selected current page:\n{html}"
    );
    for forbidden in ["EXCLUDED_RANGE=", "EXCLUDED_NAME="] {
        assert!(
            !html.contains(forbidden),
            "created_by=\"-=\" must exclude the current page for every current-page selector, but found {forbidden:?}:\n{html}"
        );
    }
}

#[tokio::test]
async fn imported_listpages_authors_use_snapshot_names_and_rerender_stably() {
    const IMPORT_RUN_ID: i64 = 7_130_102;
    const TAG: &str = "verification-imported-listpages-author";
    const ALICE_SLUG: &str = "fixture-imported-listpages-author-alice";
    const BOB_SLUG: &str = "fixture-imported-listpages-author-bob";
    const INDEX_SLUG: &str = "fixture-imported-listpages-author-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (slug, title) in [
        (ALICE_SLUG, "Fixture Imported Author Alice"),
        (BOB_SLUG, "Fixture Imported Author Bob"),
    ] {
        let revision =
            create_listpages_test_page(&mut runner, site_id, slug, title, title).await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision, &[TAG]).await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture Imported ListPages Author Index",
        &format!(
            concat!(
                "[[module ListPages created_by=\"ALICE_EXAMPLE\" tags=\"+{tag}\" limit=\"20\"]]\n",
                "LITERAL=%%fullname%%\n",
                "[[/module]]\n",
                "[[module ListPages created_by=\"=\" tags=\"+{tag}\" limit=\"20\"]]\n",
                "CURRENT=%%fullname%%\n",
                "[[/module]]\n",
                "[[module ListPages created_by=\"No Such Wikidot Author\" tags=\"+{tag}\" limit=\"20\"]]\n",
                "UNKNOWN=%%fullname%%\n",
                "[[/module]]"
            ),
            tag = TAG,
        ),
    )
    .await;

    let alice_page_id = listpages_test_page_id(&runner, site_id, ALICE_SLUG).await;
    let bob_page_id = listpages_test_page_id(&runner, site_id, BOB_SLUG).await;
    let index_page_id = listpages_test_page_id(&runner, site_id, INDEX_SLUG).await;
    create_listpages_test_import_run(&runner, site_id, IMPORT_RUN_ID, 3).await;
    for fixture in [
        (alice_page_id, ALICE_SLUG, 11, "Alice Example"),
        (bob_page_id, BOB_SLUG, 12, "Bob Example"),
        (index_page_id, INDEX_SLUG, 13, "Alice Example"),
    ] {
        mark_imported_page_with_author_snapshot(&runner, site_id, IMPORT_RUN_ID, fixture)
            .await;
    }

    let alice_names = [Cow::Borrowed("ALICE_EXAMPLE")];
    let missing_names = [Cow::Borrowed("No Such Wikidot Author")];
    let admin_ids = [ADMIN_USER_ID];
    for (selector, expected) in [
        (
            AuthorSelector::Any {
                user_ids: &[],
                wikidot_snapshot_names: &alice_names,
            },
            &[ALICE_SLUG][..],
        ),
        (
            AuthorSelector::Any {
                user_ids: &[],
                wikidot_snapshot_names: &missing_names,
            },
            &[][..],
        ),
        (
            AuthorSelector::Any {
                user_ids: &[],
                wikidot_snapshot_names: &[],
            },
            &[][..],
        ),
        (AuthorSelector::None, &[][..]),
        (
            AuthorSelector::Any {
                user_ids: &admin_ids,
                wikidot_snapshot_names: &alice_names,
            },
            &[ALICE_SLUG, BOB_SLUG][..],
        ),
        (AuthorSelector::All, &[ALICE_SLUG, BOB_SLUG][..]),
    ] {
        assert_eq!(
            query_listpages_test_author_slugs(&runner, site_id, TAG, selector).await,
            expected,
            "author selector {selector:?} should remain explicit and use ID/name OR semantics"
        );
    }

    let index_page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
        }),
    )
    .expect("imported author ListPages index should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": index_page.page_category_id,
            "page_id": index_page_id,
        }),
    );

    let first_html =
        load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;

    for expected in [
        format!("LITERAL={ALICE_SLUG}"),
        format!("CURRENT={ALICE_SLUG}"),
    ] {
        assert!(
            first_html.contains(&expected),
            "snapshot author ListPages should contain {expected:?}:\n{first_html}"
        );
    }
    for forbidden in [
        format!("LITERAL={BOB_SLUG}"),
        format!("CURRENT={BOB_SLUG}"),
        format!("UNKNOWN={ALICE_SLUG}"),
        format!("UNKNOWN={BOB_SLUG}"),
    ] {
        assert!(
            !first_html.contains(&forbidden),
            "snapshot author ListPages should not contain {forbidden:?}:\n{first_html}"
        );
    }

    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": index_page.page_category_id,
            "page_id": index_page_id,
        }),
    );
    let second_html =
        load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert_eq!(
        first_html, second_html,
        "repeated snapshot-author rerenders should be byte-stable"
    );
}

#[tokio::test]
async fn included_author_tool_coauthored_branch_renders_named_page_box() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let target_revision = create_listpages_test_page(
        &mut runner,
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
        &mut runner,
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
        &mut runner,
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

#[tokio::test]
async fn listpages_fragment_content_skips_hidden_pages_by_default() {
    const INDEX_SLUG: &str = "fixture-listpages-fragment-default-index";
    const HIDDEN_SLUG: &str = "_fixture-listpages-fragment-hidden";
    const VISIBLE_SLUG: &str = "fixture-listpages-fragment-visible";
    const HIDDEN_MARKER: &str = "Hidden fragment content must not render.";
    const VISIBLE_MARKER: &str = "Visible fragment content should render.";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let index_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Fragment Default Index",
        concat!(
            "Before fragment ListPages.\n\n",
            "[[module ListPages parent=\".\" category=\"fragment\" order=\"created_at\" limit=\"1\" offset=\"0\"]]\n",
            "%%content%%\n",
            "[[/module]]\n\n",
            "After fragment ListPages."
        ),
    )
    .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fragment:fixture-listpages-fragment-category-primer",
        "Fixture Fragment Category Primer",
        "Fixture fragment category primer.",
    )
    .await;

    let hidden_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        HIDDEN_SLUG,
        "Fixture Hidden Fragment",
        HIDDEN_MARKER,
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, HIDDEN_SLUG, "fragment").await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        HIDDEN_SLUG,
        hidden_revision,
        &["verification", "verification-fragment-default"],
    )
    .await;
    set_listpages_test_created_at(
        &runner,
        site_id,
        HIDDEN_SLUG,
        OffsetDateTime::UNIX_EPOCH + Duration::seconds(1),
    )
    .await;
    set_listpages_test_parent(&mut runner, site_id, HIDDEN_SLUG, INDEX_SLUG).await;

    let visible_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        VISIBLE_SLUG,
        "Fixture Visible Fragment",
        VISIBLE_MARKER,
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, VISIBLE_SLUG, "fragment").await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        VISIBLE_SLUG,
        visible_revision,
        &["verification", "verification-fragment-default"],
    )
    .await;
    set_listpages_test_created_at(
        &runner,
        site_id,
        VISIBLE_SLUG,
        OffsetDateTime::UNIX_EPOCH + Duration::seconds(2),
    )
    .await;
    set_listpages_test_parent(&mut runner, site_id, VISIBLE_SLUG, INDEX_SLUG).await;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(INDEX_SLUG))),
    });
    let rerender = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
            "last_revision_id": index_revision,
            "revision_comments": "rerender after attaching ListPages fragments",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(
        rerender.is_none(),
        "relationship-only rerender should not create a page revision",
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("fragment ListPages index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains(VISIBLE_MARKER),
        "fragment ListPages should render the first normal fragment:\n{html}"
    );
    for forbidden in [HIDDEN_MARKER, "%%content%%", "[[module ListPages"] {
        assert!(
            !html.contains(forbidden),
            "fragment ListPages should not contain {forbidden:?}:\n{html}"
        );
    }
}

#[tokio::test]
async fn listpages_fragment_content_executes_child_includes() {
    const INDEX_SLUG: &str = "fixture-listpages-fragment-include-index";
    const FRAGMENT_SLUG: &str = "fixture-listpages-fragment-include-child";
    const INCLUDE_SLUG: &str = "fixture-listpages-fragment-include-target";
    const INCLUDE_MARKER: &str = "Included fragment dependency should render.";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let index_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Fragment Include Index",
        concat!(
            "Before included fragment.\n\n",
            "[[module ListPages parent=\".\" category=\"fragment\" order=\"created_at\" limit=\"1\"]]\n",
            "%%content%%\n",
            "[[/module]]\n\n",
            "After included fragment."
        ),
    )
    .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fragment:fixture-listpages-fragment-include-primer",
        "Fixture Fragment Include Category Primer",
        "Fixture fragment include category primer.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INCLUDE_SLUG,
        "Fixture ListPages Fragment Include Target",
        INCLUDE_MARKER,
    )
    .await;

    let fragment_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        FRAGMENT_SLUG,
        "Fixture ListPages Fragment Include Child",
        &format!(
            "Fragment before include.\n[[include {INCLUDE_SLUG}]]\nFragment after include."
        ),
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, FRAGMENT_SLUG, "fragment").await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        FRAGMENT_SLUG,
        fragment_revision,
        &["verification", "verification-fragment-include"],
    )
    .await;
    set_listpages_test_created_at(
        &runner,
        site_id,
        FRAGMENT_SLUG,
        OffsetDateTime::UNIX_EPOCH + Duration::seconds(1),
    )
    .await;
    set_listpages_test_parent(&mut runner, site_id, FRAGMENT_SLUG, INDEX_SLUG).await;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(INDEX_SLUG))),
    });
    let rerender = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
            "last_revision_id": index_revision,
            "revision_comments": "rerender after attaching include fragment",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(
        rerender.is_none(),
        "relationship-only rerender should not create a page revision",
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("fragment include ListPages index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    for expected in ["Fragment before include.", "Fragment after include."] {
        assert!(
            html.contains(expected),
            "fragment ListPages content should contain {expected:?}:\n{html}"
        );
    }
    assert!(
        html.contains(INCLUDE_MARKER)
            && !html.contains(&format!("[[include {INCLUDE_SLUG}]]")),
        "plain fragment content must execute child includes:\n{html}"
    );
}

#[tokio::test]
async fn listpages_content_executes_attachment_syntax_with_selected_page_owner() {
    const INDEX_SLUG: &str = "fixture-listpages-attachment-owner-index";
    const FRAGMENT_SLUG: &str = "fragment:fixture-listpages-attachment-owner-row";
    const FILE_NAME: &str = "2117.png";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let index_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Attachment Owner Index",
        concat!(
            "[[module ListPages category=\"fragment\" parent=\".\" limit=\"1\" order=\"created_at\" offset=\"@URL|0\"]]",
            "%%content%%",
            "[[/module]]",
        ),
    )
    .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        FRAGMENT_SLUG,
        "Fixture ListPages Attachment Owner Row",
        concat!(
            "[[include component:image-block ",
            "name=2117.png|alt=alt|alt-text=An image|",
            "link=\"https://scp-wiki.wdfiles.com/local--files/",
            "fragment:fixture-listpages-attachment-owner-row/2117.png\"]]\n",
            "[[image direct-row.png link=direct-row-full.png]]",
        ),
    )
    .await;
    set_listpages_test_parent(&mut runner, site_id, FRAGMENT_SLUG, INDEX_SLUG).await;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(INDEX_SLUG))),
    });
    let rerender = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
            "last_revision_id": index_revision,
            "revision_comments": "rerender after attaching ListPages provenance row",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(
        rerender.is_none(),
        "relationship-only rerender should not create a page revision",
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("ListPages attachment-owner index should exist");
    let html = page
        .compiled_body_html
        .expect("ListPages attachment-owner index should have compiled HTML");
    assert_eq!(
        html.matches(&format!("/local--files/{FRAGMENT_SLUG}/{FILE_NAME}"))
            .count(),
        2,
        "the component image link and image must retain the selected page owner: {html}",
    );
    assert!(
        !html.contains("[[include component:image-block ")
            && !html.contains("[[image direct-row.png link=direct-row-full.png]]")
            // A bare .png attachment shows the original on Wikidot (#2349).
            && html.contains(&format!(
                r#"<img src="https://scp-wiki.wjfiles.com/local--files/{FRAGMENT_SLUG}/direct-row.png""#
            ))
            && !html.contains("/local--resized-images/"),
        "plain selected content must render attachment syntax with row ownership: {html}",
    );
    for forbidden_owner in [
        INDEX_SLUG,
        "component:image-block",
        "component:image-block-base",
    ] {
        assert!(
            !html.contains(&format!("/local--files/{forbidden_owner}/{FILE_NAME}")),
            "ListPages consumer and component pages must not steal row attachment ownership: {html}",
        );
    }
    assert!(
        !html.contains("%22https%3A")
            && !html.contains("2117.png%22")
            && !html.contains("%222117.png"),
        "quoted include values must not become percent-encoded attachment data: {html}",
    );
}

#[tokio::test]
async fn listpages_content_keeps_same_named_attachment_owners_per_row() {
    const INDEX_SLUG: &str = "fixture-listpages-two-row-attachment-owner-index";
    const FIRST_FRAGMENT: &str =
        "fragment:fixture-listpages-two-row-attachment-owner-first";
    const SECOND_FRAGMENT: &str =
        "fragment:fixture-listpages-two-row-attachment-owner-second";
    const FILE_NAME: &str = "shared-row.png";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let index_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture Two-row ListPages Attachment Owner Index",
        concat!(
            "[[module ListPages category=\"fragment\" parent=\".\" limit=\"2\" order=\"created_at\" offset=\"0\"]]",
            "%%content%%",
            "[[/module]]",
        ),
    )
    .await;

    for (index, fragment) in [FIRST_FRAGMENT, SECOND_FRAGMENT].into_iter().enumerate() {
        create_listpages_test_page(
            &mut runner,
            site_id,
            fragment,
            "Fixture Two-row ListPages Attachment Owner Row",
            "[[include component:image-block name=shared-row.png|link=shared-row.png]]",
        )
        .await;
        set_listpages_test_created_at(
            &runner,
            site_id,
            fragment,
            OffsetDateTime::UNIX_EPOCH + Duration::seconds(index as i64 + 1),
        )
        .await;
        set_listpages_test_parent(&mut runner, site_id, fragment, INDEX_SLUG).await;
    }

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(INDEX_SLUG))),
    });
    let rerender = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
            "last_revision_id": index_revision,
            "revision_comments": "rerender after attaching two ListPages provenance rows",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(
        rerender.is_none(),
        "relationship-only rerender should not create a page revision",
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("two-row ListPages attachment-owner index should exist");
    let html = page
        .compiled_body_html
        .expect("two-row ListPages attachment-owner index should have compiled HTML");

    assert_eq!(
        html.matches(
            "[[include component:image-block name=shared-row.png|link=shared-row.png]]",
        )
        .count(),
        0,
        "plain selected rows must execute their attachment directives: {html}",
    );
    for fragment in [FIRST_FRAGMENT, SECOND_FRAGMENT] {
        assert_eq!(
            html.matches(&format!("/local--files/{fragment}/{FILE_NAME}"))
                .count(),
            1,
            "each rendered attachment must retain its selected row owner: {html}",
        );
    }
    for forbidden_owner in [
        INDEX_SLUG,
        "component:image-block",
        "component:image-block-base",
    ] {
        assert!(
            !html.contains(&format!("/local--files/{forbidden_owner}/{FILE_NAME}")),
            "neither the ListPages consumer nor a component page may steal a row attachment: {html}",
        );
    }
}

#[tokio::test]
async fn exact_name_listpages_batch_preserves_order_and_permissions() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    set_test_user_name(&runner, ADMIN_USER_ID, "Exact Batch Author").await;

    for (slug, title) in [
        ("fixture-exact-batch-a", "Exact Batch A"),
        ("fixture-exact-batch-b", "Exact Batch B"),
        ("fixture-exact-batch-c", "Exact Batch C"),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, "target").await;
    }

    let private_category = "fixture-exact-batch-private-category";
    make_listpages_test_category_admin_only(&runner, site_id, private_category).await;
    let private_slug = "fixture-exact-batch-private";
    create_listpages_test_page(
        &mut runner,
        site_id,
        private_slug,
        "Exact Batch Private",
        "private target",
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, private_slug, private_category)
        .await;

    let index_slug = "fixture-exact-batch-index";
    create_listpages_test_page(
        &mut runner,
        site_id,
        index_slug,
        "Exact Batch Index",
        &format!(
            concat!(
                "[[module ListPages fullname=\"fixture-exact-batch-c\"]]C=%%slug%%|%%created_by%%[[/module]]\n",
                "[[module ListPages fullname=\"fixture-exact-batch-a\"]]A1=%%slug%%@%%created_at|%Y %b %d %H:%M%%[[/module]]\n",
                "[[module ListPages fullname=\"fixture-exact-batch-b\"]]B=%%slug%%[[/module]]\n",
                "[[module ListPages name=\"fixture-exact-batch-a\"]]A2=%%slug%%|%%rating_votes%%[[/module]]\n",
                "[[module ListPages fullname=\"fixture-exact-batch-missing\"]]MISSING=%%slug%%[[/module]]\n",
                "[[module ListPages category=\"{}\" fullname=\"{}\"]]PRIVATE=%%slug%%[[/module]]",
            ),
            private_category, private_slug,
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
    .expect("exact-name batch index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    let c = html.find("C=fixture-exact-batch-c").unwrap();
    let a1 = html.find("A1=fixture-exact-batch-a").unwrap();
    let b = html.find("B=fixture-exact-batch-b").unwrap();
    let a2 = html.find("A2=fixture-exact-batch-a").unwrap();
    assert!(
        c < a1 && a1 < b && b < a2,
        "batch output order changed:\n{html}"
    );
    assert_eq!(
        html.matches("A1=fixture-exact-batch-a@").count(),
        1,
        "an exact-name ListPages lookup should render the unique live page once:\n{html}",
    );
    assert!(
        html.contains("C=fixture-exact-batch-c|Exact Batch Author"),
        "batched user display metadata was not substituted:\n{html}"
    );
    assert!(
        html.contains("A2=fixture-exact-batch-a|0"),
        "batched absent snapshot metadata did not use the zero-vote state:\n{html}"
    );
    assert!(
        !html.contains("MISSING="),
        "missing exact-name page rendered a row:\n{html}"
    );
    assert!(
        !html.contains("PRIVATE="),
        "private-category exact-name page was exposed:\n{html}"
    );
}

#[tokio::test]
async fn fallback_link_title_batch_preserves_singular_permission() {
    const TARGET_SLUG: &str = "fixture-fallback-title-duplicate";
    const FIRST_TITLE: &str = "Fallback duplicate first title";
    const FIRST_CATEGORY: &str = "fixture-fallback-title-first";
    const INDEX_SLUG: &str = "fixture-fallback-title-index";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        FIRST_TITLE,
        "first duplicate target",
    )
    .await;
    let first_page = PageTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page::Column::SiteId.eq(site_id))
                .add(page::Column::Slug.eq(TARGET_SLUG)),
        )
        .one(runner.context().transaction())
        .await
        .expect("first fallback target lookup should not fail")
        .expect("first fallback target should exist");
    let first_page_id = first_page.page_id;
    let first_category =
        CategoryService::get_or_create(runner.context(), site_id, FIRST_CATEGORY)
            .await
            .expect("first fallback category should be created");
    let mut first_page = first_page.into_active_model();
    first_page.page_category_id = Set(first_category.category_id);
    first_page
        .update(runner.context().transaction())
        .await
        .expect("first fallback target should move to its category");
    let selected = PageService::get_optional(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed(TARGET_SLUG)),
    )
    .await
    .expect("singular lookup should not fail")
    .expect("singular lookup should select a page");
    let selected_page_id = selected.page_id;
    assert_eq!(selected_page_id, first_page_id);
    let selected_category_id = first_category.category_id;
    make_listpages_test_category_admin_only(&runner, site_id, FIRST_CATEGORY).await;

    let selected_again = PageService::get_optional(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed(TARGET_SLUG)),
    )
    .await
    .expect("repeated singular lookup should not fail")
    .expect("repeated singular lookup should select a page");
    assert_eq!(selected_again.page_id, selected_page_id);
    let can_view_selected = PermissionService::check_user_can(
        runner.context(),
        &CheckPermissionContext {
            user_id: None,
            site_id,
            page_reference: Some(Reference::Id(selected_page_id)),
        },
        Permission {
            resource_type: Resource::Page,
            resource_category: Some(Reference::Id(selected_category_id)),
            action: Action::View,
        },
    )
    .await
    .expect("anonymous permission check should not fail");
    assert!(!can_view_selected);

    let mut source = format!("[[[{TARGET_SLUG}|]]]\n");
    for index in 0..64 {
        source.push_str(&format!(
            "[[collapsible show=\"+ {index}\" hide=\"- {index}\"]]\nbody\n[[/collapsible]]\n"
        ));
    }
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fallback duplicate title index",
        &source,
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("fallback duplicate title index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled fallback body should be included in page_get details");

    assert!(
        html.contains(&format!(
            r#"<a class="newpage" href="/{TARGET_SLUG}">{TARGET_SLUG}</a>"#
        )),
        "fallback title batch should not expose a denied page title:\n{html}",
    );
    assert!(!html.contains(FIRST_TITLE), "{html}");
}

#[tokio::test]
async fn listpages_content_body_supports_bounded_ordered_child_results() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    const INDEX_SLUG: &str = "fixture-listpages-content-body-index";

    let index_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Fixture ListPages Content Body Index",
        concat!(
            "Before content ListPages.\n\n",
            "[[module ListPages parent=\".\" order=\"created_at desc\" limit=\"2\" offset=\"0\" pagetype=\"normal\"]]\n",
            "content-body-start %%content%% content-body-end\n",
            "[[/module]]\n\n",
            "After content ListPages."
        ),
    )
    .await;

    for (index, slug, title, source) in [
        (
            0,
            "fixture-listpages-content-body-target-a",
            "Fixture ListPages Target Alpha",
            "Fixture ListPages Target Alpha marker.",
        ),
        (
            1,
            "fixture-listpages-content-body-target-b",
            "Fixture ListPages Target Beta",
            "Fixture ListPages Target Beta marker.",
        ),
        (
            2,
            "fixture-listpages-content-body-target-c",
            "Fixture ListPages Target Gamma",
            "Fixture ListPages Target Gamma marker.",
        ),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, source).await;
        set_listpages_test_created_at(
            &runner,
            site_id,
            slug,
            OffsetDateTime::UNIX_EPOCH + Duration::seconds(index + 1),
        )
        .await;
        set_listpages_test_parent(&mut runner, site_id, slug, INDEX_SLUG).await;
    }

    let excluded_slug = "fixture-listpages-content-body-excluded";
    create_listpages_test_page(
        &mut runner,
        site_id,
        excluded_slug,
        "Fixture ListPages Excluded",
        "Fixture ListPages Excluded marker.",
    )
    .await;

    let private_category = "fixture-listpages-private-view";
    make_listpages_test_category_admin_only(&runner, site_id, private_category).await;
    let private_slug = "fixture-listpages-content-body-private";
    create_listpages_test_page(
        &mut runner,
        site_id,
        private_slug,
        "Fixture ListPages Private",
        "Fixture ListPages Private marker.",
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, private_slug, private_category)
        .await;
    set_listpages_test_created_at(
        &runner,
        site_id,
        private_slug,
        OffsetDateTime::UNIX_EPOCH + Duration::seconds(4),
    )
    .await;
    set_listpages_test_parent(&mut runner, site_id, private_slug, INDEX_SLUG).await;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(INDEX_SLUG))),
    });
    let rerender = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
            "last_revision_id": index_revision,
            "revision_comments": "rerender after attaching content ListPages children",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(
        rerender.is_none(),
        "relationship-only rerender should not create a page revision",
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("content ListPages index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    let target_c = html.find("Fixture ListPages Target Gamma marker.");
    let target_b = html.find("Fixture ListPages Target Beta marker.");
    assert!(
        target_c.is_some() && target_b.is_some(),
        "created_at desc content ListPages should render target C and B content:\n{html}",
    );
    let target_c = target_c.expect("checked target C exists");
    let target_b = target_b.expect("checked target B exists");
    assert!(
        target_c < target_b,
        "created_at desc content ListPages should render target C before target B:\n{html}",
    );

    for expected in [
        "content-body-start",
        "content-body-end",
        "Fixture ListPages Target Gamma marker.",
        "Fixture ListPages Target Beta marker.",
    ] {
        assert!(
            html.contains(expected),
            "content ListPages fixture should contain {expected:?}:\n{html}"
        );
    }

    for forbidden in [
        "Fixture ListPages Target Alpha marker.",
        "Fixture ListPages Excluded marker.",
        "Fixture ListPages Private marker.",
        "%%content%%",
        "[[module ListPages",
    ] {
        assert!(
            !html.contains(forbidden),
            "content ListPages fixture should not contain {forbidden:?}:\n{html}"
        );
    }
}
