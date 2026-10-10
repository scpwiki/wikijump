/*
 * tests/page_meta_tag.rs
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
use cuid2::cuid;
use deepwell::constants::{ADMIN_USER_ID, SYSTEM_USER_ID};
use deepwell::error::prelude::*;
use deepwell::services::RequestContext;
use deepwell::services::page::CreatePageOutput;
use deepwell::services::page_meta_tag::PageMetaTag;
use deepwell::services::permission::PermissionService;
use deepwell::services::role::{
    GrantUserRoleInput, InternalCreateRoleInput, RoleService, UpdateRolePermissionsInput,
};
use deepwell::services::user::{CreateUser, UserService};
use deepwell::services::view::{GetArticleViewOutput, GetPageViewOutput};
use deepwell::types::{Action, Permission, Reference, Resource, UserType};
use serde_json::json;
use str_macro::str;

async fn create_page(runner: &mut TestRunner) -> (i64, String, CreatePageOutput) {
    let site = run_endpoint!(runner, site_get, json!({"site": "scpaiueouiuiuiui"}))
        .expect("editable authoring site should exist")
        .site;
    let slug = format!("edit-meta-{}", cuid());
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site.site_id),
        page_reference: Some(Reference::Slug(slug.clone().into())),
        ..Default::default()
    });
    let page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site.site_id,
            "wikitext": "",
            "title": "Edit meta fixture",
            "alt_title": null,
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "create edit meta fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site.site_id),
        page_reference: Some(Reference::Id(page.page_id)),
        ..Default::default()
    });
    (site.site_id, slug, page)
}

async fn set_tag(
    runner: &TestRunner,
    site_id: i64,
    page_id: i64,
    name: &str,
    content: &str,
    all_pages: bool,
) {
    run_endpoint!(
        runner,
        page_meta_tag_set,
        json!({
            "site_id": site_id,
            "page_id": page_id,
            "name": name,
            "content": content,
            "all_pages": all_pages,
        }),
    );
}

async fn get_tags(runner: &TestRunner, site_id: i64, page_id: i64) -> Vec<PageMetaTag> {
    run_endpoint!(
        runner,
        page_meta_tags,
        json!({"site_id": site_id, "page_id": page_id}),
    )
}

#[tokio::test]
async fn page_meta_tags_support_site_and_page_round_trips() {
    let mut runner = TestRunner::setup().await;
    let (site_id, _, page) = create_page(&mut runner).await;

    set_tag(&runner, site_id, page.page_id, "robots", "index", true).await;
    set_tag(&runner, site_id, page.page_id, "a", "one", false).await;
    set_tag(&runner, site_id, page.page_id, "b", "two", false).await;
    assert_eq!(
        get_tags(&runner, site_id, page.page_id).await,
        vec![
            PageMetaTag {
                name: "robots".into(),
                content: "index".into(),
                all_pages: true
            },
            PageMetaTag {
                name: "a".into(),
                content: "one".into(),
                all_pages: false
            },
            PageMetaTag {
                name: "b".into(),
                content: "two".into(),
                all_pages: false
            },
        ],
    );

    set_tag(&runner, site_id, page.page_id, "a", "updated", false).await;
    run_endpoint!(
        runner,
        page_meta_tag_delete,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "name": "b",
            "all_pages": false,
        }),
    );
    assert_eq!(
        get_tags(&runner, site_id, page.page_id).await,
        vec![
            PageMetaTag {
                name: "robots".into(),
                content: "index".into(),
                all_pages: true
            },
            PageMetaTag {
                name: "a".into(),
                content: "updated".into(),
                all_pages: false
            },
        ],
    );

    run_endpoint!(
        runner,
        page_meta_tag_delete,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "name": "a",
            "all_pages": false,
        }),
    );
    assert_eq!(
        get_tags(&runner, site_id, page.page_id).await,
        vec![PageMetaTag {
            name: "robots".into(),
            content: "index".into(),
            all_pages: true,
        }],
    );
}

#[tokio::test]
async fn page_meta_tag_overrides_site_tag_in_effective_reads_and_article_view() {
    let mut runner = TestRunner::setup().await;
    let (site_id, slug, page) = create_page(&mut runner).await;
    set_tag(&runner, site_id, page.page_id, "description", "site", true).await;
    set_tag(&runner, site_id, page.page_id, "description", "page", false).await;

    assert_eq!(
        get_tags(&runner, site_id, page.page_id).await,
        vec![PageMetaTag {
            name: "description".into(),
            content: "page".into(),
            all_pages: false,
        }],
    );

    let article = run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": slug, "extra": ""},
            "locales": ["en"],
        }),
    );
    let GetArticleViewOutput {
        page: GetPageViewOutput::Found { meta_tags, .. },
        ..
    } = article
    else {
        panic!("new public page should produce a Found article view");
    };
    assert_eq!(meta_tags, get_tags(&runner, site_id, page.page_id).await);
}

#[tokio::test]
async fn page_meta_tag_endpoints_reject_invalid_page_and_values() {
    let mut runner = TestRunner::setup().await;
    let (site_id, _, page) = create_page(&mut runner).await;

    let invalid_page = run_endpoint_err!(
        runner,
        page_meta_tags,
        json!({"site_id": site_id, "page_id": 0}),
    );
    assert_contains_error!(invalid_page, ErrorType::Page);

    let invalid_name = run_endpoint_err!(
        runner,
        page_meta_tag_set,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "name": "bad name",
            "content": "value",
            "all_pages": false,
        }),
    );
    assert_contains_error!(invalid_name, ErrorType::BadRequest);

    let empty_content = run_endpoint_err!(
        runner,
        page_meta_tag_set,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "name": "description",
            "content": "",
            "all_pages": false,
        }),
    );
    assert_contains_error!(empty_content, ErrorType::BadRequest);
}

#[tokio::test]
async fn page_editor_cannot_mutate_site_wide_meta_tags() {
    let mut runner = TestRunner::setup().await;
    let (site_id, _, page) = create_page(&mut runner).await;
    let page_model = deepwell::services::PageService::get(
        runner.context(),
        site_id,
        Reference::Id(page.page_id),
    )
    .await
    .expect("fixture page should exist");
    let suffix = cuid();
    let user_id = UserService::create(
        runner.context(),
        CreateUser {
            user_type: UserType::Regular,
            name: format!("Edit Meta User {suffix}"),
            email: format!("edit-meta-{suffix}@example.com"),
            locales: vec![str!("en")],
            password: format!("fixture-{suffix}"),
            bypass_filter: true,
            bypass_email_verification: true,
            override_user_id: None,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("page editor should be created")
    .user_id;
    let role = RoleService::create(
        runner.context(),
        InternalCreateRoleInput {
            site_id,
            name: format!("Edit Meta Role {suffix}"),
            description: None,
            is_virtual: false,
            parent_role_id: None,
            creating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("page editor role should be created");
    PermissionService::update_permissions_for_role(
        runner.context(),
        UpdateRolePermissionsInput {
            site_id,
            role_reference: Reference::Id(role.role_id),
            new_permissions: vec![Permission {
                resource_type: Resource::Page,
                resource_category: Some(Reference::Id(page_model.page_category_id)),
                action: Action::Edit,
            }],
            cascade_removals: false,
            updating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("page edit permission should be granted");
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
    .expect("page editor role should be assigned");
    runner.set_request_context(RequestContext {
        user_id: Some(user_id),
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(page.page_id)),
        ..Default::default()
    });

    set_tag(&runner, site_id, page.page_id, "description", "page", false).await;
    assert_eq!(
        get_tags(&runner, site_id, page.page_id).await,
        vec![PageMetaTag {
            name: "description".into(),
            content: "page".into(),
            all_pages: false,
        }],
    );
    let error = run_endpoint_err!(
        runner,
        page_meta_tag_set,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "name": "robots",
            "content": "noindex",
            "all_pages": true,
        }),
    );
    assert_contains_error!(error, ErrorType::PermissionDenied);

    // Anonymous direct API requests must be denied for both page-local and
    // site-wide scope, and a denied request must leave existing metadata
    // unchanged. This exercises the Deepwell boundary without relying on the
    // Framerail control's client-side guard.
    runner.set_request_context(RequestContext {
        user_id: None,
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(page.page_id)),
        ..Default::default()
    });
    let page_write = run_endpoint_err!(
        runner,
        page_meta_tag_set,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "name": "description",
            "content": "anonymous-page-overwrite",
            "all_pages": false,
        }),
    );
    assert_contains_error!(page_write, ErrorType::PermissionDenied);

    let site_write = run_endpoint_err!(
        runner,
        page_meta_tag_set,
        json!({
            "site_id": site_id,
            "page_id": page.page_id,
            "name": "robots",
            "content": "noindex",
            "all_pages": true,
        }),
    );
    assert_contains_error!(site_write, ErrorType::PermissionDenied);

    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Id(page.page_id)),
        ..Default::default()
    });
    assert_eq!(
        get_tags(&runner, site_id, page.page_id).await,
        vec![PageMetaTag {
            name: "description".into(),
            content: "page".into(),
            all_pages: false,
        }],
        "denied anonymous requests must not change page or site metadata"
    );
}
