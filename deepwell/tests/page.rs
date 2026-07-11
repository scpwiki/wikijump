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
use deepwell::constants::{
    ADMIN_USER_ID, ANONYMOUS_USER_ID, SAMPLE_USER_ID, SYSTEM_USER_ID, UNKNOWN_USER_ID,
};
use deepwell::error::prelude::*;
use deepwell::models::file;
use deepwell::models::page::{self, Entity as PageTable};
use deepwell::models::page_category::{self, Entity as PageCategoryTable};
use deepwell::models::page_revision::Entity as PageRevisionTable;
use deepwell::models::role_permission::{self, Entity as RolePermissionTable};
use deepwell::models::text_block;
use deepwell::models::user::Entity as UserTable;
use deepwell::services::blob::{EMPTY_BLOB_HASH, EMPTY_BLOB_MIME};
use deepwell::services::category::CategoryService;
use deepwell::services::file_revision::CreateFirstFileRevision;
use deepwell::services::forum::{CreateForumCategory, CreateForumGroup};
use deepwell::services::forum_post::CreateForumPost;
use deepwell::services::forum_thread::CreateForumThread;
use deepwell::services::page_query::{
    CategoriesSelector, DateSelector, FoundPageFields, IncludedCategories,
    OrderBySelector, OrderProperty, PageParentSelector, PageQuery, PageQueryService,
    PageTypeSelector, PaginationSelector, RangeSelector, TagCondition,
};
use deepwell::services::permission::{PermissionCache, PermissionService};
use deepwell::services::role::{
    GrantUserRoleInput, InternalCreateRoleInput, RoleService, UpdateRolePermissionsInput,
};
use deepwell::services::session::CreateSession;
use deepwell::services::view::{GetArticleViewOutput, GetPageViewOutput};
use deepwell::services::{
    FileRevisionService, ForumPostService, ForumService, ForumThreadService,
    RenderService, RequestContext, SessionService,
};
use deepwell::types::{
    Action, PageId, PageRevisionType, Permission, Reference, Resource, TextBlockType,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set,
};
use serde_json::json;
use std::borrow::Cow;
use std::collections::BTreeSet;
use time::{Duration, OffsetDateTime};

use ftml::data::{PageInfo, ScoreValue};
use ftml::layout::Layout;
use ftml::settings::{WikitextMode, WikitextSettings};

fn set_mutation_request_context(
    runner: &mut TestRunner,
    user_id: i64,
    site_id: i64,
    page_reference: Reference<'static>,
) {
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(user_id),
        site_id: Some(site_id),
        page_reference: Some(page_reference),
    });
}

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
async fn rerender_uses_latest_navigation_page_revision() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;

    let nav_top = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "nav:top",
        }),
    )
    .expect("seeded nav:top should exist");
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(nav_top.page_id),
    );
    run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": nav_top.page_id,
            "last_revision_id": nav_top.revision_id,
            "revision_comments": "replace navigation fixture",
            "user_id": ADMIN_USER_ID,
            "wikitext": "* latest navigation marker",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("editing nav:top should create a revision");

    let home = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "home",
        }),
    )
    .expect("seeded home page should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": home.page_category_id,
            "page_id": home.page_id,
        }),
    );

    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {
                "slug": "home",
                "extra": "",
            },
            "locales": ["en-US", "en"],
        }),
    );
    let top_bar = match view {
        GetPageViewOutput::Found {
            compiled_top_bar_html,
            ..
        } => compiled_top_bar_html.expect("top bar should be compiled"),
        other => panic!("expected found page view, got {other:?}"),
    };
    assert!(
        top_bar.contains("latest navigation marker"),
        "rerender should use the latest nav:top revision:\n{top_bar}"
    );
    assert!(
        !top_bar.contains("Wikijump Blog"),
        "rerender reused stale nav:top wikitext:\n{top_bar}"
    );
}

#[tokio::test]
async fn article_view_cache_respects_anonymous_permission_revocation() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;
    let category_slug = "article-cache-permission-revocation";
    let slug = "article-cache-permission-revocation:source";
    let category_id =
        CategoryService::get_or_create(runner.context(), site_id, category_slug)
            .await
            .expect("cache permission category should be created")
            .category_id;
    let root_role = RoleService::get(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed("root")),
    )
    .await
    .expect("root role should exist");
    let guest_role = RoleService::get(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed("guest")),
    )
    .await
    .expect("guest role should exist");
    for role_id in [root_role.role_id, guest_role.role_id] {
        role_permission::ActiveModel {
            role_id: Set(role_id),
            site_id: Set(site_id),
            resource_type: Set(Resource::Page),
            resource_category_id: Set(Some(category_id)),
            action: Set(Action::View),
            ..Default::default()
        }
        .insert(runner.context().transaction())
        .await
        .expect("scoped cache test permission should be inserted");
    }

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(slug)),
    );
    let created = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "cached anonymous article body",
            "title": "Article cache permission revocation",
            "alt_title": null,
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "create cache permission page",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let page = PageTable::find_by_id(created.page_id)
        .one(runner.context().transaction())
        .await
        .expect("page lookup should not fail")
        .expect("created page should exist");
    let mut page = page.into_active_model();
    page.from_wikidot = Set(true);
    page.update(runner.context().transaction())
        .await
        .expect("page should be marked imported");

    let first = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {
                "slug": slug,
                "extra": "",
            },
            "locales": ["en-US", "en"],
        }),
    );
    let GetArticleViewOutput {
        page: GetPageViewOutput::Found { .. },
        article_page_cache_key: Some(first_cache_key),
        ..
    } = first
    else {
        panic!("first article view should populate the anonymous cache");
    };
    assert!(
        first_cache_key.contains(":permission=site=")
            && first_cache_key.contains(",user="),
        "article cache key must include the anonymous permission fence: {first_cache_key}"
    );

    RolePermissionTable::delete_many()
        .filter(role_permission::Column::RoleId.eq(guest_role.role_id))
        .filter(role_permission::Column::SiteId.eq(site_id))
        .filter(role_permission::Column::ResourceType.eq(Resource::Page))
        .filter(role_permission::Column::ResourceCategoryId.eq(category_id))
        .filter(role_permission::Column::Action.eq(Action::View))
        .exec(runner.context().transaction())
        .await
        .expect("guest scoped view permission should be revoked");
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("permission cache invalidation should run");

    let second = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {
                "slug": slug,
                "extra": "",
            },
            "locales": ["en-US", "en"],
        }),
    );
    let GetArticleViewOutput {
        page: GetPageViewOutput::Permissions { banned: false, .. },
        article_page_cache_key: Some(second_cache_key),
        ..
    } = second
    else {
        panic!(
            "cached article data must not bypass revoked anonymous page:view permission"
        );
    };
    assert!(
        second_cache_key.contains(":permission=site=")
            && second_cache_key.contains(",user="),
        "permission revocation must move anonymous article cache reads to a new key: {second_cache_key}"
    );
    assert_ne!(
        first_cache_key, second_cache_key,
        "permission revocation must move anonymous article cache reads to a new key"
    );
}

#[tokio::test]
async fn wikidot_site_include_uses_local_dependency_page_for_site_qualified_include() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site.site_id,
        Reference::Slug(Cow::Borrowed("theme:codex-include-fallback")),
    );
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

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site.site_id,
        Reference::Slug(Cow::Borrowed("include-consumer")),
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
async fn missing_remote_site_include_does_not_fall_back_to_same_slug_local_page() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let slug = "missing-remote-include-self-cycle";

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site.site_id,
        Reference::Slug(Cow::Borrowed(slug)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site.site_id,
            "wikitext": concat!(
                "Before missing remote include.\n",
                "[[include :missing-remote:missing-remote-include-self-cycle]]\n",
                "After missing remote include.\n",
            ),
            "title": "Missing Remote Include",
            "alt_title": null,
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "create missing remote include regression",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site.site.site_id,
            "page": slug,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("page with a missing remote include should still render");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(html.contains("Before missing remote include."), "{html}");
    assert!(html.contains("After missing remote include."), "{html}");
    assert!(
        html.contains("No such page: :missing-remote:missing-remote-include-self-cycle"),
        "{html}",
    );
}

#[tokio::test]
async fn direct_message_render_leaves_image_block_include_literal() {
    let runner = TestRunner::setup().await;
    let settings =
        WikitextSettings::from_mode(WikitextMode::DirectMessage, Layout::Wikidot);
    assert!(
        !settings.enable_page_syntax,
        "DirectMessage rendering should have page syntax disabled"
    );
    let page_info = PageInfo {
        page: Cow::Borrowed(""),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed(""),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };

    let output = RenderService::render(
        runner.context(),
        "[[include component:image-block name=direct-message.jpg]]".to_owned(),
        &page_info,
        &settings,
    )
    .await
    .expect("direct message render should succeed");
    let html = output.html_output.body;

    assert!(
        html.contains("component:image-block"),
        "direct message render should keep literal include text inert:\n{html}"
    );
    for forbidden in ["scp-image-block", "local--files"] {
        assert!(
            !html.contains(forbidden),
            "direct message image-block include should not be pre-expanded into page markup:\n{html}"
        );
    }
}

#[tokio::test]
async fn html_block_render_leaves_image_block_include_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let slug = "fixture-image-block-html-literal";

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(slug)),
    );
    let page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": concat!(
                "[[html]]\n",
                "<template>[[include component:image-block name=raw-html.jpg]]</template>\n",
                "[[/html]]\n",
            ),
            "title": "Fixture Image Block HTML Literal",
            "alt_title": null,
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "create image-block HTML literal fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let html_block = run_endpoint!(
        runner,
        text_block_get_index,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "block_type": "html",
            "index": 1,
        }),
    )
    .expect("HTML block should be stored");
    let response = runner
        .context()
        .s3_tblocks_bucket()
        .get_object(&html_block.s3_filename)
        .await
        .expect("HTML block object should be readable");
    let html =
        String::from_utf8(response.into()).expect("HTML block object should be UTF-8");

    for forbidden in ["scp-image-block".to_owned(), format!("local--files/{slug}")] {
        assert!(
            !html.contains(&forbidden),
            "HTML block image-block include should not be pre-expanded:\n{html}"
        );
    }
    assert!(
        html.contains("[[include component:image-block name=raw-html.jpg]]"),
        "HTML block should retain the literal image-block include:\n{html}"
    );
}

