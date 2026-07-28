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
use deepwell::license::License;
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
    AuthorSelector, CategoriesSelector, ComparisonOperation, DataFormSelector,
    DateSelector, FoundPageFields, IncludedCategories, OrderBySelector, OrderProperty,
    PageParentSelector, PageQuery, PageQueryService, PageTypeSelector,
    PaginationSelector, RangeSelector, ScoreSelector, TagCondition,
};
use deepwell::services::permission::{
    CheckPermissionContext, PermissionCache, PermissionService,
};
use deepwell::services::render::UrlArguments;
use deepwell::services::role::{
    GrantUserRoleInput, InternalCreateRoleInput, RoleService, UpdateRolePermissionsInput,
};
use deepwell::services::score::ScoreValue as QueryScoreValue;
use deepwell::services::session::CreateSession;
use deepwell::services::site::UpdateSiteBody;
use deepwell::services::view::{GetArticleViewOutput, GetPageViewOutput};
use deepwell::services::{
    FileRevisionService, ForumPostService, ForumService, ForumThreadService, LinkService,
    PageService, RenderService, RequestContext, SessionService, SettingsService,
    SiteService, TextService,
};
use deepwell::types::{
    Action, ConnectionType, Maybe, PageId, PageRevisionType, Permission, Reference,
    Resource, TextBlockType,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel,
    QueryFilter, Set, Statement, Value,
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

async fn set_stored_point_vote(runner: &TestRunner, page_id: i64, value: i16) {
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "INSERT INTO page_vote (from_wikidot, page_id, user_id, value) VALUES (false, $1, $2, $3)",
            [
                Value::from(page_id),
                Value::from(ADMIN_USER_ID),
                Value::from(value),
            ],
        ))
        .await
        .expect("score fixture should receive its stored point vote");
}

async fn create_imported_breadcrumb_page(
    runner: &mut TestRunner,
    site_id: i64,
    category_id: i64,
    slug: &str,
    title: &str,
) -> i64 {
    set_mutation_request_context(
        runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Owned(slug.to_owned())),
    );
    let created = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "Imported breadcrumb fixture",
            "title": title,
            "alt_title": null,
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "create imported breadcrumb fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let page = PageTable::find_by_id(created.page_id)
        .one(runner.context().transaction())
        .await
        .expect("breadcrumb page lookup should not fail")
        .expect("created breadcrumb page should exist");
    let mut page = page.into_active_model();
    page.page_category_id = Set(category_id);
    page.from_wikidot = Set(true);
    page.update(runner.context().transaction())
        .await
        .expect("breadcrumb page should be marked as imported");

    created.page_id
}

async fn imported_breadcrumb_article_view(
    runner: &mut TestRunner,
    site_id: i64,
    slug: &str,
    session_token: Option<&str>,
) -> GetArticleViewOutput {
    run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": {"slug": slug, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    )
}

#[tokio::test]
async fn imported_page_layout_provenance_preserves_explicit_page_override() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let category = CategoryService::get(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed("_default")),
    )
    .await
    .expect("seeded default category should exist");
    let page_id = create_imported_breadcrumb_page(
        &mut runner,
        site_id,
        category.category_id,
        "imported-layout-override",
        "Imported Layout Override",
    )
    .await;

    let page = PageTable::find_by_id(page_id)
        .one(runner.context().transaction())
        .await
        .expect("imported page lookup should not fail")
        .expect("imported page should exist");
    let mut page = page.into_active_model();
    page.layout = Set(Some("wikijump".to_owned()));
    page.update(runner.context().transaction())
        .await
        .expect("explicit imported-page layout override should update");

    assert_eq!(
        SettingsService::get_layout(runner.context(), site_id, Some(page_id))
            .await
            .expect("effective layout lookup should succeed"),
        Layout::Wikijump,
    );
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
async fn article_view_uses_category_license_and_site_fallback() {
    const SITE_SLUG: &str = "test";
    const PAGE_SLUG: &str = "category-license:article";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("seeded site should exist")
        .site;
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Slug(Cow::Borrowed(PAGE_SLUG)),
    );
    let created = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site_id,
            "wikitext": "Category license fixture",
            "title": "Category license fixture",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create category license fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let page = PageTable::find_by_id(created.page_id)
        .one(runner.context().transaction())
        .await
        .expect("page lookup should not fail")
        .expect("created page should exist");
    let category = PageCategoryTable::find_by_id(page.page_category_id)
        .one(runner.context().transaction())
        .await
        .expect("category lookup should not fail")
        .expect("created category should exist");
    let mut category = category.into_active_model();
    category.license = Set(Some(License::CcBy30.to_string()));
    category
        .update(runner.context().transaction())
        .await
        .expect("category license update should succeed");

    let explicit = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site.site_id,
            "session_token": null,
            "route": {"slug": PAGE_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    assert_eq!(explicit.viewer.license_url, License::CcBy30.url());
    assert_eq!(
        explicit.viewer.license_kind,
        deepwell::services::view::ViewerLicenseKind::Standard,
    );

    let category = PageCategoryTable::find_by_id(page.page_category_id)
        .one(runner.context().transaction())
        .await
        .expect("category lookup should not fail")
        .expect("created category should exist");
    let mut category = category.into_active_model();
    category.license = Set(Some(String::from("other")));
    category.license_other =
        Set(Some(String::from("Codex %%year%% <strong>Strong</strong>")));
    category
        .update(runner.context().transaction())
        .await
        .expect("custom category license update should succeed");

    let custom = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site.site_id,
            "session_token": null,
            "route": {"slug": PAGE_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    assert_eq!(
        custom.viewer.license_kind,
        deepwell::services::view::ViewerLicenseKind::Other,
    );
    let custom_html = custom.viewer.license_html.unwrap();
    assert!(custom_html.starts_with("Codex 20"));
    assert!(custom_html.ends_with(" <strong>Strong</strong>"));

    let category = PageCategoryTable::find_by_id(page.page_category_id)
        .one(runner.context().transaction())
        .await
        .expect("category lookup should not fail")
        .expect("created category should exist");
    let mut category = category.into_active_model();
    category.license = Set(None);
    category.license_other = Set(None);
    category
        .update(runner.context().transaction())
        .await
        .expect("category inheritance update should succeed");

    let inherited = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site.site_id,
            "session_token": null,
            "route": {"slug": PAGE_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    assert_eq!(inherited.viewer.license_url, site.license.url());
}

#[tokio::test]
async fn article_view_uses_effective_page_discussion_policy_and_stored_nesting() {
    const SITE_SLUG: &str = "test";
    const PAGE_SLUG: &str = "forum-policy:article";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("seeded site should exist")
        .site;
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Slug(Cow::Borrowed(PAGE_SLUG)),
    );
    let created = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site_id,
            "wikitext": "Page discussion policy fixture",
            "title": "Page discussion policy fixture",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create discussion policy fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let page = PageTable::find_by_id(created.page_id)
        .one(runner.context().transaction())
        .await
        .expect("page lookup should not fail")
        .expect("created page should exist");

    SiteService::update(
        runner.context(),
        Reference::Id(site.site_id),
        UpdateSiteBody {
            forum_max_nest_level: Maybe::Set(3),
            ..Default::default()
        },
        ADMIN_USER_ID,
        common::IP_ADDRESS,
    )
    .await
    .expect("forum nesting update should succeed");
    assert_eq!(
        SettingsService::get_forum_settings(runner.context(), site.site_id, None)
            .await
            .expect("stored forum settings should resolve")
            .max_nest_level,
        3
    );
    let invalid_nesting = SiteService::update(
        runner.context(),
        Reference::Id(site.site_id),
        UpdateSiteBody {
            forum_max_nest_level: Maybe::Set(11),
            ..Default::default()
        },
        ADMIN_USER_ID,
        common::IP_ADDRESS,
    )
    .await
    .expect_err("forum nesting above ten must fail closed");
    assert_contains_error!(invalid_nesting, ErrorType::BadRequest);

    let default_category = CategoryService::get(
        runner.context(),
        site.site_id,
        Reference::Slug(Cow::Borrowed("_default")),
    )
    .await
    .expect("default category should exist");
    let mut default_category = default_category.into_active_model();
    default_category.per_page_discussion = Set(Some(true));
    default_category
        .update(runner.context().transaction())
        .await
        .expect("default discussion policy should update");

    let inherited = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site.site_id,
            "session_token": null,
            "route": {"slug": PAGE_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    assert!(matches!(
        inherited.page,
        GetPageViewOutput::Found {
            page_discussion,
            ..
        } if page_discussion.enabled
    ));

    let category = PageCategoryTable::find_by_id(page.page_category_id)
        .one(runner.context().transaction())
        .await
        .expect("category lookup should not fail")
        .expect("page category should exist");
    let mut category = category.into_active_model();
    category.per_page_discussion = Set(Some(false));
    category
        .update(runner.context().transaction())
        .await
        .expect("category discussion override should update");

    let disabled = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site.site_id,
            "session_token": null,
            "route": {"slug": PAGE_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    assert!(matches!(
        disabled.page,
        GetPageViewOutput::Found {
            page_discussion,
            ..
        } if !page_discussion.enabled
    ));
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
    let compiled_at_before = home.compiled_at;
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
    let rerendered_home = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "home",
        }),
    )
    .expect("rerendered home page should exist");
    assert!(rerendered_home.compiled_at > compiled_at_before);
    assert!(
        rerendered_home
            .compiled_generator
            .ends_with("; deepwell-render/v1")
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
async fn article_cache_and_include_dependencies_use_exact_template_source() {
    const TEMPLATE_SLUG: &str = "article-cache-template-dependency:_template";
    const TEMPLATE_ARTICLE_SLUG: &str = "article-cache-template-dependency:templated";
    const DIRECT_ARTICLE_SLUG: &str = "article-cache-direct-dependency:direct";
    const COMPOSED_TEMPLATE_SLUG: &str = "article-cache-composed-dependency:_template";
    const COMPOSED_ARTICLE_SLUG: &str = "article-cache-composed-dependency:templated";
    const INCLUDE_TEMPLATE_SLUG: &str = "article-cache-template-include:_template";
    const INCLUDE_ARTICLE_SLUG: &str = "article-cache-template-include:templated";
    const INCLUDE_SLUG: &str = "component:cache-template-dependency";
    const REQUEST_DEPENDENT_LIST_PAGES: &str =
        "[[module ListPages offset=\"@URL|1\"]]%%title_linked%%[[/module]]";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        TEMPLATE_SLUG,
        "Request-dependent exact template",
        &format!("{REQUEST_DEPENDENT_LIST_PAGES}\n%%content%%"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        TEMPLATE_ARTICLE_SLUG,
        "Template dependency article",
        "cache-safe stored page source",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        DIRECT_ARTICLE_SLUG,
        "Direct dependency article",
        REQUEST_DEPENDENT_LIST_PAGES,
    )
    .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        COMPOSED_TEMPLATE_SLUG,
        "Request dependency split across template composition",
        "[[module ListPages offset=\"@U%%content%%\"]]%%title_linked%%[[/module]]",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        COMPOSED_ARTICLE_SLUG,
        "Composed request dependency article",
        "RL|1",
    )
    .await;

    for slug in [
        TEMPLATE_ARTICLE_SLUG,
        DIRECT_ARTICLE_SLUG,
        COMPOSED_ARTICLE_SLUG,
    ] {
        let page = PageTable::find()
            .filter(
                sea_orm::Condition::all()
                    .add(page::Column::SiteId.eq(site_id))
                    .add(page::Column::Slug.eq(slug)),
            )
            .one(runner.context().transaction())
            .await
            .expect("cache dependency page lookup should not fail")
            .expect("cache dependency page should exist");
        let mut page = page.into_active_model();
        page.from_wikidot = Set(true);
        page.update(runner.context().transaction())
            .await
            .expect("cache dependency page should be marked imported");

        let metadata = run_endpoint!(
            runner,
            article_view_cache_metadata,
            json!({
                "site_id": site_id,
                "session_token": null,
                "route": {"slug": slug, "extra": ""},
                "locales": ["en-US", "en"],
            }),
        );
        assert_eq!(
            metadata.article_page_cache_key, None,
            "request-dependent ListPages must deny anonymous caching when authored in {slug}",
        );
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        INCLUDE_SLUG,
        "Template include dependency",
        "template include dependency body",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INCLUDE_TEMPLATE_SLUG,
        "Include exact template",
        &format!("[[include {INCLUDE_SLUG}]]\n%%content%%"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INCLUDE_ARTICLE_SLUG,
        "Template include article",
        "article body without an authored include",
    )
    .await;

    let include = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": INCLUDE_SLUG}),
    )
    .expect("template include dependency should exist");
    let article = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": INCLUDE_ARTICLE_SLUG}),
    )
    .expect("template include article should exist");
    let connections = LinkService::get_connections_from(
        runner.context(),
        article.page_id,
        Some(&[ConnectionType::IncludeMessy]),
    )
    .await
    .expect("template include article connections should load");
    assert!(
        connections
            .present
            .iter()
            .any(|connection| connection.to_page_id == include.page_id),
        "a template-only include must record the article-to-include dependency used by include outdating",
    );
}

