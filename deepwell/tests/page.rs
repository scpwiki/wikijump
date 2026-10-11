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

#[path = "page/page_modules.rs"]
mod page_modules;

#[path = "page/theme_previewer.rs"]
mod theme_previewer;

#[path = "page/forum.rs"]
mod forum;

#[path = "page/membership.rs"]
mod membership;

#[path = "page/list_pages_runtime.rs"]
mod list_pages_runtime;

#[path = "page/page_move.rs"]
mod page_move;

#[path = "page/page_query.rs"]
mod page_query;

use self::common::TestRunner;
use cuid2::cuid;
use deepwell::api::{
    ServerState, build_server_at, build_server_state, build_server_state_without_workers,
};
use deepwell::config::{Config, Secrets};
use deepwell::constants::{
    ADMIN_USER_ID, ANONYMOUS_USER_ID, SAMPLE_USER_ID, SYSTEM_USER_ID, UNKNOWN_USER_ID,
};
use deepwell::error::exn_error_to_rpc_error;
use deepwell::error::prelude::*;
use deepwell::hash::{blob_hash_to_hex, sha512_hash};
use deepwell::license::License;
use deepwell::models::audit_log::{Column as AuditLogColumn, Entity as AuditLogTable};
use deepwell::models::blob_pending::{self, Entity as BlobPendingTable};
use deepwell::models::file;
use deepwell::models::file_revision::Entity as FileRevisionTable;
use deepwell::models::forum_post::{self, Entity as ForumPostTable};
use deepwell::models::forum_thread::Entity as ForumThreadTable;
use deepwell::models::known_user;
use deepwell::models::page::{self, Entity as PageTable};
use deepwell::models::page_category::{self, Entity as PageCategoryTable};
use deepwell::models::page_revision::{self, Entity as PageRevisionTable};
use deepwell::models::role_permission::{self, Entity as RolePermissionTable};
use deepwell::models::session::Entity as SessionTable;
use deepwell::models::text;
use deepwell::models::text_block;
use deepwell::models::user::Entity as UserTable;
use deepwell::services::blob::{ContentTypeDescriptor, EMPTY_BLOB_HASH, EMPTY_BLOB_MIME};
use deepwell::services::category::CategoryService;
use deepwell::services::file_revision::CreateFirstFileRevision;
use deepwell::services::forum::{
    CreateForumCategory, CreateForumGroup, DeleteForumCategory, DeleteForumGroup,
    GetForumStructure,
};
use deepwell::services::forum_post::{
    CreateForumPost, DeleteForumPost, UpdateForumPost, UpdateForumPostBody,
};
use deepwell::services::forum_thread::CreateForumThread;
use deepwell::services::import::{ImportService, ImportUser, ImportedUserType};
use deepwell::services::page::{CreatePage, GetPageOutput};
use deepwell::services::page_draft::{PageDraftPageType, PageDraftService};
use deepwell::services::page_lock::{CreatePageLockInput, PageLockService};
use deepwell::services::page_query::{
    AuthorSelector, CategoriesSelector, ComparisonOperation, DataFormSelector,
    DateSelector, FoundPageFields, IncludedCategories, OrderBySelector, OrderProperty,
    PageParentSelector, PageQuery, PageQueryService, PageTypeSelector,
    PaginationSelector, RangeSelector, ScoreSelector, TagCondition,
};
use deepwell::services::page_revision::{PageRevisionService, RerenderType};
use deepwell::services::permission::{
    CheckPermissionContext, PermissionCache, PermissionService,
};
use deepwell::services::public_cache::PublicContentCache;
use deepwell::services::relation::{
    CreatePageWatch, CreateSiteMember, PageAttribution, PageAttributionKind,
    RelationObject, RelationReference, RemovePageWatch, SiteMemberAccepted,
    SiteMemberData,
};
use deepwell::services::render::{LegacyActionRegistry, UrlArgumentPair, UrlArguments};
use deepwell::services::role::{
    GrantUserRoleInput, InternalCreateRoleInput, RoleService, UpdateRolePermissionsInput,
};
use deepwell::services::score::ScoreValue as QueryScoreValue;
use deepwell::services::session::CreateSession;
use deepwell::services::site::UpdateSiteBody;
use deepwell::services::text_block::{MIME_HTML, TextBlock};
use deepwell::services::user::UpdateUserBody;
use deepwell::services::view::{GetArticleViewOutput, GetPageViewOutput};
use deepwell::services::{
    FileRevisionService, ForumPostService, ForumService, ForumThreadService, LinkService,
    PageService, RelationService, RenderService, RequestContext, ServiceContext,
    SessionService, SettingsService, SiteService, TextBlockService, TextService,
    ThemeSetting, UserService,
};
use deepwell::types::{
    Action, ConnectionType, Maybe, PageId, PageLockType, PageRevisionType, Permission,
    Reference, RelationType, RerenderDepth, Resource, TextBlockType,
};
use futures::FutureExt;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, Set, Statement,
    TransactionTrait, Value,
};
use serde_json::{Value as JsonValue, json};
use sha1::{Digest as Sha1Digest, Sha1};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::panic::{AssertUnwindSafe, resume_unwind};
use std::time::Duration as StdDuration;
use time::{Duration, OffsetDateTime};

use ftml::data::{KarmaLevel, PageInfo, ScoreValue};
use ftml::layout::Layout;
use ftml::settings::{WikitextMode, WikitextSettings};
use redis::AsyncCommands;

#[tokio::test]
async fn documented_ftml_owned_syntax_has_public_preview_regressions() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded mirror site should exist")
        .site;
    runner.set_request_context(RequestContext {
        site_id: Some(site.site_id),
        ..Default::default()
    });

    let cases = [
        (
            "bibliography",
            "((bibcite alpha))\n[[bibliography]]\n: alpha : Public preview bibliography marker\n[[/bibliography]]",
            "[[bibliography]]",
        ),
        (
            "block-formatting",
            "[[=]]\nPublic preview centered marker\n[[/=]]",
            "[[=]]",
        ),
        (
            "block-quotes",
            "> Public preview quote marker",
            "> Public preview quote marker",
        ),
        (
            "code-blocks",
            "[[code]]\nPublic preview code marker\n[[/code]]",
            "[[code]]",
        ),
        ("date", "[[date 1237135440 format=\"%e %b %Y\"]]", "[[date"),
        (
            "definition-lists",
            ": Public preview term : Public preview definition",
            ": Public preview term :",
        ),
        (
            "footnotes",
            "Public preview footnote marker[[footnote]]Public preview note body[[/footnote]]\n[[footnoteblock]]",
            "[[footnote]]",
        ),
        (
            "headings",
            "+ Public preview heading marker",
            "+ Public preview heading marker",
        ),
        (
            "horizontal-rules",
            "Public preview before rule\n----\nPublic preview after rule",
            "----",
        ),
        (
            "inline-formatting",
            "**Public preview bold marker**",
            "**Public preview bold marker**",
        ),
        (
            "lists",
            "* Public preview list marker",
            "* Public preview list marker",
        ),
        ("math", "[[math]]\nx^2\n[[/math]]", "[[math]]"),
        (
            "table-of-contents",
            "[[toc]]\n+ Public preview toc heading",
            "[[toc]]",
        ),
        (
            "tables",
            "|| Public preview table marker ||",
            "|| Public preview table marker ||",
        ),
        (
            "text-size",
            "[[size 150%]]Public preview size marker[[/size]]",
            "[[size 150%]]",
        ),
    ];

    for (label, wikitext, forbidden_literal) in cases {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site.site_id,
                "title": format!("Public syntax preview {label}"),
                "wikitext": wikitext,
            }),
        );
        assert!(
            !preview.body.contains(forbidden_literal),
            "{label} should be consumed by the public preview parser: {}",
            preview.body,
        );
    }

    let note = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site.site_id,
            "title": "Public syntax preview note",
            "wikitext": "[[note]]\nPublic preview note marker\n[[/note]]",
        }),
    );
    assert!(note.body.contains("[[note]]"));
    assert!(note.body.contains("[[/note]]"));

    let paragraphs = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site.site_id,
            "title": "Public syntax preview paragraphs",
            "wikitext": "Public preview paragraph one\n\nPublic preview paragraph two",
        }),
    );
    assert!(
        paragraphs
            .body
            .contains("<p>Public preview paragraph one</p>")
    );
    assert!(
        paragraphs
            .body
            .contains("<p>Public preview paragraph two</p>")
    );

    let typography = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site.site_id,
            "title": "Public syntax preview typography",
            "wikitext": "Public preview dots... and em -- dash",
        }),
    );
    assert!(!typography.body.contains("dots..."));
    assert!(!typography.body.contains("em -- dash"));

    let escaping = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site.site_id,
            "title": "Public syntax preview universal escaping",
            "wikitext": "@<Public preview umlaut: &#252;>@",
        }),
    );
    assert!(escaping.body.contains("Public preview umlaut: ü"));
    assert!(!escaping.body.contains("@<"));
}

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
async fn public_page_view_resolves_observed_url_aliases_to_the_canonical_page() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scpaiueouiuiuiui"}))
        .expect("editable local authoring site should exist")
        .site;
    let slug = format!("url-alias-{}", cuid());

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Slug(Cow::Owned(slug.clone())),
    );
    let created = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site_id,
            "wikitext": "URL alias target body",
            "title": "URL alias target",
            "alt_title": null,
            "slug": slug.clone(),
            "layout": "wikidot",
            "revision_comments": "create URL alias target",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    // Imported rows can contain a slug that aliases another page. An exact
    // stored slug must keep precedence over the alias target.
    let canonical_page = PageTable::find_by_id(created.page_id)
        .one(runner.context().transaction())
        .await
        .expect("canonical page lookup should not fail")
        .expect("canonical page should exist");
    let canonical_revision = PageRevisionTable::find_by_id(
        canonical_page
            .latest_revision_id
            .expect("canonical page should have a revision"),
    )
    .one(runner.context().transaction())
    .await
    .expect("canonical revision lookup should not fail")
    .expect("canonical page revision should exist");
    let collision_slug = slug.replacen('-', "+", 1);
    let collision_page = page::ActiveModel {
        created_at: Set(canonical_page.created_at),
        from_wikidot: Set(canonical_page.from_wikidot),
        site_id: Set(canonical_page.site_id),
        page_category_id: Set(canonical_page.page_category_id),
        slug: Set(collision_slug.clone()),
        discussion_thread_id: Set(canonical_page.discussion_thread_id),
        layout: Set(canonical_page.layout),
        ..Default::default()
    }
    .insert(runner.context().transaction())
    .await
    .expect("exact alias-shaped page should be inserted");
    let collision_revision = page_revision::ActiveModel {
        revision_type: Set(canonical_revision.revision_type),
        created_at: Set(canonical_revision.created_at),
        revision_number: Set(canonical_revision.revision_number),
        page_id: Set(collision_page.page_id),
        site_id: Set(canonical_revision.site_id),
        user_id: Set(canonical_revision.user_id),
        from_wikidot: Set(canonical_revision.from_wikidot),
        changes: Set(canonical_revision.changes),
        wikitext_hash: Set(canonical_revision.wikitext_hash),
        compiled_body_html_hash: Set(canonical_revision.compiled_body_html_hash),
        compiled_body_styles_hash: Set(canonical_revision.compiled_body_styles_hash),
        compiled_top_bar_html_hash: Set(canonical_revision.compiled_top_bar_html_hash),
        compiled_side_bar_html_hash: Set(canonical_revision.compiled_side_bar_html_hash),
        compiled_at: Set(canonical_revision.compiled_at),
        compiled_generator: Set(canonical_revision.compiled_generator),
        comments: Set("exact alias collision fixture".to_owned()),
        hidden: Set(canonical_revision.hidden),
        title: Set("Exact alias collision".to_owned()),
        alt_title: Set(canonical_revision.alt_title),
        slug: Set(collision_slug.clone()),
        tags: Set(canonical_revision.tags),
        ..Default::default()
    }
    .insert(runner.context().transaction())
    .await
    .expect("exact alias-shaped revision should be inserted");
    let mut collision_page = collision_page.into_active_model();
    collision_page.latest_revision_id = Set(Some(collision_revision.revision_id));
    collision_page
        .update(runner.context().transaction())
        .await
        .expect("exact alias-shaped page should reference its revision");

    runner.set_request_context(RequestContext {
        site_id: Some(site.site_id),
        ..Default::default()
    });
    let alias_name = slug.strip_prefix("url-").expect("test slug prefix");
    let aliases = [
        format!("URL-{alias_name}"),
        slug.replacen('-', "_", 1),
        slug.replacen('-', ".", 1),
        slug.replacen('-', "--", 1),
        format!("{slug}-"),
        format!("{slug}."),
        format!("{slug} "),
    ];
    for alias in aliases {
        let view = run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site.site_id,
                "session_token": null,
                "route": {"slug": alias, "extra": ""},
                "locales": ["en-US", "en"],
            }),
        );
        match view {
            GetPageViewOutput::Found {
                page,
                redirect_page,
                ..
            } => {
                assert_eq!(page.slug, slug);
                assert_eq!(redirect_page.as_deref(), Some(slug.as_str()));
            }
            other => panic!("expected canonical page for URL alias, got {other:?}"),
        }
    }

    let collision = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site.site_id,
            "session_token": null,
            "route": {"slug": collision_slug.clone(), "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let GetPageViewOutput::Found {
        page,
        redirect_page: None,
        ..
    } = collision
    else {
        panic!("an exact stored alias-shaped page should win without redirecting");
    };
    assert_eq!(page.slug, collision_slug);
}

async fn import_cacheable_page_attribution_fixture(
    runner: &mut TestRunner,
    site_id: i64,
    label: &str,
) -> (i64, String) {
    let run_id = cuid();
    let page_id = rand::random_range(1_700_000_000_i64..1_799_999_999_i64);
    let revision_id = rand::random_range(1_800_000_000_i64..1_899_999_999_i64);
    let slug = format!("page-attribution-cache-{label}-{run_id}");

    set_mutation_request_context(
        runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Owned(slug.clone())),
    );
    run_endpoint!(
        runner,
        import_wikidot_page,
        json!({
            "page_id": page_id,
            "site_id": site_id,
            "created_at": "1970-01-01T00:00:00Z",
            "slug": slug,
            "locked": false,
            "discussion_thread_id": null,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    run_endpoint!(
        runner,
        import_wikidot_page_revision,
        json!({
            "revision_id": revision_id,
            "revision_type": "create",
            "created_at": time::OffsetDateTime::UNIX_EPOCH,
            "updated_at": null,
            "revision_number": 0,
            "page_id": page_id,
            "site_id": site_id,
            "user_id": ADMIN_USER_ID,
            "wikitext": format!("Imported attribution cache {label} fixture."),
            "comments": format!("import page attribution cache {label} fixture"),
            "title": format!("Page attribution cache {label} fixture"),
            "slug": slug,
            "tags": [],
        }),
    );
    runner
        .context()
        .run_post_commit_actions()
        .await
        .expect("import post-commit actions should complete before cache assertions");

    (page_id, slug)
}

struct CachedAttributionArticle {
    attributions: Vec<PageAttribution>,
    cache_key: String,
    fence: String,
}

async fn load_cached_attribution_article(
    runner: &mut TestRunner,
    site_id: i64,
    slug: &str,
) -> CachedAttributionArticle {
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
    match run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": slug, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    ) {
        GetArticleViewOutput {
            page: GetPageViewOutput::Found { attributions, .. },
            article_page_cache_key: Some(cache_key),
            public_content_cache_fence: Some(fence),
            ..
        } => CachedAttributionArticle {
            attributions,
            cache_key,
            fence,
        },
        other => panic!("expected a cacheable imported article, got {other:?}"),
    }
}

async fn import_cacheable_gallery_fixture(
    runner: &mut TestRunner,
    site_id: i64,
) -> (i64, String) {
    let run_id = cuid();
    let page_id = rand::random_range(1_700_000_000_i64..1_799_999_999_i64);
    let revision_id = rand::random_range(1_800_000_000_i64..1_899_999_999_i64);
    let slug = format!("gallery-file-cache-{run_id}");

    set_mutation_request_context(
        runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Owned(slug.clone())),
    );
    run_endpoint!(
        runner,
        import_wikidot_page,
        json!({
            "page_id": page_id,
            "site_id": site_id,
            "created_at": "1970-01-01T00:00:00Z",
            "slug": slug,
            "locked": false,
            "discussion_thread_id": null,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    run_endpoint!(
        runner,
        import_wikidot_page_revision,
        json!({
            "revision_id": revision_id,
            "revision_type": "create",
            "created_at": time::OffsetDateTime::UNIX_EPOCH,
            "updated_at": null,
            "revision_number": 0,
            "page_id": page_id,
            "site_id": site_id,
            "user_id": ADMIN_USER_ID,
            "wikitext": "[[gallery]]",
            "comments": "import Gallery file-cache fixture",
            "title": "Gallery file-cache fixture",
            "slug": slug,
            "tags": [],
        }),
    );
    runner
        .context()
        .run_post_commit_actions()
        .await
        .expect("Gallery import post-commit actions should complete");

    (page_id, slug)
}

struct CachedGalleryArticle {
    body: String,
    cache_key: String,
    fence: String,
}

async fn load_cached_gallery_article(
    runner: &mut TestRunner,
    site_id: i64,
    slug: &str,
) -> CachedGalleryArticle {
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
    match run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": slug, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    ) {
        GetArticleViewOutput {
            page:
                GetPageViewOutput::Found {
                    compiled_body_html, ..
                },
            article_page_cache_key: Some(cache_key),
            public_content_cache_fence: Some(fence),
            ..
        } => CachedGalleryArticle {
            body: compiled_body_html,
            cache_key,
            fence,
        },
        other => panic!("expected a cacheable imported Gallery article, got {other:?}"),
    }
}

async fn complete_gallery_file_mutation(
    runner: &mut TestRunner,
    site_id: i64,
    page_id: i64,
    slug: &str,
    before: &CachedGalleryArticle,
) -> CachedGalleryArticle {
    let before_commit = load_cached_gallery_article(runner, site_id, slug).await;
    assert_eq!(before_commit.fence, before.fence);
    assert_eq!(before_commit.cache_key, before.cache_key);

    runner
        .context()
        .run_post_commit_actions()
        .await
        .expect("file mutation post-commit actions should complete");
    let before_worker = load_cached_gallery_article(runner, site_id, slug).await;
    assert_ne!(before_worker.fence, before.fence);
    assert_ne!(before_worker.cache_key, before.cache_key);
    assert_eq!(
        before_worker.body, before.body,
        "the transaction-local rerender simulation has not run yet",
    );

    rerender_file_fixture_page(runner, page_id).await;
    runner
        .context()
        .run_post_commit_actions()
        .await
        .expect("Gallery rerender post-commit actions should complete");
    load_cached_gallery_article(runner, site_id, slug).await
}

async fn set_page_rating_policy(
    runner: &TestRunner,
    category_id: i64,
    enabled: bool,
    permission: &str,
) {
    let category = PageCategoryTable::find_by_id(category_id)
        .one(runner.context().transaction())
        .await
        .expect("Rate policy category lookup should succeed")
        .expect("Rate policy category should exist");
    let mut category = category.into_active_model();
    category.rating_enabled = Set(Some(enabled));
    category.rating_permission = Set(Some(permission.to_owned()));
    category
        .update(runner.context().transaction())
        .await
        .expect("Rate policy category update should succeed");
}

#[tokio::test]
async fn component_css_edit_refreshes_only_the_recorded_dependent_page() {
    const SITE_SLUG: &str = "scpaiueouiuiuiui";
    const COMPONENT_SLUG: &str = "component:authoring-css-invalidation";
    const DEPENDENT_SLUG: &str = "authoring-css-dependent";
    const UNRELATED_SLUG: &str = "authoring-css-unrelated";
    const RED_CSS: &str = "[[module CSS]]\n.authoring-color { color: red; }\n[[/module]]";
    const BLUE_CSS: &str =
        "[[module CSS]]\n.authoring-color { color: blue; }\n[[/module]]";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("the editable local authoring site should exist")
        .site;
    let site_id = site.site_id;

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(COMPONENT_SLUG)),
    );
    let component = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": RED_CSS,
            "title": "Authoring CSS component",
            "alt_title": null,
            "slug": COMPONENT_SLUG,
            "layout": "wikidot",
            "revision_comments": "create authoring CSS component",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(DEPENDENT_SLUG)),
    );
    let dependent = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": format!("[[include {COMPONENT_SLUG}]]\nDependent body"),
            "title": "Authoring CSS dependent",
            "alt_title": null,
            "slug": DEPENDENT_SLUG,
            "layout": "wikidot",
            "revision_comments": "create CSS dependent",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(UNRELATED_SLUG)),
    );
    let unrelated = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "Unrelated body",
            "title": "Authoring CSS unrelated",
            "alt_title": null,
            "slug": UNRELATED_SLUG,
            "layout": "wikidot",
            "revision_comments": "create unrelated page",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let before_dependent = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": dependent.page_id,
            "details": {"compiled_html": true},
        }),
    )
    .expect("dependent page should exist before the component edit");
    let before_unrelated = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": unrelated.page_id,
            "details": {"compiled_html": true},
        }),
    )
    .expect("unrelated page should exist before the component edit");
    assert!(
        before_dependent
            .compiled_body_styles
            .as_ref()
            .is_some_and(|styles| styles
                .iter()
                .any(|style| style.contains("color: red"))),
        "the dependent page should initially carry the component's red CSS",
    );

    let dependencies = LinkService::get_to(
        runner.context(),
        component.page_id,
        Some(&[
            ConnectionType::IncludeMessy,
            ConnectionType::IncludeElements,
            ConnectionType::Component,
        ]),
    )
    .await
    .expect("component dependents should be readable");
    assert_eq!(
        dependencies
            .connections
            .iter()
            .filter(|connection| connection.from_page_id == dependent.page_id)
            .count(),
        1,
        "the component should record the dependent page exactly once",
    );
    assert!(
        dependencies
            .connections
            .iter()
            .all(|connection| connection.from_page_id != unrelated.page_id),
        "the unrelated page must not enter the component dependency graph",
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(component.page_id),
    );
    run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": component.page_id,
            "last_revision_id": component.revision_id,
            "revision_comments": "change authoring CSS to blue",
            "user_id": ADMIN_USER_ID,
            "wikitext": BLUE_CSS,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("component CSS edit should create a revision");

    let dependent_page =
        PageService::get(runner.context(), site_id, Reference::Id(dependent.page_id))
            .await
            .expect("dependent page identity should remain available");
    PageRevisionService::rerender(
        runner.context(),
        PageId::from_page_model(&dependent_page),
        RerenderDepth::default(),
        RerenderType::Full,
    )
    .await
    .expect("the queued dependent rerender should succeed");

    let served_dependent = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": DEPENDENT_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let GetArticleViewOutput {
        page:
            GetPageViewOutput::Found {
                compiled_body_styles: served_styles,
                ..
            },
        ..
    } = served_dependent
    else {
        panic!("the dependent article should be served after its queued rerender");
    };
    assert!(
        served_styles
            .iter()
            .any(|style| style.contains("color: blue")),
        "the dependent article's next public read should serve the updated CSS",
    );
    assert!(
        served_styles
            .iter()
            .all(|style| !style.contains("color: red")),
        "the dependent article's next public read must not reuse stale CSS",
    );

    let after_dependent = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": dependent.page_id,
            "details": {"compiled_html": true},
        }),
    )
    .expect("dependent page should remain readable after the component edit");
    let after_unrelated = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": unrelated.page_id,
            "details": {"compiled_html": true},
        }),
    )
    .expect("unrelated page should remain readable after the component edit");
    let after_styles = after_dependent
        .compiled_body_styles
        .as_ref()
        .expect("dependent compiled styles should be populated");
    assert!(
        after_styles
            .iter()
            .any(|style| style.contains("color: blue"))
    );
    assert!(
        after_styles
            .iter()
            .all(|style| !style.contains("color: red"))
    );
    assert_ne!(
        after_dependent.compiled_at, before_dependent.compiled_at,
        "the dependent revision should be recompiled",
    );
    assert_eq!(after_dependent.revision_id, before_dependent.revision_id);
    assert_eq!(
        after_unrelated.compiled_body_styles, before_unrelated.compiled_body_styles,
        "the unrelated page's compiled styles must remain untouched",
    );
    assert_eq!(
        after_unrelated.compiled_at, before_unrelated.compiled_at,
        "the unrelated page must not be recompiled",
    );
    assert_eq!(after_unrelated.revision_id, before_unrelated.revision_id);
}

#[tokio::test]
async fn component_css_include_visibility_and_lifecycle_matrix() {
    const SITE_SLUG: &str = "scpaiueouiuiuiui";
    const CROSS_SITE_SLUG: &str = "test";
    const PRIVATE_CATEGORY: &str = "privatea1061";
    const PRIVATE_COMPONENT_SLUG: &str = "privatea1061:component";
    const PRIVATE_DEPENDENT_SLUG: &str = "a1061-private-dependent";
    const CROSS_COMPONENT_SLUG: &str = "component:a1061-cross-site";
    const CROSS_DEPENDENT_SLUG: &str = "a1061-cross-dependent";
    const DELETED_COMPONENT_SLUG: &str = "component:a1061-deleted";
    const DELETED_DEPENDENT_SLUG: &str = "a1061-deleted-dependent";
    const PRIVATE_RED: &str =
        "[[module CSS]]\n.a1061-private { color: rgb(101, 1, 1); }\n[[/module]]";
    const PRIVATE_BLUE: &str =
        "[[module CSS]]\n.a1061-private { color: rgb(1, 1, 101); }\n[[/module]]";
    const CROSS_RED: &str =
        "[[module CSS]]\n.a1061-cross { color: rgb(102, 2, 2); }\n[[/module]]";
    const CROSS_BLUE: &str =
        "[[module CSS]]\n.a1061-cross { color: rgb(2, 2, 102); }\n[[/module]]";
    const DELETED_RED: &str =
        "[[module CSS]]\n.a1061-deleted { color: rgb(103, 3, 3); }\n[[/module]]";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("editable A1061 site should exist")
        .site;
    let cross_site = run_endpoint!(runner, site_get, json!({"site": CROSS_SITE_SLUG}))
        .expect("cross-site A1061 fixture site should exist")
        .site;

    make_page_mutation_test_category_for_user(
        &runner,
        site.site_id,
        PRIVATE_CATEGORY,
        ADMIN_USER_ID,
        &[Action::View, Action::Create, Action::Edit, Action::Delete],
        "a1061-private-admin",
    )
    .await;
    PermissionCache::invalidate_site(runner.context(), site.site_id)
        .await
        .expect("A1061 private permission cache should invalidate");

    let create = async |runner: &mut TestRunner,
                        site_id: i64,
                        slug: &str,
                        title: &str,
                        wikitext: &str| {
        set_mutation_request_context(
            runner,
            ADMIN_USER_ID,
            site_id,
            Reference::Slug(Cow::Owned(slug.to_owned())),
        );
        run_endpoint!(
            runner,
            page_create,
            json!({
                "site_id": site_id,
                "wikitext": wikitext,
                "title": title,
                "alt_title": null,
                "slug": slug,
                "layout": "wikidot",
                "revision_comments": "A1061 visibility and lifecycle fixture",
                "user_id": ADMIN_USER_ID,
                "ip_address": common::IP_ADDRESS,
            }),
        )
    };

    let private_component = create(
        &mut runner,
        site.site_id,
        PRIVATE_COMPONENT_SLUG,
        "A1061 private component",
        PRIVATE_RED,
    )
    .await;
    let private_dependent = create(
        &mut runner,
        site.site_id,
        PRIVATE_DEPENDENT_SLUG,
        "A1061 private dependent",
        &format!("[[include {PRIVATE_COMPONENT_SLUG}]]\nPrivate dependent"),
    )
    .await;
    let private_before = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site.site_id,
            "page": private_dependent.page_id,
            "details": {"compiled_html": true},
        }),
    )
    .expect("private dependent should exist");
    let private_before_styles = private_before
        .compiled_body_styles
        .as_ref()
        .expect("private dependent styles should be populated");
    assert!(
        private_before_styles
            .iter()
            .all(|style| !style.contains("a1061-private")),
        "a public dependent must not adopt CSS from an anonymously denied include",
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(private_component.page_id),
    );
    run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site.site_id,
            "page": private_component.page_id,
            "last_revision_id": private_component.revision_id,
            "revision_comments": "A1061 denied component edit",
            "user_id": ADMIN_USER_ID,
            "wikitext": PRIVATE_BLUE,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("private component edit should succeed for its authorized actor");
    let private_dependent_page = PageService::get(
        runner.context(),
        site.site_id,
        Reference::Id(private_dependent.page_id),
    )
    .await
    .expect("private dependent identity should remain available");
    PageRevisionService::rerender(
        runner.context(),
        PageId::from_page_model(&private_dependent_page),
        RerenderDepth::default(),
        RerenderType::Full,
    )
    .await
    .expect("private dependent rerender should succeed");
    let private_after = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site.site_id,
            "page": private_dependent.page_id,
            "details": {"compiled_html": true},
        }),
    )
    .expect("private dependent should remain readable");
    assert!(
        private_after
            .compiled_body_styles
            .as_ref()
            .is_some_and(|styles| styles
                .iter()
                .all(|style| !style.contains("a1061-private"))),
        "editing a denied component must not make its CSS visible or reuse a privileged compile",
    );

    let cross_component = create(
        &mut runner,
        cross_site.site_id,
        CROSS_COMPONENT_SLUG,
        "A1061 cross-site component",
        CROSS_RED,
    )
    .await;
    let cross_dependent = create(
        &mut runner,
        site.site_id,
        CROSS_DEPENDENT_SLUG,
        "A1061 cross-site dependent",
        &format!(
            "[[include :{CROSS_SITE_SLUG}:{CROSS_COMPONENT_SLUG}]]\nCross-site dependent"
        ),
    )
    .await;
    let cross_before = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site.site_id,
            "page": cross_dependent.page_id,
            "details": {"compiled_html": true},
        }),
    )
    .expect("cross-site dependent should exist");
    assert!(
        cross_before
            .compiled_body_styles
            .as_ref()
            .is_some_and(|styles| styles
                .iter()
                .any(|style| style.contains("rgb(102, 2, 2)"))),
        "an explicit public cross-site include should retain its documented CSS behavior",
    );
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        cross_site.site_id,
        Reference::Id(cross_component.page_id),
    );
    run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": cross_site.site_id,
            "page": cross_component.page_id,
            "last_revision_id": cross_component.revision_id,
            "revision_comments": "A1061 cross-site component edit",
            "user_id": ADMIN_USER_ID,
            "wikitext": CROSS_BLUE,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("cross-site component edit should succeed");
    let cross_dependent_page = PageService::get(
        runner.context(),
        site.site_id,
        Reference::Id(cross_dependent.page_id),
    )
    .await
    .expect("cross-site dependent identity should remain available");
    PageRevisionService::rerender(
        runner.context(),
        PageId::from_page_model(&cross_dependent_page),
        RerenderDepth::default(),
        RerenderType::Full,
    )
    .await
    .expect("cross-site dependent rerender should succeed");
    let cross_after = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site.site_id,
            "page": cross_dependent.page_id,
            "details": {"compiled_html": true},
        }),
    )
    .expect("cross-site dependent should remain readable");
    let cross_after_styles = cross_after
        .compiled_body_styles
        .as_ref()
        .expect("cross-site dependent styles should be populated");
    assert!(
        cross_after_styles
            .iter()
            .any(|style| style.contains("rgb(2, 2, 102)"))
            && cross_after_styles
                .iter()
                .all(|style| !style.contains("rgb(102, 2, 2)")),
        "an explicit cross-site include may update only through its public source without stale CSS",
    );

    let imported_external_page = create(
        &mut runner,
        site.site_id,
        "component:a1061-imported-external",
        "A1061 imported external component",
        "[[div class=\"a1061-imported-external\"]]\nImported cross-site snapshot marker\n[[/div]]",
    )
    .await;
    let import_run_row = runner
        .context()
        .transaction()
        .query_one_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            format!(
                "INSERT INTO wikidot_corpus_import_run (site_id, source_branch, source_site, manifest_sha256, manifest_row_count, complete_inventory, state, summary) VALUES ({}, 'source-bundle', 'a1061-remote', decode(repeat('00', 32), 'hex'), 1, FALSE, 'done', '{{}}'::jsonb) RETURNING import_run_id",
                site.site_id,
            ),
        ))
        .await
        .expect("external import run fixture should insert")
        .expect("external import run should return its ID");
    let import_run_id = import_run_row
        .try_get::<i64>("", "import_run_id")
        .expect("external import run ID should be readable");
    let source_entity_id = uuid::Uuid::new_v4();
    runner
        .context()
        .transaction()
        .execute_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            format!(
                "INSERT INTO wikidot_page_snapshot (page_id, source_branch, source_site, source_entity_id, source_fullname, source_created_at, source_updated_at, source_revision_count, imported_rating, comments, source_sha256, meta_sha256, meta_json, last_import_run_id) VALUES ({}, 'source-bundle', 'a1061-remote', '{source_entity_id}', 'component:a1061-imported-external', NOW(), NOW(), 1, 0, 0, decode(repeat('00', 32), 'hex'), decode(repeat('00', 32), 'hex'), '{{}}'::jsonb, {import_run_id})",
                imported_external_page.page_id,
            ),
        ))
        .await
        .expect("external source provenance should insert");
    let imported_external_dependent = create(
        &mut runner,
        site.site_id,
        "a1061-imported-external-dependent",
        "A1061 imported external dependent",
        "[[include :a1061-remote:component:a1061-imported-external]]\nDependent body",
    )
    .await;
    let imported_external_output = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site.site_id,
            "page": imported_external_dependent.page_id,
            "details": {"compiled_html": true},
        }),
    )
    .expect("the external-source dependent should remain readable");
    let imported_external_html = imported_external_output
        .compiled_body_html
        .as_deref()
        .expect("external-source dependent should have compiled HTML");
    assert!(
        imported_external_html.contains("Imported cross-site snapshot marker")
            && !imported_external_html.contains("does not exist"),
        "an explicit cross-site include should resolve its source-site snapshot from the local mirror",
    );

    let deleted_component = create(
        &mut runner,
        site.site_id,
        DELETED_COMPONENT_SLUG,
        "A1061 deleted component",
        DELETED_RED,
    )
    .await;
    let deleted_dependent = create(
        &mut runner,
        site.site_id,
        DELETED_DEPENDENT_SLUG,
        "A1061 deleted dependent",
        &format!("[[include {DELETED_COMPONENT_SLUG}]]\nDeleted dependent"),
    )
    .await;
    let deleted_before = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site.site_id,
            "page": deleted_dependent.page_id,
            "details": {"compiled_html": true},
        }),
    )
    .expect("deleted-source dependent should exist");
    assert!(
        deleted_before
            .compiled_body_styles
            .as_ref()
            .is_some_and(|styles| styles
                .iter()
                .any(|style| style.contains("a1061-deleted"))),
        "deleted-source fixture should begin with the component CSS",
    );
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(deleted_component.page_id),
    );
    run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site.site_id,
            "page": deleted_component.page_id,
            "last_revision_id": deleted_component.revision_id,
            "revision_comments": "A1061 deleted component",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let deleted_dependent_page = PageService::get(
        runner.context(),
        site.site_id,
        Reference::Id(deleted_dependent.page_id),
    )
    .await
    .expect("deleted-source dependent identity should remain available");
    PageRevisionService::rerender(
        runner.context(),
        PageId::from_page_model(&deleted_dependent_page),
        RerenderDepth::default(),
        RerenderType::Full,
    )
    .await
    .expect("deleted-source dependent rerender should succeed");
    let deleted_after = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site.site_id,
            "page": deleted_dependent.page_id,
            "details": {"compiled_html": true},
        }),
    )
    .expect("deleted-source dependent should remain readable");
    assert!(
        deleted_after
            .compiled_body_styles
            .as_ref()
            .is_some_and(|styles| styles
                .iter()
                .all(|style| !style.contains("a1061-deleted"))),
        "a deleted component must not leave stale CSS in a rerendered dependent",
    );
}

#[tokio::test]
async fn revision_diff_returns_typed_lines_without_exposing_hidden_source() {
    const SITE_SLUG: &str = "scpaiueouiuiuiui";
    const PAGE_SLUG: &str = "authoring-revision-diff";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("the editable local authoring site should exist")
        .site;
    let site_id = site.site_id;
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(PAGE_SLUG)),
    );
    let created = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "alpha\nold <script>alert(1)</script>\nomega",
            "title": "Authoring revision diff",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create revision diff fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let edited = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": created.page_id,
            "last_revision_id": created.revision_id,
            "revision_comments": "edit revision diff fixture",
            "user_id": ADMIN_USER_ID,
            "wikitext": "alpha\nnew & safe\nomega",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("revision diff fixture edit should create a revision");

    let diff = run_endpoint!(
        runner,
        page_revision_diff,
        json!({
            "site_id": site_id,
            "page_id": created.page_id,
            "from_revision_number": 0,
            "to_revision_number": 1,
        }),
    )
    .expect("two visible revisions should produce a diff");
    assert_eq!(
        serde_json::to_value(diff).expect("revision diff should serialize"),
        json!({
            "site_id": site_id,
            "page_id": created.page_id,
            "from_revision_number": 0,
            "to_revision_number": 1,
            "lines": [
                {"kind": "unchanged", "text": "alpha"},
                {"kind": "removed", "text": "old <script>alert(1)</script>"},
                {"kind": "added", "text": "new & safe"},
                {"kind": "unchanged", "text": "omega"},
            ],
        }),
    );

    let missing = run_endpoint!(
        runner,
        page_revision_diff,
        json!({
            "site_id": site_id,
            "page_id": created.page_id,
            "from_revision_number": 0,
            "to_revision_number": 99,
        }),
    );
    assert!(
        missing.is_none(),
        "a missing revision must not widen to another source"
    );

    run_endpoint!(
        runner,
        page_revision_edit,
        json!({
            "site_id": site_id,
            "page_id": created.page_id,
            "revision_id": created.revision_id,
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
            "hidden": ["wikitext"],
        }),
    );
    let hidden = run_endpoint!(
        runner,
        page_revision_diff,
        json!({
            "site_id": site_id,
            "page_id": created.page_id,
            "from_revision_number": 0,
            "to_revision_number": edited.revision_number,
        }),
    );
    assert!(
        hidden.is_none(),
        "a hidden source revision must make the entire pair unavailable",
    );
}

#[tokio::test]
async fn page_revision_visibility_accepts_only_known_fields_and_fails_closed() {
    const SITE_SLUG: &str = "scpaiueouiuiuiui";
    const PAGE_SLUG: &str = "authoring-revision-hidden-fields";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("the editable local authoring site should exist")
        .site;
    let site_id = site.site_id;
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(PAGE_SLUG)),
    );
    let created = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "first revision body",
            "title": "First revision title",
            "alt_title": "First revision alternate title",
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "first revision comments",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": created.page_id,
            "last_revision_id": created.revision_id,
            "revision_comments": "create a newer revision",
            "user_id": ADMIN_USER_ID,
            "wikitext": "second revision body",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("a newer revision should be created");

    let hidden = [
        "wikitext",
        "compiled",
        "comments",
        "title",
        "alt_title",
        "slug",
        "tags",
    ];
    let revision = run_endpoint!(
        runner,
        page_revision_edit,
        json!({
            "site_id": site_id,
            "page_id": created.page_id,
            "revision_id": created.revision_id,
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
            "hidden": hidden,
            "details": {"wikitext": true, "compiled_html": true},
        }),
    );
    assert_eq!(revision.hidden, hidden.map(str::to_owned));
    assert!(revision.wikitext.is_none());
    assert!(revision.compiled_body_html.is_none());
    assert!(revision.compiled_body_styles.is_none());
    assert!(revision.compiled_top_bar_html.is_none());
    assert!(revision.compiled_side_bar_html.is_none());
    assert!(revision.comments.is_none());
    assert!(revision.title.is_none());
    assert!(revision.alt_title.is_none());
    assert!(revision.slug.is_none());
    assert!(revision.tags.is_none());

    let persisted = run_endpoint!(
        runner,
        page_revision_get,
        json!({
            "site_id": site_id,
            "page_id": created.page_id,
            "revision_number": 0,
            "details": {"wikitext": true, "compiled_html": true},
        }),
    )
    .expect("the moderated revision should remain readable");
    assert_eq!(persisted.hidden, hidden.map(str::to_owned));
    assert!(persisted.wikitext.is_none());
    assert!(persisted.compiled_body_html.is_none());

    let stored = PageRevisionTable::find_by_id(created.revision_id)
        .one(runner.context().transaction())
        .await
        .expect("the revision lookup should succeed")
        .expect("the revision should exist");
    let hidden_before_rejection = stored.hidden.clone();
    let updated_at_before_rejection = stored.updated_at;
    let error = run_endpoint_err!(
        runner,
        page_revision_edit,
        json!({
            "site_id": site_id,
            "page_id": created.page_id,
            "revision_id": created.revision_id,
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
            "hidden": ["comments", "Comments"],
        }),
    );
    assert_contains_error!(error, ErrorType::PageRevision);
    let stored = PageRevisionTable::find_by_id(created.revision_id)
        .one(runner.context().transaction())
        .await
        .expect("the revision lookup after rejection should succeed")
        .expect("the revision should still exist");
    assert_eq!(stored.hidden, hidden_before_rejection);
    assert_eq!(stored.updated_at, updated_at_before_rejection);

    let wrong_site = run_endpoint_err!(
        runner,
        page_revision_edit,
        json!({
            "site_id": site_id + 1,
            "page_id": created.page_id,
            "revision_id": created.revision_id,
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
            "hidden": ["unknown"],
        }),
    );
    assert_contains_error!(wrong_site, ErrorType::PageRevision);
    let stored = PageRevisionTable::find_by_id(created.revision_id)
        .one(runner.context().transaction())
        .await
        .expect("the revision lookup after wrong-site rejection should succeed")
        .expect("the revision should still exist");
    assert_eq!(stored.hidden, hidden_before_rejection);
    assert_eq!(stored.updated_at, updated_at_before_rejection);
}

#[tokio::test]
async fn page_revision_visibility_changes_are_audited_once_with_attribution() {
    const SITE_SLUG: &str = "scpaiueouiuiuiui";
    const PAGE_SLUG: &str = "authoring-revision-visibility-audit";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("the editable local authoring site should exist")
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
            "wikitext": "revision body",
            "title": "Revision visibility audit",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "comments to moderate",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let update = json!({
        "site_id": site.site_id,
        "page_id": created.page_id,
        "revision_id": created.revision_id,
        "user_id": ADMIN_USER_ID,
        "ip_address": common::IP_ADDRESS,
        "hidden": ["comments"],
    });
    let revision = run_endpoint!(runner, page_revision_edit, update.clone());
    assert_eq!(revision.hidden, vec![String::from("comments")]);

    let events = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("page_revision.update_visibility"))
        .filter(AuditLogColumn::ExtraId1.eq(created.revision_id))
        .all(runner.context().transaction())
        .await
        .expect("the page revision visibility audit lookup should succeed");
    assert_eq!(events.len(), 1, "the changed visibility must be audited");
    let event = &events[0];
    assert_eq!(event.site_id, Some(site.site_id));
    assert_eq!(event.page_id, Some(created.page_id));
    assert_eq!(event.extra_id_1, Some(created.revision_id));
    assert_eq!(event.user_id, Some(ADMIN_USER_ID));
    assert_eq!(event.ip_address, common::IP_ADDRESS.to_string());

    let stored = PageRevisionTable::find_by_id(created.revision_id)
        .one(runner.context().transaction())
        .await
        .expect("the updated revision lookup should succeed")
        .expect("the updated revision should exist");
    let updated_at = stored.updated_at;

    run_endpoint!(runner, page_revision_edit, update);

    let event_count = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("page_revision.update_visibility"))
        .filter(AuditLogColumn::ExtraId1.eq(created.revision_id))
        .count(runner.context().transaction())
        .await
        .expect("the page revision visibility audit count should succeed");
    assert_eq!(event_count, 1, "an unchanged hidden set is a true no-op");
    let stored = PageRevisionTable::find_by_id(created.revision_id)
        .one(runner.context().transaction())
        .await
        .expect("the no-op revision lookup should succeed")
        .expect("the no-op revision should still exist");
    assert_eq!(
        stored.updated_at, updated_at,
        "a no-op must not rewrite the row"
    );
}

#[tokio::test]
async fn page_revision_visibility_update_and_audit_roll_back_atomically() {
    const SITE_SLUG: &str = "scpaiueouiuiuiui";
    const PAGE_SLUG: &str = "authoring-revision-visibility-audit-rollback";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("the editable local authoring site should exist")
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
            "wikitext": "revision body",
            "title": "Revision visibility audit rollback",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "comments to moderate",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let transaction = runner
        .context()
        .transaction()
        .begin()
        .await
        .expect("the visibility update savepoint should begin");
    let ctx =
        ServiceContext::new(runner.state(), &transaction).with_request(RequestContext {
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site.site_id),
            page_reference: Some(Reference::Id(created.page_id)),
            ..Default::default()
        });
    deepwell::endpoints::all::page_revision_edit(
        &ctx,
        common::make_params(json!({
            "site_id": site.site_id,
            "page_id": created.page_id,
            "revision_id": created.revision_id,
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
            "hidden": ["comments"],
        })),
    )
    .await
    .expect("the transactional visibility update should succeed");

    let audit_count = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("page_revision.update_visibility"))
        .filter(AuditLogColumn::ExtraId1.eq(created.revision_id))
        .count(&transaction)
        .await
        .expect("the transactional visibility audit count should succeed");
    assert_eq!(audit_count, 1);
    transaction
        .rollback()
        .await
        .expect("the visibility update savepoint should roll back");

    let stored = PageRevisionTable::find_by_id(created.revision_id)
        .one(runner.context().transaction())
        .await
        .expect("the rolled-back revision lookup should succeed")
        .expect("the rolled-back revision should still exist");
    assert!(
        stored.hidden.is_empty(),
        "the revision update must roll back"
    );
    let audit_count = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("page_revision.update_visibility"))
        .filter(AuditLogColumn::ExtraId1.eq(created.revision_id))
        .count(runner.context().transaction())
        .await
        .expect("the rolled-back visibility audit count should succeed");
    assert_eq!(
        audit_count, 0,
        "the audit event must roll back with the update"
    );
}

#[tokio::test]
async fn page_revision_reads_reject_legacy_unknown_hidden_fields_without_panicking() {
    const SITE_SLUG: &str = "scpaiueouiuiuiui";
    const PAGE_SLUG: &str = "authoring-revision-legacy-hidden-field";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("the editable local authoring site should exist")
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
            "wikitext": "legacy malformed revision",
            "title": "Legacy malformed revision",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "legacy malformed revision",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let revision = PageRevisionTable::find_by_id(created.revision_id)
        .one(runner.context().transaction())
        .await
        .expect("the revision lookup should succeed")
        .expect("the revision should exist");
    let mut revision = revision.into_active_model();
    revision.hidden = Set(vec!["legacy_unknown".to_owned()]);
    revision
        .update(runner.context().transaction())
        .await
        .expect("the malformed legacy fixture should be constructed");

    let error = run_endpoint_err!(
        runner,
        page_revision_get,
        json!({
            "site_id": site.site_id,
            "page_id": created.page_id,
            "revision_number": 0,
            "details": {"wikitext": true, "compiled_html": true},
        }),
    );
    assert_contains_error!(error, ErrorType::PageRevision);
}

#[tokio::test]
async fn latest_page_revision_still_cannot_hide_wikitext() {
    const SITE_SLUG: &str = "scpaiueouiuiuiui";
    const PAGE_SLUG: &str = "authoring-latest-revision-wikitext";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("the editable local authoring site should exist")
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
            "wikitext": "latest revision source remains visible",
            "title": "Latest revision source",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "latest revision source",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let before = run_endpoint!(
        runner,
        page_revision_get,
        json!({
            "site_id": site.site_id,
            "page_id": created.page_id,
            "revision_number": 0,
            "details": {"wikitext": true},
        }),
    )
    .expect("the latest revision should initially be readable");
    let error = run_endpoint_err!(
        runner,
        page_revision_edit,
        json!({
            "site_id": site.site_id,
            "page_id": created.page_id,
            "revision_id": created.revision_id,
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
            "hidden": ["wikitext"],
        }),
    );
    assert_contains_error!(error, ErrorType::CannotHideLatestRevision);
    assert_eq!(ErrorType::CannotHideLatestRevision.code(), 4302);

    let revision = run_endpoint!(
        runner,
        page_revision_get,
        json!({
            "site_id": site.site_id,
            "page_id": created.page_id,
            "revision_number": 0,
            "details": {"wikitext": true},
        }),
    )
    .expect("the latest revision should remain readable");
    assert!(revision.hidden.is_empty());
    assert_eq!(revision.updated_at, before.updated_at);
    assert_eq!(
        revision.wikitext.as_deref(),
        Some("latest revision source remains visible")
    );
}

#[tokio::test]
async fn normal_page_create_allocates_category_numbers_without_consuming_conflicts() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site;
    let category =
        CategoryService::get_or_create(runner.context(), site.site_id, "issue")
            .await
            .expect("issue category should exist");
    CategoryService::update(
        runner.context(),
        site.site_id,
        Reference::Id(category.category_id),
        deepwell::services::category::UpdateCategoryBody {
            autonumber_enabled: Maybe::Set(true),
            ..Default::default()
        },
        Some(category.settings_revision),
        SYSTEM_USER_ID,
        common::IP_ADDRESS,
    )
    .await
    .expect("autonumber should be enabled");

    let explicit_category =
        CategoryService::get_or_create(runner.context(), site.site_id, "notes")
            .await
            .expect("notes category should exist");
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Slug(Cow::Borrowed("notes:explicit")),
    );
    let explicit = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site_id,
            "wikitext": "Explicit page",
            "title": "Explicit page",
            "alt_title": null,
            "slug": "notes:explicit",
            "layout": "wikidot",
            "revision_comments": "create explicit page",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(explicit.slug, "notes:explicit");
    let explicit_category = CategoryService::get(
        runner.context(),
        site.site_id,
        Reference::Id(explicit_category.category_id),
    )
    .await
    .expect("notes category should remain available");
    assert_eq!(explicit_category.autonumber_next, 1);
    let themed_category = CategoryService::update(
        runner.context(),
        site.site_id,
        Reference::Id(explicit_category.category_id),
        deepwell::services::category::UpdateCategoryBody {
            theme: Maybe::Set(ThemeSetting::External {
                url: String::from("https://themes.example/notes.css"),
            }),
            ..Default::default()
        },
        Some(explicit_category.settings_revision),
        SYSTEM_USER_ID,
        common::IP_ADDRESS,
    )
    .await
    .expect("external theme should be stored");
    assert_eq!(themed_category.settings_revision, 1);
    let stale_theme = CategoryService::update(
        runner.context(),
        site.site_id,
        Reference::Id(themed_category.category_id),
        deepwell::services::category::UpdateCategoryBody {
            theme: Maybe::Set(ThemeSetting::BuiltIn { id: 1 }),
            ..Default::default()
        },
        Some(0),
        SYSTEM_USER_ID,
        common::IP_ADDRESS,
    )
    .await
    .expect_err("stale category revision should be rejected");
    assert_contains_error!(stale_theme, ErrorType::BadRequest);
    let themed = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site.site_id,
            "session_token": null,
            "route": { "slug": "notes:explicit", "extra": "" },
            "locales": ["en"],
        }),
    );
    assert!(matches!(
        themed.page,
        GetPageViewOutput::Found {
            theme: ThemeSetting::External { ref url },
            ..
        } if url == "https://themes.example/notes.css"
    ));

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Slug(Cow::Borrowed("issue:new-issue")),
    );
    let first = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site_id,
            "wikitext": "First numbered issue",
            "title": "First numbered issue",
            "alt_title": null,
            "slug": "issue:new-issue",
            "layout": "wikidot",
            "revision_comments": "create first numbered issue",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(first.slug, "issue:1");

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Slug(Cow::Borrowed("issue:another-suggestion")),
    );
    let second = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site_id,
            "wikitext": "Second numbered issue",
            "title": "Second numbered issue",
            "alt_title": null,
            "slug": "issue:another-suggestion",
            "layout": "wikidot",
            "revision_comments": "create second numbered issue",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(second.slug, "issue:2");

    let default_category =
        CategoryService::get_or_create(runner.context(), site.site_id, "_default")
            .await
            .expect("default category should exist");
    CategoryService::update(
        runner.context(),
        site.site_id,
        Reference::Id(default_category.category_id),
        deepwell::services::category::UpdateCategoryBody {
            theme: Maybe::Set(ThemeSetting::External {
                url: String::from("https://themes.example/default.css"),
            }),
            ..Default::default()
        },
        Some(default_category.settings_revision),
        SYSTEM_USER_ID,
        common::IP_ADDRESS,
    )
    .await
    .expect("default category theme should be stored");
    let inherited = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site.site_id,
            "session_token": null,
            "route": { "slug": "issue:1", "extra": "" },
            "locales": ["en"],
        }),
    );
    assert!(matches!(
        inherited.page,
        GetPageViewOutput::Found {
            theme: ThemeSetting::External { ref url },
            ..
        } if url == "https://themes.example/default.css"
    ));

    let current = CategoryService::get(
        runner.context(),
        site.site_id,
        Reference::Id(category.category_id),
    )
    .await
    .expect("issue category should remain available");
    assert_eq!(current.autonumber_next, 3);

    let mut stale_allocator = current.into_active_model();
    stale_allocator.autonumber_next = Set(2);
    stale_allocator
        .update(runner.context().transaction())
        .await
        .expect("stale allocator fixture should be installed");
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Slug(Cow::Borrowed("issue:conflicting-suggestion")),
    );
    let error = run_endpoint_err!(
        runner,
        page_create,
        json!({
            "site_id": site.site_id,
            "wikitext": "Conflicting numbered issue",
            "title": "Conflicting numbered issue",
            "alt_title": null,
            "slug": "issue:conflicting-suggestion",
            "layout": "wikidot",
            "revision_comments": "attempt conflicting numbered issue",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PageExists);
    let after_conflict = CategoryService::get(
        runner.context(),
        site.site_id,
        Reference::Id(category.category_id),
    )
    .await
    .expect("issue category should remain available");
    assert_eq!(after_conflict.autonumber_next, 2);
}

#[tokio::test]
async fn normal_page_creates_serialize_conflict_checks_on_category_settings() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site;
    let state = runner.state().clone();
    runner.teardown().await;

    let first_transaction = state
        .database
        .begin()
        .await
        .expect("first page-create transaction should start");
    let first_context = ServiceContext::new(&state, &first_transaction);
    PageService::create(
        &first_context,
        CreatePage {
            site_id: site.site_id,
            wikitext: String::from("First concurrent normal page"),
            title: String::from("First concurrent normal page"),
            alt_title: None,
            tags: Vec::new(),
            slug: format!("normal-concurrency-first-{}", cuid()),
            layout: Some(Layout::Wikidot),
            revision_comments: String::from("exercise normal page concurrency"),
            user_id: ADMIN_USER_ID,
            bypass_filter: true,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("first normal page should be created");

    let second_transaction = state
        .database
        .begin()
        .await
        .expect("second page-create transaction should start");
    let second_context = ServiceContext::new(&state, &second_transaction);
    let mut second_create = Box::pin(PageService::create(
        &second_context,
        CreatePage {
            site_id: site.site_id,
            wikitext: String::from("Second concurrent normal page"),
            title: String::from("Second concurrent normal page"),
            alt_title: None,
            tags: Vec::new(),
            slug: format!("normal-concurrency-second-{}", cuid()),
            layout: Some(Layout::Wikidot),
            revision_comments: String::from("exercise normal page concurrency"),
            user_id: ADMIN_USER_ID,
            bypass_filter: true,
            ip_address: common::IP_ADDRESS,
        },
    ));
    assert!(
        tokio::time::timeout(StdDuration::from_millis(100), second_create.as_mut())
            .await
            .is_err(),
        "a normal page create must wait for the category conflict lock",
    );

    drop(first_context);
    first_transaction
        .rollback()
        .await
        .expect("first page-create transaction should roll back");
    tokio::time::timeout(StdDuration::from_secs(2), second_create.as_mut())
        .await
        .expect("second normal page create should proceed after the category lock is released")
        .expect("second normal page should be created");
    drop(second_create);
    drop(second_context);
    second_transaction
        .rollback()
        .await
        .expect("second page-create transaction should roll back");
}

#[tokio::test]
async fn autonumber_allocator_holds_the_category_lock_until_request_completion() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site;
    let category =
        CategoryService::get(runner.context(), site.site_id, Reference::from("_default"))
            .await
            .expect("seeded default category should exist");
    let state = runner.state().clone();

    let first_transaction = state
        .database
        .begin()
        .await
        .expect("first allocator transaction should start");
    let first_context = ServiceContext::new(&state, &first_transaction);
    CategoryService::get_for_update(
        &first_context,
        site.site_id,
        Reference::Id(category.category_id),
    )
    .await
    .expect("first allocator should lock the category");

    let second_transaction = state
        .database
        .begin()
        .await
        .expect("competing allocator transaction should start");
    let second_context = ServiceContext::new(&state, &second_transaction);
    let mut second_lock = Box::pin(CategoryService::get_for_update(
        &second_context,
        site.site_id,
        Reference::Id(category.category_id),
    ));
    assert!(
        tokio::time::timeout(StdDuration::from_millis(100), second_lock.as_mut())
            .await
            .is_err(),
        "a concurrent page create must wait for the category allocator lock",
    );

    drop(first_context);
    first_transaction
        .rollback()
        .await
        .expect("first allocator transaction should roll back");
    let locked_category =
        tokio::time::timeout(StdDuration::from_secs(2), second_lock.as_mut())
            .await
            .expect("competing allocator should proceed after the first request ends")
            .expect("competing allocator lock should succeed");
    assert_eq!(locked_category.category_id, category.category_id);
    drop(second_lock);
    drop(second_context);
    second_transaction
        .rollback()
        .await
        .expect("competing allocator transaction should roll back");
}

#[tokio::test]
async fn failed_page_create_rolls_back_its_allocated_number() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site;
    let original =
        CategoryService::get(runner.context(), site.site_id, Reference::from("_default"))
            .await
            .expect("seeded default category should exist");
    let state = runner.state().clone();

    let transaction = state
        .database
        .begin()
        .await
        .expect("failed-create transaction should start");
    let context = ServiceContext::new(&state, &transaction);
    let category =
        CategoryService::get(&context, site.site_id, Reference::Id(original.category_id))
            .await
            .expect("default category should be available in the request transaction");
    let mut category = category.into_active_model();
    category.autonumber_enabled = Set(true);
    category.autonumber_next = Set(8_000_000_000_000_000_000);
    category
        .update(&transaction)
        .await
        .expect("autonumber failure fixture should be installed");

    PageService::create(
        &context,
        CreatePage {
            site_id: site.site_id,
            wikitext: String::from("This request must roll back."),
            title: String::from("Failed autonumber request"),
            alt_title: None,
            tags: Vec::new(),
            slug: String::from("suggested-page-name"),
            layout: Some(Layout::Wikidot),
            revision_comments: String::from("exercise failed allocator rollback"),
            user_id: i64::MAX,
            bypass_filter: true,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect_err("the unknown revision author should fail after number allocation");

    drop(context);
    transaction
        .rollback()
        .await
        .expect("failed page-create transaction should roll back");

    let assertion_transaction = state
        .database
        .begin()
        .await
        .expect("allocator assertion transaction should start");
    let assertion_context = ServiceContext::new(&state, &assertion_transaction);
    let after_failure = CategoryService::get(
        &assertion_context,
        site.site_id,
        Reference::Id(original.category_id),
    )
    .await
    .expect("default category should remain available after rollback");
    assert_eq!(
        after_failure.autonumber_enabled,
        original.autonumber_enabled
    );
    assert_eq!(after_failure.autonumber_next, original.autonumber_next);
    drop(assertion_context);
    assertion_transaction
        .rollback()
        .await
        .expect("allocator assertion transaction should roll back");
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
async fn imported_redirect_noredirect_renders_live_error_block() {
    const SITE_SLUG: &str = "test";
    const SOURCE_SLUG: &str = "redirect-noredirect-live";
    const TARGET_SLUG: &str = "redirect-target-live";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("seeded test site should exist")
        .site;

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Slug(Cow::Borrowed(TARGET_SLUG)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site_id,
            "wikitext": "Redirect target",
            "title": "Redirect target",
            "alt_title": null,
            "slug": TARGET_SLUG,
            "layout": "wikidot",
            "revision_comments": "create redirect target",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Slug(Cow::Borrowed(SOURCE_SLUG)),
    );
    let created = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site_id,
            "wikitext": format!("[[module Redirect destination=\"{TARGET_SLUG}\"]]"),
            "title": "Redirect source",
            "alt_title": null,
            "slug": SOURCE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create imported redirect source",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let page = PageTable::find_by_id(created.page_id)
        .one(runner.context().transaction())
        .await
        .expect("redirect source page lookup should not fail")
        .expect("redirect source page should exist");
    let mut page = page.into_active_model();
    page.from_wikidot = Set(true);
    page.update(runner.context().transaction())
        .await
        .expect("redirect source page should be marked imported");

    let revision = PageRevisionTable::find_by_id(created.revision_id)
        .one(runner.context().transaction())
        .await
        .expect("redirect source revision lookup should not fail")
        .expect("redirect source revision should exist");
    let mut revision = revision.into_active_model();
    revision.from_wikidot = Set(true);
    revision
        .update(runner.context().transaction())
        .await
        .expect("redirect source revision should be marked imported");

    let redirected = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site.site_id,
            "session_token": null,
            "route": {"slug": SOURCE_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let GetArticleViewOutput {
        page:
            GetPageViewOutput::Found {
                redirect_page: Some(redirect_page),
                redirect_kind: Some(_),
                ..
            },
        ..
    } = redirected
    else {
        panic!("bare imported Redirect page should return Wikidot-module redirect");
    };
    assert_eq!(redirect_page, format!("/{TARGET_SLUG}"));

    let suppressed = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site.site_id,
            "session_token": null,
            "route": {"slug": SOURCE_SLUG, "extra": "noredirect/true"},
            "locales": ["en-US", "en"],
        }),
    );
    let GetArticleViewOutput {
        page:
            GetPageViewOutput::Found {
                redirect_page: None,
                redirect_kind: None,
                compiled_body_html,
                ..
            },
        ..
    } = suppressed
    else {
        panic!("noredirect page view should render the imported redirect source");
    };
    assert!(
        compiled_body_html.contains(r#"<div class="error-block">"#),
        "noredirect view should render Wikidot's error-block notice: {compiled_body_html}",
    );
    assert!(
        compiled_body_html.contains(&format!(
            "This is the Redirect module that redirects the browser directly to the &quot;{TARGET_SLUG}&quot; page."
        )),
        "noredirect view should identify the redirect destination like live Wikidot: {compiled_body_html}",
    );
    assert!(
        !compiled_body_html.contains("[[module Redirect"),
        "noredirect view must not expose the raw Redirect module source: {compiled_body_html}",
    );
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
async fn initial_page_creation_uses_destination_category_layout() {
    const SITE_SLUG: &str = "test";
    const CATEGORY_SLUG: &str = "category-layout-create";
    const PAGE_SLUG: &str = "category-layout-create:page";
    const LAYOUT_MARKER: &str = "destination category layout";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        site_update,
        json!({
            "site": site_id,
            "user_id": ADMIN_USER_ID,
            "expected_settings_revision": site.settings.revision,
            "layout": "wikidot",
            "ip_address": common::IP_ADDRESS,
        }),
    );

    // Fixture-only setup: category layout has no registered update endpoint.
    // This immutable precondition must not be used to observe create behavior.
    let category =
        CategoryService::get_or_create(runner.context(), site_id, CATEGORY_SLUG)
            .await
            .expect("destination category fixture should exist");
    let mut category = category.into_active_model();
    category.layout = Set(Some("wikijump".to_owned()));
    category
        .update(runner.context().transaction())
        .await
        .expect("destination category layout fixture should update");

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(PAGE_SLUG)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": format!("{{{{{LAYOUT_MARKER}}}}}"),
            "title": "Category layout creation fixture",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": null,
            "revision_comments": "create category layout fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": PAGE_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let compiled_body_html = match view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found anonymous page view, got {other:?}"),
    };

    assert_eq!(
        compiled_body_html,
        format!(r#"<p><code class="wj-monospace">{LAYOUT_MARKER}</code></p>"#),
        "the stored public body should use the destination category layout",
    );
}

#[tokio::test]
async fn initial_page_creation_explicit_layout_beats_destination_category_layout() {
    const SITE_SLUG: &str = "test";
    const CATEGORY_SLUG: &str = "explicit-layout-create";
    const PAGE_SLUG: &str = "explicit-layout-create:page";
    const LAYOUT_MARKER: &str = "explicit page layout";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        site_update,
        json!({
            "site": site_id,
            "user_id": ADMIN_USER_ID,
            "expected_settings_revision": site.settings.revision,
            "layout": "wikijump",
            "ip_address": common::IP_ADDRESS,
        }),
    );

    // Fixture-only setup: category layout has no registered update endpoint.
    // This immutable precondition must not be used to observe create behavior.
    let category =
        CategoryService::get_or_create(runner.context(), site_id, CATEGORY_SLUG)
            .await
            .expect("destination category fixture should exist");
    let mut category = category.into_active_model();
    category.layout = Set(Some("wikijump".to_owned()));
    category
        .update(runner.context().transaction())
        .await
        .expect("destination category layout fixture should update");

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(PAGE_SLUG)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": format!("{{{{{LAYOUT_MARKER}}}}}"),
            "title": "Explicit page layout creation fixture",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create explicit layout fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": PAGE_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let compiled_body_html = match view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found anonymous page view, got {other:?}"),
    };

    assert_eq!(
        compiled_body_html,
        format!(r#"<p><tt>{LAYOUT_MARKER}</tt></p>"#),
        "the stored public body should use the explicit page layout",
    );
}

#[tokio::test]
async fn initial_page_creation_without_category_override_uses_site_layout() {
    const SITE_SLUG: &str = "test";
    const PAGE_SLUG: &str = "site-layout-create:page";
    const LAYOUT_MARKER: &str = "site fallback layout";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        site_update,
        json!({
            "site": site_id,
            "user_id": ADMIN_USER_ID,
            "expected_settings_revision": site.settings.revision,
            "layout": "wikidot",
            "ip_address": common::IP_ADDRESS,
        }),
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(PAGE_SLUG)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": format!("{{{{{LAYOUT_MARKER}}}}}"),
            "title": "Site fallback layout creation fixture",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": null,
            "revision_comments": "create site fallback layout fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": PAGE_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let compiled_body_html = match view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found anonymous page view, got {other:?}"),
    };

    assert_eq!(
        compiled_body_html,
        format!(r#"<p><tt>{LAYOUT_MARKER}</tt></p>"#),
        "the stored public body should fall back to the site layout",
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
async fn page_editing_history_preserves_each_page_change_as_a_recoverable_revision() {
    let mut runner = TestRunner::setup().await;

    const SITE_SLUG: &str = "test";
    const PAGE_SLUG: &str = "page-editing-history-fixture";
    const MOVED_PAGE_SLUG: &str = "page-editing-history-fixture-moved";

    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("seeded site should exist")
        .site;
    let site_id = site.site_id;
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(PAGE_SLUG.into())),
    });

    let created = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "++ One\n\nInitial",
            "title": "History fixture",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": null,
            "revision_comments": "create history fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let content_edit = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": created.page_id,
            "last_revision_id": created.revision_id,
            "revision_comments": "edit page content",
            "user_id": ADMIN_USER_ID,
            "wikitext": "++ One\n\nSection replacement",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("content edit should create a revision");

    let title_edit = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": created.page_id,
            "last_revision_id": content_edit.revision_id,
            "revision_comments": "edit page title",
            "user_id": ADMIN_USER_ID,
            "title": "Renamed history title",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("title edit should create a revision");

    let moved = run_endpoint!(
        runner,
        page_move,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
            "new_slug": MOVED_PAGE_SLUG,
            "last_revision_id": title_edit.revision_id,
            "revision_comments": "rename history fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(moved.revision_number, 3);

    let revisions = run_endpoint!(
        runner,
        page_revision_range,
        json!({
            "site_id": site_id,
            "page_id": created.page_id,
            "revision_number": moved.revision_number,
            "revision_direction": "before",
            "limit": 4,
            "details": {"wikitext": true},
        }),
    );
    let revisions = serde_json::to_value(revisions)
        .expect("revision history should serialize at the public endpoint");
    assert_eq!(revisions.as_array().map(Vec::len), Some(4));
    assert_eq!(
        revisions
            .as_array()
            .unwrap()
            .iter()
            .map(|revision| revision["revision_number"].as_i64().unwrap())
            .collect::<Vec<_>>(),
        vec![3, 2, 1, 0],
    );
    assert_eq!(revisions[0]["revision_type"], "move");
    assert_eq!(revisions[1]["revision_type"], "regular");
    assert_eq!(revisions[2]["revision_type"], "regular");
    assert_eq!(revisions[3]["revision_type"], "create");
    assert_eq!(revisions[2]["wikitext"], "++ One\n\nSection replacement");
    assert_eq!(revisions[1]["wikitext"], "++ One\n\nSection replacement");
    assert_eq!(revisions[3]["wikitext"], "++ One\n\nInitial");
    assert_eq!(revisions[1]["title"], "Renamed history title");
    assert_eq!(revisions[2]["title"], "History fixture");

    let stale = run_endpoint_err!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": created.page_id,
            "last_revision_id": created.revision_id,
            "revision_comments": "stale edit must not overwrite history",
            "user_id": ADMIN_USER_ID,
            "wikitext": "stale content",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(stale, ErrorType::NotLatestRevisionId);
}

#[tokio::test]
async fn documented_expression_parser_functions_render_at_the_public_preview_seam() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;

    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site.site_id,
            "title": "Documented expression parser functions",
            "wikitext": concat!(
                "ABS=[[#expr abs(-100) ]]\n",
                "MIN=[[#expr min(4, 1, -4, 6, -10) ]]\n",
                "MAX=[[#expr max(4, 1, -4, 6, -10) ]]\n",
                "MATH=[[#expr 2*(2-1) ]]\n",
                "IF_TRUE=[[#if true | TRUE | FALSE ]]\n",
                "IF_FALSE=[[#if false | TRUE | FALSE ]]\n",
                "IF_ZERO=[[#if 0 | TRUE | FALSE ]]\n",
                "IF_NULL=[[#if null | TRUE | FALSE ]]\n",
                "IF_NULL_UPPER=[[#if NULL | TRUE | FALSE ]]\n",
                "IFEXPR=[[#ifexpr 2*(2-1) == 2 | YES | NO ]]",
            ),
        }),
    );

    for expected in [
        "ABS=100",
        "MIN=-10",
        "MAX=6",
        "MATH=2",
        "IF_TRUE=TRUE",
        "IF_FALSE=FALSE",
        "IF_ZERO=FALSE",
        "IF_NULL=FALSE",
        "IF_NULL_UPPER=TRUE",
        "IFEXPR=YES",
    ] {
        assert!(
            preview.body.contains(expected),
            "documented expression output {expected:?} missing from:\n{}",
            preview.body,
        );
    }
    assert!(!preview.body.contains("[[#expr"));
    assert!(!preview.body.contains("[[#if"));
}

#[tokio::test]
async fn expression_size_budget_matches_the_public_preview_seam() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;

    let old_boundary_control = format!("[[#expr 1{}+0]]", " ".repeat(254));
    let at_bound = format!("[[#expr 1{}+0]]", " ".repeat(16 * 1024 - 3));
    let over_bound = format!("[[#expr 1{}+0]]", " ".repeat(16 * 1024 - 2));
    assert_eq!(old_boundary_control.len() - 10, 257);
    assert_eq!(at_bound.len() - 10, 16 * 1024);
    assert_eq!(over_bound.len() - 10, 16 * 1024 + 1);

    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site.site_id,
            "title": "Expression size boundary",
            "wikitext": format!(
                "OLD_BOUNDARY_CONTROL={old_boundary_control}\nAT_BOUND={at_bound}\nOVER_BOUND={over_bound}"
            ),
        }),
    );

    assert!(
        preview.body.contains("OLD_BOUNDARY_CONTROL=1"),
        "the retained live 257-byte control must evaluate after removing the obsolete 256-byte cliff:\n{}",
        preview.body,
    );
    assert!(
        preview.body.contains("AT_BOUND=1"),
        "the local 16 KiB expression budget must still evaluate at its boundary:\n{}",
        preview.body,
    );
    assert!(
        !preview.body.contains("OVER_BOUND=1"),
        "the expression one byte beyond the local 16 KiB safety budget must not evaluate:\n{}",
        preview.body,
    );
    assert!(
        preview.body.contains("OVER_BOUND=[<a href=\"#expr\">1")
            && preview.body.contains("+0</a>]"),
        "the expression beyond the local 16 KiB safety budget must remain unevaluated and continue through the ordinary literal-wikitext public seam:\n{}",
        preview.body,
    );
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
        None,
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
        None,
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
            .ends_with("; deepwell-render/v14")
    );
}

async fn create_navigation_fixture_page(
    runner: &mut TestRunner,
    site_id: i64,
    slug: &'static str,
    title: &'static str,
    wikitext: &'static str,
) -> GetPageOutput {
    set_mutation_request_context(
        runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(slug)),
    );
    let created = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": wikitext,
            "title": title,
            "alt_title": null,
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": format!("create {title}"),
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": created.page_id,
        }),
    )
    .unwrap_or_else(|| panic!("navigation fixture page {slug} should exist"))
}

async fn rerender_navigation_fixture_page(runner: &mut TestRunner, page: &GetPageOutput) {
    set_mutation_request_context(
        runner,
        ADMIN_USER_ID,
        page.site_id,
        Reference::Id(page.page_id),
    );
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": page.site_id,
            "category_id": page.page_category_id,
            "page_id": page.page_id,
        }),
    );
}

async fn public_page_view_navigation(
    runner: &mut TestRunner,
    site_id: i64,
    slug: &'static str,
    found: bool,
) -> (Option<String>, Option<String>, Option<bool>) {
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
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
    match (found, view) {
        (
            true,
            GetPageViewOutput::Found {
                compiled_top_bar_html,
                compiled_side_bar_html,
                page_revision,
                ..
            },
        ) => (
            compiled_top_bar_html,
            compiled_side_bar_html,
            Some(page_revision.updated_at.is_some()),
        ),
        (
            false,
            GetPageViewOutput::Missing {
                compiled_top_bar_html,
                compiled_side_bar_html,
                ..
            },
        ) => (compiled_top_bar_html, compiled_side_bar_html, None),
        (_, other) => panic!("unexpected public page view for {slug}: {other:?}"),
    }
}

async fn public_page_view_top_bar(
    runner: &mut TestRunner,
    site_id: i64,
    slug: &'static str,
    found: bool,
) -> Option<String> {
    public_page_view_navigation(runner, site_id, slug, found)
        .await
        .0
}

#[tokio::test]
async fn page_create_refreshes_only_executable_dynamic_navigation_after_recording_latest_revision()
 {
    for (case, nav_source, rerender_expected) in [
        (
            "mixed-whitespace",
            "[[module\tListPages \t tags=\"+fresh\"\t separate=\"no\" wrapper=\"no\"]]NAV=%%fullname%%[[/module]]",
            true,
        ),
        (
            "whitespace-include",
            "[[ \t include nav:create-freshness-component-whitespace-include]]",
            true,
        ),
        (
            "cr-include",
            "[[\rinclude nav:create-freshness-component-whitespace-include]]",
            false,
        ),
        (
            "comment-include",
            "[[[!-- parser-space --]include nav:create-freshness-component-whitespace-include]]",
            false,
        ),
        (
            "code-literal",
            "[[code]]\n[[module ListPages tags=\"+fresh\"]]NAV=%%fullname%%[[/module]]\n[[/code]]",
            false,
        ),
        (
            "invalid-name",
            "[[module ListPagesExtra tags=\"+fresh\"]]NAV=%%fullname%%[[/module]]",
            false,
        ),
    ] {
        let mut runner = TestRunner::setup().await;
        let site_id =
            run_endpoint!(runner, site_get, json!({"site": "scpaiueouiuiuiui"}),)
                .expect("editable local authoring site should exist")
                .site
                .site_id;
        let top_slug =
            Box::leak(format!("nav:create-freshness-top-{case}").into_boxed_str());
        let side_slug =
            Box::leak(format!("nav:create-freshness-side-{case}").into_boxed_str());
        let page_slug =
            Box::leak(format!("create-freshness-{case}:target").into_boxed_str());
        let top_title =
            Box::leak(format!("Page create freshness top {case}").into_boxed_str());
        let side_title =
            Box::leak(format!("Page create freshness side {case}").into_boxed_str());
        if case == "whitespace-include" {
            create_navigation_fixture_page(
                &mut runner,
                site_id,
                "nav:create-freshness-component-whitespace-include",
                "Page create freshness whitespace include component",
                "[[module ListPages tags=\"+fresh\" separate=\"no\" wrapper=\"no\"]]NAV=%%fullname%%[[/module]]",
            )
            .await;
        }
        create_navigation_fixture_page(
            &mut runner,
            site_id,
            top_slug,
            top_title,
            nav_source,
        )
        .await;
        create_navigation_fixture_page(
            &mut runner,
            site_id,
            side_slug,
            side_title,
            nav_source,
        )
        .await;
        let site = run_endpoint!(runner, site_get, json!({"site": site_id}),)
            .expect("editable local authoring site should still exist");
        runner.set_request_context(RequestContext {
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site_id),
            ..Default::default()
        });
        run_endpoint!(
            runner,
            site_update,
            json!({
                "site": site_id,
                "user_id": ADMIN_USER_ID,
                "expected_settings_revision": site.settings.revision,
                "top_bar_page": top_slug,
                "side_bar_page": side_slug,
                "ip_address": common::IP_ADDRESS,
            }),
        );
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
                "wikitext": "Page creation navigation freshness target body",
                "title": format!("Page creation navigation freshness target {case}"),
                "alt_title": null,
                "slug": page_slug,
                "tags": ["fresh"],
                "layout": "wikidot",
                "revision_comments": format!("create navigation freshness target {case}"),
                "user_id": ADMIN_USER_ID,
                "ip_address": common::IP_ADDRESS,
            }),
        );

        let (top_bar, side_bar, timing) =
            public_page_view_navigation(&mut runner, site_id, page_slug, true).await;
        let top_bar = top_bar.expect("configured top bar should be compiled");
        let side_bar = side_bar.expect("configured side bar should be compiled");
        assert_eq!(
            timing.expect("found view has revision lifecycle state"),
            rerender_expected,
            "{case}",
        );
        for (position, navigation) in [("top", top_bar), ("side", side_bar)] {
            assert_eq!(
                navigation.contains(&format!("NAV={page_slug}")),
                rerender_expected,
                "{case} {position} navigation must reflect whether its module executes",
            );
        }
    }
}

#[tokio::test]
async fn deleted_site_navigation_is_absent_from_found_and_missing_page_views() {
    const NAV_SLUG: &str = "nav:deleted-site-navigation-fixture";
    const PAGE_SLUG: &str = "deleted-site-navigation-article";
    const MISSING_SLUG: &str = "deleted-site-navigation-missing";
    const MARKER: &str = "deleted site navigation marker";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scpaiueouiuiuiui"}),)
        .expect("editable local authoring site should exist");
    let site_id = site.site.site_id;
    let navigation = create_navigation_fixture_page(
        &mut runner,
        site_id,
        NAV_SLUG,
        "Deleted site navigation fixture",
        MARKER,
    )
    .await;
    let article = create_navigation_fixture_page(
        &mut runner,
        site_id,
        PAGE_SLUG,
        "Deleted site navigation article",
        "Deleted site navigation article body",
    )
    .await;

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        site_update,
        json!({
            "site": site_id,
            "user_id": ADMIN_USER_ID,
            "expected_settings_revision": site.settings.revision,
            "top_bar_page": NAV_SLUG,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    rerender_navigation_fixture_page(&mut runner, &article).await;

    for (slug, found) in [(PAGE_SLUG, true), (MISSING_SLUG, false)] {
        let top_bar = public_page_view_top_bar(&mut runner, site_id, slug, found)
            .await
            .expect("configured top bar should be present");
        assert!(
            top_bar.contains(MARKER),
            "configured top bar should contain the marker for {slug}:\n{top_bar}",
        );
    }

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(navigation.page_id),
    );
    run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": navigation.page_id,
            "last_revision_id": navigation.revision_id,
            "revision_comments": "delete configured site navigation",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    rerender_navigation_fixture_page(&mut runner, &article).await;

    for (slug, found) in [(PAGE_SLUG, true), (MISSING_SLUG, false)] {
        let top_bar = public_page_view_top_bar(&mut runner, site_id, slug, found).await;
        assert_eq!(
            top_bar, None,
            "deleted configured navigation must be absent for {slug}",
        );
    }

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(navigation.page_id),
    );
    run_endpoint!(
        runner,
        page_restore,
        json!({
            "site_id": site_id,
            "page_id": navigation.page_id,
            "slug": NAV_SLUG,
            "revision_comments": "restore configured site navigation",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    rerender_navigation_fixture_page(&mut runner, &article).await;

    for (slug, found) in [(PAGE_SLUG, true), (MISSING_SLUG, false), (NAV_SLUG, true)] {
        let top_bar = public_page_view_top_bar(&mut runner, site_id, slug, found)
            .await
            .expect("restored top bar should be present");
        assert!(
            top_bar.contains(MARKER),
            "restored top bar should contain the marker for {slug}:\n{top_bar}",
        );
    }
}

#[tokio::test]
async fn deleted_category_navigation_does_not_fall_back_to_site_navigation() {
    const SITE_NAV_SLUG: &str = "nav:deleted-category-site-fallback-fixture";
    const CATEGORY_NAV_SLUG: &str = "nav:deleted-category-navigation-fixture";
    const PAGE_SLUG: &str = "deleted-category-navigation:article";
    const MISSING_SLUG: &str = "deleted-category-navigation:missing";
    const SITE_MARKER: &str = "site navigation fallback must stay absent";
    const CATEGORY_MARKER: &str = "deleted category navigation marker";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;
    create_navigation_fixture_page(
        &mut runner,
        site_id,
        SITE_NAV_SLUG,
        "Category navigation site fallback fixture",
        SITE_MARKER,
    )
    .await;
    let category_navigation = create_navigation_fixture_page(
        &mut runner,
        site_id,
        CATEGORY_NAV_SLUG,
        "Deleted category navigation fixture",
        CATEGORY_MARKER,
    )
    .await;
    let article = create_navigation_fixture_page(
        &mut runner,
        site_id,
        PAGE_SLUG,
        "Deleted category navigation article",
        "Deleted category navigation article body",
    )
    .await;

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        site_update,
        json!({
            "site": site_id,
            "user_id": ADMIN_USER_ID,
            "expected_settings_revision": site.settings.revision,
            "top_bar_page": SITE_NAV_SLUG,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": article.page_category_id,
            "user_id": ADMIN_USER_ID,
            "top_bar_page": CATEGORY_NAV_SLUG,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    rerender_navigation_fixture_page(&mut runner, &article).await;

    for (slug, found) in [(PAGE_SLUG, true), (MISSING_SLUG, false)] {
        let top_bar = public_page_view_top_bar(&mut runner, site_id, slug, found)
            .await
            .expect("category top bar should be present");
        assert!(top_bar.contains(CATEGORY_MARKER), "{slug}:\n{top_bar}");
        assert!(!top_bar.contains(SITE_MARKER), "{slug}:\n{top_bar}");
    }

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(category_navigation.page_id),
    );
    run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": category_navigation.page_id,
            "last_revision_id": category_navigation.revision_id,
            "revision_comments": "delete configured category navigation",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    rerender_navigation_fixture_page(&mut runner, &article).await;

    for (slug, found) in [(PAGE_SLUG, true), (MISSING_SLUG, false)] {
        let top_bar = public_page_view_top_bar(&mut runner, site_id, slug, found).await;
        assert_eq!(
            top_bar, None,
            "a deleted category override must stay absent rather than inherit site navigation for {slug}",
        );
    }
}

#[tokio::test]
async fn wikidot_fragment_only_double_hash_href_survives_preview_and_saved_page() {
    const SOURCE: &str = r###"[[a href="##"]]Issue 610 fragment closer[[/a]] ##red|Issue 610 color boundary##"###;
    const EXPECTED_ANCHOR: &str = r###"<a href="##">Issue 610 fragment closer</a>"###;
    const EXPECTED_COLOR: &str =
        r###"<span style="color: red">Issue 610 color boundary</span>"###;

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "Issue 610 fragment preview",
            "wikitext": SOURCE,
        }),
    );
    assert!(
        preview.body.contains(EXPECTED_ANCHOR),
        "PagePreview must preserve the exact fragment-only href:\n{}",
        preview.body,
    );
    assert!(
        !preview.body.contains(r#"href="/&#35;&#35;""#),
        "PagePreview rewrote the fragment-only href as a path:\n{}",
        preview.body,
    );
    assert!(
        preview.body.contains(EXPECTED_COLOR),
        "PagePreview must still render an ordinary color marker outside the link:\n{}",
        preview.body,
    );

    let nav_side = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "nav:side",
        }),
    )
    .expect("seeded nav:side should exist");
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(nav_side.page_id),
    );
    run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": nav_side.page_id,
            "last_revision_id": nav_side.revision_id,
            "revision_comments": "replace sidebar with Issue 610 fragment fixture",
            "user_id": ADMIN_USER_ID,
            "wikitext": SOURCE,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("editing nav:side should create a revision");

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

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
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
    let side_bar = match view {
        GetPageViewOutput::Found {
            compiled_side_bar_html,
            ..
        } => compiled_side_bar_html.expect("side bar should be compiled"),
        other => panic!("expected found page view, got {other:?}"),
    };
    assert!(
        side_bar.contains(EXPECTED_ANCHOR),
        "saved navigation rerender must preserve the exact fragment-only href:\n{side_bar}",
    );
    assert!(
        !side_bar.contains("All wikis") && !side_bar.contains(r#"href="/&#35;&#35;""#),
        "page_view reused stale navigation or rewrote the href:\n{side_bar}",
    );
    assert!(
        side_bar.contains(EXPECTED_COLOR),
        "saved navigation rerender must still render an ordinary color marker outside the link:\n{side_bar}",
    );
}

#[tokio::test]
async fn page_attribution_update_refreshes_cached_anonymous_article_view() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    let (page_id, slug) =
        import_cacheable_page_attribution_fixture(&mut runner, site.site_id, "update")
            .await;
    let before = load_cached_attribution_article(&mut runner, site.site_id, &slug).await;
    assert!(before.attributions.is_empty());

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(page_id),
    );
    run_endpoint!(
        runner,
        page_attribution_update,
        json!({
            "site_id": site.site_id,
            "page": page_id,
            "updated_by": ADMIN_USER_ID,
            "attributions": [{
                "user_id": SAMPLE_USER_ID,
                "metadata": {
                    "attribution_type": "author",
                    "attribution_date": "2026-08-13",
                },
            }],
        }),
    );
    runner
        .context()
        .run_post_commit_actions()
        .await
        .expect("page attribution update post-commit actions should complete");

    let after = load_cached_attribution_article(&mut runner, site.site_id, &slug).await;
    assert_ne!(after.fence, before.fence);
    assert_ne!(after.cache_key, before.cache_key);
    assert_eq!(after.attributions.len(), 1);
    assert_eq!(after.attributions[0].user_id, SAMPLE_USER_ID);
    assert_eq!(
        after.attributions[0].metadata.attribution_type,
        PageAttributionKind::Author,
    );
    assert_eq!(
        after.attributions[0].metadata.attribution_date,
        time::Date::from_calendar_date(2026, time::Month::August, 13)
            .expect("fixture attribution date should be valid"),
    );
}

#[tokio::test]
async fn page_attribution_delete_refreshes_cached_article_only_when_rows_are_cleared() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    let (page_id, slug) =
        import_cacheable_page_attribution_fixture(&mut runner, site.site_id, "delete")
            .await;

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(page_id),
    );
    run_endpoint!(
        runner,
        page_attribution_update,
        json!({
            "site_id": site.site_id,
            "page": page_id,
            "updated_by": ADMIN_USER_ID,
            "attributions": [{
                "user_id": SAMPLE_USER_ID,
                "metadata": {
                    "attribution_type": "author",
                    "attribution_date": "2026-08-13",
                },
            }],
        }),
    );
    runner
        .context()
        .run_post_commit_actions()
        .await
        .expect("attribution setup post-commit actions should complete");

    let before = load_cached_attribution_article(&mut runner, site.site_id, &slug).await;
    assert_eq!(before.attributions.len(), 1);

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(page_id),
    );
    run_endpoint!(
        runner,
        page_attribution_delete,
        json!({
            "site_id": site.site_id,
            "page": page_id,
            "removed_by": ADMIN_USER_ID,
        }),
    );
    runner
        .context()
        .run_post_commit_actions()
        .await
        .expect("effective attribution delete post-commit actions should complete");

    let after_delete =
        load_cached_attribution_article(&mut runner, site.site_id, &slug).await;
    assert!(after_delete.attributions.is_empty());
    assert_ne!(after_delete.fence, before.fence);
    assert_ne!(after_delete.cache_key, before.cache_key);

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(page_id),
    );
    run_endpoint!(
        runner,
        page_attribution_delete,
        json!({
            "site_id": site.site_id,
            "page": page_id,
            "removed_by": ADMIN_USER_ID,
        }),
    );
    runner
        .context()
        .run_post_commit_actions()
        .await
        .expect("repeated attribution delete post-commit drain should succeed");

    let after_repeated_delete =
        load_cached_attribution_article(&mut runner, site.site_id, &slug).await;
    assert!(after_repeated_delete.attributions.is_empty());
    assert_eq!(after_repeated_delete.fence, after_delete.fence);
    assert_eq!(after_repeated_delete.cache_key, after_delete.cache_key);
}

#[tokio::test]
async fn renderer_epoch_invalidates_pre_freeze_compiled_artifacts() {
    const SLUG: &str = "renderer-epoch-cache-fixture";
    const CURRENT_BODY: &str = "renderer epoch current body";
    const STALE_BODY: &str = "stale deepwell-render/v9 body";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;
    create_listpages_test_page(
        &mut runner,
        site_id,
        SLUG,
        "Renderer epoch cache fixture",
        CURRENT_BODY,
    )
    .await;

    let page = PageTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page::Column::SiteId.eq(site_id))
                .add(page::Column::Slug.eq(SLUG)),
        )
        .one(runner.context().transaction())
        .await
        .expect("renderer epoch page lookup should not fail")
        .expect("renderer epoch page should exist");
    let mut page = page.into_active_model();
    page.from_wikidot = Set(true);
    page.update(runner.context().transaction())
        .await
        .expect("renderer epoch page should be marked imported");

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
    let input = json!({
        "site_id": site_id,
        "session_token": null,
        "route": {"slug": SLUG, "extra": ""},
        "locales": ["en-US", "en"],
    });
    let mut stale_page = run_endpoint!(runner, page_view, input.clone());
    let GetPageViewOutput::Found {
        page_revision,
        compiled_body_html,
        ..
    } = &mut stale_page
    else {
        panic!("renderer epoch fixture should have a public page view");
    };
    page_revision.compiled_generator = "fixture-ftml; deepwell-render/v9".to_owned();
    *compiled_body_html = STALE_BODY.to_owned();

    let metadata = run_endpoint!(runner, article_view_cache_metadata, input.clone());
    let current_key = metadata
        .article_page_cache_key
        .expect("imported static page should have an anonymous cache key");
    assert!(
        current_key.starts_with("deepwell:article-view:page:v14:"),
        "source-freeze cache key must carry the final renderer epoch: {current_key}",
    );
    let stale_key = current_key.replacen(
        "deepwell:article-view:page:v14:",
        "deepwell:article-view:page:v11:",
        1,
    );
    assert_ne!(stale_key, current_key);
    let stale_json =
        serde_json::to_string(&stale_page).expect("stale page should serialize");
    let mut redis = runner.context().redis();
    redis
        .set::<_, _, ()>(&stale_key, &stale_json)
        .await
        .expect("stale v11 page should be inserted into the test cache");
    redis
        .set::<_, _, ()>(&current_key, &stale_json)
        .await
        .expect("stale artifact under the current cache key should be inserted");
    drop(redis);

    let view = run_endpoint!(runner, article_view, input);
    let GetArticleViewOutput {
        page: GetPageViewOutput::Found {
            compiled_body_html, ..
        },
        article_page_cache_key: Some(served_key),
        ..
    } = view
    else {
        panic!("renderer epoch fixture should return an article view");
    };
    assert_eq!(served_key, current_key);
    assert!(compiled_body_html.contains(CURRENT_BODY));
    assert!(!compiled_body_html.contains(STALE_BODY));
}

#[tokio::test]
async fn page_view_rerenders_stale_persisted_compiled_artifact() {
    const SLUG: &str = "renderer-epoch-persisted-fixture";
    const CURRENT_BODY: &str = "renderer epoch persisted current body";
    const STALE_BODY: &str = "stale deepwell-render/v9 persisted body";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;
    let revision_id = create_listpages_test_page(
        &mut runner,
        site_id,
        SLUG,
        "Renderer epoch persisted fixture",
        CURRENT_BODY,
    )
    .await;

    let stale_body_hash = TextService::create(runner.context(), STALE_BODY.to_owned())
        .await
        .expect("stale compiled body should be stored");
    let revision = PageRevisionTable::find_by_id(revision_id)
        .one(runner.context().transaction())
        .await
        .expect("renderer epoch revision lookup should not fail")
        .expect("renderer epoch revision should exist");
    let mut revision = revision.into_active_model();
    revision.compiled_body_html_hash = Set(stale_body_hash.to_vec());
    revision.compiled_generator = Set("fixture-ftml; deepwell-render/v9".to_owned());
    revision
        .update(runner.context().transaction())
        .await
        .expect("stale compiled artifact should be attached");

    let anonymous_error = run_endpoint_err!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    assert!(format!("{anonymous_error:?}").contains("failed to generate page view"));
    let anonymous_article_error = run_endpoint_err!(
        runner,
        article_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    assert!(
        format!("{anonymous_article_error:?}").contains("failed to generate page view")
    );

    let admin_session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "renderer epoch stale artifact test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");
    let article = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site_id,
            "session_token": admin_session_token.clone(),
            "route": {"slug": SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let GetArticleViewOutput {
        page:
            GetPageViewOutput::Found {
                page_revision,
                compiled_body_html,
                ..
            },
        ..
    } = article
    else {
        panic!("article view should refresh a stale persisted page");
    };
    assert!(
        page_revision
            .compiled_generator
            .ends_with("; deepwell-render/v14")
    );
    assert!(compiled_body_html.contains(CURRENT_BODY));
    assert!(!compiled_body_html.contains(STALE_BODY));
    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": admin_session_token,
            "route": {"slug": SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let GetPageViewOutput::Found {
        page_revision,
        compiled_body_html,
        ..
    } = view
    else {
        panic!("renderer epoch fixture should return a found page view");
    };
    assert!(
        page_revision
            .compiled_generator
            .ends_with("; deepwell-render/v14"),
        "page view must expose the current compiled generator",
    );
    assert!(compiled_body_html.contains(CURRENT_BODY));
    assert!(!compiled_body_html.contains(STALE_BODY));

    let persisted = PageRevisionTable::find_by_id(revision_id)
        .one(runner.context().transaction())
        .await
        .expect("rerendered revision lookup should not fail")
        .expect("rerendered revision should exist");
    assert!(
        persisted
            .compiled_generator
            .ends_with("; deepwell-render/v14"),
        "read-time refresh should persist the current compiled generator",
    );
    let persisted_body =
        TextService::get(runner.context(), &persisted.compiled_body_html_hash)
            .await
            .expect("rerendered compiled body should be readable");
    assert!(persisted_body.contains(CURRENT_BODY));
    assert!(!persisted_body.contains(STALE_BODY));
}

#[tokio::test]
async fn article_view_cache_respects_anonymous_permission_revocation() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;
    let category_slug = "article-cache-permission-revocation";
    let slug = "article-cache-permission-revocation:source-page";
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

    let missing = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {
                "slug": "article-cache-permission-revocation:missing",
                "extra": "",
            },
            "locales": ["en-US", "en"],
        }),
    );
    let GetArticleViewOutput {
        page: GetPageViewOutput::Permissions { banned: false, .. },
        ..
    } = missing
    else {
        panic!(
            "a missing page in a category without page:view permission must not expose the missing-page action surface"
        );
    };

    let alias = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {
                "slug": "article-cache-permission-revocation:source+page",
                "extra": "",
            },
            "locales": ["en-US", "en"],
        }),
    );
    let GetArticleViewOutput {
        page: GetPageViewOutput::Permissions { banned: false, .. },
        ..
    } = alias
    else {
        panic!(
            "an observed URL alias must preserve the canonical page's view permission"
        );
    };
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
async fn imported_countpages_sources_are_excluded_from_article_view_cache_metadata() {
    const DIRECT_SLUG: &str = "article-cache-countpages-direct:holder";
    const WHITESPACE_FALLBACK_SLUG: &str =
        "article-cache-countpages-whitespace-fallback:holder";
    const TEMPLATE_SLUG: &str = "article-cache-countpages-template:_template";
    const TEMPLATED_SLUG: &str = "article-cache-countpages-template:holder";
    const LITERAL_SLUG: &str = "article-cache-countpages-literal:holder";
    const EXECUTABLE_COUNT_PAGES: &str = "[[module CountPages category=\"article-cache-countpages-target\"]]%%total%%[[/module]]";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        DIRECT_SLUG,
        "Imported direct CountPages cache fixture",
        EXECUTABLE_COUNT_PAGES,
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        WHITESPACE_FALLBACK_SLUG,
        "Imported whitespace fallback CountPages cache fixture",
        "[[module CountPages category=\"article-cache-countpages-target\" tags=\"@URL |+fresh\"]]%%total%%[[/module]]",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        TEMPLATE_SLUG,
        "Imported templated CountPages cache template",
        &format!("{EXECUTABLE_COUNT_PAGES}\n%%content%%"),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        TEMPLATED_SLUG,
        "Imported templated CountPages cache fixture",
        "cache-safe templated page body",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        LITERAL_SLUG,
        "Imported literal CountPages cache fixture",
        "[[code]]\n[[module CountPages category=\"article-cache-countpages-target\"]]%%total%%[[/module]]\n[[/code]]",
    )
    .await;

    for slug in [
        DIRECT_SLUG,
        WHITESPACE_FALLBACK_SLUG,
        TEMPLATED_SLUG,
        LITERAL_SLUG,
    ] {
        let page = PageTable::find()
            .filter(
                sea_orm::Condition::all()
                    .add(page::Column::SiteId.eq(site_id))
                    .add(page::Column::Slug.eq(slug)),
            )
            .one(runner.context().transaction())
            .await
            .expect("CountPages cache fixture lookup should succeed")
            .expect("CountPages cache fixture should exist");
        let mut page = page.into_active_model();
        page.from_wikidot = Set(true);
        page.update(runner.context().transaction())
            .await
            .expect("CountPages cache fixture should be marked imported");
    }

    for slug in [DIRECT_SLUG, WHITESPACE_FALLBACK_SLUG, TEMPLATED_SLUG] {
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
            "executable CountPages must deny imported article caching for {slug}",
        );
    }

    let literal_metadata = run_endpoint!(
        runner,
        article_view_cache_metadata,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": LITERAL_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    assert!(
        literal_metadata.article_page_cache_key.is_some(),
        "literal CountPages text must remain eligible for imported article caching",
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
async fn page_preview_omits_iftags_in_included_sources() {
    const COMPONENT_SLUG: &str = "component:fixture-iftags-preview-source";
    const ACTIVE_MARKER: &str = "Included positive branch";
    const NEGATIVE_MARKER: &str = "Included negative branch";

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
            "wikitext": format!(
                "[[iftags +component]]{ACTIVE_MARKER}[[/iftags]]\n[[iftags -component]]{NEGATIVE_MARKER}[[/iftags]]"
            ),
            "title": "PagePreview iftags source",
            "alt_title": null,
            "slug": COMPONENT_SLUG,
            "layout": "wikidot",
            "tags": ["component"],
            "revision_comments": "create PagePreview iftags fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site.site.site_id,
            "title": "PagePreview iftags consumer",
            "wikitext": format!(
                "X\n[[include {COMPONENT_SLUG}]]\n[[iftags +component]]ROOT[[/iftags]]\n[[iftags -component]]ROOT_NEGATIVE[[/iftags]]\nY"
            ),
        }),
    );
    assert!(preview.body.contains("<p>X</p>") && preview.body.contains("<p>Y</p>"));
    assert!(!preview.body.contains(ACTIVE_MARKER), "{}", preview.body);
    assert!(!preview.body.contains(NEGATIVE_MARKER), "{}", preview.body);
    assert!(!preview.body.contains("ROOT_NEGATIVE"), "{}", preview.body);
}

#[tokio::test]
async fn include_parser_functions_follow_argument_and_caller_ownership_phases() {
    const COMPONENT_SLUG: &str = "component:fixture-parser-function-phases";
    const CANONICAL_SAVED_SLUG: &str =
        "fixture-parser-function-phases-canonical-consumer";
    const SAVED_SLUG: &str = "fixture-parser-function-phases-consumer";
    const COMPONENT_SOURCE: &str = concat!(
        "IF=[[#if {$x} | YES | NO ]]\n",
        "IFEXPR=[[#ifexpr {$n} > 0 | POS | NONPOS ]]\n",
        "EXPR=[[#expr {$n}+2 ]]\n",
        "IF_TIGHT=[[#if {$x}|TIGHT_Y|TIGHT_N]]\n",
        "IF_UPPER=[[#IF {$x} | UPPER_Y | UPPER_N ]]\n",
        "RAWX={$x}\n",
        "RAWN={$n}\n",
    );

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(COMPONENT_SLUG)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": COMPONENT_SOURCE,
            "title": "Parser function phase fixture",
            "alt_title": null,
            "slug": COMPONENT_SLUG,
            "layout": "wikidot",
            "revision_comments": "create parser function phase fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let canonical_source = format!("[[include {COMPONENT_SLUG} |x=1|n=1]]");
    let canonical = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "Canonical parser function phase preview",
            "wikitext": canonical_source,
        }),
    );
    for expected in ["IF=YES", "IFEXPR=POS", "EXPR=3", "RAWX=1", "RAWN=1"] {
        assert!(
            canonical.body.contains(expected),
            "canonical include should substitute variables before evaluation ({expected}): {}",
            canonical.body,
        );
    }

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(CANONICAL_SAVED_SLUG)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": canonical_source,
            "title": "Saved canonical parser function phase",
            "alt_title": null,
            "slug": CANONICAL_SAVED_SLUG,
            "layout": "wikidot",
            "revision_comments": "create saved canonical parser function phase fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let canonical_saved = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": CANONICAL_SAVED_SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("saved canonical parser function phase page should exist")
    .compiled_body_html
    .expect("saved canonical parser function phase page should have compiled HTML");
    for expected in ["IF=YES", "IFEXPR=POS", "EXPR=3", "RAWX=1", "RAWN=1"] {
        assert!(
            canonical_saved.contains(expected),
            "saved canonical include should substitute variables before evaluation ({expected}): {canonical_saved}",
        );
    }

    let argument = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "Argument parser function phase preview",
            "wikitext": format!(
                "[[include {COMPONENT_SLUG} |x=[[#if 1 | 1 | 0 ]]|n=1]]"
            ),
        }),
    );
    assert!(
        argument.body.contains("RAWX=[[#if 1"),
        "include pipe grammar must bind only the first argument segment: {}",
        argument.body,
    );
    assert!(
        !argument.body.contains("RAWX=1"),
        "an argument parser function must not execute before include collection: {}",
        argument.body,
    );

    let literal_source =
        format!("[[code]]\n[[include {COMPONENT_SLUG} |x=1|n=1]]\n[[/code]]");
    let literal = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "Caller literal parser function phase preview",
            "wikitext": literal_source,
        }),
    );
    assert!(
        literal.body.contains(r#"<div class="code""#),
        "{}",
        literal.body
    );
    for evaluated in ["IF=YES", "IFEXPR=POS", "EXPR=3"] {
        assert!(
            !literal.body.contains(evaluated),
            "caller code ownership must keep included parser functions literal ({evaluated}): {}",
            literal.body,
        );
    }

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(SAVED_SLUG)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": literal_source,
            "title": "Saved caller literal parser function phase",
            "alt_title": null,
            "slug": SAVED_SLUG,
            "layout": "wikidot",
            "revision_comments": "create saved parser function phase fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let saved = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": SAVED_SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("saved parser function phase page should exist")
    .compiled_body_html
    .expect("saved parser function phase page should have compiled HTML");
    assert!(saved.contains(r#"<div class="code""#), "{saved}");
    for evaluated in ["IF=YES", "IFEXPR=POS", "EXPR=3"] {
        assert!(
            !saved.contains(evaluated),
            "saved caller code ownership must keep included parser functions literal ({evaluated}): {saved}",
        );
    }
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
            "[[f=image centered-leaf.png]]\n",
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
            "[[image literal-quoted-thumb.png link=\"literal quoted full.png\"]]\n",
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
        html.contains("/local--files/fragment:attachment-owner-leaf/centered-leaf.png")
            && !html.contains(
                "/local--files/fixture-attachment-owner-consumer/centered-leaf.png",
            ),
        "f=image must retain the nested included source owner: {html}",
    );
    assert!(
        html.contains("/local--files/fragment:attachment-owner-leaf/leaf%20two.png"),
        "a quoted relative filename must retain its owner and be URL-encoded in final HTML: {html}"
    );
    for forwarded in ["forwarded.png", "forwarded%20two.png"] {
        let owned = format!("/local--files/fragment:attachment-owner-leaf/{forwarded}");
        let expected_occurrences = 2;
        assert_eq!(
            html.matches(&owned).count(),
            expected_occurrences,
            "a forwarded attachment must use its leaf-owned URL wherever Wikidot accepts the argument shape: {html}"
        );
        let second_owned =
            format!("/local--files/fragment:attachment-owner-second-leaf/{forwarded}");
        assert_eq!(
            html.matches(&second_owned).count(),
            expected_occurrences,
            "same-valued forwarded occurrences from another leaf must retain their distinct owner wherever Wikidot accepts the argument shape: {html}"
        );
    }
    assert_eq!(
        html.matches(
            "/local--files/component:attachment-owner-wrapper/thumb-forwarded.png",
        )
        .count(),
        2,
        "a bare composite value must retain ordinary substitution for src, belong to the wrapper that authored it, and not acquire an ignored bare link: {html}",
    );
    assert!(
        html.contains("/local--files/component:attachment-owner-base/literal-thumb.png")
            && !html.contains(
                "/local--files/component:attachment-owner-base/literal-full.png"
            ),
        "a literal bare image target must retain the base source owner while Wikidot's ignored bare link remains absent: {html}"
    );
    assert!(
        html.contains(
            "/local--files/component:attachment-owner-base/literal-quoted-thumb.png"
        ) && html.contains(
            "/local--files/component:attachment-owner-base/literal%20quoted%20full.png"
        ),
        "a literal quoted link in included content must retain both its quotes and base source owner: {html}"
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
                "[[module CSS show=\"true\"]]\n",
                ".shown { color: blue; }\n",
                ".shown::after { content: \"<unsafe>\"; }\n",
                "[[/module]]\n",
                "[[module CSS show=\"yes\" disable=\"true\"]]\n",
                ".shown-disabled { color: purple; }\n",
                "[[/module]]\n",
                "[[module CSS show=\"TRUE\"]]\n",
                ".show-uppercase { color: orange; }\n",
                "[[/module]]\n",
                "[[module CSS show = \"true\"]]\n",
                ".show-spaced { color: cyan; }\n",
                "[[/module]]\n",
                "[[module CSS disable=\"true\"]]\n",
                ".disabled { color: gray; }\n",
                "[[/module]]\n",
                "[[module CSS disable=\"yes\"]]\n",
                ".disabled-yes { color: gray; }\n",
                "[[/module]]\n",
                "[[module CSS disable=\"TRUE\"]]\n",
                ".disable-uppercase { color: black; }\n",
                "[[/module]]\n",
                "[[module CSS disable=\"true\" disable=\"false\"]]\n",
                ".duplicate-active { color: green; }\n",
                "[[/module]]\n",
                "[[module CSS disable=\"false\" disable=\"true\"]]\n",
                ".duplicate-disabled { color: green; }\n",
                "[[/module]]\n",
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
    assert!(
        compiled_body_html.contains(r#"<div class="code""#),
        "show=true CSS module should render a visible code block: {compiled_body_html}",
    );
    assert!(compiled_body_html.contains(r#"<span class="hl-identifier">.shown</span>"#));
    assert!(compiled_body_html.contains(r#"&quot;&lt;unsafe&gt;&quot;"#));
    assert!(
        compiled_body_html
            .contains(r#"<span class="hl-identifier">.shown-disabled</span>"#)
    );
    assert!(!compiled_body_html.contains(".show-uppercase { color: orange; }"));
    assert!(!compiled_body_html.contains(".show-spaced { color: cyan; }"));

    assert_eq!(compiled_body_styles.len(), 7);
    assert!(compiled_body_styles[0].contains(".first { color: red; }"));
    assert!(compiled_body_styles[1].contains(".shown { color: blue; }"));
    assert!(compiled_body_styles[1].contains(r#"\3C unsafe>"#));
    assert!(
        !compiled_body_styles
            .iter()
            .any(|css| css.contains(".shown-disabled"))
    );
    assert!(
        !compiled_body_styles
            .iter()
            .any(|css| css.contains(".disabled { color: gray; }"))
    );
    assert!(
        !compiled_body_styles
            .iter()
            .any(|css| css.contains(".disabled-yes"))
    );
    assert!(
        compiled_body_styles
            .iter()
            .any(|css| css.contains(".show-uppercase"))
    );
    assert!(
        compiled_body_styles
            .iter()
            .any(|css| css.contains(".show-spaced"))
    );
    assert!(
        compiled_body_styles
            .iter()
            .any(|css| css.contains(".disable-uppercase"))
    );
    assert!(
        compiled_body_styles
            .iter()
            .any(|css| css.contains(".duplicate-active"))
    );
    assert!(
        !compiled_body_styles
            .iter()
            .any(|css| css.contains(".duplicate-disabled"))
    );
    assert!(compiled_body_styles[6].contains(r"\3C /style>\3C meta"));
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
async fn saved_page_view_rewrites_exact_cn_interwiki_embed_iframes() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;
    let slug = "cn-interwiki-embed-fixture";

    create_listpages_test_page(
        &mut runner,
        site_id,
        slug,
        "CN Interwiki Embed Fixture",
        concat!(
            "[[embed]]\n",
            r#"<iframe src="//interwiki.scpwikicn.com/interwikiFrame.html?lang=cn&community=scp&type=sidebar&pagename=cn-interwiki-embed-fixture" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
            "\n[[/embed]]\n",
            "[[embed]]\n",
            r#"<iframe src="//interwiki.scpwikicn.com/styleFrame.html?priority=0&type=sidebar&theme=https%3A%2F%2Finterwiki.scpwikicn.com%2Fcss%2Fstyle.css" style="display: none"></iframe>"#,
            "\n[[/embed]]\n",
            "[[embed]]\n",
            r#"<iframe src="//cn.interwiki.scpwikicn.com/interwikiFrame.html?stop=subdomain" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
            "\n[[/embed]]\n",
            "[[embed]]\n",
            r#"<iframe src="http://interwiki.scpwikicn.com/interwikiFrame.html?stop=http" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
            "\n[[/embed]]\n",
            "[[embed]]\n",
            r#"<iframe src="//interwiki.scpwikicn.com/interwikiFrames.html?stop=path-typo" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
            "\n[[/embed]]\n",
            "[[embed]]\n",
            r#"<iframe class="html-block-iframe scpnet-interwiki-frame" src="//interwiki.scpwikicn.com/interwikiFrame.html?stop=reordered" allowtransparency="true"></iframe>"#,
            "\n[[/embed]]\n",
            "[[embed]]\n",
            r#"<iframe src="//interwiki.scpwikicn.com/interwikiFrame.html?stop=added" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame" title="added"></iframe>"#,
            "\n[[/embed]]\n",
            "[!-- [[embed]]\n",
            r#"<iframe src="//interwiki.scpwikicn.com/interwikiFrame.html?stop=comment" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
            "\n[[/embed]] --]\n",
            "[[code]]\n[[embed]]\n",
            r#"<iframe src="//interwiki.scpwikicn.com/interwikiFrame.html?stop=code" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
            "\n[[/embed]]\n[[/code]]\n",
            r#"@@[[embed]]<iframe src="//interwiki.scpwikicn.com/interwikiFrame.html?stop=escape" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>[[/embed]]@@"#,
            "\n[[html]]\n[[embed]]\n",
            r#"<iframe src="//interwiki.scpwikicn.com/interwikiFrame.html?stop=html" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
            "\n[[/embed]]\n[[/html]]\n",
            r#"[[iframe //interwiki.scpwikicn.com/interwikiFrame.html?stop=iframe]]"#,
            "\n[[embed]]\n",
            r#"<iframe src="//interwiki.scpwikicn.com/interwikiFrame.html?stop=unbalanced" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
        ),
    )
    .await;

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
    let html = match view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found page view, got {other:?}"),
    };

    for expected in [
        r#"<iframe src="/-/wikidot-interwiki/interwikiFrame.html?lang=cn&community=scp&type=sidebar&pagename=cn-interwiki-embed-fixture" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
        r#"<iframe src="/-/wikidot-interwiki/styleFrame.html?priority=0&type=sidebar&theme=https%3A%2F%2Finterwiki.scpwikicn.com%2Fcss%2Fstyle.css" style="display: none"></iframe>"#,
    ] {
        assert!(
            html.contains(expected),
            "saved page_view should rewrite the exact CN interwiki iframe and preserve its query verbatim: {html}",
        );
        assert_eq!(html.matches(expected).count(), 1, "{html}");
    }
    assert_eq!(
        html.matches("/-/wikidot-interwiki/").count(),
        2,
        "wildcard/subdomain, HTTP, path typo, attribute changes, malformed embed, literal-owned source, and [[iframe]] must remain outside the exact rewrite contract: {html}",
    );
    for stop in [
        "stop=subdomain",
        "stop=http",
        "stop=path-typo",
        "stop=reordered",
        "stop=added",
        "stop=unbalanced",
        "stop=comment",
        "stop=code",
        "stop=escape",
        "stop=html",
        "stop=iframe",
    ] {
        assert!(
            !html.contains(&format!("/-/wikidot-interwiki/interwikiFrame.html?{stop}")),
            "unsupported boundary {stop:?} must not be rewritten: {html}",
        );
    }
}

#[tokio::test]
async fn saved_page_view_rewrites_cn_interwiki_embeds_after_listpages_expansion() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;
    let slug = "fixture:cn-interwiki-listpages-consumer";

    create_listpages_test_page(
        &mut runner,
        site_id,
        "component:cn-interwiki-parameterized",
        "Parameterized CN Interwiki Fixture",
        concat!(
            "[[module ListPages range=\".\" limit=\"1\"]]\n",
            "[[embed]]\n",
            r#"<iframe src="//interwiki.scpwikicn.com/interwikiFrame.html?lang={$lang}&community={$community}&type={$type}&pagename=%%fullname%%" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
            "\n[[/embed]]\n",
            "[[/module]]",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "component:cn-interwiki-sidebar",
        "CN Interwiki Sidebar Fixture",
        concat!(
            "[[module ListPages range=\".\" limit=\"1\"]]\n",
            "[[embed]]\n",
            r#"<iframe src="//interwiki.scpwikicn.com/interwikiFrame.html?lang=cn&community=scp&type=sidebar&pagename=%%name%%" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
            "\n[[/embed]]\n\n",
            "[[embed]]\n",
            r#"<iframe src="//interwiki.scpwikicn.com/styleFrame.html?priority=0&type=sidebar&theme=https%3A%2F%2Finterwiki.scpwikicn.com%2Fcss%2Fstyle.css" style="display: none"></iframe>"#,
            "\n[[/embed]]\n",
            "[[/module]]\n",
            "[[module ListPages range=\".\" limit=\"1\"]]\n",
            "[[embed]]\n",
            r#"<iframe src="//interwiki.scpwikicn.com/interwikiFrame.html?lang=cn&community=wl&type=sidebar&pagename=%%fullname%%" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
            "\n[[/embed]]\n\n",
            "[[embed]]\n",
            r#"<iframe src="//interwiki.scpwikicn.com/styleFrame.html?priority=0&type=sidebar&theme=https%3A%2F%2Finterwiki.scpwikicn.com%2Fcss%2Fstyle-wl.css" style="display: none"></iframe>"#,
            "\n[[/embed]]\n",
            "[[/module]]",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        slug,
        "CN Interwiki ListPages Consumer",
        concat!(
            "[[include component:cn-interwiki-parameterized\n",
            "|lang=cn\n",
            "|community=scp\n",
            "|type=sidebar\n",
            "]]\n",
            "[[include component:cn-interwiki-sidebar]]",
        ),
    )
    .await;

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
    let html = match view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found page view, got {other:?}"),
    };

    for expected in [
        r#"<iframe src="/-/wikidot-interwiki/interwikiFrame.html?lang=cn&community=scp&type=sidebar&pagename=fixture:cn-interwiki-listpages-consumer" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
        r#"<iframe src="/-/wikidot-interwiki/interwikiFrame.html?lang=cn&community=scp&type=sidebar&pagename=cn-interwiki-listpages-consumer" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
        r#"<iframe src="/-/wikidot-interwiki/styleFrame.html?priority=0&type=sidebar&theme=https%3A%2F%2Finterwiki.scpwikicn.com%2Fcss%2Fstyle.css" style="display: none"></iframe>"#,
        r#"<iframe src="/-/wikidot-interwiki/interwikiFrame.html?lang=cn&community=wl&type=sidebar&pagename=fixture:cn-interwiki-listpages-consumer" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
        r#"<iframe src="/-/wikidot-interwiki/styleFrame.html?priority=0&type=sidebar&theme=https%3A%2F%2Finterwiki.scpwikicn.com%2Fcss%2Fstyle-wl.css" style="display: none"></iframe>"#,
    ] {
        assert_eq!(
            html.matches(expected).count(),
            1,
            "saved page_view should rewrite the retained CN ListPages shape after include and delayed-variable expansion: {html}",
        );
    }
    assert_eq!(html.matches("/-/wikidot-interwiki/").count(), 5, "{html}");
    for unresolved in [
        "{$lang}",
        "{$community}",
        "{$type}",
        "%%fullname%%",
        "%%name%%",
    ] {
        assert!(!html.contains(unresolved), "{unresolved}: {html}");
    }
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
async fn wikidot_gallery_preview_uses_the_typed_no_page_error_boundary() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist")
        .site;
    runner.set_request_context(RequestContext {
        site_id: Some(site.site_id),
        ..Default::default()
    });

    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site.site_id,
            "title": "Gallery no-page fixture",
            "wikitext": concat!(
                "GALLERY_BEFORE\n",
                "[[gallery]]\n",
                "[[code]]\n[[gallery]]\n[[/code]]\n",
                "A[[gallery]]B\n",
                "GALLERY_AFTER",
            ),
        }),
    );

    assert_eq!(
        preview
            .body
            .matches(r#"<div class="error-block">Error selecting page.</div>"#)
            .count(),
        1,
        "only the authored own-line Gallery should execute:\n{}",
        preview.body,
    );
    assert!(
        preview.body.contains("<pre><code>[[gallery]]</code></pre>"),
        "Gallery inside code must remain literal:\n{}",
        preview.body,
    );
    assert!(
        preview.body.contains("A[[gallery]]B"),
        "Gallery in prose must remain literal:\n{}",
        preview.body,
    );
    assert!(
        !preview.body.contains("wj-gallery"),
        "FTML Gallery requirement IDs must not reach served HTML:\n{}",
        preview.body,
    );
}

#[tokio::test]
async fn wikidot_gallery_selects_authorized_current_page_images_after_page_acl() {
    const PAGE_SLUG: &str = "fixture-gallery-current-page";
    const PRIVATE_PAGE_SLUG: &str = "fixture-gallery-private:current-page";
    const PRIVATE_CATEGORY: &str = "fixture-gallery-private";
    const SOURCE: &str = "[[gallery size=\"thumbnail\"]]";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist")
        .site;
    let site_id = site.site_id;
    create_listpages_test_page(
        &mut runner,
        site_id,
        PAGE_SLUG,
        "Gallery current page fixture",
        SOURCE,
    )
    .await;
    let page_id = listpages_test_page_id(&runner, site_id, PAGE_SLUG).await;
    let page = PageTable::find_by_id(page_id)
        .one(runner.context().transaction())
        .await
        .expect("Gallery page lookup should succeed")
        .expect("Gallery page should exist");

    let empty_html = load_listpages_test_compiled_html(&runner, site_id, PAGE_SLUG).await;
    assert!(
        empty_html.contains(
            r#"<div class="error-block">Sorry, we couldn't find any images attached to this page.</div>"#,
        ),
        "a saved empty Gallery must match the frozen Wikidot error contract:\n{empty_html}",
    );

    create_file_fixture_with_mime(&runner, site_id, page_id, "image-b.png", "image/png")
        .await;
    create_file_fixture_with_mime(&runner, site_id, page_id, "notes.txt", "text/plain")
        .await;
    create_file_fixture_with_mime(&runner, site_id, page_id, "image-a.png", "image/png")
        .await;

    let page_info = PageInfo {
        page: Cow::Borrowed(PAGE_SLUG),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("Gallery current page fixture"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let rendered = RenderService::render_page_for_viewer(
        runner.context(),
        SOURCE.to_owned(),
        &page_info,
        Layout::Wikidot,
        PageId {
            site_id,
            category_id: page.page_category_id,
            page_id,
        },
        Some(ADMIN_USER_ID),
        UrlArguments::default(),
    )
    .await
    .expect("authorized current-page Gallery should render")
    .html_output
    .body;

    let first = rendered
        .find("image-a.png")
        .expect("name-ordered Gallery should contain image-a.png");
    let second = rendered
        .find("image-b.png")
        .expect("name-ordered Gallery should contain image-b.png");
    assert!(
        first < second,
        "Gallery files should use name order:\n{rendered}"
    );
    assert!(
        rendered.contains(r#"<div class="gallery-box" id="gallery-box-1">"#),
        "Gallery must expose the frozen Wikidot wrapper:\n{rendered}",
    );
    assert!(
        rendered.contains(r#"class="gallery-image-size-thumbnail""#),
        "Gallery must expose the frozen thumbnail class:\n{rendered}",
    );
    assert!(
        rendered.contains(
            "https://scp-wiki.wjfiles.com/local--files/fixture-gallery-current-page/image-a.png",
        ),
        "Gallery links must retain the original local site/page/file identity:\n{rendered}",
    );
    assert!(
        rendered.contains(
            "https://scp-wiki.wjfiles.com/local--resized-images/fixture-gallery-current-page/image-a.png/thumbnail.jpg",
        ),
        "Gallery images must use the documented resized asset identity:\n{rendered}",
    );
    assert!(!rendered.contains("notes.txt"), "{rendered}");
    assert!(!rendered.contains("wj-gallery"), "{rendered}");

    let explicit = RenderService::render_page_for_viewer(
        runner.context(),
        "[[gallery size=\"thumbnail\"]]\n: image-b.png\n: notes.txt\n[[/gallery]]"
            .to_owned(),
        &page_info,
        Layout::Wikidot,
        PageId {
            site_id,
            category_id: page.page_category_id,
            page_id,
        },
        Some(ADMIN_USER_ID),
        UrlArguments::default(),
    )
    .await
    .expect("explicit current-page Gallery files should render")
    .html_output
    .body;
    assert_eq!(
        explicit.matches("gallery-item thumbnail").count(),
        1,
        "explicit Gallery selection should retain authored order and skip non-images:\n{explicit}",
    );
    assert!(explicit.contains("image-b.png"), "{explicit}");
    assert!(
        explicit.contains(
            r#"<img src="https://scp-wiki.wjfiles.com/local--resized-images/fixture-gallery-current-page/image-b.png/thumbnail.jpg" alt="" class="gallery-image-size-thumbnail" />"#,
        ),
        "explicit filename entries must use the selected resized asset:
{explicit}",
    );
    assert!(
        !explicit.contains(
            r#"<img src="https://scp-wiki.wjfiles.com/local--files/fixture-gallery-current-page/image-b.png" alt="" class="gallery-image-size-thumbnail" />"#,
        ),
        "explicit filename entries must not use the original asset as the image source:
{explicit}",
    );
    assert!(!explicit.contains("image-a.png"), "{explicit}");
    assert!(!explicit.contains("notes.txt"), "{explicit}");

    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        PRIVATE_PAGE_SLUG,
        "Private Gallery current page fixture",
        SOURCE,
    )
    .await;
    set_listpages_test_category_slug(
        &runner,
        site_id,
        PRIVATE_PAGE_SLUG,
        PRIVATE_CATEGORY,
    )
    .await;
    let private_page_id =
        listpages_test_page_id(&runner, site_id, PRIVATE_PAGE_SLUG).await;
    let private_page = PageTable::find_by_id(private_page_id)
        .one(runner.context().transaction())
        .await
        .expect("private Gallery page lookup should succeed")
        .expect("private Gallery page should exist");
    create_file_fixture_with_mime(
        &runner,
        site_id,
        private_page_id,
        "private-gallery-image.png",
        "image/png",
    )
    .await;
    let private_page_info = PageInfo {
        page: Cow::Borrowed(PRIVATE_PAGE_SLUG),
        category: Some(Cow::Borrowed(PRIVATE_CATEGORY)),
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("Private Gallery current page fixture"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let anonymous = RenderService::render_page_for_viewer(
        runner.context(),
        SOURCE.to_owned(),
        &private_page_info,
        Layout::Wikidot,
        PageId {
            site_id,
            category_id: private_page.page_category_id,
            page_id: private_page_id,
        },
        None,
        UrlArguments::default(),
    )
    .await
    .expect("private Gallery should fail closed for an anonymous renderer")
    .html_output
    .body;
    assert!(anonymous.contains("Error selecting page."), "{anonymous}");
    assert!(
        !anonymous.contains("private-gallery-image.png"),
        "{anonymous}"
    );
    assert!(!anonymous.contains("wjfiles"), "{anonymous}");
}

#[tokio::test]
async fn file_mutation_deferred_actions_fence_warm_anonymous_gallery_cache() {
    const FILE_NAME: &str = "cache-fresh-gallery-image.png";
    const EMPTY_GALLERY: &str =
        "Sorry, we couldn't find any images attached to this page.";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    let (page_id, slug) =
        import_cacheable_gallery_fixture(&mut runner, site.site_id).await;

    let warm = load_cached_gallery_article(&mut runner, site.site_id, &slug).await;
    assert!(warm.body.contains(EMPTY_GALLERY), "{}", warm.body);

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(page_id),
    );
    let mut png = vec![
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0,
        0, 1, 8, 4, 0, 0, 0, 181, 28, 12, 2, 0, 0, 0, 11, 73, 68, 65, 84, 120, 218, 99,
        100, 248, 15, 0, 1, 5, 1, 1, 39, 24, 227, 102, 0, 0, 0, 0, 73, 69, 78, 68, 174,
        66, 96, 130,
    ];
    let pending_blob_id = cuid();
    png.extend_from_slice(pending_blob_id.as_bytes());
    let (pending, png) = create_committed_page_pending_blob_with_data_fixture(
        &runner,
        ADMIN_USER_ID,
        site.site_id,
        page_id,
        pending_blob_id,
        png,
    )
    .await;
    let created = run_endpoint!(
        runner,
        file_create,
        json!({
            "site_id": site.site_id,
            "page_id": page_id,
            "name": FILE_NAME,
            "uploaded_blob_id": pending.pending_blob_id.clone(),
            "revision_comments": "create Gallery cache image",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let fresh =
        complete_gallery_file_mutation(&mut runner, site.site_id, page_id, &slug, &warm)
            .await;
    assert!(fresh.body.contains(FILE_NAME), "{}", fresh.body);
    assert!(!fresh.body.contains(EMPTY_GALLERY), "{}", fresh.body);

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(page_id),
    );
    run_endpoint!(
        runner,
        file_delete,
        json!({
            "site_id": site.site_id,
            "page_id": page_id,
            "file": created.file_id,
            "last_revision_id": created.file_revision_id,
            "revision_comments": "delete Gallery cache image",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let deleted =
        complete_gallery_file_mutation(&mut runner, site.site_id, page_id, &slug, &fresh)
            .await;
    assert!(deleted.body.contains(EMPTY_GALLERY), "{}", deleted.body);
    assert!(!deleted.body.contains(FILE_NAME), "{}", deleted.body);

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(page_id),
    );
    run_endpoint!(
        runner,
        file_restore,
        json!({
            "site_id": site.site_id,
            "page_id": page_id,
            "file_id": created.file_id,
            "revision_comments": "restore Gallery cache image",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let restored = complete_gallery_file_mutation(
        &mut runner,
        site.site_id,
        page_id,
        &slug,
        &deleted,
    )
    .await;
    assert!(restored.body.contains(FILE_NAME), "{}", restored.body);
    assert!(!restored.body.contains(EMPTY_GALLERY), "{}", restored.body);

    cleanup_committed_page_pending_blob_fixture(runner.state(), &pending, &png)
        .await
        .expect("Gallery PNG fixture cleanup should succeed");
}

#[tokio::test]
async fn changed_file_revision_fences_but_noop_and_failed_revision_do_not() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    let page = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": "home"}),
    )
    .expect("seeded home page should exist");
    let pending_blob_id = create_prefinalized_empty_page_blob_fixture(
        &runner,
        ADMIN_USER_ID,
        site.site_id,
        page.page_id,
    )
    .await;
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(page.page_id),
    );
    let created = run_endpoint!(
        runner,
        file_create,
        json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "name": "file-revision-fence.png",
            "uploaded_blob_id": pending_blob_id,
            "revision_comments": "create file revision fence fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    runner
        .context()
        .run_post_commit_actions()
        .await
        .expect("file fixture post-commit actions should complete");
    let initial_fence = PublicContentCache::cache_fence(runner.context(), site.site_id)
        .await
        .expect("initial public-content fence should be readable");

    let no_op = run_endpoint!(
        runner,
        file_edit,
        json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "file_id": created.file_id,
            "last_revision_id": created.file_revision_id,
            "revision_comments": "semantic no-op",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(no_op.is_none());
    runner
        .context()
        .run_post_commit_actions()
        .await
        .expect("no-op post-commit drain should succeed");
    assert_eq!(
        PublicContentCache::cache_fence(runner.context(), site.site_id)
            .await
            .expect("post-no-op fence should be readable"),
        initial_fence,
    );

    let error = run_endpoint_err!(
        runner,
        file_edit,
        json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "file_id": created.file_id,
            "last_revision_id": created.file_revision_id,
            "revision_comments": "invalid empty-name edit",
            "user_id": ADMIN_USER_ID,
            "name": "",
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::FileNameEmpty);
    runner
        .context()
        .run_post_commit_actions()
        .await
        .expect("failed edit post-commit drain should succeed");
    assert_eq!(
        PublicContentCache::cache_fence(runner.context(), site.site_id)
            .await
            .expect("post-failure fence should be readable"),
        initial_fence,
    );

    run_endpoint!(
        runner,
        file_edit,
        json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "file_id": created.file_id,
            "last_revision_id": created.file_revision_id,
            "revision_comments": "changed name",
            "user_id": ADMIN_USER_ID,
            "name": "file-revision-fence-renamed.png",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("changed file edit should create a revision");
    runner
        .context()
        .run_post_commit_actions()
        .await
        .expect("changed edit post-commit actions should complete");
    assert_ne!(
        PublicContentCache::cache_fence(runner.context(), site.site_id)
            .await
            .expect("post-change fence should be readable"),
        initial_fence,
    );
}

#[tokio::test]
async fn wikidot_gallery_explicit_entries_resolve_only_owned_visible_files() {
    const TARGET_SLUG: &str = "fixture-gallery-explicit-target";
    const FILE_NAME: &str = "gallery image.png";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist")
        .site;
    create_listpages_test_page(
        &mut runner,
        site.site_id,
        TARGET_SLUG,
        "Gallery explicit target",
        "Gallery target page.",
    )
    .await;
    let target_page_id = listpages_test_page_id(&runner, site.site_id, TARGET_SLUG).await;
    create_file_fixture_with_mime(
        &runner,
        site.site_id,
        target_page_id,
        FILE_NAME,
        "image/png",
    )
    .await;
    runner.set_request_context(RequestContext {
        site_id: Some(site.site_id),
        ..Default::default()
    });

    let source = concat!(
        "[[gallery size=\"thumbnail\"]]\n",
        ": https://scp-wiki.wikidot.com/local--files/fixture-gallery-explicit-target/gallery%20image.png\n",
        ": https://example.invalid/local--files/fixture-gallery-explicit-target/gallery%20image.png\n",
        ": https://scp-jp.wikidot.com/local--files/fixture-gallery-explicit-target/gallery%20image.png\n",
        "[[/gallery]]",
    );
    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site.site_id,
            "title": "Gallery explicit entries",
            "wikitext": source,
        }),
    );

    assert_eq!(
        preview.body.matches("gallery-item thumbnail").count(),
        1,
        "only the exact site-owned file may become a Gallery row:\n{}",
        preview.body,
    );
    assert!(
        preview.body.contains(
            "https://scp-wiki.wjfiles.com/local--files/fixture-gallery-explicit-target/gallery%20image.png",
        ),
        "the resolved entry must use the local owned-file route:\n{}",
        preview.body,
    );
    assert!(
        preview.body.contains(
            r#"<img src="https://scp-wiki.wjfiles.com/local--files/fixture-gallery-explicit-target/gallery%20image.png" alt="" class="gallery-image-size-thumbnail" />"#,
        ),
        "an explicit owned URL must remain the image source:\n{}",
        preview.body,
    );
    assert!(
        !preview.body.contains("example.invalid"),
        "{}",
        preview.body
    );
    assert!(!preview.body.contains("scp-jp"), "{}", preview.body);
    assert!(!preview.body.contains("wj-gallery"), "{}", preview.body);
}

#[tokio::test]
async fn wikidot_gallery_preview_enforces_size_viewer_and_invalid_option_matrix() {
    const TARGET_SLUG: &str = "fixture-gallery-options-preview-target";
    const FILE_NAME: &str = "gallery-options.png";
    const SELECTION_ERROR: &str =
        r#"<div class="error-block">Error selecting page.</div>"#;

    struct RenderCase {
        case_id: &'static str,
        arguments: &'static str,
        expected_size: &'static str,
    }

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist")
        .site;
    create_listpages_test_page(
        &mut runner,
        site.site_id,
        TARGET_SLUG,
        "Gallery options preview target",
        "Gallery options target page.",
    )
    .await;
    let target_page_id = listpages_test_page_id(&runner, site.site_id, TARGET_SLUG).await;
    create_file_fixture_with_mime(
        &runner,
        site.site_id,
        target_page_id,
        FILE_NAME,
        "image/png",
    )
    .await;
    runner.set_request_context(RequestContext {
        site_id: Some(site.site_id),
        ..Default::default()
    });

    let valid_cases = [
        RenderCase {
            case_id: "M1043_SIZE_DEFAULT",
            arguments: "",
            expected_size: "thumbnail",
        },
        RenderCase {
            case_id: "M1043_SIZE_SQUARE",
            arguments: r#"size="square""#,
            expected_size: "square",
        },
        RenderCase {
            case_id: "M1043_SIZE_THUMBNAIL",
            arguments: r#"size="thumbnail""#,
            expected_size: "thumbnail",
        },
        RenderCase {
            case_id: "M1043_SIZE_SMALL",
            arguments: r#"size="small""#,
            expected_size: "small",
        },
        RenderCase {
            case_id: "M1043_SIZE_MEDIUM",
            arguments: r#"size="medium""#,
            expected_size: "medium",
        },
        RenderCase {
            case_id: "M1043_VIEWER_YES",
            arguments: r#"viewer="yes""#,
            expected_size: "thumbnail",
        },
        RenderCase {
            case_id: "M1043_VIEWER_TRUE",
            arguments: r#"viewer="true""#,
            expected_size: "thumbnail",
        },
        RenderCase {
            case_id: "M1043_VIEWER_NO",
            arguments: r#"viewer="no""#,
            expected_size: "thumbnail",
        },
        RenderCase {
            case_id: "M1043_VIEWER_FALSE",
            arguments: r#"viewer="false""#,
            expected_size: "thumbnail",
        },
    ];
    for case in valid_cases {
        let arguments = if case.arguments.is_empty() {
            String::new()
        } else {
            format!(" {}", case.arguments)
        };
        let source = format!(
            "[[gallery{arguments}]]\n: https://scp-wiki.wikidot.com/local--files/{TARGET_SLUG}/{FILE_NAME}\n[[/gallery]]",
        );
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site.site_id,
                "title": case.case_id,
                "wikitext": source,
            }),
        );

        assert_eq!(
            preview.body.matches("gallery-item ").count(),
            1,
            "{} should render exactly one owned image:\n{}",
            case.case_id,
            preview.body,
        );
        assert!(
            preview
                .body
                .contains(&format!(r#"class="gallery-item {}""#, case.expected_size)),
            "{} should use the independent expected item size {}:\n{}",
            case.case_id,
            case.expected_size,
            preview.body,
        );
        assert!(
            preview.body.contains(&format!(
                r#"class="gallery-image-size-{}""#,
                case.expected_size,
            )),
            "{} should use the independent expected image size {}:\n{}",
            case.case_id,
            case.expected_size,
            preview.body,
        );
        assert!(
            preview.body.contains(&format!(
                r#"<img src="https://scp-wiki.wjfiles.com/local--files/{TARGET_SLUG}/{FILE_NAME}" alt="" class="gallery-image-size-{}""#,
                case.expected_size,
            )),
            "{} should preserve the original asset for an owned explicit URL:\n{}",
            case.case_id,
            preview.body,
        );
        assert!(
            !preview.body.contains(&format!(
                "/local--resized-images/{TARGET_SLUG}/{FILE_NAME}/{}.jpg",
                case.expected_size,
            )),
            "{} must not replace an owned explicit URL with a resized asset:\n{}",
            case.case_id,
            preview.body,
        );
        assert!(
            preview.body.contains(r#"class="with-lb""#),
            "{} should retain the static lightbox anchor; viewer activation is a browser concern:\n{}",
            case.case_id,
            preview.body,
        );
    }

    for (case_id, arguments) in [
        ("M1043_INVALID_SIZE", r#"size="large""#),
        ("M1043_EMPTY_LAST_SIZE", r#"size="medium" size="""#),
        ("M1043_INVALID_ORDER", r#"order="score""#),
        ("M1043_EMPTY_LAST_ORDER", r#"order="name" order="""#),
        ("M1043_INVALID_VIEWER", r#"viewer="maybe""#),
        ("M1043_EMPTY_LAST_VIEWER", r#"viewer="yes" viewer="""#),
    ] {
        let source = format!(
            "[[gallery {arguments}]]\n: https://scp-wiki.wikidot.com/local--files/{TARGET_SLUG}/{FILE_NAME}\n[[/gallery]]",
        );
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site.site_id,
                "title": case_id,
                "wikitext": source,
            }),
        );

        assert_eq!(
            preview.body, SELECTION_ERROR,
            "{case_id} must fail closed at the exact public selection-error boundary",
        );
    }
}

#[tokio::test]
async fn wikidot_gallery_saved_page_view_enforces_order_and_last_occurrence_matrix() {
    const PAGE_SLUG: &str = "fixture-gallery-options-saved";
    const ALPHA_NEW: &str = "alpha-new.png";
    const ZULU_OLD: &str = "zulu-old.png";

    struct OrderCase {
        case_id: &'static str,
        arguments: &'static str,
        first: &'static str,
        second: &'static str,
    }

    let cases = [
        OrderCase {
            case_id: "M1043_ORDER_DEFAULT",
            arguments: "",
            first: ALPHA_NEW,
            second: ZULU_OLD,
        },
        OrderCase {
            case_id: "M1043_ORDER_NAME",
            arguments: r#"order="name""#,
            first: ALPHA_NEW,
            second: ZULU_OLD,
        },
        OrderCase {
            case_id: "M1043_ORDER_NAME_DESC",
            arguments: r#"order="name desc""#,
            first: ZULU_OLD,
            second: ALPHA_NEW,
        },
        OrderCase {
            case_id: "M1043_ORDER_NAME_DESC_ALIAS",
            arguments: r#"order="nameDesc""#,
            first: ZULU_OLD,
            second: ALPHA_NEW,
        },
        OrderCase {
            case_id: "M1043_ORDER_CREATED_AT",
            arguments: r#"order="created_at""#,
            first: ZULU_OLD,
            second: ALPHA_NEW,
        },
        OrderCase {
            case_id: "M1043_ORDER_DATE_ADDED_ALIAS",
            arguments: r#"order="dateAdded""#,
            first: ZULU_OLD,
            second: ALPHA_NEW,
        },
        OrderCase {
            case_id: "M1043_ORDER_CREATED_AT_DESC",
            arguments: r#"order="created_at desc""#,
            first: ALPHA_NEW,
            second: ZULU_OLD,
        },
        OrderCase {
            case_id: "M1043_ORDER_DATE_ADDED_DESC_ALIAS",
            arguments: r#"order="dateAddedDesc""#,
            first: ALPHA_NEW,
            second: ZULU_OLD,
        },
        OrderCase {
            case_id: "M1043_LAST_CANONICAL_OCCURRENCE",
            arguments: concat!(
                r#"size="square" size="medium" "#,
                r#"order="name desc" order="created_at" "#,
                r#"viewer="no" viewer="yes""#,
            ),
            first: ZULU_OLD,
            second: ALPHA_NEW,
        },
    ];
    let source = cases
        .iter()
        .map(|case| {
            let arguments = if case.arguments.is_empty() {
                String::new()
            } else {
                format!(" {}", case.arguments)
            };
            format!(
                "{}_BEGIN\n[[gallery{arguments}]]\n{}_END",
                case.case_id, case.case_id,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist")
        .site;
    create_listpages_test_page(
        &mut runner,
        site.site_id,
        PAGE_SLUG,
        "Gallery saved options matrix",
        &source,
    )
    .await;
    let page_id = listpages_test_page_id(&runner, site.site_id, PAGE_SLUG).await;
    let page = PageTable::find_by_id(page_id)
        .one(runner.context().transaction())
        .await
        .expect("Gallery options page lookup should succeed")
        .expect("Gallery options page should exist");
    let zulu_file_id = create_file_fixture_with_mime(
        &runner,
        site.site_id,
        page_id,
        ZULU_OLD,
        "image/png",
    )
    .await;
    let alpha_file_id = create_file_fixture_with_mime(
        &runner,
        site.site_id,
        page_id,
        ALPHA_NEW,
        "image/png",
    )
    .await;
    for (file_id, created_at) in [
        (
            zulu_file_id,
            OffsetDateTime::from_unix_timestamp(1_600_000_000)
                .expect("old fixture timestamp should be valid"),
        ),
        (
            alpha_file_id,
            OffsetDateTime::from_unix_timestamp(1_700_000_000)
                .expect("new fixture timestamp should be valid"),
        ),
    ] {
        let file = file::Entity::find_by_id(file_id)
            .one(runner.context().transaction())
            .await
            .expect("Gallery option file lookup should succeed")
            .expect("Gallery option file should exist");
        let mut file = file.into_active_model();
        file.created_at = Set(created_at);
        file.update(runner.context().transaction())
            .await
            .expect("Gallery option fixture timestamp should update");
    }

    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site.site_id,
            "category_id": page.page_category_id,
            "page_id": page_id,
        }),
    );
    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site.site_id,
            "session_token": null,
            "route": {"slug": PAGE_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let rendered = match view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected Gallery options page view, got {other:?}"),
    };

    for case in cases {
        let begin = format!("{}_BEGIN", case.case_id);
        let end = format!("{}_END", case.case_id);
        let start = rendered.find(&begin).unwrap_or_else(|| {
            panic!("{} should expose its begin sentinel", case.case_id)
        });
        let tail = &rendered[start + begin.len()..];
        let finish = tail
            .find(&end)
            .unwrap_or_else(|| panic!("{} should expose its end sentinel", case.case_id));
        let fragment = &tail[..finish];
        let first = fragment.find(case.first).unwrap_or_else(|| {
            panic!(
                "{} should contain {}:\n{fragment}",
                case.case_id, case.first
            )
        });
        let second = fragment.find(case.second).unwrap_or_else(|| {
            panic!(
                "{} should contain {}:\n{fragment}",
                case.case_id, case.second,
            )
        });
        assert!(
            first < second,
            "{} should order {} before {} from independent fixture dimensions:\n{}",
            case.case_id,
            case.first,
            case.second,
            fragment,
        );

        if case.case_id == "M1043_LAST_CANONICAL_OCCURRENCE" {
            assert_eq!(
                fragment.matches(r#"class="gallery-item medium""#).count(),
                2,
                "the last canonical size occurrence should replace the earlier size:\n{fragment}",
            );
            assert_eq!(
                fragment.matches(r#"class="with-lb""#).count(),
                2,
                "the last canonical viewer occurrence should replace the earlier viewer:\n{fragment}",
            );
            assert!(
                !fragment.contains("gallery-item square"),
                "the replaced size must not survive:\n{fragment}",
            );
        }
    }
}

#[tokio::test]
async fn wikidot_files_saved_modules_use_distinct_container_suffixes() {
    const SLUG: &str = "fixture-files-module-instance-distinct";
    const SOURCE: &str =
        "FILES_ONE\n[[module Files]]\nFILES_TWO\n[[module Files]]\nFILES_END";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist")
        .site;
    create_listpages_test_page(
        &mut runner,
        site.site_id,
        SLUG,
        "Files module instance distinct fixture",
        SOURCE,
    )
    .await;
    let page_id = listpages_test_page_id(&runner, site.site_id, SLUG).await;
    let saved = saved_article_view_body(&runner, site.site_id, SLUG).await;

    assert!(saved.contains("FILES_ONE"), "{saved}");
    assert!(saved.contains("FILES_TWO"), "{saved}");
    assert!(!saved.contains("No such module"), "{saved}");

    let containers = files_module_container_suffixes(&saved);
    assert_eq!(
        containers.len(),
        2,
        "one saved render should emit one container per Files module:\n{saved}",
    );
    assert_ne!(
        containers[0], containers[1],
        "each Files module in one render needs its own instance suffix:\n{saved}",
    );
    assert_eq!(
        files_module_function_suffixes(&saved),
        containers,
        "each refresh function must use its own container suffix:\n{saved}",
    );
    assert_eq!(
        files_module_selector_suffixes(&saved),
        containers,
        "each refresh selector must use its own container suffix:\n{saved}",
    );
    for suffix in &containers {
        assert!(
            !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit()),
            "saved Files suffixes must be unpadded decimal digits: {suffix:?}",
        );
        assert_ne!(
            suffix,
            &page_id.to_string(),
            "the module-instance suffix must stay distinct from the saved page id:\n{saved}",
        );
    }
    assert_eq!(
        saved.matches(&format!("p.page_id={page_id};")).count(),
        2,
        "every refresh script must keep the real saved page id:\n{saved}",
    );
}

#[tokio::test]
async fn wikidot_files_and_flickr_modules_match_the_frozen_empty_contracts() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist")
        .site;
    runner.set_request_context(RequestContext {
        site_id: Some(site.site_id),
        ..Default::default()
    });
    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site.site_id,
            "title": "Files preview fixture",
            "wikitext": "FILES_BEFORE\n[[module Files]]\nFILES_AFTER",
        }),
    );
    assert!(
        preview.body.contains(r#"<div id="files-">"#),
        "Files PagePreview must retain the evidenced empty-id state:\n{}",
        preview.body,
    );
    assert!(
        preview.body.contains("No files attached to this page."),
        "{}",
        preview.body,
    );
    assert!(
        preview.body.contains("Manage attachments"),
        "{}",
        preview.body,
    );
    assert!(!preview.body.contains("No such module"), "{}", preview.body);

    let flickr = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site.site_id,
            "title": "Flickr preview fixture",
            "wikitext": "FLICKR_BEFORE\n[[module FlickrGallery]]\nFLICKR_AFTER",
        }),
    );
    for expected in [
        r#"<div class="flickr-gallery-box makeHoverTitles">"#,
        "Sorry, no photos.",
        "moduleName ::: edit/PagePreviewModule",
        "mode ::: page",
        "source ::: [[module FlickrGallery]]",
        "title ::: Flickr preview fixture",
    ] {
        assert!(
            flickr.body.contains(expected),
            "Flickr PagePreview should contain {expected:?}:\n{}",
            flickr.body,
        );
    }
    assert!(!flickr.body.contains("No such module"), "{}", flickr.body);
}

#[tokio::test]
async fn wikidot_files_saved_empty_module_matches_the_live_container_and_refresh_script()
{
    const SLUG: &str = "fixture-files-module-saved-empty";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist")
        .site;
    create_listpages_test_page(
        &mut runner,
        site.site_id,
        SLUG,
        "Files module saved empty fixture",
        "FILES_BEFORE\n[[module Files]]\nFILES_AFTER",
    )
    .await;
    let page_id = listpages_test_page_id(&runner, site.site_id, SLUG).await;
    let saved = saved_article_view_body(&runner, site.site_id, SLUG).await;

    assert!(saved.contains("FILES_BEFORE"), "{saved}");
    assert!(saved.contains("FILES_AFTER"), "{saved}");
    assert!(!saved.contains("No such module"), "{saved}");

    let containers = files_module_container_suffixes(&saved);
    assert_eq!(containers.len(), 1, "{saved}");
    let suffix = &containers[0];
    assert!(
        !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit()),
        "saved empty Files suffixes are unpadded decimals: {suffix:?}",
    );
    assert_ne!(suffix, &page_id.to_string(), "{saved}");
    assert!(
        saved.contains(&format!(r#"<div id="files-{suffix}">"#)),
        "the saved container carries only the instance id:\n{saved}",
    );
    assert!(saved.contains("No files attached to this page."), "{saved}");
    assert!(
        saved.contains(&format!("function updateFileSimpleList{suffix}(pageNo)")),
        "{saved}",
    );
    assert!(
        saved.contains(&format!("var containerElId = 'files-{suffix}';")),
        "{saved}",
    );
    assert!(saved.contains(&format!("p.page_id={page_id};")), "{saved}");
    assert!(
        saved.contains(r#"OZONE.ajax.requestModule("files/PageFilesSimpleModule""#),
        "{saved}",
    );
    assert!(
        saved.contains(r#"class="manage-attachments-link""#),
        "{saved}",
    );
    assert!(
        saved.contains("WIKIDOT.page.listeners.filesClick(null)"),
        "{saved}",
    );
    assert!(!saved.contains("page-files"), "{saved}");
}

#[tokio::test]
async fn wikidot_files_saved_populated_module_matches_the_live_row_contract() {
    const SLUG: &str = "fixture-files-module-populated";
    const JPEG_NAME: &str = "that man&\".jpg";
    const JPEG_LABEL: &str = "JPEG image data";
    const JPEG_DESCRIPTION: &str = "JPEG image data, EXIF standard";
    const PNG_NAME: &str = "zz-megabyte.png";
    const PNG_LABEL: &str = "PNG image data";
    const PNG_DESCRIPTION: &str =
        "PNG image data, 1 x 1, 8-bit/color RGBA, non-interlaced";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist")
        .site;
    create_listpages_test_page(
        &mut runner,
        site.site_id,
        SLUG,
        "Files module populated fixture",
        "FILES_BEFORE\n[[module Files]]\nFILES_AFTER",
    )
    .await;
    let page_id = listpages_test_page_id(&runner, site.site_id, SLUG).await;
    let jpeg_file_id = create_file_fixture_with_descriptor(
        &runner,
        site.site_id,
        page_id,
        JPEG_NAME,
        180_296,
        Some(ContentTypeDescriptor {
            label: JPEG_LABEL.to_owned(),
            description: JPEG_DESCRIPTION.to_owned(),
        }),
    )
    .await;
    let png_file_id = create_file_fixture_with_descriptor(
        &runner,
        site.site_id,
        page_id,
        PNG_NAME,
        3_145_728,
        Some(ContentTypeDescriptor {
            label: PNG_LABEL.to_owned(),
            description: PNG_DESCRIPTION.to_owned(),
        }),
    )
    .await;

    let saved = saved_article_view_body(&runner, site.site_id, SLUG).await;
    assert!(!saved.contains("No such module"), "{saved}");

    let containers = files_module_container_suffixes(&saved);
    assert_eq!(containers.len(), 1, "{saved}");
    let suffix = &containers[0];
    assert!(
        saved.contains(&format!(r#"<div id="files-{suffix}">"#)),
        "{saved}",
    );
    assert!(
        saved.contains(&format!("function updateFileSimpleList{suffix}(pageNo)")),
        "{saved}",
    );
    assert!(
        saved.contains(&format!("var containerElId = 'files-{suffix}';")),
        "{saved}",
    );
    assert!(saved.contains(&format!("p.page_id={page_id};")), "{saved}");
    assert_ne!(suffix, &page_id.to_string(), "{saved}");

    assert!(
        saved.contains(
            r#"<table class="page-files"><tr><th>File name</th><th>File type</th><th>Size</th><th></th></tr>"#
        ),
        "the observed header has exactly the four live columns:\n{saved}",
    );
    assert_eq!(
        saved.matches("<th>").count(),
        4,
        "the observed table has no date or user column:\n{saved}",
    );

    assert!(
        saved.contains(
            r#"<a href="/local--files/fixture-files-module-populated/that%20man%26%22.jpg">that man&amp;".jpg</a>"#
        ),
        "the escaped name owns the local--files href:\n{saved}",
    );
    assert!(
        saved.contains(&format!(
            r#"<span title="{JPEG_DESCRIPTION}">{JPEG_LABEL}</span>"#
        )),
        "{saved}",
    );
    assert!(
        saved.contains(&format!(
            r#"<span title="{PNG_DESCRIPTION}">{PNG_LABEL}</span>"#
        )),
        "{saved}",
    );
    assert!(saved.contains("176.07 kB"), "{saved}");
    assert!(saved.contains("3 MB"), "{saved}");
    assert!(
        saved.contains(&format!(
            r#"onclick="WIKIDOT.modules.PageFilesModule.listeners.fileMoreInfo(event,{jpeg_file_id})""#
        )),
        "{saved}",
    );
    assert!(
        saved.contains(&format!(
            r#"onclick="WIKIDOT.modules.PageFilesModule.listeners.fileMoreInfo(event,{png_file_id})""#
        )),
        "{saved}",
    );
    let jpeg_row = saved
        .find(r#"that man&amp;".jpg"#)
        .expect("the escaped JPEG name should render");
    let png_row = saved.find(PNG_NAME).expect("the PNG name should render");
    assert!(
        jpeg_row < png_row,
        "rows keep the observed ascending name order:\n{saved}",
    );
    assert!(
        saved.contains(r#"class="manage-attachments-link""#),
        "{saved}",
    );
    assert!(
        saved.contains("WIKIDOT.page.listeners.filesClick(null)"),
        "{saved}",
    );
}

#[tokio::test]
async fn wikidot_mirror_files_module_omits_local_fixture_files_from_source_inventory() {
    const SLUG: &str = "fixture-imported-files-module-origin";
    const SOURCE_NAME: &str = "theend.jpg";
    const LOCAL_NAME: &str = "SCP.txt";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist")
        .site;
    create_listpages_test_page(
        &mut runner,
        site.site_id,
        SLUG,
        "Imported Files module provenance fixture",
        "[[module Files]]",
    )
    .await;
    let page_id = listpages_test_page_id(&runner, site.site_id, SLUG).await;
    let source_file_id = create_file_fixture_with_descriptor(
        &runner,
        site.site_id,
        page_id,
        SOURCE_NAME,
        27_980,
        Some(ContentTypeDescriptor {
            label: "JPEG image data".to_owned(),
            description: "JPEG image data".to_owned(),
        }),
    )
    .await;
    let local_file_id = create_file_fixture_with_descriptor(
        &runner,
        site.site_id,
        page_id,
        LOCAL_NAME,
        27_980,
        Some(ContentTypeDescriptor {
            label: "text/plain".to_owned(),
            description: "Plain text".to_owned(),
        }),
    )
    .await;

    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "UPDATE site SET from_wikidot = TRUE WHERE site_id = $1",
            [Value::from(site.site_id)],
        ))
        .await
        .expect("fixture site should be marked as a source mirror");
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "UPDATE page SET from_wikidot = TRUE WHERE page_id = $1",
            [Value::from(page_id)],
        ))
        .await
        .expect("fixture page should be marked as imported");
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "UPDATE file SET from_wikidot = TRUE WHERE file_id = $1",
            [Value::from(source_file_id)],
        ))
        .await
        .expect("fixture should distinguish source and local file origin");
    assert!(
        file::Entity::find_by_id(source_file_id)
            .one(transaction)
            .await
            .expect("source file origin should load")
            .expect("source file should exist")
            .from_wikidot
    );
    assert!(
        !file::Entity::find_by_id(local_file_id)
            .one(transaction)
            .await
            .expect("local file origin should load")
            .expect("local file should exist")
            .from_wikidot
    );
    assert!(
        PageTable::find_by_id(page_id)
            .one(transaction)
            .await
            .expect("page origin should load")
            .expect("page should exist")
            .from_wikidot
    );
    assert!(
        deepwell::models::site::Entity::find_by_id(site.site_id)
            .one(transaction)
            .await
            .expect("site origin should load")
            .expect("site should exist")
            .from_wikidot
    );
    rerender_file_fixture_page(&runner, page_id).await;

    let saved = saved_article_view_body(&runner, site.site_id, SLUG).await;
    assert!(
        saved.contains(SOURCE_NAME),
        "source attachment should appear:\n{saved}"
    );
    assert!(
        !saved.contains(LOCAL_NAME),
        "local fixture attachment must not appear in a source mirror inventory:\n{saved}",
    );
}

#[tokio::test]
async fn wikidot_files_saved_view_requires_page_view_and_complete_descriptors() {
    const DENIED_SLUG: &str = "fixture-files-denied:module-view";
    const DENIED_CATEGORY: &str = "fixture-files-denied";
    const MISSING_DESCRIPTOR_SLUG: &str = "fixture-files-module-missing-descriptor";
    const MISSING_REVISION_SLUG: &str = "fixture-files-module-missing-revision";
    const SMALL_SIZE_SLUG: &str = "fixture-files-module-small-size";
    const OVERFLOW_SLUG: &str = "fixture-files-module-overflow";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist")
        .site;

    // A page the anonymous viewer cannot see must leave the module literal:
    // no page id, file name, container, or refresh script may escape.
    make_listpages_test_category_admin_only(&runner, site.site_id, DENIED_CATEGORY).await;
    create_listpages_test_page(
        &mut runner,
        site.site_id,
        DENIED_SLUG,
        "Denied Files module fixture",
        "[[module Files]]",
    )
    .await;
    set_listpages_test_category_slug(&runner, site.site_id, DENIED_SLUG, DENIED_CATEGORY)
        .await;
    let denied_page_id = listpages_test_page_id(&runner, site.site_id, DENIED_SLUG).await;
    create_file_fixture_with_mime(
        &runner,
        site.site_id,
        denied_page_id,
        "denied-must-not-leak.png",
        "image/png",
    )
    .await;
    let denied_page = PageTable::find_by_id(denied_page_id)
        .one(runner.context().transaction())
        .await
        .expect("denied Files page lookup should succeed")
        .expect("denied Files page should exist");
    let denied_page_info = PageInfo {
        page: Cow::Borrowed(DENIED_SLUG),
        category: Some(Cow::Borrowed(DENIED_CATEGORY)),
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("Denied Files module fixture"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let denied = RenderService::render_page_for_viewer(
        runner.context(),
        "[[module Files]]".to_owned(),
        &denied_page_info,
        Layout::Wikidot,
        PageId {
            site_id: site.site_id,
            category_id: denied_page.page_category_id,
            page_id: denied_page_id,
        },
        None,
        UrlArguments::default(),
    )
    .await
    .expect("a denied Files module should fail closed")
    .html_output
    .body;
    assert!(denied.contains("No such module"), "{denied}");
    assert!(!denied.contains(r#"id="files-"#), "{denied}");
    assert!(!denied.contains(&denied_page_id.to_string()), "{denied}");
    assert!(!denied.contains("denied-must-not-leak.png"), "{denied}");
    assert!(!denied.contains("updateFileSimpleList"), "{denied}");

    // A missing page fails closed the same way and has no file inventory to
    // query at all.
    let missing_page_info = PageInfo {
        page: Cow::Borrowed("fixture-files-module-missing-page"),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("Missing Files module fixture"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let missing_page_target_id = i64::MAX;
    let missing_page = RenderService::render_page_for_viewer(
        runner.context(),
        "[[module Files]]".to_owned(),
        &missing_page_info,
        Layout::Wikidot,
        PageId {
            site_id: site.site_id,
            category_id: denied_page.page_category_id,
            page_id: missing_page_target_id,
        },
        None,
        UrlArguments::default(),
    )
    .await
    .expect("a missing Files page should fail closed")
    .html_output
    .body;
    assert!(missing_page.contains("No such module"), "{missing_page}");
    assert!(!missing_page.contains(r#"id="files-"#), "{missing_page}");
    assert!(
        !missing_page.contains(&missing_page_target_id.to_string()),
        "{missing_page}"
    );
    assert!(
        !missing_page.contains("updateFileSimpleList"),
        "{missing_page}"
    );

    create_listpages_test_page(
        &mut runner,
        site.site_id,
        MISSING_DESCRIPTOR_SLUG,
        "Files module missing descriptor fixture",
        "[[module Files]]",
    )
    .await;
    let missing_page_id =
        listpages_test_page_id(&runner, site.site_id, MISSING_DESCRIPTOR_SLUG).await;
    create_file_fixture_with_descriptor(
        &runner,
        site.site_id,
        missing_page_id,
        "must-not-leak.bin",
        12_345,
        None,
    )
    .await;
    let missing_view = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site.site_id,
            "session_token": null,
            "route": {"slug": MISSING_DESCRIPTOR_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let GetPageViewOutput::Found {
        compiled_body_html: missing_html,
        ..
    } = missing_view.page
    else {
        panic!("missing-descriptor Files fixture should still serve its page");
    };
    assert!(
        !missing_html.contains("must-not-leak.bin"),
        "{missing_html}"
    );
    assert!(!missing_html.contains("page-files"), "{missing_html}");

    create_listpages_test_page(
        &mut runner,
        site.site_id,
        MISSING_REVISION_SLUG,
        "Files module missing revision fixture",
        "[[module Files]]",
    )
    .await;
    let missing_revision_page_id =
        listpages_test_page_id(&runner, site.site_id, MISSING_REVISION_SLUG).await;
    file::ActiveModel {
        name: Set("orphan-must-not-leak.bin".to_owned()),
        site_id: Set(site.site_id),
        page_id: Set(missing_revision_page_id),
        ..Default::default()
    }
    .insert(runner.context().transaction())
    .await
    .expect("orphan file fixture should be inserted");
    rerender_file_fixture_page(&runner, missing_revision_page_id).await;
    let missing_revision_view = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site.site_id,
            "session_token": null,
            "route": {"slug": MISSING_REVISION_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let GetPageViewOutput::Found {
        compiled_body_html: missing_revision_html,
        ..
    } = missing_revision_view.page
    else {
        panic!("missing-revision Files fixture should still serve its page");
    };
    assert!(
        !missing_revision_html.contains("orphan-must-not-leak.bin"),
        "{missing_revision_html}"
    );
    assert!(
        !missing_revision_html.contains("page-files"),
        "{missing_revision_html}"
    );

    for (slug, hidden_field) in [
        ("fixture-files-module-hidden-name", "name"),
        ("fixture-files-module-hidden-s3-hash", "s3_hash"),
        ("fixture-files-module-hidden-mime", "mime"),
        ("fixture-files-module-hidden-size", "size"),
        (
            "fixture-files-module-unknown-hidden",
            "future-private-field",
        ),
    ] {
        create_listpages_test_page(
            &mut runner,
            site.site_id,
            slug,
            "Files module moderated revision fixture",
            "[[module Files]]",
        )
        .await;
        let moderated_page_id = listpages_test_page_id(&runner, site.site_id, slug).await;
        let moderated_file_id = create_file_fixture_with_descriptor(
            &runner,
            site.site_id,
            moderated_page_id,
            "moderated-must-not-leak.bin",
            4096,
            Some(ContentTypeDescriptor {
                label: "data".to_owned(),
                description: "data".to_owned(),
            }),
        )
        .await;
        let revision = FileRevisionService::get_latest(
            runner.context(),
            site.site_id,
            moderated_page_id,
            moderated_file_id,
        )
        .await
        .expect("moderated file revision fixture should be readable");
        let mut revision = revision.into_active_model();
        revision.hidden = Set(vec![hidden_field.to_owned()]);
        revision
            .update(runner.context().transaction())
            .await
            .expect("moderated file revision fixture should be updated");
        rerender_file_fixture_page(&runner, moderated_page_id).await;

        let moderated_view = run_endpoint!(
            runner,
            article_view,
            json!({
                "site_id": site.site_id,
                "session_token": null,
                "route": {"slug": slug, "extra": ""},
                "locales": ["en-US", "en"],
            }),
        );
        let GetPageViewOutput::Found {
            compiled_body_html: moderated_html,
            ..
        } = moderated_view.page
        else {
            panic!("moderated Files fixture should still serve its page");
        };
        assert!(
            !moderated_html.contains("moderated-must-not-leak.bin"),
            "hidden field {hidden_field} leaked a moderated row: {moderated_html}",
        );
        assert!(
            !moderated_html.contains("page-files"),
            "hidden field {hidden_field} exposed a partial table: {moderated_html}",
        );
    }

    create_listpages_test_page(
        &mut runner,
        site.site_id,
        SMALL_SIZE_SLUG,
        "Files module unobserved small-size fixture",
        "[[module Files]]",
    )
    .await;
    let small_page_id =
        listpages_test_page_id(&runner, site.site_id, SMALL_SIZE_SLUG).await;
    create_file_fixture_with_descriptor(
        &runner,
        site.site_id,
        small_page_id,
        "unobserved-small.bin",
        1023,
        Some(ContentTypeDescriptor {
            label: "data".to_owned(),
            description: "data".to_owned(),
        }),
    )
    .await;
    let small_view = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site.site_id,
            "session_token": null,
            "route": {"slug": SMALL_SIZE_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let GetPageViewOutput::Found {
        compiled_body_html: small_html,
        ..
    } = small_view.page
    else {
        panic!("small-size Files fixture should still serve its page");
    };
    assert!(!small_html.contains("unobserved-small.bin"), "{small_html}");
    assert!(!small_html.contains("page-files"), "{small_html}");

    create_listpages_test_page(
        &mut runner,
        site.site_id,
        OVERFLOW_SLUG,
        "Files module overflow fixture",
        "[[module Files]]",
    )
    .await;
    let overflow_page_id =
        listpages_test_page_id(&runner, site.site_id, OVERFLOW_SLUG).await;
    for index in 0_i64..16 {
        insert_file_fixture_with_descriptor(
            &runner,
            site.site_id,
            overflow_page_id,
            &format!("overflow-{index:02}.png"),
            1024 + index,
            Some(ContentTypeDescriptor {
                label: "PNG image data".to_owned(),
                description: "PNG image data, 1 x 1, 8-bit/color RGBA, non-interlaced"
                    .to_owned(),
            }),
        )
        .await;
    }
    rerender_file_fixture_page(&runner, overflow_page_id).await;
    let overflow_view = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site.site_id,
            "session_token": null,
            "route": {"slug": OVERFLOW_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let GetPageViewOutput::Found {
        compiled_body_html: overflow_html,
        ..
    } = overflow_view.page
    else {
        panic!("overflow Files fixture should still serve its page");
    };
    assert!(
        !overflow_html.contains("overflow-00.png"),
        "{overflow_html}"
    );
    assert!(!overflow_html.contains("page-files"), "{overflow_html}");
}

#[tokio::test]
async fn wikidot_standalone_actions_keep_exact_html_and_expose_typed_sidecars() {
    const SLUG: &str = "legacy-action-render-fixture";
    const SOURCE: &str = concat!(
        "[[button edit text=\"Edit here\" onclick=\"alert(1)\"]]\n",
        "[[button history]]\n",
        "[[button source]]\n",
        "[[button print class=\"custom-action\" style=\"color: #444\"]]\n",
        "[[button set-tags -* +favorite text=\"Change tags\"]]\n",
        "[[button unsupported text=\"Never active\"]]",
    );

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(SLUG)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": SOURCE,
            "title": "Legacy action render fixture",
            "alt_title": null,
            "slug": SLUG,
            "layout": "wikidot",
            "revision_comments": "create legacy action render fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let saved = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": SLUG,
            "details": {"compiled_html": true},
        }),
    )
    .expect("legacy action page should exist")
    .compiled_body_html
    .expect("legacy action page should have compiled HTML");
    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let view_actions = match view {
        GetPageViewOutput::Found { legacy_actions, .. } => legacy_actions,
        other => panic!("expected found page view, got {other:?}"),
    };
    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "Legacy action preview fixture",
            "wikitext": SOURCE,
        }),
    );

    let view_actions = serde_json::to_value(view_actions).unwrap();
    assert_eq!(
        &view_actions.as_array().unwrap()[..4],
        &[
            json!({"type": "edit"}),
            json!({"type": "history"}),
            json!({"type": "source"}),
            json!({"type": "print"}),
        ],
    );
    assert_eq!(view_actions[4]["type"], "set-tags");
    assert_eq!(view_actions[4]["index"], 4);
    assert_eq!(view_actions[4]["fingerprint"].as_str().unwrap().len(), 32);
    assert_eq!(
        serde_json::to_value(&preview.legacy_actions).unwrap(),
        view_actions,
    );

    for html in [saved, preview.body] {
        assert_eq!(
            html.matches(r#"class="wiki-standalone-button""#).count(),
            4,
            "standalone actions without a custom class must retain the Wikidot button class: {html}",
        );
        assert!(
            html.contains(r#"class="custom-action""#),
            "the custom print class must replace the default class: {html}",
        );
        assert_eq!(
            html.matches(r#"href="javascript:;""#).count(),
            5,
            "all supported standalone actions must retain the inert Wikidot href: {html}",
        );
        for label in [
            "Edit here",
            "history",
            "view source",
            "print",
            "Change tags",
        ] {
            assert!(
                html.contains(&format!(">{label}</a>")),
                "standalone action must retain its Wikidot label: {html}",
            );
        }
        assert!(
            html.contains(r#"style="color: #444""#),
            "the print action must retain its Wikidot style: {html}",
        );
        assert!(
            !html.contains("wj-button-"),
            "internal FTML IDs must not leak: {html}"
        );
        assert!(
            !html.contains("data-wikijump"),
            "Wikijump hooks must remain outside served Wikidot DOM: {html}"
        );
        for generated_handler in [
            r#"onclick="WIKIDOT.page.listeners.editClick(event)""#,
            r#"onclick="WIKIDOT.page.listeners.historyClick(event)""#,
            r#"onclick="WIKIDOT.page.listeners.viewSourceClick(event)""#,
            r#"onclick="WIKIDOT.page.listeners.printClick(event)""#,
            r#"onclick="WIKIDOT.page.listeners.updateTagsByButton(event, &#39;-* +favorite&#39;)""#,
        ] {
            assert!(
                html.contains(generated_handler),
                "generated Wikidot action handler is missing: {generated_handler}: {html}",
            );
        }
        assert!(
            !html.contains("alert(1)"),
            "authored script must stay inert: {html}"
        );
        assert!(
            html.contains(
                r#"<div class="error-block"><em>unsupported</em> is not a valid button type</div>"#,
            ),
            "unsupported actions must fail closed: {html}",
        );
    }
}

#[tokio::test]
async fn wikidot_set_tags_action_resolves_server_descriptor_and_is_revision_idempotent() {
    const SLUG: &str = "legacy-action-set-tags-fixture";
    const SOURCE: &str =
        "[[button set-tags -* +favorite +_book -_movie text=\"Change tags\"]]";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(SLUG)),
    );
    let created = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": SOURCE,
            "title": "Legacy action set-tags fixture",
            "alt_title": null,
            "tags": ["ordinary", "favorite", "_movie", "_kept"],
            "slug": SLUG,
            "layout": "wikidot",
            "revision_comments": "create legacy action set-tags fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let action_fingerprint = LegacyActionRegistry::from_wikidot_source(SOURCE)
        .fingerprint(0)
        .expect("set-tags descriptor should have a fingerprint");
    let mismatched = run_endpoint_err!(
        runner,
        wikidot_legacy_set_tags,
        json!({
            "page_id": created.page_id,
            "last_revision_id": created.revision_id,
            "action_index": 0,
            "action_fingerprint": "00000000000000000000000000000000",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(mismatched, ErrorType::Page);
    let changed = run_endpoint!(
        runner,
        wikidot_legacy_set_tags,
        json!({
            "page_id": created.page_id,
            "last_revision_id": created.revision_id,
            "action_index": 0,
            "action_fingerprint": action_fingerprint.clone(),
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("first set-tags activation should create a revision");
    let page =
        run_endpoint!(runner, page_get, json!({"site_id": site_id, "page": SLUG}),)
            .expect("set-tags page should still exist");
    assert_eq!(page.tags, ["_kept", "favorite", "_book"]);

    let repeated = run_endpoint!(
        runner,
        wikidot_legacy_set_tags,
        json!({
            "page_id": created.page_id,
            "last_revision_id": changed.revision_id,
            "action_index": 0,
            "action_fingerprint": action_fingerprint.clone(),
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(repeated.is_none(), "same action should be idempotent");

    // The forged descriptor above must not consume the local action bucket.
    // The state-changing activation plus nine valid idempotent repeats are
    // accepted without leaking page or actor state through the throttle; the
    // next valid renderer-bound activation is denied.
    for _ in 0..8 {
        let repeated = run_endpoint!(
            runner,
            wikidot_legacy_set_tags,
            json!({
                "page_id": created.page_id,
                "last_revision_id": changed.revision_id,
                "action_index": 0,
                "action_fingerprint": action_fingerprint.clone(),
                "user_id": ADMIN_USER_ID,
                "ip_address": common::IP_ADDRESS,
            }),
        );
        assert!(repeated.is_none(), "same action should remain idempotent");
    }
    let throttled = run_endpoint_err!(
        runner,
        wikidot_legacy_set_tags,
        json!({
            "page_id": created.page_id,
            "last_revision_id": changed.revision_id,
            "action_index": 0,
            "action_fingerprint": action_fingerprint.clone(),
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(throttled, ErrorType::Page);

    let stale = run_endpoint_err!(
        runner,
        wikidot_legacy_set_tags,
        json!({
            "page_id": created.page_id,
            "last_revision_id": created.revision_id,
            "action_index": 0,
            "action_fingerprint": action_fingerprint,
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(stale, ErrorType::Page);
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SetTagsMutationSnapshot {
    tags: Vec<String>,
    revision_id: i64,
    revision_count: i32,
    updated_at: Option<OffsetDateTime>,
    deleted_at: Option<OffsetDateTime>,
    compiled_body_html: Option<String>,
    page_edit_audit_count: u64,
}

async fn snapshot_set_tags_mutation_state(
    runner: &mut TestRunner,
    site_id: i64,
    page_id: i64,
    allow_deleted: bool,
) -> SetTagsMutationSnapshot {
    set_mutation_request_context(runner, ADMIN_USER_ID, site_id, Reference::Id(page_id));
    let page = if allow_deleted {
        run_endpoint!(
            runner,
            page_get_direct,
            json!({
                "site_id": site_id,
                "page_id": page_id,
                "allow_deleted": true,
                "details": {"compiled_html": true},
            }),
        )
    } else {
        run_endpoint!(
            runner,
            page_get,
            json!({
                "site_id": site_id,
                "page": page_id,
                "details": {"compiled_html": true},
            }),
        )
    }
    .expect("set-tags denial fixture should remain readable by its administrator");
    let page_edit_audit_count = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("page.edit"))
        .filter(AuditLogColumn::PageId.eq(page_id))
        .count(runner.context().transaction())
        .await
        .expect("set-tags denial audit supplement should be readable");

    SetTagsMutationSnapshot {
        tags: page.tags,
        revision_id: page.revision_id,
        revision_count: page.page_revision_count,
        updated_at: page.page_updated_at,
        deleted_at: page.page_deleted_at,
        compiled_body_html: page.compiled_body_html,
        page_edit_audit_count,
    }
}

#[tokio::test]
async fn wikidot_set_tags_denials_preserve_public_page_state() {
    const LOCKED_CATEGORY: &str = "fixture-set-tags-denial-locked-category";
    const PRIVATE_CATEGORY: &str = "fixture-set-tags-denial-private-category";
    const LOCKED_SLUG: &str = "fixture-set-tags-denial-locked";
    const DELETED_SLUG: &str = "fixture-set-tags-denial-deleted";
    const PRIVATE_SLUG: &str = "fixture-set-tags-denial-private";
    const ROUTE_TARGET_SLUG: &str = "fixture-set-tags-denial-route-target";
    const ROUTE_MISMATCH_SLUG: &str = "fixture-set-tags-denial-other-route";
    const ACTOR_MISMATCH_SLUG: &str = "fixture-set-tags-denial-actor";
    const CROSS_SITE_TARGET_SLUG: &str = "fixture-set-tags-denial-cross-site-target";
    const CROSS_SITE_ROUTE_SLUG: &str = "fixture-set-tags-denial-cross-site-route";
    const SOURCE: &str = "[[button set-tags +favorite text=\"Change tags\"]]";

    #[derive(Debug)]
    struct DenialCase {
        name: &'static str,
        target_page_id: i64,
        allow_deleted: bool,
        request_actor_id: i64,
        request_site_id: i64,
        route_slug: &'static str,
        submitted_user_id: i64,
    }

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let cross_site = run_endpoint!(runner, site_get, json!({"site": "scp-jp"}))
        .expect("seeded SCP-JP site should exist");
    let cross_site_id = cross_site.site.site_id;

    for (slug, title) in [
        (LOCKED_SLUG, "Set-tags locked target"),
        (DELETED_SLUG, "Set-tags deleted target"),
        (PRIVATE_SLUG, "Set-tags private target"),
        (ROUTE_TARGET_SLUG, "Set-tags route target"),
        (ROUTE_MISMATCH_SLUG, "Set-tags mismatched route"),
        (ACTOR_MISMATCH_SLUG, "Set-tags actor target"),
        (CROSS_SITE_TARGET_SLUG, "Set-tags cross-site target"),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, SOURCE).await;
    }
    create_listpages_test_page(
        &mut runner,
        cross_site_id,
        CROSS_SITE_ROUTE_SLUG,
        "Set-tags cross-site route",
        SOURCE,
    )
    .await;

    let mut pages = Vec::new();
    for slug in [
        LOCKED_SLUG,
        DELETED_SLUG,
        PRIVATE_SLUG,
        ROUTE_TARGET_SLUG,
        ACTOR_MISMATCH_SLUG,
        CROSS_SITE_TARGET_SLUG,
    ] {
        pages.push(
            run_endpoint!(runner, page_get, json!({"site_id": site_id, "page": slug}),)
                .expect("set-tags denial target should exist"),
        );
    }
    let locked_page_id = pages[0].page_id;
    let deleted_page_id = pages[1].page_id;
    let private_page_id = pages[2].page_id;
    let route_target_page_id = pages[3].page_id;
    let actor_mismatch_page_id = pages[4].page_id;
    let cross_site_target_page_id = pages[5].page_id;

    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        LOCKED_CATEGORY,
        SAMPLE_USER_ID,
        &[Action::View, Action::Edit],
        "set-tags-editor",
    )
    .await;
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;
    set_listpages_test_category_slug(&runner, site_id, LOCKED_SLUG, LOCKED_CATEGORY)
        .await;
    set_listpages_test_category_slug(&runner, site_id, PRIVATE_SLUG, PRIVATE_CATEGORY)
        .await;
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("set-tags denial permission cache should be invalidated");
    PageLockService::create(
        runner.context(),
        site_id,
        ADMIN_USER_ID,
        Reference::Id(locked_page_id),
        CreatePageLockInput {
            page: Reference::Id(locked_page_id),
            expires_at: None,
            from_wikidot: false,
            lock_type: PageLockType::PermissionOnly,
            reason: Some("set-tags denial matrix".to_owned()),
            override_existing: false,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("set-tags denial page lock should be created");

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(deleted_page_id),
    );
    let deleted_revision_id = pages[1].revision_id;
    run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": deleted_page_id,
            "last_revision_id": deleted_revision_id,
            "revision_comments": "delete set-tags denial target",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let fingerprint = LegacyActionRegistry::from_wikidot_source(SOURCE)
        .fingerprint(0)
        .expect("set-tags denial fixture should have a registry fingerprint");
    let cases = [
        DenialCase {
            name: "locked",
            target_page_id: locked_page_id,
            allow_deleted: false,
            request_actor_id: SAMPLE_USER_ID,
            request_site_id: site_id,
            route_slug: LOCKED_SLUG,
            submitted_user_id: SAMPLE_USER_ID,
        },
        DenialCase {
            name: "deleted",
            target_page_id: deleted_page_id,
            allow_deleted: true,
            request_actor_id: ADMIN_USER_ID,
            request_site_id: site_id,
            route_slug: DELETED_SLUG,
            submitted_user_id: ADMIN_USER_ID,
        },
        DenialCase {
            name: "private",
            target_page_id: private_page_id,
            allow_deleted: false,
            request_actor_id: SAMPLE_USER_ID,
            request_site_id: site_id,
            route_slug: PRIVATE_SLUG,
            submitted_user_id: SAMPLE_USER_ID,
        },
        DenialCase {
            name: "route mismatch",
            target_page_id: route_target_page_id,
            allow_deleted: false,
            request_actor_id: ADMIN_USER_ID,
            request_site_id: site_id,
            route_slug: ROUTE_MISMATCH_SLUG,
            submitted_user_id: ADMIN_USER_ID,
        },
        DenialCase {
            name: "actor mismatch",
            target_page_id: actor_mismatch_page_id,
            allow_deleted: false,
            request_actor_id: ADMIN_USER_ID,
            request_site_id: site_id,
            route_slug: ACTOR_MISMATCH_SLUG,
            submitted_user_id: SAMPLE_USER_ID,
        },
        DenialCase {
            name: "cross-site",
            target_page_id: cross_site_target_page_id,
            allow_deleted: false,
            request_actor_id: ADMIN_USER_ID,
            request_site_id: cross_site_id,
            route_slug: CROSS_SITE_ROUTE_SLUG,
            submitted_user_id: ADMIN_USER_ID,
        },
    ];

    for case in cases {
        let before = snapshot_set_tags_mutation_state(
            &mut runner,
            site_id,
            case.target_page_id,
            case.allow_deleted,
        )
        .await;
        set_mutation_request_context(
            &mut runner,
            case.request_actor_id,
            case.request_site_id,
            Reference::Slug(Cow::Borrowed(case.route_slug)),
        );
        let error = run_endpoint_err!(
            runner,
            wikidot_legacy_set_tags,
            json!({
                "page_id": case.target_page_id,
                "last_revision_id": before.revision_id,
                "action_index": 0,
                "action_fingerprint": fingerprint.clone(),
                "user_id": case.submitted_user_id,
                "ip_address": common::IP_ADDRESS,
            }),
        );
        assert_contains_error!(error, ErrorType::Page);

        let after = snapshot_set_tags_mutation_state(
            &mut runner,
            site_id,
            case.target_page_id,
            case.allow_deleted,
        )
        .await;
        assert_eq!(
            after, before,
            "{} set-tags denial must preserve public page and audit state",
            case.name,
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
    assert!(html.contains(
        r#"<span class="ratedown btn btn-default"><a href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, -1)" title="I don't like it">–</a></span>"#,
    ));
    assert!(html.contains(
        r#"<span class="cancel btn btn-default"><a href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.cancelVote(event)" title="Cancel my vote">x</a></span>"#,
    ));
    assert!(!html.contains("data-wikijump"));
    assert!(!html.contains(r#"<div class="page-rate-widget-box"><p>"#));
    assert!(!html.contains(r#"<p><div class="page-rate-widget-box">"#));
    assert!(!html.contains(r#"<a href="javascript:;"><span class="rateup"#));
    assert_eq!(html.matches(r#"class="rate-points""#).count(), 1);
    assert!(!html.contains("WIKIJUMPWIKIDOTCOMPATHTML"));
}

#[tokio::test]
async fn page_render_bodyless_rate_does_not_claim_a_later_sibling_module_closer() {
    let runner = TestRunner::setup().await;
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let page_info = PageInfo {
        page: Cow::Borrowed("rate-sibling-module-fixture"),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("Rate Sibling Module Fixture"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let source = concat!(
        "[[module Rate]]\n",
        "[[div class=\"article-start\"]]VISIBLE ARTICLE BODY[[/div]]\n",
        "[[module CSS]]\n",
        ".sibling { display: block; }\n",
        "[[/module]]\n",
        "VISIBLE ARTICLE TAIL\n",
    );

    let output =
        RenderService::render(runner.context(), source.to_owned(), &page_info, &settings)
            .await
            .expect("bodyless Rate followed by a sibling module should render");
    let html = output.html_output.body;

    assert!(html.contains("VISIBLE ARTICLE BODY"), "{html}");
    assert!(html.contains("VISIBLE ARTICLE TAIL"), "{html}");
    assert_eq!(html.matches(r#"class="page-rate-widget-box""#).count(), 1);
    assert!(!html.contains("[[module CSS]]"), "{html}");
}

#[tokio::test]
async fn page_render_basalt_rate_does_not_claim_active_iftags_through_eof() {
    let runner = TestRunner::setup().await;
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut page_info = PageInfo {
        page: Cow::Borrowed("basalt"),
        category: Some(Cow::Borrowed("theme")),
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("Basalt"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: vec![Cow::Borrowed("co-authored"), Cow::Borrowed("theme")],
        language: Cow::Borrowed("en"),
    };

    let active = RenderService::render(
        runner.context(),
        include_str!("../seeder/theme-basalt.ftml").to_owned(),
        &page_info,
        &settings,
    )
    .await
    .expect("page render with Rate inside an active gate should succeed");
    let active_html = active.html_output.body;

    assert!(
        active_html.contains(r#"class="page-rate-widget-box""#),
        "{active_html}"
    );
    assert!(
        active_html.contains("Basalt</strong> is an aesthetic theme"),
        "{active_html}"
    );
    assert!(!active_html.contains("[[iftags"), "{active_html}");
    assert!(!active_html.contains("[[/iftags]]"), "{active_html}");

    page_info.tags.clear();
    let inactive = RenderService::render(
        runner.context(),
        include_str!("../seeder/theme-basalt.ftml").to_owned(),
        &page_info,
        &settings,
    )
    .await
    .expect("page render with the Basalt theme gate inactive should succeed");
    let inactive_html = inactive.html_output.body;

    assert!(
        !inactive_html.contains(r#"class="page-rate-widget-box""#),
        "{inactive_html}"
    );
    assert!(
        !inactive_html.contains("Basalt</strong> is an aesthetic theme"),
        "{inactive_html}"
    );

    page_info.tags.push(Cow::Borrowed("theme"));
    let literal = RenderService::render(
        runner.context(),
        concat!(
            "[[iftags +theme]]\n",
            "[[code]]\n",
            "[[/iftags]]\n",
            "[[/code]]\n",
            "visible after literal\n",
            "[[/iftags]]\n",
        )
        .to_owned(),
        &page_info,
        &settings,
    )
    .await
    .expect("page render with a conditional closer inside code should succeed");
    let literal_html = literal.html_output.body;

    assert!(
        literal_html.contains("visible after literal"),
        "{literal_html}"
    );
    assert!(
        literal_html.contains("<pre><code>[[/iftags]]"),
        "{literal_html}"
    );

    let paired_rate = RenderService::render(
        runner.context(),
        concat!(
            "[[iftags +theme]]\n",
            "[[module Rate]]\n",
            "[[/iftags]]\n",
            "[[/module]]\n",
            "visible after paired Rate\n",
            "[[/iftags]]\n",
        )
        .to_owned(),
        &page_info,
        &settings,
    )
    .await
    .expect("page render with a conditional closer inside paired Rate should succeed");
    let paired_rate_html = paired_rate.html_output.body;

    assert!(
        paired_rate_html.contains(r#"class="page-rate-widget-box""#),
        "{paired_rate_html}"
    );
    assert!(
        paired_rate_html.contains("visible after paired Rate"),
        "{paired_rate_html}"
    );
    assert!(
        !paired_rate_html.contains("[[/iftags]]"),
        "{paired_rate_html}"
    );
}

#[tokio::test]
async fn saved_rate_sidecar_binds_exact_revision_and_mutates_idempotently() {
    const SLUG: &str = "fixture-rate-action-sidecar";
    const SOURCE: &str = "[[module Rate]]";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let revision_id = create_listpages_test_page(
        &mut runner,
        site_id,
        SLUG,
        "Fixture Rate Action Sidecar",
        SOURCE,
    )
    .await;
    let page =
        run_endpoint!(runner, page_get, json!({"site_id": site_id, "page": SLUG}),)
            .expect("Rate sidecar page should exist");
    let session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "Rate action sidecar test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("Rate action actor session should be created");

    let anonymous = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    // Anonymous viewers receive the same descriptors so their clicks reach the
    // authoritative vote endpoint; they have no current vote of their own.
    let anonymous_registry = match anonymous {
        GetPageViewOutput::Found {
            rate_actions: Some(registry),
            ..
        } => registry,
        other => panic!("expected anonymous Rate sidecar, got {other:?}"),
    };
    assert_eq!(anonymous_registry.current_value, None);
    assert_eq!(anonymous_registry.actions.len(), 3);

    let authenticated = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token.clone(),
            "route": {"slug": SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let registry = match authenticated {
        GetPageViewOutput::Found {
            rate_actions: Some(registry),
            ..
        } => registry,
        other => panic!("expected authenticated Rate sidecar, got {other:?}"),
    };
    assert_eq!(registry.site_id, site_id);
    assert_eq!(registry.page_id, page.page_id);
    assert_eq!(registry.revision_id, revision_id);
    assert_eq!(registry.current_value, None);
    let actions = serde_json::to_value(&registry.actions).unwrap();
    assert_eq!(actions[0]["type"], "rate");
    assert_eq!(actions[1]["type"], "rate");
    assert_eq!(actions[2]["type"], "rate-cancel");
    assert_eq!(actions[0]["value"], 1);
    assert_eq!(actions[1]["value"], -1);

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(SLUG)),
    );
    let activate = |action: &serde_json::Value| {
        json!({
            "page_id": registry.page_id,
            "last_revision_id": registry.revision_id,
            "action_index": action["index"],
            "action_fingerprint": action["fingerprint"],
            "value": 5,
            "score": 500,
            "user_id": SAMPLE_USER_ID,
            "site_id": -1,
        })
    };

    let first = run_endpoint!(runner, wikidot_legacy_rate, activate(&actions[0]));
    assert_eq!(first.score, ScoreValue::Integer(1));
    let repeated = run_endpoint!(runner, wikidot_legacy_rate, activate(&actions[0]));
    assert_eq!(repeated.score, ScoreValue::Integer(1));
    let changed = run_endpoint!(runner, wikidot_legacy_rate, activate(&actions[1]));
    assert_eq!(changed.score, ScoreValue::Integer(-1));
    let canceled = run_endpoint!(runner, wikidot_legacy_rate, activate(&actions[2]));
    assert_eq!(canceled.score, ScoreValue::Integer(0));
    let repeated_cancel =
        run_endpoint!(runner, wikidot_legacy_rate, activate(&actions[2]));
    assert_eq!(repeated_cancel.score, ScoreValue::Integer(0));

    let forged = run_endpoint_err!(
        runner,
        wikidot_legacy_rate,
        json!({
            "page_id": registry.page_id,
            "last_revision_id": registry.revision_id,
            "action_index": 0,
            "action_fingerprint": "00000000000000000000000000000000",
        }),
    );
    assert_contains_error!(forged, ErrorType::PermissionDenied);
    let stale = run_endpoint_err!(
        runner,
        wikidot_legacy_rate,
        json!({
            "page_id": registry.page_id,
            "last_revision_id": registry.revision_id - 1,
            "action_index": actions[0]["index"],
            "action_fingerprint": actions[0]["fingerprint"],
        }),
    );
    assert_contains_error!(stale, ErrorType::NotLatestRevisionId);

    set_page_rating_policy(&runner, page.page_category_id, false, "registered").await;
    let disabled_view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token.clone(),
            "route": {"slug": SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    assert!(matches!(
        disabled_view,
        GetPageViewOutput::Found {
            rate_actions: None,
            ..
        }
    ));
    let disabled = run_endpoint_err!(runner, wikidot_legacy_rate, activate(&actions[0]),);
    assert_contains_error!(disabled, ErrorType::PermissionDenied);

    set_page_rating_policy(&runner, page.page_category_id, true, "members").await;
    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(SLUG)),
    );
    let non_member =
        run_endpoint_err!(runner, wikidot_legacy_rate, activate(&actions[0]),);
    assert_contains_error!(non_member, ErrorType::PermissionDenied);
    RelationService::create_site_member(
        runner.context(),
        CreateSiteMember {
            site_id,
            user_id: SAMPLE_USER_ID,
            metadata: SiteMemberData {
                accepted: SiteMemberAccepted::SelfJoined,
            },
            created_by: SYSTEM_USER_ID,
        },
        common::IP_ADDRESS,
    )
    .await
    .expect("Rate policy fixture actor should become a member");
    let member = run_endpoint!(runner, wikidot_legacy_rate, activate(&actions[0]));
    assert_eq!(member.score, ScoreValue::Integer(1));

    set_page_rating_policy(&runner, page.page_category_id, true, "registered").await;
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(SLUG)),
    );
    PageLockService::create(
        runner.context(),
        site_id,
        ADMIN_USER_ID,
        Reference::Id(page.page_id),
        CreatePageLockInput {
            page: Reference::Id(page.page_id),
            expires_at: None,
            from_wikidot: false,
            lock_type: PageLockType::PermissionOnly,
            reason: Some("Rate policy fixture".to_owned()),
            override_existing: false,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("Rate policy fixture lock should be created");
    let locked_view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": {"slug": SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    assert!(matches!(
        locked_view,
        GetPageViewOutput::Found {
            rate_actions: None,
            ..
        }
    ));
    let locked = run_endpoint_err!(runner, wikidot_legacy_rate, activate(&actions[0]),);
    assert_contains_error!(locked, ErrorType::PermissionDenied);
    PageLockService::remove(
        runner.context(),
        site_id,
        ADMIN_USER_ID,
        Reference::Id(page.page_id),
        common::IP_ADDRESS,
    )
    .await
    .expect("Rate policy fixture lock removal should succeed")
    .expect("Rate policy fixture lock should remain active until removal");

    let stored_page = PageTable::find_by_id(page.page_id)
        .one(runner.context().transaction())
        .await
        .expect("Rate deletion fixture lookup should succeed")
        .expect("Rate deletion fixture page should exist");
    let mut stored_page = stored_page.into_active_model();
    stored_page.deleted_at = Set(Some(OffsetDateTime::now_utc()));
    stored_page
        .update(runner.context().transaction())
        .await
        .expect("Rate deletion fixture should be soft deleted");
    let deleted = run_endpoint_err!(runner, wikidot_legacy_rate, activate(&actions[0]),);
    assert_contains_error!(deleted, ErrorType::PageVote);
}

#[tokio::test]
async fn page_render_inline_rate_module_matches_the_block_placement_boundary() {
    let runner = TestRunner::setup().await;
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let page_info = PageInfo {
        page: Cow::Borrowed("inline-rate-widget-fixture"),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("Inline Rate Widget Fixture"),
        alt_title: None,
        score: ScoreValue::Integer(7),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };

    let output = RenderService::render(
        runner.context(),
        "PROBE_BEGIN\n[[module Rate]]\nPROBE_END".to_owned(),
        &page_info,
        &settings,
    )
    .await
    .expect("an inline-position Rate module should render");
    let html = output.html_output.body;

    let before = html
        .find("<p>PROBE_BEGIN</p>")
        .expect("live Wikidot closes the preceding paragraph");
    let widget = html
        .find(r#"<div class="page-rate-widget-box">"#)
        .expect("the inline-position module should emit its block widget");
    let after = html
        .find("<p>PROBE_END</p>")
        .expect("live Wikidot reopens a following paragraph");
    assert!(before < widget && widget < after, "{html}");
    assert!(!html.contains("WIKIJUMPWIKIDOTCOMPATHTML"), "{html}");
}

#[tokio::test]
async fn page_render_star_rate_module_consumes_body_and_substitutes_live_variables() {
    const CATEGORY: &str = "fixture-rate-module-stars";
    const SLUG: &str = "fixture-rate-module-stars:holder";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    set_listpages_test_category_rating_type(&runner, site_id, CATEGORY, "stars").await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        SLUG,
        "Fixture Rate Module Stars",
        concat!(
            "[[module Rate]]\n",
            "Average Rating %%rating%% from %%rating_votes%% votes percent=%%rating_percent%% decimal=%%rating_decimal%%\n",
            "[[/module]]",
        ),
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, SLUG, CATEGORY).await;
    let page_id = listpages_test_page_id(&runner, site_id, SLUG).await;
    let page_category_id = PageTable::find_by_id(page_id)
        .one(runner.context().transaction())
        .await
        .expect("Rate star fixture page lookup should not fail")
        .expect("Rate star fixture page should exist")
        .page_category_id;
    runner
        .context()
        .transaction()
        .execute_raw(Statement::from_sql_and_values(
            runner.context().transaction().get_database_backend(),
            "INSERT INTO page_vote (from_wikidot, page_id, user_id, rating_system, value) \
             VALUES (false, $1, $2, 'stars', 4)",
            [Value::from(page_id), Value::from(ADMIN_USER_ID)],
        ))
        .await
        .expect("Rate star fixture should receive its stored star vote");
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(SLUG.into())),
    });
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": page_category_id,
            "page_id": page_id,
        }),
    );

    let html = load_listpages_test_compiled_html(&runner, site_id, SLUG).await;

    assert!(
        html.contains(concat!(
            r#"<div class="page-rate-widget"><div class="page-rate-widget-start" data-rating="4"></div>"#,
            r#"<div class="page-rate-widget-start-text">Average Rating "#,
            r#"<span class="page-rate-widget-start-text-rating">4</span>"#,
            r#" from <span class="page-rate-widget-start-text-rating-votes">1</span>"#,
            r#" votes percent=<span class="page-rate-widget-start-text-rating-percent">80</span>"#,
            r#" decimal=%%rating_decimal%%</div></div>"#,
        )),
        "star Rate module body should match live Wikidot's star wrapper, variable spans, and literal rating_decimal:\n{html}",
    );
    assert!(!html.contains(r#"class="page-rate-widget-box""#), "{html}");
    assert!(!html.contains("[[/module]]"), "{html}");
    assert!(!html.contains("WIKIJUMPWIKIDOTCOMPATHTML"), "{html}");
}

#[tokio::test]
async fn wikidot_user_blocks_match_live_preview_and_saved_page_identity_boundaries() {
    const EXTANT_USER_ID: i64 = 19_102_600;
    const DELETED_USER_ID: i64 = 19_102_601;
    const NAME_ONLY_USER_ID: i64 = 19_102_602;
    const COLLISION_FIRST_USER_ID: i64 = 19_102_603;
    const COLLISION_SECOND_USER_ID: i64 = 19_102_604;
    const UNICODE_USER_ID: i64 = 19_102_605;
    const NUMERIC_ID_USER_ID: i64 = 2;
    const DISPLAY_NUMERIC_NAME_USER_ID: i64 = 19_102_606;
    const PAGE_SLUG: &str = "fixture-wikidot-user-identity-matrix";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "INSERT INTO known_user (user_id) VALUES ($1), ($2), ($3), ($4), ($5), ($6), ($7), ($8)",
            [
                Value::from(EXTANT_USER_ID),
                Value::from(DELETED_USER_ID),
                Value::from(NAME_ONLY_USER_ID),
                Value::from(COLLISION_FIRST_USER_ID),
                Value::from(COLLISION_SECOND_USER_ID),
                Value::from(UNICODE_USER_ID),
                Value::from(NUMERIC_ID_USER_ID),
                Value::from(DISPLAY_NUMERIC_NAME_USER_ID),
            ],
        ))
        .await
        .expect("user-block known-user fixtures should be inserted");
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            concat!(
                "INSERT INTO wikidot_user (",
                "user_id, created_at, fetched_at, is_deleted, name, slug, karma, is_pro",
                ") VALUES ",
                "($1, NOW() - INTERVAL '1 second', NOW(), FALSE, 'Extant User', 'extant-user', 5, FALSE), ",
                "($2, NOW() - INTERVAL '1 second', NOW(), TRUE, 'Deleted User', 'deleted-user', 0, FALSE), ",
                "($3, NOW() - INTERVAL '1 second', NOW(), FALSE, 'Name Only User', 'name-only-slug', 5, FALSE), ",
                "($4, NOW() - INTERVAL '1 second', NOW(), FALSE, 'Shared Person', 'shared-person-first', 5, FALSE), ",
                "($5, NOW() - INTERVAL '1 second', NOW(), FALSE, 'Shared_Person', 'shared-person-second', 5, FALSE), ",
                "($6, NOW() - INTERVAL '1 second', NOW(), FALSE, 'Éclair\tName\u{00a0}JP', 'unicode-name', 5, FALSE), ",
                "($8, NOW() - INTERVAL '1 second', NOW(), FALSE, '2', 'display-two', 5, FALSE)",
            ),
            [
                Value::from(EXTANT_USER_ID),
                Value::from(DELETED_USER_ID),
                Value::from(NAME_ONLY_USER_ID),
                Value::from(COLLISION_FIRST_USER_ID),
                Value::from(COLLISION_SECOND_USER_ID),
                Value::from(UNICODE_USER_ID),
                Value::from(NUMERIC_ID_USER_ID),
                Value::from(DISPLAY_NUMERIC_NAME_USER_ID),
            ],
        ))
        .await
        .expect("user-block Wikidot fixtures should be inserted");

    ImportService::add_user(
        runner.context(),
        ImportUser {
            user_id: i32::try_from(NUMERIC_ID_USER_ID).unwrap(),
            created_at: OffsetDateTime::UNIX_EPOCH,
            fetched_at: OffsetDateTime::UNIX_EPOCH + Duration::seconds(1),
            wikidot_user_type: ImportedUserType::Extant {
                name: "Numeric Target".to_owned(),
                slug: "numeric-target".to_owned(),
            },
            avatar_uploaded_blob_id: None,
            real_name: None,
            gender: None,
            birthday: None,
            location: None,
            biography: None,
            website: None,
            karma: KarmaLevel::Three,
            is_pro: false,
            importing_user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("numeric Wikidot source identity should be materialized through importer");

    let source = format!(
        concat!(
            "NAME=[[user Extant User]]\n",
            "NAME_ONLY=[[user Name Only User]]\n",
            "COLLISION=[[*user Shared Person]]\n",
            "UNKNOWN_AVATAR=[[*user Unknown Avatar User]]\n",
            "UNICODE=[[user éCLAIR\u{00a0}name\tjp]]\n",
            "NUMERIC_ID=[[*user 2]]\n",
            "NUMERIC_NAME=[[user display-two]]\n",
            "ID=[[*user {EXTANT_USER_ID}]]\n",
            "DELETED=[[user Deleted User]]\n",
            "A=[[user v7ws=\"alpha\tbeta\u{00a0}gamma\"]]\n",
            "B=[[user v7ser=\"serialized body\"]]\n",
            "C=[[user v7text=\"visible text\"]]\n",
            "D=[[user v7arg=\"one\" v7arg=\"two\"]]\n",
            "E=[[user v7arg=\"\"]]\n",
            "F=[[user v7UnknownArgument=\"x\"]]\n",
            "G=[[user v7arg='single quoted' data-v7=unquoted]]",
        ),
        EXTANT_USER_ID = EXTANT_USER_ID,
    );
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
            "title": "Wikidot user identity matrix",
            "wikitext": source,
        }),
    );
    let html = preview.body;

    create_listpages_test_page(
        &mut runner,
        site_id,
        PAGE_SLUG,
        "Fixture Wikidot User Identity Matrix",
        &source,
    )
    .await;
    let saved_page = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": PAGE_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let saved_html = match saved_page {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected saved Wikidot user matrix, got {other:?}"),
    };

    for (label, output) in [
        ("preview", html.as_str()),
        ("saved page", saved_html.as_str()),
    ] {
        assert!(
            output.contains("http://www.wikidot.com/user:info/extant-user")
                && output.contains(&format!(
                    "WIKIDOT.page.listeners.userInfo({EXTANT_USER_ID}); return false;"
                ))
                && output.contains(">Extant User</a>"),
            "{label} extant name and numeric ID references should share the imported identity:\n{output}",
        );
        assert!(
            output.contains("http://www.wikidot.com/user:info/name-only-slug")
                && output.contains(&format!(
                    "WIKIDOT.page.listeners.userInfo({NAME_ONLY_USER_ID}); return false;"
                ))
                && output.contains(">Name Only User</a>"),
            "{label} should resolve the imported display name even when its slug differs:\n{output}",
        );
        assert!(
            output.contains("http://www.wikidot.com/user:info/unicode-name")
                && output.contains(&format!(
                    "WIKIDOT.page.listeners.userInfo({UNICODE_USER_ID}); return false;"
                ))
                && output.contains(">Éclair"),
            "{label} should use the shared Unicode whitespace and case normalization contract:\n{output}",
        );
        assert!(
            !output.contains("http://www.wikidot.com/user:info/numeric-target")
                && !output.contains("WIKIDOT.page.listeners.userInfo(2); return false;")
                && !output.contains(">Numeric Target</a>"),
            "{label} digit lookup keys must resolve by name, never by numeric ID:\n{output}",
        );
        assert!(
            output.contains("http://www.wikidot.com/user:info/display-two")
                && output.contains(&format!(
                    "WIKIDOT.page.listeners.userInfo({DISPLAY_NUMERIC_NAME_USER_ID}); return false;"
                ))
                && output.contains(">2</a>"),
            "{label} should still resolve the separately loaded user whose display name is 2:\n{output}",
        );
    }
    assert_eq!(
        html.matches(&format!(
            "WIKIDOT.page.listeners.userInfo({EXTANT_USER_ID})"
        ))
        .count(),
        1,
        "only the plain extant row links the imported identity; numeric identity text is a name key, never an ID link:\n{html}",
    );
    // Sealed anonymous PagePreview cases 1421 and 1423-1428 in
    // /mnt/oracle-store/wjlab/issue-scout-20260731/v7-full-syntax/comparison.json
    // (SHA-256 87e8ae77db8cfe526fcd46fb13e2843968fcc80d9e6bebdc0692bd636f57ff76).
    let live_missing_user_fragments = [
        concat!(
            r#"<span class="error-inline"><em>Deleted User</em>"#,
            " does not match any existing user name</span>",
        ),
        concat!(
            r#"<span class="error-inline"><em>Unknown Avatar User</em>"#,
            " does not match any existing user name</span>",
        ),
        concat!(
            "<span class=\"error-inline\"><em>v7ws=&quot;alpha beta\u{a0}gamma&quot;</em>",
            " does not match any existing user name</span>",
        ),
        concat!(
            r#"<span class="error-inline"><em>v7ser=&quot;serialized body&quot;</em>"#,
            " does not match any existing user name</span>",
        ),
        concat!(
            r#"<span class="error-inline"><em>v7text=&quot;visible text&quot;</em>"#,
            " does not match any existing user name</span>",
        ),
        concat!(
            r#"<span class="error-inline"><em>v7arg=&quot;one&quot; v7arg=&quot;two&quot;</em>"#,
            " does not match any existing user name</span>",
        ),
        concat!(
            r#"<span class="error-inline"><em>v7arg=&quot;&quot;</em>"#,
            " does not match any existing user name</span>",
        ),
        concat!(
            r#"<span class="error-inline"><em>v7UnknownArgument=&quot;x&quot;</em>"#,
            " does not match any existing user name</span>",
        ),
        concat!(
            r#"<span class="error-inline"><em>v7arg=&#39;single quoted&#39; data-v7=unquoted</em>"#,
            " does not match any existing user name</span>",
        ),
    ];
    for (label, output) in [
        ("preview", html.as_str()),
        ("saved page", saved_html.as_str()),
    ] {
        for expected in live_missing_user_fragments {
            assert!(
                output.contains(expected),
                "{label} must keep the exact live missing-user text inside span.error-inline:\n{output}",
            );
        }
        let numeric_identity_fragment = format!(
            "<span class=\"error-inline\"><em>{EXTANT_USER_ID}</em> does not match any existing user name</span>",
        );
        assert!(
            output.contains(&numeric_identity_fragment),
            "{label} must keep the exact live missing-user text for numeric identity text:\n{output}",
        );
        assert_eq!(
            output
                .matches("does not match any existing user name")
                .count(),
            11,
            "{label} must fail closed for the deleted identity, colliding name, starred unknown, seven unknown lookup keys, and numeric identity text:\n{output}",
        );
        assert!(
            !output.contains("user:info/deleted-user"),
            "{label} must not expose a profile URL for the deleted identity:\n{output}",
        );
        assert!(
            !output.contains(&format!("user:info/{EXTANT_USER_ID}")),
            "{label} must not expose a profile URL for numeric identity text:\n{output}",
        );
        assert!(
            output.contains(concat!(
                r#"<span class="error-inline"><em>Shared Person</em>"#,
                " does not match any existing user name</span>",
            )),
            "{label} must fail closed for colliding normalized display names:\n{output}",
        );
        assert!(
            !output.contains("user:info/shared-person-first"),
            "{label}: {output}"
        );
        assert!(
            !output.contains("user:info/shared-person-second"),
            "{label}: {output}"
        );
        for marker in [
            "DELETED=",
            "COLLISION=",
            "UNKNOWN_AVATAR=",
            "ID=",
            "A=",
            "B=",
            "C=",
            "D=",
            "E=",
            "F=",
            "G=",
        ] {
            let start = output
                .match_indices(marker)
                .find_map(|(start, _)| {
                    let before = output[..start].trim_end_matches(['\r', '\n']);
                    (before.is_empty()
                        || ["<p>", "<br>", "<br/>", "<br />"]
                            .iter()
                            .any(|boundary| before.ends_with(boundary)))
                    .then_some(start)
                })
                .unwrap_or_else(|| panic!("{label} missing {marker}: {output}"));
            let case = &output[start..];
            let end = case
                .find("<br")
                .or_else(|| case.find("</p>"))
                .unwrap_or(case.len());
            let case = &case[..end];
            for forbidden in ["<a", "onclick=", "printuser", "avatar"] {
                assert!(
                    !case.contains(forbidden),
                    "{label} {marker} must not emit {forbidden}: {case}"
                );
            }
        }
    }
}

#[tokio::test]
async fn basalt_runtime_state_users_rerender_identity_profile_subtrees() {
    let runtime_state: serde_json::Value = serde_json::from_str(include_str!(
        "../../install/local/wikidot-verification/fixtures/open87-basalt-users/runtime-state.json"
    ))
    .expect("the committed Basalt runtime-state fixture should be valid JSON");
    let evidence: serde_json::Value = serde_json::from_str(include_str!(
        "../../install/local/wikidot-verification/fixtures/open87-basalt-users/evidence.jsonl"
    ))
    .expect("the committed Basalt evidence projection should be valid JSON");
    let fixture_users = runtime_state["wikidot_users"]
        .as_array()
        .expect("the Basalt runtime-state fixture should contain Wikidot users");
    let evidence_users = evidence["users"]
        .as_array()
        .expect("the Basalt evidence projection should contain user counts");
    assert_eq!(fixture_users.len(), 3);

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let transaction = runner.context().transaction();
    for user in fixture_users {
        let user_id = user["user_id"]
            .as_i64()
            .expect("fixture user ID should be an integer");
        let name = user["name"]
            .as_str()
            .expect("fixture user name should be a string");
        let slug = user["slug"]
            .as_str()
            .expect("fixture user slug should be a string");
        transaction
            .execute_raw(Statement::from_sql_and_values(
                transaction.get_database_backend(),
                "INSERT INTO known_user (user_id) VALUES ($1)",
                [Value::from(user_id)],
            ))
            .await
            .expect("Basalt known-user fixture row should be inserted");
        transaction
            .execute_raw(Statement::from_sql_and_values(
                transaction.get_database_backend(),
                concat!(
                    "INSERT INTO wikidot_user (",
                    "user_id, created_at, fetched_at, is_deleted, name, slug, karma, is_pro",
                    ") VALUES ($1, NOW() - INTERVAL '1 second', NOW(), FALSE, $2, $3, 0, FALSE)",
                ),
                [
                    Value::from(user_id),
                    Value::from(name.to_owned()),
                    Value::from(slug.to_owned()),
                ],
            ))
            .await
            .expect("Basalt Wikidot user fixture row should be inserted");
    }

    let basalt = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site.site_id, "page": "theme:basalt"}),
    )
    .expect("the seeded Basalt page should exist");
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site.site.site_id),
        page_reference: Some(Reference::Id(basalt.page_id)),
    });
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site.site.site_id,
            "category_id": basalt.page_category_id,
            "page_id": basalt.page_id,
        }),
    );
    let basalt = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site.site.site_id,
            "page": "theme:basalt",
            "details": {"compiled": true},
        }),
    )
    .expect("the seeded Basalt page should exist after rerender");
    let html = basalt
        .compiled_body_html
        .expect("the rerendered Basalt page should include compiled HTML");
    assert_eq!(
        html.matches(r#"<span class="printuser avatarhover">"#)
            .count(),
        evidence["occurrences"]["source_total"]
            .as_u64()
            .expect("source total should be an integer") as usize,
    );

    let more_memos_prefix = concat!(
        r#"<div class="collapsible-block"><div class="collapsible-block-folded">"#,
        r#"<a class="collapsible-block-link" href="javascript:;">+&nbsp;More&nbsp;memos</a></div>"#,
        r#"<div class="collapsible-block-unfolded" style="display:none">"#,
    );
    assert_eq!(html.matches(more_memos_prefix).count(), 1);
    let more_memos_start = html
        .find(more_memos_prefix)
        .expect("the + More memos collapsible should exist");
    let collapsed_content = html[more_memos_start..]
        .find(r#"<div class="collapsible-block-content">"#)
        .map(|offset| more_memos_start + offset)
        .expect("the + More memos collapsible should have content");
    let collapsed_end = html[collapsed_content..]
        .find("</div></div>")
        .map(|offset| collapsed_content + offset)
        .expect("the + More memos content should close");
    let collapsed_html = &html[collapsed_content..collapsed_end];

    for user in fixture_users {
        let user_id = user["user_id"].as_i64().expect("fixture user ID");
        let name = user["name"].as_str().expect("fixture user name");
        let slug = user["slug"].as_str().expect("fixture user slug");
        let counts = evidence_users
            .iter()
            .find(|entry| entry["user_id"].as_i64() == Some(user_id))
            .expect("each runtime-state user should have evidence counts");
        let occurrences = counts["source_occurrences"]
            .as_u64()
            .expect("source occurrences should be an integer")
            as usize;
        let profile = format!("http://www.wikidot.com/user:info/{slug}");
        let onclick =
            format!("WIKIDOT.page.listeners.userInfo({user_id}); return false;");
        let avatar_prefix = format!(
            r#"<span class="printuser avatarhover"><a href="{profile}" onclick="{onclick}"><img class="small" "#,
        );
        let name_suffix =
            format!(r#"</a><a href="{profile}" onclick="{onclick}">{name}</a></span>"#,);
        assert_eq!(html.matches(&avatar_prefix).count(), occurrences, "{html}");
        assert_eq!(html.matches(&name_suffix).count(), occurrences, "{html}");

        if counts["source_occurrences"] != counts["rendered_avatar_occurrences"] {
            assert_eq!(collapsed_html.matches(&avatar_prefix).count(), 1, "{html}");
            assert_eq!(collapsed_html.matches(&name_suffix).count(), 1, "{html}");
        }
    }
}

#[tokio::test]
async fn listusers_module_matches_live_preview_and_runtime_viewer() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    set_test_user_name(&runner, SAMPLE_USER_ID, "ListUsers Person").await;
    set_test_user_name(&runner, ADMIN_USER_ID, "ListUsers Admin").await;

    let preview_source = concat!(
        "[[module ListUsers users=\".\"]]\n",
        "**%%title%%** (//%%name%%//) #%%number%% UNKNOWN %%missing%%\n",
        "[[/module]]\n",
        "[[module ListUsers]]\n",
        "OMIT %%title%%\n",
        "[[/module]]",
    );

    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: None,
    });
    let anonymous_preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "ListUsers anonymous preview",
            "wikitext": preview_source,
        }),
    );
    assert!(
        !anonymous_preview.body.contains("ListUsers Person")
            && !anonymous_preview.body.contains("[[module ListUsers"),
        "users=\".\" should render nothing for anonymous viewers:\n{}",
        anonymous_preview.body,
    );
    assert!(
        anonymous_preview.body.contains(
            r#"<div class="error-block">Currently only users="." is implemented.</div>"#
        ),
        "omitted users should match live Wikidot's error block:\n{}",
        anonymous_preview.body,
    );

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(SAMPLE_USER_ID),
        site_id: Some(site_id),
        page_reference: None,
    });
    let authenticated_preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "ListUsers authenticated preview",
            "wikitext": preview_source,
        }),
    );
    assert!(
        authenticated_preview
            .body
            .contains(r#"<p><strong>ListUsers Person</strong> (<em>listusers-person</em>) #-5 UNKNOWN %%missing%%</p>"#),
        "authenticated users=\".\" should substitute the current viewer and leave unknown variables literal:\n{}",
        authenticated_preview.body,
    );
    assert!(
        authenticated_preview.body.contains(
            r#"<div class="error-block">Currently only users="." is implemented.</div>"#
        ),
        "authenticated omitted users should still render the live error block:\n{}",
        authenticated_preview.body,
    );

    let page_source = concat!(
        "[[html]]\n",
        "<p>stable authored block</p>\n",
        "[[/html]]\n",
        "VIEW_START\n",
        "[[module ListUsers users=\".\"]]\n",
        "VIEW %%title%% %%name%% %%number%%\n",
        "[[html]]\n",
        "<p>viewer-only %%title%% %%name%% %%number%%</p>\n",
        "[[/html]]\n",
        "[[/module]]\n",
        "VIEW_END",
    );
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-listusers-viewer",
        "Fixture ListUsers Viewer",
        page_source,
    )
    .await;

    let stored_page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "fixture-listusers-viewer",
        }),
    )
    .expect("ListUsers holder should exist");
    assert!(
        !stored_page
            .compiled_body_html
            .as_deref()
            .unwrap_or_default()
            .contains("VIEW ListUsers Person"),
        "stored revision HTML must not bake in the editor identity:\n{:?}",
        stored_page.compiled_body_html,
    );
    let stored_text_blocks =
        snapshot_page_text_blocks(&runner, stored_page.page_id).await;
    assert_eq!(
        stored_text_blocks.len(),
        1,
        "the authoritative save should persist only the viewer-independent HTML block",
    );

    let sample_session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: SAMPLE_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "ListUsers runtime viewer test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("sample session should be created");

    let anonymous_view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": "fixture-listusers-viewer", "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let anonymous_body = match anonymous_view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found anonymous ListUsers view, got {other:?}"),
    };
    assert!(
        !anonymous_body.contains("VIEW ListUsers Person")
            && !anonymous_body.contains("[[module ListUsers"),
        "anonymous page view should render an empty users=\".\" module:\n{anonymous_body}",
    );
    assert_eq!(
        snapshot_page_text_blocks(&runner, stored_page.page_id).await,
        stored_text_blocks,
        "an anonymous GET rerender must not rewrite the saved page's hosted text blocks",
    );

    let authenticated_view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": sample_session_token,
            "route": {"slug": "fixture-listusers-viewer", "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let authenticated_body = match authenticated_view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found authenticated ListUsers view, got {other:?}"),
    };
    assert!(
        authenticated_body.contains("<p>VIEW ListUsers Person listusers-person -5</p>"),
        "authenticated page view should render ListUsers for the request viewer:\n{authenticated_body}",
    );
    assert_eq!(
        snapshot_page_text_blocks(&runner, stored_page.page_id).await,
        stored_text_blocks,
        "a viewer-dependent GET rerender must not persist that viewer's identity to hosted text blocks",
    );

    let admin_session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "ListUsers second runtime viewer test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");
    let admin_view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": admin_session_token,
            "route": {"slug": "fixture-listusers-viewer", "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let admin_body = match admin_view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found admin ListUsers view, got {other:?}"),
    };
    assert!(
        admin_body.contains("<p>VIEW ListUsers Admin listusers-admin -1</p>"),
        "a second authenticated viewer should receive their own ListUsers output:\n{admin_body}",
    );
    assert_eq!(
        snapshot_page_text_blocks(&runner, stored_page.page_id).await,
        stored_text_blocks,
        "a second viewer-dependent GET rerender must leave the authoritative text blocks unchanged",
    );
}

#[tokio::test]
async fn members_module_ssr_and_ajax_share_100_row_duplicate_relation_boundary() {
    const FIXTURE_USER_ID: i64 = 19_103_500;

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let transaction = runner.context().transaction();

    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            concat!(
                "DELETE FROM relation WHERE dest_type = 'site' AND dest_id = $1 ",
                "AND from_type = 'user' AND relation_type IN ('member', 'site-member')",
            ),
            [Value::from(site_id)],
        ))
        .await
        .expect("existing member-directory fixtures should be cleared");
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            concat!(
                "INSERT INTO known_user (user_id) ",
                "SELECT $1 + fixture.n::BIGINT FROM generate_series(0, 100) AS fixture(n)",
            ),
            [Value::from(FIXTURE_USER_ID)],
        ))
        .await
        .expect("SSR member known-user fixtures should be inserted");
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            concat!(
                "INSERT INTO wikidot_user (",
                "user_id, created_at, fetched_at, is_deleted, name, slug, karma, is_pro",
                ") SELECT $1 + fixture.n::BIGINT, ",
                "TIMESTAMPTZ '2020-01-01 00:00:00+00', NOW(), fixture.n > 50, ",
                "'SSR Member ' || lpad(fixture.n::TEXT, 3, '0'), ",
                "'ssr-member-' || lpad(fixture.n::TEXT, 3, '0'), 2, FALSE ",
                "FROM generate_series(0, 100) AS fixture(n)",
            ),
            [Value::from(FIXTURE_USER_ID)],
        ))
        .await
        .expect("SSR member identity fixtures should be inserted");
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            concat!(
                "INSERT INTO relation (",
                "relation_type, dest_type, dest_id, from_type, from_id, metadata, created_by, created_at",
                ") SELECT 'member', 'site', $2, 'user', $1 + fixture.n::BIGINT, ",
                "'{}'::jsonb, $3, ",
                "TIMESTAMPTZ '2020-01-01 00:00:00+00' + fixture.n * INTERVAL '1 second' ",
                "FROM generate_series(0, 100) AS fixture(n)",
            ),
            [
                Value::from(FIXTURE_USER_ID),
                Value::from(site_id),
                Value::from(SYSTEM_USER_ID),
            ],
        ))
        .await
        .expect("SSR member relation fixtures should be inserted");

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "Members SSR cardinality",
            "wikitext": "[[module Members]]",
        }),
    );
    assert_eq!(
        preview.body.matches("<tr>").count(),
        51,
        "a 51-member default directory should fit on one supported 100-row page:\n{}",
        preview.body,
    );
    assert!(
        preview.body.contains("SSR Member 050")
            && !preview.body.contains("class=\"pager\""),
        "the 51st member must render without a continuation target:\n{}",
        preview.body,
    );

    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            concat!(
                "UPDATE wikidot_user SET is_deleted = FALSE ",
                "WHERE user_id BETWEEN $1 + 51 AND $1 + 100",
            ),
            [Value::from(FIXTURE_USER_ID)],
        ))
        .await
        .expect("continuation identity fixtures should become eligible");
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            concat!(
                "INSERT INTO relation (",
                "relation_type, dest_type, dest_id, from_type, from_id, metadata, created_by, created_at",
                ") VALUES ('site-member', 'site', $2, 'user', $1, '{}'::jsonb, $3, ",
                "TIMESTAMPTZ '2020-01-01 00:03:20+00')",
            ),
            [
                Value::from(FIXTURE_USER_ID),
                Value::from(site_id),
                Value::from(SYSTEM_USER_ID),
            ],
        ))
        .await
        .expect("duplicate legacy member relation should be inserted");

    const PAGE_SLUG: &str = "fixture-members-ssr-cardinality";
    create_listpages_test_page(
        &mut runner,
        site_id,
        PAGE_SLUG,
        "Fixture Members SSR cardinality",
        concat!(
            "DEFAULT_START\n",
            "[[module Members]]\n",
            "DEFAULT_END\n",
            "NONDEFAULT_START\n",
            "[[module Members order=\"nameDesc\"]]\n",
            "NONDEFAULT_END",
        ),
    )
    .await;
    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": PAGE_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let html = match view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected saved Members cardinality fixture, got {other:?}"),
    };
    let default = html
        .split_once("DEFAULT_START")
        .and_then(|(_, tail)| tail.split_once("DEFAULT_END"))
        .map(|(default, _)| default)
        .expect("default directory should remain between authored markers");
    let nondefault = html
        .split_once("NONDEFAULT_START")
        .and_then(|(_, tail)| tail.split_once("NONDEFAULT_END"))
        .map(|(nondefault, _)| nondefault)
        .expect("non-default directory should remain between authored markers");

    assert_eq!(
        default.matches("<tr>").count(),
        100,
        "the default saved-page slice must match the public AMC cardinality:\n{default}",
    );
    for index in 0..100 {
        assert!(
            default.contains(&format!("SSR Member {index:03}")),
            "SSR page 1 must contain consecutive distinct member {index:03} despite a duplicate relation:\n{default}",
        );
    }
    assert!(
        default.contains("SSR Member 099")
            && !default.contains("SSR Member 100")
            && default.contains(r#"<span class="pager-no">page 1 of 2</span>"#)
            && default.contains("updateMemberList1(2)")
            && default.contains("membership/MembersListModule"),
        "the 101st default member must remain behind the supported page-2 continuation:\n{default}",
    );
    assert_eq!(
        nondefault.matches("<tr>").count(),
        100,
        "the documented non-default order may render its evidenced first slice:\n{nondefault}",
    );
    assert!(
        nondefault.find("SSR Member 100") < nondefault.find("SSR Member 099")
            && !nondefault.contains("SSR Member 000")
            && !nondefault.contains("class=\"pager\"")
            && !nondefault.contains("updateMemberList")
            && !nondefault.contains("membership/MembersListModule"),
        "a non-default first slice must not advertise an AMC continuation Framerail rejects:\n{nondefault}",
    );

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
    let continuation = run_endpoint!(
        runner,
        wikidot_members_list_module,
        json!({
            "site_id": site_id,
            "parameters": {"group": "", "order": "joined", "page": "2"},
        }),
    );
    assert_eq!(continuation.status, "ok");
    assert_eq!(continuation.body.matches("<tr>").count(), 1);
    assert!(
        continuation.body.contains("SSR Member 100")
            && !continuation.body.contains("SSR Member 099"),
        "the supported continuation must begin exactly after the 100-row SSR slice:\n{}",
        continuation.body,
    );
}

#[tokio::test]
async fn members_module_queries_only_visible_site_members_and_roles() {
    const ALPHA_USER_ID: i64 = 19_103_200;
    const ZETA_USER_ID: i64 = 19_103_201;
    const DELETED_USER_ID: i64 = 19_103_202;
    const OTHER_SITE_USER_ID: i64 = 19_103_203;

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let other_site_id = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded comparison site should exist")
        .site
        .site_id;
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "INSERT INTO known_user (user_id) VALUES ($1), ($2), ($3), ($4)",
            [
                Value::from(ALPHA_USER_ID),
                Value::from(ZETA_USER_ID),
                Value::from(DELETED_USER_ID),
                Value::from(OTHER_SITE_USER_ID),
            ],
        ))
        .await
        .expect("member-directory known-user fixtures should be inserted");
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            concat!(
                "INSERT INTO wikidot_user (",
                "user_id, created_at, fetched_at, is_deleted, name, slug, karma, is_pro",
                ") VALUES ",
                "($1, NOW() - INTERVAL '1 second', NOW(), FALSE, 'Alpha Member', 'alpha-member', 2, FALSE), ",
                "($2, NOW() - INTERVAL '1 second', NOW(), FALSE, 'Zeta Member', 'zeta-member', 4, FALSE), ",
                "($3, NOW() - INTERVAL '1 second', NOW(), TRUE, 'Deleted Member', 'deleted-member', 0, FALSE), ",
                "($4, NOW() - INTERVAL '1 second', NOW(), FALSE, 'Other Site Member', 'other-site-member', 3, FALSE)",
            ),
            [
                Value::from(ALPHA_USER_ID),
                Value::from(ZETA_USER_ID),
                Value::from(DELETED_USER_ID),
                Value::from(OTHER_SITE_USER_ID),
            ],
        ))
        .await
        .expect("member-directory Wikidot fixtures should be inserted");
    for user_id in [ALPHA_USER_ID, ZETA_USER_ID, DELETED_USER_ID] {
        RelationService::create_site_member(
            runner.context(),
            CreateSiteMember {
                site_id,
                user_id,
                metadata: SiteMemberData {
                    accepted: SiteMemberAccepted::Accepted(SYSTEM_USER_ID),
                },
                created_by: SYSTEM_USER_ID,
            },
            common::IP_ADDRESS,
        )
        .await
        .expect("member-directory fixture membership should be created");
    }
    RelationService::create_site_member(
        runner.context(),
        CreateSiteMember {
            site_id: other_site_id,
            user_id: OTHER_SITE_USER_ID,
            metadata: SiteMemberData {
                accepted: SiteMemberAccepted::Accepted(SYSTEM_USER_ID),
            },
            created_by: SYSTEM_USER_ID,
        },
        common::IP_ADDRESS,
    )
    .await
    .expect("comparison-site member fixture should be created");
    let moderator_role = RoleService::get(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed("moderator")),
    )
    .await
    .expect("seeded moderator role should exist");
    RoleService::grant_role_to_user(
        runner.context(),
        GrantUserRoleInput {
            user_id: ALPHA_USER_ID,
            role_id: moderator_role.role_id,
            site_id,
            assigning_user_id: SYSTEM_USER_ID,
            expires_at: None,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("member-directory moderator fixture should be granted");

    let source = concat!(
        "MEMBERS_START\n",
        "[[module Members order=\"nameDesc\" showSince=\"false\"]]\n",
        "MEMBERS_END\n",
        "MODERATORS_START\n",
        "[[module Members group=\"moderators\"]]\n",
        "MODERATORS_END\n",
        "INVALID_START\n",
        "[[module Members group=\"owners\"]]\n",
        "INVALID_END",
    );
    let mut bodies = Vec::new();
    for actor in [None, Some(SAMPLE_USER_ID), Some(ADMIN_USER_ID)] {
        runner.set_request_context(RequestContext {
            session: None,
            user_id: actor,
            site_id: Some(site_id),
            page_reference: None,
        });
        bodies.push(
            run_endpoint!(
                runner,
                wikidot_page_preview,
                json!({
                    "site_id": site_id,
                    "title": "Members directory matrix",
                    "wikitext": source,
                }),
            )
            .body,
        );
    }

    for (actor, html) in ["anonymous", "non-member", "administrator"]
        .into_iter()
        .zip(&bodies)
    {
        let members = html
            .split_once("MEMBERS_START")
            .and_then(|(_, tail)| tail.split_once("MEMBERS_END"))
            .map(|(members, _)| members)
            .expect("member directory should remain between authored markers");
        let moderators = html
            .split_once("MODERATORS_START")
            .and_then(|(_, tail)| tail.split_once("MODERATORS_END"))
            .map(|(moderators, _)| moderators)
            .expect("moderator directory should remain between authored markers");

        let zeta = members
            .find("Zeta Member")
            .expect("visible Zeta member should render");
        let alpha = members
            .find("Alpha Member")
            .expect("visible Alpha member should render");
        assert!(
            zeta < alpha,
            "nameDesc must order after identity filtering: {members}"
        );
        assert!(
            !members.contains("Deleted Member")
                && !members.contains("since <span class=\"odate"),
            "{actor} output must hide deleted identities and honor showSince=false:\n{members}",
        );
        assert!(
            html.contains(r#"id="ml-1""#)
                && html.contains(r#"id="ml-2""#)
                && moderators.contains("Alpha Member")
                && !moderators.contains("Zeta Member"),
            "module IDs must be unique and moderator rows must come from this site's active role assignments:\n{html}",
        );
        assert!(
            !members.contains("updateMemberList")
                && !members.contains("membership/MembersListModule")
                && !moderators.contains("updateMemberList")
                && !moderators.contains("membership/MembersListModule"),
            "non-default order and group directories must not advertise an unsupported continuation:\n{html}",
        );
        assert!(
            !html.contains("ml-607935")
                && !html.contains("lambert-eggman")
                && !html.contains("user:info/deleted-member")
                && !html.contains("Other Site Member"),
            "directory output must not retain captured, deleted, or cross-site identities:\n{html}",
        );
        let invalid = html
            .split_once("INVALID_START")
            .and_then(|(_, tail)| tail.split_once("INVALID_END"))
            .map(|(invalid, _)| invalid)
            .expect("invalid module should remain between authored markers");
        assert!(
            invalid.contains("No such module")
                && !invalid.contains("Alpha Member")
                && !invalid.contains("Zeta Member"),
            "unverified groups must fail closed instead of widening the site query:\n{invalid}",
        );
    }
}

#[tokio::test]
async fn members_list_ajax_paginates_only_filtered_site_identities() {
    const FIXTURE_USER_ID: i64 = 19_103_300;

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let other_site_id = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded comparison site should exist")
        .site
        .site_id;
    let transaction = runner.context().transaction();

    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            concat!(
                "INSERT INTO known_user (user_id) ",
                "SELECT $1 + fixture.n::BIGINT FROM generate_series(0, 104) AS fixture(n)",
            ),
            [Value::from(FIXTURE_USER_ID)],
        ))
        .await
        .expect("Members Ajax known-user fixtures should be inserted");
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            concat!(
                "INSERT INTO wikidot_user (",
                "user_id, created_at, fetched_at, is_deleted, name, slug, karma, is_pro",
                ") ",
                "SELECT $1 + fixture.n::BIGINT, ",
                "TIMESTAMPTZ '2020-01-01 00:00:00+00' + fixture.n * INTERVAL '1 second', ",
                "TIMESTAMPTZ '2021-01-01 00:00:00+00', fixture.n = 102, ",
                "'AMC Member ' || lpad(fixture.n::TEXT, 3, '0'), ",
                "'amc-member-' || lpad(fixture.n::TEXT, 3, '0'), 2, FALSE ",
                "FROM generate_series(0, 102) AS fixture(n) ",
                "UNION ALL SELECT $1 + 104, TIMESTAMPTZ '2020-01-01 00:00:00+00', ",
                "TIMESTAMPTZ '2021-01-01 00:00:00+00', FALSE, ",
                "'Other Site AMC Member', 'other-site-amc-member', 2, FALSE",
            ),
            [Value::from(FIXTURE_USER_ID)],
        ))
        .await
        .expect("Members Ajax Wikidot-user fixtures should be inserted");
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            concat!(
                "INSERT INTO relation (",
                "relation_type, dest_type, dest_id, from_type, from_id, metadata, created_by, created_at",
                ") ",
                "SELECT CASE WHEN fixture.n = 101 THEN 'site-member' ELSE 'member' END, ",
                "'site', $2, 'user', $1 + fixture.n::BIGINT, '{}'::jsonb, $3, ",
                "TIMESTAMPTZ '2020-01-01 00:00:00+00' + (fixture.n + 2) * INTERVAL '1 second' ",
                "FROM generate_series(0, 101) AS fixture(n) ",
                "UNION ALL SELECT 'member', 'site', $2, 'user', $1 + 102, '{}'::jsonb, $3, ",
                "TIMESTAMPTZ '2020-01-01 00:00:00+00' ",
                "UNION ALL SELECT 'member', 'site', $2, 'user', $1 + 103, '{}'::jsonb, $3, ",
                "TIMESTAMPTZ '2020-01-01 00:00:01+00' ",
                "UNION ALL SELECT 'member', 'site', $4, 'user', $1 + 104, '{}'::jsonb, $3, ",
                "TIMESTAMPTZ '2019-12-31 23:59:59+00'",
            ),
            [
                Value::from(FIXTURE_USER_ID),
                Value::from(site_id),
                Value::from(SYSTEM_USER_ID),
                Value::from(other_site_id),
            ],
        ))
        .await
        .expect("Members Ajax relation fixtures should be inserted");

    let request_for = |request_site_id: i64, page: &str| {
        json!({
            "site_id": request_site_id,
            "parameters": {
                "group": "",
                "order": "joined",
                "page": page,
            },
        })
    };
    let request = |page: &str| request_for(site_id, page);
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        ..Default::default()
    });

    let page_one = run_endpoint!(runner, wikidot_members_list_module, request("1"));
    assert_eq!(page_one.status, "ok");
    assert_eq!(page_one.body.matches("<tr>").count(), 100);
    for index in 0..100 {
        assert!(
            page_one.body.contains(&format!("AMC Member {index:03}")),
            "page 1 should contain filtered member {index:03}:\n{}",
            page_one.body,
        );
    }
    assert!(
        !page_one.body.contains("AMC Member 100")
            && !page_one.body.contains("AMC Member 102")
            && !page_one.body.contains("Other Site AMC Member")
            && page_one
                .body
                .contains(r#"<span class="pager-no">page 1 of 2</span>"#),
        "page 1 must filter identities before its 100-row slice:\n{}",
        page_one.body,
    );

    let page_zero = run_endpoint!(runner, wikidot_members_list_module, request("0"));
    assert_eq!(page_zero.status, "ok");
    assert_eq!(page_zero.body.matches("<tr>").count(), 100);
    for index in 0..100 {
        assert!(
            page_zero.body.contains(&format!("AMC Member {index:03}")),
            "page 0 should alias page 1 member {index:03}:\n{}",
            page_zero.body,
        );
    }
    assert!(
        page_zero
            .body
            .contains(r#"<span class="pager-no">page 1 of 2</span>"#),
        "page 0 must use the page 1 pager state:\n{}",
        page_zero.body,
    );

    let page_two = run_endpoint!(runner, wikidot_members_list_module, request("2"));
    assert_eq!(page_two.status, "ok");
    assert_eq!(page_two.body.matches("<tr>").count(), 2);
    assert!(
        page_two.body.find("AMC Member 100") < page_two.body.find("AMC Member 101")
            && !page_two.body.contains("AMC Member 099")
            && !page_two.body.contains("AMC Member 102")
            && page_two
                .body
                .contains(r#"<span class="pager-no">page 2 of 2</span>"#)
            && page_two.body.contains(r#"<span class="current">2</span>"#)
            && page_two.body.contains("&laquo; previous")
            && !page_two.body.contains("next &raquo;"),
        "the filtered second page should retain the observed pager contract:\n{}",
        page_two.body,
    );

    for (actor_class, actor) in [
        ("sandbox-admin", ADMIN_USER_ID),
        ("sandbox-member", FIXTURE_USER_ID + 104),
        ("sandbox-moderator-or-nonmember", SAMPLE_USER_ID),
    ] {
        runner.set_request_context(RequestContext {
            user_id: Some(actor),
            site_id: Some(other_site_id),
            ..Default::default()
        });
        for page in ["2", "1468"] {
            let empty = run_endpoint!(
                runner,
                wikidot_members_list_module,
                request_for(other_site_id, page),
            );
            assert_eq!(empty.status, "ok");
            let (container, empty_body) = empty
                .body
                .split_once("\">")
                .expect("empty member response should have a container");
            assert!(
                container.starts_with("\n<div id=\"ml-")
                    && container
                        .trim_start_matches("\n<div id=\"ml-")
                        .bytes()
                        .all(|byte| byte.is_ascii_digit())
                    && empty_body
                        == "\n\t\tNo users.\t\t<div style=\"text-align: center\">\n\t\t\t\t\n\t</div>\n</div>"
                    && !empty.body.contains("<table>")
                    && !empty.body.contains("MembersListModule")
                    && !empty.body.contains("class=\"pager\""),
                "{actor_class} page {page} should retain the observed exact empty body:\n{}",
                empty.body,
            );
        }
    }

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        ..Default::default()
    });

    for parameters in [
        json!({"group": "", "order": "joined"}),
        json!({"group": "members", "order": "joined", "page": "1"}),
        json!({"group": "", "order": "name", "page": "1"}),
        json!({"group": "", "order": "joined", "page": "-1"}),
        json!({"group": "", "order": "joined", "page": "01"}),
        json!({"group": "", "order": "joined", "page": "1.0"}),
        json!({"group": "", "order": "joined", "page": "1", "extra": "1"}),
        json!({"group": "", "order": "joined", "page": "4294967296"}),
    ] {
        let rejected = run_endpoint!(
            runner,
            wikidot_members_list_module,
            json!({"site_id": site_id, "parameters": parameters}),
        );
        assert_eq!(rejected.status, "not_ok");
        assert!(rejected.body.is_empty());
    }

    let guest_role = RoleService::get(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed("guest")),
    )
    .await
    .expect("guest role should exist");
    let anonymous_role = RoleService::get(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed("anonymous")),
    )
    .await
    .expect("anonymous role should exist");
    let everyone_role = RoleService::get(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed("everyone")),
    )
    .await
    .expect("everyone role should exist");
    RolePermissionTable::delete_many()
        .filter(role_permission::Column::RoleId.is_in([
            guest_role.role_id,
            anonymous_role.role_id,
            everyone_role.role_id,
        ]))
        .filter(role_permission::Column::SiteId.eq(site_id))
        .filter(role_permission::Column::ResourceType.eq(Resource::Page))
        .filter(role_permission::Column::ResourceCategoryId.is_null())
        .filter(role_permission::Column::Action.eq(Action::View))
        .exec(runner.context().transaction())
        .await
        .expect("anonymous public page view permissions should be revoked");
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("member directory permission cache should be invalidated");
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
    let private_site = run_endpoint!(runner, wikidot_members_list_module, request("1"));
    assert_eq!(private_site.status, "not_ok");
    assert!(private_site.body.is_empty());

    runner.set_request_context(RequestContext {
        site_id: Some(other_site_id),
        ..Default::default()
    });
    let mismatched_site =
        run_endpoint_err!(runner, wikidot_members_list_module, request("1"),);
    assert_contains_error!(mismatched_site, ErrorType::PermissionDenied);
}

#[tokio::test]
async fn request_argument_page_views_do_not_persist_hosted_text_blocks() {
    fn section<'a>(html: &'a str, start: &str, end: &str) -> &'a str {
        html.split_once(start)
            .unwrap_or_else(|| panic!("missing section start {start:?}"))
            .1
            .split_once(end)
            .unwrap_or_else(|| panic!("missing section end {end:?}"))
            .0
    }

    const HOLDER: &str = "fixture-readonly-view-holder";
    const ALPHA: &str = "fixture-readonly-view-target-alpha";
    const BRAVO: &str = "fixture-readonly-view-target-bravo";
    const ALPHA_TAG: &str = "fixture-readonly-view-alpha-tag";
    const BRAVO_TAG: &str = "fixture-readonly-view-bravo-tag";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (slug, title, tag) in [
        (ALPHA, "Alpha Read-only View Target", ALPHA_TAG),
        (BRAVO, "Bravo Read-only View Target", BRAVO_TAG),
    ] {
        let revision_id =
            create_listpages_test_page(&mut runner, site_id, slug, title, "target body")
                .await;
        set_listpages_test_tags(&mut runner, site_id, slug, revision_id, &[tag]).await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER,
        "Fixture Read-only View Holder",
        concat!(
            "[[html]]<p>stable request-argument block</p>[[/html]]\n",
            "PAGER_START\n",
            "[[module ListPages name=\"fixture-readonly-view-target-*\" order=\"title\" separate=\"no\" perPage=\"1\" limit=\"2\"]]\n",
            "PAGER %%title%%\n",
            "[[html]]<p>PAGER_BLOCK %%title%%</p>[[/html]]\n",
            "[[/module]]\n",
            "PAGER_END\n",
            "TAG_START\n",
            "[[module ListPages name=\"fixture-readonly-view-target-*\" tags=\"@URL\" order=\"title\" separate=\"no\" limit=\"2\"]]\n",
            "TAG %%title%%\n",
            "[[html]]<p>TAG_BLOCK %%title%%</p>[[/html]]\n",
            "[[/module]]\n",
            "TAG_END",
        ),
    )
    .await;

    let holder = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": HOLDER}),
    )
    .expect("read-only view holder should exist");
    let stored_text_blocks = snapshot_page_text_blocks(&runner, holder.page_id).await;
    assert!(
        stored_text_blocks
            .iter()
            .any(|(_, contents)| String::from_utf8_lossy(contents)
                .contains("stable request-argument block")),
        "the authoritative render should persist its stable hosted block",
    );
    let stored_text_count = persisted_text_count(&runner).await;

    let view = async |extra: &str| {
        run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": null,
                "route": {"slug": HOLDER, "extra": extra},
                "locales": ["en-US", "en"],
            }),
        )
    };

    let second_page = match view("/p/2").await {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found /p/2 page view, got {other:?}"),
    };
    let pager = section(&second_page, "PAGER_START", "PAGER_END");
    assert!(
        pager.contains("PAGER Bravo Read-only View Target")
            && !pager.contains("PAGER Alpha Read-only View Target"),
        "/p/2 should render the second ListPages slice:\n{second_page}",
    );
    assert_eq!(
        snapshot_page_text_blocks(&runner, holder.page_id).await,
        stored_text_blocks,
        "GET /p/2 must not replace the authoritative hosted blocks with its selected row",
    );
    assert_eq!(
        persisted_text_count(&runner).await,
        stored_text_count,
        "GET /p/2 must not persist request-specific body, navigation, or style text",
    );

    let tagged = match view(&format!("/tag/{BRAVO_TAG}")).await {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found /tag page view, got {other:?}"),
    };
    let tagged_rows = section(&tagged, "TAG_START", "TAG_END");
    assert!(
        tagged_rows.contains("TAG Bravo Read-only View Target")
            && !tagged_rows.contains("TAG Alpha Read-only View Target"),
        "/tag should change only the request-specific ListPages result:\n{tagged}",
    );
    assert_eq!(
        snapshot_page_text_blocks(&runner, holder.page_id).await,
        stored_text_blocks,
        "GET /tag must not persist request-selected hosted blocks",
    );
    assert_eq!(
        persisted_text_count(&runner).await,
        stored_text_count,
        "GET /tag must not persist request-specific body, navigation, or style text",
    );
}

#[tokio::test]
async fn listdrafts_module_matches_live_empty_draft_state() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let source = "DRAFTS_START\n[[module ListDrafts pageType=\"exists\"]]\nDRAFTS_END";

    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: None,
    });
    for (case_id, source, closing_is_literal) in [
        (
            "exact-existing-page-filter",
            r#"[[module ListDrafts pageType="exists"]]"#,
            false,
        ),
        ("omitted-filter", "[[module ListDrafts]]", false),
        (
            "exact-missing-page-filter",
            r#"[[module ListDrafts pageType="notexists"]]"#,
            false,
        ),
        (
            "unsupported-filter-is-omitted",
            r#"[[module ListDrafts pageType="other"]]"#,
            false,
        ),
        (
            "empty-filter-is-omitted",
            r#"[[module ListDrafts pageType=""]]"#,
            false,
        ),
        (
            "single-quoted-filter-is-omitted",
            "[[module ListDrafts pageType='exists']]",
            false,
        ),
        (
            "bare-filter-is-omitted",
            "[[module ListDrafts pageType=exists]]",
            false,
        ),
        (
            "module-name-is-case-insensitive",
            r#"[[module LISTDRAFTS pageType="exists"]]"#,
            false,
        ),
        (
            "argument-name-is-case-sensitive",
            r#"[[module ListDrafts PAGETYPE="exists"]]"#,
            false,
        ),
        (
            "standalone-opener-leaves-closing-marker-literal",
            "[[module ListDrafts pageType=\"exists\"]]\n[[/module]]",
            true,
        ),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": format!("ListDrafts empty preview: {case_id}"),
                "wikitext": source,
            }),
        );
        assert!(
            preview.body.contains(r#"<div class="list-drafts-box">"#),
            "{case_id}: ListDrafts should render Wikidot's empty draft-list wrapper:\n{}",
            preview.body,
        );
        assert!(
            !preview.body.contains("[[module ListDrafts")
                && !preview.body.contains("[[module LISTDRAFTS")
                && !preview.body.contains("list-drafts-item"),
            "{case_id}: empty ListDrafts should not leak its opener or render draft items:\n{}",
            preview.body,
        );
        assert_eq!(
            preview.body.contains("[[/module]]"),
            closing_is_literal,
            "{case_id}: standalone closing-marker behavior should match the live preview:\n{}",
            preview.body,
        );
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-listdrafts-empty",
        "Fixture ListDrafts Empty",
        source,
    )
    .await;

    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": "fixture-listdrafts-empty", "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let body = match view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found ListDrafts view, got {other:?}"),
    };
    assert!(
        body.contains("DRAFTS_START")
            && body.contains(r#"<div class="list-drafts-box">"#)
            && body.contains("DRAFTS_END"),
        "saved page view should render the ListDrafts wrapper in place:\n{body}",
    );
    assert!(
        !body.contains("[[module ListDrafts") && !body.contains("list-drafts-item"),
        "saved page view should not leak raw ListDrafts source or render nonexistent drafts:\n{body}",
    );
}

#[tokio::test]
async fn listdrafts_module_reads_persisted_drafts_and_enforces_target_filters() {
    const EXISTING_SLUG: &str = "fixture-listdrafts-existing-target";
    const ABSENT_SLUG: &str = "fixture-listdrafts-absent-target";
    const PRIVATE_CATEGORY: &str = "fixture-listdrafts-private-category";
    const PRIVATE_SLUG: &str = "fixture-listdrafts-private-target";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let existing_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        EXISTING_SLUG,
        "Fixture ListDrafts Existing Target",
        "published existing target",
    )
    .await;
    let existing_page_id = PageRevisionTable::find_by_id(existing_revision)
        .one(runner.context().transaction())
        .await
        .expect("existing ListDrafts target revision should be readable")
        .expect("existing ListDrafts target revision should exist")
        .page_id;
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;
    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        PRIVATE_CATEGORY,
        ADMIN_USER_ID,
        &[Action::View, Action::Create, Action::Edit],
        "listdrafts-admin-mutator",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        PRIVATE_SLUG,
        "Fixture ListDrafts Private Target",
        "private published target",
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, PRIVATE_SLUG, PRIVATE_CATEGORY)
        .await;
    let private_page_id = PageTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page::Column::SiteId.eq(site_id))
                .add(page::Column::Slug.eq(PRIVATE_SLUG)),
        )
        .one(runner.context().transaction())
        .await
        .expect("private ListDrafts target should be readable")
        .expect("private ListDrafts target should exist")
        .page_id;

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(existing_page_id),
    );
    assert!(run_endpoint!(
        runner,
        page_draft_save,
        json!({
            "site_id": site_id,
            "user_id": ADMIN_USER_ID,
            "page_id": Some(existing_page_id),
            "slug": EXISTING_SLUG.to_owned(),
            "title": "Existing draft v1".to_owned(),
            "wikitext": "existing draft source v1".to_owned(),
        })
    ));
    assert!(run_endpoint!(
        runner,
        page_draft_save,
        json!({
            "site_id": site_id,
            "user_id": ADMIN_USER_ID,
            "page_id": null,
            "slug": ABSENT_SLUG.to_owned(),
            "title": "Absent draft v1".to_owned(),
            "wikitext": "absent draft source v1".to_owned(),
        })
    ));
    assert!(run_endpoint!(
        runner,
        page_draft_save,
        json!({
            "site_id": site_id,
            "user_id": ADMIN_USER_ID,
            "page_id": Some(private_page_id),
            "slug": PRIVATE_SLUG.to_owned(),
            "title": "Private draft must stay hidden".to_owned(),
            "wikitext": "private draft source".to_owned(),
        })
    ));
    assert!(run_endpoint!(
        runner,
        page_draft_save,
        json!({
            "site_id": site_id,
            "user_id": ADMIN_USER_ID,
            "page_id": null,
            "slug": format!("{PRIVATE_CATEGORY}:absent"),
            "title": "Private absent draft must stay hidden".to_owned(),
            "wikitext": "private absent draft source".to_owned(),
        })
    ));

    let all = PageDraftService::list(runner.context(), site_id, PageDraftPageType::All)
        .await
        .expect("all persisted drafts should be readable");
    assert_eq!(all.len(), 4);
    assert!(all.iter().any(|draft| draft.slug == EXISTING_SLUG));
    assert!(all.iter().any(|draft| draft.slug == ABSENT_SLUG));
    assert!(
        all.iter()
            .any(|draft| draft.slug == format!("{PRIVATE_CATEGORY}:absent"))
    );

    let existing =
        PageDraftService::list(runner.context(), site_id, PageDraftPageType::Exists)
            .await
            .expect("existing-page draft filter should be readable");
    let existing_slugs = existing
        .iter()
        .map(|draft| draft.slug.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        existing_slugs,
        BTreeSet::from([EXISTING_SLUG, PRIVATE_SLUG]),
    );

    runner.set_request_context(RequestContext {
        user_id: None,
        site_id: Some(site_id),
        ..Default::default()
    });
    let all_preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "ListDrafts persisted all",
            "wikitext": "[[module ListDrafts]]",
        }),
    );
    assert!(all_preview.body.contains("list-drafts-item"));
    assert!(all_preview.body.contains("Existing draft v1"));
    assert!(all_preview.body.contains("Absent draft v1"));
    assert!(!all_preview.body.contains("Private draft must stay hidden"));
    assert!(
        !all_preview
            .body
            .contains("Private absent draft must stay hidden")
    );

    let exists_preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "ListDrafts persisted existing",
            "wikitext": r##"[[module ListDrafts pageType="exists"]]"##,
        }),
    );
    assert!(exists_preview.body.contains("Existing draft v1"));
    assert!(!exists_preview.body.contains("Absent draft v1"));
    assert!(
        !exists_preview
            .body
            .contains("Private draft must stay hidden")
    );

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Id(private_page_id),
    );
    let error = run_endpoint_err!(
        runner,
        page_draft_save,
        json!({
            "site_id": site_id,
            "user_id": SAMPLE_USER_ID,
            "page_id": Some(private_page_id),
            "slug": PRIVATE_SLUG.to_owned(),
            "title": "must not edit private target".to_owned(),
            "wikitext": "must not edit private target".to_owned(),
        })
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);
    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Slug(Cow::Owned(format!("{PRIVATE_CATEGORY}:absent"))),
    );
    let error = run_endpoint_err!(
        runner,
        page_draft_save,
        json!({
            "site_id": site_id,
            "user_id": SAMPLE_USER_ID,
            "page_id": null,
            "slug": format!("{PRIVATE_CATEGORY}:absent"),
            "title": "must not create private target draft".to_owned(),
            "wikitext": "must not create private target draft".to_owned(),
        })
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    for (case_id, source) in [
        (
            "notexists-is-omitted",
            r##"[[module ListDrafts pageType="notexists"]]"##,
        ),
        ("empty-is-omitted", r##"[[module ListDrafts pageType=""]]"##),
        (
            "unsupported-is-omitted",
            r##"[[module ListDrafts pageType="other"]]"##,
        ),
        (
            "single-quoted-is-omitted",
            "[[module ListDrafts pageType='exists']]",
        ),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": format!("ListDrafts {case_id}"),
                "wikitext": source,
            }),
        );
        assert!(preview.body.contains("Existing draft v1"), "{case_id}");
        assert!(preview.body.contains("Absent draft v1"), "{case_id}");
    }

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(existing_page_id),
    );
    assert!(run_endpoint!(
        runner,
        page_draft_save,
        json!({
            "site_id": site_id,
            "user_id": ADMIN_USER_ID,
            "page_id": Some(existing_page_id),
            "slug": EXISTING_SLUG.to_owned(),
            "title": "Existing draft v2".to_owned(),
            "wikitext": "existing draft source v2".to_owned(),
        })
    ));
    let updated =
        PageDraftService::list(runner.context(), site_id, PageDraftPageType::Exists)
            .await
            .expect("updated existing-page draft should be readable");
    assert_eq!(updated.len(), 2);
    assert!(
        updated.iter().any(
            |draft| draft.slug == EXISTING_SLUG && draft.title == "Existing draft v2"
        )
    );

    assert!(run_endpoint!(
        runner,
        page_draft_exists,
        json!({
            "site_id": site_id,
            "user_id": ADMIN_USER_ID,
            "page_id": null,
            "slug": ABSENT_SLUG.to_owned(),
        })
    ));
    assert!(run_endpoint!(
        runner,
        page_draft_remove,
        json!({
            "site_id": site_id,
            "user_id": ADMIN_USER_ID,
            "page_id": null,
            "slug": ABSENT_SLUG.to_owned(),
        })
    ));
    assert!(!run_endpoint!(
        runner,
        page_draft_exists,
        json!({
            "site_id": site_id,
            "user_id": ADMIN_USER_ID,
            "page_id": null,
            "slug": ABSENT_SLUG.to_owned(),
        })
    ));

    runner.set_request_context(RequestContext {
        user_id: None,
        site_id: Some(site_id),
        ..Default::default()
    });
    let error = run_endpoint_err!(
        runner,
        page_draft_save,
        json!({
            "site_id": site_id,
            "user_id": ADMIN_USER_ID,
            "page_id": null,
            "slug": "fixture-listdrafts-anonymous-save".to_owned(),
            "title": "must not persist".to_owned(),
            "wikitext": "must not persist".to_owned(),
        })
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);
}

#[tokio::test]
async fn categories_runtime_inventory_is_scoped_to_the_request_actor() {
    const PRIVATE_CATEGORY: &str = "fixture-categories-private-runtime";
    const PRIVATE_PAGE: &str = "fixture-categories-private-runtime-page";
    const PUBLIC_PAGE: &str = "fixture-categories-public-runtime-page";
    const HOLDER: &str = "fixture-categories-runtime-holder";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        PUBLIC_PAGE,
        "Fixture Categories Public Runtime Page",
        "public Categories runtime fixture",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        PRIVATE_PAGE,
        "Fixture Categories Private Runtime Page",
        "private Categories runtime fixture",
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, PRIVATE_PAGE, PRIVATE_CATEGORY)
        .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER,
        "Fixture Categories Runtime Holder",
        "CATEGORIES_RUNTIME_START\n[[module Categories includeHidden=\"true\"]]\nCATEGORIES_RUNTIME_END",
    )
    .await;

    let admin_session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "Categories actor runtime test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");

    let body = |view| match view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected a found Categories page view, got {other:?}"),
    };

    let anonymous = body(run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": HOLDER, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    ));
    assert!(
        anonymous.contains("CATEGORIES_RUNTIME_START")
            && anonymous.contains("CATEGORIES_RUNTIME_END")
            && anonymous.contains("<h3>_default</h3>"),
        "anonymous page_view should runtime-render the public category inventory:\n{anonymous}",
    );
    assert!(
        !anonymous.contains(PRIVATE_CATEGORY),
        "anonymous Categories output must not reveal an inaccessible category:\n{anonymous}",
    );

    let admin = body(run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": admin_session_token,
            "route": {"slug": HOLDER, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    ));
    assert!(
        admin.contains(&format!("<h3>{PRIVATE_CATEGORY}</h3>")),
        "an authorized actor should receive the private category in the same public page_view seam:\n{admin}",
    );
}

#[tokio::test]
async fn sitechanges_default_snapshot_filters_before_the_initial_page_limit() {
    const PRIVATE_CATEGORY: &str = "fixture-sitechanges-private";
    const PRIVATE_PAGE: &str = "fixture-sitechanges-private-page";
    const PUBLIC_PAGE: &str = "fixture-sitechanges-public-page";
    const HIDDEN_PAGE: &str = "fixture-sitechanges-hidden-page";
    const PUBLIC_TITLE: &str = "Fixture SiteChanges Public Page";
    const PRIVATE_TITLE: &str = "Fixture SiteChanges Private Page";
    const HIDDEN_TITLE: &str = "Fixture SiteChanges Hidden Page";
    const HOLDER: &str = "fixture-sitechanges-holder";
    const SOURCE: &str = "SITECHANGES_START\n[[module SiteChanges]]\nSITECHANGES_END";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        PUBLIC_PAGE,
        PUBLIC_TITLE,
        "public SiteChanges fixture",
    )
    .await;
    let hidden_revision_id = create_listpages_test_page(
        &mut runner,
        site_id,
        HIDDEN_PAGE,
        HIDDEN_TITLE,
        "hidden SiteChanges fixture",
    )
    .await;
    let hidden_page_id = PageTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page::Column::SiteId.eq(site_id))
                .add(page::Column::Slug.eq(HIDDEN_PAGE)),
        )
        .one(runner.context().transaction())
        .await
        .expect("hidden SiteChanges page lookup should succeed")
        .expect("hidden SiteChanges page should exist")
        .page_id;
    run_endpoint!(
        runner,
        page_revision_edit,
        json!({
            "site_id": site_id,
            "page_id": hidden_page_id,
            "revision_id": hidden_revision_id,
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
            "hidden": ["comments", "title", "slug"],
        }),
    );
    let mut private_revision_id = create_listpages_test_page(
        &mut runner,
        site_id,
        PRIVATE_PAGE,
        PRIVATE_TITLE,
        "private SiteChanges fixture",
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, PRIVATE_PAGE, PRIVATE_CATEGORY)
        .await;
    for index in 0..22 {
        let tag = format!("sitechanges-private-{index}");
        private_revision_id = set_listpages_test_tags(
            &mut runner,
            site_id,
            PRIVATE_PAGE,
            private_revision_id,
            &[&tag],
        )
        .await;
    }
    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER,
        "Fixture SiteChanges Holder",
        SOURCE,
    )
    .await;

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "SiteChanges anonymous preview",
            "wikitext": SOURCE,
        }),
    );
    for expected in [
        r#"<div class="site-changes-box">"#,
        "Revision types:",
        r#"id="rev-type-all" checked="checked""#,
        r#"id="rev-category""#,
        r#"id="rev-perpage""#,
        r#"class="changes-list" id="site-changes-list""#,
        PUBLIC_TITLE,
    ] {
        assert!(
            preview.body.contains(expected),
            "anonymous preview should contain {expected:?}:\n{}",
            preview.body,
        );
    }
    for forbidden in [
        PRIVATE_TITLE,
        HIDDEN_TITLE,
        PRIVATE_CATEGORY,
        "[[module SiteChanges",
    ] {
        assert!(
            !preview.body.contains(forbidden),
            "anonymous preview must not contain {forbidden:?}:\n{}",
            preview.body,
        );
    }

    let anonymous_view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": HOLDER, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let anonymous_body = match anonymous_view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected a found SiteChanges page view, got {other:?}"),
    };
    assert!(
        anonymous_body.contains(PUBLIC_TITLE)
            && !anonymous_body.contains(PRIVATE_TITLE)
            && !anonymous_body.contains(HIDDEN_TITLE),
        "saved anonymous page_view must filter hidden revisions before selecting its first 20 rows:\n{anonymous_body}",
    );

    let admin_session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "SiteChanges actor runtime test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");
    let admin_view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": admin_session_token,
            "route": {"slug": HOLDER, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let admin_body = match admin_view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected a found admin SiteChanges page view, got {other:?}"),
    };
    assert!(
        admin_body.contains(PRIVATE_TITLE)
            && admin_body.contains(&format!(">{PRIVATE_CATEGORY}</option>"))
            && admin_body.contains(r#"<div class="pager">"#),
        "an authorized actor should receive private changes, the category selector, and a pager:\n{admin_body}",
    );
}

#[tokio::test]
async fn wikidot_site_changes_ajax_endpoint_filters_before_pagination_and_matches_observed_reads()
 {
    const PRIVATE_CATEGORY: &str = "fixture-sitechanges-ajax-private";
    const PRIVATE_PAGE: &str = "fixture-sitechanges-ajax-private-page";
    const PRIVATE_TITLE: &str = "Fixture SiteChanges Ajax Private Page";
    const PUBLIC_PAGE: &str = "fixture-sitechanges-ajax-public-page";
    const PUBLIC_TITLE: &str = "Fixture SiteChanges Ajax Public Page";
    const HOLDER: &str = "fixture-sitechanges-ajax-holder";

    async fn insert_source_revisions(
        runner: &TestRunner,
        initial_revision_id: i64,
        count: i32,
        revision_number_offset: i32,
        time_offset_seconds: i32,
    ) {
        let transaction = runner.context().transaction();
        transaction
            .execute_raw(Statement::from_sql_and_values(
                transaction.get_database_backend(),
                concat!(
                    "INSERT INTO page_revision (",
                    "revision_type, created_at, updated_at, revision_number, page_id, ",
                    "site_id, user_id, from_wikidot, changes, wikitext_hash, ",
                    "compiled_body_html_hash, compiled_body_styles_hash, ",
                    "compiled_top_bar_html_hash, compiled_side_bar_html_hash, ",
                    "compiled_at, compiled_generator, comments, hidden, title, ",
                    "alt_title, slug, tags",
                    ") SELECT ",
                    "'regular', NOW() + make_interval(secs => ",
                    "($4 + fixture.sequence)::DOUBLE PRECISION), ",
                    "source.updated_at, $3 + fixture.sequence, source.page_id, ",
                    "source.site_id, source.user_id, source.from_wikidot, ",
                    "ARRAY['wikitext']::TEXT[], source.wikitext_hash, ",
                    "source.compiled_body_html_hash, source.compiled_body_styles_hash, ",
                    "source.compiled_top_bar_html_hash, source.compiled_side_bar_html_hash, ",
                    "source.compiled_at, source.compiled_generator, ",
                    "'source fixture ' || ($3 + fixture.sequence), source.hidden, ",
                    "source.title, source.alt_title, source.slug, source.tags ",
                    "FROM page_revision source ",
                    "CROSS JOIN generate_series(1, $2::INTEGER) fixture(sequence) ",
                    "WHERE source.revision_id = $1",
                ),
                [
                    Value::from(initial_revision_id),
                    Value::from(count),
                    Value::from(revision_number_offset),
                    Value::from(time_offset_seconds),
                ],
            ))
            .await
            .expect("SiteChanges source fixtures should be inserted");
    }

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;

    let public_revision_id = create_listpages_test_page(
        &mut runner,
        site_id,
        PUBLIC_PAGE,
        PUBLIC_TITLE,
        "public SiteChanges Ajax fixture",
    )
    .await;
    let private_revision_id = create_listpages_test_page(
        &mut runner,
        site_id,
        PRIVATE_PAGE,
        PRIVATE_TITLE,
        "private SiteChanges Ajax fixture",
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, PRIVATE_PAGE, PRIVATE_CATEGORY)
        .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER,
        "Fixture SiteChanges Ajax Holder",
        "[[module SiteChanges]]",
    )
    .await;

    insert_source_revisions(&runner, public_revision_id, 2_105, 0, 100).await;
    insert_source_revisions(&runner, private_revision_id, 25, 0, 1_000).await;

    let holder = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": HOLDER}),
    )
    .expect("SiteChanges Ajax holder should exist");
    let public_page = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": PUBLIC_PAGE}),
    )
    .expect("SiteChanges Ajax public page should exist");

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(
            "fixture-sitechanges-unrelated-request-page",
        ))),
        ..Default::default()
    });

    let mut page_bodies = Vec::new();
    for page in ["1", "2", "3"] {
        let output = run_endpoint!(
            runner,
            wikidot_site_changes_module,
            json!({
                "site_id": site_id,
                "page_id": holder.page_id.to_string(),
                "page": page,
                "perpage": "20",
                "category_id": "",
                "options": "{\"all\":true}",
            }),
        );
        assert_eq!(output.status, "ok");
        assert!(
            !output.body.contains(PRIVATE_TITLE),
            "private revisions must be filtered before page selection:\n{}",
            output.body,
        );
        page_bodies.push(output.body);
    }

    for (body, first_revision, last_revision, outside_revision) in [
        (&page_bodies[0], 2_105, 2_086, 2_085),
        (&page_bodies[1], 2_085, 2_066, 2_065),
        (&page_bodies[2], 2_065, 2_046, 2_045),
    ] {
        assert!(body.contains(PUBLIC_TITLE));
        assert!(body.contains(&format!("(rev. {first_revision})")), "{body}");
        assert!(body.contains(&format!("(rev. {last_revision})")), "{body}");
        assert!(
            !body.contains(&format!("(rev. {outside_revision})")),
            "adjacent-page revision leaked across the page boundary: {body}",
        );
    }
    assert!(
        page_bodies[0].contains(r#">page 1</span>"#)
            && page_bodies[0].contains("updateList(3)")
            && page_bodies[0].contains("next &raquo;"),
        "page one should preserve the sealed pager shape: {}",
        page_bodies[0],
    );
    assert!(
        page_bodies[1].contains(r#">page 2</span>"#)
            && page_bodies[1].contains("&laquo; previous")
            && page_bodies[1].contains("updateList(4)"),
        "page two should preserve the sealed pager shape: {}",
        page_bodies[1],
    );
    assert!(
        page_bodies[2].contains(r#">page 3</span>"#)
            && page_bodies[2].contains("updateList(5)")
            && page_bodies[2].contains(r#"<span class="dots">...</span>"#),
        "page three should preserve the sealed pager shape: {}",
        page_bodies[2],
    );

    // client-page-one-default and client-later-page: browser-only host fields are
    // absent, and page 2 remains the second 1000-row page.
    let wikidot_py_page_one = run_endpoint!(
        runner,
        wikidot_site_changes_module,
        json!({
            "site_id": site_id,
            "page": "1",
            "perpage": "1000",
            "options": "{\"all\":true}",
        }),
    );
    assert_eq!(wikidot_py_page_one.status, "ok");
    assert!(wikidot_py_page_one.body.contains("(rev. 2105)"));
    assert!(wikidot_py_page_one.body.contains("(rev. 1106)"));
    assert!(!wikidot_py_page_one.body.contains("(rev. 1105)"));
    assert!(!wikidot_py_page_one.body.contains(PRIVATE_TITLE));

    let wikidot_py_page_two = run_endpoint!(
        runner,
        wikidot_site_changes_module,
        json!({
            "site_id": site_id,
            "page": "2",
            "perpage": "1000",
            "options": "{\"all\":true}",
        }),
    );
    assert_eq!(wikidot_py_page_two.status, "ok");
    assert!(wikidot_py_page_two.body.contains("(rev. 1105)"));
    assert!(wikidot_py_page_two.body.contains("(rev. 106)"));
    assert!(!wikidot_py_page_two.body.contains("(rev. 1106)"));

    // control-bad-perpage must not run an unbounded query.
    let malformed_perpage = run_endpoint!(
        runner,
        wikidot_site_changes_module,
        json!({
            "site_id": site_id,
            "page": "1",
            "perpage": "not-a-number",
            "options": "{\"all\":true}",
        }),
    );
    assert_eq!(malformed_perpage.status, "ok");
    assert_eq!(
        malformed_perpage.body,
        "\tSorry, no revisions matching your criteria.",
    );

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        ..Default::default()
    });
    let authorized = run_endpoint!(
        runner,
        wikidot_site_changes_module,
        json!({
            "site_id": site_id,
            "page": "2",
            "perpage": "1000",
            "options": "{\"all\":true}",
        }),
    );
    assert_eq!(authorized.status, "ok");
    assert!(authorized.body.contains(PRIVATE_TITLE));

    runner.set_request_context(RequestContext {
        site_id: Some(site_id + 1),
        ..Default::default()
    });
    run_endpoint_err!(
        runner,
        wikidot_site_changes_module,
        json!({
            "site_id": site_id,
            "page": "1",
            "perpage": "1000",
            "options": "{\"all\":true}",
        }),
    );
    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });

    for mixed in [
        json!({
            "site_id": site_id,
            "page_id": holder.page_id.to_string(),
            "page": "1",
            "perpage": "1000",
            "options": "{\"all\":true}",
        }),
        json!({
            "site_id": site_id,
            "page": "1",
            "perpage": "1000",
            "category_id": "",
            "options": "{\"all\":true}",
        }),
    ] {
        let output = run_endpoint!(runner, wikidot_site_changes_module, mixed);
        assert_eq!(output.status, "not_ok");
        assert!(output.body.is_empty());
    }

    create_empty_file_fixture(
        &runner,
        site_id,
        public_page.page_id,
        "sitechanges-ajax-file.txt",
    )
    .await;
    let files = run_endpoint!(
        runner,
        wikidot_site_changes_module,
        json!({
            "site_id": site_id,
            "page_id": holder.page_id.to_string(),
            "page": "1",
            "perpage": "20",
            "category_id": "",
            "options": "{\"files\":true}",
        }),
    );
    assert_eq!(files.status, "ok");
    assert!(
        files.body.contains(PUBLIC_TITLE)
            && files.body.contains("file/attachment action\">F")
            && files.body.contains("create file fixture")
            && !files.body.contains("source fixture"),
        "files filter should select only stored file activity:\n{}",
        files.body,
    );

    for options in ["{\"source\":true}", "{}"] {
        let output = run_endpoint!(
            runner,
            wikidot_site_changes_module,
            json!({
                "site_id": site_id,
                "page_id": holder.page_id.to_string(),
                "page": "1",
                "perpage": "20",
                "category_id": "",
                "options": options,
            }),
        );
        assert_eq!(output.status, "ok");
        assert!(
            output.body.contains("source fixture 2105"),
            "{}",
            output.body
        );
        if options.contains("source") {
            assert!(!output.body.contains("file/attachment action"));
        }
    }

    for (page, category_id) in [("999999", ""), ("1", "999999999")] {
        let output = run_endpoint!(
            runner,
            wikidot_site_changes_module,
            json!({
                "site_id": site_id,
                "page_id": holder.page_id.to_string(),
                "page": page,
                "perpage": "20",
                "category_id": category_id,
                "options": "{\"all\":true}",
            }),
        );
        assert_eq!(output.status, "ok");
        assert_eq!(output.body, "Sorry, no revisions matching your criteria.");
    }

    let perpage_ten = run_endpoint!(
        runner,
        wikidot_site_changes_module,
        json!({
            "site_id": site_id,
            "page_id": holder.page_id.to_string(),
            "page": "1",
            "perpage": "10",
            "category_id": "",
            "options": "{\"all\":true}",
        }),
    );
    assert_eq!(perpage_ten.status, "ok");
    assert!(perpage_ten.body.contains("(rev. 2105)"));
    assert!(
        !perpage_ten.body.contains("(rev. 2085)"),
        "perpage 10 page one must hold only the first ten rows:\n{}",
        perpage_ten.body,
    );

    for invalid in [
        json!({"page_id": "0", "page": "1", "perpage": "20", "category_id": "", "options": "{\"all\":true}"}),
        json!({"page_id": holder.page_id.to_string(), "page": "0", "perpage": "20", "category_id": "", "options": "{\"all\":true}"}),
        json!({"page_id": holder.page_id.to_string(), "page": "1", "perpage": "20", "category_id": "missing", "options": "{\"all\":true}"}),
        json!({"page_id": holder.page_id.to_string(), "page": "1", "perpage": "20", "category_id": "", "options": "{\"all\":false}"}),
    ] {
        let mut input = invalid;
        input["site_id"] = json!(site_id);
        let output = run_endpoint!(runner, wikidot_site_changes_module, input);
        assert_eq!(output.status, "not_ok");
        assert!(output.body.is_empty());
    }

    run_endpoint_err!(
        runner,
        wikidot_site_changes_module,
        json!({
            "site_id": site_id,
            "page_id": holder.page_id.to_string(),
            "page": "1",
            "perpage": "20",
            "category_id": "",
            "options": "{\"all\":true}",
            "unknown": "value",
        }),
    );

    insert_source_revisions(&runner, private_revision_id, 5_000, 25, 10_000).await;
    let saturated_ajax = run_endpoint!(
        runner,
        wikidot_site_changes_module,
        json!({
            "site_id": site_id,
            "page_id": holder.page_id.to_string(),
            "page": "1",
            "perpage": "20",
            "category_id": "",
            "options": "{\"all\":true}",
        }),
    );
    assert_eq!(saturated_ajax.status, "not_ok");
    assert!(
        saturated_ajax.body.is_empty(),
        "raw-scan saturation must not expose partial rows:\n{}",
        saturated_ajax.body,
    );

    let saturated_snapshot = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "SiteChanges saturated anonymous preview",
            "wikitext": "[[module SiteChanges]]",
        }),
    );
    assert!(
        saturated_snapshot
            .body
            .contains(r#"[[module <em>SiteChanges</em>]] No such module"#,),
        "a saturated initial snapshot should follow the literal fail-closed path:\n{}",
        saturated_snapshot.body,
    );
    for forbidden in [PUBLIC_TITLE, PRIVATE_TITLE, r#"class="changes-list""#] {
        assert!(
            !saturated_snapshot.body.contains(forbidden),
            "a saturated initial snapshot must not expose partial row {forbidden:?}:\n{}",
            saturated_snapshot.body,
        );
    }
}

#[tokio::test]
async fn loginstatus_page_source_matches_live_unavailable_module() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let source = "[[module LoginStatus]]";

    for user_id in [None, Some(SAMPLE_USER_ID)] {
        runner.set_request_context(RequestContext {
            session: None,
            user_id,
            site_id: Some(site_id),
            page_reference: None,
        });
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": "LoginStatus page-source preview",
                "wikitext": source,
            }),
        );
        assert!(
            preview.body.contains(
                r#"<div class="error-block">[[module <em>LoginStatus</em>]] No such module, please <a href="http://www.wikidot.com/doc:modules" target="_blank">check available modules</a> and fix this page.</div>"#
            ),
            "LoginStatus in page source should match live Wikidot's unavailable-module error",
        );
    }
}

#[tokio::test]
async fn layout_only_modules_page_source_match_live_unavailable_module() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for module_name in [
        "NaviBar",
        "FooterBar",
        "PageOptionsBottom",
        "AdModuleAboveContent",
        "AdModuleBelowContent",
        "AdModuleAboveSidebar",
        "AdModuleBelowSidebar",
        "AdModuleBelowFooter",
    ] {
        runner.set_request_context(RequestContext {
            session: None,
            user_id: None,
            site_id: Some(site_id),
            page_reference: None,
        });
        let source = format!("[[module {module_name}]]");
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": "Layout-only module page-source preview",
                "wikitext": source,
            }),
        );
        let expected = format!(
            r#"<div class="error-block">[[module <em>{module_name}</em>]] No such module, please <a href="http://www.wikidot.com/doc:modules" target="_blank">check available modules</a> and fix this page.</div>"#,
        );
        assert!(
            preview.body.contains(&expected),
            "{module_name} in page source should match live Wikidot's unavailable-module error:\n{}",
            preview.body,
        );
    }
}

#[tokio::test]
async fn ad_module_page_source_matches_live_empty_output() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for source in [
        "AD_START\n[[module Ad]]\nAD_END",
        "AD_START\n[[module Ad label=\"custom_location\"]]\nAD_END",
        "AD_START\n[[module AD foo=\"bar\"]]\nAD_END",
    ] {
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
                "title": "Ad module page-source preview",
                "wikitext": source,
            }),
        );
        assert!(
            preview.body.contains("AD_START") && preview.body.contains("AD_END"),
            "Ad module should preserve surrounding content:\n{}",
            preview.body,
        );
        assert!(
            !preview.body.contains("[[module Ad")
                && !preview.body.contains("[[module AD")
                && !preview.body.contains("error-block"),
            "Ad module should render empty without unavailable-module markup:\n{}",
            preview.body,
        );
    }
}

#[tokio::test]
async fn adsenseunit_module_matches_live_deprecated_empty_output() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for source in [
        "ADSENSE_START\n[[module AdSenseUnit]]\nADSENSE_END",
        "ADSENSE_START\n[[module AdSenseUnit label=\"your ad\"]]\nADSENSE_END",
    ] {
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
                "title": "AdSenseUnit deprecated preview",
                "wikitext": source,
            }),
        );
        assert!(
            preview.body.contains("ADSENSE_START")
                && preview.body.contains("ADSENSE_END"),
            "AdSenseUnit should preserve surrounding content:\n{}",
            preview.body,
        );
        assert!(
            !preview.body.contains("[[module AdSenseUnit")
                && !preview.body.contains("error-block"),
            "Deprecated AdSenseUnit should render empty without unavailable-module markup:\n{}",
            preview.body,
        );
    }
}

#[tokio::test]
async fn syntax_links_match_frozen_live_semantics_with_documented_security_divergence() {
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
            "title": "Frozen live link syntax boundaries",
            "wikitext": concat!(
                "[[[some page| custom text]]]\n\n",
                "[*https://example.com/ Example]\n\n",
                "[# empty link]\n\n",
                "[support@example.com email me]\n\n",
                "[wikipedia:Albert_Einstein Albert]\n\n",
                "[#_editpage edit]\n\n",
                "[#_notawikidotcommand unknown]",
            ),
        }),
    );
    let html = preview.body;

    for expected in [
        r#"<a class="newpage" href="/some-page">custom text</a>"#,
        // Issue #1388 owns the Wikijump-only rel hardening; Wikidot layout
        // intentionally keeps this new-window anchor without rel.
        r#"<a href="https://example.com/" target="_blank">Example</a>"#,
        r#"<a href="javascript:;">empty link</a>"#,
        r#"<span class="wiki-email">moc.elpmaxe|troppus#em liame</span>"#,
        r#"<a href="http://en.wikipedia.org/wiki/Albert_Einstein" onclick="window.open(this.href, '_blank'); return false;">Albert</a>"#,
        r##"<a href="#_editpage">edit</a>"##,
        r##"<a href="#_notawikidotcommand">unknown</a>"##,
    ] {
        assert!(
            html.contains(expected),
            "public preview must retain the frozen anonymous Wikidot link boundary {expected:?}:\n{html}",
        );
    }

    assert!(!html.contains("support@example.com"));
    assert!(!html.contains("onclick=\"WIKIDOT.page"));
}

#[tokio::test]
async fn foldable_list_initial_dom_matches_frozen_live_page_preview() {
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
            "title": "Foldable list initial DOM",
            "wikitext": concat!(
                "[[div class=\"foldable-list-container\"]]\n",
                "* Main\n",
                " * Child\n",
                "  * Grandchild\n",
                "[[/div]]",
            ),
        }),
    );

    let html = preview.body;
    // Wikidot inserts serialization-only newlines between adjacent tags at
    // this boundary. Compare the DOM-bearing tag/text sequence rather than
    // requiring those non-visible whitespace text nodes from the raw AJAX
    // response.
    let normalized = html.replace(">\n<", "><");
    assert!(
        normalized.contains(concat!(
            "<div class=\"foldable-list-container\"><ul><li>Main\n",
            "<ul><li>Child\n",
            "<ul>",
            "<li>Grandchild</li></ul></li></ul></li></ul></div>",
        ),),
        "public preview must preserve the frozen foldable-list container and authored nested-list DOM:\n{html}",
    );
    assert!(!html.contains("foldable-list-toggle"));
}

#[tokio::test]
async fn social_syntax_matches_frozen_live_service_selection_and_widget_identity() {
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

    let cases: [(&str, &[&str]); 3] = [
        (
            "[[social]]",
            &[
                "BlinkList",
                "blogmarks",
                "del.icio.us",
                "digg",
                "Fark",
                "feedmelinks",
                "Furl",
                "LinkaGoGo",
                "NewsVine",
                "Netvouz",
                "Reddit",
                "YahooMyWeb",
                "Facebook",
            ],
        ),
        (
            "[[social digg,furl,del.icio.us,facebook]]",
            &["digg", "Furl", "del.icio.us", "Facebook"],
        ),
        ("[[social facebook,not-a-service]]", &["Facebook"]),
    ];

    for (source, expected_titles) in cases {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": "TITLE",
                "wikitext": source,
            }),
        );
        let html = preview.body;

        let id_prefix = r#"<span id="social"#;
        let id_start = html
            .find(id_prefix)
            .map(|index| index + id_prefix.len())
            .expect("social output should expose the live generated-id prefix");
        let id_suffix = &html[id_start..];
        let id_end = id_suffix
            .find('"')
            .expect("social generated id should terminate in the opening span");
        let generated_digits = &id_suffix[..id_end];
        assert!(
            !generated_digits.is_empty()
                && generated_digits
                    .chars()
                    .all(|character| character.is_ascii_digit()),
            "social id suffix must remain generated decimal digits: {html}",
        );
        let social_id = format!("social{generated_digits}");
        assert!(
            html.contains(&format!(r##"var socialspan = $j("#{social_id}")[0];"##)),
            "social runtime script must target the exact generated widget id: {html}",
        );
        assert!(
            html.contains(
                "http%3A%2F%2Fscp-wiki.wikidot.com%2Fajax-module-connector.php"
            ),
            "social service URLs must carry the current Wikidot site connector URL: {html}",
        );
        assert!(
            html.contains(
                "http://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--images/social/",
            ),
            "social icons must retain the frozen Wikidot resource prefix: {html}",
        );

        assert_eq!(
            html.matches(r#"style="margin: 0 2px""#).count(),
            expected_titles.len(),
            "social output must render exactly the selected supported services: {html}",
        );
        let mut cursor = 0;
        for title in expected_titles {
            let needle = format!(r#"title="{title}""#);
            let relative = html[cursor..]
                .find(&needle)
                .unwrap_or_else(|| panic!("missing social provider {title:?}: {html}"));
            cursor += relative + needle.len();
        }
        assert!(!html.contains("not-a-service"));
        assert!(!html.contains("[[social"));

        if source == "[[social]]" {
            for service_url in [
                "http://www.blinklist.com/index.php?Action=Blink/addblink.php",
                "http://blogmarks.net/my/new.php?mini=1",
                "http://del.icio.us/post?url=",
                "http://digg.com/submit?phase=2",
                "http://cgi.fark.com/cgi/fark/edit.pl?new_url=",
                "http://feedmelinks.com/categorize?from=toolbar",
                "http://www.furl.net/storeIt.jsp?u=",
                "http://www.linkagogo.com/go/AddNoPopup?url=",
                "http://www.newsvine.com/_tools/seed&amp;save?u=",
                "http://www.netvouz.com/action/submitBookmark?url=",
                "http://reddit.com/submit?url=",
                "http://myweb2.search.yahoo.com/myresults/bookmarklet?u=",
                "http://www.facebook.com/share.php?u=",
            ] {
                assert!(
                    html.contains(service_url),
                    "default social output must retain live service URL {service_url:?}: {html}",
                );
            }
            assert!(
                html.contains("new_comment=SCP+Foundation&amp;linktype=Misc"),
                "Fark output must carry the current site title with Wikidot plus encoding: {html}",
            );
            assert!(
                html.contains("encodeURIComponent(document.title)"),
                "social runtime script must preserve Wikidot title substitution: {html}",
            );
        }
    }
}

#[tokio::test]
async fn featuredsite_fails_closed_without_a_local_featured_site_authority() {
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
            "title": "FeaturedSite unavailable policy",
            "wikitext": "FEATURED_START\n[[module FeaturedSite]]\nFEATURED_END",
        }),
    );
    let html = preview.body;

    assert!(
        html.contains(
            r#"[[module <em>FeaturedSite</em>]] No such module, please <a href="https://www.wikidot.com/doc:modules" target="_blank">check available modules</a> and fix this page."#,
        ),
        "a module backed only by Wikidot's global rotation must use the established unavailable-module result:\n{html}",
    );
    assert!(
        !html.contains("target=\"_blank\" rel=\"noopener noreferrer\""),
        "Wikidot layout must not add Wikijump-only rel hardening to unavailable-module links:\n{html}",
    );
    for forbidden in [
        "featured-site-box",
        "thumbnails.wdfiles.com",
        "OZONE.dialog.hovertip",
        "scp-wiki.wikidot.com",
        "<script",
    ] {
        assert!(
            !html.contains(forbidden),
            "FeaturedSite fail-closed output must not fabricate a card or browser authority ({forbidden}):\n{html}",
        );
    }
}

#[tokio::test]
async fn static_account_modules_match_live_preview_and_page_view_basics() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let source = concat!(
        "ANONUN_START\n",
        "[[module AnonymousNotificationsUnsubscribe]]\n",
        "ANONUN_END\n",
        "USERINFO_START\n",
        "[[module UserInfo]]\n",
        "USERINFO_END\n",
        "SEARCHUSERS_START\n",
        "[[module SearchUsers]]\n",
        "SEARCHUSERS_END\n",
        "WATCHERS_START\n",
        "[[module Watchers]]\n",
        "WATCHERS_END\n",
        "MBE_START\n",
        "[[module MembershipEmailInvitation]]\n",
        "MBE_END\n",
        "WHO_START\n",
        "[[module WhoInvited]]\n",
        "WHO_END",
    );

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
            "title": "Static account modules live basics",
            "wikitext": source,
        }),
    );

    for (label, html) in [("preview", preview.body.as_str())] {
        assert!(
            html.contains(
                r#"<div class="error-block">Invalid indentification token.</div>"#
            ),
            "{label} should render AnonymousNotificationsUnsubscribe's live no-token error:\n{html}",
        );
        assert!(
            html.contains(r#"<div class="error-block">No user specified.</div>"#),
            "{label} should render UserInfo's live no-user error:\n{html}",
        );
        assert!(
            html.contains(
                r#"<div class="error-block">User search has been (temporarily) disabled. Sorry!</div>"#
            ),
            "{label} should render SearchUsers' live disabled notice:\n{html}",
        );
        assert!(
            html.contains("WATCHERS_START")
                && html.contains("<p>\n\n\n\n\n</p>")
                && html.contains("WATCHERS_END"),
            "{label} should preserve Wikidot's observed empty Watchers block boundary:\n{html}",
        );
        assert!(
            html.contains(r#"<div id="membership-email-invitation-box">"#)
                && html.contains("Sorry, the invitation could not be found.")
                && html.contains("aleady\n\t\t\tused by someone"),
            "{label} should render MembershipEmailInvitation's live missing invitation box:\n{html}",
        );
        assert!(
            html.contains(r#"<form action="dummy" id="who-invited-form" onsubmit="WIKIDOT.modules.WhoInvitedModule.listeners.lookUp(event)">"#)
                && html.contains(r#"<input type="text" id="user-lookup" size="30" class="autocomplete-input text"/>"#)
                && html.contains(r#"<div id="who-invited-results-box">"#),
            "{label} should render WhoInvited's live lookup form shell:\n{html}",
        );
        for module_name in [
            "AnonymousNotificationsUnsubscribe",
            "UserInfo",
            "SearchUsers",
            "Watchers",
            "MembershipEmailInvitation",
            "WhoInvited",
        ] {
            assert!(
                !html.contains(&format!("[[module {module_name}")),
                "{label} should consume {module_name} rather than leaking raw source:\n{html}",
            );
        }
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-static-account-modules",
        "Fixture Static Account Modules",
        source,
    )
    .await;

    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": "fixture-static-account-modules", "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let body = match view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found static account module view, got {other:?}"),
    };

    assert!(
        body.contains("ANONUN_START")
            && body.contains(
                r#"<div class="error-block">Invalid indentification token.</div>"#
            )
            && body.contains("ANONUN_END"),
        "saved page view should preserve AnonymousNotificationsUnsubscribe markers around the live error:\n{body}",
    );
    assert!(
        body.contains("USERINFO_START")
            && body.contains(r#"<div class="error-block">No user specified.</div>"#)
            && body.contains("USERINFO_END"),
        "saved page view should preserve UserInfo markers around the live error:\n{body}",
    );
    assert!(
        body.contains("SEARCHUSERS_START")
            && body.contains(
                r#"<div class="error-block">User search has been (temporarily) disabled. Sorry!</div>"#
            )
            && body.contains("SEARCHUSERS_END"),
        "saved page view should preserve SearchUsers markers around the live disabled notice:\n{body}",
    );
    assert!(
        body.contains("WATCHERS_START")
            && body.contains("<p>\n\n\n\n\n</p>")
            && body.contains("WATCHERS_END"),
        "saved page view should preserve Wikidot's observed empty Watchers block boundary:\n{body}",
    );
    assert!(
        body.contains("MBE_START")
            && body.contains(r#"<div id="membership-email-invitation-box">"#)
            && body.contains("Sorry, the invitation could not be found.")
            && body.contains("MBE_END"),
        "saved page view should preserve MembershipEmailInvitation markers around the live missing invitation box:\n{body}",
    );
    assert!(
        body.contains("WHO_START")
            && body.contains(r#"<form action="dummy" id="who-invited-form" onsubmit="WIKIDOT.modules.WhoInvitedModule.listeners.lookUp(event)">"#)
            && body.contains(r#"<div id="who-invited-results-box">"#)
            && body.contains("WHO_END"),
        "saved page view should preserve WhoInvited markers around the live form:\n{body}",
    );
    assert!(
        !body.contains("[[module AnonymousNotificationsUnsubscribe")
            && !body.contains("[[module UserInfo")
            && !body.contains("[[module SearchUsers")
            && !body.contains("[[module Watchers")
            && !body.contains("[[module MembershipEmailInvitation")
            && !body.contains("[[module WhoInvited"),
        "saved page view should consume all covered modules:\n{body}",
    );
}

#[tokio::test]
async fn search_and_feed_modules_match_live_preview_and_page_view_boundaries() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let search_error = r#"<div class="error-block">Search is temporarily unavailable, we are working to bring it online!</div>"#;
    let feed_missing = r#"<div class="error-block">No feed source specified ("src" element missing).</div>"#;
    let feed_unavailable = r#"<div class="error-block">Error processing the feed "https://example.com/feed.xml". The feed can not be accessed or contains errors. </div>"#;
    let cases = [
        ("search-bare", "[[module Search]]", search_error),
        (
            "search-mini-true",
            "[[module Search mini=\"true\"]]",
            search_error,
        ),
        (
            "search-area-pages",
            "[[module Search a=\"p\"]]",
            search_error,
        ),
        (
            "search-unknown-argument",
            "[[module Search unknown=\"x\"]]",
            search_error,
        ),
        (
            "search-single-quoted-mini",
            "[[module Search mini='true']]",
            search_error,
        ),
        ("search-uppercase-name", "[[module SEARCH]]", search_error),
        ("feed-bare", "[[module Feed]]", feed_missing),
        ("feed-empty-src", "[[module Feed src=\"\"]]", feed_missing),
        (
            "feed-missing-src-with-limit",
            "[[module Feed limit=\"1\"]]",
            feed_missing,
        ),
        (
            "feed-single-quoted-src",
            "[[module Feed src='https://example.com/feed.xml']]",
            feed_missing,
        ),
        (
            "feed-uppercase-src",
            "[[module Feed SRC=\"https://example.com/feed.xml\"]]",
            feed_missing,
        ),
        (
            "feed-valid-unavailable-src",
            "[[module Feed src=\"https://example.com/feed.xml\"]]",
            feed_unavailable,
        ),
    ];

    for (case_id, source, expected) in cases {
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
                "title": case_id,
                "wikitext": source,
            }),
        );
        assert!(
            preview.body.contains(expected),
            "{case_id} should render the frozen live boundary:\n{}",
            preview.body,
        );
        assert!(
            !preview.body.contains("[[module"),
            "{case_id} should consume the runtime module:\n{}",
            preview.body,
        );
    }

    let saved_source = concat!(
        "SEARCH_START\n[[module Search]]\nSEARCH_END\n",
        "FEED_START\n[[module Feed]]\nFEED_END",
    );
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-search-feed-boundary",
        "Fixture Search Feed Boundary",
        saved_source,
    )
    .await;
    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": "fixture-search-feed-boundary", "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let body = match view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found Search and Feed view, got {other:?}"),
    };
    assert!(
        body.contains("SEARCH_START")
            && body.contains(search_error)
            && body.contains("SEARCH_END")
            && body.contains("FEED_START")
            && body.contains(feed_missing)
            && body.contains("FEED_END")
            && !body.contains("[[module"),
        "saved page view should preserve both live module boundaries:\n{body}",
    );
}

#[tokio::test]
async fn searchall_module_matches_live_form_and_unavailable_route_contract() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let form_markers = [
        r#"<div class="search-box">"#,
        r#"<div class="query-area">"#,
        r#"<form action="dummy" id="search-form-all">"#,
        r#"<input class="text" type="text" size="30" name="query" id="search-form-all-input" value="" aria-label="Search all Wikis"/>"#,
        r#"<input class="button" type="submit" value="Search"/>"#,
        r#"<input id="search-all-pf" class="radio" type="radio" name="area" value="pf" checked="checked"/>"#,
        r#"<label for="search-all-pf">pages and forums</label>"#,
        r#"<input id="search-all-p" class="radio" type="radio" name="area" value="p"/>"#,
        r#"<label for="search-all-p">pages only</label>"#,
        r#"<input id="search-all-f" class="radio" type="radio" name="area" value="f"/>"#,
        r#"<label for="search-all-f">forums only</label>"#,
        r#"<div class="search-results">"#,
    ];

    for (case_id, source) in [
        ("searchall-bare", "[[module SearchAll]]"),
        ("searchall-uppercase-name", "[[module SEARCHALL]]"),
        (
            "searchall-unknown-argument",
            "[[module SearchAll unknown=\"x\"]]",
        ),
        (
            "searchall-single-quoted-argument",
            "[[module SearchAll unknown='x']]",
        ),
    ] {
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
                "title": case_id,
                "wikitext": source,
            }),
        );
        for marker in form_markers {
            assert!(
                preview.body.contains(marker),
                "{case_id} should render the frozen SearchAll form marker {marker}:\n{}",
                preview.body,
            );
        }
        assert!(
            !preview.body.contains("[[module"),
            "{case_id} should consume the SearchAll module:\n{}",
            preview.body,
        );
    }

    for (case_id, source, expected) in [
        (
            "searchall-inline",
            "before [[module SearchAll]] after",
            "before [[module SearchAll]] after",
        ),
        (
            "searchall-literal",
            "@@[[module SearchAll]]@@",
            "[[module SearchAll]]",
        ),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": source,
            }),
        );
        assert!(
            preview.body.contains(expected) && !preview.body.contains("No such module"),
            "{case_id} should preserve the live literal owner boundary:\n{}",
            preview.body,
        );
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-searchall-boundary",
        "Fixture SearchAll Boundary",
        "[[module SearchAll]]",
    )
    .await;
    let view = async |extra: &str| match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": "fixture-searchall-boundary", "extra": extra},
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found SearchAll page view for {extra}, got {other:?}"),
    };

    let bare = view("").await;
    assert!(
        bare.contains(r#"<form action="dummy" id="search-form-all">"#),
        "bare SearchAll view should render its form:\n{bare}",
    );
    let empty_query = view("/a/pf/q/").await;
    assert!(
        empty_query.contains(r#"<form action="dummy" id="search-form-all">"#),
        "empty SearchAll query should render its form:\n{empty_query}",
    );
    for (case_id, extra) in [
        ("searchall-route-pf-query", "/a/pf/q/wikidot"),
        ("searchall-route-p-query", "/a/p/q/wikidot"),
        ("searchall-route-f-query", "/a/f/q/wikidot"),
        ("searchall-route-unknown-area-query", "/a/x/q/wikidot"),
        ("searchall-route-query-without-area", "/q/wikidot"),
    ] {
        let queried = view(extra).await;
        assert!(
            queried.contains(
                r#"<div class="error-block">Couldnt connect to host, ElasticSearch down?</div>"#
            ) && !queried.contains("search-form-all"),
            "{case_id} should render the current live backend failure:\n{queried}",
        );
    }
}

#[tokio::test]
async fn japanese_site_localizes_module_input_accessible_names() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scpaiueouiuiuiui"}))
        .expect("local Japanese authoring site should exist");
    let site_id = site.site.site_id;

    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "Japanese module labels",
            "wikitext": "[[module NewPage]]\n[[module SearchAll]]",
        }),
    );

    assert!(
        preview.body.contains(r#"aria-label="新しいページ名""#),
        "NewPage should use the Japanese accessible name for a Japanese site:\n{}",
        preview.body,
    );
    assert!(
        preview.body.contains(r#"aria-label="すべてのWikiを検索""#),
        "SearchAll should use the Japanese accessible name for a Japanese site:\n{}",
        preview.body,
    );
}

#[tokio::test]
async fn currencyconvert_executes_only_on_the_www_plans_system_page() {
    const SOURCE: &str = concat!(
        "BEFORE\n",
        "[[module CurrencyConvert]]\n",
        "[[div class=\"plan-price\"]]\n",
        "$49.90\n",
        "[[/div]]\n",
        "[[/module]]\n",
        "AFTER",
    );

    let mut runner = TestRunner::setup().await;
    let www = run_endpoint!(runner, site_get, json!({"site": "www"}))
        .expect("seeded www system site should exist")
        .site;
    create_listpages_test_page(
        &mut runner,
        www.site_id,
        "plans",
        "Subscription plans",
        SOURCE,
    )
    .await;

    runner.set_request_context(RequestContext {
        site_id: Some(www.site_id),
        ..Default::default()
    });
    let plans = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": www.site_id,
            "session_token": null,
            "route": {"slug": "plans", "extra": ""},
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found www plans view, got {other:?}"),
    };
    assert!(
        plans.contains("BEFORE")
            && plans.contains(r#"<div class="plan-price">"#)
            && plans.contains("$49.90")
            && plans.contains("AFTER")
            && !plans.contains("CurrencyConvert")
            && !plans.contains("No such module"),
        "the www plans system page should consume CurrencyConvert and render its body:\n{plans}",
    );

    let ordinary = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded ordinary test site should exist")
        .site;
    runner.set_request_context(RequestContext {
        site_id: Some(ordinary.site_id),
        ..Default::default()
    });
    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": ordinary.site_id,
            "title": "CurrencyConvert ordinary page control",
            "wikitext": SOURCE,
        }),
    );
    assert!(
        preview.body.contains("No such module")
            && preview.body.contains("CurrencyConvert"),
        "ordinary pages must keep CurrencyConvert on Wikidot's unknown-module boundary:\n{}",
        preview.body,
    );
}

#[tokio::test]
async fn hardened_www_special_modules_execute_only_on_their_system_pages() {
    let mut runner = TestRunner::setup().await;
    let www = run_endpoint!(runner, site_get, json!({"site": "www"}))
        .expect("seeded www system site should exist")
        .site;
    for (slug, title, source) in [
        ("start:start", "Wikidot start", "[[module CreateAccount]]"),
        (
            "action:deleteaccount",
            "Delete account",
            "[[module DeleteAccount]]",
        ),
        (
            "inc:what-is-wikidot",
            "What is Wikidot",
            "[[module FrontSpecialMini]]",
        ),
        ("new-site", "New site", "[[module NewSite]]"),
        (
            "search",
            "Search sites",
            "[[module SitesTagCloud limit=\"200\"]]",
        ),
    ] {
        create_listpages_test_page(&mut runner, www.site_id, slug, title, source).await;
    }

    let anonymous_view = async |runner: &mut TestRunner, slug: &str| {
        runner.set_request_context(RequestContext {
            site_id: Some(www.site_id),
            ..Default::default()
        });
        match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": www.site_id,
                "session_token": null,
                "route": {"slug": slug, "extra": ""},
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                compiled_body_html, ..
            } => compiled_body_html,
            other => panic!("expected found www special view for {slug}, got {other:?}"),
        }
    };

    let create_account = anonymous_view(&mut runner, "start:start").await;
    assert!(
        create_account.contains("create-account-form")
            && create_account.contains("Create account")
            && create_account.contains("Please leave this checkbox blank")
            && create_account.contains("Terms of Service")
            && !create_account.contains("No such module"),
        "anonymous www start page must render CreateAccount",
    );
    let delete_account = anonymous_view(&mut runner, "action:deleteaccount").await;
    assert_eq!(
        delete_account.trim(),
        "<div class=\"error-block\">Invalid verification code. If you are terminating your account, please start again</div>",
    );
    let front_special = anonymous_view(&mut runner, "inc:what-is-wikidot").await;
    assert!(
        front_special.contains(r#"class="wikidot-front-special-stats""#)
            && front_special.contains("pages <span class=\"number\">")
            && front_special.contains("edits today <span class=\"number\">")
            && front_special.contains("people <span class=\"number\">")
            && front_special.contains("signed-up today <span class=\"number\">"),
        "FrontSpecialMini must render four volatile aggregate counters:\n{front_special}",
    );
    let new_site = anonymous_view(&mut runner, "new-site").await;
    assert!(
        new_site.contains(r#"id="new-site-box""#)
            && new_site.contains("We need you to have an account to create a new site")
            && new_site.contains("Sign in")
            && new_site.contains("Create account"),
        "anonymous NewSite must render the frozen sign-in/create-account state:\n{new_site}",
    );
    let tag_cloud = anonymous_view(&mut runner, "search").await;
    assert!(
        tag_cloud.contains(r#"class="sites-tag-cloud-box""#),
        "SitesTagCloud must own the www search system-page wrapper:\n{tag_cloud}",
    );

    let session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "www special module actor test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(www.site_id),
        ..Default::default()
    });
    let authenticated_start = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": www.site_id,
            "session_token": session_token.clone(),
            "route": {"slug": "start:start", "extra": ""},
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected authenticated www start view, got {other:?}"),
    };
    assert!(
        authenticated_start.contains("Hello,")
            && authenticated_start.contains("My Account")
            && !authenticated_start.contains("createaccount-form"),
        "authenticated CreateAccount must render Wikidot's account greeting state:\n{authenticated_start}",
    );
    let authenticated_new_site = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": www.site_id,
            "session_token": session_token,
            "route": {"slug": "new-site", "extra": ""},
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected authenticated www new-site view, got {other:?}"),
    };
    assert!(
        authenticated_new_site.contains(r#"id="new-site-form""#)
            && authenticated_new_site.contains("Title")
            && authenticated_new_site.contains("Web address")
            && authenticated_new_site.contains("Language")
            && authenticated_new_site.contains("privacy"),
        "authenticated NewSite must render the site-creation form state:\n{authenticated_new_site}",
    );

    let ordinary = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded ordinary test site should exist")
        .site;
    runner.set_request_context(RequestContext {
        site_id: Some(ordinary.site_id),
        ..Default::default()
    });
    for name in [
        "CreateAccount",
        "DeleteAccount",
        "FrontSpecialMini",
        "NewSite",
        "SitesTagCloud",
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": ordinary.site_id,
                "title": format!("{name} ordinary control"),
                "wikitext": format!("[[module {name}]]"),
            }),
        );
        assert!(
            preview.body.contains("No such module") && preview.body.contains(name),
            "ordinary pages must retain Wikidot's unknown-module boundary for {name}:\n{}",
            preview.body,
        );
    }
}

#[tokio::test]
async fn simpletodo_and_sendinvitations_modules_match_live_preview_basics() {
    const ACTIVE_CONTENT_MARKERS: [&str; 6] = [
        "<script",
        "http://www.wikidot.com/common--javascript/yahooui/animation-min.js",
        "javascript:",
        " onclick=",
        " onload=",
        " onerror=",
    ];

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let source = concat!(
        "SIMPLE_MISSING\n",
        "[[module SimpleToDo]]\n",
        "SIMPLE_EMPTY\n",
        "[[module SimpleToDo id=\"\"]]\n",
        "SIMPLE_VALID\n",
        "[[module SimpleToDo id=\"codex-live-probe\"]]\n",
        "INVITE\n",
        "[[module SendInvitations]]\n",
        "END",
    );

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
            "title": "SimpleToDo and SendInvitations live basics",
            "wikitext": source,
        }),
    );

    assert!(
        preview
            .body
            .matches("The SimpleTodo module must have an id.")
            .count()
            == 2,
        "SimpleToDo omitted and empty id should render the live error:\n{}",
        preview.body,
    );
    assert!(
        preview.body.contains(
            r#"<div class="simpletodo-box" id="simpletodo_0"><div class="title">Here is a place for your title</div>"#
        ) && preview
            .body
            .contains(r#"<div class="label">codex-live-probe</div>"#)
            && preview
                .body
                .contains(r#"<span id="simpletodo-data-edit-permission">false</span>"#),
        "SimpleToDo with id should render the live initial read-only list shell:\n{}",
        preview.body,
    );
    for forbidden in ACTIVE_CONTENT_MARKERS {
        assert!(
            !preview.body.contains(forbidden),
            "SimpleToDo must not restore active page-origin content {forbidden:?}:\n{}",
            preview.body,
        );
    }
    assert!(
        preview.body.contains(
            r#"<div class="error-block">Inviting users has been disabled due to severe abuse. Admins can still send email invitations via <a href="/_admin">site admin dashboard</a>.</div>"#
        ),
        "SendInvitations should render Wikidot's disabled-invitations error:\n{}",
        preview.body,
    );
    assert!(
        !preview.body.contains("[[module SimpleToDo")
            && !preview.body.contains("[[module SendInvitations"),
        "implemented modules should not leak raw module source:\n{}",
        preview.body,
    );

    let saved_slug = "fixture-simpletodo-saved-security";
    create_listpages_test_page(
        &mut runner,
        site_id,
        saved_slug,
        "Fixture SimpleToDo Saved Security",
        r#"[[module SimpleToDo id="<script>saved</script>"]]"#,
    )
    .await;
    let saved = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": saved_slug,
            "details": {"compiled": true},
        }),
    )
    .expect("saved SimpleToDo security fixture should exist");
    let saved_html = saved
        .compiled_body_html
        .expect("saved SimpleToDo fixture should include compiled HTML");
    assert!(
        saved_html
            .contains(r#"<div class="label">&lt;script&gt;saved&lt;/script&gt;</div>"#)
            && saved_html.matches(r#"aria-disabled="true""#).count() == 2
            && saved_html
                .contains(r#"<span id="simpletodo-data-edit-permission">false</span>"#),
        "saved SimpleToDo should preserve only the escaped read-only shell:\n{saved_html}",
    );
    for forbidden in ACTIVE_CONTENT_MARKERS {
        assert!(
            !saved_html.contains(forbidden),
            "saved SimpleToDo must not restore active page-origin content {forbidden:?}:\n{saved_html}",
        );
    }
}

#[tokio::test]
async fn mailform_without_a_trusted_action_contract_fails_closed_in_preview_and_saved_views()
 {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let source = concat!(
        "BEFORE\n",
        "[[module MailForm to=\"dummy\" button=\"Send\"]]\n",
        "* first-field\n",
        " * title: First field\n",
        " * default: harmless\n",
        " * type: text\n",
        "* second-field\n",
        " * title: Second field\n",
        " * type: textarea\n",
        "[[/module]]\n",
        "AFTER",
    );

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
            "title": "MailForm fail-closed preview",
            "wikitext": source,
        }),
    );
    assert_mailform_is_literal_and_inert(&preview.body);

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-mailform-fail-closed",
        "Fixture MailForm Fail Closed",
        source,
    )
    .await;
    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: None,
    });
    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": "fixture-mailform-fail-closed", "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let body = match view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found MailForm view, got {other:?}"),
    };
    assert_mailform_is_literal_and_inert(&body);
}

fn assert_mailform_is_literal_and_inert(body: &str) {
    assert!(
        body.contains("BEFORE")
            && body.contains("<em>MailForm</em>")
            && body.contains("No such module")
            && body.contains("AFTER"),
        "MailForm without a trusted action contract must fail closed between authored boundaries:\n{body}",
    );
    for forbidden in [
        r#"<div class="mailform-box""#,
        "<form",
        "javascript:",
        "mailformdef-",
        "MailFormModule.listeners",
    ] {
        assert!(
            !body.contains(forbidden),
            "fail-closed MailForm output must not expose active control {forbidden:?}:\n{body}",
        );
    }
}

#[tokio::test]
async fn anonymous_page_and_site_utility_modules_match_frozen_safe_states() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let cases = [
        (
            "module-own-line-redirect",
            "[[module Redirect]]",
            r#"<div class="error-block">No redirection destination specified. Please use the destination="page-name" or destination="url" attribute.</div>"#,
        ),
        (
            "module-own-line-themepreviewer",
            "[[module ThemePreviewer]]",
            r#"<div class="error-block">Preview mode error: please contact Wikidot.com for a better error message</div>"#,
        ),
        (
            "module-own-line-managesite",
            "[[module ManageSite]]",
            concat!(
                r#"<div class="row-fluid">"#,
                "\n\t",
                r#"<div class="span3 offset1">"#,
                "\n\t\t",
                r#"<div class="homer">"#,
                "\n\t\t",
                r#"<img src="/common--images/404_homer.png">"#,
                "\n\t\t</div>\n\t</div>\n\t",
                r#"<div class="span7">"#,
                "\n\t\t<h1>Doh!</h1>\n",
                "\t\t<h3>You're not signed in or you are not an administrator of this Wiki.</h3>\n",
                "\t\t\t\t",
                r#"<div class="form-actions">"#,
                "\n\t\t\t",
                r#"<a href="javascript:;" class="btn btn-primary btn-large" onclick="WIKIDOT.page.listeners.loginClick(event)">Sign in</a>"#,
                "\n\t\t</div>\n\t\t\t</div>\n</div>",
            ),
        ),
        (
            "module-own-line-clone",
            "[[module Clone]]",
            r#"<div class="error-block">You should be logged in to clone a site.</div>"#,
        ),
        (
            "module-own-line-dashboard",
            "[[module Dashboard]]",
            r#"<div class="error-block">Not allowed. Error.</div>"#,
        ),
        (
            "module-own-line-petitionadmin",
            "[[module PetitionAdmin]]",
            r#"<div class="error-block"><div class="title">Permission error</div>This tool is for use by the administrators of this site</div>"#,
        ),
        (
            "module-own-line-sitegrid",
            "[[module SiteGrid]]",
            r#"<div class="error-block">No sites provided.</div>"#,
        ),
    ];

    for (case_id, module_source, expected) in cases {
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
                "title": case_id,
                "wikitext": module_source,
            }),
        );
        assert_eq!(
            preview.body.trim(),
            expected,
            "{case_id} should match the frozen anonymous PagePreview output",
        );
        assert!(
            !preview.body.contains(module_source)
                && !preview.body.contains("No such module, please"),
            "{case_id} should consume the evidenced module without the generic fallback:\n{}",
            preview.body,
        );
    }

    let source = concat!(
        "REDIRECT\n",
        "[[module Redirect]]\n",
        "THEME\n",
        "[[module ThemePreviewer]]\n",
        "MANAGE\n",
        "[[module ManageSite]]\n",
        "CLONE\n",
        "[[module Clone]]\n",
        "DASHBOARD\n",
        "[[module Dashboard]]\n",
        "PETITION\n",
        "[[module PetitionAdmin]]\n",
        "SITEGRID\n",
        "[[module SiteGrid]]\n",
        "END",
    );

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-anonymous-site-utility-modules",
        "Fixture Anonymous Site Utility Modules",
        source,
    )
    .await;
    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: None,
    });
    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": "fixture-anonymous-site-utility-modules", "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let body = match view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found site utility module view, got {other:?}"),
    };
    for expected in [
        "You're not signed in or you are not an administrator of this Wiki.",
        "You should be logged in to clone a site.",
        "This tool is for use by the administrators of this site",
        "No sites provided.",
    ] {
        assert!(
            body.contains(expected),
            "saved anonymous utility view should contain {expected:?}:\n{body}",
        );
    }
    assert!(
        !body.contains("No such module, please")
            && !body.contains("data-wikijump-compat-clone"),
        "saved anonymous utility view must not reuse the authoring actor's compiled branch:\n{body}",
    );
}

#[tokio::test]
async fn authenticated_site_utility_preview_renders_admin_manage_and_petition_read_models()
 {
    const MANAGE_SITE_SOURCE: &str = "[[module ManageSite]]";
    const MANAGE_SITE_BODY_SHA256: &str =
        "7daaec8dc6eca01b2bec08d9959dff6d0ed562c404cd20e1d5b156bc26da0b57";
    const PETITION_ADMIN_SOURCE: &str = "[[module PetitionAdmin]]";
    const PETITION_ADMIN_BODY_SHA256: &str =
        "b80d4331c4edc47c4129f08701a14e031c07f7f302b1384614eedb38ced2124c";

    let body_sha256 = |body: &str| hex::encode(Sha256::digest(body.trim().as_bytes()));
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    SiteService::update(
        runner.context(),
        Reference::Id(site_id),
        UpdateSiteBody {
            name: Maybe::Set(
                r#"<script data-site-text="unsafe">probe</script>"#.to_owned(),
            ),
            ..Default::default()
        },
        None,
        ADMIN_USER_ID,
        common::IP_ADDRESS,
    )
    .await
    .expect("hostile site text fixture should be stored");

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        ..Default::default()
    });
    let manage_preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "ManageSite administrator read model",
            "wikitext": MANAGE_SITE_SOURCE,
        }),
    );
    let petition_preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "PetitionAdmin administrator read model",
            "wikitext": PETITION_ADMIN_SOURCE,
        }),
    );
    assert_eq!(
        body_sha256(&manage_preview.body),
        MANAGE_SITE_BODY_SHA256,
        "administrator ManageSite preview must match the captured initial manager shell exactly",
    );
    assert_eq!(
        body_sha256(&petition_preview.body),
        PETITION_ADMIN_BODY_SHA256,
        "administrator PetitionAdmin preview must match the captured no-campaign body exactly",
    );
    assert!(
        !manage_preview.body.contains("data-site-text")
            && !petition_preview.body.contains("data-site-text"),
        "captured initial read models must not interpolate dynamic site text",
    );

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-admin-manage-site-read-model",
        "Fixture Admin ManageSite Read Model",
        MANAGE_SITE_SOURCE,
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-admin-petition-read-model",
        "Fixture Admin Petition Read Model",
        PETITION_ADMIN_SOURCE,
    )
    .await;
    let administrator_session = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "site utility administrator read model".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("administrator session should be created");
    let body = |view| match view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected saved administrator site utility page, got {other:?}"),
    };
    let saved_manage = body(run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": administrator_session.clone(),
            "route": {"slug": "fixture-admin-manage-site-read-model", "extra": ""},
            "locales": ["en-US", "en"],
        }),
    ));
    let saved_petition = body(run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": administrator_session,
            "route": {"slug": "fixture-admin-petition-read-model", "extra": ""},
            "locales": ["en-US", "en"],
        }),
    ));
    assert_eq!(
        body_sha256(&saved_manage),
        MANAGE_SITE_BODY_SHA256,
        "saved administrator ManageSite must re-render the captured initial manager shell",
    );
    assert_eq!(
        body_sha256(&saved_petition),
        PETITION_ADMIN_BODY_SHA256,
        "saved administrator PetitionAdmin must re-render the captured no-campaign body",
    );
}

#[tokio::test]
async fn authenticated_site_utility_preview_denies_only_non_administrators() {
    const MANAGE_SITE_SOURCE: &str = "[[module ManageSite]]";
    const PETITION_ADMIN_SOURCE: &str = "[[module PetitionAdmin]]";
    const MANAGE_SITE_ANONYMOUS_HTML: &str = concat!(
        r#"<div class="row-fluid">"#,
        "\n\t",
        r#"<div class="span3 offset1">"#,
        "\n\t\t",
        r#"<div class="homer">"#,
        "\n\t\t",
        r#"<img src="/common--images/404_homer.png">"#,
        "\n\t\t</div>\n\t</div>\n\t",
        r#"<div class="span7">"#,
        "\n\t\t<h1>Doh!</h1>\n",
        "\t\t<h3>You're not signed in or you are not an administrator of this Wiki.</h3>\n",
        "\t\t\t\t",
        r#"<div class="form-actions">"#,
        "\n\t\t\t",
        r#"<a href="javascript:;" class="btn btn-primary btn-large" onclick="WIKIDOT.page.listeners.loginClick(event)">Sign in</a>"#,
        "\n\t\t</div>\n\t\t\t</div>\n</div>",
    );
    const MANAGE_SITE_NON_ADMIN_HTML: &str = concat!(
        r#"<div class="row-fluid">"#,
        "\n\t",
        r#"<div class="span3 offset1">"#,
        "\n\t\t",
        r#"<div class="homer">"#,
        "\n\t\t",
        r#"<img src="/common--images/404_homer.png">"#,
        "\n\t\t</div>\n\t</div>\n\t",
        r#"<div class="span7">"#,
        "\n\t\t<h1>Doh!</h1>\n",
        "\t\t<h3>You're not signed in or you are not an administrator of this Wiki.</h3>\n",
        "\t\t\t</div>\n</div>",
    );
    const PETITION_ADMIN_DENIAL_HTML: &str = r#"<div class="error-block"><div class="title">Permission error</div>This tool is for use by the administrators of this site</div>"#;

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        ..Default::default()
    });
    let anonymous_manage_site = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "ManageSite anonymous actor boundary",
            "wikitext": MANAGE_SITE_SOURCE,
        }),
    );
    let anonymous_petition_admin = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "PetitionAdmin anonymous actor boundary",
            "wikitext": PETITION_ADMIN_SOURCE,
        }),
    );
    assert_eq!(
        anonymous_manage_site.body.trim(),
        MANAGE_SITE_ANONYMOUS_HTML,
        "anonymous ManageSite must retain its login-capable live DOM",
    );
    assert_eq!(
        anonymous_petition_admin.body.trim(),
        PETITION_ADMIN_DENIAL_HTML,
        "anonymous PetitionAdmin must retain its existing permission error",
    );

    runner.set_request_context(RequestContext {
        user_id: Some(SAMPLE_USER_ID),
        site_id: Some(site_id),
        ..Default::default()
    });
    let non_admin_manage_site = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "ManageSite authenticated non-admin boundary",
            "wikitext": MANAGE_SITE_SOURCE,
        }),
    );
    let non_admin_petition_admin = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "PetitionAdmin authenticated non-admin boundary",
            "wikitext": PETITION_ADMIN_SOURCE,
        }),
    );
    assert_eq!(
        non_admin_manage_site.body.trim(),
        MANAGE_SITE_NON_ADMIN_HTML,
        "an authenticated non-admin must receive the live ManageSite denial without a sign-in action",
    );
    assert_eq!(
        non_admin_petition_admin.body.trim(),
        PETITION_ADMIN_DENIAL_HTML,
        "an authenticated non-admin must receive the live PetitionAdmin denial",
    );

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        ..Default::default()
    });
    let admin_manage_site = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "ManageSite administrator residual",
            "wikitext": MANAGE_SITE_SOURCE,
        }),
    );
    let admin_petition_admin = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "PetitionAdmin administrator residual",
            "wikitext": PETITION_ADMIN_SOURCE,
        }),
    );
    assert!(
        admin_manage_site.body.contains("id=\"site-manager-menu\"")
            && admin_manage_site.body.contains("id=\"sm-action-area\"")
            && !admin_manage_site.body.contains(MANAGE_SITE_NON_ADMIN_HTML),
        "administrator ManageSite must render the manager shell without a denial:\n{}",
        admin_manage_site.body,
    );
    assert!(
        admin_petition_admin
            .body
            .contains("id=\"petition-admin-module-box\"")
            && admin_petition_admin
                .body
                .contains("You have no petition campaigns defined.")
            && !admin_petition_admin
                .body
                .contains(PETITION_ADMIN_DENIAL_HTML),
        "administrator PetitionAdmin must render the no-campaign body without a denial:\n{}",
        admin_petition_admin.body,
    );
}

#[tokio::test]
async fn saved_site_utility_modules_recheck_the_session_actor() {
    const HOLDER: &str = "fixture-site-utility-actor-boundary";
    const SOURCE: &str = concat!(
        "MANAGE_START\n",
        "[[module ManageSite]]\n",
        "MANAGE_END\n",
        "PETITION_START\n",
        "[[module PetitionAdmin]]\n",
        "PETITION_END",
    );
    const MANAGE_SITE_NON_ADMIN_HTML: &str = concat!(
        r#"<div class="row-fluid">"#,
        "\n\t",
        r#"<div class="span3 offset1">"#,
        "\n\t\t",
        r#"<div class="homer">"#,
        "\n\t\t",
        r#"<img src="/common--images/404_homer.png">"#,
        "\n\t\t</div>\n\t</div>\n\t",
        r#"<div class="span7">"#,
        "\n\t\t<h1>Doh!</h1>\n",
        "\t\t<h3>You're not signed in or you are not an administrator of this Wiki.</h3>\n",
        "\t\t\t</div>\n</div>",
    );
    const PETITION_ADMIN_DENIAL_HTML: &str = r#"<div class="error-block"><div class="title">Permission error</div>This tool is for use by the administrators of this site</div>"#;

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER,
        "Fixture Site Utility Actor Boundary",
        SOURCE,
    )
    .await;

    let non_admin_session = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: SAMPLE_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "site utility non-admin boundary".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("non-admin session should be created");
    let administrator_session = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "site utility administrator residual".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("administrator session should be created");

    let body = |view| match view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found site utility page, got {other:?}"),
    };
    let non_admin = body(run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": non_admin_session,
            "route": {"slug": HOLDER, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    ));
    assert!(
        non_admin.contains(MANAGE_SITE_NON_ADMIN_HTML)
            && non_admin.contains(PETITION_ADMIN_DENIAL_HTML)
            && !non_admin.contains(">Sign in</a>"),
        "saved non-admin view must use the two exact authenticated denial states:\n{non_admin}",
    );

    let administrator = body(run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": administrator_session,
            "route": {"slug": HOLDER, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    ));
    assert!(
        administrator.contains("id=\"site-manager-menu\"")
            && administrator.contains("id=\"sm-action-area\"")
            && administrator.contains("id=\"petition-admin-module-box\"")
            && administrator.contains("You have no petition campaigns defined.")
            && !administrator.contains(MANAGE_SITE_NON_ADMIN_HTML)
            && !administrator.contains(PETITION_ADMIN_DENIAL_HTML),
        "saved administrator view must recheck authority and render both read models:\n{administrator}",
    );
}

#[tokio::test]
async fn typed_sitegrid_empty_body_matches_frozen_preview_and_saved_state() {
    const EMPTY_HTML: &str = r#"<div class="error-block">No sites provided.</div>"#;
    const SAVED_SOURCE: &str = "[[module SiteGrid]]\n \t\n[[/module]]";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (case_id, source) in [
        ("typed-sitegrid-empty", "[[module SiteGrid]]\n[[/module]]"),
        ("typed-sitegrid-whitespace", SAVED_SOURCE),
    ] {
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
                "title": case_id,
                "wikitext": source,
            }),
        );

        assert_eq!(
            preview.body.trim(),
            EMPTY_HTML,
            "{case_id} should resolve the typed empty SiteGrid node",
        );
        assert!(
            !preview.body.contains("No such module, please"),
            "{case_id} should not reach FTML's generic runtime-module fallback:\n{}",
            preview.body,
        );
    }

    let saved_slug = "fixture-typed-sitegrid-empty";
    create_listpages_test_page(
        &mut runner,
        site_id,
        saved_slug,
        "Fixture Typed SiteGrid Empty",
        SAVED_SOURCE,
    )
    .await;
    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: None,
    });
    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": saved_slug, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let body = match view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected saved typed SiteGrid view, got {other:?}"),
    };
    assert_eq!(
        body.trim(),
        EMPTY_HTML,
        "saved pages should resolve the same typed empty SiteGrid node",
    );
}

#[tokio::test]
async fn site_utility_modules_preserve_literal_and_reject_unsupported_shapes() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let source = concat!(
        "LITERAL\n",
        "@@[[module Clone]][[module ManageSite]]@@\n",
        "BODY\n",
        "[[module SiteGrid]]\n",
        "private-site-name\n",
        "[[/module]]\n",
        "ARGUMENTED_EMPTY_BODY\n",
        "[[module SiteGrid limit=\"0\"]]\n",
        "[[/module]]\n",
        "CASE_VARIANT\n",
        "[[module sitegrid]]\n",
        "[[/module]]\n",
        "FOREIGN_RUNTIME\n",
        "[[module RuntimeOwnershipProbe]]\n",
        "[[/module]]\n",
        "MANAGE_BODY\n",
        "[[module ManageSite]]\n",
        "unsupported\n",
        "[[/module]]\n",
        "PETITION_BODY\n",
        "[[module PetitionAdmin]]\n",
        "unsupported\n",
        "[[/module]]\n",
        "UNKNOWN_ARGUMENTS\n",
        "[[module ManageSite unexpected=\"x\"]]\n",
        "[[module PetitionAdmin unexpected=\"x\"]]\n",
        "DUPLICATE_ARGUMENTS\n",
        "[[module ManageSite unexpected=\"x\" unexpected=\"y\"]]\n",
        "[[module PetitionAdmin unexpected=\"x\" unexpected=\"y\"]]\n",
        "END",
    );

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
            "title": "Site utility ownership boundaries",
            "wikitext": source,
        }),
    );

    assert!(
        preview.body.contains("[[module Clone]]")
            && preview.body.contains("[[module ManageSite]]"),
        "literal-owned utility syntax must remain visible text:\n{}",
        preview.body,
    );
    assert!(
        !preview
            .body
            .contains("You should be logged in to clone a site.")
            && !preview.body.contains("No sites provided.")
            && !preview.body.contains("404_homer.png"),
        "literal, body-bearing, and unsupported-argument shapes must not enter the evidenced consumers:\n{}",
        preview.body,
    );
    assert!(
        preview.body.contains("No such module, please"),
        "unsupported utility arguments should retain the established fail-closed output:\n{}",
        preview.body,
    );
    for unconsumed_name in ["SiteGrid", "sitegrid", "RuntimeOwnershipProbe"] {
        assert!(
            preview
                .body
                .contains(&format!("<em>{unconsumed_name}</em>")),
            "unsupported typed runtime shape {unconsumed_name:?} should retain FTML's generic fail-closed output:\n{}",
            preview.body,
        );
    }
}

#[tokio::test]
async fn wikidot_page_preview_keeps_html_blocks_literal_while_saved_pages_execute_them() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let slug = "fixture-preview-html-lifecycle";
    let source = concat!(
        "[[html]]\n<b>X</b>\n[[/html]]\n",
        "[[html]]\n<i>Y</i>\n[[/html]]",
    );

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
            "title": "Preview HTML lifecycle",
            "wikitext": source,
        }),
    );
    assert_eq!(
        preview.body,
        concat!(
            "<p>[[html]]<br>\n&lt;b&gt;X&lt;/b&gt;<br>\n[[/html]]<br>\n",
            "[[html]]<br>\n&lt;i&gt;Y&lt;/i&gt;<br>\n[[/html]]</p>",
        ),
    );
    assert!(!preview.body.contains("<iframe"), "{}", preview.body);

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(slug)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": source,
            "title": "Fixture Preview HTML Lifecycle",
            "alt_title": null,
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "create preview HTML lifecycle fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
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
    let GetPageViewOutput::Found {
        compiled_body_html, ..
    } = view
    else {
        panic!("expected saved HTML lifecycle page, got {view:?}");
    };
    assert_eq!(
        compiled_body_html,
        concat!(
            r#"<p><iframe src="/fixture-preview-html-lifecycle/html/1" allowtransparency="true" frameborder="0" class="html-block-iframe"></iframe>"#,
            r#"</p><p><iframe src="/fixture-preview-html-lifecycle/html/2" allowtransparency="true" frameborder="0" class="html-block-iframe"></iframe></p>"#,
        ),
        "saved HTML blocks must use the numeric ordinals resolved by WWS: {compiled_body_html}",
    );
}

#[tokio::test]
async fn listpages_generated_html_preview_saved_section_controls() {
    const TARGET_SLUG: &str = "fixture-listpages-section-zero-html-target";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "ListPages Section Zero HTML Target",
        "SECTION_ONE",
    )
    .await;
    for (case, section, opener, marker, executes_when_saved) in [
        ("section-zero", 0, "html", "SECTION_ZERO_HTML", true),
        (
            "section-out-of-range",
            999,
            "html",
            "SECTION_OUT_OF_RANGE_HTML",
            true,
        ),
        ("section-one", 1, "html", "SECTION_ONE_HTML", false),
        ("invalid-opener", 0, "htmlx", "INVALID_OPENER_HTML", false),
    ] {
        let consumer_slug = format!("fixture-listpages-generated-html-{case}");
        let source = format!(
            concat!(
                "[[module ListPages name=\"{}\" separate=\"no\" wrapper=\"no\"]]\n",
                "ROW_EVALUATED:%%title%%\n",
                "[[%%content{{{}}}%%{}]]\n",
                "<b>{}</b>\n",
                "[[/html]]\n",
                "[[/module]]",
            ),
            TARGET_SLUG, section, opener, marker,
        );

        runner.set_request_context(RequestContext {
            site_id: Some(site_id),
            ..Default::default()
        });
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": format!("ListPages generated HTML {case} preview"),
                "wikitext": &source,
            }),
        );
        assert!(
            preview
                .body
                .contains(&format!("&lt;b&gt;{marker}&lt;/b&gt;"))
                && preview
                    .body
                    .contains("ROW_EVALUATED:ListPages Section Zero HTML Target")
                && !preview.body.contains("%%content{")
                && !preview.body.contains("%%title%%")
                && !preview.body.contains("[[module ListPages")
                && !preview.body.contains("html-block-iframe"),
            "{case} PagePreview must evaluate the row while keeping its generated HTML shape literal:\n{}",
            preview.body,
        );
        if executes_when_saved {
            assert!(
                preview.body.contains("[[html]]") && preview.body.contains("[[/html]]"),
                "{case}: {}",
                preview.body
            );
        } else if section == 1 {
            assert!(
                preview.body.contains("[[SECTION_ONEhtml]]")
                    && preview.body.contains("[[/html]]"),
                "{case} must preserve the nonempty-section opener and literal closer: {}",
                preview.body,
            );
        } else {
            assert!(
                preview.body.contains("[[htmlx]]") && preview.body.contains("[[/html]]"),
                "{case} must preserve the invalid opener and literal closer: {}",
                preview.body,
            );
        }

        create_listpages_test_page(
            &mut runner,
            site_id,
            &consumer_slug,
            &format!("ListPages Generated HTML {case}"),
            &source,
        )
        .await;
        runner.set_request_context(RequestContext {
            site_id: Some(site_id),
            ..Default::default()
        });
        let view = run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": null,
                "route": {"slug": &consumer_slug, "extra": ""},
                "locales": ["en-US", "en"],
            }),
        );
        let GetPageViewOutput::Found {
            compiled_body_html, ..
        } = view
        else {
            panic!("expected saved generated HTML consumer, got {view:?}");
        };
        assert_eq!(
            compiled_body_html.contains("html-block-iframe"),
            executes_when_saved,
            "{case} saved view used the wrong HTML execution state:\n{compiled_body_html}",
        );
        if executes_when_saved {
            assert!(
                compiled_body_html.contains(&format!(r#"src="/{consumer_slug}/html/1""#))
                    && !compiled_body_html.contains(marker),
                "{case} saved HTML must use its hosted iframe:\n{compiled_body_html}",
            );
        } else {
            assert!(
                compiled_body_html.contains(&format!("&lt;b&gt;{marker}&lt;/b&gt;"))
                    && compiled_body_html
                        .contains("ROW_EVALUATED:ListPages Section Zero HTML Target"),
                "{case} saved non-HTML output must retain the evaluated row and escaped payload:\n{compiled_body_html}",
            );
        }
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

/// Live capture (sandbox-for-codex, 2026-09-14): the Rate widget control
/// sequence is identical for anonymous, non-member, member, moderator, and
/// administrator readers and for existing voters in every observed rating
/// mode: plus-minus (`rateup`/`ratedown`/`cancel`), stars (`data-rating` with
/// no cancel), and disabled plus-only (`rateup`/`cancel` without `ratedown`).
#[tokio::test]
async fn page_render_rate_widget_matches_actor_and_rating_mode_matrix() {
    const PLUS_MINUS_CATEGORY: &str = "fixture-rate-matrix-plus-minus";
    const PLUS_CATEGORY: &str = "fixture-rate-matrix-plus";
    const STARS_CATEGORY: &str = "fixture-rate-matrix-stars";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    set_listpages_test_category_rating_type(&runner, site_id, PLUS_CATEGORY, "plus")
        .await;
    set_listpages_test_category_rating_type(&runner, site_id, STARS_CATEGORY, "stars")
        .await;

    let plus_minus_slug = format!("{PLUS_MINUS_CATEGORY}:holder");
    let plus_slug = format!("{PLUS_CATEGORY}:holder");
    let stars_slug = format!("{STARS_CATEGORY}:holder");
    create_listpages_test_page(
        &mut runner,
        site_id,
        &plus_minus_slug,
        "Rate matrix plus-minus",
        "Plus-minus holder.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        &plus_slug,
        "Rate matrix plus",
        "Plus holder.",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        &stars_slug,
        "Rate matrix stars",
        "Stars holder.",
    )
    .await;
    let plus_minus_id = listpages_test_page_id(&runner, site_id, &plus_minus_slug).await;
    let stars_id = listpages_test_page_id(&runner, site_id, &stars_slug).await;

    for (user_id, value) in [(ADMIN_USER_ID, 1_i16), (SAMPLE_USER_ID, -1_i16)] {
        runner
            .context()
            .transaction()
            .execute_raw(Statement::from_sql_and_values(
                runner.context().transaction().get_database_backend(),
                "INSERT INTO page_vote (from_wikidot, page_id, user_id, value) \
                 VALUES (false, $1, $2, $3)",
                [
                    Value::from(plus_minus_id),
                    Value::from(user_id),
                    Value::from(value),
                ],
            ))
            .await
            .expect("plus-minus rate matrix should receive its existing votes");
    }
    runner
        .context()
        .transaction()
        .execute_raw(Statement::from_sql_and_values(
            runner.context().transaction().get_database_backend(),
            "INSERT INTO page_vote (from_wikidot, page_id, user_id, rating_system, value) \
             VALUES (false, $1, $2, 'stars', 4)",
            [Value::from(stars_id), Value::from(SAMPLE_USER_ID)],
        ))
        .await
        .expect("stars rate matrix should receive its existing four-star vote");

    for (category, slug, title) in [
        (
            PLUS_MINUS_CATEGORY,
            plus_minus_slug.as_str(),
            "Rate matrix plus-minus",
        ),
        (PLUS_CATEGORY, plus_slug.as_str(), "Rate matrix plus"),
        (STARS_CATEGORY, stars_slug.as_str(), "Rate matrix stars"),
    ] {
        let page = PageTable::find()
            .filter(
                sea_orm::Condition::all()
                    .add(page::Column::SiteId.eq(site_id))
                    .add(page::Column::Slug.eq(slug)),
            )
            .one(runner.context().transaction())
            .await
            .expect("rate matrix holder lookup should succeed")
            .expect("rate matrix holder should exist");
        let page_info = PageInfo {
            page: Cow::Borrowed(slug),
            category: Some(Cow::Borrowed(category)),
            site: Cow::Borrowed("scp-wiki"),
            title: Cow::Borrowed(title),
            alt_title: None,
            score: ScoreValue::Integer(0),
            tags: Vec::new(),
            language: Cow::Borrowed("en"),
        };
        let render = |viewer_user_id| {
            RenderService::render_page_for_viewer(
                runner.context(),
                "RATE_START\n[[module Rate]]\nRATE_END".to_owned(),
                &page_info,
                Layout::Wikidot,
                PageId {
                    site_id,
                    category_id: page.page_category_id,
                    page_id: page.page_id,
                },
                viewer_user_id,
                UrlArguments::default(),
            )
        };
        let anonymous = render(None)
            .await
            .expect("anonymous rate matrix render should succeed")
            .html_output
            .body;
        assert!(
            anonymous.contains(r#"class="page-rate-widget"#)
                || anonymous.contains(r#"class="page-rate-widget-box""#),
            "a rate matrix page must emit its rate widget:\n{anonymous}",
        );
        for viewer in [Some(SAMPLE_USER_ID), Some(ADMIN_USER_ID)] {
            let actor = render(viewer)
                .await
                .expect("rate matrix actor render should succeed")
                .html_output
                .body;
            assert_eq!(
                actor, anonymous,
                "the Rate widget must be actor-independent in {category} mode",
            );
        }
        match category {
            PLUS_MINUS_CATEGORY => {
                assert_eq!(
                    anonymous
                        .matches(r#"class="rateup btn btn-default""#)
                        .count(),
                    1,
                    "{anonymous}",
                );
                assert_eq!(
                    anonymous
                        .matches(r#"class="ratedown btn btn-default""#)
                        .count(),
                    1,
                    "{anonymous}",
                );
                assert_eq!(
                    anonymous
                        .matches(r#"class="cancel btn btn-default""#)
                        .count(),
                    1,
                    "{anonymous}",
                );
                assert!(
                    anonymous.contains(
                        "rating:\u{a0}<span class=\"number prw54353\">0</span>"
                    ),
                    "the plus-minus existing-vote widget must show the aggregate score:\n{anonymous}",
                );
            }
            PLUS_CATEGORY => {
                assert_eq!(
                    anonymous
                        .matches(r#"class="rateup btn btn-default""#)
                        .count(),
                    1,
                    "{anonymous}",
                );
                assert_eq!(
                    anonymous
                        .matches(r#"class="cancel btn btn-default""#)
                        .count(),
                    1,
                    "{anonymous}",
                );
                assert_eq!(
                    anonymous
                        .matches(r#"class="ratedown btn btn-default""#)
                        .count(),
                    0,
                    "disabled plus-only mode has no downvote control:\n{anonymous}",
                );
            }
            STARS_CATEGORY => {
                assert!(
                    anonymous.contains(
                        r#"<div class="page-rate-widget-start" data-rating="4"></div>"#
                    ),
                    "an existing four-star vote must surface data-rating=4:\n{anonymous}",
                );
                assert_eq!(
                    anonymous
                        .matches(r#"class="rateup btn btn-default""#)
                        .count(),
                    0,
                    "{anonymous}",
                );
                assert_eq!(
                    anonymous
                        .matches(r#"class="ratedown btn btn-default""#)
                        .count(),
                    0,
                    "{anonymous}",
                );
                assert_eq!(
                    anonymous
                        .matches(r#"class="cancel btn btn-default""#)
                        .count(),
                    0,
                    "the observed five-star widget has no cancel control:\n{anonymous}",
                );
            }
            other => panic!("unexpected rate matrix category {other}"),
        }
    }
}

#[tokio::test]
async fn unknown_module_dispatch_requires_a_valid_legacy_name() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-unknown-module-boundary",
        "Fixture Unknown Module Boundary",
        concat!(
            "[[module foo=\"bar\"]]\n",
            "[[module https://example.com]]\n",
            "[[module UnknownOracleModule]]\n",
            "[[module654 class=\"\"]]",
        ),
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "fixture-unknown-module-boundary",
            "details": {"compiled": true},
        }),
    )
    .expect("unknown-module fixture should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    assert!(html.contains("[[module foo=&quot;bar&quot;]]"), "{html}");
    assert!(
        html.contains(
            "[[module <a href=\"https://example.com\">https://example.com</a>]]"
        ),
        "{html}"
    );
    assert!(html.contains("[[module <em>UnknownOracleModule</em>]] No such module"));
    assert!(html.contains("[[module654 class=&quot;&quot;]]"));
    assert_eq!(html.matches("No such module").count(), 1, "{html}");
    assert!(!html.contains("TODO: module"), "{html}");
}

#[tokio::test]
async fn page_watchers_returns_active_typed_identities_in_deterministic_order() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    const TARGET_SLUG: &str = "fixture-page-watchers-active";
    const OTHER_SLUG: &str = "fixture-page-watchers-other";

    create_listpages_test_page(
        &mut runner,
        site.site_id,
        TARGET_SLUG,
        "Fixture Page Watchers Active",
        "page watcher target",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site.site_id,
        OTHER_SLUG,
        "Fixture Page Watchers Other",
        "other page watcher target",
    )
    .await;
    let target = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": TARGET_SLUG}),
    )
    .expect("page watcher target should exist");
    let other = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": OTHER_SLUG}),
    )
    .expect("other page watcher target should exist");

    for user_id in [SAMPLE_USER_ID, ADMIN_USER_ID, ADMIN_USER_ID] {
        RelationService::create_page_watch(
            runner.context(),
            CreatePageWatch {
                page_id: target.page_id,
                user_id,
                created_by: ADMIN_USER_ID,
            },
        )
        .await
        .expect("active page watcher fixture should insert");
    }
    RelationService::create_page_watch(
        runner.context(),
        CreatePageWatch {
            page_id: target.page_id,
            user_id: SYSTEM_USER_ID,
            created_by: ADMIN_USER_ID,
        },
    )
    .await
    .expect("removed page watcher fixture should insert");
    RelationService::remove_page_watch(
        runner.context(),
        RemovePageWatch {
            page_id: target.page_id,
            user_id: SYSTEM_USER_ID,
            removed_by: ADMIN_USER_ID,
        },
    )
    .await
    .expect("removed page watcher fixture should be deleted");
    RelationService::create_page_watch(
        runner.context(),
        CreatePageWatch {
            page_id: other.page_id,
            user_id: UNKNOWN_USER_ID,
            created_by: ADMIN_USER_ID,
        },
    )
    .await
    .expect("other-page watcher fixture should insert");

    runner.set_request_context(RequestContext::default());
    let output = run_endpoint!(
        runner,
        page_watchers,
        json!({"site_id": site.site_id, "page_id": target.page_id}),
    );
    let output = serde_json::to_value(output)
        .expect("public page watcher identities should serialize");

    assert_eq!(output.as_array().map(Vec::len), Some(2));
    assert_eq!(output[0]["user-name"], "Administrator");
    assert_eq!(output[0]["user-slug"], "administrator");
    assert_eq!(output[1]["user-name"], "User");
    assert_eq!(output[1]["user-slug"], "user");
    assert!(output.to_string().find("Administrator") < output.to_string().find("User"));
    assert!(!output.to_string().contains("Unknown"));
    assert!(!output.to_string().contains("System"));
}

#[tokio::test]
async fn page_watchers_requires_target_view_permission_and_site_ownership() {
    let mut runner = TestRunner::setup().await;
    const PAGE_SLUG: &str = "fixture-private-page-watchers";
    const PRIVATE_CATEGORY: &str = "fixture-page-watchers-private-view";
    let mirror = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded mirror site should exist")
        .site;
    let other_site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;

    make_listpages_test_category_admin_only(&runner, mirror.site_id, PRIVATE_CATEGORY)
        .await;
    create_listpages_test_page(
        &mut runner,
        mirror.site_id,
        PAGE_SLUG,
        "Fixture Private Page Watchers",
        "private page watcher target",
    )
    .await;
    set_listpages_test_category_slug(
        &runner,
        mirror.site_id,
        PAGE_SLUG,
        PRIVATE_CATEGORY,
    )
    .await;
    let target = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": mirror.site_id, "page": PAGE_SLUG}),
    )
    .expect("admin should resolve the private page watcher target");
    RelationService::create_page_watch(
        runner.context(),
        CreatePageWatch {
            page_id: target.page_id,
            user_id: ADMIN_USER_ID,
            created_by: ADMIN_USER_ID,
        },
    )
    .await
    .expect("private page watcher fixture should insert");

    runner.set_request_context(RequestContext::default());
    let denied = run_endpoint_err!(
        runner,
        page_watchers,
        json!({"site_id": mirror.site_id, "page_id": target.page_id}),
    );
    assert_contains_error!(denied, ErrorType::PermissionDenied);

    let wrong_site = run_endpoint_err!(
        runner,
        page_watchers,
        json!({"site_id": other_site.site_id, "page_id": target.page_id}),
    );
    assert_contains_error!(wrong_site, ErrorType::PageNotFound);

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(mirror.site_id),
        page_reference: Some(Reference::Id(target.page_id)),
        ..Default::default()
    });
    let watchers = run_endpoint!(
        runner,
        page_watchers,
        json!({"site_id": mirror.site_id, "page_id": target.page_id}),
    );
    assert_eq!(watchers.len(), 1);
}

#[tokio::test]
async fn page_watchers_fails_closed_when_any_active_identity_is_incomplete() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    const PAGE_SLUG: &str = "fixture-page-watchers-incomplete";

    create_listpages_test_page(
        &mut runner,
        site.site_id,
        PAGE_SLUG,
        "Fixture Page Watchers Incomplete",
        "incomplete page watcher target",
    )
    .await;
    let target = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": PAGE_SLUG}),
    )
    .expect("incomplete page watcher target should exist");
    for user_id in [ADMIN_USER_ID, 19_000_001] {
        RelationService::create_page_watch(
            runner.context(),
            CreatePageWatch {
                page_id: target.page_id,
                user_id,
                created_by: ADMIN_USER_ID,
            },
        )
        .await
        .expect("incomplete page watcher fixture should insert");
    }

    runner.set_request_context(RequestContext::default());
    let error = run_endpoint_err!(
        runner,
        page_watchers,
        json!({"site_id": site.site_id, "page_id": target.page_id}),
    );
    assert_contains_error!(error, ErrorType::PageWatchRelation);
}

#[tokio::test]
async fn page_watchers_fails_closed_instead_of_returning_a_saturated_prefix() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    const PAGE_SLUG: &str = "fixture-page-watchers-saturated";

    create_listpages_test_page(
        &mut runner,
        site.site_id,
        PAGE_SLUG,
        "Fixture Page Watchers Saturated",
        "saturated page watcher target",
    )
    .await;
    let target = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": PAGE_SLUG}),
    )
    .expect("saturated page watcher target should exist");
    runner
        .context()
        .transaction()
        .execute_raw(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            concat!(
                "INSERT INTO relation ",
                "(relation_type, dest_type, dest_id, from_type, from_id, metadata, created_by) ",
                "SELECT 'watch', 'page', $1, 'user', 19000000 + n, '{}'::jsonb, $2 ",
                "FROM generate_series(1, 501) AS n",
            ),
            [Value::from(target.page_id), Value::from(ADMIN_USER_ID)],
        ))
        .await
        .expect("saturated page watcher fixtures should insert");

    runner.set_request_context(RequestContext::default());
    let error = run_endpoint_err!(
        runner,
        page_watchers,
        json!({"site_id": site.site_id, "page_id": target.page_id}),
    );
    assert_contains_error!(error, ErrorType::PageWatchRelation);
}

#[tokio::test]
async fn page_who_rated_returns_only_current_typed_votes_in_creation_order() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    const TARGET_SLUG: &str = "fixture-page-who-rated";
    const OTHER_SLUG: &str = "fixture-page-who-rated-other";
    create_listpages_test_page(
        &mut runner,
        site.site_id,
        TARGET_SLUG,
        "Fixture Page Who Rated",
        "WhoRated target",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site.site_id,
        OTHER_SLUG,
        "Fixture Page Who Rated Other",
        "other target",
    )
    .await;
    let target = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": TARGET_SLUG}),
    )
    .expect("WhoRated target should exist");
    let other = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": OTHER_SLUG}),
    )
    .expect("other target should exist");
    {
        let transaction = runner.context().transaction();
        transaction
            .execute_raw(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                concat!(
                    "INSERT INTO page_vote ",
                    "(from_wikidot, page_id, user_id, rating_system, value, deleted_at, disabled_at, disabled_by) VALUES ",
                    "(false, $1, $2, 'points', 1, NULL, NULL, NULL), ",
                    "(false, $1, $3, 'points', -1, NULL, NULL, NULL), ",
                    "(true, $1, $4, 'points', 1, NOW(), NULL, NULL), ",
                    "(true, $1, $5, 'points', -1, NULL, NOW(), $2), ",
                    "(true, $6, $4, 'points', 1, NULL, NULL, NULL)",
                ),
                [
                    Value::from(target.page_id),
                    Value::from(ADMIN_USER_ID),
                    Value::from(SAMPLE_USER_ID),
                    Value::from(SYSTEM_USER_ID),
                    Value::from(UNKNOWN_USER_ID),
                    Value::from(other.page_id),
                ],
            ))
            .await
            .expect("WhoRated vote fixtures should insert");
    }

    runner.set_request_context(RequestContext::default());
    let output = run_endpoint!(
        runner,
        page_who_rated,
        json!({"site_id": site.site_id, "page_id": target.page_id}),
    );
    let output = serde_json::to_value(output).expect("WhoRated output should serialize");
    assert_eq!(output.as_array().map(Vec::len), Some(2));
    assert_eq!(output[0]["user"]["user-name"], "Administrator");
    assert_eq!(output[0]["value"], 1);
    assert_eq!(output[1]["user"]["user-name"], "User");
    assert_eq!(output[1]["value"], -1);
    assert!(output[0].get("user_id").is_none());

    let malformed = run_endpoint_err!(
        runner,
        page_who_rated,
        json!({"site_id": site.site_id, "page_id": target.page_id, "extra": true}),
    );
    assert!(
        format!("{malformed:?}").contains("InvalidParams"),
        "JSONRPC InvalidParams error not returned:\n{:?}",
        malformed,
    );

    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "INSERT INTO known_user (user_id) VALUES (19000001)",
        ))
        .await
        .expect("incomplete WhoRated identity should insert");
    transaction
        .execute_raw(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            "INSERT INTO page_vote (from_wikidot, page_id, user_id, rating_system, value) VALUES (false, $1, 19000001, 'points', 1)",
            [Value::from(target.page_id)],
        ))
        .await
        .expect("incomplete WhoRated vote should insert");
    let incomplete = run_endpoint_err!(
        runner,
        page_who_rated,
        json!({"site_id": site.site_id, "page_id": target.page_id}),
    );
    assert_contains_error!(incomplete, ErrorType::PageVote);
}

#[tokio::test]
async fn page_who_rated_checks_view_and_visible_rating_policy_before_votes() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    const TARGET_SLUG: &str = "fixture-page-who-rated-policy";
    create_listpages_test_page(
        &mut runner,
        site.site_id,
        TARGET_SLUG,
        "Fixture Page Who Rated Policy",
        "WhoRated policy target",
    )
    .await;
    let target = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": TARGET_SLUG}),
    )
    .expect("WhoRated policy target should exist");
    let category = PageCategoryTable::find_by_id(target.page_category_id)
        .one(runner.context().transaction())
        .await
        .expect("WhoRated category lookup should succeed")
        .expect("WhoRated category should exist");
    let mut category = category.into_active_model();
    category.rating_visibility = Set(Some("anonymous".to_owned()));
    category
        .update(runner.context().transaction())
        .await
        .expect("WhoRated category visibility should update");

    runner.set_request_context(RequestContext::default());
    let hidden = run_endpoint_err!(
        runner,
        page_who_rated,
        json!({"site_id": site.site_id, "page_id": target.page_id}),
    );
    assert_contains_error!(hidden, ErrorType::PermissionDenied);

    let wrong_site = run_endpoint_err!(
        runner,
        page_who_rated,
        json!({"site_id": site.site_id + 1, "page_id": target.page_id}),
    );
    assert_contains_error!(wrong_site, ErrorType::PermissionDenied);
}

#[tokio::test]
async fn page_who_rated_fails_closed_only_after_a_scan_above_observed_live_counts() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    const TARGET_SLUG: &str = "fixture-page-who-rated-saturated";
    create_listpages_test_page(
        &mut runner,
        site.site_id,
        TARGET_SLUG,
        "Fixture Page Who Rated Saturated",
        "WhoRated saturated target",
    )
    .await;
    let target = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": TARGET_SLUG}),
    )
    .expect("WhoRated saturated target should exist");
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "INSERT INTO known_user (user_id) SELECT 19000000 + n FROM generate_series(1, 16385) AS n",
        ))
        .await
        .expect("WhoRated saturated known users should insert");
    transaction
        .execute_raw(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            concat!(
                "INSERT INTO page_vote (from_wikidot, page_id, user_id, rating_system, value) ",
                "SELECT false, $1, 19000000 + n, 'points', 1 ",
                "FROM generate_series(1, 16385) AS n",
            ),
            [Value::from(target.page_id)],
        ))
        .await
        .expect("WhoRated saturated votes should insert");

    runner.set_request_context(RequestContext::default());
    let error = run_endpoint_err!(
        runner,
        page_who_rated,
        json!({"site_id": site.site_id, "page_id": target.page_id}),
    );
    assert_contains_error!(error, ErrorType::PageVote);
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
        page_revision_get_by_id,
        json!({
            "site_id": site_id,
            "revision_id": created.revision_id,
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

    let revision = run_endpoint!(
        runner,
        page_revision_get_by_id,
        json!({
            "site_id": site_id,
            "revision_id": created.revision_id,
            "details": {
                "wikitext": true,
                "compiled_html": true
            },
        }),
    )
    .expect("admin should be allowed to resolve a private revision by ID");
    assert_eq!(revision.page_id, created.page_id);
    assert_eq!(revision.revision_id, created.revision_id);
    assert_eq!(
        revision.author.as_ref().map(|author| author.user_id),
        Some(ADMIN_USER_ID)
    );
    assert_eq!(
        revision.wikitext.as_deref(),
        Some("private revision body marker")
    );

    let revision = run_endpoint!(
        runner,
        page_revision_get_by_id,
        json!({
            "site_id": site_id + 1,
            "revision_id": created.revision_id,
        }),
    );
    assert!(
        revision.is_none(),
        "a revision ID must not cross its site boundary"
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
async fn page_revision_range_returns_resolved_authors_and_hides_missing_identities() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    const PAGE_SLUG: &str = "history-author-identity";

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
            "wikitext": "first revision",
            "title": "History author identity",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "created by the seeded administrator",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let edited = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site.site_id,
            "page": created.page_id,
            "last_revision_id": created.revision_id,
            "revision_comments": "missing author fixture",
            "user_id": ADMIN_USER_ID,
            "wikitext": "second revision",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("page edit should create a second revision");

    let missing_user_id = 19_000_001_i64;
    known_user::ActiveModel {
        user_id: Set(missing_user_id),
    }
    .insert(runner.context().transaction())
    .await
    .expect("orphan known-user fixture should insert");
    let revision = PageRevisionTable::find_by_id(edited.revision_id)
        .one(runner.context().transaction())
        .await
        .expect("revision author fixture lookup should succeed")
        .expect("edited revision should exist");
    let mut revision = revision.into_active_model();
    revision.user_id = Set(missing_user_id);
    revision
        .update(runner.context().transaction())
        .await
        .expect("revision author fixture should update");

    runner.set_request_context(RequestContext::default());
    let revisions = run_endpoint!(
        runner,
        page_revision_range,
        json!({
            "site_id": site.site_id,
            "page_id": created.page_id,
            "revision_number": 1,
            "revision_direction": "before",
            "limit": 2,
        }),
    );
    let output = serde_json::to_value(revisions)
        .expect("public page revision range output should serialize");

    assert_eq!(output[1]["author"]["user-name"], "Administrator");
    assert_eq!(output[1]["author"]["user-slug"], "administrator");
    assert_eq!(output[0]["author"], serde_json::Value::Null);
    assert_ne!(output[0]["author"], missing_user_id);

    let administrator = UserTable::find_by_id(ADMIN_USER_ID)
        .one(runner.context().transaction())
        .await
        .expect("administrator identity lookup should succeed")
        .expect("seeded administrator identity should exist");
    let mut administrator = administrator.into_active_model();
    administrator.deleted_at = Set(Some(OffsetDateTime::now_utc()));
    administrator
        .update(runner.context().transaction())
        .await
        .expect("deleted author fixture should update");
    let revisions = run_endpoint!(
        runner,
        page_revision_range,
        json!({
            "site_id": site.site_id,
            "page_id": created.page_id,
            "revision_number": 0,
            "revision_direction": "before",
            "limit": 1,
        }),
    );
    let output = serde_json::to_value(revisions)
        .expect("deleted author page revision range should serialize");
    assert_eq!(output[0]["author"], serde_json::Value::Null);
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
async fn file_revision_reads_require_parent_page_view_permission_and_tuple_binding() {
    let mut runner = TestRunner::setup().await;
    const SITE_SLUG: &str = "scp-wiki";
    const PRIVATE_PAGE_SLUG: &str = "fixture-private-file-revision-read";
    const PUBLIC_PAGE_SLUG: &str = "fixture-public-file-revision-read";
    const PRIVATE_CATEGORY: &str = "fixture-file-revision-read-private-view";
    const PRIVATE_FILE_NAME: &str = "private-revision-attachment.txt";
    const PUBLIC_FILE_NAME: &str = "public-revision-attachment.txt";

    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("Seeded site not found");
    let site_id = site.site.site_id;
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(PRIVATE_PAGE_SLUG))),
    });
    let private_page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "private file revision parent page",
            "title": "Private File Revision Read",
            "alt_title": null,
            "slug": PRIVATE_PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create private file revision read fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(private_page.parser_errors.is_empty());
    set_listpages_test_category_slug(
        &runner,
        site_id,
        PRIVATE_PAGE_SLUG,
        PRIVATE_CATEGORY,
    )
    .await;
    let private_file_id = create_empty_file_fixture(
        &runner,
        site_id,
        private_page.page_id,
        PRIVATE_FILE_NAME,
    )
    .await;

    let public_page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "public file revision parent page",
            "title": "Public File Revision Read",
            "alt_title": null,
            "slug": PUBLIC_PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create public file revision read fixture",
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

    let public_count = run_endpoint!(
        runner,
        file_revision_count,
        json!({
            "site_id": site_id,
            "page_id": public_page.page_id,
            "file": PUBLIC_FILE_NAME,
        }),
    );
    assert_eq!(public_count.revision_count.get(), 1);

    let public_revision = run_endpoint!(
        runner,
        file_revision_get,
        json!({
            "site_id": site_id,
            "page_id": public_page.page_id,
            "file_id": public_file_id,
            "revision_number": 0,
        }),
    )
    .expect("anonymous user should be allowed to view a public file revision");
    assert_eq!(public_revision.file_id, public_file_id);

    let public_revisions = run_endpoint!(
        runner,
        file_revision_range,
        json!({
            "site_id": site_id,
            "page_id": public_page.page_id,
            "file_id": public_file_id,
            "revision_number": 0,
            "revision_direction": "before",
            "limit": 1,
        }),
    );
    assert_eq!(public_revisions.len(), 1);
    assert_eq!(public_revisions[0].file_id, public_file_id);

    let mismatched_revision = run_endpoint!(
        runner,
        file_revision_get,
        json!({
            "site_id": site_id,
            "page_id": public_page.page_id,
            "file_id": private_file_id,
            "revision_number": 0,
        }),
    );
    assert!(mismatched_revision.is_none());

    let mismatched_revisions = run_endpoint!(
        runner,
        file_revision_range,
        json!({
            "site_id": site_id,
            "page_id": public_page.page_id,
            "file_id": private_file_id,
            "revision_number": 0,
            "revision_direction": "before",
            "limit": 1,
        }),
    );
    assert!(mismatched_revisions.is_empty());

    let mismatched_count_error = run_endpoint_err!(
        runner,
        file_revision_count,
        json!({
            "site_id": site_id,
            "page_id": public_page.page_id,
            "file": private_file_id,
        }),
    );
    assert_contains_error!(mismatched_count_error, ErrorType::FileNotFound);

    for error in [
        run_endpoint_err!(
            runner,
            file_revision_count,
            json!({
                "site_id": site_id,
                "page_id": private_page.page_id,
                "file": PRIVATE_FILE_NAME,
            }),
        ),
        run_endpoint_err!(
            runner,
            file_revision_get,
            json!({
                "site_id": site_id,
                "page_id": private_page.page_id,
                "file_id": private_file_id,
                "revision_number": 0,
            }),
        ),
        run_endpoint_err!(
            runner,
            file_revision_range,
            json!({
                "site_id": site_id,
                "page_id": private_page.page_id,
                "file_id": private_file_id,
                "revision_number": 0,
                "revision_direction": "before",
                "limit": 1,
            }),
        ),
    ] {
        assert_contains_error!(error, ErrorType::PermissionDenied);
    }

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(private_page.page_id)),
    });

    let private_revisions = run_endpoint!(
        runner,
        file_revision_range,
        json!({
            "site_id": site_id,
            "page_id": private_page.page_id,
            "file_id": private_file_id,
            "revision_number": 0,
            "revision_direction": "before",
            "limit": 1,
        }),
    );
    assert_eq!(private_revisions.len(), 1);
    assert_eq!(private_revisions[0].file_id, private_file_id);
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
async fn anonymous_forum_post_create_persists_private_derived_guest_identity_and_renders_gravatar()
 {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "Guest Gravatar Group".to_owned(),
            description: String::new(),
            visible: true,
            sort_index: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("guest Gravatar group should be created");
    let category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "Guest Gravatar Category".to_owned(),
            description: String::new(),
            sort_index: None,
            max_nest_level: Some(3),
            per_page_discussion: Some(false),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("guest Gravatar category should be created");
    let thread = ForumThreadService::create(
        runner.context(),
        CreateForumThread {
            forum_category_id: category.forum_category_id,
            user_id: ADMIN_USER_ID,
            associated_page_id: None,
            title: "Guest Gravatar Thread".to_owned(),
            description: String::new(),
            sticky: false,
            from_wikidot: false,
        },
    )
    .await
    .expect("guest Gravatar thread should be created");

    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: None,
    });
    let denied = run_endpoint_err!(
        runner,
        forum_post_create,
        json!({
            "site_id": site_id,
            "forum_thread_id": thread.forum_thread_id,
            "parent_post_id": null,
            "title": "",
            "wikitext": "denied guest Gravatar body",
            "guest_name": "Denied Guest",
            "guest_email_md5": "84991830db6f52c0a36a85d452311203",
        }),
    );
    assert_contains_error!(denied, ErrorType::PermissionDenied);

    let anonymous_role = RoleService::get(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed("anonymous")),
    )
    .await
    .expect("anonymous role should exist");
    role_permission::ActiveModel {
        role_id: Set(anonymous_role.role_id),
        site_id: Set(site_id),
        resource_type: Set(Resource::ForumCategory),
        resource_category_id: Set(Some(category.forum_category_id)),
        action: Set(Action::Create),
        ..Default::default()
    }
    .insert(runner.context().transaction())
    .await
    .expect("anonymous forum post permission should be inserted");
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("forum permission cache should be invalidated");

    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: None,
    });
    let created = run_endpoint!(
        runner,
        forum_post_create,
        json!({
            "site_id": site_id,
            "forum_thread_id": thread.forum_thread_id,
            "parent_post_id": null,
            "title": "",
            "wikitext": "guest Gravatar body",
            "guest_name": "Guest Name",
            "guest_email_md5": "84991830db6f52c0a36a85d452311203",
        }),
    );

    let rendered = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumViewThreadPostsModule",
            "parameters": {"t": thread.forum_thread_id.to_string(), "pageNo": "1"},
        }),
    );
    assert_eq!(rendered.status, "ok");
    assert!(
        rendered
            .body
            .contains(&format!(r#"id="fpc-{}""#, created.forum_post_id))
    );
    assert!(rendered.body.contains(concat!(
        r#"<span class="printuser avatarhover"><a href="javascript:;"><img alt="" class="small" "#,
        r#"src="http://www.gravatar.com/avatar.php?gravatar_id=84991830db6f52c0a36a85d452311203&amp;default=http://www.wikidot.com/common--images/avatars/default/a16.png&amp;size=16"/></a>Guest Name (guest)</span>"#,
    )));

    let admin_role = RoleService::get(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed("admin")),
    )
    .await
    .expect("admin role should exist");
    role_permission::ActiveModel {
        role_id: Set(admin_role.role_id),
        site_id: Set(site_id),
        resource_type: Set(Resource::ForumCategory),
        resource_category_id: Set(Some(category.forum_category_id)),
        action: Set(Action::Create),
        ..Default::default()
    }
    .insert(runner.context().transaction())
    .await
    .expect("administrator forum post permission should be inserted");
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("administrator forum permission cache should be invalidated");
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: None,
    });
    let authenticated = run_endpoint!(
        runner,
        forum_post_create,
        json!({
            "site_id": site_id,
            "forum_thread_id": thread.forum_thread_id,
            "parent_post_id": null,
            "title": "",
            "wikitext": "authenticated body",
            "guest_name": "Must Be Ignored",
            "guest_email_md5": "84991830db6f52c0a36a85d452311203",
        }),
    );
    let authenticated_rendered = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumViewThreadPostsModule",
            "parameters": {"t": thread.forum_thread_id.to_string(), "pageNo": "1"},
        }),
    );
    let authenticated_start = authenticated_rendered
        .body
        .find(&format!(r#"id="fpc-{}""#, authenticated.forum_post_id))
        .expect("authenticated forum post should render");
    let authenticated_tail = &authenticated_rendered.body[authenticated_start..];
    assert!(authenticated_tail.contains("Administrator"));
    assert!(!authenticated_tail.contains("Must Be Ignored (guest)"));

    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: None,
    });
    for input in [
        json!({
            "site_id": site_id,
            "forum_thread_id": thread.forum_thread_id,
            "parent_post_id": null,
            "title": "",
            "wikitext": "missing guest identity",
        }),
        json!({
            "site_id": site_id,
            "forum_thread_id": thread.forum_thread_id,
            "parent_post_id": null,
            "title": "",
            "wikitext": "malformed guest identity",
            "guest_name": "Guest Name",
            "guest_email_md5": "not-a-md5",
        }),
    ] {
        let error = run_endpoint_err!(runner, forum_post_create, input);
        assert_contains_error!(error, ErrorType::BadRequest);
    }
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
    let before_denied_delete =
        snapshot_page_action_mutation_state(&runner, site_id, page.page_id).await;
    let error = run_endpoint_err!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": page.page_id,
            "last_revision_id": page.revision_id,
            "revision_comments": "blocked delete without delete action",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);
    assert_eq!(
        snapshot_page_action_mutation_state(&runner, site_id, page.page_id).await,
        before_denied_delete,
        "delete permission denial must precede stale-revision checks and preserve state",
    );

    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        PRIVATE_CATEGORY,
        SAMPLE_USER_ID,
        &[Action::Delete],
        "sample-deleter",
    )
    .await;
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("page delete permission cache should be invalidated");
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
    const PAGE_SLUG: &str = "fixture-page-move-source-private:target";
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
    let before_source_denial =
        snapshot_page_action_mutation_state(&runner, site_id, page.page_id).await;
    let error = run_endpoint_err!(
        runner,
        page_move,
        json!({
            "site_id": site_id,
            "page": page.page_id,
            "new_slug": ALLOWED_DESTINATION_SLUG,
            "last_revision_id": page.revision_id,
            "revision_comments": "blocked same-category move without rename",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);
    assert_eq!(
        snapshot_page_action_mutation_state(&runner, site_id, page.page_id).await,
        before_source_denial,
        "source rename denial must preserve page, revision, category, audit, and text-block state",
    );

    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        SOURCE_CATEGORY,
        UNKNOWN_USER_ID,
        &[Action::Rename],
        "unknown-renamer",
    )
    .await;
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("page move permission cache should be invalidated");
    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );

    let before_destination_denial =
        snapshot_page_action_mutation_state(&runner, site_id, page.page_id).await;
    let error = run_endpoint_err!(
        runner,
        page_move,
        json!({
            "site_id": site_id,
            "page": page.page_id,
            "new_slug": ALLOWED_DESTINATION_SLUG,
            "last_revision_id": page.revision_id,
            "revision_comments": "blocked same-category move without destination create",
            "user_id": UNKNOWN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);
    assert_eq!(
        snapshot_page_action_mutation_state(&runner, site_id, page.page_id).await,
        before_destination_denial,
        "destination create denial must preserve page, revision, category, audit, and text-block state",
    );

    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        SOURCE_CATEGORY,
        UNKNOWN_USER_ID,
        &[Action::Create],
        "unknown-creator",
    )
    .await;
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("page move destination permission cache should be invalidated");
    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );

    let moved = run_endpoint!(
        runner,
        page_move,
        json!({
            "site_id": site_id,
            "page": page.page_id,
            "new_slug": ALLOWED_DESTINATION_SLUG,
            "last_revision_id": page.revision_id,
            "revision_comments": "authorized same-category move",
            "user_id": UNKNOWN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(moved.revision_id > page.revision_id);
}

#[tokio::test]
async fn page_get_deleted_requires_an_authenticated_request_context() {
    let runner = TestRunner::setup().await;
    let error = run_endpoint_err!(
        runner,
        page_get_deleted,
        json!({
            "site_id": 6000005,
            "slug": "missing-deleted-page",
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);
}

#[tokio::test]
async fn page_get_score_requires_view_permission_and_site_ownership() {
    const PRIVATE_CATEGORY: &str = "fixture-page-score-private";
    const PAGE_SLUG: &str = "fixture-page-score-private:target";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        PRIVATE_CATEGORY,
        ADMIN_USER_ID,
        &[Action::View, Action::Create, Action::Edit],
        "score-admin",
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
            "wikitext": "Private page score authorization source.",
            "title": "Private Page Score Authorization Source",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create private page score authorization source",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    set_listpages_test_category_slug(&runner, site_id, PAGE_SLUG, PRIVATE_CATEGORY).await;
    set_stored_point_vote(&runner, page.page_id, 3).await;
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("page score permission cache should be invalidated");

    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(PAGE_SLUG))),
    });
    let private_error = run_endpoint_err!(
        runner,
        page_get_score,
        json!({"site_id": site_id, "page": page.page_id}),
    );
    assert_contains_error!(private_error, ErrorType::PermissionDenied);

    let cross_site_error = run_endpoint_err!(
        runner,
        page_get_score,
        json!({"site_id": site_id + 1, "page": page.page_id}),
    );
    assert_contains_error!(cross_site_error, ErrorType::Permission);

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(page.page_id)),
    });
    let score = run_endpoint!(
        runner,
        page_get_score,
        json!({"site_id": site_id, "page": page.page_id}),
    );
    assert_eq!(score.score, QueryScoreValue::Integer(3));
}

#[tokio::test]
async fn page_get_deleted_filters_pages_by_delete_permission() {
    const PRIVATE_CATEGORY: &str = "fixture-page-deleted-metadata-private";
    const PAGE_SLUG: &str = "fixture-page-deleted-metadata-private:target";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        PRIVATE_CATEGORY,
        ADMIN_USER_ID,
        &[Action::View, Action::Create, Action::Edit, Action::Delete],
        "deleted-metadata-admin",
    )
    .await;
    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        PRIVATE_CATEGORY,
        SAMPLE_USER_ID,
        &[Action::View, Action::Edit],
        "deleted-metadata-editor",
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
            "wikitext": "Deleted page metadata authorization source.",
            "title": "Deleted Page Metadata Authorization Source",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create deleted page metadata authorization source",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    set_listpages_test_category_slug(&runner, site_id, PAGE_SLUG, PRIVATE_CATEGORY).await;

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
            "revision_comments": "delete deleted page metadata authorization source",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("deleted page permission cache should be invalidated");
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(SAMPLE_USER_ID),
        site_id: Some(site_id),
        page_reference: None,
    });

    let deleted_pages = run_endpoint!(
        runner,
        page_get_deleted,
        json!({
            "site_id": site_id,
            "slug": PAGE_SLUG,
        }),
    );
    assert!(
        deleted_pages.is_empty(),
        "deleted page metadata must not be returned with edit but without delete permission"
    );

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: None,
    });
    let authorized_deleted_pages = run_endpoint!(
        runner,
        page_get_deleted,
        json!({
            "site_id": site_id,
            "slug": PAGE_SLUG,
        }),
    );
    assert_eq!(authorized_deleted_pages.len(), 1);
}

#[tokio::test]
async fn restored_countpages_page_stores_and_serves_its_current_self_count() {
    const PAGE_SLUG: &str = "fixture-page-restore-countpages:self";
    const COUNT_MARKER: &str = "RESTORED_SELF_COUNT=1";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

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
            "wikitext": "[[module CountPages]]RESTORED_SELF_COUNT=%%total%%[[/module]]",
            "title": "Restored CountPages Self Count",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create CountPages restore fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let deleted = run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": page.page_id,
            "last_revision_id": page.revision_id,
            "revision_comments": "delete CountPages restore fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let deleted = serde_json::to_value(deleted)
        .expect("CountPages deletion output should serialize");
    let deleted_revision_id = deleted["revision_id"]
        .as_i64()
        .expect("CountPages deletion should return its revision ID");

    run_endpoint!(
        runner,
        page_restore,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "revision_comments": "restore CountPages fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(PAGE_SLUG))),
        ..Default::default()
    });
    let stored = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("restored CountPages page should be readable");
    assert!(stored.revision_id > deleted_revision_id);
    let stored_html = stored
        .compiled_body_html
        .expect("restored CountPages page should store compiled HTML");
    assert!(
        stored_html.contains(COUNT_MARKER),
        "restored CountPages stored HTML must count the now-live page itself:\n{stored_html}",
    );

    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": PAGE_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let GetPageViewOutput::Found {
        compiled_body_html, ..
    } = view
    else {
        panic!("restored CountPages page should have a public view: {view:?}");
    };
    assert!(
        compiled_body_html.contains(COUNT_MARKER),
        "restored CountPages public view must count the now-live page itself:\n{compiled_body_html}",
    );
}

#[tokio::test]
async fn restored_page_uses_destination_template_countpages_after_explicit_restore() {
    const SOURCE_SLUG: &str = "fixture-page-restore-countpages-source:templated";
    const DESTINATION_CATEGORY: &str = "fixture-page-restore-countpages-destination";
    const DESTINATION_TEMPLATE_SLUG: &str =
        "fixture-page-restore-countpages-destination:_template";
    const DESTINATION_SLUG: &str =
        "fixture-page-restore-countpages-destination:templated";
    const COUNT_MARKER: &str = "RESTORED_TEMPLATE_COUNT=1";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_listpages_test_page(
        &mut runner,
        site_id,
        DESTINATION_TEMPLATE_SLUG,
        "Restore CountPages Destination Template",
        "[[module CountPages]]RESTORED_TEMPLATE_COUNT=%%total%%[[/module]]\n\n%%content%%",
    )
    .await;

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(SOURCE_SLUG)),
    );
    let page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "RESTORED_TEMPLATE_CONTENT",
            "title": "Restored Destination Template CountPages",
            "alt_title": null,
            "slug": SOURCE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create destination template restore fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

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
            "revision_comments": "delete destination template restore fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    run_endpoint!(
        runner,
        page_restore,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "slug": DESTINATION_SLUG,
            "revision_comments": "restore into CountPages template category",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(DESTINATION_SLUG))),
        ..Default::default()
    });
    let stored = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": DESTINATION_SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("explicitly restored CountPages template page should be readable");
    assert_eq!(stored.page_id, page.page_id);
    assert_eq!(stored.slug, DESTINATION_SLUG);
    assert_eq!(stored.page_category_slug, DESTINATION_CATEGORY);
    let stored_html = stored
        .compiled_body_html
        .expect("explicitly restored template page should store compiled HTML");
    for expected in [COUNT_MARKER, "RESTORED_TEMPLATE_CONTENT"] {
        assert!(
            stored_html.contains(expected),
            "restored template-composed body should contain {expected:?}:\n{stored_html}",
        );
    }

    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": DESTINATION_SLUG, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let GetPageViewOutput::Found {
        page: viewed_page,
        compiled_body_html,
        ..
    } = view
    else {
        panic!("explicitly restored template page should have a public view: {view:?}");
    };
    assert_eq!(viewed_page.slug, DESTINATION_SLUG);
    assert_eq!(viewed_page.page_category_id, stored.page_category_id);
    for expected in [COUNT_MARKER, "RESTORED_TEMPLATE_CONTENT"] {
        assert!(
            compiled_body_html.contains(expected),
            "public restored template view should contain {expected:?}:\n{compiled_body_html}",
        );
    }

    assert!(
        run_endpoint!(
            runner,
            page_get,
            json!({"site_id": site_id, "page": SOURCE_SLUG}),
        )
        .is_none(),
        "the pre-restore slug must remain unavailable",
    );
}

#[tokio::test]
async fn restored_literal_and_unsupported_countpages_shapes_stay_inert() {
    const PAGE_SLUG: &str = "fixture-page-restore-countpages:literal";
    const SOURCE: &str = concat!(
        "[[code]]\n",
        "[[module CountPages]]CODE_LITERAL_COUNT=%%total%%[[/module]]\n",
        "[[/code]]\n\n",
        "[[module CountPages tags=\"@URL\"]]DYNAMIC_LITERAL_COUNT=%%total%%[[/module]]",
    );

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

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
            "wikitext": SOURCE,
            "title": "Restored Literal CountPages Shapes",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": "wikidot",
            "revision_comments": "create literal CountPages restore fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let before = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("literal CountPages fixture should be readable")
    .compiled_body_html
    .expect("literal CountPages fixture should store compiled HTML");
    assert!(before.contains("CODE_LITERAL_COUNT=%%total%%"));
    assert!(before.contains("DYNAMIC_LITERAL_COUNT=%%total%%"));

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
            "revision_comments": "delete literal CountPages restore fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    run_endpoint!(
        runner,
        page_restore,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "revision_comments": "restore literal CountPages fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    runner.set_request_context(RequestContext {
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(PAGE_SLUG))),
        ..Default::default()
    });
    let after = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("restored literal CountPages fixture should be readable")
    .compiled_body_html
    .expect("restored literal CountPages fixture should store compiled HTML");
    assert_eq!(after, before);

    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": PAGE_SLUG, "extra": "/tag/should-not-activate"},
            "locales": ["en-US", "en"],
        }),
    );
    let GetPageViewOutput::Found {
        compiled_body_html, ..
    } = view
    else {
        panic!("restored literal CountPages page should have a public view: {view:?}");
    };
    assert_eq!(compiled_body_html, before);
}

#[tokio::test]
async fn restore_rerender_failure_rolls_back_resurrection_identity_and_revision() {
    let run_id = cuid();
    let source_category = format!("fixture-page-restore-rollback-source-{run_id}");
    let destination_category =
        format!("fixture-page-restore-rollback-destination-{run_id}");
    let source_slug = format!("{source_category}:target");
    let destination_slug = format!("{destination_category}:target");
    let fault_function = format!("force_restore_rerender_failure_{run_id}");
    let fault_trigger = format!("force_restore_rerender_failure_{run_id}");

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Owned(source_slug.clone())),
    );
    let page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "[[module CountPages]]RESTORE_ROLLBACK_COUNT=%%total%%[[/module]]",
            "title": "Restore Rerender Rollback Target",
            "alt_title": null,
            "slug": source_slug,
            "layout": "wikidot",
            "revision_comments": "create restore rollback target",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
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
            "revision_comments": "delete restore rollback target",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        ..Default::default()
    });
    let before = serde_json::to_value(run_endpoint!(
        runner,
        page_get_deleted,
        json!({"site_id": site_id, "slug": source_slug}),
    ))
    .expect("deleted restore rollback state should serialize");
    assert_eq!(before.as_array().map(Vec::len), Some(1));

    runner
        .context()
        .transaction()
        .execute_unprepared(&format!(
            concat!(
                "CREATE FUNCTION pg_temp.{fault_function}() RETURNS trigger ",
                "LANGUAGE plpgsql AS $$ BEGIN ",
                "RAISE EXCEPTION 'forced restore full rerender failure'; ",
                "END $$; ",
                "CREATE TRIGGER {fault_trigger} ",
                "BEFORE UPDATE OF compiled_body_html_hash ON page_revision ",
                "FOR EACH ROW WHEN (NEW.page_id = {page_id}) ",
                "EXECUTE FUNCTION pg_temp.{fault_function}()",
            ),
            fault_function = fault_function,
            fault_trigger = fault_trigger,
            page_id = page.page_id,
        ))
        .await
        .expect("restore rerender fault trigger should install");

    let transaction = runner
        .context()
        .transaction()
        .begin()
        .await
        .expect("restore rollback savepoint should begin");
    let ctx =
        ServiceContext::new(runner.state(), &transaction).with_request(RequestContext {
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site_id),
            page_reference: Some(Reference::Id(page.page_id)),
            ..Default::default()
        });
    let error = deepwell::endpoints::all::page_restore(
        &ctx,
        common::make_params(json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "slug": destination_slug,
            "revision_comments": "force failure after resurrection publication",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        })),
    )
    .await
    .expect_err("post-publication full rerender should hit the body-update fault");
    assert!(
        format!("{error:?}").contains("forced restore full rerender failure"),
        "restore should fail in the post-publication full rerender: {error:?}",
    );
    drop(ctx);
    transaction
        .rollback()
        .await
        .expect("failed restore savepoint should roll back");

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        ..Default::default()
    });
    let after = serde_json::to_value(run_endpoint!(
        runner,
        page_get_deleted,
        json!({"site_id": site_id, "slug": source_slug}),
    ))
    .expect("rolled-back deleted page state should serialize");
    assert_eq!(after, before);
    assert!(
        run_endpoint!(
            runner,
            page_get,
            json!({"site_id": site_id, "page": destination_slug}),
        )
        .is_none(),
        "failed explicit restore must not publish the destination identity",
    );

    let view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": destination_slug, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    assert!(
        matches!(view, GetPageViewOutput::Missing { .. }),
        "failed explicit restore destination must stay publicly missing: {view:?}",
    );
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
    const CONFLICT_SLUG: &str = "fixture-page-restore-private:conflict";

    let site = run_endpoint!(runner, site_get, json!({"site": SITE_SLUG}))
        .expect("Seeded site not found");
    let site_id = site.site.site_id;

    RelationService::create_site_member(
        runner.context(),
        CreateSiteMember {
            site_id,
            user_id: SAMPLE_USER_ID,
            metadata: SiteMemberData {
                accepted: SiteMemberAccepted::SelfJoined,
            },
            created_by: SYSTEM_USER_ID,
        },
        common::IP_ADDRESS,
    )
    .await
    .expect("restore permission fixture actor should be a site member");

    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        PRIVATE_CATEGORY,
        ADMIN_USER_ID,
        &[Action::View, Action::Create, Action::Edit, Action::Delete],
        "admin-mutator",
    )
    .await;
    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        PRIVATE_CATEGORY,
        SAMPLE_USER_ID,
        &[Action::View, Action::Edit, Action::Delete],
        "sample-editor",
    )
    .await;
    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        PRIVATE_CATEGORY,
        UNKNOWN_USER_ID,
        &[Action::View, Action::Create, Action::Edit],
        "unknown-editor-creator",
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
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(CONFLICT_SLUG)),
    );
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "restore conflict target",
            "title": "Restore Conflict Target",
            "alt_title": null,
            "slug": CONFLICT_SLUG,
            "layout": "wikidot",
            "revision_comments": "create restore conflict target",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

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

    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let before_source_denial =
        snapshot_page_action_mutation_state(&runner, site_id, page.page_id).await;
    let error = run_endpoint_err!(
        runner,
        page_restore,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "slug": CONFLICT_SLUG,
            "revision_comments": "blocked restore without delete action",
            "user_id": UNKNOWN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);
    assert_eq!(
        snapshot_page_action_mutation_state(&runner, site_id, page.page_id).await,
        before_source_denial,
        "restore source denial must precede destination conflict checks and preserve state",
    );

    set_mutation_request_context(
        &mut runner,
        SAMPLE_USER_ID,
        site_id,
        Reference::Id(page.page_id),
    );
    let before_destination_denial =
        snapshot_page_action_mutation_state(&runner, site_id, page.page_id).await;

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
    assert_eq!(
        snapshot_page_action_mutation_state(&runner, site_id, page.page_id).await,
        before_destination_denial,
        "restore destination denial must preserve page, revision, category, audit, and text-block state",
    );

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
    let restored = run_endpoint!(
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
    let restored =
        serde_json::to_value(restored).expect("explicit restore output should serialize");
    assert_eq!(restored["slug"], EXPLICIT_DESTINATION_SLUG);
    assert_eq!(restored["revision_number"], 2);
    assert_eq!(restored["parser_errors"], json!([]));
    let restored_revision_id = restored["revision_id"]
        .as_i64()
        .expect("explicit restore output should include a revision ID");

    let destination_category = CategoryService::get(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed(DESTINATION_CATEGORY)),
    )
    .await
    .expect("explicit restore destination category should exist");
    let restored_page = PageService::get(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed(EXPLICIT_DESTINATION_SLUG)),
    )
    .await
    .expect("explicit restore destination slug should resolve");
    assert_eq!(restored_page.page_id, explicit_page.page_id);
    assert_eq!(restored_page.slug, EXPLICIT_DESTINATION_SLUG);
    assert_eq!(
        restored_page.page_category_id,
        destination_category.category_id
    );
    assert_eq!(restored_page.latest_revision_id, Some(restored_revision_id));

    let restored_revision =
        PageRevisionService::get_latest(runner.context(), site_id, explicit_page.page_id)
            .await
            .expect("explicit restore revision should be latest");
    assert_eq!(restored_revision.revision_id, restored_revision_id);
    assert_eq!(restored_revision.revision_type, PageRevisionType::Undelete);
    assert_eq!(restored_revision.slug, EXPLICIT_DESTINATION_SLUG);

    assert!(
        PageService::get_optional(
            runner.context(),
            site_id,
            Reference::Slug(Cow::Borrowed(EXPLICIT_PAGE_SLUG)),
        )
        .await
        .expect("old explicit restore slug lookup should succeed")
        .is_none(),
        "old explicit restore slug must no longer resolve",
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
        blob_upload,
        json!({
            "user_id": UNKNOWN_USER_ID,
            "blob_size": 0,
            "scope": "page",
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

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

    let audit_count = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("file.undelete"))
        .filter(AuditLogColumn::ExtraId1.eq(restore_file.file_id))
        .count(runner.context().transaction())
        .await
        .expect("the denied file restore audit count should succeed");
    assert_eq!(audit_count, 0, "denied restores must not be audited");

    let transaction = runner
        .context()
        .transaction()
        .begin()
        .await
        .expect("the file restore savepoint should begin");
    let ctx =
        ServiceContext::new(runner.state(), &transaction).with_request(RequestContext {
            user_id: Some(SAMPLE_USER_ID),
            site_id: Some(site_id),
            page_reference: Some(Reference::Id(page.page_id)),
            ..Default::default()
        });
    let transactional_restore = deepwell::endpoints::all::file_restore(
        &ctx,
        common::make_params(json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file_id": restore_file.file_id,
            "new_page": destination_page.page_id,
            "revision_comments": "transactional file restore",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        })),
    )
    .await
    .expect("the transactional file restore should succeed");
    let transactional_event = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("file.undelete"))
        .filter(AuditLogColumn::ExtraId1.eq(restore_file.file_id))
        .one(&transaction)
        .await
        .expect("the transactional file restore audit lookup should succeed")
        .expect("the transactional file restore should emit an audit event");
    assert_eq!(transactional_event.page_id, Some(destination_page.page_id));
    assert_eq!(
        transactional_event.extra_id_2,
        Some(transactional_restore.file_revision_id)
    );
    let transactional_file = file::Entity::find_by_id(restore_file.file_id)
        .one(&transaction)
        .await
        .expect("the transactional restored file lookup should succeed")
        .expect("the transactional restored file should exist");
    assert_eq!(transactional_file.page_id, destination_page.page_id);
    assert!(transactional_file.deleted_at.is_none());

    transaction
        .rollback()
        .await
        .expect("the file restore savepoint should roll back");
    let rolled_back_file = file::Entity::find_by_id(restore_file.file_id)
        .one(runner.context().transaction())
        .await
        .expect("the rolled-back file lookup should succeed")
        .expect("the rolled-back file should exist");
    assert_eq!(rolled_back_file.page_id, page.page_id);
    assert!(
        rolled_back_file.deleted_at.is_some(),
        "the file restore state update must roll back"
    );
    let audit_count = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("file.undelete"))
        .filter(AuditLogColumn::ExtraId1.eq(restore_file.file_id))
        .count(runner.context().transaction())
        .await
        .expect("the rolled-back file restore audit count should succeed");
    assert_eq!(
        audit_count, 0,
        "the file restore audit event must roll back with the state update"
    );

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

    let events = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("file.undelete"))
        .filter(AuditLogColumn::ExtraId1.eq(restore_file.file_id))
        .all(runner.context().transaction())
        .await
        .expect("the successful file restore audit lookup should succeed");
    assert_eq!(events.len(), 1, "a successful restore must be audited once");
    let event = &events[0];
    assert_eq!(event.user_id, Some(SAMPLE_USER_ID));
    assert_eq!(event.site_id, Some(site_id));
    assert_eq!(event.page_id, Some(restored.page_id));
    assert_eq!(event.extra_id_1, Some(restored.file_id));
    assert_eq!(event.extra_id_2, Some(restored.file_revision_id));
    assert_eq!(event.ip_address, common::IP_ADDRESS.to_string());

    let error = run_endpoint_err!(
        runner,
        file_restore,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "file_id": restore_file.file_id,
            "revision_comments": "repeated file restore",
            "user_id": SAMPLE_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(error, ErrorType::FileNotDeleted);
    let audit_count = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("file.undelete"))
        .filter(AuditLogColumn::ExtraId1.eq(restore_file.file_id))
        .count(runner.context().transaction())
        .await
        .expect("the repeated file restore audit count should succeed");
    assert_eq!(audit_count, 1, "a repeated restore must not be audited");

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

struct PagePendingBlobFixture {
    pending_blob_id: String,
    s3_path: String,
}

fn page_pending_blob_model(
    pending_blob_id: String,
    s3_path: String,
    user_id: i64,
    site_id: Option<i64>,
    page_id: Option<i64>,
    expected_length: usize,
) -> blob_pending::ActiveModel {
    let created_at = OffsetDateTime::now_utc();
    blob_pending::ActiveModel {
        external_id: Set(pending_blob_id),
        created_by: Set(user_id),
        created_at: Set(created_at),
        expires_at: Set(created_at + Duration::minutes(5)),
        expected_length: Set(expected_length
            .try_into()
            .expect("pending blob fixture length should fit in i64")),
        s3_path: Set(s3_path),
        s3_hash: Set(None),
        presign_url: Set("https://uploads.example.test/issue-1062".to_owned()),
        site_id: Set(site_id),
        page_id: Set(page_id),
        content_type_label: Set(None),
        content_type_description: Set(None),
    }
}

async fn create_request_pending_blob_fixture(
    runner: &TestRunner,
    user_id: i64,
    site_id: Option<i64>,
    page_id: Option<i64>,
) -> PagePendingBlobFixture {
    let pending_blob_id = cuid();
    let s3_path = format!("uploads/{pending_blob_id}");
    let data = format!("issue 1062 rejected upload {pending_blob_id}").into_bytes();
    let response = runner
        .state()
        .s3_files_bucket
        .put_object(&s3_path, &data)
        .await
        .expect("request pending blob fixture should be uploaded");
    assert_eq!(response.status_code(), 200);
    page_pending_blob_model(
        pending_blob_id.clone(),
        s3_path.clone(),
        user_id,
        site_id,
        page_id,
        data.len(),
    )
    .insert(runner.context().transaction())
    .await
    .expect("request pending blob fixture should be inserted");
    PagePendingBlobFixture {
        pending_blob_id,
        s3_path,
    }
}

async fn create_committed_page_pending_blob_fixture(
    runner: &TestRunner,
    user_id: i64,
    site_id: i64,
    page_id: i64,
) -> (PagePendingBlobFixture, Vec<u8>) {
    let pending_blob_id = cuid();
    let data = format!("issue 1062 page-owned upload {pending_blob_id}").into_bytes();
    create_committed_page_pending_blob_with_data_fixture(
        runner,
        user_id,
        site_id,
        page_id,
        pending_blob_id,
        data,
    )
    .await
}

async fn create_committed_page_pending_blob_with_data_fixture(
    runner: &TestRunner,
    user_id: i64,
    site_id: i64,
    page_id: i64,
    pending_blob_id: String,
    data: Vec<u8>,
) -> (PagePendingBlobFixture, Vec<u8>) {
    let s3_path = format!("uploads/{pending_blob_id}");
    let response = runner
        .state()
        .s3_files_bucket
        .put_object(&s3_path, &data)
        .await
        .expect("pending blob fixture should be uploaded");
    if response.status_code() != 200 {
        let cleanup = runner.state().s3_files_bucket.delete_object(&s3_path).await;
        panic!(
            "pending blob fixture upload returned HTTP {}; temporary cleanup: {cleanup:?}",
            response.status_code()
        );
    }
    let inserted = page_pending_blob_model(
        pending_blob_id.clone(),
        s3_path.clone(),
        user_id,
        Some(site_id),
        Some(page_id),
        data.len(),
    )
    .insert(&runner.state().database)
    .await;
    if let Err(error) = inserted {
        let cleanup = runner.state().s3_files_bucket.delete_object(&s3_path).await;
        panic!(
            "committed pending blob fixture should be inserted: {error:?}; temporary cleanup: {cleanup:?}"
        );
    }
    (
        PagePendingBlobFixture {
            pending_blob_id,
            s3_path,
        },
        data,
    )
}

async fn create_prefinalized_empty_page_blob_fixture(
    runner: &TestRunner,
    user_id: i64,
    site_id: i64,
    page_id: i64,
) -> String {
    let pending_blob_id = cuid();
    let mut pending = page_pending_blob_model(
        pending_blob_id.clone(),
        format!("uploads/{pending_blob_id}"),
        user_id,
        Some(site_id),
        Some(page_id),
        0,
    );
    pending.s3_hash = Set(Some(EMPTY_BLOB_HASH.to_vec()));
    pending.content_type_label = Set(Some("empty".to_owned()));
    pending.content_type_description = Set(Some("empty".to_owned()));
    pending
        .insert(runner.context().transaction())
        .await
        .expect("pre-finalized empty pending blob fixture should be inserted");
    pending_blob_id
}

#[tokio::test]
async fn file_edit_public_endpoint_audits_success_once_and_not_denial() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    let page = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": "home"}),
    )
    .expect("seeded test home should exist");
    let fixture_name = format!("file-edit-audit-{}.txt", cuid());
    let file_id =
        create_empty_file_fixture(&runner, site.site_id, page.page_id, &fixture_name)
            .await;
    let file = FileRevisionService::get_latest(
        runner.context(),
        site.site_id,
        page.page_id,
        file_id,
    )
    .await
    .expect("file edit audit fixture revision should exist");
    let supplied_ip = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 48));

    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site.site_id,
        Reference::Id(page.page_id),
    );
    let error = run_endpoint_err!(
        runner,
        file_edit,
        json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "file_id": file_id,
            "last_revision_id": file.revision_id,
            "revision_comments": "file edit audit denial",
            "user_id": UNKNOWN_USER_ID,
            "name": format!("denied-{fixture_name}"),
            "ip_address": supplied_ip,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);
    assert_eq!(
        AuditLogTable::find()
            .filter(AuditLogColumn::EventType.eq("file.edit"))
            .filter(AuditLogColumn::ExtraId1.eq(file_id))
            .count(runner.context().transaction())
            .await
            .expect("denied file-edit audit count should succeed"),
        0,
        "denied file edits must not be audited",
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(page.page_id),
    );
    let edited = run_endpoint!(
        runner,
        file_edit,
        json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "file_id": file_id,
            "last_revision_id": file.revision_id,
            "revision_comments": "file edit audit success",
            "user_id": ADMIN_USER_ID,
            "name": format!("edited-{fixture_name}"),
            "ip_address": supplied_ip,
        }),
    )
    .expect("changed file edit should create a revision");

    let events = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("file.edit"))
        .filter(AuditLogColumn::ExtraId1.eq(file_id))
        .all(runner.context().transaction())
        .await
        .expect("successful file-edit audit lookup should succeed");
    assert_eq!(events.len(), 1, "successful file edit should audit once");
    let event = &events[0];
    assert_eq!(event.user_id, Some(ADMIN_USER_ID));
    assert_eq!(event.site_id, Some(site.site_id));
    assert_eq!(event.page_id, Some(page.page_id));
    assert_eq!(event.extra_id_1, Some(file_id));
    assert_eq!(event.extra_id_2, Some(edited.file_revision_id));
    assert_eq!(event.ip_address, supplied_ip.to_string());
}

#[tokio::test]
async fn file_edit_semantic_no_op_emits_no_audit_event() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    let page = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": "home"}),
    )
    .expect("seeded test home should exist");
    let fixture_name = format!("file-edit-no-op-{}.txt", cuid());
    let file_id =
        create_empty_file_fixture(&runner, site.site_id, page.page_id, &fixture_name)
            .await;
    let file = FileRevisionService::get_latest(
        runner.context(),
        site.site_id,
        page.page_id,
        file_id,
    )
    .await
    .expect("file edit no-op fixture revision should exist");
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(page.page_id),
    );

    let edited = run_endpoint!(
        runner,
        file_edit,
        json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "file_id": file_id,
            "last_revision_id": file.revision_id,
            "revision_comments": "semantically empty file edit",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(
        edited.is_none(),
        "semantic no-op should not create a revision"
    );
    assert_eq!(
        AuditLogTable::find()
            .filter(AuditLogColumn::EventType.eq("file.edit"))
            .filter(AuditLogColumn::ExtraId1.eq(file_id))
            .count(runner.context().transaction())
            .await
            .expect("file-edit no-op audit count should succeed"),
        0,
        "semantic no-op must not be audited",
    );
}

#[tokio::test]
async fn file_edit_revision_and_audit_roll_back_together() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    let page = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": "home"}),
    )
    .expect("seeded test home should exist");
    let fixture_name = format!("file-edit-rollback-{}.txt", cuid());
    let file_id =
        create_empty_file_fixture(&runner, site.site_id, page.page_id, &fixture_name)
            .await;
    let file = FileRevisionService::get_latest(
        runner.context(),
        site.site_id,
        page.page_id,
        file_id,
    )
    .await
    .expect("file edit rollback fixture revision should exist");

    let transaction = runner
        .context()
        .transaction()
        .begin()
        .await
        .expect("file-edit rollback savepoint should begin");
    let ctx =
        ServiceContext::new(runner.state(), &transaction).with_request(RequestContext {
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site.site_id),
            page_reference: Some(Reference::Id(page.page_id)),
            ..Default::default()
        });
    let edited = deepwell::endpoints::all::file_edit(
        &ctx,
        common::make_params(json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "file_id": file_id,
            "last_revision_id": file.revision_id,
            "revision_comments": "transactional file edit",
            "user_id": ADMIN_USER_ID,
            "name": format!("edited-{fixture_name}"),
            "ip_address": common::IP_ADDRESS,
        })),
    )
    .await
    .expect("transactional file edit should succeed")
    .expect("transactional file edit should create a revision");

    assert!(
        FileRevisionTable::find_by_id(edited.file_revision_id)
            .one(&transaction)
            .await
            .expect("transactional file revision lookup should succeed")
            .is_some()
    );
    assert_eq!(
        AuditLogTable::find()
            .filter(AuditLogColumn::EventType.eq("file.edit"))
            .filter(AuditLogColumn::ExtraId1.eq(file_id))
            .filter(AuditLogColumn::ExtraId2.eq(edited.file_revision_id))
            .count(&transaction)
            .await
            .expect("transactional file-edit audit lookup should succeed"),
        1,
    );
    drop(ctx);
    transaction
        .rollback()
        .await
        .expect("file-edit savepoint should roll back");

    assert!(
        FileRevisionTable::find_by_id(edited.file_revision_id)
            .one(runner.context().transaction())
            .await
            .expect("rolled-back file revision lookup should succeed")
            .is_none()
    );
    assert_eq!(
        AuditLogTable::find()
            .filter(AuditLogColumn::EventType.eq("file.edit"))
            .filter(AuditLogColumn::ExtraId1.eq(file_id))
            .filter(AuditLogColumn::ExtraId2.eq(edited.file_revision_id))
            .count(runner.context().transaction())
            .await
            .expect("rolled-back file-edit audit lookup should succeed"),
        0,
    );
    let current = FileRevisionService::get_latest(
        runner.context(),
        site.site_id,
        page.page_id,
        file_id,
    )
    .await
    .expect("rolled-back file edit fixture revision should remain");
    assert_eq!(current.revision_id, file.revision_id);
    assert_eq!(current.name, fixture_name);
}

#[tokio::test]
async fn file_rollback_public_endpoint_audits_only_created_revisions() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    let page = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": "home"}),
    )
    .expect("seeded test home should exist");
    let fixture_name = format!("file-rollback-audit-{}.txt", cuid());
    let file_id =
        create_empty_file_fixture(&runner, site.site_id, page.page_id, &fixture_name)
            .await;
    let initial = FileRevisionService::get_latest(
        runner.context(),
        site.site_id,
        page.page_id,
        file_id,
    )
    .await
    .expect("file rollback audit fixture revision should exist");
    let edited_name = format!("edited-{fixture_name}");
    let supplied_ip = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 49));

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(page.page_id),
    );
    let edited = run_endpoint!(
        runner,
        file_edit,
        json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "file_id": file_id,
            "last_revision_id": initial.revision_id,
            "revision_comments": "prepare file rollback audit",
            "user_id": ADMIN_USER_ID,
            "name": edited_name,
            "ip_address": supplied_ip,
        }),
    )
    .expect("file edit should prepare rollback audit fixture");

    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site.site_id,
        Reference::Id(page.page_id),
    );
    let error = run_endpoint_err!(
        runner,
        file_rollback,
        json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "file": edited_name,
            "last_revision_id": edited.file_revision_id,
            "revision_number": initial.revision_number,
            "revision_comments": "denied file rollback audit",
            "user_id": UNKNOWN_USER_ID,
            "ip_address": supplied_ip,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);
    assert_eq!(
        AuditLogTable::find()
            .filter(AuditLogColumn::EventType.eq("file.rollback"))
            .filter(AuditLogColumn::ExtraId1.eq(file_id))
            .count(runner.context().transaction())
            .await
            .expect("denied file-rollback audit count should succeed"),
        0,
        "denied file rollbacks must not be audited",
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(page.page_id),
    );
    let rolled_back = run_endpoint!(
        runner,
        file_rollback,
        json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "file": edited_name,
            "last_revision_id": edited.file_revision_id,
            "revision_number": initial.revision_number,
            "revision_comments": "successful file rollback audit",
            "user_id": ADMIN_USER_ID,
            "ip_address": supplied_ip,
        }),
    )
    .expect("changed file rollback should create a revision");

    let events = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("file.rollback"))
        .filter(AuditLogColumn::ExtraId1.eq(file_id))
        .all(runner.context().transaction())
        .await
        .expect("successful file-rollback audit lookup should succeed");
    assert_eq!(
        events.len(),
        1,
        "successful file rollback should audit once"
    );
    let event = &events[0];
    assert_eq!(event.user_id, Some(ADMIN_USER_ID));
    assert_eq!(event.site_id, Some(site.site_id));
    assert_eq!(event.page_id, Some(page.page_id));
    assert_eq!(event.extra_id_1, Some(file_id));
    assert_eq!(event.extra_id_2, Some(rolled_back.file_revision_id));
    assert_eq!(event.extra_number, Some(initial.revision_number));
    assert_eq!(event.ip_address, supplied_ip.to_string());

    let error = run_endpoint_err!(
        runner,
        file_rollback,
        json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "file": fixture_name,
            "last_revision_id": edited.file_revision_id,
            "revision_number": initial.revision_number,
            "revision_comments": "stale file rollback audit",
            "user_id": ADMIN_USER_ID,
            "ip_address": supplied_ip,
        }),
    );
    assert_contains_error!(error, ErrorType::NotLatestRevisionId);

    let no_op = run_endpoint!(
        runner,
        file_rollback,
        json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "file": fixture_name,
            "last_revision_id": rolled_back.file_revision_id,
            "revision_number": initial.revision_number,
            "revision_comments": "no-op file rollback audit",
            "user_id": ADMIN_USER_ID,
            "ip_address": supplied_ip,
        }),
    );
    assert!(no_op.is_none(), "unchanged file rollback should be a no-op");
    assert_eq!(
        AuditLogTable::find()
            .filter(AuditLogColumn::EventType.eq("file.rollback"))
            .filter(AuditLogColumn::ExtraId1.eq(file_id))
            .count(runner.context().transaction())
            .await
            .expect("stale and no-op file-rollback audit count should succeed"),
        1,
        "stale and no-op file rollbacks must not be audited",
    );
}

#[tokio::test]
async fn file_rollback_revision_and_audit_roll_back_together() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    let page = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": "home"}),
    )
    .expect("seeded test home should exist");
    let fixture_name = format!("file-rollback-transaction-{}.txt", cuid());
    let file_id =
        create_empty_file_fixture(&runner, site.site_id, page.page_id, &fixture_name)
            .await;
    let initial = FileRevisionService::get_latest(
        runner.context(),
        site.site_id,
        page.page_id,
        file_id,
    )
    .await
    .expect("file rollback transaction fixture revision should exist");
    let edited_name = format!("edited-{fixture_name}");
    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(page.page_id),
    );
    let edited = run_endpoint!(
        runner,
        file_edit,
        json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "file_id": file_id,
            "last_revision_id": initial.revision_id,
            "revision_comments": "prepare transactional file rollback",
            "user_id": ADMIN_USER_ID,
            "name": edited_name,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("file edit should prepare transactional rollback fixture");

    let transaction = runner
        .context()
        .transaction()
        .begin()
        .await
        .expect("file-rollback savepoint should begin");
    let ctx =
        ServiceContext::new(runner.state(), &transaction).with_request(RequestContext {
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site.site_id),
            page_reference: Some(Reference::Id(page.page_id)),
            ..Default::default()
        });
    let rolled_back = deepwell::endpoints::all::file_rollback(
        &ctx,
        common::make_params(json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "file": edited_name,
            "last_revision_id": edited.file_revision_id,
            "revision_number": initial.revision_number,
            "revision_comments": "transactional file rollback",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        })),
    )
    .await
    .expect("transactional file rollback should succeed")
    .expect("transactional file rollback should create a revision");

    assert!(
        FileRevisionTable::find_by_id(rolled_back.file_revision_id)
            .one(&transaction)
            .await
            .expect("transactional file revision lookup should succeed")
            .is_some()
    );
    assert_eq!(
        AuditLogTable::find()
            .filter(AuditLogColumn::EventType.eq("file.rollback"))
            .filter(AuditLogColumn::ExtraId1.eq(file_id))
            .filter(AuditLogColumn::ExtraId2.eq(rolled_back.file_revision_id))
            .count(&transaction)
            .await
            .expect("transactional file-rollback audit lookup should succeed"),
        1,
    );
    drop(ctx);
    transaction
        .rollback()
        .await
        .expect("file-rollback savepoint should roll back");

    assert!(
        FileRevisionTable::find_by_id(rolled_back.file_revision_id)
            .one(runner.context().transaction())
            .await
            .expect("rolled-back file revision lookup should succeed")
            .is_none()
    );
    assert_eq!(
        AuditLogTable::find()
            .filter(AuditLogColumn::EventType.eq("file.rollback"))
            .filter(AuditLogColumn::ExtraId1.eq(file_id))
            .filter(AuditLogColumn::ExtraId2.eq(rolled_back.file_revision_id))
            .count(runner.context().transaction())
            .await
            .expect("rolled-back file-rollback audit lookup should succeed"),
        0,
    );
    let current = FileRevisionService::get_latest(
        runner.context(),
        site.site_id,
        page.page_id,
        file_id,
    )
    .await
    .expect("rolled-back file rollback fixture revision should remain");
    assert_eq!(current.revision_id, edited.file_revision_id);
    assert_eq!(current.name, edited_name);
}

#[tokio::test]
async fn file_create_public_endpoint_audits_success_once_and_not_denial() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    let page = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": "home"}),
    )
    .expect("seeded test home should exist");
    let pending_blob_id = create_prefinalized_empty_page_blob_fixture(
        &runner,
        ADMIN_USER_ID,
        site.site_id,
        page.page_id,
    )
    .await;
    let supplied_ip = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 47));

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(page.page_id),
    );
    let created = run_endpoint!(
        runner,
        file_create,
        json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "name": format!("audit-success-{pending_blob_id}.txt"),
            "uploaded_blob_id": pending_blob_id,
            "revision_comments": "file create audit success",
            "user_id": ADMIN_USER_ID,
            "ip_address": supplied_ip,
        }),
    );

    let events = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("file.create"))
        .filter(AuditLogColumn::ExtraId1.eq(created.file_id))
        .filter(AuditLogColumn::ExtraId2.eq(created.file_revision_id))
        .all(runner.context().transaction())
        .await
        .expect("file-create audit lookup should succeed");
    assert_eq!(events.len(), 1, "successful file create should audit once");
    let event = &events[0];
    assert_eq!(event.user_id, Some(ADMIN_USER_ID));
    assert_eq!(event.site_id, Some(site.site_id));
    assert_eq!(event.page_id, Some(page.page_id));
    assert_eq!(event.extra_id_1, Some(created.file_id));
    assert_eq!(event.extra_id_2, Some(created.file_revision_id));
    assert_eq!(event.ip_address, supplied_ip.to_string());

    let event_count_before_denial = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("file.create"))
        .count(runner.context().transaction())
        .await
        .expect("file-create audit count before denial should succeed");
    set_mutation_request_context(
        &mut runner,
        UNKNOWN_USER_ID,
        site.site_id,
        Reference::Id(page.page_id),
    );
    let error = run_endpoint_err!(
        runner,
        file_create,
        json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "name": "audit-denied.txt",
            "uploaded_blob_id": "not-used-before-permission-denial",
            "revision_comments": "file create audit denial",
            "user_id": UNKNOWN_USER_ID,
            "ip_address": supplied_ip,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);
    let event_count = AuditLogTable::find()
        .filter(AuditLogColumn::EventType.eq("file.create"))
        .count(runner.context().transaction())
        .await
        .expect("file-create audit count after denial should succeed");
    assert_eq!(
        event_count, event_count_before_denial,
        "denied file create should not be audited",
    );
}

#[tokio::test]
async fn file_create_file_revision_and_audit_roll_back_together() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    let page = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": "home"}),
    )
    .expect("seeded test home should exist");
    let pending_blob_id = create_prefinalized_empty_page_blob_fixture(
        &runner,
        ADMIN_USER_ID,
        site.site_id,
        page.page_id,
    )
    .await;

    let transaction = runner
        .context()
        .transaction()
        .begin()
        .await
        .expect("file-create rollback savepoint should begin");
    let ctx =
        ServiceContext::new(runner.state(), &transaction).with_request(RequestContext {
            user_id: Some(ADMIN_USER_ID),
            site_id: Some(site.site_id),
            page_reference: Some(Reference::Id(page.page_id)),
            ..Default::default()
        });
    let created = deepwell::endpoints::all::file_create(
        &ctx,
        common::make_params(json!({
            "site_id": site.site_id,
            "page_id": page.page_id,
            "name": format!("audit-rollback-{pending_blob_id}.txt"),
            "uploaded_blob_id": pending_blob_id,
            "revision_comments": "file create audit rollback",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        })),
    )
    .await
    .expect("transactional file create should succeed");

    assert!(
        file::Entity::find_by_id(created.file_id)
            .one(&transaction)
            .await
            .expect("transactional file lookup should succeed")
            .is_some()
    );
    assert!(
        FileRevisionTable::find_by_id(created.file_revision_id)
            .one(&transaction)
            .await
            .expect("transactional file revision lookup should succeed")
            .is_some()
    );
    assert_eq!(
        AuditLogTable::find()
            .filter(AuditLogColumn::EventType.eq("file.create"))
            .filter(AuditLogColumn::ExtraId1.eq(created.file_id))
            .filter(AuditLogColumn::ExtraId2.eq(created.file_revision_id))
            .count(&transaction)
            .await
            .expect("transactional file-create audit lookup should succeed"),
        1,
    );
    drop(ctx);
    transaction
        .rollback()
        .await
        .expect("file-create savepoint should roll back");

    assert!(
        file::Entity::find_by_id(created.file_id)
            .one(runner.context().transaction())
            .await
            .expect("rolled-back file lookup should succeed")
            .is_none()
    );
    assert!(
        FileRevisionTable::find_by_id(created.file_revision_id)
            .one(runner.context().transaction())
            .await
            .expect("rolled-back file revision lookup should succeed")
            .is_none()
    );
    assert_eq!(
        AuditLogTable::find()
            .filter(AuditLogColumn::EventType.eq("file.create"))
            .filter(AuditLogColumn::ExtraId1.eq(created.file_id))
            .filter(AuditLogColumn::ExtraId2.eq(created.file_revision_id))
            .count(runner.context().transaction())
            .await
            .expect("rolled-back file-create audit lookup should succeed"),
        0,
    );
}

async fn cleanup_committed_page_pending_blob_fixture(
    state: &deepwell::api::ServerState,
    fixture: &PagePendingBlobFixture,
    data: &[u8],
) -> std::result::Result<(), String> {
    let mut failures = Vec::new();
    if let Err(error) = BlobPendingTable::delete_by_id(&fixture.pending_blob_id)
        .exec(&state.database)
        .await
    {
        failures.push(format!("pending row cleanup failed: {error:?}"));
    }
    for (label, path) in [
        ("temporary", fixture.s3_path.clone()),
        (
            "permanent",
            blob_hash_to_hex(&sha512_hash(data)).to_string(),
        ),
    ] {
        match state.s3_files_bucket.delete_object(&path).await {
            Ok(response) if response.status_code() == 204 => {}
            Ok(response) => failures.push(format!(
                "{label} object cleanup returned HTTP {}",
                response.status_code()
            )),
            Err(error) => {
                failures.push(format!("{label} object cleanup failed: {error:?}"))
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}

async fn set_context_and_assert_page_file_absent(
    runner: &mut TestRunner,
    user_id: i64,
    site_id: i64,
    page_id: i64,
    name: &str,
) {
    set_mutation_request_context(runner, user_id, site_id, Reference::Id(page_id));
    let files = run_endpoint!(
        runner,
        page_get_files,
        json!({
            "site_id": site_id,
            "page_id": page_id,
            "deleted": false,
        }),
    );
    assert!(
        files.iter().all(|file| file.name != name),
        "failed file mutation must not expose {name:?} through page_get_files"
    );
}

async fn cancel_request_pending_blob(
    runner: &mut TestRunner,
    user_id: i64,
    pending_blob_id: &str,
) {
    runner.set_request_context(RequestContext {
        user_id: Some(user_id),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        blob_cancel,
        json!({
            "user_id": user_id,
            "pending_blob_id": pending_blob_id,
        }),
    );
    assert!(
        BlobPendingTable::find_by_id(pending_blob_id)
            .one(runner.context().transaction())
            .await
            .expect("cancelled pending blob lookup should succeed")
            .is_none(),
        "blob_cancel should remove pending state"
    );
}

async fn cancel_request_pending_blob_fixture(
    runner: &mut TestRunner,
    user_id: i64,
    fixture: &PagePendingBlobFixture,
) {
    cancel_request_pending_blob(runner, user_id, &fixture.pending_blob_id).await;
    let temporary = runner
        .state()
        .s3_files_bucket
        .get_object(&fixture.s3_path)
        .await
        .expect("cancelled temporary upload lookup should succeed");
    assert_eq!(
        temporary.status_code(),
        404,
        "blob_cancel should remove the temporary upload"
    );
}

#[tokio::test]
async fn file_create_commits_only_its_actor_and_route_owned_pending_blob() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site;
    let page = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": "home"}),
    )
    .expect("seeded test home should exist");
    let other_page = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site.site_id, "page": "nav:top"}),
    )
    .expect("seeded test navigation page should exist");
    let other_site =
        run_endpoint!(runner, site_get, json!({"site": "sandbox-for-codex"}))
            .expect("seeded sandbox site should exist")
            .site;
    let other_site_page = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": other_site.site_id, "page": "start"}),
    )
    .expect("seeded sandbox start page should exist");

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(page.page_id),
    );
    let scoped = run_endpoint!(
        runner,
        blob_upload,
        json!({
            "user_id": ADMIN_USER_ID,
            "blob_size": 0,
            "scope": "page",
        }),
    );
    let scoped_row = BlobPendingTable::find_by_id(&scoped.pending_blob_id)
        .one(runner.context().transaction())
        .await
        .expect("page-scoped pending lookup should succeed")
        .expect("page-scoped pending row should exist");
    assert_eq!(scoped_row.created_by, ADMIN_USER_ID);
    assert_eq!(scoped_row.site_id, Some(site.site_id));
    assert_eq!(scoped_row.page_id, Some(page.page_id));
    cancel_request_pending_blob(&mut runner, ADMIN_USER_ID, &scoped.pending_blob_id)
        .await;

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site.site_id,
        Reference::Id(page.page_id),
    );
    let unscoped = run_endpoint!(
        runner,
        blob_upload,
        json!({
            "user_id": ADMIN_USER_ID,
            "blob_size": 0,
            "scope": "unscoped",
        }),
    );
    let unscoped_row = BlobPendingTable::find_by_id(&unscoped.pending_blob_id)
        .one(runner.context().transaction())
        .await
        .expect("unscoped pending lookup should succeed")
        .expect("unscoped pending row should exist");
    assert_eq!(unscoped_row.site_id, None);
    assert_eq!(unscoped_row.page_id, None);
    cancel_request_pending_blob(&mut runner, ADMIN_USER_ID, &unscoped.pending_blob_id)
        .await;

    let (owned, expected_data) = create_committed_page_pending_blob_fixture(
        &runner,
        ADMIN_USER_ID,
        site.site_id,
        page.page_id,
    )
    .await;
    let verification = AssertUnwindSafe(async {
        let owned_name = format!("{}.txt", owned.pending_blob_id);
        set_mutation_request_context(
            &mut runner,
            ADMIN_USER_ID,
            site.site_id,
            Reference::Id(page.page_id),
        );
        let created = run_endpoint!(
            runner,
            file_create,
            json!({
                "site_id": site.site_id,
                "page_id": page.page_id,
                "name": owned_name,
                "uploaded_blob_id": owned.pending_blob_id,
                "revision_comments": "issue 1062 owned upload",
                "user_id": ADMIN_USER_ID,
                "ip_address": common::IP_ADDRESS,
            }),
        );
        assert!(created.file_id > 0);
        let moved_pending = BlobPendingTable::find_by_id(&owned.pending_blob_id)
            .one(&runner.state().database)
            .await
            .expect("moved pending descriptor lookup should succeed")
            .expect("moved pending blob should remain available for retry");
        assert_eq!(
            moved_pending.content_type_label.as_deref(),
            Some("ASCII text")
        );
        assert_eq!(
            moved_pending.content_type_description.as_deref(),
            Some("ASCII text, with no line terminators")
        );
        let created_revision = FileRevisionService::get_latest(
            runner.context(),
            site.site_id,
            page.page_id,
            created.file_id,
        )
        .await
        .expect("created file revision should be readable");
        assert_eq!(
            created_revision.content_type_label.as_deref(),
            Some("ASCII text")
        );
        assert_eq!(
            created_revision.content_type_description.as_deref(),
            Some("ASCII text, with no line terminators")
        );
        let files = run_endpoint!(
            runner,
            page_get_files,
            json!({
                "site_id": site.site_id,
                "page_id": page.page_id,
                "deleted": false,
            }),
        );
        let listed = files
            .iter()
            .find(|file| file.file_id == created.file_id)
            .expect("page_get_files should expose the committed file relation");
        assert_eq!(listed.name, owned_name);
        assert_eq!(listed.page_id, page.page_id);
        assert_eq!(listed.size, expected_data.len() as i64);
        let viewed = run_endpoint!(
            runner,
            file_get,
            json!({
                "site_id": site.site_id,
                "page_id": page.page_id,
                "file": created.file_id,
                "details": {"data": true},
            }),
        )
        .expect("file_get should expose the committed file");
        assert_eq!(viewed.file_id, created.file_id);
        assert_eq!(
            viewed.data.as_ref().map(|data| data.as_ref()),
            Some(expected_data.as_slice())
        );

        let other_actor = create_request_pending_blob_fixture(
            &runner,
            SAMPLE_USER_ID,
            Some(site.site_id),
            Some(page.page_id),
        )
        .await;
        let other_actor_name = "issue-1062-other-actor.txt";
        set_mutation_request_context(
            &mut runner,
            ADMIN_USER_ID,
            site.site_id,
            Reference::Id(page.page_id),
        );
        let error = run_endpoint_err!(
            runner,
            file_create,
            json!({
                "site_id": site.site_id,
                "page_id": page.page_id,
                "name": other_actor_name,
                "uploaded_blob_id": other_actor.pending_blob_id,
                "revision_comments": "reject another actor pending upload",
                "user_id": ADMIN_USER_ID,
                "ip_address": common::IP_ADDRESS,
            }),
        );
        assert_contains_error!(error, ErrorType::BlobNotFound);
        assert!(
            !format!("{error:?}").contains(&other_actor.pending_blob_id),
            "ownership rejection must not reflect the pending blob token"
        );
        set_context_and_assert_page_file_absent(
            &mut runner,
            ADMIN_USER_ID,
            site.site_id,
            page.page_id,
            other_actor_name,
        )
        .await;
        cancel_request_pending_blob_fixture(&mut runner, SAMPLE_USER_ID, &other_actor)
            .await;

        for (target_site_id, target_page_id, label) in [
            (site.site_id, other_page.page_id, "cross-page"),
            (other_site.site_id, other_site_page.page_id, "cross-site"),
        ] {
            let misplaced = create_request_pending_blob_fixture(
                &runner,
                ADMIN_USER_ID,
                Some(site.site_id),
                Some(page.page_id),
            )
            .await;
            let name = format!("issue-1062-{label}.txt");
            set_mutation_request_context(
                &mut runner,
                ADMIN_USER_ID,
                target_site_id,
                Reference::Id(target_page_id),
            );
            let error = run_endpoint_err!(
                runner,
                file_create,
                json!({
                    "site_id": target_site_id,
                    "page_id": target_page_id,
                    "name": name,
                    "uploaded_blob_id": misplaced.pending_blob_id,
                    "revision_comments": format!("reject {label} pending upload"),
                    "user_id": ADMIN_USER_ID,
                    "ip_address": common::IP_ADDRESS,
                }),
            );
            assert_contains_error!(error, ErrorType::BlobNotFound);
            assert!(
                !format!("{error:?}").contains(&misplaced.pending_blob_id),
                "scope rejection must not reflect the pending blob token"
            );
            set_context_and_assert_page_file_absent(
                &mut runner,
                ADMIN_USER_ID,
                target_site_id,
                target_page_id,
                &name,
            )
            .await;
            cancel_request_pending_blob_fixture(&mut runner, ADMIN_USER_ID, &misplaced)
                .await;
        }

        let generic =
            create_request_pending_blob_fixture(&runner, ADMIN_USER_ID, None, None).await;
        let generic_name = "issue-1062-unscoped.txt";
        set_mutation_request_context(
            &mut runner,
            ADMIN_USER_ID,
            site.site_id,
            Reference::Id(page.page_id),
        );
        let error = run_endpoint_err!(
            runner,
            file_create,
            json!({
                "site_id": site.site_id,
                "page_id": page.page_id,
                "name": generic_name,
                "uploaded_blob_id": generic.pending_blob_id,
                "revision_comments": "reject unscoped pending upload",
                "user_id": ADMIN_USER_ID,
                "ip_address": common::IP_ADDRESS,
            }),
        );
        assert_contains_error!(error, ErrorType::BlobNotFound);
        assert!(
            !format!("{error:?}").contains(&generic.pending_blob_id),
            "unscoped rejection must not reflect the pending blob token"
        );
        set_context_and_assert_page_file_absent(
            &mut runner,
            ADMIN_USER_ID,
            site.site_id,
            page.page_id,
            generic_name,
        )
        .await;
        cancel_request_pending_blob_fixture(&mut runner, ADMIN_USER_ID, &generic).await;

        cancel_request_pending_blob(&mut runner, ADMIN_USER_ID, &owned.pending_blob_id)
            .await;
        set_mutation_request_context(
            &mut runner,
            ADMIN_USER_ID,
            site.site_id,
            Reference::Id(page.page_id),
        );
        let viewed_after_cancel = run_endpoint!(
            runner,
            file_get,
            json!({
                "site_id": site.site_id,
                "page_id": page.page_id,
                "file": created.file_id,
                "details": {"data": true},
            }),
        )
        .expect("blob_cancel must not remove permanent content-addressed file bytes");
        assert_eq!(
            viewed_after_cancel.data.as_ref().map(|data| data.as_ref()),
            Some(expected_data.as_slice())
        );
    })
    .catch_unwind()
    .await;

    let state = runner.state().clone();
    let teardown = AssertUnwindSafe(runner.teardown()).catch_unwind().await;
    let cleanup =
        cleanup_committed_page_pending_blob_fixture(&state, &owned, &expected_data).await;
    if let Err(payload) = verification {
        if let Err(error) = &cleanup {
            eprintln!("committed pending fixture cleanup after panic failed: {error}");
        }
        resume_unwind(payload);
    }
    if let Err(payload) = teardown {
        if let Err(error) = &cleanup {
            eprintln!(
                "committed pending fixture cleanup after teardown panic failed: {error}"
            );
        }
        resume_unwind(payload);
    }
    cleanup.expect("committed pending fixture cleanup should succeed");
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
    const PUBLIC_HTML_BYTES: &[u8] = b"public hosted HTML raw bytes";
    const PRIVATE_HTML_BYTES: &[u8] = b"private hosted HTML raw bytes";
    const DUPLICATE_HTML_BYTES: &[u8] = b"duplicate hosted HTML raw bytes";
    const TWO_VISIBLE_HTML_BYTES: &[u8] = b"two visible hosted HTML raw bytes";
    const OVERFLOW_HTML_BYTES: &[u8] = b"overflow hosted HTML raw bytes";

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
        Some(PRIVATE_HTML_BYTES),
    )
    .await;
    create_text_block_fixture(
        &runner,
        page.page_id,
        TextBlockType::Code,
        2,
        Some("secret-code"),
        "sentinel-private-code-block",
        None,
    )
    .await;
    create_text_block_fixture(
        &runner,
        page.page_id,
        TextBlockType::Html,
        2,
        None,
        "sentinel-private-duplicate-html-block",
        Some(DUPLICATE_HTML_BYTES),
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
    TextBlockService::add_blocks(
        runner.context(),
        public_page.page_id,
        TextBlockType::Html,
        &[TextBlock {
            text: std::str::from_utf8(PUBLIC_HTML_BYTES).unwrap(),
            text_type: None,
            mime: MIME_HTML,
            name: None,
        }],
    )
    .await
    .expect("public HTML block should be uploaded through add_blocks");
    create_text_block_fixture(
        &runner,
        public_page.page_id,
        TextBlockType::Html,
        2,
        None,
        "sentinel-public-duplicate-html-block",
        Some(DUPLICATE_HTML_BYTES),
    )
    .await;
    create_text_block_fixture(
        &runner,
        public_page.page_id,
        TextBlockType::Html,
        3,
        None,
        "sentinel-public-visible-duplicate-html-block-1",
        Some(TWO_VISIBLE_HTML_BYTES),
    )
    .await;
    create_text_block_fixture(
        &runner,
        public_page.page_id,
        TextBlockType::Html,
        4,
        None,
        "sentinel-public-visible-duplicate-html-block-2",
        Some(TWO_VISIBLE_HTML_BYTES),
    )
    .await;
    for index in 5_i16..=37_i16 {
        create_text_block_fixture(
            &runner,
            public_page.page_id,
            TextBlockType::Html,
            index,
            None,
            &format!("sentinel-overflow-html-block-{index}"),
            Some(OVERFLOW_HTML_BYTES),
        )
        .await;
    }

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
    assert!(
        public_block
            .s3_filename
            .starts_with(&format!("text-blocks/{}/html/", public_page.page_id))
    );

    let public_sha1 = format!("{:x}", Sha1::digest(PUBLIC_HTML_BYTES));
    let public_by_hash = run_endpoint!(
        runner,
        text_block_get_index,
        json!({
            "site_id": site_id,
            "block_type": "html",
            "sha1": public_sha1.clone(),
        }),
    )
    .expect("public HTML block should resolve by raw-byte SHA-1");
    assert_eq!(public_by_hash.index, 1);
    assert_eq!(public_by_hash.s3_filename, public_block.s3_filename);

    let duplicate_sha1 = format!("{:x}", Sha1::digest(DUPLICATE_HTML_BYTES));
    let duplicate_anonymous = run_endpoint!(
        runner,
        text_block_get_index,
        json!({
            "site_id": site_id,
            "block_type": "html",
            "sha1": duplicate_sha1.clone(),
        }),
    )
    .expect("anonymous lookup should skip the private duplicate and find public HTML");
    assert_eq!(duplicate_anonymous.index, 2);
    assert_eq!(
        duplicate_anonymous.s3_filename,
        "sentinel-public-duplicate-html-block"
    );

    let two_visible_sha1 = format!("{:x}", Sha1::digest(TWO_VISIBLE_HTML_BYTES));
    let two_visible = run_endpoint!(
        runner,
        text_block_get_index,
        json!({
            "site_id": site_id,
            "block_type": "html",
            "sha1": two_visible_sha1,
        }),
    );
    assert!(
        two_visible.is_none(),
        "two ACL-visible rows with one SHA-1 must fail closed"
    );

    let overflow_sha1 = format!("{:x}", Sha1::digest(OVERFLOW_HTML_BYTES));
    let overflow = run_endpoint!(
        runner,
        text_block_get_index,
        json!({
            "site_id": site_id,
            "block_type": "html",
            "sha1": overflow_sha1,
        }),
    );
    assert!(
        overflow.is_none(),
        "33 matching rows must fail closed before ACL resolution"
    );

    let mismatch_sha1 = format!(
        "{}{}",
        if public_sha1.as_bytes()[0] == b'0' {
            '1'
        } else {
            '0'
        },
        &public_sha1[1..],
    );
    let mismatch = run_endpoint!(
        runner,
        text_block_get_index,
        json!({
            "site_id": site_id,
            "block_type": "html",
            "sha1": mismatch_sha1,
        }),
    );
    assert!(mismatch.is_none());

    let private_sha1 = format!("{:x}", Sha1::digest(PRIVATE_HTML_BYTES));
    let private_anonymous = run_endpoint!(
        runner,
        text_block_get_index,
        json!({
            "site_id": site_id,
            "block_type": "html",
            "sha1": private_sha1.clone(),
        }),
    );
    assert!(private_anonymous.is_none());

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

    let private_by_hash = run_endpoint!(
        runner,
        text_block_get_index,
        json!({
            "site_id": site_id,
            "block_type": "html",
            "sha1": private_sha1,
            "session_token": admin_session_token,
        }),
    )
    .expect("admin should resolve private HTML block by raw-byte SHA-1");
    assert_eq!(private_by_hash.index, 1);
    assert_eq!(private_by_hash.s3_filename, "sentinel-private-html-block");

    let duplicate_admin = run_endpoint!(
        runner,
        text_block_get_index,
        json!({
            "site_id": site_id,
            "block_type": "html",
            "sha1": duplicate_sha1,
            "session_token": admin_session_token,
        }),
    );
    assert!(
        duplicate_admin.is_none(),
        "private plus public rows are two ACL-visible matches for admin"
    );

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

    let old_html_bytes: Vec<u8> = runner
        .state()
        .s3_tblocks_bucket
        .get_object(&public_block.s3_filename)
        .await
        .expect("existing HTML object should be readable")
        .into();
    let html_prefix = format!("text-blocks/{}/html/", public_page.page_id);
    let old_html_keys = runner
        .state()
        .s3_tblocks_bucket
        .list(html_prefix.clone(), None)
        .await
        .expect("existing HTML objects should be listable")
        .into_iter()
        .flat_map(|page| page.contents)
        .map(|object| object.key)
        .collect::<BTreeSet<_>>();
    runner
        .context()
        .transaction()
        .execute_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            format!(
                "ALTER TABLE text_block ADD CONSTRAINT issue1370_force_insert_{} CHECK (false) NOT VALID",
                public_page.page_id
            ),
        ))
        .await
        .expect("forced text-block insert failure should be installed");
    let failed_replacement = TextBlockService::add_blocks(
        runner.context(),
        public_page.page_id,
        TextBlockType::Html,
        &[TextBlock {
            text: "this replacement must not overwrite the old object",
            text_type: None,
            mime: MIME_HTML,
            name: None,
        }],
    )
    .await;
    assert!(
        failed_replacement.is_err(),
        "a forced database failure must fail the replacement"
    );
    let new_html_keys = runner
        .state()
        .s3_tblocks_bucket
        .list(html_prefix, None)
        .await
        .expect("HTML objects should remain listable after rollback")
        .into_iter()
        .flat_map(|page| page.contents)
        .map(|object| object.key)
        .collect::<BTreeSet<_>>();
    assert!(
        old_html_keys == new_html_keys,
        "a failed replacement must remove only its newly uploaded object"
    );
    let preserved_html_bytes: Vec<u8> = runner
        .state()
        .s3_tblocks_bucket
        .get_object(&public_block.s3_filename)
        .await
        .expect("the old HTML object should survive rollback")
        .into();
    assert_eq!(
        preserved_html_bytes, old_html_bytes,
        "a failed same-index replacement must preserve the old object bytes"
    );
    runner
        .state()
        .s3_tblocks_bucket
        .delete_object(&public_block.s3_filename)
        .await
        .expect("public HTML fixture cleanup should succeed");
    runner.teardown().await;
}

#[tokio::test]
async fn startup_backfills_legacy_html_sha1_and_validates_constraint() {
    let state = build_server_state_without_workers(
        Config::integration_testing(),
        Secrets::load(),
    )
    .await
    .expect("workerless Deepwell state should build");
    let page_id = PageTable::find()
        .one(&state.database)
        .await
        .expect("seeded page lookup should succeed")
        .expect("seeded page should exist")
        .page_id;
    let block_index = i16::MAX;
    let rejected_block_index = i16::MAX - 1;
    let missing_block_index = i16::MAX - 2;
    let s3_filename = format!("issue1370-startup-legacy-{}", cuid());
    let missing_s3_filename = format!("issue1370-startup-missing-{}", cuid());
    let raw_bytes = b"legacy HTML bytes for startup backfill";

    state
        .s3_tblocks_bucket
        .put_object(&s3_filename, raw_bytes)
        .await
        .expect("legacy HTML bytes should upload");
    let txn = state
        .database
        .begin()
        .await
        .expect("legacy fixture transaction should begin");
    txn.execute_raw(Statement::from_string(
        txn.get_database_backend(),
        "ALTER TABLE text_block ADD COLUMN IF NOT EXISTS wikidot_sha1 BYTEA",
    ))
    .await
    .expect("legacy fixture should establish the SHA-1 column");
    txn.execute_raw(Statement::from_string(
        txn.get_database_backend(),
        "ALTER TABLE text_block DROP CONSTRAINT IF EXISTS text_block_html_wikidot_sha1_present",
    ))
    .await
    .expect("legacy fixture should remove the validation constraint");
    text_block::ActiveModel {
        block_type: Set(TextBlockType::Html),
        page_id: Set(page_id),
        block_index: Set(missing_block_index),
        s3_filename: Set(missing_s3_filename),
        block_name: Set(None),
        text_type: Set(None),
        wikidot_sha1: Set(None),
    }
    .insert(&txn)
    .await
    .expect("missing legacy HTML row should be inserted");
    text_block::ActiveModel {
        block_type: Set(TextBlockType::Html),
        page_id: Set(page_id),
        block_index: Set(block_index),
        s3_filename: Set(s3_filename.clone()),
        block_name: Set(None),
        text_type: Set(None),
        wikidot_sha1: Set(None),
    }
    .insert(&txn)
    .await
    .expect("legacy HTML row should be inserted");
    txn.execute_raw(Statement::from_string(
        txn.get_database_backend(),
        "ALTER TABLE text_block ADD CONSTRAINT text_block_html_wikidot_sha1_present CHECK (block_type <> 'html' OR wikidot_sha1 IS NOT NULL) NOT VALID",
    ))
    .await
    .expect("legacy fixture should restore the unvalidated constraint");
    txn.commit()
        .await
        .expect("legacy fixture transaction should commit");
    drop(state);

    assert!(
        build_server_state(Config::integration_testing(), Secrets::load())
            .await
            .is_err(),
        "startup must fail when legacy HTML backfill cannot read S3"
    );
    let cleanup_state = build_server_state_without_workers(
        Config::integration_testing(),
        Secrets::load(),
    )
    .await
    .expect("workerless state should clean up the failed startup fixture");
    text_block::Entity::delete_by_id((TextBlockType::Html, page_id, missing_block_index))
        .exec(&cleanup_state.database)
        .await
        .expect("missing legacy row cleanup should succeed");
    drop(cleanup_state);

    let state = build_server_state(Config::integration_testing(), Secrets::load())
        .await
        .expect("startup should backfill and validate legacy HTML rows");
    let populated =
        text_block::Entity::find_by_id((TextBlockType::Html, page_id, block_index))
            .one(&state.database)
            .await
            .expect("backfilled row lookup should succeed")
            .expect("backfilled row should remain present");
    assert_eq!(
        populated.wikidot_sha1,
        Some(Sha1::digest(raw_bytes).to_vec())
    );

    let rejected = text_block::ActiveModel {
        block_type: Set(TextBlockType::Html),
        page_id: Set(page_id),
        block_index: Set(rejected_block_index),
        s3_filename: Set(format!("issue1370-startup-rejected-{}", cuid())),
        block_name: Set(None),
        text_type: Set(None),
        wikidot_sha1: Set(None),
    }
    .insert(&state.database)
    .await;
    assert!(
        rejected.is_err(),
        "validated constraint must reject a remaining NULL HTML identity"
    );

    text_block::Entity::delete_by_id((TextBlockType::Html, page_id, block_index))
        .exec(&state.database)
        .await
        .expect("legacy row cleanup should succeed");
    state
        .s3_tblocks_bucket
        .delete_object(&s3_filename)
        .await
        .expect("legacy object cleanup should succeed");
}

async fn create_empty_file_fixture(
    runner: &TestRunner,
    site_id: i64,
    page_id: i64,
    name: &str,
) -> i64 {
    create_file_fixture_with_mime(runner, site_id, page_id, name, EMPTY_BLOB_MIME).await
}

async fn create_file_fixture_with_mime(
    runner: &TestRunner,
    site_id: i64,
    page_id: i64,
    name: &str,
    mime: &str,
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
            mime: mime.to_owned(),
            content_type: Some(ContentTypeDescriptor {
                label: mime.to_owned(),
                description: mime.to_owned(),
            }),
            blob_created: false,
            revision_comments: "create file fixture".to_owned(),
        },
    )
    .await
    .expect("file revision fixture should be created");

    file.file_id
}

async fn create_file_fixture_with_descriptor(
    runner: &TestRunner,
    site_id: i64,
    page_id: i64,
    name: &str,
    size: i64,
    content_type: Option<ContentTypeDescriptor>,
) -> i64 {
    let file_id = insert_file_fixture_with_descriptor(
        runner,
        site_id,
        page_id,
        name,
        size,
        content_type,
    )
    .await;
    rerender_file_fixture_page(runner, page_id).await;
    file_id
}

async fn insert_file_fixture_with_descriptor(
    runner: &TestRunner,
    site_id: i64,
    page_id: i64,
    name: &str,
    size: i64,
    content_type: Option<ContentTypeDescriptor>,
) -> i64 {
    let file = file::ActiveModel {
        name: Set(name.to_owned()),
        site_id: Set(site_id),
        page_id: Set(page_id),
        ..Default::default()
    }
    .insert(runner.context().transaction())
    .await
    .expect("descriptor file fixture should be inserted");
    FileRevisionService::create_first(
        runner.context(),
        CreateFirstFileRevision {
            site_id,
            page_id,
            file_id: file.file_id,
            user_id: ADMIN_USER_ID,
            name: name.to_owned(),
            s3_hash: EMPTY_BLOB_HASH,
            size,
            mime: "image/jpeg".to_owned(),
            content_type,
            blob_created: false,
            revision_comments: "create descriptor file fixture".to_owned(),
        },
    )
    .await
    .expect("descriptor file revision fixture should be created");

    file.file_id
}

async fn rerender_file_fixture_page(runner: &TestRunner, page_id: i64) {
    let page = PageTable::find_by_id(page_id)
        .one(runner.context().transaction())
        .await
        .expect("descriptor page fixture lookup should succeed")
        .expect("descriptor page fixture should exist");
    PageRevisionService::rerender(
        runner.context(),
        PageId::from_page_model(&page),
        RerenderDepth::default(),
        RerenderType::Full,
    )
    .await
    .expect("descriptor page fixture should rerender after its file revision");
}

async fn create_text_block_fixture(
    runner: &TestRunner,
    page_id: i64,
    block_type: TextBlockType,
    block_index: i16,
    block_name: Option<&str>,
    s3_filename: &str,
    raw_bytes: Option<&[u8]>,
) {
    text_block::ActiveModel {
        block_type: Set(block_type),
        page_id: Set(page_id),
        block_index: Set(block_index),
        s3_filename: Set(s3_filename.to_owned()),
        block_name: Set(block_name.map(str::to_owned)),
        text_type: Set(None),
        wikidot_sha1: Set(raw_bytes.map(|bytes| Sha1::digest(bytes).to_vec())),
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct PageActionMutationSnapshot {
    page: page::Model,
    revision_count: u64,
    categories: Vec<page_category::Model>,
    audit_count: u64,
    text_blocks: Vec<(text_block::Model, Vec<u8>)>,
}

async fn snapshot_page_action_mutation_state(
    runner: &TestRunner,
    site_id: i64,
    page_id: i64,
) -> PageActionMutationSnapshot {
    let page = PageTable::find_by_id(page_id)
        .one(runner.context().transaction())
        .await
        .expect("page-action snapshot page lookup should succeed")
        .expect("page-action snapshot page should exist");
    let revision_count = PageRevisionTable::find()
        .filter(page_revision::Column::PageId.eq(page_id))
        .count(runner.context().transaction())
        .await
        .expect("page-action snapshot revision count should be readable");
    let categories = PageCategoryTable::find()
        .filter(page_category::Column::SiteId.eq(site_id))
        .order_by_asc(page_category::Column::CategoryId)
        .all(runner.context().transaction())
        .await
        .expect("page-action snapshot categories should be readable");
    let audit_count = AuditLogTable::find()
        .filter(AuditLogColumn::PageId.eq(page_id))
        .count(runner.context().transaction())
        .await
        .expect("page-action snapshot audit count should be readable");
    let text_blocks = snapshot_page_text_blocks(runner, page_id).await;

    PageActionMutationSnapshot {
        page,
        revision_count,
        categories,
        audit_count,
        text_blocks,
    }
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

async fn snapshot_page_text_blocks(
    runner: &TestRunner,
    page_id: i64,
) -> Vec<(text_block::Model, Vec<u8>)> {
    let mut rows = text_block::Entity::find()
        .filter(text_block::Column::PageId.eq(page_id))
        .all(runner.context().transaction())
        .await
        .expect("text-block snapshot query should succeed");
    rows.sort_by(|left, right| left.s3_filename.cmp(&right.s3_filename));

    let mut snapshot = Vec::with_capacity(rows.len());
    for row in rows {
        let object = runner
            .context()
            .s3_tblocks_bucket()
            .get_object(&row.s3_filename)
            .await
            .expect("text-block snapshot object should be readable");
        snapshot.push((row, object.into()));
    }
    snapshot
}

async fn persisted_text_count(runner: &TestRunner) -> u64 {
    text::Entity::find()
        .count(runner.context().transaction())
        .await
        .expect("persisted text count should be readable")
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
async fn listpages_plain_content_shares_the_outer_include_budget() {
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
    .expect("a structurally isolated content section should remain non-recursive");
    assert_eq!(
        output.html_output.body.matches(INCLUDE_MARKER).count(),
        1,
        "only the direct outer include may execute",
    );
    assert!(
        output
            .html_output
            .body
            .contains(&format!("[[include {COMPONENT_SLUG}]]")),
        "the selected child include must remain authored text",
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
    .expect("plain content may use the final include slot while a numbered section stays literal");
    assert_eq!(
        output.html_output.body.matches(INCLUDE_MARKER).count(),
        256,
        "plain selected content must execute within the caller's final include slot",
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
        "the selected child include must contribute its dependency backlink",
    );
    assert_eq!(
        output
            .html_output
            .body
            .matches(&format!("[[include {COMPONENT_SLUG}]]"))
            .count(),
        1,
        "only numbered section content may preserve the authored include token",
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
    .expect("selected child includes within the shared public limit should render");
    assert_eq!(
        output.html_output.body.matches(INCLUDE_MARKER).count(),
        256,
        "direct and selected child includes must share the public include limit",
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
    .expect_err(
        "repeated selected child includes must fail closed when the shared limit is exhausted",
    );
    assert!(
        format!("{error:?}")
            .contains("include expansion exceeded maximum total includes"),
        "the public failure must identify the include ceiling: {error:?}",
    );
    let output = RenderService::render_corpus_page(
        runner.context(),
        repeated_child,
        &page_info,
        Layout::Wikidot,
        page_id,
    )
    .await
    .expect("the trusted corpus include budget should cover both selected rows");
    assert_eq!(
        output.html_output.body.matches(INCLUDE_MARKER).count(),
        384,
        "corpus rendering must execute direct and selected child includes",
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
        "every executed direct and selected include must create a backlink",
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
    .expect_err(
        "two selected include-heavy rows must fail closed when their partition exceeds the public budget",
    );
    assert!(
        format!("{error:?}")
            .contains("include expansion exceeded maximum total includes"),
        "the public failure must identify the include ceiling: {error:?}",
    );

    let output = RenderService::render_corpus_page(
        runner.context(),
        over_budget,
        &page_info,
        Layout::Wikidot,
        page_id,
    )
    .await
    .expect(
        "trusted corpus rendering should execute both selected rows within its budget",
    );
    assert_eq!(
        output.html_output.body.matches(INCLUDE_MARKER).count(),
        384,
        "the corpus render must execute direct and selected child includes",
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
        "selected child includes must contribute dependency backlinks",
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
        "the first three nonempty content-backed modules should render",
    );
    assert!(
        output.html_output.body.contains("BROAD PRESERVED"),
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
        output.html_output.body.contains("RANDOM PRESERVED"),
        "a random query with an actual selected content row must consume one content-module slot: {}",
        output.html_output.body,
    );
    assert!(
        !output
            .html_output
            .body
            .contains("RANDOM PRESERVED %%content%%"),
    );
    assert!(
        output.html_output.body.contains("EXPANDED ONE")
            && !output.html_output.body.contains("EXPANDED ONE %%content%%"),
        "the third nonempty content-backed module must render: {}",
        output.html_output.body,
    );
    assert!(
        [
            "EXPANDED TWO %%content%%",
            "EXPANDED THREE %%content%%",
            "PRESERVED %%content%%",
        ]
        .iter()
        .all(|module| output.html_output.body.contains(module)),
        "content-backed modules after the third nonempty module must remain literal: {}",
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
async fn listpages_zero_row_content_modules_do_not_exhaust_the_work_budget() {
    const INDEX_SLUG: &str = "fixture-listpages-zero-row-content-budget-index";
    const CHILD_SLUG: &str = "fixture-listpages-zero-row-content-budget-child";
    const CHILD_MARKER: &str = "ZERO_ROW_CONTENT_BUDGET_CHILD";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    create_listpages_test_page(
        &mut runner,
        site_id,
        CHILD_SLUG,
        "ListPages Zero Row Content Budget Child",
        CHILD_MARKER,
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "ListPages Zero Row Content Budget Index",
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
    .expect("zero-row content budget index should exist");
    let page_info = PageInfo {
        page: Cow::Borrowed(INDEX_SLUG),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("ListPages Zero Row Content Budget Index"),
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
    let empty_modules = (0..41)
        .map(|index| {
            format!(
                "[[module ListPages name=\"definitely-missing-{INDEX_SLUG}\" \
                 limit=\"250\"]]EMPTY-{index} %%content%%[[/module]]\n",
            )
        })
        .collect::<String>();
    let source = format!(
        "{empty_modules}[[module ListPages name=\"{CHILD_SLUG}\" limit=\"1\"]]\
         SELECTED %%content%%[[/module]]",
    );

    let output = RenderService::render_page(
        runner.context(),
        source,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect("zero-row content modules should not exhaust selected-content work");
    let html = output.html_output.body;
    assert_eq!(
        html.matches(r#"<div class="list-pages-box"></div>"#)
            .count(),
        41,
        "all zero-row modules should execute as empty wrappers:\n{html}",
    );
    assert!(
        html.contains(CHILD_MARKER)
            && !html.contains("SELECTED %%content%%")
            && !html.contains("EMPTY-0 %%content%%")
            && !html.contains("EMPTY-40 %%content%%"),
        "the later nonempty content module should retain the full work budget:\n\
         {html}",
    );
}

#[tokio::test]
async fn listpages_current_page_content_shares_the_module_work_budget() {
    const INDEX_SLUG: &str = "fixture-listpages-current-page-content-budget-index";
    const CONTENT_MARKER: &str = "CURRENT_PAGE_CONTENT_BUDGET_BODY";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "ListPages Current Page Content Budget Index",
        CONTENT_MARKER,
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
    .expect("current-page content budget index should exist");
    let page_info = PageInfo {
        page: Cow::Borrowed(INDEX_SLUG),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("ListPages Current Page Content Budget Index"),
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
    let module = |label: &str| {
        format!(
            "[[module ListPages range=\".\" limit=\"1\" separate=\"no\" wrapper=\"no\"]]{label} %%content%%[[/module]]\n",
        )
    };
    let source = ["ONE", "TWO", "THREE", "FOUR"]
        .into_iter()
        .map(module)
        .collect::<String>();

    let output = RenderService::render_page(
        runner.context(),
        source,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect("current-page content budget overflow should preserve the later module");
    let html = output.html_output.body;
    assert_eq!(
        html.matches(CONTENT_MARKER).count(),
        3,
        "only the first three current-page content modules may expand:\n{html}",
    );
    assert!(
        html.contains("FOUR %%content%%"),
        "the fourth current-page content module must remain literal:\n{html}",
    );
}

#[tokio::test]
async fn listpages_template_body_over_the_render_budget_remains_literal() {
    const AT_LIMIT_SLUG: &str = "fixture-listpages-template-body-budget-at-limit";
    const OVER_LIMIT_SLUG: &str = "fixture-listpages-template-body-budget-over-limit";
    const TEMPLATE_BODY_BUDGET: usize = 256 * 1024;

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let template_body = |sentinel: &str, bytes: usize| {
        let variable = "%%title%%";
        assert!(sentinel.len() + variable.len() <= bytes);
        format!(
            "{sentinel}{}{variable}",
            "x".repeat(bytes - sentinel.len() - variable.len()),
        )
    };
    let module_source = |body: &str| {
        format!(
            "[[module ListPages range=\".\" limit=\"1\" separate=\"no\" wrapper=\"no\"]]{body}[[/module]]",
        )
    };

    create_listpages_test_page(
        &mut runner,
        site_id,
        AT_LIMIT_SLUG,
        "ListPages Template Body Budget At Limit",
        &module_source(&template_body(
            "AT_LIMIT_TEMPLATE_SENTINEL",
            TEMPLATE_BODY_BUDGET,
        )),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        OVER_LIMIT_SLUG,
        "ListPages Template Body Budget Over Limit",
        &module_source(&template_body(
            "OVER_LIMIT_TEMPLATE_SENTINEL",
            TEMPLATE_BODY_BUDGET + 1,
        )),
    )
    .await;

    let at_limit =
        load_listpages_test_compiled_html(&runner, site_id, AT_LIMIT_SLUG).await;
    assert!(
        at_limit.contains("AT_LIMIT_TEMPLATE_SENTINEL")
            && at_limit.contains("ListPages Template Body Budget At Limit")
            && !at_limit.contains("%%title%%"),
        "a ListPages template exactly at the body budget must still expand:\n{}",
        &at_limit[..at_limit.len().min(4_096)],
    );

    let over_limit =
        load_listpages_test_compiled_html(&runner, site_id, OVER_LIMIT_SLUG).await;
    assert!(
        over_limit.contains("OVER_LIMIT_TEMPLATE_SENTINEL")
            && over_limit.contains("%%title%%"),
        "an oversized ListPages template must remain literal instead of expanding:\n{}",
        &over_limit[..over_limit.len().min(4_096)],
    );
}

#[tokio::test]
async fn listpages_aggregate_generated_output_budget_preserves_overflowing_module() {
    const INDEX_SLUG: &str = "fixture-listpages-generated-byte-budget-index";
    const TARGET_PREFIX: &str = "fixture-listpages-generated-byte-budget-row";
    const ROW_COUNT: usize = 11;
    const TITLE_BYTES: usize = 200;
    const TITLE_VARIABLE_REPETITIONS: usize = 8_192;
    const GENERATED_BYTE_BUDGET: usize = 16 * 1024 * 1024;

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let title = "T".repeat(TITLE_BYTES);
    for index in 0..ROW_COUNT {
        let slug = format!("{TARGET_PREFIX}-{index:02}");
        create_listpages_test_page(&mut runner, site_id, &slug, &title, "row").await;
    }
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "ListPages Generated Byte Budget Index",
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
    .expect("ListPages generated-byte budget index should exist");
    let body = format!(
        "GENERATED_BYTE_BUDGET_SENTINEL{}",
        "%%title%%".repeat(TITLE_VARIABLE_REPETITIONS),
    );
    let source = format!(
        "[[module ListPages name=\"{TARGET_PREFIX}-*\" order=\"name\" limit=\"{ROW_COUNT}\" perPage=\"{ROW_COUNT}\" separate=\"no\" wrapper=\"no\"]]{body}[[/module]]",
    );
    const {
        assert!(
            ROW_COUNT * TITLE_BYTES * TITLE_VARIABLE_REPETITIONS > GENERATED_BYTE_BUDGET,
            "the fixture must exceed the intended aggregate output budget",
        );
    }
    let page_info = PageInfo {
        page: Cow::Borrowed(INDEX_SLUG),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("ListPages Generated Byte Budget Index"),
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

    let output = RenderService::render_page(
        runner.context(),
        source,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect("generated output overflow should preserve the complete module");
    let html = output.html_output.body;
    assert!(
        html.contains("GENERATED_BYTE_BUDGET_SENTINEL") && html.contains("%%title%%"),
        "a ListPages module that would exceed the aggregate generated-output budget must remain literal:\n{}",
        &html[..html.len().min(4_096)],
    );
}

#[tokio::test]
async fn listpages_content_fragment_restoration_counts_toward_generated_output_budget() {
    const INDEX_SLUG: &str = "fixture-listpages-content-fragment-budget-index";
    const TARGET_SLUG: &str = "fixture-listpages-content-fragment-budget-target";
    const SENTINEL: &str = "CONTENT_FRAGMENT_BUDGET_SENTINEL";
    const DIRECTIVE_BYTES: usize = 2 * 1024;
    const GENERATED_BYTE_BUDGET: usize = 16 * 1024 * 1024;
    const CONTENT_VARIABLE: &str = "%%content{1}%%";
    const CONTENT_REPETITIONS: usize = GENERATED_BYTE_BUDGET / DIRECTIVE_BYTES + 1;

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let directive = format!("[[{}]]", "x".repeat(DIRECTIVE_BYTES - 4));
    assert_eq!(directive.len(), DIRECTIVE_BYTES);
    create_listpages_test_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        "ListPages Content Fragment Budget Target",
        &directive,
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "ListPages Content Fragment Budget Index",
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
    .expect("ListPages content-fragment budget index should exist");
    let body = format!("{SENTINEL}{}", CONTENT_VARIABLE.repeat(CONTENT_REPETITIONS));
    assert!(body.len() <= 256 * 1024);
    const {
        assert!(
            DIRECTIVE_BYTES * CONTENT_REPETITIONS > GENERATED_BYTE_BUDGET,
            "restored content must exceed the render-wide generated-output budget",
        );
    }
    let source = format!(
        "[[module ListPages name=\"{TARGET_SLUG}\" limit=\"1\" separate=\"no\" wrapper=\"no\"]]{body}[[/module]]",
    );
    let page_info = PageInfo {
        page: Cow::Borrowed(INDEX_SLUG),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("ListPages Content Fragment Budget Index"),
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

    let output = RenderService::render_page(
        runner.context(),
        source,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect("restored content overflow should preserve the complete module");
    let html = output.html_output.body;
    assert!(
        html.contains(SENTINEL) && html.contains(CONTENT_VARIABLE),
        "escaped content hidden behind compact compatibility markers must still count toward the generated-output budget:\n{}",
        &html[..html.len().min(4_096)],
    );
}

#[tokio::test]
async fn listpages_following_paragraph_boundary_counts_toward_generated_output_budget() {
    const INDEX_SLUG: &str = "fixture-listpages-paragraph-byte-budget-index";
    const TARGET_PREFIX: &str = "fixture-listpages-paragraph-byte-budget-row";
    const ROW_COUNT: usize = 4;
    const TITLE_VARIABLE_REPETITIONS: usize = 20_701;
    const GENERATED_BYTE_BUDGET: usize = 16 * 1024 * 1024;
    const WRAPPER_OPEN: &str = "[[div class=\"list-pages-box\"]]\n";
    const WRAPPER_TRAILING_SPACE: &str = "\n    \n    \n    \n    ";
    const WRAPPER_CLOSE: &str = "[[/div]]";
    const WRAPPER_CLOSING_BOUNDARY_BYTES: usize = 1;
    const SENTINEL: &str = "PARAGRAPH_BYTE_BUDGET_SENTINEL";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let mut title_lengths = [200usize; ROW_COUNT];
    let fixed_without_padding = |title_lengths: &[usize; ROW_COUNT]| {
        WRAPPER_OPEN.len()
            + WRAPPER_TRAILING_SPACE.len()
            + WRAPPER_CLOSE.len()
            + WRAPPER_CLOSING_BOUNDARY_BYTES
            + ROW_COUNT * (SENTINEL.len() + 1)
            + TITLE_VARIABLE_REPETITIONS * title_lengths.iter().sum::<usize>()
    };
    let title_length_adjustment = (0..ROW_COUNT)
        .find(|adjustment| {
            title_lengths[0] += adjustment;
            let divisible = (GENERATED_BYTE_BUDGET
                - fixed_without_padding(&title_lengths))
            .is_multiple_of(ROW_COUNT);
            title_lengths[0] -= adjustment;
            divisible
        })
        .expect("one title-length adjustment must make row padding integral");
    title_lengths[0] += title_length_adjustment;
    let fixed_without_padding = fixed_without_padding(&title_lengths);
    let row_padding_bytes = (GENERATED_BYTE_BUDGET - fixed_without_padding) / ROW_COUNT;
    let body = format!(
        "{SENTINEL}{}{}",
        "%%title%%".repeat(TITLE_VARIABLE_REPETITIONS),
        "x".repeat(row_padding_bytes),
    );
    assert!(
        body.len() <= 256 * 1024,
        "the fixture must remain within the per-template source budget",
    );
    assert_eq!(
        WRAPPER_OPEN.len()
            + WRAPPER_TRAILING_SPACE.len()
            + WRAPPER_CLOSE.len()
            + WRAPPER_CLOSING_BOUNDARY_BYTES
            + title_lengths
                .iter()
                .map(|title_length| {
                    SENTINEL.len()
                        + TITLE_VARIABLE_REPETITIONS * title_length
                        + row_padding_bytes
                        + 1
                })
                .sum::<usize>(),
        GENERATED_BYTE_BUDGET,
        "the generated module must fill the output budget exactly before paragraph repair",
    );

    for (index, title_length) in title_lengths.into_iter().enumerate() {
        let slug = format!("{TARGET_PREFIX}-{index:02}");
        create_listpages_test_page(
            &mut runner,
            site_id,
            &slug,
            &"T".repeat(title_length),
            "row",
        )
        .await;
    }
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "ListPages Paragraph Byte Budget Index",
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
    .expect("ListPages paragraph-byte budget index should exist");
    let module = format!(
        "[[module ListPages name=\"{TARGET_PREFIX}-*\" order=\"name\" limit=\"{ROW_COUNT}\" perPage=\"{ROW_COUNT}\" separate=\"no\" wrapper=\"yes\"]]{body}[[/module]]",
    );
    let at_limit_source = format!("{module}\n\nAFTER_MODULE");
    let over_limit_source = format!("{module}\nAFTER_MODULE");
    let page_info = PageInfo {
        page: Cow::Borrowed(INDEX_SLUG),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("ListPages Paragraph Byte Budget Index"),
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

    let at_limit_output = RenderService::render_page(
        runner.context(),
        at_limit_source,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect("exact generated-output boundary should still render");
    let at_limit_html = at_limit_output.html_output.body;
    assert_eq!(
        at_limit_html.matches(SENTINEL).count(),
        ROW_COUNT,
        "the complete module must render when it fills the output budget exactly",
    );
    assert!(
        !at_limit_html.contains("%%title%%"),
        "the exact generated-output boundary must not be treated as overflow",
    );
    drop(at_limit_html);

    let output = RenderService::render_page(
        runner.context(),
        over_limit_source,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect("paragraph-boundary output overflow should preserve the module");
    let html = output.html_output.body;
    assert!(
        html.contains(SENTINEL) && html.contains("%%title%%"),
        "the post-render paragraph repair byte must not bypass the shared output budget:\n{}",
        &html[..html.len().min(4_096)],
    );
}

#[tokio::test]
async fn listpages_module_count_over_the_render_budget_remains_literal() {
    const INDEX_SLUG: &str = "fixture-listpages-module-count-budget-index";
    const MODULE_COUNT_BUDGET: usize = 512;

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "ListPages Module Count Budget Index",
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
    .expect("ListPages module-count budget index should exist");
    let module = concat!(
        "[[module ListPages range=\".\" limit=\"1\" separate=\"no\" wrapper=\"no\"]]",
        "MODULE_COUNT_BUDGET_SENTINEL%%title%%",
        "[[/module]]\n",
    );
    let at_limit_source = module.repeat(MODULE_COUNT_BUDGET);
    let source = module.repeat(MODULE_COUNT_BUDGET + 1);
    let page_info = PageInfo {
        page: Cow::Borrowed(INDEX_SLUG),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("ListPages Module Count Budget Index"),
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

    let at_limit_output = RenderService::render_page(
        runner.context(),
        at_limit_source,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect("module-count boundary should still render");
    let at_limit_html = at_limit_output.html_output.body;
    assert_eq!(
        at_limit_html
            .matches("ListPages Module Count Budget Index")
            .count(),
        MODULE_COUNT_BUDGET,
        "every ListPages module at the exact render-wide count boundary must expand",
    );
    assert!(
        !at_limit_html.contains("%%title%%"),
        "the exact module-count boundary must not be treated as overflow",
    );

    let output = RenderService::render_page(
        runner.context(),
        source,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect("module-count overflow should preserve the complete source");
    let html = output.html_output.body;
    assert!(
        html.contains("MODULE_COUNT_BUDGET_SENTINEL") && html.contains("%%title%%"),
        "ListPages source above the per-render module-count budget must remain literal:\n{}",
        &html[..html.len().min(4_096)],
    );
}

#[tokio::test]
async fn listpages_module_source_over_the_render_budget_remains_literal() {
    const INDEX_SLUG: &str = "fixture-listpages-module-source-budget-index";
    const MODULE_SOURCE_BUDGET: usize = 2 * 1024 * 1024;
    const MODULES: usize = 9;
    const OPENING: &str =
        "[[module ListPages range=\".\" limit=\"1\" separate=\"no\" wrapper=\"no\"]]";
    const CLOSING: &str = "[[/module]]";
    const SENTINEL: &str = "MODULE_SOURCE_BUDGET_SENTINEL";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "ListPages Module Source Budget Index",
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
    .expect("ListPages module-source budget index should exist");
    let build_source = |total_module_bytes: usize| {
        let mut source = String::with_capacity(total_module_bytes + MODULES);
        let base_module_bytes = total_module_bytes / MODULES;
        let remainder = total_module_bytes % MODULES;
        for index in 0..MODULES {
            let module_bytes = base_module_bytes + usize::from(index < remainder);
            let body_bytes = module_bytes - OPENING.len() - CLOSING.len();
            let fixed_body_bytes = SENTINEL.len() + "%%title%%".len();
            assert!(
                body_bytes <= 256 * 1024 && body_bytes >= fixed_body_bytes,
                "each generated fixture body must fit the individual template budget",
            );
            source.push_str(OPENING);
            source.push_str(SENTINEL);
            source.push_str(&"s".repeat(body_bytes - fixed_body_bytes));
            source.push_str("%%title%%");
            source.push_str(CLOSING);
            source.push('\n');
        }
        assert_eq!(
            source.len() - MODULES,
            total_module_bytes,
            "only separator newlines may sit outside the matched module-source total",
        );
        source
    };
    let at_limit_source = build_source(MODULE_SOURCE_BUDGET);
    let over_limit_source = build_source(MODULE_SOURCE_BUDGET + 1);
    let page_info = PageInfo {
        page: Cow::Borrowed(INDEX_SLUG),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("ListPages Module Source Budget Index"),
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

    let at_limit_output = RenderService::render_page(
        runner.context(),
        at_limit_source,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect("module-source boundary should still render");
    let at_limit_html = at_limit_output.html_output.body;
    assert_eq!(
        at_limit_html
            .matches("ListPages Module Source Budget Index")
            .count(),
        MODULES,
        "every module at the exact aggregate source-byte boundary must expand",
    );
    assert!(
        !at_limit_html.contains("%%title%%"),
        "the exact aggregate module-source boundary must not be treated as overflow",
    );

    let output = RenderService::render_page(
        runner.context(),
        over_limit_source,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect("module-source overflow should preserve the complete source");
    let html = output.html_output.body;
    assert!(
        html.contains("MODULE_SOURCE_BUDGET_SENTINEL") && html.contains("%%title%%"),
        "ListPages source above the aggregate module-source budget must remain literal:\n{}",
        &html[..html.len().min(4_096)],
    );
}

#[tokio::test]
async fn random_countpages_capped_sample_does_not_reveal_private_matches() {
    const INDEX_SLUG: &str = "fixture-random-countpages-private-cap-index";
    const PRIVATE_CATEGORY: &str = "fixture-random-countpages-private-cap";
    const PRIVATE_PAGE_COUNT: i32 = 5_000;

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    create_listpages_test_page(
        &mut runner,
        site_id,
        INDEX_SLUG,
        "Random CountPages Private Cap Index",
        "placeholder",
    )
    .await;
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;
    let private_category_id =
        CategoryService::get_or_create(runner.context(), site_id, PRIVATE_CATEGORY)
            .await
            .expect("private random CountPages category should exist")
            .category_id;
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "INSERT INTO page \
             (created_at, updated_at, deleted_at, from_wikidot, site_id, \
              latest_revision_id, page_category_id, slug, discussion_thread_id, layout) \
             SELECT NOW(), NULL, NULL, false, $1, NULL, $2, \
                    'fixture-random-countpages-private-row-' || series, NULL, 'wikidot' \
             FROM generate_series(1, $3) AS series",
            [
                Value::from(site_id),
                Value::from(private_category_id),
                Value::from(PRIVATE_PAGE_COUNT),
            ],
        ))
        .await
        .expect("private random CountPages fixtures should be inserted");
    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": INDEX_SLUG,
        }),
    )
    .expect("random CountPages private-cap index should exist");
    runner.set_request_context(RequestContext::default());
    let page_info = PageInfo {
        page: Cow::Borrowed(INDEX_SLUG),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("Random CountPages Private Cap Index"),
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
    let source = format!(
        concat!(
            "[[module CountPages category=\"fixture-random-countpages-empty\" order=\"random\"]]",
            "EMPTY=%%total%%",
            "[[/module]]\n",
            "[[module CountPages category=\"{PRIVATE_CATEGORY}\" order=\"random\"]]",
            "PRIVATE=%%total%%",
            "[[/module]]",
        ),
        PRIVATE_CATEGORY = PRIVATE_CATEGORY,
    );

    let output = RenderService::render_page(
        runner.context(),
        source,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments::default(),
    )
    .await
    .expect("anonymous random CountPages render should complete");
    let html = output.html_output.body;
    assert!(
        html.contains("EMPTY=0") && html.contains("PRIVATE=0"),
        "no matches and a capped private-only sample must have the same anonymous result:\n{html}",
    );
    assert!(
        !html.contains("%%total%%"),
        "private match existence must not be exposed through literal fallback:\n{html}",
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
    .expect_err("ordinary ListPages content must enforce the public include ceiling");
    assert!(
        format!("{public_list_pages_error:?}")
            .contains("include expansion exceeded maximum total includes 256"),
        "the public ListPages failure must identify the ordinary include ceiling: {public_list_pages_error:?}",
    );

    let list_pages_output = RenderService::render_corpus_page(
        runner.context(),
        list_pages_wikitext,
        &page_info,
        Layout::Wikidot,
        page_id,
    )
    .await
    .expect("trusted ListPages content should execute the evidenced dense include shape");
    assert_eq!(
        list_pages_output
            .html_output
            .body
            .matches(&format!("[[include {COMPONENT_SLUG}]]"))
            .count(),
        0,
        "trusted plain content must not expose authored include tokens",
    );
    assert_eq!(
        list_pages_output.html_output.body.matches(MARKER).count(),
        INCLUDE_COUNT,
        "trusted plain content must execute every selected include within its corpus budget",
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
async fn page_select_requires_an_authenticated_request_context() {
    let runner = TestRunner::setup().await;
    for selectors in [
        json!({}),
        json!({"categories": []}),
        json!({"tags_any": []}),
        json!({"categories": [], "tags_any": []}),
    ] {
        let mut params = selectors;
        params["site"] = json!("scp-wiki");
        let error = run_endpoint_err!(runner, page_select, params);
        assert_contains_error!(error, ErrorType::PermissionDenied);
    }
}

#[tokio::test]
async fn page_select_resolves_the_site_before_empty_selectors() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site.site.site_id),
        page_reference: None,
    });

    let baseline = exn_error_to_rpc_error(run_endpoint_err!(
        runner,
        page_select,
        json!({"site": "xmlrpc-missing-site"}),
    ));
    let baseline_shape = (baseline.code(), baseline.message().to_owned());

    for selectors in [
        json!({"categories": []}),
        json!({"tags_any": []}),
        json!({"categories": [], "tags_any": []}),
        json!({"tags_all": []}),
        json!({"tags_none": []}),
    ] {
        let mut params = selectors;
        params["site"] = json!("xmlrpc-missing-site");
        let error =
            exn_error_to_rpc_error(run_endpoint_err!(runner, page_select, params,));
        assert_eq!(
            (error.code(), error.message().to_owned()),
            baseline_shape,
            "empty selectors must preserve the ordinary missing-site fault shape",
        );
    }

    for selectors in [
        json!({"categories": []}),
        json!({"tags_any": []}),
        json!({"categories": [], "tags_any": []}),
    ] {
        let mut params = selectors;
        params["site"] = json!("scp-wiki");
        let selected = run_endpoint!(runner, page_select, params);
        assert!(selected.is_empty());
    }
}

#[tokio::test]
async fn page_select_filters_pages_by_authenticated_view_permission() {
    const VISIBLE_CATEGORY: &str = "xmlrpc-page-select-visible";
    const PRIVATE_CATEGORY: &str = "xmlrpc-page-select-private";
    const VISIBLE_SLUG: &str = "xmlrpc-page-select-visible:source";
    const PRIVATE_SLUG: &str = "xmlrpc-page-select-private:source";

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
        "page-select-viewer",
    )
    .await;
    make_page_mutation_test_category_for_user(
        &runner,
        site_id,
        PRIVATE_CATEGORY,
        ADMIN_USER_ID,
        &[Action::View],
        "page-select-admin",
    )
    .await;

    for (slug, category) in [
        (VISIBLE_SLUG, VISIBLE_CATEGORY),
        (PRIVATE_SLUG, PRIVATE_CATEGORY),
    ] {
        create_listpages_test_page(
            &mut runner,
            site_id,
            slug,
            "XML-RPC Page Select Permission Source",
            "XML-RPC page select permission source.",
        )
        .await;
        set_listpages_test_category_slug(&runner, site_id, slug, category).await;
    }

    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("page selection permission cache should be invalidated");
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(SAMPLE_USER_ID),
        site_id: Some(site_id),
        page_reference: None,
    });

    let selected = run_endpoint!(
        runner,
        page_select,
        json!({
            "site": "scp-wiki",
            "categories": [VISIBLE_CATEGORY, PRIVATE_CATEGORY],
            "order": "slug asc",
        }),
    );

    assert_eq!(selected, [VISIBLE_SLUG.to_owned()]);
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
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site.site.site_id),
        page_reference: None,
    });

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

async fn set_listpages_test_updated_at(
    runner: &TestRunner,
    site_id: i64,
    slug: &str,
    updated_at: OffsetDateTime,
) {
    let page = PageTable::find()
        .filter(
            sea_orm::Condition::all()
                .add(page::Column::SiteId.eq(site_id))
                .add(page::Column::Slug.eq(slug)),
        )
        .one(runner.context().transaction())
        .await
        .expect("updated_at test page lookup should not fail")
        .expect("updated_at test page should exist");
    let mut model = page.into_active_model();
    model.updated_at = Set(Some(updated_at));
    model
        .update(runner.context().transaction())
        .await
        .expect("updated_at test page update should not fail");
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

async fn mark_imported_page_with_author_snapshot(
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

async fn saved_article_view_body(
    runner: &TestRunner,
    site_id: i64,
    slug: &str,
) -> String {
    let view = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": slug, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let GetPageViewOutput::Found {
        compiled_body_html, ..
    } = view.page
    else {
        panic!("saved fixture page {slug} should be publicly viewable");
    };
    compiled_body_html
}

fn files_module_container_suffixes(html: &str) -> Vec<String> {
    let mut suffixes = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find(r#"<div id="files-"#) {
        let tail = &rest[start + r#"<div id="files-"#.len()..];
        let end = tail
            .find('"')
            .expect("a Files container id must close its attribute");
        let suffix = &tail[..end];
        assert!(
            !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit()),
            "a saved Files container suffix must be unpadded decimal digits: {suffix:?}",
        );
        suffixes.push(suffix.to_owned());
        rest = &tail[end..];
    }
    suffixes
}

fn files_module_function_suffixes(html: &str) -> Vec<String> {
    files_module_delimited_suffixes(html, "function updateFileSimpleList", '(')
}

fn files_module_selector_suffixes(html: &str) -> Vec<String> {
    files_module_delimited_suffixes(html, "containerElId = 'files-", '\'')
}

fn files_module_delimited_suffixes(
    html: &str,
    prefix: &str,
    terminator: char,
) -> Vec<String> {
    let mut suffixes = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find(prefix) {
        let tail = &rest[start + prefix.len()..];
        let end = tail
            .find(terminator)
            .expect("a Files module suffix must terminate");
        let suffix = &tail[..end];
        assert!(
            !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit()),
            "a saved Files module suffix must be unpadded decimal digits: {suffix:?}",
        );
        suffixes.push(suffix.to_owned());
        rest = &tail[end..];
    }
    suffixes
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
async fn countpages_saved_page_views_follow_current_matching_page_state() {
    async fn load_public_view(
        runner: &TestRunner,
        site_id: i64,
        slug: &str,
        extra: &str,
    ) -> String {
        match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": null,
                "route": {"slug": slug, "extra": extra},
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                compiled_body_html, ..
            } => compiled_body_html,
            other => panic!("expected found CountPages page view, got {other:?}"),
        }
    }

    fn assert_count(html: &str, marker: &str, expected: usize) {
        assert!(
            html.contains(&format!("{marker}={expected}")),
            "CountPages view should contain {marker}={expected}:\n{html}",
        );
        assert!(
            !html.contains(&format!("{marker}=%%total%%")),
            "executable CountPages must substitute its total:\n{html}",
        );
    }

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let category = "fixture-countpages-freshness";
    let template_category = "fixture-countpages-freshness-template";
    let tag = "fixture-countpages-freshness-match";
    let direct_slug = "fixture-countpages-freshness:direct";
    let template_slug = "fixture-countpages-freshness-template:_template";
    let templated_slug = "fixture-countpages-freshness-template:templated";
    let fallback_slug = "fixture-countpages-freshness:fallback";
    let dynamic_slug = "fixture-countpages-freshness:dynamic";
    let literal_slug = "fixture-countpages-freshness:literal";
    let target_slug = "fixture-countpages-freshness:target";

    CategoryService::get_or_create(runner.context(), site_id, category)
        .await
        .expect("CountPages freshness category should be created");
    CategoryService::get_or_create(runner.context(), site_id, template_category)
        .await
        .expect("CountPages freshness template category should be created");
    create_listpages_test_page(
        &mut runner,
        site_id,
        direct_slug,
        "Fixture CountPages Freshness Direct",
        &format!(
            "[[module CountPages category=\"{category}\" tags=\"+{tag}\" limit=\"20\"]]DIRECT_COUNT=%%total%%[[/module]]",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        template_slug,
        "Fixture CountPages Freshness Template",
        &format!(
            "[[module CountPages category=\"{category}\" tags=\"+{tag}\" limit=\"20\"]]TEMPLATE_COUNT=%%total%%[[/module]]\n\n%%content%%",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        templated_slug,
        "Fixture CountPages Freshness Templated",
        "TEMPLATED_CONTENT",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        fallback_slug,
        "Fixture CountPages Freshness URL Fallback",
        &format!(
            "[[module CountPages category=\"{category}\" tags=\"@URL|+{tag}\" limit=\"20\"]]FALLBACK_COUNT=%%total%%[[/module]]",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        dynamic_slug,
        "Fixture CountPages Freshness Dynamic Literal",
        &format!(
            "[[module CountPages category=\"{category}\" tags=\"@URL\" limit=\"20\"]]DYNAMIC_COUNT=%%total%%[[/module]]",
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        literal_slug,
        "Fixture CountPages Freshness Literal",
        &format!(
            "[[code]]\n[[module CountPages category=\"{category}\" tags=\"+{tag}\" limit=\"20\"]]CODE_COUNT=%%total%%[[/module]]\n[[/code]]\n\n[[module CountPages]][[/module]]",
        ),
    )
    .await;

    let stored_direct =
        load_listpages_test_compiled_html(&runner, site_id, direct_slug).await;
    let stored_dynamic =
        load_listpages_test_compiled_html(&runner, site_id, dynamic_slug).await;
    let stored_literal =
        load_listpages_test_compiled_html(&runner, site_id, literal_slug).await;
    assert_count(&stored_direct, "DIRECT_COUNT", 0);
    assert!(stored_dynamic.contains("DYNAMIC_COUNT=%%total%%"));
    assert!(stored_literal.contains("CODE_COUNT=%%total%%"));

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Slug(Cow::Borrowed(target_slug)),
    );
    let created = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "CountPages freshness matching target.",
            "title": "Fixture CountPages Freshness Matching Target",
            "alt_title": null,
            "slug": target_slug,
            "layout": "wikidot",
            "tags": [tag],
            "revision_comments": "create matching CountPages freshness target",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let target = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": target_slug}),
    )
    .expect("CountPages freshness target should exist");

    for (slug, marker) in [
        (direct_slug, "DIRECT_COUNT"),
        (templated_slug, "TEMPLATE_COUNT"),
        (fallback_slug, "FALLBACK_COUNT"),
    ] {
        let html = load_public_view(&runner, site_id, slug, "").await;
        assert_count(&html, marker, 1);
        if slug == templated_slug {
            assert!(html.contains("TEMPLATED_CONTENT"));
        }
    }
    assert_count(
        &load_public_view(
            &runner,
            site_id,
            fallback_slug,
            "/tag/conflicting-route-tag",
        )
        .await,
        "FALLBACK_COUNT",
        1,
    );

    let dynamic_with_route =
        load_public_view(&runner, site_id, dynamic_slug, &format!("/tag/{tag}")).await;
    assert_eq!(
        dynamic_with_route, stored_dynamic,
        "bare tags=\"@URL\" must stay literal even when the route supplies a tag",
    );
    let literal_after_create = load_public_view(&runner, site_id, literal_slug, "").await;
    assert_eq!(
        literal_after_create, stored_literal,
        "CountPages in code and a closed empty marker must not create runtime freshness",
    );

    let removed_revision = set_listpages_test_tags(
        &mut runner,
        site_id,
        target_slug,
        created.revision_id,
        &[],
    )
    .await;
    for (slug, marker) in [
        (direct_slug, "DIRECT_COUNT"),
        (templated_slug, "TEMPLATE_COUNT"),
        (fallback_slug, "FALLBACK_COUNT"),
    ] {
        assert_count(
            &load_public_view(&runner, site_id, slug, "").await,
            marker,
            0,
        );
    }

    let restored_revision = set_listpages_test_tags(
        &mut runner,
        site_id,
        target_slug,
        removed_revision,
        &[tag],
    )
    .await;
    assert_count(
        &load_public_view(&runner, site_id, direct_slug, "").await,
        "DIRECT_COUNT",
        1,
    );

    set_mutation_request_context(
        &mut runner,
        ADMIN_USER_ID,
        site_id,
        Reference::Id(target.page_id),
    );
    run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": target.page_id,
            "last_revision_id": restored_revision,
            "revision_comments": "delete matching CountPages freshness target",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    for (slug, marker) in [
        (direct_slug, "DIRECT_COUNT"),
        (templated_slug, "TEMPLATE_COUNT"),
        (fallback_slug, "FALLBACK_COUNT"),
    ] {
        assert_count(
            &load_public_view(&runner, site_id, slug, "").await,
            marker,
            0,
        );
    }

    let stored_after_mutations =
        load_listpages_test_compiled_html(&runner, site_id, direct_slug).await;
    assert_eq!(
        stored_after_mutations, stored_direct,
        "matching page mutations must not rewrite the holder's save-time compilation",
    );
}

/// Live capture (sandbox-for-codex, 2026-08-06): an unclosed CountPages
/// opener with the all-category selector uses Wikidot's deprecated default
/// shell, while the following source remains outside that shell.  A closed
/// CountPages with no body is a separate literal case.
#[tokio::test]
async fn unclosed_countpages_all_category_uses_live_default_shell() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let slug = "fixture-countpages-unclosed-default-shell";
    let source = "[[module CountPages category=\"*\"]]";

    create_listpages_test_page(
        &mut runner,
        site_id,
        slug,
        "Fixture CountPages Unclosed Default Shell",
        source,
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, slug).await;
    assert!(
        html.contains(r#"<div class="list-pages-box">"#),
        "the unclosed CountPages opener should use Wikidot's default shell:\n{html}",
    );
    for expected in [
        r#"<h1><span>%%linked_title%%</span></h1>"#,
        r#"<p>by %%author%% %%date|%O ago (%e %b %Y, %H:%M)%%</p>"#,
        r#"<p>%%short%%</p>"#,
    ] {
        assert!(
            html.contains(expected),
            "the live default shell should preserve {expected:?}:\n{html}",
        );
    }
    assert!(
        !html.contains("[[module CountPages")
            && !html.contains("TODO: module CountPages"),
        "the unclosed CountPages opener must not remain literal or leak an unsupported-module marker:\n{html}",
    );
}

/// The same live boundary (sandbox-for-codex, 2026-08-06) leaves source after
/// the unclosed opener outside the deprecated shell, while a closed empty
/// CountPages module remains literal rather than selecting that fallback.
#[tokio::test]
async fn countpages_unclosed_shell_stops_at_opener_and_closed_empty_stays_literal() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let unclosed_slug = "fixture-countpages-unclosed-default-shell-tail";
    create_listpages_test_page(
        &mut runner,
        site_id,
        unclosed_slug,
        "Fixture CountPages Unclosed Default Shell Tail",
        "[[module CountPages category=\"*\"]]\nTAIL_AFTER_COUNT_PAGES",
    )
    .await;
    let unclosed_html =
        load_listpages_test_compiled_html(&runner, site_id, unclosed_slug).await;
    assert!(
        unclosed_html.contains(r#"<div class="list-pages-box">"#)
            && unclosed_html.contains("TAIL_AFTER_COUNT_PAGES"),
        "source after an unclosed CountPages opener must remain after the live default shell:\n{unclosed_html}",
    );

    let closed_slug = "fixture-countpages-closed-empty-literal";
    create_listpages_test_page(
        &mut runner,
        site_id,
        closed_slug,
        "Fixture CountPages Closed Empty Literal",
        "[[module CountPages]][[/module]]",
    )
    .await;
    let closed_html =
        load_listpages_test_compiled_html(&runner, site_id, closed_slug).await;
    assert!(
        closed_html.contains("[[module CountPages]][[/module]]"),
        "a closed empty CountPages module must remain literal:\n{closed_html}",
    );
    assert!(
        !closed_html.contains(r#"<div class="list-pages-box">"#),
        "the deprecated unclosed fallback must not apply to a closed empty module:\n{closed_html}",
    );
}

#[tokio::test]
async fn listpages_template_parser_functions_run_after_row_variable_substitution() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let tag = "verification-listpages-template-parser-functions";

    let target_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-listpages-template-parser-functions-target",
        "Fixture ListPages Template Parser Functions Target",
        "Fixture ListPages parser-function target marker.",
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        "fixture-listpages-template-parser-functions-target",
        target_revision,
        &[tag],
    )
    .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-listpages-template-parser-functions-index",
        "Fixture ListPages Template Parser Functions Index",
        &format!(
            r#"[[module ListPages category="*" tags="+{tag}" order="name" limit="1"]]
[[#ifexpr %%rating_votes%% == 0 | ZERO_VOTES | HAS_VOTES]] [[#expr %%rating_votes%% + %%rating%%]]
[[/module]]"#
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(
        &runner,
        site_id,
        "fixture-listpages-template-parser-functions-index",
    )
    .await;

    assert!(
        html.contains("<p>ZERO_VOTES 0</p>"),
        "live Wikidot substitutes the row variables before evaluating #ifexpr and #expr:\n{html}",
    );
    for forbidden in [
        "TODO: module ListPages",
        "[[#ifexpr",
        "[[#expr",
        "%%rating_votes%%",
        "%%rating%%",
    ] {
        assert!(
            !html.contains(forbidden),
            "ListPages parser-function output should not contain {forbidden:?}:\n{html}",
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

    // Frozen Wikidot oracle: countpages-inside-listpages-live.json,
    // case standalone-countpages-opening-inside-listpages-body. One matching
    // page produces exactly one empty ListPages item.
    let list_pages_box = r#"<div class="list-pages-box">"#;
    let list_pages_item = r#"<div class="list-pages-item">"#;
    assert_eq!(
        html.matches(list_pages_box).count(),
        1,
        "the outer ListPages wrapper should appear once for one matching row:\n{html}",
    );
    assert_eq!(
        html.matches(list_pages_item).count(),
        1,
        "Wikidot should emit exactly one result item for the one matching row:\n{html}",
    );
    let item_start = html.find(list_pages_item).expect("item count was checked")
        + list_pages_item.len();
    let item_end = item_start
        + html[item_start..]
            .find("</div>")
            .expect("the ListPages item should close");
    assert!(
        html[item_start..item_end].trim().is_empty(),
        "the nested CountPages opening should leave the matching item empty:\n{html}",
    );
    for forbidden in [
        "Fixture CountPages Nested ListPages Target",
        "Fixture CountPages nested target marker.",
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

    // Frozen oracle control: the same selector renders an ordinary ListPages
    // body. The parser function is evidenced by the ListPages expression
    // observation, and the commented CountPages spelling remains literal per
    // syntax-comments.md's ListPages literal-owner contract.
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-countpages-nested-listpages-control",
        "Fixture CountPages Nested ListPages Control",
        &format!(
            r#"Ordinary CountPages control.

[[module ListPages category="*" tags="+{tag}" limit="1" order="name"]]
CONTROL_TITLE=%%title%%
CONTROL_BEFORE
[[#ifexpr 1 == 1 | CONTROL_IFEXPR | CONTROL_IFEXPR_FALSE]]
[!--
[[module CountPages category="*" tags="+{tag}" limit="10"]]
--]
CONTROL_AFTER
[[/module]]"#
        ),
    )
    .await;
    let control_page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": "fixture-countpages-nested-listpages-control",
            "details": {
                "compiled": true
            },
        }),
    )
    .expect("ordinary ListPages control page should exist");
    let control_html = control_page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");
    assert_eq!(
        control_html.matches(list_pages_box).count(),
        1,
        "the ordinary body should keep rendering one matching ListPages result:\n{control_html}",
    );
    assert_eq!(
        control_html.matches(list_pages_item).count(),
        1,
        "the ordinary body should render one result item for its one matching row:\n{control_html}",
    );
    for expected in [
        "CONTROL_TITLE=Fixture CountPages Nested ListPages Target",
        "CONTROL_BEFORE",
        "CONTROL_IFEXPR",
        "CONTROL_AFTER",
    ] {
        assert!(
            control_html.contains(expected),
            "ordinary ListPages output should contain {expected:?}:\n{control_html}",
        );
    }
    for forbidden in ["CONTROL_IFEXPR_FALSE", "[[module CountPages", "%%title%%"] {
        assert!(
            !control_html.contains(forbidden),
            "ordinary ListPages output should not contain {forbidden:?}:\n{control_html}",
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
        r#"category="*" tags="+verification-count-limit-cap-literal" limit="50001""#,
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
    let category = "fixture-first-revision-tagcloud";
    let page_slug = "fixture-first-revision-tagcloud:holder";
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
            "wikitext": format!("[[module TagCloud category=\"{category}\"]]"),
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
        html.contains(&format!(
            r#"<a class="tag" href="/system:page-tags/tag/{tag}/category/{category}""#
        )) && html.contains(&format!(">{tag}<")),
        "TagCloud should be rerendered after the first revision is attached:\n{html}"
    );
}

#[tokio::test]
async fn saved_tagcloud_page_view_follows_independent_tag_mutations() {
    async fn load_public_view(runner: &TestRunner, site_id: i64, slug: &str) -> String {
        match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": null,
                "route": {"slug": slug, "extra": ""},
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                compiled_body_html, ..
            } => compiled_body_html,
            other => panic!("expected found TagCloud page view, got {other:?}"),
        }
    }

    async fn import_holder(
        runner: &mut TestRunner,
        site_id: i64,
        page_id: i64,
        revision_id: i64,
        slug: &str,
        source: &str,
    ) {
        set_mutation_request_context(
            runner,
            ADMIN_USER_ID,
            site_id,
            Reference::Slug(Cow::Owned(slug.to_owned())),
        );
        run_endpoint!(
            runner,
            import_wikidot_page,
            json!({
                "page_id": page_id,
                "site_id": site_id,
                "created_at": "2026-08-13T00:00:00Z",
                "slug": slug,
                "locked": false,
                "discussion_thread_id": null,
                "ip_address": common::IP_ADDRESS,
            }),
        );
        run_endpoint!(
            runner,
            import_wikidot_page_revision,
            json!({
                "revision_id": revision_id,
                "revision_type": "create",
                "created_at": "2026-08-13 00:00:00.0 +00:00:00",
                "updated_at": null,
                "revision_number": 0,
                "page_id": page_id,
                "site_id": site_id,
                "user_id": ADMIN_USER_ID,
                "wikitext": source,
                "comments": "import TagCloud cache fixture",
                "title": "Imported TagCloud Freshness Holder",
                "slug": slug,
                "tags": [],
            }),
        );
    }

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let category = "fixture-tagcloud-freshness";
    let holder_slug = "fixture-tagcloud-freshness:holder";
    let duplicate_2d_slug = "fixture-tagcloud-freshness:duplicate-2d";
    let target_slug = "fixture-tagcloud-freshness:target";
    let link_target = "fixture-tagcloud-freshness:tags";
    let tag = "fixture-tagcloud-freshness-current";

    CategoryService::get_or_create(runner.context(), site_id, category)
        .await
        .expect("TagCloud freshness category should be created");
    let target_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        target_slug,
        "Fixture TagCloud Freshness Target",
        "TagCloud freshness target.",
    )
    .await;
    import_holder(
        &mut runner,
        site_id,
        2_140_170_000,
        2_140_171_000,
        holder_slug,
        &format!("[[module TagCloud category=\"{category}\" target=\"{link_target}\"]]",),
    )
    .await;
    import_holder(
        &mut runner,
        site_id,
        2_140_170_001,
        2_140_171_001,
        duplicate_2d_slug,
        &format!(
            "[[module TagCloud category=\"{category}\" target=\"{link_target}\" mode=\"3d\" mode=\"2d\"]]",
        ),
    )
    .await;
    let inert_holders = [
        (
            "fixture-tagcloud-freshness:literal",
            format!(
                "[[code]]\n[[module TagCloud category=\"{category}\" target=\"{link_target}\"]]\n[[/code]]",
            ),
        ),
        (
            "fixture-tagcloud-freshness:3d",
            format!(
                "[[module TagCloud category=\"{category}\" target=\"{link_target}\" mode=\"3d\"]]",
            ),
        ),
        (
            "fixture-tagcloud-freshness:duplicate-3d",
            format!(
                "[[module TagCloud category=\"{category}\" target=\"{link_target}\" mode=\"2d\" mode=\"3d\"]]",
            ),
        ),
        (
            "fixture-tagcloud-freshness:parser-invalid",
            format!(
                "[[module TagCloud category=\"{category}\" target=\"{link_target}\" broken]]",
            ),
        ),
    ];
    for (index, (slug, source)) in inert_holders.iter().enumerate() {
        import_holder(
            &mut runner,
            site_id,
            2_140_170_002 + index as i64,
            2_140_171_002 + index as i64,
            slug,
            source,
        )
        .await;
    }

    let stored_before =
        load_listpages_test_compiled_html(&runner, site_id, holder_slug).await;
    let stored_duplicate_2d =
        load_listpages_test_compiled_html(&runner, site_id, duplicate_2d_slug).await;
    let mut stored_inert = Vec::new();
    for (slug, _) in &inert_holders {
        stored_inert.push((
            *slug,
            load_listpages_test_compiled_html(&runner, site_id, slug).await,
        ));
    }
    assert!(!stored_before.contains(tag));
    assert!(!stored_duplicate_2d.contains(tag));

    for slug in [holder_slug, duplicate_2d_slug] {
        let cache_metadata = run_endpoint!(
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
            cache_metadata.article_page_cache_key, None,
            "an imported executable TagCloud page must not cache tag-dependent HTML for {slug}",
        );
    }
    for (slug, _) in &inert_holders {
        let cache_metadata = run_endpoint!(
            runner,
            article_view_cache_metadata,
            json!({
                "site_id": site_id,
                "session_token": null,
                "route": {"slug": slug, "extra": ""},
                "locales": ["en-US", "en"],
            }),
        );
        assert!(
            cache_metadata.article_page_cache_key.is_some(),
            "literal, effective 3d, and parser-invalid TagCloud shapes should remain cache-eligible for {slug}",
        );
    }

    set_listpages_test_tags(&mut runner, site_id, target_slug, target_revision, &[tag])
        .await;

    let view_html = load_public_view(&runner, site_id, holder_slug).await;
    assert!(
        view_html.contains(&format!(
            r#"<a class="tag" href="/{link_target}/tag/{tag}/category/{category}""#,
        )),
        "saved TagCloud view should use current category tags and its authored link target:\n{view_html}",
    );
    let duplicate_2d_html = load_public_view(&runner, site_id, duplicate_2d_slug).await;
    assert!(
        duplicate_2d_html.contains(&format!(
            r#"<a class="tag" href="/{link_target}/tag/{tag}/category/{category}""#,
        )),
        "the last mode attribute should make this TagCloud executable:\n{duplicate_2d_html}",
    );
    for (slug, stored) in stored_inert {
        assert_eq!(
            load_public_view(&runner, site_id, slug).await,
            stored,
            "literal, effective 3d, and parser-invalid TagCloud shapes must not opt into runtime freshness for {slug}",
        );
    }

    let stored_after =
        load_listpages_test_compiled_html(&runner, site_id, holder_slug).await;
    assert_eq!(
        stored_after, stored_before,
        "a TagCloud GET rerender must not rewrite the holder's stored compiled artifact",
    );
}

/// Live capture (sandbox-for-codex, 2026-07-29): `TagCloud` emits a
/// `pages-tag-cloud-box` of tag anchors, filters by category, interpolates
/// font/color styles over the displayed alphabetical tag slice, treats a
/// non-empty `showHidden` value as enabling hidden tags, and uses prefixed URL
/// argument names in generated links.
#[tokio::test]
async fn tagcloud_module_renders_live_category_links_styles_and_boolean_quirks() {
    fn section_between<'a>(html: &'a str, start: &str, end: &str) -> &'a str {
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
    let category = "fixture-tagcloud-live";
    let other_category = "fixture-tagcloud-live-other";
    let tag_alpha = "fixture-tagcloud-alpha";
    let tag_beta = "fixture-tagcloud-beta";
    let tag_shared = "fixture-tagcloud-shared";
    let tag_hidden = "_fixture-tagcloud-hidden";
    let tag_outside = "fixture-tagcloud-outside";

    CategoryService::get_or_create(runner.context(), site_id, category)
        .await
        .expect("TagCloud fixture category should be created");
    CategoryService::get_or_create(runner.context(), site_id, other_category)
        .await
        .expect("TagCloud fixture other category should be created");

    for (slug, title, tags) in [
        (
            format!("{category}:alpha"),
            "Fixture TagCloud Alpha",
            vec![tag_shared, tag_alpha],
        ),
        (
            format!("{category}:beta"),
            "Fixture TagCloud Beta",
            vec![tag_shared, tag_beta],
        ),
        (
            format!("{category}:gamma"),
            "Fixture TagCloud Gamma",
            vec![tag_shared, tag_beta, tag_hidden],
        ),
        (
            format!("{other_category}:outside"),
            "Fixture TagCloud Outside",
            vec![tag_shared, tag_outside],
        ),
    ] {
        let revision =
            create_listpages_test_page(&mut runner, site_id, &slug, title, "body").await;
        set_listpages_test_tags(&mut runner, site_id, &slug, revision, &tags).await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        &format!("{category}:target"),
        "Fixture TagCloud Target",
        "target",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        &format!("{category}:holder"),
        "Fixture TagCloud Holder",
        &format!(
            concat!(
                "TAGCLOUD_START\n\n",
                "CATEGORY_START\n",
                "[[module TagCloud category=\"{category}\"]]\n",
                "CATEGORY_END\n\n",
                "SHOW_FALSE_START\n",
                "[[module TagCloud category=\"{category}\" showHidden=\"false\"]]\n",
                "SHOW_FALSE_END\n\n",
                "SHOW_EMPTY_START\n",
                "[[module TagCloud category=\"{category}\" showHidden=\"\"]]\n",
                "SHOW_EMPTY_END\n\n",
                "LIMIT_START\n",
                "[[module TagCloud category=\"{category}\" limit=\"2\"]]\n",
                "LIMIT_END\n\n",
                "PREFIX_START\n",
                "[[module TagCloud category=\"{category}\" target=\"{category}:target\" urlAttrPrefix=\"lp\"]]\n",
                "PREFIX_END\n\n",
                "CUSTOM_STYLE_START\n",
                "[[module TagCloud category=\"{category}\" minFontSize=\"10px\" maxFontSize=\"20px\" minColor=\"1,2,3\" maxColor=\"200,210,220\"]]\n",
                "CUSTOM_STYLE_END\n\n",
                "BAD_FONT_START\n",
                "[[module TagCloud category=\"{category}\" minFontSize=\"10px\" maxFontSize=\"2em\"]]\n",
                "BAD_FONT_END\n\n",
                "INVALID_LIMIT_PARTIAL_STYLE_START\n",
                "[[module TagCloud category=\"{category}\" limit=\"0\" showHidden=\"false\" maxFontSize=\"2em\" minColor=\"1,2,3\"]]\n",
                "INVALID_LIMIT_PARTIAL_STYLE_END\n\n",
                "TAGCLOUD_END",
            ),
            category = category,
        ),
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": format!("{category}:holder"),
            "details": {"compiled": true},
        }),
    )
    .expect("TagCloud holder should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    let category_html = section_between(&html, "CATEGORY_START", "CATEGORY_END");
    assert!(
        category_html.contains(r#"<div class="pages-tag-cloud-box">"#)
            && category_html.contains(&format!(
                r#"<a class="tag" href="/system:page-tags/tag/{tag_alpha}/category/{category}""#
            ))
            && category_html.contains(&format!(
                r#"<a class="tag" href="/system:page-tags/tag/{tag_beta}/category/{category}""#
            ))
            && category_html.contains(&format!(
                r#"<a class="tag" href="/system:page-tags/tag/{tag_shared}/category/{category}""#
            ))
            && category_html.contains(r#"style="font-size: 100%; color: rgb(128, 128, 192);""#)
            && category_html.contains(r#"style="font-size: 200%; color: rgb(96, 96, 160);""#)
            && category_html.contains(r#"style="font-size: 300%; color: rgb(64, 64, 128);""#)
            && !category_html.contains(tag_hidden)
            && !category_html.contains(tag_outside)
            && !category_html.contains("[[module TagCloud"),
        "TagCloud category render should match live 2D anchor, style, and filtering behavior:\n{html}",
    );

    let show_false = section_between(&html, "SHOW_FALSE_START", "SHOW_FALSE_END");
    let show_false_beta = show_false
        .find(tag_beta)
        .expect("showHidden=false should include beta");
    let show_false_hidden = show_false
        .find(tag_hidden)
        .expect("showHidden=false should include hidden tags");
    let show_false_shared = show_false
        .find(tag_shared)
        .expect("showHidden=false should include shared");
    assert!(
        show_false_beta < show_false_hidden && show_false_hidden < show_false_shared,
        "live Wikidot treats non-empty showHidden values, including \"false\", as enabling hidden tags and sorts hidden tags as if the leading underscore were absent:\n{html}",
    );

    let show_empty = section_between(&html, "SHOW_EMPTY_START", "SHOW_EMPTY_END");
    assert!(
        !show_empty.contains(tag_hidden),
        "an empty showHidden attribute should not enable hidden tags:\n{html}",
    );

    let limit = section_between(&html, "LIMIT_START", "LIMIT_END");
    assert!(
        limit.contains(tag_alpha)
            && limit.contains(tag_beta)
            && !limit.contains(tag_shared)
            && limit.contains(r#"style="font-size: 300%; color: rgb(64, 64, 128);""#),
        "TagCloud limit should truncate the alphabetical tag list before rescaling styles:\n{html}",
    );

    let prefix = section_between(&html, "PREFIX_START", "PREFIX_END");
    assert!(
        prefix.contains(&format!(
            r#"href="/{category}:target/lp_tag/{tag_alpha}/lp_category/{category}""#
        )),
        "TagCloud urlAttrPrefix should prefix generated tag and category path argument names:\n{html}",
    );

    let custom_style = section_between(&html, "CUSTOM_STYLE_START", "CUSTOM_STYLE_END");
    assert!(
        custom_style.contains(r#"style="font-size: 10px; color: rgb(1, 2, 3);""#)
            && custom_style
                .contains(r#"style="font-size: 15px; color: rgb(101, 106, 112);""#)
            && custom_style
                .contains(r#"style="font-size: 20px; color: rgb(200, 210, 220);""#),
        "TagCloud custom size and color interpolation should match live Wikidot:\n{html}",
    );

    let bad_font = section_between(&html, "BAD_FONT_START", "BAD_FONT_END");
    assert!(
        bad_font.contains(r#"<div class="error-block">"#)
            && bad_font.contains(
                "Format for minFontSize and maxFontSize must be the same (px, em, pt or %).",
            ),
        "TagCloud mismatched font units should render the live error block:\n{html}",
    );

    let invalid_limit_partial_style = section_between(
        &html,
        "INVALID_LIMIT_PARTIAL_STYLE_START",
        "INVALID_LIMIT_PARTIAL_STYLE_END",
    );
    assert!(
        invalid_limit_partial_style.contains(tag_alpha)
            && invalid_limit_partial_style.contains(tag_beta)
            && invalid_limit_partial_style.contains(tag_hidden)
            && invalid_limit_partial_style.contains(tag_shared)
            && invalid_limit_partial_style
                .contains(r#"style="font-size: 100%; color: rgb(128, 128, 192);""#)
            && invalid_limit_partial_style
                .contains(r#"style="font-size: 300%; color: rgb(64, 64, 128);""#)
            && !invalid_limit_partial_style.contains("2em")
            && !invalid_limit_partial_style.contains("rgb(1, 2, 3)"),
        "invalid limit should fall back to default, and one-sided font/color overrides should be ignored:\n{html}",
    );
}

/// Live capture (sandbox-for-codex, 2026-07-29): omitted `category` counts
/// visible tags site-wide, `skipCategoryFromUrl="true"` removes the category
/// URL argument, invalid paired colors render Wikidot's error block, and
/// `mode="3d"` remains literal because its legacy SWFObject runtime would
/// execute external and inline script in the page origin.
#[tokio::test]
async fn tagcloud_module_renders_live_sitewide_skip_3d_and_color_error() {
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
    let category = "fixture-tagcloud-edge";
    let other_category = "fixture-tagcloud-edge-other";
    let tag_alpha = "aaa-fixture-tagcloud-alpha";
    let tag_beta = "aaa-fixture-tagcloud-beta";
    let tag_outside = "aaa-fixture-tagcloud-outside";
    let tag_shared = "aaa-fixture-tagcloud-shared";
    let tag_hidden = "_aaa-fixture-tagcloud-hidden";

    CategoryService::get_or_create(runner.context(), site_id, category)
        .await
        .expect("TagCloud fixture category should be created");
    CategoryService::get_or_create(runner.context(), site_id, other_category)
        .await
        .expect("TagCloud fixture other category should be created");

    for (slug, title, tags) in [
        (
            format!("{category}:alpha"),
            "Fixture TagCloud Edge Alpha",
            vec![tag_shared, tag_alpha],
        ),
        (
            format!("{category}:beta"),
            "Fixture TagCloud Edge Beta",
            vec![tag_shared, tag_beta],
        ),
        (
            format!("{category}:gamma"),
            "Fixture TagCloud Edge Gamma",
            vec![tag_shared, tag_beta, tag_hidden],
        ),
        (
            format!("{other_category}:outside"),
            "Fixture TagCloud Edge Outside",
            vec![tag_shared, tag_outside],
        ),
    ] {
        let revision =
            create_listpages_test_page(&mut runner, site_id, &slug, title, "body").await;
        set_listpages_test_tags(&mut runner, site_id, &slug, revision, &tags).await;
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        &format!("{category}:target"),
        "Fixture TagCloud Edge Target",
        "target",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        &format!("{category}:holder"),
        "Fixture TagCloud Edge Holder",
        &format!(
            concat!(
                "TAGCLOUD_EDGE_START\n\n",
                "SITEWIDE_START\n",
                "[[module TagCloud limit=\"1000\"]]\n",
                "SITEWIDE_END\n\n",
                "SKIP_START\n",
                "[[module TagCloud category=\"{category}\" target=\"{category}:target\" skipCategoryFromUrl=\"true\"]]\n",
                "SKIP_END\n\n",
                "BAD_COLOR_START\n",
                "[[module TagCloud category=\"{category}\" minColor=\"1,2,3\" maxColor=\"999,0,0\"]]\n",
                "BAD_COLOR_END\n\n",
                "THREED_START\n",
                "[[module TagCloud category=\"{category}\" mode=\"3d\" width=\"123\" height=\"77\"]]\n",
                "THREED_END\n\n",
                "TAGCLOUD_EDGE_END",
            ),
            category = category,
        ),
    )
    .await;

    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": format!("{category}:holder"),
            "details": {"compiled": true},
        }),
    )
    .expect("TagCloud edge holder should exist");
    let html = page
        .compiled_body_html
        .expect("compiled body should be included in page_get details");

    let sitewide = section(&html, "SITEWIDE_START", "SITEWIDE_END");
    assert!(
        sitewide.contains(&format!(
            r#"<a class="tag" href="/system:page-tags/tag/{tag_alpha}""#
        )) && sitewide.contains(&format!(
            r#"<a class="tag" href="/system:page-tags/tag/{tag_beta}""#
        )) && sitewide.contains(&format!(
            r#"<a class="tag" href="/system:page-tags/tag/{tag_outside}""#
        )) && sitewide.contains(&format!(
            r#"<a class="tag" href="/system:page-tags/tag/{tag_shared}""#
        )) && !sitewide.contains("/category/")
            && !sitewide.contains(tag_hidden),
        "omitted category should render visible tags site-wide without generated category URL arguments:\n{html}",
    );

    let skip = section(&html, "SKIP_START", "SKIP_END");
    assert!(
        skip.contains(&format!(r#"href="/{category}:target/tag/{tag_alpha}""#))
            && !skip.contains("/category/"),
        "skipCategoryFromUrl=true should omit category path arguments from generated tag links:\n{html}",
    );

    let bad_color = section(&html, "BAD_COLOR_START", "BAD_COLOR_END");
    assert!(
        bad_color.contains(r#"<div class="error-block">"#)
            && bad_color.contains(
                r#"Unsupported color format. Use "RRR,GGG,BBB" for Red,Green,Blue each within 0-255 range."#,
            ),
        "TagCloud invalid paired colors should render the live error block:\n{html}",
    );

    let threed = section(&html, "THREED_START", "THREED_END");
    assert!(
        threed.contains(&format!(
            r#"[[module TagCloud category=&quot;{category}&quot; mode=&quot;3d&quot; width=&quot;123&quot; height=&quot;77&quot;]]"#
        ))
            && !threed.contains("<script")
            && !threed.contains("javascript:")
            && !threed.contains("http://d3g0gp89917ko0.cloudfront.net")
            && !threed.contains("pages-tag-cloud-box"),
        "3D TagCloud must remain literal without restoring active page-origin content:\n{html}",
    );

    let preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "TagCloud 3D safety preview",
            "wikitext": format!(
                "[[module TagCloud category=\"{category}\" mode=\"3d\" width=\"123\" height=\"77\"]]"
            ),
        }),
    );
    for forbidden in [
        "<script",
        "javascript:",
        "http://d3g0gp89917ko0.cloudfront.net",
        "SWFObject",
    ] {
        assert!(
            !preview.body.contains(forbidden),
            "TagCloud 3D preview must not restore active content {forbidden:?}:\n{}",
            preview.body,
        );
    }
}

#[tokio::test]
async fn tagcloud_module_filters_anonymous_hidden_category_pages_before_loading_tags() {
    const PRIVATE_CATEGORY: &str = "fixture-tagcloud-security-private";
    const PRIVATE_SLUG: &str = "fixture-tagcloud-security-private:source";
    const PUBLIC_SLUG: &str = "fixture-tagcloud-security-public-source";
    const HOLDER_SLUG: &str = "fixture-tagcloud-security-holder";
    const PRIVATE_ONLY_TAG: &str = "fixture-tagcloud-security-private-only";
    const PUBLIC_ONLY_TAG: &str = "fixture-tagcloud-security-public-only";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;

    let private_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        PRIVATE_SLUG,
        "TagCloud security private source",
        "private source",
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, PRIVATE_SLUG, PRIVATE_CATEGORY)
        .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        PRIVATE_SLUG,
        private_revision,
        &[PRIVATE_ONLY_TAG],
    )
    .await;

    let public_revision = create_listpages_test_page(
        &mut runner,
        site_id,
        PUBLIC_SLUG,
        "TagCloud security public source",
        "public source",
    )
    .await;
    set_listpages_test_tags(
        &mut runner,
        site_id,
        PUBLIC_SLUG,
        public_revision,
        &[PUBLIC_ONLY_TAG],
    )
    .await;

    create_listpages_test_page(
        &mut runner,
        site_id,
        HOLDER_SLUG,
        "TagCloud security holder",
        concat!(
            "SITEWIDE_START\n",
            "[[module TagCloud]]\n",
            "SITEWIDE_END\n",
            "PRIVATE_START\n",
            "[[module TagCloud category=\"fixture-tagcloud-security-private\"]]\n",
            "PRIVATE_END\n",
        ),
    )
    .await;

    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("TagCloud security permission cache should be invalidated");
    runner.set_request_context(RequestContext {
        session: None,
        user_id: None,
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(HOLDER_SLUG))),
    });
    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": HOLDER_SLUG,
            "details": {"compiled": true},
        }),
    )
    .expect("anonymous TagCloud security holder should be readable");
    let html = page
        .compiled_body_html
        .expect("anonymous TagCloud security holder should include compiled HTML");

    assert!(
        html.contains(PUBLIC_ONLY_TAG),
        "anonymous TagCloud should retain tags from a public page:\n{html}",
    );
    assert!(
        !html.contains(PRIVATE_ONLY_TAG),
        "anonymous TagCloud must not load or count tags from a hidden category:\n{html}",
    );
}

/// Live capture (sandbox-for-codex, 2026-07-29): `PageCalendar` emits a
/// `page-calendar-box`, groups viewable pages by creation year and month,
/// applies category selectors, and generates date-path links to the current
/// page or to `targetPage` / `startPage`.
#[tokio::test]
async fn pagecalendar_module_renders_live_category_links_and_counts() {
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
    let category = "fixture-pagecalendar-live";
    let other_category = "fixture-pagecalendar-live-other";
    let missing_category = "fixture-pagecalendar-live-missing";
    let target = "fixture-pagecalendar-live-target";
    let holder = "fixture-pagecalendar-live-holder";

    CategoryService::get_or_create(runner.context(), site_id, category)
        .await
        .expect("PageCalendar fixture category should be created");
    CategoryService::get_or_create(runner.context(), site_id, other_category)
        .await
        .expect("PageCalendar fixture other category should be created");

    for (slug, title, timestamp) in [
        (
            format!("{category}:alpha"),
            "Fixture PageCalendar Alpha",
            1_785_225_600,
        ),
        (
            format!("{category}:beta"),
            "Fixture PageCalendar Beta",
            1_785_226_600,
        ),
        (
            format!("{category}:gamma"),
            "Fixture PageCalendar Gamma",
            1_785_227_600,
        ),
        (
            format!("{category}:delta"),
            "Fixture PageCalendar Delta",
            1_785_228_600,
        ),
        (
            format!("{other_category}:outside"),
            "Fixture PageCalendar Outside",
            1_785_229_600,
        ),
    ] {
        create_listpages_test_page(&mut runner, site_id, &slug, title, "body").await;
        set_listpages_test_created_at(
            &runner,
            site_id,
            &slug,
            OffsetDateTime::from_unix_timestamp(timestamp)
                .expect("fixture timestamp should be valid"),
        )
        .await;
    }
    create_listpages_test_page(
        &mut runner,
        site_id,
        target,
        "Fixture PageCalendar Target",
        "target",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        holder,
        "Fixture PageCalendar Holder",
        &format!(
            concat!(
                "PAGECALENDAR_START\n\n",
                "EXPLICIT_START\n",
                "[[module PageCalendar category=\"{category}\"]]\n",
                "EXPLICIT_END\n\n",
                "PREFIX_TARGET_START\n",
                "[[module PageCalendar category=\"{category}\" targetPage=\"{target}\" urlAttrPrefix=\"lp\"]]\n",
                "PREFIX_TARGET_END\n\n",
                "START_ALIAS_START\n",
                "[[module PageCalendar category=\"{category}\" startPage=\"{target}\"]]\n",
                "START_ALIAS_END\n\n",
                "MULTI_SPACE_START\n",
                "[[module PageCalendar category=\"{category} {other_category}\" targetPage=\"{target}\"]]\n",
                "MULTI_SPACE_END\n\n",
                "MULTI_COMMA_START\n",
                "[[module PageCalendar category=\"{category},{other_category}\" targetPage=\"{target}\"]]\n",
                "MULTI_COMMA_END\n\n",
                "MISSING_START\n",
                "[[module PageCalendar category=\"{missing_category}\" targetPage=\"{target}\"]]\n",
                "MISSING_END\n\n",
                "EMPTY_CATEGORY_START\n",
                "[[module PageCalendar category=\"\" targetPage=\"{target}\"]]\n",
                "EMPTY_CATEGORY_END\n\n",
                "EMPTY_TARGET_START\n",
                "[[module PageCalendar category=\"{category}\" targetPage=\"\" urlAttrPrefix=\"lp\"]]\n",
                "EMPTY_TARGET_END\n\n",
                "DUPLICATE_START\n",
                "[[module PageCalendar category=\"{missing_category}\" category=\"{category}\" targetPage=\"{target}\"]]\n",
                "DUPLICATE_END\n\n",
                "PAGECALENDAR_END",
            ),
            category = category,
            other_category = other_category,
            missing_category = missing_category,
            target = target,
        ),
    )
    .await;

    let html = load_listpages_test_compiled_html(&runner, site_id, holder).await;

    let explicit = section(&html, "EXPLICIT_START", "EXPLICIT_END");
    assert!(
        explicit.contains(r#"<div class="page-calendar-box">"#)
            && explicit.contains(r#"<ul>"#)
            && explicit.contains(r#">2026 (4)</a>"#)
            && explicit.contains(r#">July (4)</a>"#)
            && explicit.contains(&format!(r#"href="/{holder}/date/2026""#))
            && explicit.contains(&format!(r#"href="/{holder}/date/2026.7""#)),
        "PageCalendar explicit category output should match live DOM, counts, and current-page date links:\n{html}",
    );

    let prefix_target = section(&html, "PREFIX_TARGET_START", "PREFIX_TARGET_END");
    assert!(
        prefix_target.contains(&format!(r#"href="/{target}/lp_date/2026""#))
            && prefix_target.contains(&format!(r#"href="/{target}/lp_date/2026.7""#)),
        "targetPage and urlAttrPrefix should drive generated PageCalendar date links:\n{html}",
    );

    let start_alias = section(&html, "START_ALIAS_START", "START_ALIAS_END");
    assert!(
        start_alias.contains(&format!(r#"href="/{target}/date/2026""#))
            && start_alias.contains(&format!(r#"href="/{target}/date/2026.7""#)),
        "startPage should be a PageCalendar targetPage alias:\n{html}",
    );

    for marker in ["MULTI_SPACE", "MULTI_COMMA"] {
        let multi = section(&html, &format!("{marker}_START"), &format!("{marker}_END"));
        assert!(
            multi.contains(r#">2026 (5)</a>"#) && multi.contains(r#">July (5)</a>"#),
            "PageCalendar should accept comma- and space-separated category lists:\n{html}",
        );
    }

    let missing = section(&html, "MISSING_START", "MISSING_END");
    assert!(
        missing.contains(r#"<div class="error-block">"#)
            && missing.contains("The requested categories do not (yet) exist."),
        "PageCalendar should render the live error block for a nonexistent explicit category:\n{html}",
    );

    let empty_category = section(&html, "EMPTY_CATEGORY_START", "EMPTY_CATEGORY_END");
    assert!(
        empty_category.contains(r#"<div class="error-block">"#)
            && empty_category.contains("The requested categories do not (yet) exist."),
        "PageCalendar should render the live error block for an empty explicit category:\n{html}",
    );

    let empty_target = section(&html, "EMPTY_TARGET_START", "EMPTY_TARGET_END");
    assert!(
        empty_target.contains(&format!(r#"href="/{holder}/lp_date/2026""#))
            && empty_target.contains(&format!(r#"href="/{holder}/lp_date/2026.7""#)),
        "empty PageCalendar targetPage should fall back to the current page:\n{html}",
    );

    let duplicate = section(&html, "DUPLICATE_START", "DUPLICATE_END");
    assert!(
        duplicate.contains(r#">2026 (4)</a>"#)
            && !duplicate.contains("do not (yet) exist"),
        "duplicate PageCalendar category attributes should use the last category value observed on live Wikidot:\n{html}",
    );

    let selected_month_view = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": holder, "extra": "/date/2026.7"},
            "locales": ["en-US", "en"],
        }),
    );
    let selected_month_html = match selected_month_view {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected found PageCalendar date view, got {other:?}"),
    };
    let selected_explicit =
        section(&selected_month_html, "EXPLICIT_START", "EXPLICIT_END");
    let month_link = format!(r#"<a href="/{holder}/date/2026.7">July (4)</a>"#);
    let month_link_start = selected_explicit
        .find(&month_link)
        .expect("the routed PageCalendar month should render");
    let month_row_start = selected_explicit[..month_link_start]
        .rfind("<li")
        .expect("the selected month should have a list row");
    let year_link = format!(r#"<a href="/{holder}/date/2026">2026 (4)</a>"#);
    let year_link_start = selected_explicit
        .find(&year_link)
        .expect("the routed PageCalendar year should render");
    let year_row_start = selected_explicit[..year_link_start]
        .rfind("<li")
        .expect("the selected year should have a list row");
    assert!(
        selected_explicit[month_row_start..month_link_start]
            .contains(r#"class="selected""#)
            && !selected_explicit[year_row_start..year_link_start]
                .contains(r#"class="selected""#),
        "PageCalendar month URL should select only the month row:\n{selected_month_html}",
    );

    assert!(!html.contains("[[module PageCalendar"), "{html}");
}

/// Live capture (sandbox-for-codex, 2026-07-29): despite the documentation,
/// `tags` does not filter the calendar counts. It is only propagated into
/// generated `tag` URL path arguments, with `+` replaced by spaces. When a
/// `category="@URL|fallback"` selector resolves from the URL, PageCalendar
/// carries that category forward in generated links.
#[tokio::test]
async fn pagecalendar_module_matches_live_tag_url_and_current_category_quirks() {
    fn section_between<'a>(html: &'a str, start: &str, end: &str) -> &'a str {
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
    let category = "fixture-pagecalendar-url";
    let other_category = "fixture-pagecalendar-url-other";
    let default_category = "fixture-pagecalendar-default";
    let target = "fixture-pagecalendar-url-target";
    let holder = "fixture-pagecalendar-url-holder";
    let default_holder = format!("{default_category}:holder");
    let tag_required = "fixture-pagecalendar-required";
    let tag_any_one = "fixture-pagecalendar-any-one";
    let tag_any_two = "fixture-pagecalendar-any-two";
    let tag_excluded = "fixture-pagecalendar-excluded";

    for category_slug in [category, other_category, default_category] {
        CategoryService::get_or_create(runner.context(), site_id, category_slug)
            .await
            .expect("PageCalendar fixture category should be created");
    }

    for (slug, title, tags) in [
        (
            format!("{category}:alpha"),
            "Fixture PageCalendar URL Alpha",
            vec![tag_required, tag_any_one],
        ),
        (
            format!("{category}:beta"),
            "Fixture PageCalendar URL Beta",
            vec![tag_required, tag_any_two, tag_excluded],
        ),
        (
            format!("{category}:gamma"),
            "Fixture PageCalendar URL Gamma",
            vec![tag_required, tag_any_two],
        ),
        (
            format!("{category}:delta"),
            "Fixture PageCalendar URL Delta",
            vec![tag_any_one],
        ),
        (
            format!("{other_category}:outside"),
            "Fixture PageCalendar URL Outside",
            vec![tag_required, tag_any_one],
        ),
        (
            format!("{default_category}:alpha"),
            "Fixture PageCalendar Default Alpha",
            vec![],
        ),
        (
            format!("{default_category}:beta"),
            "Fixture PageCalendar Default Beta",
            vec![],
        ),
    ] {
        let revision =
            create_listpages_test_page(&mut runner, site_id, &slug, title, "body").await;
        set_listpages_test_created_at(
            &runner,
            site_id,
            &slug,
            OffsetDateTime::from_unix_timestamp(1_785_225_600)
                .expect("fixture timestamp should be valid"),
        )
        .await;
        if !tags.is_empty() {
            set_listpages_test_tags(&mut runner, site_id, &slug, revision, &tags).await;
        }
    }
    create_listpages_test_page(
        &mut runner,
        site_id,
        target,
        "Fixture PageCalendar URL Target",
        "target",
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        holder,
        "Fixture PageCalendar URL Holder",
        &format!(
            concat!(
                "PAGECALENDAR_URL_START\n\n",
                "TAGS_START\n",
                "[[module PageCalendar category=\"{category}\" tags=\"+{tag_required} -{tag_excluded} {tag_any_one} {tag_any_two}\" targetPage=\"{target}\"]]\n",
                "TAGS_END\n\n",
                "TAGS_COMMA_START\n",
                "[[module PageCalendar category=\"{category}\" tags=\"+{tag_required},-{tag_excluded},{tag_any_one},{tag_any_two}\" targetPage=\"{target}\"]]\n",
                "TAGS_COMMA_END\n\n",
                "URL_DEFAULT_START\n",
                "[[module PageCalendar category=\"@URL|{category}\" tags=\"@URL|+{tag_required}\" targetPage=\"{target}\" urlAttrPrefix=\"lp\"]]\n",
                "URL_DEFAULT_END\n\n",
                "PAGECALENDAR_URL_END",
            ),
            category = category,
            target = target,
            tag_required = tag_required,
            tag_excluded = tag_excluded,
            tag_any_one = tag_any_one,
            tag_any_two = tag_any_two,
        ),
    )
    .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        &default_holder,
        "Fixture PageCalendar Default Holder",
        &format!(
            concat!(
                "DEFAULT_START\n",
                "[[module PageCalendar]]\n",
                "DEFAULT_END\n\n",
                "DEFAULT_TARGET_START\n",
                "[[module PageCalendar targetPage=\"{target}\" urlAttrPrefix=\"d\"]]\n",
                "DEFAULT_TARGET_END",
            ),
            target = target,
        ),
    )
    .await;
    set_listpages_test_created_at(
        &runner,
        site_id,
        &default_holder,
        OffsetDateTime::from_unix_timestamp(1_785_225_600)
            .expect("fixture timestamp should be valid"),
    )
    .await;
    let default_page = run_endpoint!(
        runner,
        page_get,
        json!({"site_id": site_id, "page": default_holder}),
    )
    .expect("PageCalendar default holder should exist");
    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": default_page.page_category_id,
            "page_id": default_page.page_id,
        }),
    );

    let holder_page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": holder,
        }),
    )
    .expect("PageCalendar URL holder should exist");
    let page_info = PageInfo {
        page: Cow::Borrowed(holder),
        category: None,
        site: Cow::Borrowed("scp-wiki"),
        title: Cow::Borrowed("Fixture PageCalendar URL Holder"),
        alt_title: None,
        score: ScoreValue::Integer(0),
        tags: Vec::new(),
        language: Cow::Borrowed("en"),
    };
    let page_id = PageId {
        site_id,
        category_id: holder_page.page_category_id,
        page_id: holder_page.page_id,
    };
    let source = format!(
        concat!(
            "PAGECALENDAR_URL_START\n\n",
            "TAGS_START\n",
            "[[module PageCalendar category=\"{category}\" tags=\"+{tag_required} -{tag_excluded} {tag_any_one} {tag_any_two}\" targetPage=\"{target}\"]]\n",
            "TAGS_END\n\n",
            "TAGS_COMMA_START\n",
            "[[module PageCalendar category=\"{category}\" tags=\"+{tag_required},-{tag_excluded},{tag_any_one},{tag_any_two}\" targetPage=\"{target}\"]]\n",
            "TAGS_COMMA_END\n\n",
            "URL_DEFAULT_START\n",
            "[[module PageCalendar category=\"@URL|{category}\" tags=\"@URL|+{tag_required}\" targetPage=\"{target}\" urlAttrPrefix=\"lp\"]]\n",
            "URL_DEFAULT_END\n\n",
            "PAGECALENDAR_URL_END",
        ),
        category = category,
        target = target,
        tag_required = tag_required,
        tag_excluded = tag_excluded,
        tag_any_one = tag_any_one,
        tag_any_two = tag_any_two,
    );
    let url_arguments = vec![
        UrlArgumentPair {
            name: "lp_category".to_owned(),
            value: Some(other_category.to_owned()),
        },
        UrlArgumentPair {
            name: "lp_tags".to_owned(),
            value: Some(format!("+{tag_required}")),
        },
        UrlArgumentPair {
            name: "lp_date".to_owned(),
            value: Some("2026.7".to_owned()),
        },
    ];
    let output = RenderService::render_page(
        runner.context(),
        source,
        &page_info,
        Layout::Wikidot,
        page_id,
        UrlArguments {
            path_arguments: &url_arguments,
            ..UrlArguments::default()
        },
    )
    .await
    .expect("PageCalendar URL render should succeed");
    let html = output.html_output.body;

    let tags = section_between(&html, "TAGS_START", "TAGS_END");
    assert!(
        tags.contains(r#">2026 (4)</a>"#)
            && tags.contains(&format!(
                r#"href="/{target}/tag/ {tag_required} -{tag_excluded} {tag_any_one} {tag_any_two}/date/2026""#
            )),
        "live PageCalendar ignores tags for counts but carries the tag expression in generated paths with plus signs replaced by spaces:\n{html}",
    );

    let tags_comma = section_between(&html, "TAGS_COMMA_START", "TAGS_COMMA_END");
    assert!(
        tags_comma.contains(r#">2026 (4)</a>"#)
            && tags_comma.contains(&format!(
                r#"href="/{target}/tag/ {tag_required},-{tag_excluded},{tag_any_one},{tag_any_two}/date/2026""#
            )),
        "live PageCalendar preserves comma-separated tag expressions in generated paths except for leading plus-to-space conversion:\n{html}",
    );

    let url_default = section_between(&html, "URL_DEFAULT_START", "URL_DEFAULT_END");
    assert!(
        url_default.contains(r#">2026 (1)</a>"#)
            && url_default.contains(&format!(
                r#"href="/{target}/lp_tag/ {tag_required}/lp_category/{other_category}/lp_date/2026""#
            ))
            && url_default.contains(r#"<li class="selected">"#)
            && url_default.contains(r#"lp_date/2026.7">July (1)</a>"#),
        "PageCalendar @URL category and tags should read prefixed URL args and propagate them to generated links:\n{html}",
    );

    let default_html =
        load_listpages_test_compiled_html(&runner, site_id, &default_holder).await;
    let default = section_between(&default_html, "DEFAULT_START", "DEFAULT_END");
    assert!(
        default.contains(r#">2026 (3)</a>"#)
            && default.contains(&format!(r#"href="/{default_holder}/date/2026""#)),
        "omitted category should default to the current page category and include the holder page itself:\n{default_html}",
    );

    let default_target =
        section_between(&default_html, "DEFAULT_TARGET_START", "DEFAULT_TARGET_END");
    assert!(
        default_target.contains(&format!(r#"href="/{target}/d_date/2026""#)),
        "default-category PageCalendar should still honor targetPage and urlAttrPrefix:\n{default_html}",
    );
}