#[tokio::test]
async fn non_scp_page_render_does_not_hardcode_scp_image_block_include() {
    let runner = TestRunner::setup().await;
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    assert!(
        settings.enable_page_syntax,
        "page rendering should exercise normal include handling"
    );
    let page_info = PageInfo {
        page: Cow::Borrowed("image-block-consumer"),
        category: None,
        site: Cow::Borrowed("sandbox-for-codex"),
        title: Cow::Borrowed("Image Block Consumer"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };

    let output = RenderService::render(
        runner.context(),
        concat!(
            "[[include component:image-block name=custom.jpg|caption=Custom block.]]\n",
            "[[include :sandbox-for-codex:component:image-block name=custom.jpg]]\n",
        )
        .to_owned(),
        &page_info,
        &settings,
    )
    .await
    .expect("non-SCP page render should succeed");
    let html = output.html_output.body;

    for forbidden in [
        "scp-image-block",
        "local--files/image-block-consumer/custom.jpg",
    ] {
        assert!(
            !html.contains(forbidden),
            "non-SCP page render should use normal include handling, not the SCP image-block prepass:\n{html}"
        );
    }
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
        "BF_DEFAULT_START\n[[module Backlinks]]\nBF_DEFAULT_END",
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
        r#"<div class="backlinks-module-box">"#,
        r#"<a href="/fixture-backlinks-linker-alpha">Fixture Backlinks Linker Alpha</a>"#,
        r#"<a href="/fixture-backlinks-linker-beta">Fixture Backlinks Linker Beta</a>"#,
        "BF_DEFAULT_END",
    ] {
        assert!(
            html.contains(expected),
            "Backlinks module output should contain {expected:?}:\n{html}"
        );
    }

    for forbidden in [
        "TODO: module Backlinks",
        "[[module Backlinks",
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
}

#[tokio::test]
async fn backlinks_module_with_unsupported_arguments_remains_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let target_slug = "fixture-backlinks-unsupported-target";

    create_listpages_test_page(
        &mut runner,
        site_id,
        target_slug,
        "Fixture Backlinks Unsupported Target",
        "Unsupported backlinks marker.\n[[module Backlinks page=\"start\"]]",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-backlinks-unsupported-linker",
        "Fixture Backlinks Unsupported Linker",
        &format!("[[[{target_slug}|unsupported target link]]]"),
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

    assert!(
        html.contains("TODO: module Backlinks") || html.contains("[[module Backlinks"),
        "unsupported Backlinks arguments should remain literal/degraded:\n{html}"
    );
    assert!(
        !html.contains("Fixture Backlinks Unsupported Linker"),
        "unsupported Backlinks arguments must not render a guessed incoming-link list:\n{html}"
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
        &target_a_slug,
        &target_b_slug,
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
async fn listpages_fragment_content_expands_child_includes() {
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

    for expected in [
        "Fragment before include.",
        INCLUDE_MARKER,
        "Fragment after include.",
    ] {
        assert!(
            html.contains(expected),
            "fragment ListPages content should contain {expected:?}:\n{html}"
        );
    }
    assert!(
        !html.contains("[[include"),
        "fragment ListPages content should expand child includes before rendering:\n{html}"
    );
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

#[tokio::test]
async fn page_revision_reads_require_page_view_permission() {
    let mut runner = TestRunner::setup().await;
    const SITE_SLUG: &str = "scp-wiki";
    const PAGE_SLUG: &str = "fixture-private-revision-read";
    const PRIVATE_CATEGORY: &str = "fixture-revision-read-private-view";

    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("Seeded site not found");
    let site_id = site.site.site_id;

    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(PAGE_SLUG))),
    });
    let created = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "private revision body marker",
            "title": "Private Revision Read",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create private revision read fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(created.parser_errors.is_empty());
    set_listpages_test_category_slug(&runner, site_id, PAGE_SLUG, PRIVATE_CATEGORY).await;

    runner.set_request_context(RequestContext::default());

    let error = run_endpoint_err!(
        runner,
        page_revision_get,
        json!({
            "site_id": site_id,
            "page_id": created.page_id,
            "revision_number": 0,
            "details": {
                "wikitext": true,
                "compiled_html": true
            },
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    let error = run_endpoint_err!(
        runner,
        page_revision_range,
        json!({
            "site_id": site_id,
            "page_id": created.page_id,
            "revision_number": 0,
            "revision_direction": "before",
            "limit": 1,
            "details": {
                "wikitext": true,
                "compiled_html": true
            },
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    let error = run_endpoint_err!(
        runner,
        page_revision_count,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(PAGE_SLUG))),
    });

    let revision = run_endpoint!(
        runner,
        page_revision_get,
        json!({
            "site_id": site_id,
            "page_id": created.page_id,
            "revision_number": 0,
            "details": {
                "wikitext": true,
                "compiled_html": true
            },
        }),
    )
    .expect("admin should be allowed to view private page revision");
    assert_eq!(
        revision.wikitext.as_deref(),
        Some("private revision body marker")
    );
    assert!(
        revision
            .compiled_body_html
            .as_deref()
            .is_some_and(|html| html.contains("private revision body marker")),
    );

    let revisions = run_endpoint!(
        runner,
        page_revision_range,
        json!({
            "site_id": site_id,
            "page_id": created.page_id,
            "revision_number": 0,
            "revision_direction": "before",
            "limit": 1,
            "details": {
                "wikitext": true,
                "compiled_html": true
            },
        }),
    );
    assert_eq!(revisions.len(), 1);
    assert_eq!(
        revisions[0].wikitext.as_deref(),
        Some("private revision body marker")
    );

    let count = run_endpoint!(
        runner,
        page_revision_count,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
        }),
    );
    assert_eq!(count.revision_count.get(), 1);
}

#[tokio::test]
async fn file_get_requires_parent_page_view_permission() {
    let mut runner = TestRunner::setup().await;
    const SITE_SLUG: &str = "scp-wiki";
    const PAGE_SLUG: &str = "fixture-private-file-read";
    const PUBLIC_PAGE_SLUG: &str = "fixture-public-file-read";
    const PRIVATE_CATEGORY: &str = "fixture-file-read-private-view";
    const FILE_NAME: &str = "private-attachment.txt";
    const PUBLIC_FILE_NAME: &str = "public-attachment.txt";

    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("Seeded site not found");
    let site_id = site.site.site_id;

    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(PAGE_SLUG))),
    });
    let page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "private file parent page",
            "title": "Private File Read",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create private file read fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(page.parser_errors.is_empty());
    set_listpages_test_category_slug(&runner, site_id, PAGE_SLUG, PRIVATE_CATEGORY).await;

    let file_id =
        create_empty_file_fixture(&runner, site_id, page.page_id, FILE_NAME).await;

    let public_page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "public file parent page",
            "title": "Public File Read",
            "alt_title": null,
            "slug": PUBLIC_PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create public file read fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(public_page.parser_errors.is_empty());
    let public_file_id = create_empty_file_fixture(
        &runner,
        site_id,
        public_page.page_id,
        PUBLIC_FILE_NAME,
    )
    .await;

    runner.set_request_context(RequestContext::default());

    let public_output = run_endpoint!(
        runner,
        file_get,
        json!({
            "site_id": site_id,
            "page_id": public_page.page_id,
            "file": PUBLIC_FILE_NAME,
            "details": {
                "data": false
            },
        }),
    )
    .expect("anonymous user should be allowed to view public page file");
    assert_eq!(public_output.file_id, public_file_id);
    assert_eq!(public_output.name, PUBLIC_FILE_NAME);

    let error = run_endpoint_err!(
        runner,
        file_get,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file": FILE_NAME,
            "details": {
                "data": false
            },
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(PAGE_SLUG))),
    });

    let output = run_endpoint!(
        runner,
        file_get,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file": FILE_NAME,
            "details": {
                "data": false
            },
        }),
    )
    .expect("admin should be allowed to view private page file");

    assert_eq!(output.file_id, file_id);
    assert_eq!(output.name, FILE_NAME);
    assert_eq!(output.mime, EMPTY_BLOB_MIME);
    assert_eq!(output.s3_hash.as_ref(), &EMPTY_BLOB_HASH);
    assert!(output.data.is_none());
}

#[tokio::test]
async fn forum_post_reads_require_parent_page_view_permission() {
    let mut runner = TestRunner::setup().await;
    const SITE_SLUG: &str = "scp-wiki";
    const PAGE_SLUG: &str = "fixture-private-forum-post-read";
    const PUBLIC_PAGE_SLUG: &str = "fixture-public-forum-post-read";
    const PRIVATE_CATEGORY: &str = "fixture-forum-post-read-private-view";

    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("Seeded site not found");
    let site_id = site.site.site_id;

    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(PAGE_SLUG))),
    });
    let page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "private forum post parent page",
            "title": "Private Forum Post Read",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create private forum post read fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(page.parser_errors.is_empty());
    set_listpages_test_category_slug(&runner, site_id, PAGE_SLUG, PRIVATE_CATEGORY).await;

    let public_page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "public forum post parent page",
            "title": "Public Forum Post Read",
            "alt_title": null,
            "slug": PUBLIC_PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create public forum post read fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(public_page.parser_errors.is_empty());

    let group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "Forum Post Read ACL Group".to_owned(),
            description: "Forum post read ACL fixture group".to_owned(),
            visible: true,
            sort_index: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("forum group fixture should be created");
    let forum_category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "Forum Post Read ACL Category".to_owned(),
            description: "Forum post read ACL fixture category".to_owned(),
            sort_index: None,
            max_nest_level: Some(3),
            per_page_discussion: Some(true),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("forum category fixture should be created");

    let private_thread = ForumThreadService::create(
        runner.context(),
        CreateForumThread {
            forum_category_id: forum_category.forum_category_id,
            user_id: ADMIN_USER_ID,
            associated_page_id: Some(page.page_id),
            title: "Private forum post read thread".to_owned(),
            description: String::new(),
            sticky: false,
            from_wikidot: false,
        },
    )
    .await
    .expect("private forum thread fixture should be created");
    let private_post = ForumPostService::create(
        runner.context(),
        CreateForumPost {
            forum_thread_id: private_thread.forum_thread_id,
            parent_post_id: None,
            user_id: ADMIN_USER_ID,
            title: "Private forum post read title".to_owned(),
            wikitext: "private forum post body marker".to_owned(),
            comments: "create private forum post fixture".to_owned(),
            from_wikidot: false,
        },
    )
    .await
    .expect("private forum post fixture should be created");
    assert!(private_post.parser_errors.is_empty());

    let public_thread = ForumThreadService::create(
        runner.context(),
        CreateForumThread {
            forum_category_id: forum_category.forum_category_id,
            user_id: ADMIN_USER_ID,
            associated_page_id: Some(public_page.page_id),
            title: "Public forum post read thread".to_owned(),
            description: String::new(),
            sticky: false,
            from_wikidot: false,
        },
    )
    .await
    .expect("public forum thread fixture should be created");
    let public_post = ForumPostService::create(
        runner.context(),
        CreateForumPost {
            forum_thread_id: public_thread.forum_thread_id,
            parent_post_id: None,
            user_id: ADMIN_USER_ID,
            title: "Public forum post read title".to_owned(),
            wikitext: "public forum post body marker".to_owned(),
            comments: "create public forum post fixture".to_owned(),
            from_wikidot: false,
        },
    )
    .await
    .expect("public forum post fixture should be created");
    assert!(public_post.parser_errors.is_empty());

    runner.set_request_context(RequestContext::default());

    let public_selection = run_endpoint!(
        runner,
        forum_post_select,
        json!({
            "site_id": site_id,
            "page": PUBLIC_PAGE_SLUG,
        }),
    );
    assert_eq!(public_selection, vec![public_post.forum_post_id]);

    let private_selection = run_endpoint!(
        runner,
        forum_post_select,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
        }),
    );
    assert!(private_selection.is_empty());

    let visible_posts = run_endpoint!(
        runner,
        forum_post_get,
        json!({
            "site_id": site_id,
            "posts": [private_post.forum_post_id, public_post.forum_post_id],
        }),
    );
    assert_eq!(visible_posts.len(), 1);
    let visible_post = serde_json::to_value(&visible_posts[0])
        .expect("forum post output should serialize");
    assert_eq!(visible_post["id"], json!(public_post.forum_post_id));
    assert_eq!(
        visible_post["content"],
        json!("public forum post body marker")
    );

    let private_summary = run_endpoint!(
        runner,
        forum_post_page_summary,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
        }),
    );
    let private_summary_value = serde_json::to_value(private_summary)
        .expect("private forum summary should serialize");
    assert_eq!(private_summary_value["comments"], json!(0));

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(PAGE_SLUG))),
    });

    let admin_selection = run_endpoint!(
        runner,
        forum_post_select,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
        }),
    );
    assert_eq!(admin_selection, vec![private_post.forum_post_id]);

    let admin_posts = run_endpoint!(
        runner,
        forum_post_get,
        json!({
            "site_id": site_id,
            "posts": [private_post.forum_post_id],
        }),
    );
    assert_eq!(admin_posts.len(), 1);
    let admin_post = serde_json::to_value(&admin_posts[0])
        .expect("admin forum post output should serialize");
    assert_eq!(admin_post["id"], json!(private_post.forum_post_id));
    assert_eq!(
        admin_post["content"],
        json!("private forum post body marker")
    );

    let admin_summary = run_endpoint!(
        runner,
        forum_post_page_summary,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
        }),
    );
    let admin_summary_value = serde_json::to_value(admin_summary)
        .expect("admin forum summary should serialize");
    assert_eq!(admin_summary_value["comments"], json!(1));
}