#[tokio::test]
async fn imported_breadcrumbs_hide_private_and_deleted_ancestors() {
    const IMPORT_RUN_ID: i64 = 7_700_398;
    const PUBLIC_PARENT_SLUG: &str = "breadcrumb-public:visible-parent";
    const PUBLIC_CHILD_SLUG: &str = "breadcrumb-public:deleted-parent-child";
    const PRIVATE_PARENT_SLUG: &str = "breadcrumb-private:secret-parent";
    const PRIVATE_CHILD_SLUG: &str = "breadcrumb-public:private-parent-child";
    const PRIVATE_ROOT_SLUG: &str = "breadcrumb-public:visible-root";

    let mut runner = TestRunner::setup().await;
    let (site_id, public_parent_id, guest_role_id, private_category_id) =
        Box::pin(async {
            let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
                .expect("seeded test site should exist");
            let site_id = site.site.site_id;
            let public_category = CategoryService::get_or_create(
                runner.context(),
                site_id,
                "breadcrumb-public",
            )
            .await
            .expect("public breadcrumb category should be created");
            let private_category = CategoryService::get_or_create(
                runner.context(),
                site_id,
                "breadcrumb-private",
            )
            .await
            .expect("private breadcrumb category should be created");
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

            for (role_id, category_id) in [
                (root_role.role_id, public_category.category_id),
                (guest_role.role_id, public_category.category_id),
                (root_role.role_id, private_category.category_id),
                (guest_role.role_id, private_category.category_id),
            ] {
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
                .expect("breadcrumb view permission should be inserted");
            }
            PermissionCache::invalidate_site(runner.context(), site_id)
                .await
                .expect("breadcrumb permission cache should be invalidated");
            RoleService::grant_role_to_user(
                runner.context(),
                GrantUserRoleInput {
                    site_id,
                    user_id: ADMIN_USER_ID,
                    role_id: root_role.role_id,
                    assigning_user_id: SYSTEM_USER_ID,
                    expires_at: None,
                    ip_address: common::IP_ADDRESS,
                },
            )
            .await
            .expect("authenticated breadcrumb viewer should receive the root role");

            let public_parent_id = create_imported_breadcrumb_page(
                &mut runner,
                site_id,
                public_category.category_id,
                PUBLIC_PARENT_SLUG,
                "Visible Parent",
            )
            .await;
            let public_child_id = create_imported_breadcrumb_page(
                &mut runner,
                site_id,
                public_category.category_id,
                PUBLIC_CHILD_SLUG,
                "Public Child",
            )
            .await;
            let private_parent_id = create_imported_breadcrumb_page(
                &mut runner,
                site_id,
                private_category.category_id,
                PRIVATE_PARENT_SLUG,
                "Private Parent Secret",
            )
            .await;
            let private_child_id = create_imported_breadcrumb_page(
                &mut runner,
                site_id,
                public_category.category_id,
                PRIVATE_CHILD_SLUG,
                "Private Parent Child",
            )
            .await;
            let private_root_id = create_imported_breadcrumb_page(
                &mut runner,
                site_id,
                public_category.category_id,
                PRIVATE_ROOT_SLUG,
                "Visible Root Above Private Parent",
            )
            .await;

            let transaction = runner.context().transaction();
            for sql in [
                format!(
                    r#"
INSERT INTO wikidot_corpus_import_run (
    import_run_id, site_id, source_branch, source_site, manifest_sha256,
    manifest_row_count, complete_inventory, state, summary
) VALUES (
    {IMPORT_RUN_ID}, {site_id}, 'test', 'test',
    decode(repeat('00', 32), 'hex'), 5, false, 'metadata_done', '{{}}'::jsonb
)
"#,
                ),
                format!(
                    r#"
INSERT INTO wikidot_page_snapshot (
    page_id, source_branch, source_site, source_entity_id, source_fullname,
    source_created_at, source_updated_at, source_revision_count,
    imported_rating, title_shown, parent_fullname, comments, source_sha256,
    meta_sha256, meta_json, last_import_run_id
) VALUES
    ({public_parent_id}, 'test', 'test',
     '39800000-0000-4000-8000-000000000001', '{PUBLIC_PARENT_SLUG}',
     NOW(), NOW(), 1, 0, 'Visible Parent', NULL, 0,
     decode(repeat('01', 32), 'hex'), decode(repeat('11', 32), 'hex'),
     '{{}}'::jsonb, {IMPORT_RUN_ID}),
    ({public_child_id}, 'test', 'test',
     '39800000-0000-4000-8000-000000000002', '{PUBLIC_CHILD_SLUG}',
     NOW(), NOW(), 1, 0, 'Public Child', '{PUBLIC_PARENT_SLUG}', 0,
     decode(repeat('02', 32), 'hex'), decode(repeat('12', 32), 'hex'),
     '{{}}'::jsonb, {IMPORT_RUN_ID}),
    ({private_parent_id}, 'test', 'test',
     '39800000-0000-4000-8000-000000000003', '{PRIVATE_PARENT_SLUG}',
     NOW(), NOW(), 1, 0, 'Private Parent Secret', '{PRIVATE_ROOT_SLUG}', 0,
     decode(repeat('03', 32), 'hex'), decode(repeat('13', 32), 'hex'),
     '{{}}'::jsonb, {IMPORT_RUN_ID}),
    ({private_child_id}, 'test', 'test',
     '39800000-0000-4000-8000-000000000004', '{PRIVATE_CHILD_SLUG}',
     NOW(), NOW(), 1, 0, 'Private Parent Child', '{PRIVATE_PARENT_SLUG}', 0,
     decode(repeat('04', 32), 'hex'), decode(repeat('14', 32), 'hex'),
     '{{}}'::jsonb, {IMPORT_RUN_ID}),
    ({private_root_id}, 'test', 'test',
     '39800000-0000-4000-8000-000000000005', '{PRIVATE_ROOT_SLUG}',
     NOW(), NOW(), 1, 0, 'Visible Root Above Private Parent', NULL, 0,
     decode(repeat('05', 32), 'hex'), decode(repeat('15', 32), 'hex'),
     '{{}}'::jsonb, {IMPORT_RUN_ID})
"#,
                ),
            ] {
                transaction
                    .execute_raw(Statement::from_string(
                        transaction.get_database_backend(),
                        sql,
                    ))
                    .await
                    .expect("breadcrumb snapshot fixture SQL should succeed");
            }

            (
                site_id,
                public_parent_id,
                guest_role.role_id,
                private_category.category_id,
            )
        })
        .await;

    Box::pin(async {
        runner.set_request_context(RequestContext::default());
        let visible_view = imported_breadcrumb_article_view(
            &mut runner,
            site_id,
            PUBLIC_CHILD_SLUG,
            None,
        )
        .await;
        let (visible_breadcrumbs, visible_cache_key) = match visible_view {
            GetArticleViewOutput {
                page:
                    GetPageViewOutput::Found {
                        wikidot_breadcrumbs,
                        ..
                    },
                article_page_cache_key: Some(cache_key),
                ..
            } => (wikidot_breadcrumbs, cache_key),
            other => panic!("expected cached public imported page, got {other:?}"),
        };
        assert_eq!(visible_breadcrumbs.len(), 2);
        assert_eq!(visible_breadcrumbs[0].slug, PUBLIC_PARENT_SLUG);
        assert_eq!(visible_breadcrumbs[0].title, "Visible Parent");

        let public_parent = PageTable::find_by_id(public_parent_id)
            .one(runner.context().transaction())
            .await
            .expect("public parent lookup should not fail")
            .expect("public parent should exist");
        let mut public_parent = public_parent.into_active_model();
        public_parent.deleted_at = Set(Some(OffsetDateTime::now_utc()));
        public_parent
            .update(runner.context().transaction())
            .await
            .expect("public parent should be soft-deleted");

        let deleted_parent_view = imported_breadcrumb_article_view(
            &mut runner,
            site_id,
            PUBLIC_CHILD_SLUG,
            None,
        )
        .await;
        let (deleted_parent_breadcrumbs, deleted_parent_cache_key) =
            match deleted_parent_view {
                GetArticleViewOutput {
                    page:
                        GetPageViewOutput::Found {
                            wikidot_breadcrumbs,
                            ..
                        },
                    article_page_cache_key: Some(cache_key),
                    ..
                } => (wikidot_breadcrumbs, cache_key),
                other => panic!("expected cached child of deleted parent, got {other:?}"),
            };
        assert_eq!(
            deleted_parent_cache_key, visible_cache_key,
            "ancestor deletion should exercise the existing cached article response"
        );
        assert!(
            deleted_parent_breadcrumbs.is_empty(),
            "deleted ancestor metadata must not be returned"
        );
    })
    .await;

    Box::pin(async {
        let private_parent_view = imported_breadcrumb_article_view(
            &mut runner,
            site_id,
            PRIVATE_CHILD_SLUG,
            None,
        )
        .await;
        let (private_parent_breadcrumbs, private_cache_key) = match private_parent_view {
            GetArticleViewOutput {
                page:
                    GetPageViewOutput::Found {
                        wikidot_breadcrumbs,
                        ..
                    },
                article_page_cache_key: Some(cache_key),
                ..
            } => (wikidot_breadcrumbs, cache_key),
            other => panic!("expected cached child of private parent, got {other:?}"),
        };
        assert_eq!(
            private_parent_breadcrumbs
                .iter()
                .map(|breadcrumb| breadcrumb.slug.as_str())
                .collect::<Vec<_>>(),
            [PRIVATE_ROOT_SLUG, PRIVATE_PARENT_SLUG, PRIVATE_CHILD_SLUG],
        );

        let admin_session_token = SessionService::create(
            runner.context(),
            CreateSession {
                user_id: ADMIN_USER_ID,
                ip_address: common::IP_ADDRESS,
                user_agent: "breadcrumb privacy test".to_owned(),
                restricted: false,
            },
        )
        .await
        .expect("admin session should be created");
        let authenticated_before = imported_breadcrumb_article_view(
            &mut runner,
            site_id,
            PRIVATE_CHILD_SLUG,
            Some(&admin_session_token),
        )
        .await;
        assert!(matches!(
            authenticated_before,
            GetArticleViewOutput {
                page: GetPageViewOutput::Found { ref wikidot_breadcrumbs, .. },
                ..
            } if wikidot_breadcrumbs.len() == 3
        ));

        RolePermissionTable::delete_many()
            .filter(role_permission::Column::RoleId.eq(guest_role_id))
            .filter(role_permission::Column::SiteId.eq(site_id))
            .filter(role_permission::Column::ResourceType.eq(Resource::Page))
            .filter(role_permission::Column::ResourceCategoryId.eq(private_category_id))
            .filter(role_permission::Column::Action.eq(Action::View))
            .exec(runner.context().transaction())
            .await
            .expect("guest private breadcrumb permission should be revoked");
        PermissionCache::invalidate_site(runner.context(), site_id)
            .await
            .expect("breadcrumb permission cache should be invalidated after revocation");

        let anonymous_after = imported_breadcrumb_article_view(
            &mut runner,
            site_id,
            PRIVATE_CHILD_SLUG,
            None,
        )
        .await;
        match anonymous_after {
            GetArticleViewOutput {
                page:
                    GetPageViewOutput::Found {
                        wikidot_breadcrumbs,
                        ..
                    },
                article_page_cache_key: Some(cache_key),
                ..
            } => {
                assert!(wikidot_breadcrumbs.is_empty());
                assert!(
                    !wikidot_breadcrumbs
                        .iter()
                        .any(|item| item.slug == PRIVATE_ROOT_SLUG)
                );
                assert_ne!(cache_key, private_cache_key);
            }
            other => {
                panic!("expected anonymous cached child after revocation, got {other:?}")
            }
        }

        let authenticated_after = imported_breadcrumb_article_view(
            &mut runner,
            site_id,
            PRIVATE_CHILD_SLUG,
            Some(&admin_session_token),
        )
        .await;
        assert!(matches!(
            authenticated_after,
            GetArticleViewOutput {
                page: GetPageViewOutput::Found { ref wikidot_breadcrumbs, .. },
                ..
            } if wikidot_breadcrumbs.len() == 3
        ));
    })
    .await;
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
    let styles = page
        .compiled_body_styles
        .expect("compiled styles should be included in page_get details")
        .join("\n");

    assert!(
        styles.contains("theme%3Abasalt/3"),
        "compiled page should include CSS from the local theme dependency: {styles}"
    );
    assert!(
        html.contains("Include consumer body marker."),
        "compiled page should retain the consumer page body"
    );
    assert!(
        !html.contains("margin-top: -12rem !important")
            && !html.contains("#top-bar ul ul")
            && !html.contains("left: -272px !important"),
        "compiled Basalt page must not override the provenance-backed theme shell: {html}"
    );
}