#[tokio::test]
async fn page_get_files_requires_parent_page_view_permission() {
    let mut runner = TestRunner::setup().await;
    const SITE_SLUG: &str = "scp-wiki";
    const PAGE_SLUG: &str = "fixture-private-file-list";
    const PUBLIC_PAGE_SLUG: &str = "fixture-public-file-list";
    const PRIVATE_CATEGORY: &str = "fixture-file-list-private-view";
    const FILE_NAME: &str = "private-list-attachment.txt";
    const PUBLIC_FILE_NAME: &str = "public-list-attachment.txt";

    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("Seeded site not found");
    let site_id = site.site.site_id;

    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(PAGE_SLUG))),
    });
    let page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "private file list parent page",
            "title": "Private File List",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create private file list fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(page.parser_errors.is_empty());
    set_listpages_test_category_slug(&runner, site_id, PAGE_SLUG, PRIVATE_CATEGORY).await;

    let file_id =
        create_empty_file_fixture(&runner, site_id, page.page_id, FILE_NAME).await;

    let public_page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "public file list parent page",
            "title": "Public File List",
            "alt_title": null,
            "slug": PUBLIC_PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create public file list fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(public_page.parser_errors.is_empty());
    let public_file_id = create_empty_file_fixture(
        &runner,
        site_id,
        public_page.page_id,
        PUBLIC_FILE_NAME,
    )
    .await;

    runner.set_request_context(RequestContext::default());

    let public_output = run_endpoint!(
        runner,
        page_get_files,
        json!({
            "site_id": site_id,
            "page_id": public_page.page_id,
            "deleted": false,
        }),
    );
    assert_eq!(public_output.len(), 1);
    assert_eq!(public_output[0].file_id, public_file_id);
    assert_eq!(public_output[0].name, PUBLIC_FILE_NAME);

    let error = run_endpoint_err!(
        runner,
        page_get_files,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "deleted": false,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(PAGE_SLUG))),
    });

    let output = run_endpoint!(
        runner,
        page_get_files,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "deleted": false,
        }),
    );

    assert_eq!(output.len(), 1);
    assert_eq!(output[0].file_id, file_id);
    assert_eq!(output[0].name, FILE_NAME);
    assert_eq!(output[0].mime, EMPTY_BLOB_MIME);
    assert_eq!(output[0].s3_hash.as_ref(), &EMPTY_BLOB_HASH);
    assert!(output[0].data.is_none());
}

#[tokio::test]
async fn page_mutations_require_page_permissions() {
    let mut runner = TestRunner::setup().await;
    const SITE_SLUG: &str = "scp-wiki";
    const PRIVATE_CATEGORY: &str = "fixture-page-mutation-private";
    const PAGE_SLUG: &str = "fixture-page-mutation-private:target";
    const BLOCKED_SLUG: &str = "fixture-page-mutation-private:blocked";

    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("Seeded site not found");
    let site_id = site.site.site_id;

    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        PRIVATE_CATEGORY,
        SAMPLE_USER_ID,
        &[Action::View, Action::Create, Action::Edit],
        "sample-mutator",
    )
    .await;

    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(BLOCKED_SLUG)),
    );
    let error = run_endpoint_err!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "blocked private page create",
            "title": "Blocked Private Page",
            "alt_title": null,
            "slug": BLOCKED_SLUG,
            "layout": "wikidot",
            "revision_comments": "blocked create",
            "user_id": UNKNOWN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(PAGE_SLUG)),
    );
    let page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "private page mutation target",
            "title": "Private Page Mutation Target",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create private mutation target",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(page.parser_errors.is_empty());

    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let error = run_endpoint_err!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": page.page_id,
            "last_revision_id": page.revision_id,
            "revision_comments": "blocked edit",
            "user_id": UNKNOWN_USER_ID,
            "title": "Unauthorized Edit",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let edit = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": page.page_id,
            "last_revision_id": page.revision_id,
            "revision_comments": "authorized edit",
            "user_id": SAMPLE_USER_ID,
            "title": "Authorized Edit",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("admin page edit should create a revision");
    assert!(edit.revision_id > page.revision_id);

    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let error = run_endpoint_err!(
        runner,
        page_rollback,
        json!({
            "site_id": site_id,
            "page": page.page_id,
            "last_revision_id": edit.revision_id,
            "revision_number": 0,
            "revision_comments": "blocked rollback",
            "user_id": UNKNOWN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let rollback = run_endpoint!(
        runner,
        page_rollback,
        json!({
            "site_id": site_id,
            "page": page.page_id,
            "last_revision_id": edit.revision_id,
            "revision_number": 0,
            "revision_comments": "authorized rollback",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("authorized page rollback should create a revision");
    assert!(rollback.revision_id > edit.revision_id);

    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let error = run_endpoint_err!(
        runner,
        page_set_layout,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "layout": "wikidot",
            "user_id": UNKNOWN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    run_endpoint!(
        runner,
        page_set_layout,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "layout": "wikidot",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let error = run_endpoint_err!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": page.page_id,
            "last_revision_id": rollback.revision_id,
            "revision_comments": "blocked delete",
            "user_id": UNKNOWN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let _deleted = run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": page.page_id,
            "last_revision_id": rollback.revision_id,
            "revision_comments": "authorized delete",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
}

#[tokio::test]
async fn page_move_requires_destination_create_permission() {
    let mut runner = TestRunner::setup().await;
    const SITE_SLUG: &str = "scp-wiki";
    const SOURCE_CATEGORY: &str = "fixture-page-move-source-private";
    const DESTINATION_CATEGORY: &str = "fixture-page-move-destination-private";
    const PAGE_SLUG: &str = "fixture-page-move-source-private:target";
    const BLOCKED_DESTINATION_SLUG: &str = "fixture-page-move-destination-private:target";
    const ALLOWED_DESTINATION_SLUG: &str = "fixture-page-move-source-private:moved";

    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("Seeded site not found");
    let site_id = site.site.site_id;

    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        SOURCE_CATEGORY,
        SAMPLE_USER_ID,
        &[Action::View, Action::Create, Action::Edit],
        "sample-mutator",
    )
    .await;
    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        DESTINATION_CATEGORY,
        ADMIN_USER_ID,
        &[Action::View, Action::Create, Action::Edit],
        "admin-mutator",
    )
    .await;

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(PAGE_SLUG)),
    );
    let page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "private page move target",
            "title": "Private Page Move Target",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create private move target",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(page.parser_errors.is_empty());

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let error = run_endpoint_err!(
        runner,
        page_move,
        json!({
            "site_id": site_id,
            "page": page.page_id,
            "new_slug": BLOCKED_DESTINATION_SLUG,
            "last_revision_id": page.revision_id,
            "revision_comments": "blocked cross-category move",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    let moved = run_endpoint!(
        runner,
        page_move,
        json!({
            "site_id": site_id,
            "page": page.page_id,
            "new_slug": ALLOWED_DESTINATION_SLUG,
            "last_revision_id": page.revision_id,
            "revision_comments": "authorized same-category move",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(moved.revision_id > page.revision_id);
}

#[tokio::test]
async fn page_restore_default_slug_requires_destination_create_permission() {
    let mut runner = TestRunner::setup().await;
    const SITE_SLUG: &str = "scp-wiki";
    const PRIVATE_CATEGORY: &str = "fixture-page-restore-private";
    const DESTINATION_CATEGORY: &str = "fixture-page-restore-destination-private";
    const PAGE_SLUG: &str = "fixture-page-restore-private:target";
    const EXPLICIT_PAGE_SLUG: &str = "fixture-page-restore-private:explicit";
    const EXPLICIT_DESTINATION_SLUG: &str =
        "fixture-page-restore-destination-private:explicit";

    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("Seeded site not found");
    let site_id = site.site.site_id;

    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        PRIVATE_CATEGORY,
        ADMIN_USER_ID,
        &[Action::View, Action::Create, Action::Edit],
        "admin-mutator",
    )
    .await;
    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        PRIVATE_CATEGORY,
        SAMPLE_USER_ID,
        &[Action::View, Action::Edit],
        "sample-editor",
    )
    .await;
    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        DESTINATION_CATEGORY,
        ADMIN_USER_ID,
        &[Action::View, Action::Create, Action::Edit],
        "admin-mutator",
    )
    .await;

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(PAGE_SLUG)),
    );
    let page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "private page restore target",
            "title": "Private Page Restore Target",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create private restore target",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(page.parser_errors.is_empty());

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let _deleted = run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": page.page_id,
            "last_revision_id": page.revision_id,
            "revision_comments": "sample delete before restore",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let error = run_endpoint_err!(
        runner,
        page_restore,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "revision_comments": "blocked default restore",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let _restored = run_endpoint!(
        runner,
        page_restore,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "revision_comments": "authorized default restore",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(EXPLICIT_PAGE_SLUG)),
    );
    let explicit_page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "private explicit page restore target",
            "title": "Private Explicit Page Restore Target",
            "alt_title": null,
            "slug": EXPLICIT_PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create private explicit restore target",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(explicit_page.parser_errors.is_empty());

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Id(explicit_page.page_id),
    );
    let _deleted = run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": explicit_page.page_id,
            "last_revision_id": explicit_page.revision_id,
            "revision_comments": "sample delete before explicit restore",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let error = run_endpoint_err!(
        runner,
        page_restore,
        json!({
            "site_id": site_id,
            "page_id": explicit_page.page_id,
            "slug": EXPLICIT_DESTINATION_SLUG,
            "revision_comments": "blocked explicit restore",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(explicit_page.page_id),
    );
    let _restored = run_endpoint!(
        runner,
        page_restore,
        json!({
            "site_id": site_id,
            "page_id": explicit_page.page_id,
            "slug": EXPLICIT_DESTINATION_SLUG,
            "revision_comments": "authorized explicit restore",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
}

#[tokio::test]
async fn file_mutations_require_parent_page_edit_permission() {
    let mut runner = TestRunner::setup().await;
    const SITE_SLUG: &str = "scp-wiki";
    const PRIVATE_CATEGORY: &str = "fixture-file-mutation-private";
    const BLOCKED_DESTINATION_CATEGORY: &str =
        "fixture-file-mutation-destination-private";
    const PAGE_SLUG: &str = "fixture-file-mutation-private:target";
    const DESTINATION_PAGE_SLUG: &str = "fixture-file-mutation-private:destination";
    const BLOCKED_DESTINATION_PAGE_SLUG: &str =
        "fixture-file-mutation-destination-private:blocked";
    const FILE_EDIT_NAME: &str = "private-edit.txt";
    const FILE_MOVE_NAME: &str = "private-move.txt";
    const FILE_DELETE_NAME: &str = "private-delete.txt";
    const FILE_RESTORE_NAME: &str = "private-restore.txt";
    const FILE_ROLLBACK_NAME: &str = "private-rollback.txt";

    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("Seeded site not found");
    let site_id = site.site.site_id;

    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        PRIVATE_CATEGORY,
        SAMPLE_USER_ID,
        &[Action::View, Action::Create, Action::Edit],
        "sample-mutator",
    )
    .await;
    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        BLOCKED_DESTINATION_CATEGORY,
        ADMIN_USER_ID,
        &[Action::View, Action::Create, Action::Edit],
        "admin-mutator",
    )
    .await;

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(PAGE_SLUG)),
    );
    let page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "private file mutation parent page",
            "title": "Private File Mutation",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create private file mutation fixture",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(page.parser_errors.is_empty());
    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(DESTINATION_PAGE_SLUG)),
    );
    let destination_page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "private file mutation destination page",
            "title": "Private File Mutation Destination",
            "alt_title": null,
            "slug": DESTINATION_PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create private file mutation destination",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(destination_page.parser_errors.is_empty());
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(BLOCKED_DESTINATION_PAGE_SLUG)),
    );
    let blocked_destination_page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "private file mutation blocked destination page",
            "title": "Private File Mutation Blocked Destination",
            "alt_title": null,
            "slug": BLOCKED_DESTINATION_PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create private file mutation blocked destination",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(blocked_destination_page.parser_errors.is_empty());

    let edit_file_id =
        create_empty_file_fixture(&runner, site_id, page.page_id, FILE_EDIT_NAME).await;
    let move_file_id =
        create_empty_file_fixture(&runner, site_id, page.page_id, FILE_MOVE_NAME).await;
    let delete_file_id =
        create_empty_file_fixture(&runner, site_id, page.page_id, FILE_DELETE_NAME).await;
    let restore_file_id =
        create_empty_file_fixture(&runner, site_id, page.page_id, FILE_RESTORE_NAME)
            .await;
    let rollback_file_id =
        create_empty_file_fixture(&runner, site_id, page.page_id, FILE_ROLLBACK_NAME)
            .await;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(SAMPLE_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(page.page_id)),
    });
    let file = run_endpoint!(
        runner,
        file_get,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file": FILE_DELETE_NAME,
            "details": {
                "data": false
            },
        }),
    )
    .expect("admin should be allowed to view private file mutation fixture");
    assert_eq!(file.file_id, delete_file_id);

    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let error = run_endpoint_err!(
        runner,
        file_create,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "name": "blocked-create.txt",
            "uploaded_blob_id": "not-used-before-permission-denial",
            "revision_comments": "blocked file create",
            "user_id": UNKNOWN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let edit_file = run_endpoint!(
        runner,
        file_get,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file": FILE_EDIT_NAME,
            "details": {
                "data": false
            },
        }),
    )
    .expect("edit file fixture should be visible to the authorized user");
    assert_eq!(edit_file.file_id, edit_file_id);

    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let error = run_endpoint_err!(
        runner,
        file_edit,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file_id": edit_file.file_id,
            "last_revision_id": edit_file.revision_id,
            "revision_comments": "blocked file edit",
            "user_id": UNKNOWN_USER_ID,
            "name": "blocked-edit.txt",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let edited = run_endpoint!(
        runner,
        file_edit,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file_id": edit_file.file_id,
            "last_revision_id": edit_file.revision_id,
            "revision_comments": "authorized file edit",
            "user_id": SAMPLE_USER_ID,
            "name": "authorized-edit.txt",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("authorized file edit should create a revision");
    assert!(edited.file_revision_id > edit_file.revision_id);

    let move_file = run_endpoint!(
        runner,
        file_get,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file": FILE_MOVE_NAME,
            "details": {
                "data": false
            },
        }),
    )
    .expect("move file fixture should be visible to the authorized user");
    assert_eq!(move_file.file_id, move_file_id);

    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let error = run_endpoint_err!(
        runner,
        file_move,
        json!({
            "site_id": site_id,
            "file_id": move_file.file_id,
            "current_page_id": page.page_id,
            "destination_page": destination_page.page_id,
            "last_revision_id": move_file.revision_id,
            "revision_comments": "blocked file move",
            "user_id": UNKNOWN_USER_ID,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let error = run_endpoint_err!(
        runner,
        file_move,
        json!({
            "site_id": site_id,
            "file_id": move_file.file_id,
            "current_page_id": page.page_id,
            "destination_page": blocked_destination_page.page_id,
            "last_revision_id": move_file.revision_id,
            "revision_comments": "blocked file move destination",
            "user_id": SAMPLE_USER_ID,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    let moved = run_endpoint!(
        runner,
        file_move,
        json!({
            "site_id": site_id,
            "file_id": move_file.file_id,
            "current_page_id": page.page_id,
            "destination_page": destination_page.page_id,
            "last_revision_id": move_file.revision_id,
            "revision_comments": "authorized file move",
            "user_id": SAMPLE_USER_ID,
        }),
    )
    .expect("authorized file move should create a revision");
    assert!(moved.file_revision_id > move_file.revision_id);

    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let error = run_endpoint_err!(
        runner,
        file_delete,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file": FILE_DELETE_NAME,
            "last_revision_id": file.revision_id,
            "revision_comments": "blocked file delete",
            "user_id": UNKNOWN_USER_ID,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let deleted = run_endpoint!(
        runner,
        file_delete,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file": FILE_DELETE_NAME,
            "last_revision_id": file.revision_id,
            "revision_comments": "authorized file delete",
            "user_id": SAMPLE_USER_ID,
        }),
    );
    assert_eq!(deleted.file_id, delete_file_id);
    assert!(deleted.file_revision_id > file.revision_id);

    let restore_file = run_endpoint!(
        runner,
        file_get,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file": FILE_RESTORE_NAME,
            "details": {
                "data": false
            },
        }),
    )
    .expect("restore file fixture should be visible to the authorized user");
    assert_eq!(restore_file.file_id, restore_file_id);
    let deleted_restore = run_endpoint!(
        runner,
        file_delete,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file": FILE_RESTORE_NAME,
            "last_revision_id": restore_file.revision_id,
            "revision_comments": "prepare file restore",
            "user_id": SAMPLE_USER_ID,
        }),
    );

    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let error = run_endpoint_err!(
        runner,
        file_restore,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file_id": restore_file.file_id,
            "revision_comments": "blocked file restore",
            "user_id": UNKNOWN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let error = run_endpoint_err!(
        runner,
        file_restore,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file_id": restore_file.file_id,
            "new_page": blocked_destination_page.page_id,
            "revision_comments": "blocked file restore destination",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    let restored = run_endpoint!(
        runner,
        file_restore,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file_id": restore_file.file_id,
            "revision_comments": "authorized file restore",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(restored.file_id, restore_file_id);
    assert!(restored.file_revision_id > deleted_restore.file_revision_id);

    let rollback_file = run_endpoint!(
        runner,
        file_get,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file": FILE_ROLLBACK_NAME,
            "details": {
                "data": false
            },
        }),
    )
    .expect("rollback file fixture should be visible to the authorized user");
    assert_eq!(rollback_file.file_id, rollback_file_id);
    let edited_for_rollback = run_endpoint!(
        runner,
        file_edit,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file_id": rollback_file.file_id,
            "last_revision_id": rollback_file.revision_id,
            "revision_comments": "prepare file rollback",
            "user_id": SAMPLE_USER_ID,
            "name": "rollback-new-name.txt",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("authorized file edit should prepare a rollback target");

    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let error = run_endpoint_err!(
        runner,
        file_rollback,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file": "rollback-new-name.txt",
            "last_revision_id": edited_for_rollback.file_revision_id,
            "revision_number": 0,
            "revision_comments": "blocked file rollback",
            "user_id": UNKNOWN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let rolled_back = run_endpoint!(
        runner,
        file_rollback,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file": "rollback-new-name.txt",
            "last_revision_id": edited_for_rollback.file_revision_id,
            "revision_number": 0,
            "revision_comments": "authorized file rollback",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("authorized file rollback should create a revision");
    assert!(rolled_back.file_revision_id > edited_for_rollback.file_revision_id);
}

#[tokio::test]
async fn parent_mutations_require_child_page_edit_permission() {
    let mut runner = TestRunner::setup().await;
    const SITE_SLUG: &str = "scp-wiki";
    const PRIVATE_CATEGORY: &str = "fixture-parent-mutation-private";
    const CHILD_SLUG: &str = "fixture-parent-mutation-private:child";
    const PARENT_SLUG: &str = "fixture-parent-mutation-private:parent";

    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("Seeded site not found");
    let site_id = site.site.site_id;

    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        PRIVATE_CATEGORY,
        SAMPLE_USER_ID,
        &[Action::View, Action::Create, Action::Edit],
        "sample-mutator",
    )
    .await;

    for (slug, title) in [
        (CHILD_SLUG, "Private Child Page"),
        (PARENT_SLUG, "Private Parent Page"),
    ] {
        set_mutation_request_context(
            &mut runner,
            SAMPLE_USER_ID,
            site_id,
            Reference::Slug(Cow::Borrowed(slug)),
        );
        let page = run_endpoint!(
            runner,
            page_create,
            json!({
                "site_id": site_id,
                "wikitext": title,
                "title": title,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "create parent mutation fixture",
                "user_id": SAMPLE_USER_ID,
                "ip_address": common::IP_ADDRESS,
            }),
        );
        assert!(page.parser_errors.is_empty());
    }

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ANONYMOUS_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(CHILD_SLUG))),
    });
    let error = run_endpoint_err!(
        runner,
        parent_set,
        json!({
            "site_id": site_id,
            "parent": PARENT_SLUG,
            "child": CHILD_SLUG,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(SAMPLE_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(CHILD_SLUG))),
    });
    let created = run_endpoint!(
        runner,
        parent_set,
        json!({
            "site_id": site_id,
            "parent": PARENT_SLUG,
            "child": CHILD_SLUG,
        }),
    );
    assert!(created.is_some());

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ANONYMOUS_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(CHILD_SLUG))),
    });
    let error = run_endpoint_err!(
        runner,
        parent_remove,
        json!({
            "site_id": site_id,
            "parent": PARENT_SLUG,
            "child": CHILD_SLUG,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(SAMPLE_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(CHILD_SLUG))),
    });
    let removed = run_endpoint!(
        runner,
        parent_remove,
        json!({
            "site_id": site_id,
            "parent": PARENT_SLUG,
            "child": CHILD_SLUG,
        }),
    );
    assert!(removed.was_deleted);

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ANONYMOUS_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(CHILD_SLUG))),
    });
    let error = run_endpoint_err!(
        runner,
        parent_update,
        json!({
            "site_id": site_id,
            "child": CHILD_SLUG,
            "user_id": SAMPLE_USER_ID,
            "add": [PARENT_SLUG],
            "remove": null,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(SAMPLE_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(CHILD_SLUG))),
    });
    let updated = run_endpoint!(
        runner,
        parent_update,
        json!({
            "site_id": site_id,
            "child": CHILD_SLUG,
            "user_id": ANONYMOUS_USER_ID,
            "add": [PARENT_SLUG],
            "remove": null,
        }),
    );
    assert_eq!(updated.added.as_ref().map(Vec::len), Some(1));
}