#[tokio::test]
async fn included_iftags_closer_survives_unmatched_inline_raw_on_an_earlier_line() {
    const COMPONENT_SLUG: &str = "component:fixture-iftags-unmatched-inline-raw";
    const CONSUMER_SLUG: &str = "fixture-iftags-unmatched-inline-raw-consumer";
    const PREVIEW_MARKER: &str = "Fixture preview payload marker";
    const DOCUMENTATION_MARKER: &str = "Fixture component documentation must not leak";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let component_wikitext = [
        "[[div class=\"preview\"]]\n",
        "{$text}\n",
        "[[/div]]\n",
        "[[iftags +component]]\n",
        DOCUMENTATION_MARKER,
        "\n* Escaping with @@\n",
        "[[/iftags]]\n",
    ]
    .concat();

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site.site_id,
        Reference::Slug(Cow::Borrowed(COMPONENT_SLUG)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site.site_id,
            "wikitext": component_wikitext,
            "title": "Conditional Include Fixture",
            "alt_title": null,
            "slug": COMPONENT_SLUG,
            "layout": "wikidot",
            "revision_comments": "create conditional include fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site.site_id,
        Reference::Slug(Cow::Borrowed(CONSUMER_SLUG)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site.site_id,
            "wikitext": format!("[[include {COMPONENT_SLUG} | text={PREVIEW_MARKER}]]\n"),
            "title": "Conditional Include Consumer",
            "alt_title": null,
            "slug": CONSUMER_SLUG,
            "layout": "wikidot",
            "revision_comments": "create conditional include consumer",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site.site.site_id,
            "page": CONSUMER_SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("conditional include consumer should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains(PREVIEW_MARKER),
        "preview payload should remain: {html}"
    );
    assert!(
        !html.contains(DOCUMENTATION_MARKER)
            && !html.contains("[[iftags")
            && !html.contains("[[/iftags]]"),
        "inactive component documentation and its boundaries must be absent: {html}",
    );
}

#[tokio::test]
async fn unbound_include_variables_remain_literal_in_attributes_and_text() {
    const COMPONENT_SLUG: &str = "component:fixture-unbound-include-variable";
    const CONSUMER_SLUG: &str = "fixture-unbound-include-variable-consumer";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site.site_id,
        Reference::Slug(Cow::Borrowed(COMPONENT_SLUG)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site.site_id,
            "wikitext": concat!(
                "[[div_ class=\"fixture {$missing}\"]]\n",
                "[[div_ class=\"label\"]]\n",
                "{$missing}\n",
                "[[/div]]\n",
                "[[/div]]",
            ),
            "title": "Unbound Include Variable",
            "alt_title": null,
            "slug": COMPONENT_SLUG,
            "layout": "wikidot",
            "revision_comments": "create unbound include variable fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site.site_id,
        Reference::Slug(Cow::Borrowed(CONSUMER_SLUG)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site.site_id,
            "wikitext": format!("[[include {COMPONENT_SLUG}]]"),
            "title": "Unbound Include Variable Consumer",
            "alt_title": null,
            "slug": CONSUMER_SLUG,
            "layout": "wikidot",
            "revision_comments": "create unbound include variable consumer",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site.site.site_id,
            "page": CONSUMER_SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("unbound include variable consumer should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("class=\"fixture {$missing}\""),
        "unbound class variable should remain literal: {html}",
    );
    assert!(
        html.contains("<div class=\"label\">{$missing}</div>"),
        "unbound text variable should remain literal: {html}",
    );
}

#[tokio::test]
async fn nested_include_image_blocks_keep_their_attachment_page_owner() {
    const SITE_SLUG: &str = "scp-wiki";
    const FRAGMENT_SLUG: &str = "fragment:attachment-owner-leaf";
    const SECOND_FRAGMENT_SLUG: &str = "fragment:attachment-owner-second-leaf";
    const WRAPPER_SLUG: &str = "component:attachment-owner-wrapper";
    const BASE_SLUG: &str = "component:attachment-owner-base";
    const CROSS_FRAGMENT_SLUG: &str = "fragment:attachment-owner-cross-leaf";
    const CROSS_WRAPPER_SLUG: &str = "component:attachment-owner-cross-wrapper";
    const CROSS_BASE_SLUG: &str = "component:attachment-owner-cross-base";
    const CONSUMER_SLUG: &str = "fixture-attachment-owner-consumer";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let cross_site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded cross-site fixture site should exist");
    let cross_site_id = cross_site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        FRAGMENT_SLUG,
        "Attachment Owner Leaf",
        concat!(
            "[[include component:image-block name=leaf.png|link=#]]\n",
            "[[include component:image-block name=2117.png|alt=alt|alt-text=An image|link=\"https://scp-wiki.wdfiles.com/local--files/fragment:attachment-owner-leaf/2117.png\"]]\n",
            "[[image direct-leaf.png]]\n",
            "[[image \"leaf two.png\"]]\n",
            "[[include component:attachment-owner-wrapper",
            " | asset=forwarded.png",
            " | spaced=forwarded two.png",
            "]]\n",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        WRAPPER_SLUG,
        "Attachment Owner Wrapper",
        &format!(
            concat!(
                "[[include {base_slug} | name={{$asset}} ",
                "[!-- trailing [x] | still comment --] | href={{$asset}} | ",
                "spaced={{$spaced}} | composite=thumb-{{$asset}}]]\n",
                "[[include component:image-block name={{$asset}}|link={{$asset}}]]",
            ),
            base_slug = BASE_SLUG,
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        SECOND_FRAGMENT_SLUG,
        "Attachment Owner Second Leaf",
        "[[include component:attachment-owner-wrapper | asset=forwarded.png | spaced=forwarded two.png]]",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        BASE_SLUG,
        "Attachment Owner Base",
        concat!(
            "[[image {$name} link={$href}]]\n",
            "[[image \"{$spaced}\" link=\"{$spaced}\"]]\n",
            "[[image {$composite} link={$composite}]]\n",
            "[[image literal-thumb.png link=literal-full.png]]\n",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        cross_site_id,
        CROSS_BASE_SLUG,
        "Cross-site Attachment Owner Base",
        "[[image \"{$name}\" link=\"{$name}\"]]",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        cross_site_id,
        CROSS_WRAPPER_SLUG,
        "Cross-site Attachment Owner Wrapper",
        &format!("[[include {CROSS_BASE_SLUG} | name={{$asset}}]]"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        cross_site_id,
        CROSS_FRAGMENT_SLUG,
        "Cross-site Attachment Owner Leaf",
        &format!("[[include {CROSS_WRAPPER_SLUG} | asset=cross site ?#%[]日本.png]]"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        CONSUMER_SLUG,
        "Attachment Owner Consumer",
        &format!(
            "[[include {FRAGMENT_SLUG}]]\n[[include {SECOND_FRAGMENT_SLUG}]]\n[[include :test:{CROSS_FRAGMENT_SLUG}]]\n[[include component:image-block name=root.png|link=#]]"
        ),
    )
    .await;

    let consumer = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": CONSUMER_SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("nested include attachment-owner consumer should exist");
    let html = consumer
        .compiled_body_html
        .expect("nested include attachment-owner consumer should have compiled HTML");

    assert!(
        html.contains("/local--files/fragment:attachment-owner-leaf/leaf.png"),
        "the nested included source must own its relative attachment: {html}"
    );
    assert_eq!(
        html.matches("/local--files/fragment:attachment-owner-leaf/2117.png")
            .count(),
        2,
        "the SCP-2117-shaped src and href must both retain the fragment attachment owner after host localization: {html}"
    );
    assert!(
        !html.contains("%22https%3A")
            && !html.contains("/local--files/fixture-attachment-owner-consumer/2117.png")
            && !html.contains("/local--files/component:image-block/2117.png")
            && !html.contains("/local--files/component:image-block-base/2117.png"),
        "the quoted link must not be encoded as an attachment and the consumer must not steal the source: {html}"
    );
    assert!(
        html.contains("/local--files/fragment:attachment-owner-leaf/direct-leaf.png"),
        "a direct relative image must retain the nested included source owner: {html}"
    );
    assert!(
        html.contains("/local--files/fragment:attachment-owner-leaf/leaf%20two.png"),
        "a quoted relative filename must retain its owner and be URL-encoded in final HTML: {html}"
    );
    for forwarded in ["forwarded.png", "forwarded%20two.png"] {
        let owned = format!("/local--files/fragment:attachment-owner-leaf/{forwarded}");
        let expected_occurrences = if forwarded == "forwarded.png" { 4 } else { 2 };
        assert_eq!(
            html.matches(&owned).count(),
            expected_occurrences,
            "a forwarded attachment must use its leaf-owned URL for both href and src: {html}"
        );
        let second_owned =
            format!("/local--files/fragment:attachment-owner-second-leaf/{forwarded}");
        assert_eq!(
            html.matches(&second_owned).count(),
            expected_occurrences,
            "same-valued forwarded occurrences from another leaf must retain their distinct owner: {html}"
        );
    }
    assert_eq!(
        html.matches(
            "/local--files/component:attachment-owner-wrapper/thumb-forwarded.png",
        )
        .count(),
        4,
        "a composite value must retain ordinary substitution and belong to the wrapper that authored the composite: {html}",
    );
    assert!(
        html.contains("/local--files/component:attachment-owner-base/literal-thumb.png")
            && html.contains(
                "/local--files/component:attachment-owner-base/literal-full.png"
            ),
        "literal image target and link must independently retain the base source owner: {html}"
    );
    let cross_owned = concat!(
        "test.wdfiles.com/local--files/fragment:attachment-owner-cross-leaf/",
        "cross%20site%20%3F%23%25%5B%5D%E6%97%A5%E6%9C%AC.png",
    );
    assert_eq!(
        html.matches(cross_owned).count(),
        0,
        "an unqualified nested include in a cross-site source resolves against the original callsite site: {html}",
    );
    assert!(
        html.contains(
            "Included page &quot;component:attachment-owner-cross-wrapper&quot; does not exist",
        ),
        "the missing callsite-local nested target must remain visible: {html}",
    );
    assert!(
        html.contains("/local--files/fixture-attachment-owner-consumer/root.png"),
        "the root source must retain ownership of its own relative attachment: {html}"
    );
    assert!(
        !html.contains("/local--files/fixture-attachment-owner-consumer/leaf.png")
            && !html.contains(
                "/local--files/fixture-attachment-owner-consumer/direct-leaf.png"
            )
            && !html.contains(
                "/local--files/fixture-attachment-owner-consumer/leaf%20two.png"
            )
            && !html
                .contains("/local--files/component:attachment-owner-wrapper/forwarded")
            && !html.contains("/local--files/component:attachment-owner-base/forwarded")
            && !html
                .contains("/local--files/component:attachment-owner-wrapper/leaf.png"),
        "neither the consumer nor an intermediate include may steal the leaf attachment: {html}"
    );
}

#[tokio::test]
async fn page_view_separates_generated_css_modules_from_compiled_body_html() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;
    let slug = "generated-css-head-fixture";

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
            "wikitext": concat!(
                "[[module CSS]]\n.first { color: red; }\n[[/module]]\n",
                "Generated CSS body marker.\n",
                "[[module CSS]]\n",
                ".second::after { content: \"</style><meta name=forged>\"; }\n",
                "[[/module]]\n",
                "[[html]]\n<style>.authored { color: green; }</style>\n[[/html]]\n",
            ),
            "title": "Generated CSS Head Fixture",
            "alt_title": null,
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "create generated CSS head fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(created.parser_errors.is_empty());

    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": slug, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let (compiled_body_html, compiled_body_styles) = match view {
        GetPageViewOutput::Found {
            compiled_body_html,
            compiled_body_styles,
            ..
        } => (compiled_body_html, compiled_body_styles),
        other => panic!("expected found page view, got {other:?}"),
    };

    assert!(compiled_body_html.contains("Generated CSS body marker."));
    assert!(!compiled_body_html.contains("<style"));
    assert_eq!(compiled_body_styles.len(), 2);
    assert!(compiled_body_styles[0].contains(".first { color: red; }"));
    assert!(compiled_body_styles[1].contains(r"\3C /style>\3C meta"));
    assert!(
        !compiled_body_styles
            .iter()
            .any(|css| css.contains(".authored"))
    );

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": slug,
            "details": {"compiled": true},
        }),
    )
    .expect("fixture page should exist");
    assert_eq!(
        page.compiled_body_html.as_deref(),
        Some(compiled_body_html.as_str())
    );
    assert_eq!(
        page.compiled_body_styles.as_ref(),
        Some(&compiled_body_styles),
    );

    let revision = run_endpoint!(
        runner,
        page_revision_get,
        json!({
            "site_id": site_id,
            "page_id": created.page_id,
            "revision_number": 0,
            "details": {"compiled_html": true},
        }),
    )
    .expect("fixture revision should exist");
    assert_eq!(revision.compiled_body_styles, Some(compiled_body_styles));
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
    assert_eq!(
        html.matches(
            "Included page &quot;missing-remote-include-self-cycle&quot; does not exist",
        )
        .count(),
        2,
        "{html}",
    );
}

#[tokio::test]
async fn render_scoped_include_source_cache_preserves_occurrence_semantics() {
    const SITE_SLUG: &str = "scp-wiki";
    const COMPONENT_SLUG: &str = "component:include-source-cache-cell";
    const CONSUMER_SLUG: &str = "fixture-include-source-cache-consumer";
    const PRIVATE_COMPONENT_SLUG: &str = "component:include-source-cache-private";
    const PRIVATE_CONSUMER_SLUG: &str = "fixture-include-source-cache-private-consumer";
    const PRIVATE_CATEGORY_SLUG: &str = "fixture-include-source-cache-private";
    const CYCLE_COMPONENT_SLUG: &str = "component:include-source-cache-cycle";
    const CYCLE_CONSUMER_SLUG: &str = "fixture-include-source-cache-cycle-consumer";
    const INCLUDE_COUNT: usize = 24;

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        COMPONENT_SLUG,
        "Include Source Cache Cell",
        "CACHE-{$label}-END",
    )
    .await;
    let repeated_includes = (0..INCLUDE_COUNT)
        .map(|index| {
            let target = match index % 3 {
                0 => COMPONENT_SLUG.to_owned(),
                1 => format!(":{SITE_SLUG}:{COMPONENT_SLUG}"),
                _ => format!("{COMPONENT_SLUG}#variant-{index}"),
            };
            format!("[[include {target} | label=occurrence-{index}]]\n")
        })
        .collect::<String>();
    create_listpages_test_page(
        &mut runner,
        site_id,
        CONSUMER_SLUG,
        "Include Source Cache Consumer",
        &repeated_includes,
    )
    .await;

    let component = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": COMPONENT_SLUG,
        }),
    )
    .expect("include source cache component should exist");
    let consumer = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": CONSUMER_SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("include source cache consumer should exist");
    let html = consumer
        .compiled_body_html
        .expect("include source cache consumer should have compiled HTML");
    for index in 0..INCLUDE_COUNT {
        let marker = format!("CACHE-occurrence-{index}-END");
        assert!(
            html.contains(&marker),
            "each cached raw source clone should receive its own variables: missing {marker} in {html}",
        );
    }

    let connections = LinkService::get_connections_from(
        runner.context(),
        consumer.page_id,
        Some(&[ConnectionType::IncludeMessy]),
    )
    .await
    .expect("include source cache consumer connections should load");
    let include_connection = connections
        .present
        .iter()
        .find(|connection| connection.to_page_id == component.page_id)
        .expect("the repeated include target should have a present connection");
    assert_eq!(
        include_connection.count, INCLUDE_COUNT as i32,
        "raw-source reuse must not deduplicate include occurrence backlinks",
    );

    create_listpages_test_page(
        &mut runner,
        site_id,
        PRIVATE_COMPONENT_SLUG,
        "Private Include Source Cache Cell",
        "PRIVATE_INCLUDE_SOURCE_MUST_NOT_RENDER",
    )
    .await;
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY_SLUG)
        .await;
    set_listpages_test_category_slug(
        &runner,
        site_id,
        PRIVATE_COMPONENT_SLUG,
        PRIVATE_CATEGORY_SLUG,
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        PRIVATE_CONSUMER_SLUG,
        "Private Include Source Cache Consumer",
        &format!(
            "Before private includes.\n[[include {PRIVATE_COMPONENT_SLUG}]]\n[[include {PRIVATE_COMPONENT_SLUG}]]\nAfter private includes.\n",
        ),
    )
    .await;
    let private_consumer = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": PRIVATE_CONSUMER_SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("private include source cache consumer should exist");
    let private_html = private_consumer
        .compiled_body_html
        .expect("private include source cache consumer should have compiled HTML");
    assert!(private_html.contains("Before private includes."));
    assert!(private_html.contains("After private includes."));
    assert!(!private_html.contains("PRIVATE_INCLUDE_SOURCE_MUST_NOT_RENDER"));
    assert_eq!(
        private_html
            .matches("Included page &quot;component:include-source-cache-private&quot; does not exist")
            .count(),
        2,
        "a cached permission denial must still render each missing occurrence",
    );

    let cycle_revision_id = create_listpages_test_page(
        &mut runner,
        site_id,
        CYCLE_COMPONENT_SLUG,
        "Include Source Cache Cycle",
        "placeholder",
    )
    .await;
    let cycle_wikitext = format!("[[include {CYCLE_COMPONENT_SLUG}]]\n");
    let cycle_wikitext_hash = TextService::create(runner.context(), cycle_wikitext)
        .await
        .expect("cyclic include source should be stored");
    let cycle_revision = PageRevisionTable::find_by_id(cycle_revision_id)
        .one(runner.context().transaction())
        .await
        .expect("cyclic include revision lookup should not fail")
        .expect("cyclic include revision should exist");
    let mut cycle_revision = cycle_revision.into_active_model();
    cycle_revision.wikitext_hash = Set(cycle_wikitext_hash.to_vec());
    cycle_revision
        .update(runner.context().transaction())
        .await
        .expect("cyclic source should be attached without rendering it");
    create_listpages_test_page(
        &mut runner,
        site_id,
        CYCLE_CONSUMER_SLUG,
        "Include Source Cache Cycle Consumer",
        "placeholder",
    )
    .await;
    let cycle_consumer = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": CYCLE_CONSUMER_SLUG,
        }),
    )
    .expect("include cycle consumer should exist");
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(CYCLE_CONSUMER_SLUG))),
    });
    let page_info = PageInfo {
        page: Cow::Borrowed(CYCLE_CONSUMER_SLUG),
        category: None,
        site: Cow::Borrowed(SITE_SLUG),
        title: Cow::Borrowed("Include Source Cache Cycle Consumer"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let cycle_error = RenderService::render_page(
        runner.context(),
        format!("[[include {CYCLE_COMPONENT_SLUG}]]\n"),
        &page_info,
        Layout::Wikidot,
        PageId {
            site_id,
            category_id: cycle_consumer.page_category_id,
            page_id: cycle_consumer.page_id,
        },
        UrlArguments::default(),
    )
    .await
    .expect_err("raw-source cache hits must not bypass include cycle depth checks");
    assert!(
        format!("{cycle_error:?}").contains("include expansion exceeded maximum depth 8"),
        "cached recursion should retain the established depth failure: {cycle_error:?}",
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
async fn page_render_emits_wikidot_rate_widget_structure() {
    let runner = TestRunner::setup().await;
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let page_info = PageInfo {
        page: Cow::Borrowed("rate-widget-fixture"),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("Rate Widget Fixture"),
        alt_title: None,
        score: ScoreValue::Integer(396),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };

    let output = RenderService::render(
        runner.context(),
        "[[=]]\n[[module Rate]]\n[[/=]]\n".to_owned(),
        &page_info,
        &settings,
    )
    .await
    .expect("page render with a rate module should succeed");
    let html = output.html_output.body;

    assert!(html.contains(
        "<div style=\"text-align: center;\"><div class=\"page-rate-widget-box\"><span class=\"rate-points\">rating:\u{a0}<span class=\"number prw54353\">+396</span></span>",
    ), "rate widget must be a direct child of its alignment container:\n{html}");
    assert!(html.contains(
        r#"<span class="rateup btn btn-default"><a href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, 1)" title="I like it">+</a></span>"#,
    ));
    assert!(!html.contains(r#"<div class="page-rate-widget-box"><p>"#));
    assert!(!html.contains(r#"<p><div class="page-rate-widget-box">"#));
    assert!(!html.contains(r#"<a href="javascript:;"><span class="rateup"#));
    assert_eq!(html.matches(r#"class="rate-points""#).count(), 1);
    assert!(!html.contains("WIKIJUMPWIKIDOTCOMPATHTML"));
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

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let category = "fixture-nextpreviouspage";
    let required_tag = "fixture-nextpreviouspage-required";
    let shared_tag = "fixture-nextpreviouspage-shared";

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
    let holder_revision = edited_holder.revision_id;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        holder_slug,
        holder_revision,
        &[shared_tag],
    )
    .await;

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
            "NOARG_END",
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

#[tokio::test]
async fn list_pages_saved_view_preserves_live_pagination_path_shapes() {
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
            && reverse_order.contains(&format!("href=\"/{HOLDER}/p/1/tag/alpha\"")),
        "the pager replaces a numeric page argument in place without reordering later arguments:\n{reverse_order}",
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
        r#"<div class="backlinks-module-box" data-wikijump-compat-backlinks="1">"#,
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
async fn backlinks_module_page_argument_targets_the_named_page() {
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

    assert!(html.contains(r#"<div class="backlinks-module-box">"#));
    assert!(!html.contains("TODO: module Backlinks"));
    assert!(!html.contains("[[module Backlinks"));
    assert!(
        !html.contains("Fixture Backlinks Unsupported Linker"),
        "page=\"start\" must not render backlinks to the containing page:\n{html}"
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
            "[[module Categories]]\n",
            "[[module categories includeHidden=\"true\"]]\n",
            "[[module CATEGORIES INCLUDEHIDDEN=\"false\"]]",
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

    assert_eq!(
        html.matches(&format!("<h3>{VISIBLE_CATEGORY}</h3>"))
            .count(),
        3
    );
    assert_eq!(
        html.matches(&format!("<h3>{HIDDEN_CATEGORY}</h3>")).count(),
        1
    );
    assert_eq!(
        html.matches("<h3>_default</h3>").count(),
        3,
        "_default remains visible without includeHidden:\n{html}"
    );
    for (category, category_id) in [
        (VISIBLE_CATEGORY, visible_category.category_id),
        (HIDDEN_CATEGORY, hidden_category.category_id),
    ] {
        assert!(
            html.contains(&format!(
                concat!(
                    "<h3>{category}</h3>\n",
                    "<a href=\"javascript:;\" id=\"category-pages-toggler-{category_id}\" ",
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
async fn page_tree_module_renders_current_page_hierarchy_with_live_depth_dom() {
    const ROOT: &str = "fixture-pagetree-root";
    const ALPHA: &str = "fixture-pagetree-alpha";
    const BETA: &str = "fixture-pagetree-beta";
    const GRANDCHILD: &str = "fixture-pagetree-grandchild";
    const GREAT_GRANDCHILD: &str = "fixture-pagetree-great-grandchild";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        ROOT,
        "PageTree Root",
        concat!(
            "PT_DEFAULT_START\n[[module PageTree depth=\"2\"]]\nPT_DEFAULT_END\n",
            "PT_SHOW_START\n[[module PageTree showRoot=\"true\" depth=\"1\"]]\nPT_SHOW_END\n",
            "PT_CASE_START\n[[module PageTree Showroot=\"true\" Depth=\"1\"]]\nPT_CASE_END",
        ),
    )
    .await;
    for (slug, title) in [
        (ALPHA, "Alpha Child"),
        (BETA, "Beta Child"),
        (GRANDCHILD, "Alpha Grandchild"),
        (GREAT_GRANDCHILD, "Alpha Great Grandchild"),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, "PageTree fixture")
            .await;
    }
    for (child, parent) in [
        (ALPHA, ROOT),
        (BETA, ROOT),
        (GRANDCHILD, ALPHA),
        (GREAT_GRANDCHILD, GRANDCHILD),
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
    let section = |start: &str, end: &str| {
        let start = html.find(start).expect("section start should render");
        let end = html[start..]
            .find(end)
            .map(|offset| start + offset)
            .expect("section end should render");
        &html[start..end]
    };

    let default = section("PT_DEFAULT_START", "PT_DEFAULT_END");
    assert!(
        default.contains(&format!(r#"<a href="/{ALPHA}">Alpha Child</a>"#)),
        "{default}",
    );
    assert!(
        default.contains(&format!(r#"<a href="/{GRANDCHILD}">Alpha Grandchild</a>"#))
    );
    assert!(default.contains(&format!(r#"<a href="/{BETA}">Beta Child</a>"#)));
    assert!(!default.contains("PageTree Root"));
    assert!(!default.contains("Alpha Great Grandchild"));
    assert!(
        default.find("Alpha Child") < default.find("Beta Child"),
        "siblings should preserve page_parent creation order:\n{default}"
    );

    let show_root = section("PT_SHOW_START", "PT_SHOW_END");
    assert!(show_root.contains(&format!(r#"<a href="/{ROOT}">PageTree Root</a>"#)));
    assert!(show_root.contains("Alpha Child"));
    assert!(show_root.contains("Beta Child"));
    assert!(!show_root.contains("Alpha Grandchild"));

    let case_variant = section("PT_CASE_START", "PT_CASE_END");
    assert!(!case_variant.contains("PageTree Root"));
    assert!(case_variant.contains("Alpha Great Grandchild"));
    for unsupported_wrapper in ["class=", " id=", "data-"] {
        assert!(
            !case_variant.contains(unsupported_wrapper),
            "PageTree DOM must remain plain ul, li, and a elements:\n{case_variant}"
        );
    }
    assert!(!html.contains("[[module PageTree"));
    assert!(!html.contains("TODO: module PageTree"));
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
    assert!(empty.contains(&format!("EMPTY_ROW {PREFIX}-1")), "{empty}");
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
    assert!(
        !zero_html.contains(ZERO_PRE) && !zero_html.contains(ZERO_POST),
        "zero-row ListPages must omit both prelude and postlude:\n{zero_html}"
    );
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
async fn listpages_link_and_fullname_keep_distinct_wikidot_identities() {
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
            "[[[%%link%%|absolute link]]]\n",
            "[[[%%fullname%%/noredirect/true|qualified name]]]\n",
            "[[/module]]",
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, INDEX_SLUG).await;
    assert!(
        html.contains(&format!(
            "href=\"http://scp-wiki.wikidot.com/{TARGET_SLUG}/noredirect/true\""
        )),
        "%%link%% must render the complete live-compatible Wikidot URL:\n{html}"
    );
    assert!(
        html.contains(&format!("href=\"/{TARGET_SLUG}/noredirect/true\"")),
        "%%fullname%% must render the category-qualified internal page name:\n{html}"
    );
    assert!(
        !html.contains(&format!("href=\"/{TARGET_SLUG}\"")),
        "%%link%% must not collapse to the internal full-name URL:\n{html}"
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
    const TARGET_SLUG: &str = "fixture-listpages-form-target";
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
                "%%tags%%\n",
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
        "live tagTarget is a no-op and must not make %%tags%% clickable:\n{tag_target}"
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
        "<h1 ",
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
        !html.contains("Second paragraph."),
        "the default template renders the summary, not the full content:\n{html}"
    );
    assert!(!html.contains("[[module ListPages"), "{html}");
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
        set_imported_author(&runner, site_id, IMPORT_RUN_ID, fixture).await;
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
async fn listpages_content_keeps_the_selected_pages_attachment_owner() {
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
    let selected_owner = format!("/local--files/{FRAGMENT_SLUG}/{FILE_NAME}");
    assert_eq!(
        html.matches(&selected_owner).count(),
        2,
        "ListPages row image src and href must both retain the selected page owner: {html}",
    );
    for direct_file in ["direct-row.png", "direct-row-full.png"] {
        assert!(
            html.contains(&format!("/local--files/{FRAGMENT_SLUG}/{direct_file}")),
            "a direct ListPages row image target and link must retain the selected page owner: {html}",
        );
    }
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
async fn listpages_content_keeps_same_named_attachments_separate_per_row() {
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

    for fragment in [FIRST_FRAGMENT, SECOND_FRAGMENT] {
        let row_owner = format!("/local--files/{fragment}/{FILE_NAME}");
        assert_eq!(
            html.matches(&row_owner).count(),
            2,
            "each ListPages row must independently own both src and href for the same filename: {html}",
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
async fn exact_name_listpages_batch_preserves_order_duplicates_and_permissions() {
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

    let duplicate_insert = runner
        .context()
        .transaction()
        .execute_raw(Statement::from_sql_and_values(
            runner.context().transaction().get_database_backend(),
            "INSERT INTO page (created_at, from_wikidot, site_id, page_category_id, slug, layout) SELECT TIMESTAMPTZ '2030-01-01 00:00:00+00' + duplicates.duplicate_number * INTERVAL '1 minute', source.from_wikidot, source.site_id, source.page_category_id, source.slug, source.layout FROM page AS source CROSS JOIN generate_series(1, 1000) AS duplicates(duplicate_number) WHERE source.site_id = $1 AND source.slug = $2 AND source.deleted_at IS NULL",
            [
                Value::from(site_id),
                Value::from("fixture-exact-batch-a".to_owned()),
            ],
        ))
        .await
        .expect("duplicate live exact-name pages should be inserted");
    assert_eq!(
        duplicate_insert.rows_affected(),
        1000,
        "exact-name duplicate fixture should fill the combined batch window before later slugs",
    );

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
                "[[module ListPages full_slug=\"fixture-exact-batch-a\"]]A1=%%slug%%@%%created_at|%Y %b %d %H:%M%%[[/module]]\n",
                "[[module ListPages fullslug=\"fixture-exact-batch-b\"]]B=%%slug%%[[/module]]\n",
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
        20,
        "batched duplicate rows should preserve the live default ListPages page size instead of collapsing to one row:\n{html}",
    );
    let newest_duplicate = html.find("2030 Jan 02 01:40").unwrap();
    let next_duplicate = html.find("2030 Jan 02 01:39").unwrap();
    assert!(
        newest_duplicate < next_duplicate,
        "batched duplicate rows should retain PageQuery order:\n{html}"
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
async fn fallback_link_title_batch_preserves_singular_duplicate_permission() {
    const TARGET_SLUG: &str = "fixture-fallback-title-duplicate";
    const FIRST_TITLE: &str = "Fallback duplicate first title";
    const SECOND_SLUG: &str = "fixture-fallback-title-duplicate-source";
    const SECOND_TITLE: &str = "Fallback duplicate second title";
    const FIRST_CATEGORY: &str = "fixture-fallback-title-first";
    const SECOND_CATEGORY: &str = "fixture-fallback-title-second";
    const INDEX_SLUG: &str = "fixture-fallback-title-index";
    const DEFAULT_LABEL: &str = "Fixture Fallback Title Duplicate";

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
    create_listpages_test_page(
        &mut runner,
        site_id,
        SECOND_SLUG,
        SECOND_TITLE,
        "second duplicate target",
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
    let second_page = PageTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page::Column::SiteId.eq(site_id))
                .add(page::Column::Slug.eq(SECOND_SLUG)),
        )
        .one(runner.context().transaction())
        .await
        .expect("second fallback target lookup should not fail")
        .expect("second fallback target should exist");
    let first_page_id = first_page.page_id;
    let second_page_id = second_page.page_id;
    let first_category =
        CategoryService::get_or_create(runner.context(), site_id, FIRST_CATEGORY)
            .await
            .expect("first fallback category should be created");
    let second_category =
        CategoryService::get_or_create(runner.context(), site_id, SECOND_CATEGORY)
            .await
            .expect("second fallback category should be created");
    let mut first_page = first_page.into_active_model();
    first_page.page_category_id = Set(first_category.category_id);
    first_page
        .update(runner.context().transaction())
        .await
        .expect("first fallback target should move to its category");
    let mut second_page = second_page.into_active_model();
    second_page.slug = Set(TARGET_SLUG.to_owned());
    second_page.page_category_id = Set(second_category.category_id);
    second_page
        .update(runner.context().transaction())
        .await
        .expect("second fallback target should become an active duplicate");

    let selected = PageService::get_optional(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed(TARGET_SLUG)),
    )
    .await
    .expect("singular duplicate lookup should not fail")
    .expect("singular duplicate lookup should select a page");
    let selected_page_id = selected.page_id;

    let (selected_category_slug, selected_category_id, other_page_id, other_category_id) =
        if selected_page_id == first_page_id {
            (
                FIRST_CATEGORY,
                first_category.category_id,
                second_page_id,
                second_category.category_id,
            )
        } else {
            assert_eq!(selected_page_id, second_page_id);
            (
                SECOND_CATEGORY,
                second_category.category_id,
                first_page_id,
                first_category.category_id,
            )
        };
    make_listpages_test_category_admin_only(&runner, site_id, selected_category_slug)
        .await;

    let selected_again = PageService::get_optional(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed(TARGET_SLUG)),
    )
    .await
    .expect("repeated singular duplicate lookup should not fail")
    .expect("repeated singular duplicate lookup should select a page");
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
    .expect("anonymous duplicate permission check should not fail");
    assert!(!can_view_selected);
    let can_view_other = PermissionService::check_user_can(
        runner.context(),
        &CheckPermissionContext {
            user_id: None,
            site_id,
            page_reference: Some(Reference::Id(other_page_id)),
        },
        Permission {
            resource_type: Resource::Page,
            resource_category: Some(Reference::Id(other_category_id)),
            action: Action::View,
        },
    )
    .await
    .expect("anonymous non-selected duplicate permission check should not fail");
    assert!(can_view_other);

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
        html.contains(&format!(r#"<a href="/{TARGET_SLUG}">{DEFAULT_LABEL}</a>"#)),
        "fallback title batch should use the singular lookup's denied permission decision:\n{html}",
    );
    assert!(!html.contains(FIRST_TITLE), "{html}");
    assert!(!html.contains(SECOND_TITLE), "{html}");
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
async fn page_delete_ignores_deleted_include_consumers_during_outdating() {
    let mut runner = TestRunner::setup().await;
    const SITE_SLUG: &str = "scp-wiki";
    const SOURCE_SLUG: &str = "component:fixture-delete-after-consumer-source";
    const CONSUMER_SLUG: &str = "fixture-delete-before-included-source";

    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("Seeded site not found");
    let site_id = site.site.site_id;

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(SOURCE_SLUG)),
    );
    let source = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "Included source body",
            "title": "Included Source",
            "alt_title": null,
            "slug": SOURCE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create included source",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(CONSUMER_SLUG)),
    );
    let consumer = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": format!("[[include {SOURCE_SLUG}]]"),
            "title": "Include Consumer",
            "alt_title": null,
            "slug": CONSUMER_SLUG,
            "layout": "wikidot",
            "revision_comments": "create include consumer",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(consumer.parser_errors.is_empty());

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(consumer.page_id),
    );
    run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": consumer.page_id,
            "last_revision_id": consumer.revision_id,
            "revision_comments": "delete include consumer first",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(source.page_id),
    );
    run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": source.page_id,
            "last_revision_id": source.revision_id,
            "revision_comments": "delete included source after consumer",
            "user_id": ADMIN_USER_ID,
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
    const SAME_ROW_CHILD_SLUG: &str = "fixture-listpages-include-budget-same-row-child";
    const COMMENT_CHILD_SLUG: &str = "fixture-listpages-include-budget-comment-child";
    const SECTION_CHILD_SLUG: &str = "fixture-listpages-include-budget-section-child";
    const GENERATED_SEPARATOR_COMPONENT_SLUG: &str =
        "component:listpages-generated-separator";
    const GENERATED_SEPARATOR_CHILD_SLUG: &str =
        "fixture-listpages-generated-separator-child";
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
    let same_row_child_wikitext =
        format!("[[include {COMPONENT_SLUG}]]\n=====\nPLAIN_SECTION\n");
    create_listpages_test_page(
        &mut runner,
        site_id,
        SAME_ROW_CHILD_SLUG,
        "ListPages Include Budget Same Row Child",
        &same_row_child_wikitext,
    )
    .await;
    let comment_child_wikitext =
        format!("[!--\n=====\n[[include {COMPONENT_SLUG}]]\n--]\n");
    create_listpages_test_page(
        &mut runner,
        site_id,
        COMMENT_CHILD_SLUG,
        "ListPages Include Budget Comment Child",
        &comment_child_wikitext,
    )
    .await;
    let section_child_wikitext = format!(
        "{}=====\n[[include {COMPONENT_SLUG}]]\n",
        format!("[[include {COMPONENT_SLUG}]]\n").repeat(255),
    );
    create_listpages_test_page(
        &mut runner,
        site_id,
        SECTION_CHILD_SLUG,
        "ListPages Include Budget Section Child",
        &section_child_wikitext,
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        GENERATED_SEPARATOR_COMPONENT_SLUG,
        "ListPages Generated Separator Component",
        "=====\nGENERATED_FROM_UNSELECTED_INCLUDE\n",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        GENERATED_SEPARATOR_CHILD_SLUG,
        "ListPages Generated Separator Child",
        &format!(
            "[[include {GENERATED_SEPARATOR_COMPONENT_SLUG}]]\n=====\nSOURCE_SELECTED_SECTION\n"
        ),
    )
    .await;

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
    let selected_section = format!(
        "[[include {COMPONENT_SLUG}]]\n[[module ListPages name=\"{SECTION_CHILD_SLUG}\"]]\n%%content{{2}}%%\n[[/module]]"
    );
    let output = RenderService::render_page(
        runner.context(),
        selected_section,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect("a structurally isolated content section should expand independently");
    assert_eq!(
        output.html_output.body.matches(INCLUDE_MARKER).count(),
        2,
        "includes outside an isolated requested section must not consume the render budget",
    );

    let generated_separator = format!(
        "[[module ListPages name=\"{GENERATED_SEPARATOR_CHILD_SLUG}\"]]\n%%content{{2}}%%\n[[/module]]"
    );
    let output = RenderService::render_page(
        runner.context(),
        generated_separator,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect("an unselected include must not create content section separators");
    assert!(
        output.html_output.body.contains("SOURCE_SELECTED_SECTION"),
        "the authored source section must determine content{{N}}: {}",
        output.html_output.body,
    );
    assert!(
        !output
            .html_output
            .body
            .contains("GENERATED_FROM_UNSELECTED_INCLUDE"),
        "an include outside the requested source section must remain unexpanded: {}",
        output.html_output.body,
    );
    assert_eq!(
        output
            .html_output
            .backlinks
            .included_pages
            .iter()
            .filter(|page| page.page() == GENERATED_SEPARATOR_COMPONENT_SLUG)
            .count(),
        0,
        "an include outside the requested source section must not create a backlink",
    );

    let comment_section = format!(
        "[[module ListPages name=\"{COMMENT_CHILD_SLUG}\"]]\nCOMMENT_ROW_RENDERED %%content{{2}}%%\n[[/module]]"
    );
    let output = RenderService::render_page(
        runner.context(),
        comment_section,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect(
        "ListPages should preserve whole-page literal context before selecting a section",
    );
    assert!(
        output.html_output.body.contains("COMMENT_ROW_RENDERED"),
        "the comment-boundary fixture must select and render its ListPages row",
    );
    assert!(
        !output.html_output.body.contains(INCLUDE_MARKER),
        "an include inside a comment spanning the selected section must remain inactive: {}",
        output.html_output.body,
    );
    assert_eq!(
        output
            .html_output
            .backlinks
            .included_pages
            .iter()
            .filter(|page| page.page() == COMPONENT_SLUG)
            .count(),
        0,
        "an inactive include crossing a section boundary must not create a backlink",
    );

    let direct_to_public_limit = format!("[[include {COMPONENT_SLUG}]]\n").repeat(255);
    let full_and_first_section = format!(
        "{direct_to_public_limit}[[module ListPages name=\"{SAME_ROW_CHILD_SLUG}\"]]\n%%content%%%%content{{1}}%%\n[[/module]]"
    );
    let output = RenderService::render_page(
        runner.context(),
        full_and_first_section,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect(
        "full content and a section in one row should share one child include expansion",
    );
    assert_eq!(
        output.html_output.body.matches(INCLUDE_MARKER).count(),
        257,
        "the once-expanded child include should render through both content variables",
    );
    assert_eq!(
        output
            .html_output
            .backlinks
            .included_pages
            .iter()
            .filter(|page| page.page() == COMPONENT_SLUG)
            .count(),
        256,
        "overlapping content variables in one row must charge and record the child include once",
    );

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
        UrlArguments::default(),
    )
    .await
    .expect("128 direct and 128 ListPages includes should fit the public limit");
    assert_eq!(
        output.html_output.body.matches(INCLUDE_MARKER).count(),
        256,
        "the render at the public limit should expand every include",
    );

    let repeated_child = format!("{direct_includes}{}{}", list_pages(1), list_pages(1),);
    let error = RenderService::render_page(
        runner.context(),
        repeated_child.clone(),
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect_err("separate ListPages blocks must charge the repeated child occurrence");
    assert!(
        format!("{error:?}")
            .contains("include expansion exceeded maximum total includes 256"),
        "separate ListPages blocks must share the public include ceiling: {error:?}",
    );
    let output = RenderService::render_corpus_page(
        runner.context(),
        repeated_child,
        &page_info,
        Layout::Wikidot,
        page_id,
    )
    .await
    .expect("the corpus budget should allow both repeated ListPages block occurrences");
    assert_eq!(
        output.html_output.body.matches(INCLUDE_MARKER).count(),
        384,
        "the child content must render at every ListPages block occurrence",
    );
    assert_eq!(
        output
            .html_output
            .backlinks
            .included_pages
            .iter()
            .filter(|page| page.page() == COMPONENT_SLUG)
            .count(),
        384,
        "separate ListPages blocks must record every child include occurrence",
    );

    let over_budget = format!("{direct_includes}{}", list_pages(2));
    let error = RenderService::render_page(
        runner.context(),
        over_budget.clone(),
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
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
    assert_eq!(
        output
            .html_output
            .backlinks
            .included_pages
            .iter()
            .filter(|page| page.page() == COMPONENT_SLUG)
            .count(),
        384,
        "separate ListPages rows must record every child include occurrence",
    );
}

#[tokio::test]
async fn listpages_content_runtime_budget_preserves_later_modules() {
    const INDEX_SLUG: &str = "fixture-listpages-content-row-budget-index";
    const CHILD_SLUG: &str = "fixture-listpages-content-row-budget-child";
    const CHILD_MARKER: &str = "LISTPAGES_CONTENT_ROW_BUDGET_CHILD";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        CHILD_SLUG,
        "ListPages Content Row Budget Child",
        CHILD_MARKER,
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "ListPages Content Row Budget Index",
        "placeholder",
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
        }),
    )
    .expect("ListPages content-row budget index should exist");
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
        title: Cow::Borrowed("ListPages Content Row Budget Index"),
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
    let wikitext = format!(
        "[[module ListPages name=\"{CHILD_SLUG}\" limit=\"250\"]]BROAD PRESERVED %%content%%[[/module]]\n[[module ListPages name=\"{CHILD_SLUG}\" limit=\"5000\" perPage=\"1\" order=\"random\"]]RANDOM PRESERVED %%content%%[[/module]]\n[[module ListPages name=\"{CHILD_SLUG}\" limit=\"1\"]]EXPANDED ONE %%content%%[[/module]]\n[[module ListPages name=\"{CHILD_SLUG}\" limit=\"1\"]]EXPANDED TWO %%content%%[[/module]]\n[[module ListPages name=\"{CHILD_SLUG}\" limit=\"1\"]]EXPANDED THREE %%content%%[[/module]]\n[[module ListPages name=\"{CHILD_SLUG}\" limit=\"1\"]]PRESERVED %%content%%[[/module]]\n[[module ListPages name=\"{CHILD_SLUG}\" limit=\"1\"]]METADATA %%title%%[[/module]]",
    );

    let output = RenderService::render_page(
        runner.context(),
        wikitext,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect("content runtime overflow should preserve the complete module");

    assert_eq!(
        output.html_output.body.matches(CHILD_MARKER).count(),
        3,
        "the first three deterministic content-backed modules should render",
    );
    assert!(
        output
            .html_output
            .body
            .contains(&format!("BROAD PRESERVED {CHILD_MARKER}")),
        "a broad deterministic request with a sparse result must expand its actual row: {}",
        output.html_output.body,
    );
    assert!(
        !output
            .html_output
            .body
            .contains("BROAD PRESERVED %%content%%")
    );
    assert!(
        output
            .html_output
            .body
            .contains("RANDOM PRESERVED %%content%%"),
        "a random content-backed module must remain literal without consuming the deterministic module budget: {}",
        output.html_output.body,
    );
    assert!(
        output
            .html_output
            .body
            .contains("EXPANDED THREE %%content%%"),
        "the fourth deterministic content-backed query must remain literal: {}",
        output.html_output.body,
    );
    assert!(
        output
            .html_output
            .body
            .contains("METADATA ListPages Content Row Budget Child"),
        "a later metadata-only module should still render: {}",
        output.html_output.body,
    );
}

#[tokio::test]
async fn corpus_render_supports_dense_includes_without_raising_public_limit() {
    const COMPONENT_SLUG: &str = "component:dense-include-cell";
    const PAGE_SLUG: &str = "fixture-dense-includes";
    const CHILD_SLUG: &str = "fixture-dense-listpages-child";
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
    let child_revision_id = create_listpages_test_page(
        &mut runner,
        site_id,
        CHILD_SLUG,
        "Dense ListPages Child",
        "placeholder",
    )
    .await;
    let wikitext_hash = TextService::create(runner.context(), wikitext.clone())
        .await
        .expect("dense child source should be stored");
    let child_revision = PageRevisionTable::find_by_id(child_revision_id)
        .one(runner.context().transaction())
        .await
        .expect("dense child revision lookup should not fail")
        .expect("dense child revision should exist");
    let mut child_revision = child_revision.into_active_model();
    child_revision.wikitext_hash = Set(wikitext_hash.to_vec());
    child_revision
        .update(runner.context().transaction())
        .await
        .expect("dense child source should be attached without public rendering");
    set_listpages_test_parent(&mut runner, site_id, CHILD_SLUG, PAGE_SLUG).await;
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
        UrlArguments::default(),
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

    let list_pages_wikitext = concat!(
        "[[module ListPages parent=\".\" limit=\"1\"]]",
        "%%content%%",
        "[[/module]]",
    )
    .to_owned();
    let public_list_pages_error = RenderService::render_page(
        runner.context(),
        list_pages_wikitext.clone(),
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect_err("ordinary ListPages content must retain the public include ceiling");
    assert!(
        format!("{public_list_pages_error:?}")
            .contains("include expansion exceeded maximum total includes 256")
    );

    let list_pages_output = RenderService::render_corpus_page(
        runner.context(),
        list_pages_wikitext,
        &page_info,
        Layout::Wikidot,
        page_id,
    )
    .await
    .expect("trusted ListPages content should inherit the corpus include ceiling");
    assert_eq!(
        list_pages_output.html_output.body.matches(MARKER).count(),
        INCLUDE_COUNT,
        "ListPages %%content%% should render every corpus-provenanced include",
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
async fn page_tags_select_filters_pages_by_authenticated_view_permission() {
    const VISIBLE_CATEGORY: &str = "xmlrpc-tags-visible";
    const HIDDEN_CATEGORY: &str = "xmlrpc-tags-hidden";
    const VISIBLE_SLUG: &str = "xmlrpc-tags-visible:source";
    const HIDDEN_SLUG: &str = "xmlrpc-tags-hidden:source";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        VISIBLE_CATEGORY,
        SAMPLE_USER_ID,
        &[Action::View],
        "sample-viewer",
    )
    .await;
    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        HIDDEN_CATEGORY,
        ADMIN_USER_ID,
        &[Action::View],
        "admin-viewer",
    )
    .await;

    let visible_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        VISIBLE_SLUG,
        "Visible XML-RPC Tag Source",
        "Visible tag source",
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        VISIBLE_SLUG,
        visible_revision,
        &["xmlrpc-visible-only", "xmlrpc-shared"],
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, VISIBLE_SLUG, VISIBLE_CATEGORY)
        .await;

    let hidden_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        HIDDEN_SLUG,
        "Hidden XML-RPC Tag Source",
        "Hidden tag source",
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        HIDDEN_SLUG,
        hidden_revision,
        &["xmlrpc-hidden-only", "xmlrpc-shared"],
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, HIDDEN_SLUG, HIDDEN_CATEGORY)
        .await;

    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("tag selection permission cache should be invalidated");
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(SAMPLE_USER_ID),
        site_id: Some(site_id),
        page_reference: None,
    });

    let hidden_page_tags = run_endpoint!(
        runner,
        page_tags_select,
        json!({
            "site": "scp-wiki",
            "pages": [HIDDEN_SLUG],
        }),
    );
    assert!(hidden_page_tags.is_empty());

    let hidden_category_tags = run_endpoint!(
        runner,
        page_tags_select,
        json!({
            "site": "scp-wiki",
            "categories": [HIDDEN_CATEGORY],
        }),
    );
    assert!(hidden_category_tags.is_empty());

    let mixed_tags = run_endpoint!(
        runner,
        page_tags_select,
        json!({
            "site": "scp-wiki",
            "pages": [VISIBLE_SLUG, HIDDEN_SLUG],
        }),
    );
    assert_eq!(
        mixed_tags.into_iter().collect::<BTreeSet<_>>(),
        BTreeSet::from(["xmlrpc-shared".to_owned(), "xmlrpc-visible-only".to_owned(),])
    );

    let visible_tags = run_endpoint!(
        runner,
        page_tags_select,
        json!({
            "site": "scp-wiki",
            "categories": [VISIBLE_CATEGORY],
            "pages": [VISIBLE_SLUG, HIDDEN_SLUG],
        }),
    );
    assert_eq!(
        visible_tags.into_iter().collect::<BTreeSet<_>>(),
        BTreeSet::from(["xmlrpc-shared".to_owned(), "xmlrpc-visible-only".to_owned(),])
    );

    let all_tags = run_endpoint!(
        runner,
        page_tags_select,
        json!({
            "site": "scp-wiki",
        }),
    );
    assert!(all_tags.contains(&"xmlrpc-visible-only".to_owned()));
    assert!(all_tags.contains(&"xmlrpc-shared".to_owned()));
    assert!(!all_tags.contains(&"xmlrpc-hidden-only".to_owned()));
}

#[tokio::test]
async fn page_tags_select_requires_an_authenticated_request_context() {
    let runner = TestRunner::setup().await;
    let error = run_endpoint_err!(
        runner,
        page_tags_select,
        json!({
            "site": "scp-wiki",
            "pages": [],
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);
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
            set_stored_point_vote(&runner, output.page_id, vote).await;
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

async fn set_listpages_test_category_rating_type(
    runner: &TestRunner,
    site_id: i64,
    category_slug: &str,
    rating_type: &str,
) {
    let category =
        CategoryService::get_or_create(runner.context(), site_id, category_slug)
            .await
            .expect("rating category test category should exist");
    let mut model = category.into_active_model();
    model.rating_type = Set(Some(rating_type.to_owned()));
    model
        .update(runner.context().transaction())
        .await
        .expect("rating category test update should not fail");
}

async fn set_listpages_test_category_template_page(
    runner: &TestRunner,
    site_id: i64,
    category_slug: &str,
    template_page_id: i64,
) {
    let category =
        CategoryService::get_or_create(runner.context(), site_id, category_slug)
            .await
            .expect("template category test category should exist");
    let mut model = category.into_active_model();
    model.template_page_id = Set(Some(template_page_id));
    model
        .update(runner.context().transaction())
        .await
        .expect("template category test update should not fail");
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

async fn create_listpages_test_import_run(
    runner: &TestRunner,
    site_id: i64,
    import_run_id: i64,
    row_count: i64,
) {
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            r#"
INSERT INTO wikidot_corpus_import_run (
    import_run_id, site_id, source_branch, source_site, manifest_sha256,
    manifest_row_count, complete_inventory, state, summary
) VALUES ($1, $2, 'author-selector-test', $3, decode(repeat('ab', 32), 'hex'), $4, false, 'metadata_done', '{}'::jsonb)
"#,
            [
                Value::from(import_run_id),
                Value::from(site_id),
                Value::from(format!("author-selector-test-{import_run_id}")),
                Value::from(row_count),
            ],
        ))
        .await
        .expect("author selector import run fixture should be inserted");
}

async fn set_imported_author(
    runner: &TestRunner,
    site_id: i64,
    import_run_id: i64,
    fixture: (i64, &str, u64, &str),
) {
    let (page_id, slug, source_entity_suffix, created_by_name) = fixture;
    let page = PageTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page::Column::SiteId.eq(site_id))
                .add(page::Column::PageId.eq(page_id)),
        )
        .one(runner.context().transaction())
        .await
        .expect("author selector page lookup should not fail")
        .expect("author selector page should exist");
    let mut page = page.into_active_model();
    page.from_wikidot = Set(true);
    page.update(runner.context().transaction())
        .await
        .expect("author selector page should be marked as imported");

    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            r#"
INSERT INTO wikidot_page_snapshot (
    page_id, source_branch, source_site, source_entity_id, source_fullname,
    source_created_at, source_updated_at, source_revision_count, imported_rating,
    created_by_name, comments, source_sha256, meta_sha256, meta_json,
    last_import_run_id
) VALUES (
    $1, 'author-selector-test', $2, $3::uuid, $4,
    NOW(), NOW(), 1, 0, $5, 0, decode(repeat('bc', 32), 'hex'),
    decode(repeat('cd', 32), 'hex'), '{}'::jsonb, $6
)
"#,
            [
                Value::from(page_id),
                Value::from(format!("author-selector-test-{import_run_id}")),
                Value::from(format!(
                    "71300000-0000-4000-8000-{source_entity_suffix:012x}"
                )),
                Value::from(slug.to_owned()),
                Value::from(created_by_name.to_owned()),
                Value::from(import_run_id),
            ],
        ))
        .await
        .expect("author selector snapshot fixture should be inserted");
}

async fn listpages_test_page_id(runner: &TestRunner, site_id: i64, slug: &str) -> i64 {
    PageTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page::Column::SiteId.eq(site_id))
                .add(page::Column::Slug.eq(slug)),
        )
        .one(runner.context().transaction())
        .await
        .expect("ListPages page ID lookup should not fail")
        .expect("ListPages page ID fixture should exist")
        .page_id
}

async fn load_listpages_test_compiled_html(
    runner: &TestRunner,
    site_id: i64,
    slug: &str,
) -> String {
    run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": slug,
            "details": {"compiled": true},
        }),
    )
    .expect("ListPages compiled HTML fixture should be readable")
    .compiled_body_html
    .expect("ListPages compiled HTML fixture should have compiled HTML")
}

async fn query_listpages_test_author_slugs(
    runner: &TestRunner,
    site_id: i64,
    tag: &str,
    author: AuthorSelector<'_>,
) -> Vec<String> {
    let all_tags = [Cow::Borrowed(tag)];
    PageQueryService::find(
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
                untagged: false,
            },
            page_parent: PageParentSelector::All,
            contains_outgoing_links: &[],
            creation_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            update_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            author,
            score: &[],
            votes: &[],
            offset: 0,
            range: RangeSelector::Current,
            name: None,
            slug: None,
            slugs: &[],
            data_form_fields: &[],
            order: Some(OrderBySelector {
                property: OrderProperty::FullSlug,
                ascending: true,
            }),
            candidate_limit: None,
            pagination: PaginationSelector {
                limit: Some(20),
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
    .expect("author selector PageQuery should succeed")
    .pages
    .into_iter()
    .map(|page| {
        page.slug
            .expect("author selector query should request slug")
    })
    .collect()
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
async fn countpages_inside_listpages_body_matches_live_empty_item_behavior() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-countpages-nested-listpages";

    let target_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-countpages-nested-listpages-target",
        "Fixture CountPages Nested ListPages Target",
        "Fixture CountPages nested target marker.",
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        "fixture-countpages-nested-listpages-target",
        target_revision,
        &[tag],
    )
    .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-countpages-nested-listpages-index",
        "Fixture CountPages Nested ListPages Index",
        &format!(
            r#"Nested CountPages marker.

[[module ListPages category="*" tags="+{tag}" limit="1" order="name"]]
OUTER=%%title%%
[[module CountPages category="*" tags="+{tag}" limit="10"]]
INNER_TOTAL=%%total%%; INNER_COUNT=%%count%%; [[[start|start link]]]
[[/module]]
[[/module]]"#
        ),
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "fixture-countpages-nested-listpages-index",
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("CountPages nested-in-ListPages index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains(r#"<div class="list-pages-box">"#)
            && html.contains(r#"<div class="list-pages-item">"#),
        "Wikidot still emits the outer ListPages container and an empty result item:\n{html}",
    );
    for forbidden in [
        "OUTER=",
        "INNER_TOTAL=",
        "INNER_COUNT=",
        "[[module CountPages",
        "%%total%%",
        "%%count%%",
        "start link",
    ] {
        assert!(
            !html.contains(forbidden),
            "CountPages inside a ListPages body should not expose {forbidden:?}, matching the live empty-item behavior:\n{html}",
        );
    }
}

#[tokio::test]
async fn inline_countpages_inside_listpages_body_matches_live_legacy_split_behavior() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-countpages-inline-listpages";

    let target_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-countpages-inline-listpages-target",
        "Fixture CountPages Inline ListPages Target",
        "Fixture CountPages inline target marker.",
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        "fixture-countpages-inline-listpages-target",
        target_revision,
        &[tag],
    )
    .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-countpages-inline-listpages-index",
        "Fixture CountPages Inline ListPages Index",
        &format!(
            r#"Inline CountPages marker.

[[module ListPages category="*" tags="+{tag}" limit="1" order="name"]]
BEFORE_INLINE
[[module CountPages category="*" tags="+{tag}" limit="10"]]INLINE_COUNT=%%total%%[[/module]]
AFTER_INLINE
[[/module]]"#
        ),
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "fixture-countpages-inline-listpages-index",
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("inline CountPages nested-in-ListPages index should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(
        html.contains("Fixture CountPages Inline ListPages Target"),
        "live Wikidot renders the outer ListPages with its default template before the split inline CountPages tail:\n{html}",
    );
    assert!(
        html.contains("BEFORE_INLINE") && html.contains("AFTER_INLINE"),
        "live Wikidot preserves text surrounding the inline CountPages line as downstream page source:\n{html}",
    );
    assert_eq!(
        html.matches(r#"<div class="list-pages-box">"#).count(),
        2,
        "live Wikidot emits the default outer ListPages box plus a second legacy list-pages-box around following text:\n{html}",
    );
    for forbidden in [
        "INLINE_COUNT=",
        "[[module CountPages",
        "%%total%%",
        "module CountPages",
    ] {
        assert!(
            !html.contains(forbidden),
            "inline CountPages inside a ListPages body should not expose {forbidden:?}, matching the live legacy split behavior:\n{html}",
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
async fn countpages_without_selectors_uses_current_category_and_normal_page_defaults() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let category_slug = "countpages-default-selector";

    for (slug, title) in [
        (
            format!("{category_slug}:target"),
            "Fixture CountPages Default Selector Target",
        ),
        (
            format!("{category_slug}:_hidden"),
            "Fixture CountPages Default Selector Hidden",
        ),
    ] {
        create_listpages_test_page(
            &mut runner,
            site_id,
            &slug,
            title,
            "Fixture CountPages default selector marker.",
        )
        .await;
    }

    let index_slug = format!("{category_slug}:index");
    create_listpages_test_page(
        &mut runner,
        site_id,
        &index_slug,
        "Fixture CountPages Default Selector Index",
        "CountPages default selector marker.\n\n[[module CountPages]]\nDEFAULT_NO_LIMIT_TOTAL=%%total%%; COUNT_ALIAS=%%count%%; [[[target|Target Link]]]\n[[/module]]",
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
        html.contains("DEFAULT_NO_LIMIT_TOTAL=2; COUNT_ALIAS=2"),
        "CountPages without explicit selectors should count normal pages in the current category, including the index page and excluding hidden pages:\n{html}"
    );
    assert!(
        html.contains("Target Link"),
        "CountPages should render ordinary wiki syntax in the module body:\n{html}"
    );
    for forbidden in [
        "[[module CountPages",
        "%%total%%",
        "%%count%%",
        "Fixture CountPages Default Selector Hidden",
    ] {
        assert!(
            !html.contains(forbidden),
            "CountPages default selector fixture should not contain {forbidden:?}:\n{html}"
        );
    }
    assert!(
        !html.contains("DEFAULT_NO_LIMIT_TOTAL=3"),
        "CountPages default pagetype should exclude underscore hidden pages:\n{html}"
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
async fn countpages_rating_filters_apply_scores() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let html = render_countpages_test_fixture_with_targets(
        &mut runner,
        site.site.site_id,
        "fixture-countpages-rating-filter",
        "verification-count-rating-filter",
        r#"tags="+verification-count-rating-filter" rating=">0" limit="20""#,
        "RATING_FILTER_COUNT=%%total%%",
        &[(
            "target-a",
            "Fixture CountPages Rating Filter Target",
            "Fixture CountPages rating filter marker.",
        )],
    )
    .await;

    assert!(
        html.contains("RATING_FILTER_COUNT=0"),
        "CountPages should apply the rating selector to the zero-score target:\n{html}"
    );
    assert!(
        !html.contains("%%total%%") && !html.contains("[[module CountPages"),
        "CountPages should substitute a complete rating-filtered count:\n{html}"
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
async fn first_revision_countpages_rating_filter_renders_exact_count() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let slug = "fixture-countpages-first-revision-rating-filter";
    let tag = "verification-count-first-revision-rating-filter";

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
            "title": "Fixture CountPages First Revision Rating Filter",
            "alt_title": null,
            "tags": [tag],
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "create first revision rating-filtered CountPages test page",
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
        html.contains("FIRST_REVISION_UNSUPPORTED_COUNT=0"),
        "the first revision should render the exact rating-filtered count:\n{html}"
    );
    assert!(
        !html.contains("%%total%%") && !html.contains("[[module CountPages"),
        "the first revision should not retain literal CountPages syntax:\n{html}"
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
                untagged: false,
            },
            page_parent: PageParentSelector::NoParent,
            contains_outgoing_links: &[],
            creation_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            update_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            author: AuthorSelector::All,
            score: &[],
            votes: &[],
            offset: 0,
            range: RangeSelector::Current,
            name: None,
            slug: None,
            slugs: &[],
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
                untagged: false,
            },
            page_parent: PageParentSelector::All,
            contains_outgoing_links: &[],
            creation_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            update_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            author: AuthorSelector::All,
            score: &[],
            votes: &[],
            offset: 0,
            range: RangeSelector::Current,
            name: None,
            slug: None,
            slugs: &[],
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

    let author_filter = [ADMIN_USER_ID];
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
                untagged: false,
            },
            page_parent: PageParentSelector::All,
            contains_outgoing_links: &[],
            creation_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            update_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            author: AuthorSelector::Any {
                user_ids: &author_filter,
                wikidot_snapshot_names: &[],
            },
            score: &[],
            votes: &[],
            offset: 0,
            range: RangeSelector::Current,
            name: None,
            slug: None,
            slugs: &[],
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
            set_stored_point_vote(&runner, output.page_id, vote).await;
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
            untagged: false,
        },
        page_parent: PageParentSelector::NoParent,
        contains_outgoing_links: &[],
        creation_date: DateSelector::FromPresent {
            start: OffsetDateTime::UNIX_EPOCH,
        },
        update_date: DateSelector::FromPresent {
            start: OffsetDateTime::UNIX_EPOCH,
        },
        author: AuthorSelector::All,
        score: &[],
        votes: &[],
        offset: 0,
        range: RangeSelector::Current,
        name: None,
        slug: None,
        slugs: &[],
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
async fn page_query_vote_filter_and_order_use_imported_snapshot_vote_counts() {
    const IMPORT_RUN_ID: i64 = 7_130_559;
    const ZERO: &str = "fixture-vote-filter-zero";
    const LOCAL_TWO: &str = "fixture-vote-filter-local-two";
    const IMPORTED: &str = "fixture-vote-filter-imported";
    const LEGACY_IMPORTED: &str = "fixture-vote-filter-legacy-imported";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (slug, title) in [
        (ZERO, "Vote Filter Zero"),
        (LOCAL_TWO, "Vote Filter Local Two"),
        (IMPORTED, "Vote Filter Imported"),
        (LEGACY_IMPORTED, "Vote Filter Legacy Imported"),
    ] {
        create_listpages_test_page(
            &mut runner,
            site_id,
            slug,
            title,
            "Vote filter fixture.",
        )
        .await;
    }

    let zero_id = listpages_test_page_id(&runner, site_id, ZERO).await;
    let local_two_id = listpages_test_page_id(&runner, site_id, LOCAL_TWO).await;
    let imported_id = listpages_test_page_id(&runner, site_id, IMPORTED).await;
    let legacy_imported_id =
        listpages_test_page_id(&runner, site_id, LEGACY_IMPORTED).await;

    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "INSERT INTO page_vote (from_wikidot, page_id, user_id, value) VALUES \
             (false, $1, $2, 1), \
             (false, $1, $3, -1), \
             (false, $4, $2, 1), \
             (true, $4, $3, 1), \
             (true, $5, $2, 1)",
            [
                Value::from(local_two_id),
                Value::from(ADMIN_USER_ID),
                Value::from(SAMPLE_USER_ID),
                Value::from(imported_id),
                Value::from(legacy_imported_id),
            ],
        ))
        .await
        .expect("vote-count fixtures should be inserted");
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "INSERT INTO page_vote (from_wikidot, page_id, user_id, value, deleted_at, disabled_at, disabled_by) VALUES \
             (false, $1, $2, 1, NOW(), NULL, NULL), \
             (false, $1, $3, 1, NULL, NOW(), $4)",
            [
                Value::from(imported_id),
                Value::from(UNKNOWN_USER_ID),
                Value::from(SYSTEM_USER_ID),
                Value::from(ADMIN_USER_ID),
            ],
        ))
        .await
        .expect("inactive vote-count fixtures should be inserted");

    create_listpages_test_import_run(&runner, site_id, IMPORT_RUN_ID, 1).await;
    set_imported_author(
        &runner,
        site_id,
        IMPORT_RUN_ID,
        (imported_id, IMPORTED, 560, "Imported Vote Author"),
    )
    .await;
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "UPDATE wikidot_page_snapshot SET meta_json = jsonb_build_object('votes_count', '7') WHERE page_id = $1",
            [Value::from(imported_id)],
        ))
        .await
        .expect("imported vote-count fixture should receive snapshot votes");

    assert_eq!(
        PageQueryService::effective_vote_count(runner.context(), zero_id)
            .await
            .expect("zero vote count should load"),
        0,
    );
    assert_eq!(
        PageQueryService::effective_vote_count(runner.context(), imported_id)
            .await
            .expect("imported vote count should load"),
        8,
        "snapshot votes_count should combine with active local non-Wikidot votes only",
    );

    let fixture_slugs = [
        Cow::Borrowed(ZERO),
        Cow::Borrowed(LOCAL_TWO),
        Cow::Borrowed(IMPORTED),
        Cow::Borrowed(LEGACY_IMPORTED),
    ];
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
            all_present: &[],
            none_present: &[],
            untagged: false,
        },
        page_parent: PageParentSelector::All,
        contains_outgoing_links: &[],
        creation_date: DateSelector::FromPresent {
            start: OffsetDateTime::UNIX_EPOCH,
        },
        update_date: DateSelector::FromPresent {
            start: OffsetDateTime::UNIX_EPOCH,
        },
        author: AuthorSelector::All,
        score: &[],
        votes: &[],
        offset: 0,
        range: RangeSelector::Current,
        name: None,
        slug: None,
        slugs: &fixture_slugs,
        data_form_fields: &[],
        order: Some(OrderBySelector {
            property: OrderProperty::PageSlug,
            ascending: true,
        }),
        candidate_limit: None,
        pagination: PaginationSelector {
            limit: Some(20),
            ..Default::default()
        },
        variables: &[],
        fields: FoundPageFields {
            slug: true,
            ..Default::default()
        },
    };

    let selector = [ScoreSelector {
        score: QueryScoreValue::Integer(1),
        comparison: ComparisonOperation::GreaterThan,
    }];
    let mut filtered_query = base_query.clone();
    filtered_query.votes = &selector;
    let filtered = PageQueryService::find(runner.context(), filtered_query)
        .await
        .expect("vote-count filter should not fail")
        .pages
        .into_iter()
        .map(|row| row.slug.expect("slug field should be requested"))
        .collect::<Vec<_>>();
    assert_eq!(filtered, [IMPORTED.to_owned(), LOCAL_TWO.to_owned()]);

    let mut ordered_query = base_query;
    ordered_query.order = Some(OrderBySelector {
        property: OrderProperty::Votes,
        ascending: false,
    });
    let ordered = PageQueryService::find(runner.context(), ordered_query)
        .await
        .expect("vote-count order should not fail")
        .pages
        .into_iter()
        .map(|row| row.slug.expect("slug field should be requested"))
        .collect::<Vec<_>>();
    assert_eq!(
        ordered,
        [
            IMPORTED.to_owned(),
            LOCAL_TWO.to_owned(),
            LEGACY_IMPORTED.to_owned(),
            ZERO.to_owned(),
        ],
    );
}

#[tokio::test]
async fn page_query_data_form_field_ordering_uses_static_form_values() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-page-query-data-form-order";

    for (slug, mainword, albums) in [
        ("fixture-data-form-order-beta", "beta", "2"),
        ("fixture-data-form-order-alpha", "alpha", "10"),
        ("fixture-data-form-order-gamma", "gamma", "1"),
    ] {
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            slug,
            slug,
            &format!(
                "mainword: {mainword}\nalbums: {albums}\n\nData form order fixture.",
            ),
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision, &[tag]).await;
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
            untagged: false,
        },
        page_parent: PageParentSelector::All,
        contains_outgoing_links: &[],
        creation_date: DateSelector::FromPresent {
            start: OffsetDateTime::UNIX_EPOCH,
        },
        update_date: DateSelector::FromPresent {
            start: OffsetDateTime::UNIX_EPOCH,
        },
        author: AuthorSelector::All,
        score: &[],
        votes: &[],
        offset: 0,
        range: RangeSelector::Current,
        name: None,
        slug: None,
        slugs: &[],
        data_form_fields: &[],
        order: Some(OrderBySelector {
            property: OrderProperty::DataFormFieldName {
                field: Cow::Borrowed("mainword"),
                numeric: false,
            },
            ascending: true,
        }),
        candidate_limit: None,
        pagination: PaginationSelector {
            limit: Some(20),
            ..Default::default()
        },
        variables: &[],
        fields: FoundPageFields {
            slug: true,
            ..Default::default()
        },
    };

    let ordered = PageQueryService::find(runner.context(), base_query.clone())
        .await
        .expect("data-form string ordering should not fail")
        .pages
        .into_iter()
        .map(|row| row.slug.expect("slug field should be requested"))
        .collect::<Vec<_>>();
    assert_eq!(
        ordered,
        [
            "fixture-data-form-order-alpha",
            "fixture-data-form-order-beta",
            "fixture-data-form-order-gamma",
        ],
    );

    let mut integer_order_query = base_query;
    integer_order_query.order = Some(OrderBySelector {
        property: OrderProperty::DataFormFieldName {
            field: Cow::Borrowed("albums"),
            numeric: true,
        },
        ascending: false,
    });
    let ordered = PageQueryService::find(runner.context(), integer_order_query)
        .await
        .expect("data-form integer ordering should not fail")
        .pages
        .into_iter()
        .map(|row| row.slug.expect("slug field should be requested"))
        .collect::<Vec<_>>();
    assert_eq!(
        ordered,
        [
            "fixture-data-form-order-alpha",
            "fixture-data-form-order-beta",
            "fixture-data-form-order-gamma",
        ],
    );
}

#[tokio::test]
async fn page_query_score_filter_plans_preserve_imported_and_local_vote_semantics() {
    const IMPORT_RUN_ID: i64 = 7_130_558;
    const PREFIX: &str = "fixture-score-filter-plan";
    const HIGH: &str = "fixture-score-filter-plan-high";
    const ZERO: &str = "fixture-score-filter-plan-zero";
    const LOW: &str = "fixture-score-filter-plan-low";
    const IMPORTED: &str = "fixture-score-filter-plan-imported";
    const LARGE_INTEGER: &str = "fixture-score-filter-plan-large-integer";
    const INACTIVE_VOTES: &str = "fixture-score-filter-plan-inactive-votes";
    const LEGACY_IMPORTED_VOTE: &str = "fixture-score-filter-plan-legacy-imported-vote";
    const DELETED_PAGE: &str = "fixture-score-filter-plan-soft-deleted";
    const DUMMY_PREFIX: &str = "fixture-score-filter-plan-dummy-";
    const LARGE_INTEGER_SCORE: i64 = 9_007_199_254_740_993;

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (slug, title) in [
        (HIGH, "Score Filter Plan High"),
        (ZERO, "Score Filter Plan Zero"),
        (LOW, "Score Filter Plan Low"),
        (IMPORTED, "Score Filter Plan Imported"),
        (LARGE_INTEGER, "Score Filter Plan Large Integer"),
        (INACTIVE_VOTES, "Score Filter Plan Inactive Votes"),
        (LEGACY_IMPORTED_VOTE, "Score Filter Plan Legacy Vote"),
        (DELETED_PAGE, "Score Filter Plan Soft Deleted"),
    ] {
        create_listpages_test_page(
            &mut runner,
            site_id,
            slug,
            title,
            "Score filter plan fixture.",
        )
        .await;
    }

    let high_id = listpages_test_page_id(&runner, site_id, HIGH).await;
    let low_id = listpages_test_page_id(&runner, site_id, LOW).await;
    let imported_id = listpages_test_page_id(&runner, site_id, IMPORTED).await;
    let large_integer_id = listpages_test_page_id(&runner, site_id, LARGE_INTEGER).await;
    let inactive_votes_id =
        listpages_test_page_id(&runner, site_id, INACTIVE_VOTES).await;
    let legacy_imported_vote_id =
        listpages_test_page_id(&runner, site_id, LEGACY_IMPORTED_VOTE).await;
    let deleted_page_id = listpages_test_page_id(&runner, site_id, DELETED_PAGE).await;

    for (page_id, value) in [
        (high_id, 5),
        (low_id, -2),
        (imported_id, 2),
        (inactive_votes_id, 9),
        (deleted_page_id, 20),
    ] {
        set_stored_point_vote(&runner, page_id, value).await;
    }

    create_listpages_test_import_run(&runner, site_id, IMPORT_RUN_ID, 2).await;
    set_imported_author(
        &runner,
        site_id,
        IMPORT_RUN_ID,
        (imported_id, IMPORTED, 558, "Imported Score Author"),
    )
    .await;
    set_imported_author(
        &runner,
        site_id,
        IMPORT_RUN_ID,
        (
            large_integer_id,
            LARGE_INTEGER,
            559,
            "Large Integer Score Author",
        ),
    )
    .await;

    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "UPDATE wikidot_page_snapshot SET imported_rating = 7 WHERE page_id = $1",
            [Value::from(imported_id)],
        ))
        .await
        .expect("imported score fixture should receive its snapshot rating");
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "UPDATE wikidot_page_snapshot SET imported_rating = $1 WHERE page_id = $2",
            [
                Value::from(LARGE_INTEGER_SCORE),
                Value::from(large_integer_id),
            ],
        ))
        .await
        .expect("large integer score fixture should receive its snapshot rating");
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "INSERT INTO page_vote (from_wikidot, page_id, user_id, value) VALUES (true, $1, $2, -5), (true, $3, $2, 3)",
            [
                Value::from(imported_id),
                Value::from(SAMPLE_USER_ID),
                Value::from(legacy_imported_vote_id),
            ],
        ))
        .await
        .expect("Wikidot vote fixtures should be inserted");
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "INSERT INTO page_vote (page_id, user_id, value, deleted_at, disabled_at, disabled_by) VALUES ($1, $2, -20, NOW(), NULL, NULL), ($1, $3, -20, NULL, NOW(), $4)",
            [
                Value::from(inactive_votes_id),
                Value::from(SAMPLE_USER_ID),
                Value::from(SYSTEM_USER_ID),
                Value::from(ADMIN_USER_ID),
            ],
        ))
        .await
        .expect("inactive score fixtures should be inserted");

    let deleted_page = PageTable::find_by_id(deleted_page_id)
        .one(transaction)
        .await
        .expect("soft-deleted score fixture lookup should succeed")
        .expect("soft-deleted score fixture should exist");
    let mut deleted_page = deleted_page.into_active_model();
    deleted_page.deleted_at = Set(Some(OffsetDateTime::now_utc()));
    deleted_page
        .update(transaction)
        .await
        .expect("score fixture page should be soft-deleted");

    let category_id = PageTable::find_by_id(high_id)
        .one(transaction)
        .await
        .expect("score fixture category lookup should succeed")
        .expect("score fixture page should exist")
        .page_category_id;
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "INSERT INTO page (site_id, page_category_id, slug) SELECT $1, $2, $3 || '-' || value FROM generate_series(1, 513) AS value",
            [
                Value::from(site_id),
                Value::from(category_id),
                Value::from(format!("{PREFIX}-dummy")),
            ],
        ))
        .await
        .expect("broad score-plan probe fixtures should be inserted");

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
            all_present: &[],
            none_present: &[],
            untagged: false,
        },
        page_parent: PageParentSelector::All,
        contains_outgoing_links: &[],
        creation_date: DateSelector::FromPresent {
            start: OffsetDateTime::UNIX_EPOCH,
        },
        update_date: DateSelector::FromPresent {
            start: OffsetDateTime::UNIX_EPOCH,
        },
        author: AuthorSelector::All,
        score: &[],
        votes: &[],
        offset: 0,
        range: RangeSelector::Current,
        name: None,
        slug: None,
        slugs: &[],
        data_form_fields: &[],
        order: Some(OrderBySelector {
            property: OrderProperty::PageSlug,
            ascending: true,
        }),
        candidate_limit: None,
        pagination: PaginationSelector {
            limit: Some(1_000),
            ..PaginationSelector::default()
        },
        variables: &[],
        fields: FoundPageFields {
            slug: true,
            ..FoundPageFields::default()
        },
    };
    let fixture_slugs = [
        Cow::Borrowed(HIGH),
        Cow::Borrowed(ZERO),
        Cow::Borrowed(LOW),
        Cow::Borrowed(IMPORTED),
        Cow::Borrowed(LARGE_INTEGER),
        Cow::Borrowed(INACTIVE_VOTES),
        Cow::Borrowed(LEGACY_IMPORTED_VOTE),
        Cow::Borrowed(DELETED_PAGE),
    ];

    fn selected_slugs(pages: deepwell::services::page_query::FoundPages) -> Vec<String> {
        pages
            .pages
            .into_iter()
            .map(|page| page.slug.expect("score filter query requested slugs"))
            .collect()
    }

    for (threshold, comparison, expected) in [
        (
            0,
            ComparisonOperation::GreaterThan,
            vec![
                HIGH.to_owned(),
                IMPORTED.to_owned(),
                INACTIVE_VOTES.to_owned(),
                LARGE_INTEGER.to_owned(),
                LEGACY_IMPORTED_VOTE.to_owned(),
            ],
        ),
        (0, ComparisonOperation::LessThan, vec![LOW.to_owned()]),
        (
            8,
            ComparisonOperation::GreaterThan,
            vec![
                IMPORTED.to_owned(),
                INACTIVE_VOTES.to_owned(),
                LARGE_INTEGER.to_owned(),
            ],
        ),
        (0, ComparisonOperation::Equal, vec![ZERO.to_owned()]),
    ] {
        let score = [ScoreSelector {
            score: QueryScoreValue::Integer(threshold),
            comparison,
        }];

        let mut correlated_query = base_query.clone();
        correlated_query.score = &score;
        correlated_query.slugs = &fixture_slugs;
        let correlated = PageQueryService::find(runner.context(), correlated_query)
            .await
            .expect("candidate-correlated score filter should succeed");

        let mut site_wide_query = base_query.clone();
        site_wide_query.score = &score;
        site_wide_query.name = Some(Cow::Owned(format!("{PREFIX}-*")));
        let site_wide = PageQueryService::find(runner.context(), site_wide_query)
            .await
            .expect("site-wide score filter should succeed");

        assert_eq!(selected_slugs(correlated), expected);
        assert_eq!(
            selected_slugs(site_wide)
                .into_iter()
                .filter(|slug| !slug.starts_with(DUMMY_PREFIX))
                .collect::<Vec<_>>(),
            expected,
        );
    }

    let bounded_score = [
        ScoreSelector {
            score: QueryScoreValue::Integer(0),
            comparison: ComparisonOperation::GreaterOrEqualThan,
        },
        ScoreSelector {
            score: QueryScoreValue::Integer(5),
            comparison: ComparisonOperation::LessOrEqualThan,
        },
    ];
    let expected_bounded = vec![
        HIGH.to_owned(),
        LEGACY_IMPORTED_VOTE.to_owned(),
        ZERO.to_owned(),
    ];

    let mut correlated_query = base_query.clone();
    correlated_query.score = &bounded_score;
    correlated_query.slugs = &fixture_slugs;
    let correlated = PageQueryService::find(runner.context(), correlated_query)
        .await
        .expect("candidate-correlated repeated score filters should succeed");

    let mut site_wide_query = base_query.clone();
    site_wide_query.score = &bounded_score;
    site_wide_query.name = Some(Cow::Owned(format!("{PREFIX}-*")));
    let site_wide = PageQueryService::find(runner.context(), site_wide_query)
        .await
        .expect("site-wide repeated score filters should succeed");

    assert_eq!(selected_slugs(correlated), expected_bounded);
    assert_eq!(
        selected_slugs(site_wide)
            .into_iter()
            .filter(|slug| !slug.starts_with(DUMMY_PREFIX))
            .collect::<Vec<_>>(),
        expected_bounded,
    );

    for (threshold, expected) in [
        (LARGE_INTEGER_SCORE - 1, Vec::new()),
        (LARGE_INTEGER_SCORE, vec![LARGE_INTEGER.to_owned()]),
    ] {
        let score = [ScoreSelector {
            score: QueryScoreValue::Integer(threshold),
            comparison: ComparisonOperation::Equal,
        }];

        let mut correlated_query = base_query.clone();
        correlated_query.score = &score;
        correlated_query.slugs = &fixture_slugs;
        let correlated = PageQueryService::find(runner.context(), correlated_query)
            .await
            .expect("large integer candidate-correlated score filter should succeed");

        let mut site_wide_query = base_query.clone();
        site_wide_query.score = &score;
        site_wide_query.name = Some(Cow::Owned(format!("{PREFIX}-*")));
        let site_wide = PageQueryService::find(runner.context(), site_wide_query)
            .await
            .expect("large integer site-wide score filter should succeed");

        assert_eq!(selected_slugs(correlated), expected);
        assert_eq!(
            selected_slugs(site_wide)
                .into_iter()
                .filter(|slug| !slug.starts_with(DUMMY_PREFIX))
                .collect::<Vec<_>>(),
            expected,
        );
    }
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
                untagged: false,
            },
            page_parent: PageParentSelector::All,
            contains_outgoing_links: &[],
            creation_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            update_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            author: AuthorSelector::All,
            score: &[],
            votes: &[],
            offset: 0,
            range: RangeSelector::Current,
            name: None,
            slug: None,
            slugs: &[],
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
                untagged: false,
            },
            page_parent: PageParentSelector::All,
            contains_outgoing_links: &[],
            creation_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            update_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            author: AuthorSelector::All,
            score: &[],
            votes: &[],
            offset: 0,
            range: RangeSelector::Current,
            name: None,
            slug: None,
            slugs: &[],
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
async fn page_query_data_form_candidate_cap_marks_partial_result_incomplete() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-page-query-data-form-cap";

    for suffix in ["a", "b"] {
        let slug = format!("fixture-page-query-data-form-cap-{suffix}");
        let revision = create_listpages_test_page(
            &mut runner,
            site_id,
            &slug,
            "Fixture PageQuery data form cap",
            "status: wanted\n\nFixture PageQuery data form cap marker.",
        )
        .await;
        set_listpages_test_tags(&mut runner, site_id, &slug, revision, &[tag]).await;
    }

    let all_tags = [Cow::Borrowed(tag)];
    let data_form_fields = [DataFormSelector {
        field: Cow::Borrowed("status"),
        value: Cow::Borrowed("wanted"),
        negated: false,
    }];
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
                untagged: false,
            },
            page_parent: PageParentSelector::All,
            contains_outgoing_links: &[],
            creation_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            update_date: DateSelector::FromPresent {
                start: OffsetDateTime::UNIX_EPOCH,
            },
            author: AuthorSelector::All,
            score: &[],
            votes: &[],
            offset: 0,
            range: RangeSelector::Current,
            name: None,
            slug: None,
            slugs: &[],
            data_form_fields: &data_form_fields,
            order: Some(OrderBySelector {
                property: OrderProperty::PageSlug,
                ascending: true,
            }),
            candidate_limit: Some(1),
            pagination: PaginationSelector {
                limit: Some(20),
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
    .expect("capped data form query should return an explicitly incomplete result");

    assert_eq!(result.pages.total(), 1);
    assert_eq!(result.metadata.candidate_count, Some(1));
    assert!(result.metadata.cap_exceeded);
    assert!(result.metadata.filtering_deferred_to_rust);
    assert!(!result.metadata.exact_count_safe);
}