#[tokio::test]
async fn text_block_get_index_requires_parent_page_view_permission() {
    let mut runner = TestRunner::setup().await;
    const SITE_SLUG: &str = "scp-wiki";
    const PAGE_SLUG: &str = "fixture-private-text-block-read";
    const PUBLIC_PAGE_SLUG: &str = "fixture-public-text-block-read";
    const PRIVATE_CATEGORY: &str = "fixture-text-block-read-private-view";

    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("Seeded site not found");
    let site_id = site.site.site_id;

    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(PAGE_SLUG))),
    });
    let page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "private hosted text block parent page",
            "title": "Private Text Block Read",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create private text block fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(page.parser_errors.is_empty());
    set_listpages_test_category_slug(&runner, site_id, PAGE_SLUG, PRIVATE_CATEGORY).await;
    create_text_block_fixture(
        &runner,
        page.page_id,
        TextBlockType::Html,
        1,
        None,
        "sentinel-private-html-block",
    )
    .await;
    create_text_block_fixture(
        &runner,
        page.page_id,
        TextBlockType::Code,
        2,
        Some("secret-code"),
        "sentinel-private-code-block",
    )
    .await;

    let public_page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "public hosted text block parent page",
            "title": "Public Text Block Read",
            "alt_title": null,
            "slug": PUBLIC_PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create public text block fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(public_page.parser_errors.is_empty());
    create_text_block_fixture(
        &runner,
        public_page.page_id,
        TextBlockType::Html,
        1,
        None,
        &format!("{}_html_1", public_page.page_id),
    )
    .await;

    runner.set_request_context(RequestContext::default());

    let public_block = run_endpoint!(
        runner,
        text_block_get_index,
        json!({
            "site_id": site_id,
            "page_id": public_page.page_id,
            "block_type": "html",
            "index": 1,
        }),
    )
    .expect("public text block should exist");
    assert_eq!(public_block.index, 1);
    assert_eq!(
        public_block.s3_filename,
        format!("{}_html_1", public_page.page_id)
    );

    let error = run_endpoint_err!(
        runner,
        text_block_get_index,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "block_type": "html",
            "index": 1,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    let error = run_endpoint_err!(
        runner,
        text_block_get_index,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "block_type": "code",
            "name": "secret-code",
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    let admin_session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "deepwell text block test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");

    let private_html = run_endpoint!(
        runner,
        text_block_get_index,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "block_type": "html",
            "index": 1,
            "session_token": admin_session_token,
        }),
    )
    .expect("private HTML block should exist");
    assert_eq!(private_html.index, 1);
    assert_eq!(private_html.s3_filename, "sentinel-private-html-block");

    let private_code = run_endpoint!(
        runner,
        text_block_get_index,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "block_type": "code",
            "name": "secret-code",
            "session_token": admin_session_token,
        }),
    )
    .expect("private named code block should exist");
    assert_eq!(private_code.index, 2);
    assert_eq!(private_code.s3_filename, "sentinel-private-code-block");
}

async fn create_empty_file_fixture(
    runner: &TestRunner,
    site_id: i64,
    page_id: i64,
    name: &str,
) -> i64 {
    let file = file::ActiveModel {
        name: Set(name.to_owned()),
        site_id: Set(site_id),
        page_id: Set(page_id),
        ..Default::default()
    }
    .insert(runner.context().transaction())
    .await
    .expect("file fixture should be inserted");
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
            mime: EMPTY_BLOB_MIME.to_owned(),
            blob_created: false,
            revision_comments: "create file fixture".to_owned(),
        },
    )
    .await
    .expect("file revision fixture should be created");

    file.file_id
}

async fn create_text_block_fixture(
    runner: &TestRunner,
    page_id: i64,
    block_type: TextBlockType,
    block_index: i16,
    block_name: Option<&str>,
    s3_filename: &str,
) {
    text_block::ActiveModel {
        block_type: Set(block_type),
        page_id: Set(page_id),
        block_index: Set(block_index),
        s3_filename: Set(s3_filename.to_owned()),
        block_name: Set(block_name.map(str::to_owned)),
        text_type: Set(None),
    }
    .insert(runner.context().transaction())
    .await
    .expect("text block fixture should be inserted");
}

async fn make_listpages_test_category_admin_only(
    runner: &TestRunner,
    site_id: i64,
    category_slug: &str,
) {
    let category_id =
        CategoryService::get_or_create(runner.context(), site_id, category_slug)
            .await
            .expect("private ListPages category should be created")
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
    .expect("private ListPages role should be created");
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
    .expect("private ListPages role permissions should be updated");
    RoleService::grant_role_to_user(
        runner.context(),
        GrantUserRoleInput {
            site_id,
            user_id: ADMIN_USER_ID,
            role_id: role.role_id,
            assigning_user_id: SYSTEM_USER_ID,
            expires_at: None,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("admin should receive private ListPages role");
}

async fn make_page_mutation_test_category_for_user(
    runner: &TestRunner,
    site_id: i64,
    category_slug: &str,
    user_id: i64,
    actions: &[Action],
    role_suffix: &str,
) {
    let category_id =
        CategoryService::get_or_create(runner.context(), site_id, category_slug)
            .await
            .expect("private mutation category should be created")
            .category_id;
    let role = RoleService::create(
        runner.context(),
        InternalCreateRoleInput {
            site_id,
            name: format!("{category_slug}-{role_suffix}"),
            description: None,
            is_virtual: false,
            parent_role_id: None,
            creating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("private mutation role should be created");
    PermissionService::update_permissions_for_role(
        runner.context(),
        UpdateRolePermissionsInput {
            site_id,
            role_reference: Reference::Id(role.role_id),
            new_permissions: actions
                .iter()
                .copied()
                .map(|action| Permission {
                    resource_type: Resource::Page,
                    resource_category: Some(Reference::Id(category_id)),
                    action,
                })
                .collect(),
            cascade_removals: false,
            updating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("private mutation role permissions should be updated");
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
    .expect("user should receive private mutation role");
}

async fn set_test_user_name(runner: &TestRunner, user_id: i64, name: &str) {
    let user = UserTable::find_by_id(user_id)
        .one(runner.context().transaction())
        .await
        .expect("test user lookup should not fail")
        .expect("test user should exist");
    let mut model = user.into_active_model();
    model.name = Set(name.to_owned());
    model.slug = Set(name.to_ascii_lowercase().replace(' ', "-"));
    model
        .update(runner.context().transaction())
        .await
        .expect("test user update should not fail");
}

async fn create_listpages_test_page(
    runner: &mut TestRunner,
    site_id: i64,
    slug: &str,
    title: &str,
    wikitext: &str,
) -> i64 {
    set_mutation_request_context(
        runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Owned(slug.to_owned())),
    );
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

#[tokio::test]
async fn listpages_content_shares_the_render_include_budget() {
    const COMPONENT_SLUG: &str = "component:listpages-include-budget-cell";
    const INDEX_SLUG: &str = "fixture-listpages-include-budget-index";
    const INCLUDE_MARKER: &str = "LISTPAGES_INCLUDE_BUDGET_CELL";
    const INCLUDES_PER_SOURCE: usize = 128;

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        COMPONENT_SLUG,
        "ListPages Include Budget Cell",
        INCLUDE_MARKER,
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "ListPages Include Budget Index",
        "placeholder",
    )
    .await;

    let child_wikitext =
        format!("[[include {COMPONENT_SLUG}]]\n").repeat(INCLUDES_PER_SOURCE);
    for (slug, title) in [
        (
            "fixture-listpages-include-budget-child-a",
            "ListPages Include Budget Child A",
        ),
        (
            "fixture-listpages-include-budget-child-b",
            "ListPages Include Budget Child B",
        ),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, &child_wikitext)
            .await;
        set_listpages_test_parent(&mut runner, site_id, slug, INDEX_SLUG).await;
    }

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
        }),
    )
    .expect("ListPages include-budget index should exist");
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(INDEX_SLUG))),
    });

    let page_info = PageInfo {
        page: Cow::Borrowed(INDEX_SLUG),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("ListPages Include Budget Index"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let page_id = PageId {
        site_id,
        category_id: page.page_category_id,
        page_id: page.page_id,
    };
    let direct_includes =
        format!("[[include {COMPONENT_SLUG}]]\n").repeat(INCLUDES_PER_SOURCE);
    let list_pages = |limit| {
        format!(
            "[[module ListPages parent=\".\" order=\"name\" limit=\"{limit}\"]]\n%%content%%\n[[/module]]"
        )
    };

    let within_budget = format!("{direct_includes}{}", list_pages(1));
    let output = RenderService::render_page(
        runner.context(),
        within_budget,
        &page_info,
        Layout::Wikidot,
        page_id,
    )
    .await
    .expect("128 direct and 128 ListPages includes should fit the public limit");
    assert_eq!(
        output.html_output.body.matches(INCLUDE_MARKER).count(),
        256,
        "the render at the public limit should expand every include",
    );

    let over_budget = format!("{direct_includes}{}", list_pages(2));
    let error = RenderService::render_page(
        runner.context(),
        over_budget.clone(),
        &page_info,
        Layout::Wikidot,
        page_id,
    )
    .await
    .expect_err("ListPages child content must not reset the public include budget");
    assert!(
        format!("{error:?}")
            .contains("include expansion exceeded maximum total includes 256"),
        "the shared-budget failure should report the render's original public limit: {error:?}",
    );

    let output = RenderService::render_corpus_page(
        runner.context(),
        over_budget,
        &page_info,
        Layout::Wikidot,
        page_id,
    )
    .await
    .expect("the trusted corpus limit should remain available to ListPages content");
    assert_eq!(
        output.html_output.body.matches(INCLUDE_MARKER).count(),
        384,
        "the corpus render should expand direct includes and both ListPages rows",
    );
}

#[tokio::test]
async fn corpus_render_supports_dense_includes_without_raising_public_limit() {
    const COMPONENT_SLUG: &str = "component:dense-include-cell";
    const PAGE_SLUG: &str = "fixture-dense-includes";
    const INCLUDE_COUNT: usize = 1_266;
    const MARKER: &str = "DENSE_INCLUDE_CELL";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        COMPONENT_SLUG,
        "Dense Include Cell",
        MARKER,
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        PAGE_SLUG,
        "Dense Includes",
        "placeholder",
    )
    .await;
    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
        }),
    )
    .expect("dense include fixture should exist");

    let wikitext = format!("[[include {COMPONENT_SLUG}]]\n").repeat(INCLUDE_COUNT);
    let page_info = PageInfo {
        page: Cow::Borrowed(PAGE_SLUG),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("Dense Includes"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let page_id = PageId {
        site_id,
        category_id: page.page_category_id,
        page_id: page.page_id,
    };

    let public_error = RenderService::render_page(
        runner.context(),
        wikitext.clone(),
        &page_info,
        Layout::Wikidot,
        page_id,
    )
    .await
    .expect_err("ordinary render must retain the public include ceiling");
    assert!(
        format!("{public_error:?}")
            .contains("include expansion exceeded maximum total includes 256")
    );

    let output = RenderService::render_corpus_page(
        runner.context(),
        wikitext,
        &page_info,
        Layout::Wikidot,
        page_id,
    )
    .await
    .expect("trusted corpus render should accept the observed dense include shape");

    assert_eq!(
        output.html_output.body.matches(MARKER).count(),
        INCLUDE_COUNT,
        "every corpus-provenanced include occurrence should render",
    );
}

#[tokio::test]
async fn page_tags_select_filters_latest_page_tags() {
    const DEFAULT_SLUG: &str = "xmlrpc-tags-default-source";
    const NAV_SLUG: &str = "xmlrpc-tags-nav-source";
    const MISSING_SLUG: &str = "xmlrpc-tags-missing-source";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let default_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        DEFAULT_SLUG,
        "XML-RPC Default Tag Source",
        "Default tag source",
    )
    .await;
    let default_revision = set_listpages_test_tags(
        &mut runner,
        site_id,
        DEFAULT_SLUG,
        default_revision,
        &["stale-default", "stale-shared"],
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        DEFAULT_SLUG,
        default_revision,
        &["xmlrpc-default", "shared-tag"],
    )
    .await;

    let nav_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        NAV_SLUG,
        "XML-RPC Nav Tag Source",
        "Nav tag source",
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, NAV_SLUG, "nav").await;
    let nav_revision = set_listpages_test_tags(
        &mut runner,
        site_id,
        NAV_SLUG,
        nav_revision,
        &["stale-nav", "stale-shared"],
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        NAV_SLUG,
        nav_revision,
        &["xmlrpc-nav", "shared-tag"],
    )
    .await;

    let nav_tags = run_endpoint!(
        runner,
        page_tags_select,
        json!({
            "site": "scp-wiki",
            "categories": ["nav"],
            "pages": [DEFAULT_SLUG, NAV_SLUG],
        }),
    );
    assert_eq!(
        nav_tags.into_iter().collect::<BTreeSet<_>>(),
        BTreeSet::from(["shared-tag".to_owned(), "xmlrpc-nav".to_owned()])
    );

    let page_tags = run_endpoint!(
        runner,
        page_tags_select,
        json!({
            "site": "scp-wiki",
            "pages": [DEFAULT_SLUG],
        }),
    );
    assert_eq!(
        page_tags.into_iter().collect::<BTreeSet<_>>(),
        BTreeSet::from(["shared-tag".to_owned(), "xmlrpc-default".to_owned()])
    );

    let empty_tags = run_endpoint!(
        runner,
        page_tags_select,
        json!({
            "site": "scp-wiki",
            "pages": [MISSING_SLUG],
        }),
    );
    assert!(empty_tags.is_empty());
}

#[tokio::test]
async fn page_select_filters_pages_with_page_query_semantics() {
    const TAG: &str = "xmlrpc-page-select-target";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (slug, title, category, vote) in [
        (
            "xmlrpc-page-select-high",
            "XML-RPC Page Select High",
            "_default",
            5,
        ),
        (
            "xmlrpc-page-select-zero",
            "XML-RPC Page Select Zero",
            "_default",
            0,
        ),
        (
            "xmlrpc-page-select-low",
            "XML-RPC Page Select Low",
            "_default",
            -2,
        ),
        (
            "xmlrpc-page-select-nav",
            "XML-RPC Page Select Nav",
            "nav",
            5,
        ),
    ] {
        set_mutation_request_context(
            &mut runner,
            ADMIN_USER_ID,
            site_id,
            Reference::Slug(Cow::Borrowed(slug)),
        );
        let output = run_endpoint!(
            runner,
            page_create,
            json!({
                "site_id": site_id,
                "wikitext": "XML-RPC page selection target.",
                "title": title,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "create XML-RPC page selection test page",
                "user_id": ADMIN_USER_ID,
                "ip_address": common::IP_ADDRESS,
            }),
        );
        if category != "_default" {
            set_listpages_test_category_slug(&runner, site_id, slug, category).await;
        }
        set_listpages_test_tags(&mut runner, site_id, slug, output.revision_id, &[TAG])
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

    let selected = run_endpoint!(
        runner,
        page_select,
        json!({
            "site": "scp-wiki",
            "pagetype": "normal",
            "categories": ["_default"],
            "tags_all": [TAG],
            "created_by": ADMIN_USER_ID.to_string(),
            "rating": ">=0",
            "order": "rating desc",
        }),
    );

    assert_eq!(
        selected,
        [
            "xmlrpc-page-select-high".to_owned(),
            "xmlrpc-page-select-zero".to_owned(),
        ],
        "pages.select should apply category, tag, creator, rating, and score ordering filters",
    );
}

#[tokio::test]
async fn page_select_treats_blank_optional_filters_as_absent() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let slug = "xmlrpc-page-select-blank-optionals";

    create_listpages_test_page(
        &mut runner,
        site_id,
        slug,
        "XML-RPC Page Select Blank Optionals",
        "XML-RPC blank optional filter target.",
    )
    .await;

    let selected = run_endpoint!(
        runner,
        page_select,
        json!({
            "site": "scp-wiki",
            "categories": ["_default"],
            "tags_all": [],
            "tags_none": [],
            "parent": "   ",
            "created_by": "",
            "rating": "",
            "order": "",
        }),
    );

    assert!(
        selected.iter().any(|selected_slug| selected_slug == slug),
        "blank optional pages.select filters should behave as absent instead of filtering out the page",
    );
}

#[tokio::test]
async fn page_select_rejects_non_finite_rating_filters() {
    let runner = TestRunner::setup().await;

    for rating in ["NaN", "inf", "-infinity"] {
        let error = run_endpoint_err!(
            runner,
            page_select,
            json!({
                "site": "scp-wiki",
                "rating": rating,
            }),
        );
        assert_contains_error!(error, ErrorType::Page);
    }
}

async fn set_listpages_test_category_slug(
    runner: &TestRunner,
    site_id: i64,
    slug: &str,
    category_slug: &str,
) {
    let category = PageCategoryTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page_category::Column::SiteId.eq(site_id))
                .add(page_category::Column::Slug.eq(category_slug)),
        )
        .one(runner.context().transaction())
        .await
        .expect("category test lookup should not fail")
        .expect("category test category should exist");
    let page = PageTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page::Column::SiteId.eq(site_id))
                .add(page::Column::Slug.eq(slug)),
        )
        .one(runner.context().transaction())
        .await
        .expect("category test page lookup should not fail")
        .expect("category test page should exist");
    let mut model = page.into_active_model();
    model.page_category_id = Set(category.category_id);
    model
        .update(runner.context().transaction())
        .await
        .expect("category test page update should not fail");
}

async fn set_listpages_test_tags(
    runner: &mut TestRunner,
    site_id: i64,
    slug: &str,
    last_revision_id: i64,
    tags: &[&str],
) -> i64 {
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
    output.revision_id
}

async fn set_listpages_test_parent(
    runner: &mut TestRunner,
    site_id: i64,
    slug: &str,
    parent: &str,
) {
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Owned(slug.to_owned()))),
    });

    run_endpoint!(
        runner,
        parent_update,
        json!({
            "site_id": site_id,
            "child": slug,
            "user_id": ADMIN_USER_ID,
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
    render_page_module_test_fixture_with_targets(
        runner,
        site_id,
        slug_prefix,
        tag,
        "ListPages",
        module_head,
        body,
        targets,
    )
    .await
}

async fn render_countpages_test_fixture_with_targets(
    runner: &mut TestRunner,
    site_id: i64,
    slug_prefix: &str,
    tag: &str,
    module_head: &str,
    body: &str,
    targets: &[(&str, &str, &str)],
) -> String {
    render_page_module_test_fixture_with_targets(
        runner,
        site_id,
        slug_prefix,
        tag,
        "CountPages",
        module_head,
        body,
        targets,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn render_page_module_test_fixture_with_targets(
    runner: &mut TestRunner,
    site_id: i64,
    slug_prefix: &str,
    tag: &str,
    module_name: &str,
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
        &format!("Fixture {module_name} Index"),
        &format!(
            "{module_name} start marker.\n\n[[module {module_name} {module_head}]]\n{body}\n[[/module]]\n\n{module_name} end marker."
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
    .expect("page module index should exist");

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
async fn listpages_perpage_renders_wikidot_pager_controls() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-list-pager";

    for index in 0..45 {
        let slug = format!("fixture-listpages-pager-target-{index:02}");
        let title = format!("Fixture ListPages Pager Target {index:02}");
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            &slug,
            &title,
            &format!("Fixture ListPages Pager Target {index:02} marker."),
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, &slug, revision, &[tag]).await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-listpages-pager-index",
        "Fixture ListPages Pager Index",
        &format!(
            "ListPages pager marker.\n\n[[module ListPages category=\"*\" tags=\"+{tag}\" perPage=\"20\" order=\"name\"]]\n* %%title%% :: %%slug%%\n[[/module]]"
        ),
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "fixture-listpages-pager-index",
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("ListPages pager index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    for expected in [
        "Fixture ListPages Pager Target 00",
        "fixture-listpages-pager-target-00",
        "Fixture ListPages Pager Target 19",
        "fixture-listpages-pager-target-19",
        r#"<div class="pager">"#,
        r#"<span class="current">1</span>"#,
        ">2</a>",
        ">3</a>",
        "next »",
    ] {
        assert!(
            html.contains(expected),
            "perPage ListPages fixture should contain {expected:?}:\n{html}",
        );
    }

    for forbidden in [
        "Fixture ListPages Pager Target 20",
        "fixture-listpages-pager-target-20",
        "[[module ListPages",
        "%%title%%",
    ] {
        assert!(
            !html.contains(forbidden),
            "perPage ListPages first page should not contain {forbidden:?}:\n{html}",
        );
    }
}

#[tokio::test]
async fn countpages_substitutes_total_for_tagged_pages() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let html = render_countpages_test_fixture_with_targets(
        &mut runner,
        site.site.site_id,
        "fixture-countpages-total",
        "verification-count-total",
        r#"category="*" tags="+verification-count-total" order="name" limit="20""#,
        "ORACLE_COUNT_SHARED=%%total%%",
        &[
            (
                "target-a",
                "Fixture CountPages Target Alpha",
                "Fixture CountPages Target Alpha marker.",
            ),
            (
                "target-b",
                "Fixture CountPages Target Beta",
                "Fixture CountPages Target Beta marker.",
            ),
            (
                "target-c",
                "Fixture CountPages Target Gamma",
                "Fixture CountPages Target Gamma marker.",
            ),
        ],
    )
    .await;

    assert!(
        html.contains("ORACLE_COUNT_SHARED=3"),
        "CountPages fixture should substitute %%total%% with the matching page count:\n{html}"
    );
    for forbidden in [
        "[[module CountPages",
        "%%total%%",
        "Fixture ListPages Excluded",
    ] {
        assert!(
            !html.contains(forbidden),
            "CountPages fixture should not contain {forbidden:?}:\n{html}"
        );
    }
}

#[tokio::test]
async fn countpages_category_filter_counts_matching_pages() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-count-category";
    let category_slug = "countpages-test-category";
    let index_slug = "fixture-countpages-category-index";
    CategoryService::get_or_create(runner.context(), site_id, category_slug)
        .await
        .expect("CountPages test category should be created");

    for (slug, category) in [
        ("fixture-countpages-category-fragment-a", category_slug),
        ("fixture-countpages-category-fragment-b", category_slug),
        ("fixture-countpages-category-default", "_default"),
    ] {
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            slug,
            "Fixture CountPages Category Target",
            "Fixture CountPages category marker.",
        )
        .await;
        set_listpages_test_category_slug(&runner, site_id, slug, category).await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision, &[tag]).await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        index_slug,
        "Fixture CountPages Category Index",
        &format!(
            "CountPages category marker.\n\n[[module CountPages category=\"{category_slug}\" tags=\"+{tag}\" order=\"name\" limit=\"20\"]]\nFRAGMENT_COUNT=%%total%%\n[[/module]]"
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
    .expect("CountPages category index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("FRAGMENT_COUNT=2"),
        "CountPages fixture should count only matching category pages:\n{html}"
    );
    for forbidden in ["[[module CountPages", "%%total%%"] {
        assert!(
            !html.contains(forbidden),
            "CountPages category fixture should not contain {forbidden:?}:\n{html}"
        );
    }
}

#[tokio::test]
async fn countpages_with_limit_defaults_to_current_category() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-count-default-category";
    let category_slug = "countpages-current-category-default";

    for slug in [
        format!("{category_slug}:target-a"),
        format!("{category_slug}:target-b"),
        "fixture-countpages-default-category-excluded".to_owned(),
    ] {
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            &slug,
            "Fixture CountPages Default Category Target",
            "Fixture CountPages default category marker.",
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, &slug, revision, &[tag]).await;
    }

    let index_slug = format!("{category_slug}:index");
    create_listpages_test_page(
        &mut runner,
        site_id,
        &index_slug,
        "Fixture CountPages Default Category Index",
        &format!(
            "CountPages default category marker.\n\n[[module CountPages tags=\"+{tag}\" order=\"name\" limit=\"20\"]]\nDEFAULT_CATEGORY_COUNT=%%total%%\n[[/module]]"
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
    .expect("CountPages default category index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("DEFAULT_CATEGORY_COUNT=2"),
        "limited CountPages without category should count current category only:\n{html}"
    );
    assert!(
        !html.contains("[[module CountPages") && !html.contains("%%total%%"),
        "CountPages default category fixture should render completely:\n{html}"
    );
}

#[tokio::test]
async fn countpages_without_limit_or_static_filter_remains_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let category_slug = "countpages-no-limit-literal";

    let index_slug = format!("{category_slug}:index");
    create_listpages_test_page(
        &mut runner,
        site_id,
        &index_slug,
        "Fixture CountPages No Limit Literal Index",
        "CountPages no-limit marker.\n\n[[module CountPages]]\nNO_LIMIT_COUNT=%%total%%\n[[/module]]",
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
    .expect("CountPages no-limit index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("NO_LIMIT_COUNT=%%total%%")
            || html.contains("[[module CountPages")
            || html.contains("module CountPages"),
        "CountPages without an explicit limit or static filter should remain literal/degraded:\n{html}"
    );
    assert!(
        !html.contains("NO_LIMIT_COUNT=1"),
        "CountPages without an explicit limit or static filter must not run an unbounded partial count:\n{html}"
    );
}

#[tokio::test]
async fn countpages_static_tag_filter_without_limit_substitutes_total() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let category_slug = "countpages-no-limit-static";
    let tag = "verification-count-no-limit-static";
    let target_slug = format!("{category_slug}:target");
    let revision = create_listpages_test_page(
        &mut runner,
        site_id,
        &target_slug,
        "Fixture CountPages No Limit Static Target",
        "Fixture CountPages no-limit static target marker.",
    )
    .await;
    set_listpages_test_tags(&mut runner, site_id, &target_slug, revision, &[tag]).await;

    let index_slug = format!("{category_slug}:index");
    create_listpages_test_page(
        &mut runner,
        site_id,
        &index_slug,
        "Fixture CountPages No Limit Static Index",
        &format!(
            "CountPages static no-limit marker.\n\n[[module CountPages tags=\"+{tag}\"]]\nSTATIC_NO_LIMIT_COUNT=%%total%%\n[[/module]]"
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
    .expect("CountPages static no-limit index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("STATIC_NO_LIMIT_COUNT=1"),
        "CountPages with a static tag filter and no explicit limit should substitute the bounded total:\n{html}"
    );
    assert!(
        !html.contains("[[module CountPages") && !html.contains("%%total%%"),
        "CountPages static-filter fixture should render completely:\n{html}"
    );
}

#[tokio::test]
async fn countpages_inside_code_block_remains_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-count-code-literal";
    let revision = create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-countpages-code-literal-target",
        "Fixture CountPages Code Literal Target",
        "Fixture CountPages code literal marker.",
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        "fixture-countpages-code-literal-target",
        revision,
        &[tag],
    )
    .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-countpages-code-literal-index",
        "Fixture CountPages Code Literal Index",
        &format!(
            "CountPages code literal marker.\n\n[[code]]\n[[module CountPages tags=\"+{tag}\" limit=\"20\"]]\nCODE_LITERAL_COUNT=%%total%%\n[[/module]]\n[[/code]]"
        ),
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "fixture-countpages-code-literal-index",
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("CountPages code literal index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("CODE_LITERAL_COUNT=%%total%%"),
        "CountPages inside code blocks should remain literal:\n{html}"
    );
    assert!(
        !html.contains("CODE_LITERAL_COUNT=1"),
        "CountPages inside code blocks must not substitute totals:\n{html}"
    );
}

#[tokio::test]
async fn countpages_unprefixed_tags_use_or_semantics() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag_a = "verification-count-any-alpha";
    let tag_b = "verification-count-any-beta";

    for (slug, tags) in [
        ("fixture-countpages-any-target-alpha", vec![tag_a]),
        ("fixture-countpages-any-target-beta", vec![tag_b]),
        ("fixture-countpages-any-target-both", vec![tag_a, tag_b]),
    ] {
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            slug,
            "Fixture CountPages Any Tag Target",
            "Fixture CountPages any tag marker.",
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision, &tags).await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-countpages-any-index",
        "Fixture CountPages Any Tag Index",
        &format!(
            "CountPages any tag marker.\n\n[[module CountPages tags=\"{tag_a} {tag_b}\" order=\"name\" limit=\"20\"]]\nANY_TAG_COUNT=%%total%%\n[[/module]]"
        ),
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "fixture-countpages-any-index",
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("CountPages any-tag index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("ANY_TAG_COUNT=3"),
        "unprefixed CountPages tags should match pages with any listed tag:\n{html}"
    );
    assert!(
        !html.contains("ANY_TAG_COUNT=1"),
        "unprefixed CountPages tags must not require every listed tag:\n{html}"
    );
}

#[tokio::test]
async fn countpages_artwork_hub_url_fallback_ignores_display_options() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tags = [
        "verification-count-artwork",
        "verification-count-artist",
        "verification-count-comic",
    ];

    for index in 0..25 {
        let tag = tags[index % tags.len()];
        let slug = format!("fixture-countpages-artwork-url-target-{index:02}");
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            &slug,
            "Fixture CountPages Artwork URL Target",
            "Fixture CountPages artwork URL marker.",
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, &slug, revision, &[tag]).await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-countpages-artwork-url-excluded",
        "Fixture CountPages Artwork URL Excluded",
        "Fixture CountPages artwork URL excluded marker.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-countpages-artwork-url-index",
        "Fixture CountPages Artwork URL Index",
        "CountPages artwork URL marker.\n\n[[module CountPages order=\"created_at desc\" wrapper=\"no\" category=\"*\" separate=\"false\" perPage=\"20\" tags=\"@URL|verification-count-artwork verification-count-artist verification-count-comic\"]]\nCurrently listing %%total%% pages.\n[[/module]]",
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "fixture-countpages-artwork-url-index",
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("CountPages artwork URL index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("Currently listing 25 pages."),
        "CountPages should use the @URL fallback tags and count all matching pages; perPage/wrapper/separate are display options for this module:\n{html}"
    );
    for forbidden in [
        "[[module CountPages",
        "%%total%%",
        "Currently listing 20 pages.",
        "Fixture CountPages Artwork URL Excluded",
    ] {
        assert!(
            !html.contains(forbidden),
            "CountPages artwork URL fixture should not contain {forbidden:?}:\n{html}"
        );
    }
}