#[tokio::test]
async fn listpages_deferred_forms_remain_unsupported() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");

    // `created_by="-="` used to belong here. It is now supported and evidenced,
    // and `created_by_exclusion_omits_the_containing_pages_author` in
    // tests/list_pages.rs asserts the rendered exclusion instead.
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
async fn excessive_score_selectors_preserve_listpages_and_countpages_modules() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let excessive_selectors = r#" score=">=0""#.repeat(65);
    let forged_marker = "WIKIJUMPWIKIDOTCOMPATTEXTffffffffffffffffffffffffffffffffI0X";
    let target_slug = "fixture-score-selector-cap-target";
    let target_tag = "verification-score-selector-cap";
    let target_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        target_slug,
        "Fixture Score Selector Cap Target",
        "Fixture score selector cap target marker.",
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        target_slug,
        target_revision,
        &[target_tag],
    )
    .await;

    let list_head = format!(r#"tags="+{target_tag}" limit="20"{excessive_selectors}"#);
    let count_head = format!(r#"tags="+{target_tag}" limit="20"{excessive_selectors}"#);
    let index_slug = "fixture-score-selector-cap-index";
    let index_source = format!(
        "[[module ListPages {list_head}]]\nSCORE_SELECTOR_CAP_LIST=%%slug%% <script>&\"' {forged_marker}\n[[/module]]\n\n[[module CountPages {count_head}]]\nSCORE_SELECTOR_CAP_COUNT=%%total%% <script>&\"' {forged_marker}\n[[/module]]",
    );
    create_listpages_test_page(
        &mut runner,
        site_id,
        index_slug,
        "Fixture Score Selector Cap Index",
        &index_source,
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
    .expect("score selector cap index should exist");
    let html = page
        .compiled_body_html
        .expect("score selector cap index should include compiled HTML");

    assert!(
        html.contains("SCORE_SELECTOR_CAP_LIST=%%slug%%"),
        "ListPages must preserve an excessive score-selector module instead of running a truncated query:\n{html}",
    );
    assert!(
        html.contains("SCORE_SELECTOR_CAP_COUNT=%%total%%"),
        "CountPages must preserve an excessive score-selector module instead of returning a partial count:\n{html}",
    );
    assert!(
        !html.contains(target_slug),
        "capped modules must not query rows:\n{html}"
    );
    assert_eq!(
        html.matches("&lt;script&gt;&amp;&quot;&#39;").count(),
        2,
        "both preserved modules must restore dangerous text only after HTML escaping:\n{html}",
    );
    assert!(
        !html.contains("<script>"),
        "preserved syntax must stay inert:\n{html}"
    );
    assert_eq!(
        html.matches(forged_marker).count(),
        2,
        "authored marker-shaped text must not resolve through the shared registry:\n{html}",
    );
}

#[tokio::test]
async fn countpages_does_not_execute_modules_owned_by_ftml_text_constructs() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-countpages-ftml-text-owner";
    let target_slug = "fixture-countpages-ftml-text-owner-target";
    let target_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        target_slug,
        "Fixture CountPages FTML Text Owner Target",
        "Fixture CountPages FTML text owner target marker.",
    )
    .await;
    set_listpages_test_tags(&mut runner, site_id, target_slug, target_revision, &[tag])
        .await;

    let hidden_module = |marker: &str| {
        format!(r#"[[module CountPages tags="+{tag}"]]{marker}=%%total%%[[/module]]"#,)
    };
    let owned_markers = [
        "ownedlabelsingle",
        "ownedlabeltriple",
        "ownedlabelanchor",
        "ownedquotedhead",
        "ownedtargetsingle",
        "ownedtargetanchor",
        "ownedtargettriple",
    ];
    let count_only =
        format!(r#"[[module CountPages tags="+{tag}"]]%%total%%[[/module]]"#,);
    let source = format!(
        "[https://e.test/ {} label]\n\n\
         [[[target|{} label]]]\n\n\
         [#toc {} label]\n\n\
         [[span title='{} label']]body[[/span]]\n\n\
         [https://e.test/{} label]\n\n\
         [#toc{} label]\n\n\
         [[[target {} suffix]]]\n\n\
         ##rgb(1,2,{count_only})|owned color body##\n\n\
         [[module CountPages tags=\"+{tag}\"]]ownedlive=%%total%%[[/module]]",
        hidden_module(owned_markers[0]),
        hidden_module(owned_markers[1]),
        hidden_module(owned_markers[2]),
        hidden_module(owned_markers[3]),
        hidden_module(owned_markers[4]),
        hidden_module(owned_markers[5]),
        hidden_module(owned_markers[6]),
    );
    let index_slug = "fixture-countpages-ftml-text-owner-index";
    create_listpages_test_page(
        &mut runner,
        site_id,
        index_slug,
        "Fixture CountPages FTML Text Owner Index",
        &source,
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
    .expect("CountPages FTML text owner index should exist");
    let html = page
        .compiled_body_html
        .expect("CountPages FTML text owner index should include compiled HTML");

    assert!(
        html.contains("ownedlive=1"),
        "a CountPages module outside FTML-owned text must still execute:\n{html}",
    );
    for marker in owned_markers {
        assert!(
            html.contains(marker),
            "FTML-owned CountPages source should remain represented in rendered output for {marker}:\n{html}",
        );
        assert!(
            !html.contains(&format!("{marker}=1")),
            "CountPages must not execute inside an FTML-owned text construct for {marker}:\n{html}",
        );
    }
    assert!(
        html.contains("owned color body"),
        "the pinned-valid color construct should render its body:\n{html}",
    );
    assert!(
        !html.contains("rgb(1,2,1)"),
        "CountPages must not execute while FTML owns the color descriptor:\n{html}",
    );
}

#[tokio::test]
async fn countpages_preserves_runtime_unsafe_outer_heads_without_executing_inner_modules()
{
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-countpages-runtime-unsafe-head";
    let target_slug = "fixture-countpages-runtime-unsafe-head-target";
    let target_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        target_slug,
        "Fixture CountPages Runtime Unsafe Head Target",
        "Fixture CountPages runtime unsafe head target marker.",
    )
    .await;
    set_listpages_test_tags(&mut runner, site_id, target_slug, target_revision, &[tag])
        .await;

    let nested_module = |head: &str, marker: &str| {
        format!(
            "[[module CountPages {head}]]\n\
             {marker}outer=%%total%% <script>&\"'\n\
             [[module CountPages tags=\"+{tag}\"]]{marker}inner=%%total%%[[/module]]\n\
             [[/module]]",
        )
    };
    let cases = [
        (
            "ownedquote",
            nested_module(r#"name = "secret@site.example" wrapper=no"#, "ownedquote"),
        ),
        (
            "embeddedquote",
            nested_module(r#"name = "secret"wrapper="no""#, "embeddedquote"),
        ),
        (
            "escapedquote",
            nested_module(r#"name = "secret\" wrapper=no"#, "escapedquote"),
        ),
        (
            "unicodeseparator",
            nested_module("limit\\\n\u{00a0}=\"1\"", "unicodeseparator"),
        ),
    ];
    let source = format!(
        "{}\n\n{}\n\n{}\n\n{}\n\n\
         [[module CountPages tags=\"+{tag}\"]]nestedlive=%%total%%[[/module]]",
        cases[0].1, cases[1].1, cases[2].1, cases[3].1,
    );
    let index_slug = "fixture-countpages-runtime-unsafe-head-index";
    create_listpages_test_page(
        &mut runner,
        site_id,
        index_slug,
        "Fixture CountPages Runtime Unsafe Head Index",
        &source,
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
    .expect("CountPages runtime unsafe head index should exist");
    let html = page
        .compiled_body_html
        .expect("CountPages runtime unsafe head index should include compiled HTML");

    assert!(
        html.contains("nestedlive=1"),
        "an outside CountPages module must still execute:\n{html}",
    );
    for (marker, _) in cases {
        for suffix in ["outer", "inner"] {
            assert!(
                html.contains(&format!("{marker}{suffix}=%%total%%")),
                "the original {marker} {suffix} module text must remain preserved:\n{html}",
            );
            assert!(
                !html.contains(&format!("{marker}{suffix}=1")),
                "neither the runtime-unsafe outer module nor its valid inner module may execute for {marker}:\n{html}",
            );
        }
    }
    assert_eq!(
        html.matches("&lt;script&gt;&amp;&quot;&#39;").count(),
        4,
        "every preserved unsafe outer module must restore authored text only after escaping:\n{html}",
    );
    assert!(
        !html.contains("<script>"),
        "preserved runtime-unsafe module syntax must stay inert:\n{html}",
    );
}

#[tokio::test]
async fn score_selectors_at_limit_render_listpages_normally() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let selectors_at_limit = r#" score=">=0""#.repeat(64);
    let list_tag = "verification-list-score-selector-limit";
    let list_head = format!(r#"tags="+{list_tag}" limit="20"{selectors_at_limit}"#);
    let target_slug = "fixture-list-score-selector-limit-target";
    let target_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        target_slug,
        "Fixture ListPages Score Selector Limit Target",
        "Fixture ListPages score selector limit marker.",
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        target_slug,
        target_revision,
        &[list_tag],
    )
    .await;
    let index_slug = "fixture-list-score-selector-limit-index";
    create_listpages_test_page(
        &mut runner,
        site_id,
        index_slug,
        "Fixture ListPages Score Selector Limit Index",
        &format!(
            "[[module ListPages {list_head}]]\nSCORE_SELECTOR_LIMIT_LIST=%%slug%%\n[[/module]]"
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
    .expect("ListPages score selector limit index should exist");
    let list_html = page
        .compiled_body_html
        .expect("ListPages score selector limit index should include compiled HTML");
    assert!(
        list_html.contains(&format!("SCORE_SELECTOR_LIMIT_LIST={target_slug}")),
        "ListPages selectors at the limit must still execute normally:\n{list_html}",
    );
}

#[tokio::test]
async fn score_selectors_at_limit_render_countpages_normally() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let selectors_at_limit = r#" score=">=0""#.repeat(64);
    let count_tag = "verification-count-score-selector-limit";
    let count_head = format!(r#"tags="+{count_tag}" limit="20"{selectors_at_limit}"#);
    let target_slug = "fixture-count-score-selector-limit-target";
    let target_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        target_slug,
        "Fixture CountPages Score Selector Limit Target",
        "Fixture CountPages score selector limit marker.",
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        target_slug,
        target_revision,
        &[count_tag],
    )
    .await;
    let index_slug = "fixture-count-score-selector-limit-index";
    create_listpages_test_page(
        &mut runner,
        site_id,
        index_slug,
        "Fixture CountPages Score Selector Limit Index",
        &format!(
            "[[module CountPages {count_head}]]\nSCORE_SELECTOR_LIMIT_COUNT=%%total%%\n[[/module]]"
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
    .expect("CountPages score selector limit index should exist");
    let count_html = page
        .compiled_body_html
        .expect("CountPages score selector limit index should include compiled HTML");
    assert!(
        count_html.contains("SCORE_SELECTOR_LIMIT_COUNT=1"),
        "CountPages selectors at the limit must still execute normally:\n{count_html}",
    );
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