#[tokio::test]
async fn countpages_unsupported_filters_remain_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let html = render_countpages_test_fixture_with_targets(
        &mut runner,
        site.site.site_id,
        "fixture-countpages-rating-literal",
        "verification-count-rating-literal",
        r#"tags="+verification-count-rating-literal" rating=">0" limit="20""#,
        "RATING_FILTER_COUNT=%%total%%",
        &[(
            "target-a",
            "Fixture CountPages Rating Literal Target",
            "Fixture CountPages rating literal marker.",
        )],
    )
    .await;

    assert!(
        html.contains("RATING_FILTER_COUNT=%%total%%")
            || html.contains("[[module CountPages")
            || html.contains("module CountPages"),
        "CountPages with unsupported filters should remain literal/degraded:\n{html}"
    );
    assert!(
        !html.contains("RATING_FILTER_COUNT=1"),
        "CountPages with unsupported filters must not substitute a partial count:\n{html}"
    );
}

#[tokio::test]
async fn countpages_dynamic_selectors_remain_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let html = render_countpages_test_fixture_with_targets(
        &mut runner,
        site.site.site_id,
        "fixture-countpages-dynamic-literal",
        "verification-count-dynamic-literal",
        r#"tags="@URL" limit="20""#,
        "DYNAMIC_SELECTOR_COUNT=%%total%%",
        &[(
            "target-a",
            "Fixture CountPages Dynamic Selector Target",
            "Fixture CountPages dynamic selector marker.",
        )],
    )
    .await;

    assert!(
        html.contains("DYNAMIC_SELECTOR_COUNT=%%total%%")
            || html.contains("[[module CountPages")
            || html.contains("module CountPages"),
        "CountPages with dynamic selectors should remain literal/degraded:\n{html}"
    );
    assert!(
        !html.contains("DYNAMIC_SELECTOR_COUNT=1"),
        "CountPages with dynamic selectors must not substitute a widened count:\n{html}"
    );
}

#[tokio::test]
async fn countpages_current_page_filters_remain_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let html = render_countpages_test_fixture_with_targets(
        &mut runner,
        site.site.site_id,
        "fixture-countpages-range-filter-literal",
        "verification-count-range-filter-literal",
        r#"range="." tags="+verification-count-range-filter-literal""#,
        "RANGE_FILTER_COUNT=%%total%%",
        &[(
            "target-a",
            "Fixture CountPages Range Filter Literal Target",
            "Fixture CountPages range filter literal marker.",
        )],
    )
    .await;

    assert!(
        html.contains("RANGE_FILTER_COUNT=%%total%%")
            || html.contains("[[module CountPages")
            || html.contains("module CountPages"),
        "CountPages range=. with additional filters should remain literal/degraded:\n{html}"
    );
    assert!(
        !html.contains("RANGE_FILTER_COUNT=1"),
        "CountPages range=. with filters must not ignore filters and count the current page:\n{html}"
    );
}

#[tokio::test]
async fn countpages_current_page_category_filters_remain_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let html = render_countpages_test_fixture_with_targets(
        &mut runner,
        site.site.site_id,
        "fixture-countpages-range-category-literal",
        "verification-count-range-category-literal",
        r#"range="." category="other-category""#,
        "RANGE_CATEGORY_COUNT=%%total%%",
        &[(
            "target-a",
            "Fixture CountPages Range Category Literal Target",
            "Fixture CountPages range category literal marker.",
        )],
    )
    .await;

    assert!(
        html.contains("RANGE_CATEGORY_COUNT=%%total%%")
            || html.contains("[[module CountPages")
            || html.contains("module CountPages"),
        "CountPages range=. with category filters should remain literal/degraded:\n{html}"
    );
    assert!(
        !html.contains("RANGE_CATEGORY_COUNT=1"),
        "CountPages range=. with category filters must not ignore filters and count the current page:\n{html}"
    );
}

#[tokio::test]
async fn countpages_broad_category_without_limit_remains_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let html = render_countpages_test_fixture_with_targets(
        &mut runner,
        site.site.site_id,
        "fixture-countpages-broad-category-literal",
        "verification-count-broad-category-literal",
        r#"category="*""#,
        "BROAD_CATEGORY_COUNT=%%total%%",
        &[(
            "target-a",
            "Fixture CountPages Broad Category Literal Target",
            "Fixture CountPages broad category literal marker.",
        )],
    )
    .await;

    assert!(
        html.contains("BROAD_CATEGORY_COUNT=%%total%%")
            || html.contains("[[module CountPages")
            || html.contains("module CountPages"),
        "CountPages category=* without a limit should remain literal/degraded:\n{html}"
    );
    assert!(
        !html.contains("BROAD_CATEGORY_COUNT=1"),
        "CountPages category=* without a limit must not materialize the whole site:\n{html}"
    );
}

#[tokio::test]
async fn countpages_current_author_uses_creation_revision_after_first_render() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-count-current-author-create";

    for slug in [
        "fixture-countpages-current-author-target-a",
        "fixture-countpages-current-author-target-b",
    ] {
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            slug,
            "Fixture CountPages Current Author Target",
            "Fixture CountPages current author target marker.",
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision, &[tag]).await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-countpages-current-author-index",
        "Fixture CountPages Current Author Index",
        &format!(
            "CountPages current author marker.\n\n[[module CountPages created_by=\"=\" tags=\"+{tag}\" limit=\"20\"]]\nCURRENT_AUTHOR_COUNT=%%total%%\n[[/module]]"
        ),
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "fixture-countpages-current-author-index",
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("CountPages current-author index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("CURRENT_AUTHOR_COUNT=2"),
        "created_by=\"=\" should use the creation revision once the first render is refreshed:\n{html}"
    );
    assert!(
        !html.contains("CURRENT_AUTHOR_COUNT=0"),
        "created_by=\"=\" must not keep the pre-revision no-match result after page creation:\n{html}"
    );
}

#[tokio::test]
async fn countpages_limit_above_scan_cap_remains_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let html = render_countpages_test_fixture_with_targets(
        &mut runner,
        site.site.site_id,
        "fixture-countpages-limit-cap-literal",
        "verification-count-limit-cap-literal",
        r#"category="*" tags="+verification-count-limit-cap-literal" limit="5001""#,
        "LIMIT_CAP_COUNT=%%total%%",
        &[(
            "target-a",
            "Fixture CountPages Limit Cap Target",
            "Fixture CountPages limit cap marker.",
        )],
    )
    .await;

    assert!(
        html.contains("LIMIT_CAP_COUNT=%%total%%")
            || html.contains("[[module CountPages")
            || html.contains("module CountPages"),
        "CountPages with an explicit limit above the scan cap should remain literal/degraded:\n{html}"
    );
    assert!(
        !html.contains("LIMIT_CAP_COUNT=1"),
        "CountPages must not silently substitute a partial count above the scan cap:\n{html}"
    );
}

#[tokio::test]
async fn countpages_current_page_tag_selectors_remain_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");

    for (slug_prefix, module_head, marker) in [
        (
            "fixture-countpages-current-tag-literal",
            r#"tags="=""#,
            "CURRENT_TAG_COUNT",
        ),
        (
            "fixture-countpages-current-tags-literal",
            r#"tags="+==""#,
            "CURRENT_TAGS_COUNT",
        ),
    ] {
        let html = render_countpages_test_fixture_with_targets(
            &mut runner,
            site.site.site_id,
            slug_prefix,
            "verification-count-current-tag-literal",
            module_head,
            &format!("{marker}=%%total%%"),
            &[(
                "target-a",
                "Fixture CountPages Current Tag Target",
                "Fixture CountPages current tag marker.",
            )],
        )
        .await;

        assert!(
            html.contains(&format!("{marker}=%%total%%"))
                || html.contains("[[module CountPages")
                || html.contains("module CountPages"),
            "CountPages current-page tag selector should remain literal/degraded:\n{html}"
        );
        assert!(
            !html.contains(&format!("{marker}=1")),
            "CountPages current-page tag selector must not substitute a guessed count:\n{html}"
        );
    }
}

#[tokio::test]
async fn countpages_no_tags_selector_remains_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let html = render_countpages_test_fixture_with_targets(
        &mut runner,
        site.site.site_id,
        "fixture-countpages-no-tags-literal",
        "verification-count-no-tags-literal",
        r#"tags="-" limit="20""#,
        "NO_TAGS_COUNT=%%total%%",
        &[(
            "target-a",
            "Fixture CountPages No Tags Target",
            "Fixture CountPages no-tags marker.",
        )],
    )
    .await;

    assert!(
        html.contains("NO_TAGS_COUNT=%%total%%")
            || html.contains("[[module CountPages")
            || html.contains("module CountPages"),
        "CountPages tags=\"-\" should remain literal/degraded:\n{html}"
    );
    assert!(
        !html.contains("NO_TAGS_COUNT=1"),
        "CountPages tags=\"-\" must not count tagged pages as no-tag pages:\n{html}"
    );
}

#[tokio::test]
async fn countpages_not_current_author_selector_remains_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let html = render_countpages_test_fixture_with_targets(
        &mut runner,
        site.site.site_id,
        "fixture-countpages-not-current-author-literal",
        "verification-count-not-current-author-literal",
        r#"created_by="-=""#,
        "NOT_CURRENT_AUTHOR_COUNT=%%total%%",
        &[(
            "target-a",
            "Fixture CountPages Not Current Author Target",
            "Fixture CountPages not current author marker.",
        )],
    )
    .await;

    assert!(
        html.contains("NOT_CURRENT_AUTHOR_COUNT=%%total%%")
            || html.contains("[[module CountPages")
            || html.contains("module CountPages"),
        "CountPages created_by=\"-=\" should remain literal/degraded:\n{html}"
    );
    assert!(
        !html.contains("NOT_CURRENT_AUTHOR_COUNT=1"),
        "CountPages created_by=\"-=\" must not substitute a guessed count:\n{html}"
    );
}

#[tokio::test]
async fn countpages_before_after_ranges_remain_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");

    for (slug_prefix, module_head, marker) in [
        (
            "fixture-countpages-before-range-literal",
            r#"range="before""#,
            "BEFORE_RANGE_COUNT",
        ),
        (
            "fixture-countpages-after-range-literal",
            r#"range="after""#,
            "AFTER_RANGE_COUNT",
        ),
    ] {
        let html = render_countpages_test_fixture_with_targets(
            &mut runner,
            site.site.site_id,
            slug_prefix,
            "verification-count-before-after-range-literal",
            module_head,
            &format!("{marker}=%%total%%"),
            &[(
                "target-a",
                "Fixture CountPages Before After Range Target",
                "Fixture CountPages before/after range marker.",
            )],
        )
        .await;

        assert!(
            html.contains(&format!("{marker}=%%total%%"))
                || html.contains("[[module CountPages")
                || html.contains("module CountPages"),
            "CountPages before/after range selector should remain literal/degraded:\n{html}"
        );
        assert!(
            !html.contains(&format!("{marker}=1")),
            "CountPages before/after range selector must not substitute a guessed count:\n{html}"
        );
    }
}

#[tokio::test]
async fn first_revision_rerenders_tag_dependent_countpages() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let slug = "fixture-countpages-first-revision-tag-rerender";
    let tag = "verification-count-first-revision-tag-rerender";

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(slug)),
    );
    let output = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": format!(
                "CountPages first-revision marker.\n\n[[module CountPages tags=\"+{tag}\" limit=\"20\"]]\nSELF_TAG_COUNT=%%total%%\n[[/module]]"
            ),
            "title": "Fixture CountPages First Revision Tag Rerender",
            "alt_title": null,
            "tags": [tag],
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "create first revision CountPages test page",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(output.slug, slug);

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": slug,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("first-revision CountPages page should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("SELF_TAG_COUNT=1"),
        "CountPages should be rerendered after the first revision is attached:\n{html}"
    );
    assert!(
        !html.contains("SELF_TAG_COUNT=0") && !html.contains("%%total%%"),
        "CountPages must not keep the pre-latest-revision result:\n{html}"
    );
}

#[tokio::test]
async fn first_revision_countpages_unsupported_filter_remains_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let slug = "fixture-countpages-first-revision-unsupported-literal";
    let tag = "verification-count-first-revision-unsupported-literal";

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(slug)),
    );
    let output = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": format!(
                "CountPages first-revision unsupported marker.\n\n[[module CountPages tags=\"+{tag}\" rating=\">0\" limit=\"20\"]]\nFIRST_REVISION_UNSUPPORTED_COUNT=%%total%%\n[[/module]]"
            ),
            "title": "Fixture CountPages First Revision Unsupported Literal",
            "alt_title": null,
            "tags": [tag],
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "create first revision unsupported CountPages test page",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(output.slug, slug);

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": slug,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("first-revision unsupported CountPages page should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("FIRST_REVISION_UNSUPPORTED_COUNT=%%total%%")
            || html.contains("[[module CountPages")
            || html.contains("module CountPages"),
        "unsupported CountPages filters should remain literal after first-revision rerender:\n{html}"
    );
    assert!(
        !html.contains("FIRST_REVISION_UNSUPPORTED_COUNT=1"),
        "unsupported CountPages filters must not substitute a partial first-revision count:\n{html}"
    );
}

#[tokio::test]
async fn first_revision_rerenders_included_countpages() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let component_slug = "component:fixture-first-revision-included-countpages";
    let page_slug = "fixture-first-revision-included-countpages";
    let tag = "verification-first-revision-included-countpages";

    create_listpages_test_page(
        &mut runner,
        site_id,
        component_slug,
        "Fixture First Revision Included CountPages Component",
        &format!(
            "[[module CountPages tags=\"+{tag}\" limit=\"20\"]]\nINCLUDED_COUNT=%%total%%\n[[/module]]"
        ),
    )
    .await;

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(page_slug)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": format!("[[include {component_slug}]]"),
            "title": "Fixture First Revision Included CountPages",
            "alt_title": null,
            "tags": [tag],
            "slug": page_slug,
            "layout": "wikidot",
            "revision_comments": "create first revision include CountPages fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": page_slug,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("first-revision included CountPages page should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("INCLUDED_COUNT=1"),
        "included CountPages should be rerendered after the first revision is attached:\n{html}"
    );
}

#[tokio::test]
async fn first_revision_rerenders_tagcloud() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let page_slug = "fixture-first-revision-tagcloud";
    let tag = "verification-first-revision-tagcloud";

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(page_slug)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "[[module TagCloud]]",
            "title": "Fixture First Revision TagCloud",
            "alt_title": null,
            "tags": [tag],
            "slug": page_slug,
            "layout": "wikidot",
            "revision_comments": "create first revision TagCloud fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": page_slug,
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("first-revision TagCloud page should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains(&format!(r#"/system:page-tags/tag/{tag}">"#))
            && html.contains(&format!(">{tag}<")),
        "TagCloud should be rerendered after the first revision is attached:\n{html}"
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
            &mut runner,
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
            candidate_limit: None,
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
        &mut runner,
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
            candidate_limit: None,
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
            candidate_limit: None,
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
        set_mutation_request_context(
            &mut runner,
            ADMIN_USER_ID,
            site_id,
            Reference::Slug(Cow::Borrowed(slug)),
        );
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
        candidate_limit: None,
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
async fn page_query_find_with_metadata_marks_sql_limited_results() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-page-query-metadata-sql";

    for slug in [
        "fixture-page-query-metadata-sql-a",
        "fixture-page-query-metadata-sql-b",
    ] {
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            slug,
            "Fixture PageQuery Metadata SQL",
            "Fixture PageQuery Metadata SQL marker.",
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision, &[tag]).await;
    }

    let all_tags = [Cow::Borrowed(tag)];
    let result = PageQueryService::find_with_metadata(
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
            candidate_limit: None,
            pagination: PaginationSelector {
                limit: Some(1),
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
    .expect("metadata query should not fail");

    assert_eq!(result.pages.total(), 1);
    assert_eq!(result.metadata.candidate_count, Some(1));
    assert!(result.metadata.sql_limit_offset_applied);
    assert!(!result.metadata.filtering_deferred_to_rust);
    assert!(!result.metadata.ordering_deferred_to_rust);
    assert!(!result.metadata.cap_exceeded);
    assert!(result.metadata.exact_count_safe);
}

#[tokio::test]
async fn page_query_find_with_metadata_marks_deferred_score_ordering() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-page-query-metadata-score";

    for slug in [
        "fixture-page-query-metadata-score-a",
        "fixture-page-query-metadata-score-b",
    ] {
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            slug,
            "Fixture PageQuery Metadata Score",
            "Fixture PageQuery Metadata Score marker.",
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision, &[tag]).await;
    }

    let all_tags = [Cow::Borrowed(tag)];
    let result = PageQueryService::find_with_metadata(
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
                property: OrderProperty::Score,
                ascending: true,
            }),
            candidate_limit: None,
            pagination: PaginationSelector {
                limit: Some(1),
                ..Default::default()
            },
            variables: &[],
            fields: FoundPageFields {
                slug: true,
                score: true,
                ..Default::default()
            },
        },
    )
    .await
    .expect("metadata query should not fail");

    assert_eq!(result.pages.total(), 1);
    assert_eq!(result.metadata.candidate_count, Some(2));
    assert!(!result.metadata.sql_limit_offset_applied);
    assert!(!result.metadata.filtering_deferred_to_rust);
    assert!(result.metadata.ordering_deferred_to_rust);
    assert!(!result.metadata.cap_exceeded);
    assert!(!result.metadata.exact_count_safe);
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
