/*
 * tests/permission.rs
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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#[macro_use]
mod common;

use self::common::TestRunner;
use deepwell::constants::SYSTEM_USER_ID;
use deepwell::license::License;
use deepwell::services::category::CategoryService;
use deepwell::services::permission::{
    CheckPermissionContext, DecoratedPermission, PermissionService,
};
use deepwell::services::role::{
    GrantUserRoleInput, InternalCreateRoleInput, RoleService, UpdateRolePermissionsInput,
};
use deepwell::services::site::{CreateSite, SiteService};
use deepwell::services::user::{CreateUser, UserService};
use deepwell::services::{RequestContext, ServiceContext};
use deepwell::types::{Action, Permission, Reference, Resource, UserType};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use str_macro::str;

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);
const TEST_CATEGORY_NAME: &str = "test-category";
const OTHER_CATEGORY_NAME: &str = "other-category";

fn next_n() -> u64 {
    FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed)
}
struct PermissionFixture {
    site_id: i64,
    // A page category to use for testing category-scoped permissions
    category_id: i64,
    other_category_id: i64,
    user_a: i64,
    user_b: i64,
    user_c: i64,
}

impl PermissionFixture {
    async fn setup(runner: &TestRunner) -> Self {
        let ctx = runner.context();
        let n = next_n();

        let site = SiteService::create(
            ctx,
            CreateSite {
                slug: format!("perm-test-{n}"),
                name: format!("Permission test site {n}"),
                tagline: String::new(),
                description: format!("Permission test site {n}"),
                default_page: None,
                layout: None,
                license: License::CcBySa40,
                locale: String::from("en"),
                ip_address: common::IP_ADDRESS,
            },
        )
        .await
        .expect("Failed to create test site");
        let site_id = site.site_id;

        // Page category for scoped permission tests
        let category_id =
            CategoryService::get_or_create(ctx, site_id, TEST_CATEGORY_NAME)
                .await
                .expect("Failed to create page category")
                .category_id;

        // Another category to test that scoped permissions don't apply to other categories
        let other_category_id =
            CategoryService::get_or_create(ctx, site_id, OTHER_CATEGORY_NAME)
                .await
                .expect("Failed to create other page category")
                .category_id;

        // RoleA: page:view + page:edit, both unscoped
        let role_a = create_role(ctx, site_id, "RoleA", None).await;
        add_perms_to_role(
            ctx,
            site_id,
            role_a,
            vec![
                Permission {
                    resource_type: Resource::Page,
                    resource_category: None,
                    action: Action::View,
                },
                Permission {
                    resource_type: Resource::Page,
                    resource_category: None,
                    action: Action::Edit,
                },
            ],
        )
        .await;

        // RoleB: page:create + page:edit scoped to test-category only
        let role_b = create_role(ctx, site_id, "RoleB", None).await;
        add_perms_to_role(
            ctx,
            site_id,
            role_b,
            vec![
                Permission {
                    resource_type: Resource::Page,
                    resource_category: Some(Reference::Id(category_id)),
                    action: Action::Create,
                },
                Permission {
                    resource_type: Resource::Page,
                    resource_category: Some(Reference::Id(category_id)),
                    action: Action::Edit,
                },
            ],
        )
        .await;

        let user_a = create_user(ctx, n, "a").await;
        let user_b = create_user(ctx, n, "b").await;
        let user_c = create_user(ctx, n, "c").await;

        grant_role(ctx, site_id, user_a, role_a).await;
        grant_role(ctx, site_id, user_b, role_b).await;
        // user_c doesn't have any roles

        PermissionFixture {
            site_id,
            category_id,
            other_category_id,
            user_a,
            user_b,
            user_c,
        }
    }
}

// Test helpers

async fn create_role(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    name: &str,
    parent_role_id: Option<i64>,
) -> i64 {
    RoleService::create(
        ctx,
        InternalCreateRoleInput {
            site_id,
            name: name.to_owned(),
            description: None,
            is_virtual: false,
            parent_role_id,
            creating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("Failed to create role")
    .role_id
}

async fn add_perms_to_role(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    role_id: i64,
    permissions: Vec<Permission<'static>>,
) {
    PermissionService::update_permissions_for_role(
        ctx,
        UpdateRolePermissionsInput {
            site_id,
            role_reference: Reference::Id(role_id),
            new_permissions: permissions,
            cascade_removals: false,
            updating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("Failed to add permissions to role");
}

async fn grant_role(ctx: &ServiceContext<'_>, site_id: i64, user_id: i64, role_id: i64) {
    RoleService::grant_role_to_user(
        ctx,
        GrantUserRoleInput {
            site_id,
            user_id,
            role_id,
            assigning_user_id: SYSTEM_USER_ID,
            expires_at: None,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("Failed to grant role to user");
}

async fn create_user(ctx: &ServiceContext<'_>, fixture_n: u64, label: &str) -> i64 {
    UserService::create(
        ctx,
        CreateUser {
            user_type: UserType::Regular,
            name: format!("Perm Test {fixture_n} {label}"),
            email: format!("perm-{fixture_n}-{label}@email.com"),
            locales: vec![str!("en")],
            password: String::from("password"),
            bypass_filter: true,
            bypass_email_verification: true,
            override_user_id: None,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("Failed to create test user")
    .user_id
}

#[must_use]
async fn check(
    runner: &TestRunner,
    user_id: Option<i64>,
    site_id: i64,
    resource: Resource,
    category_id: Option<i64>,
    action: Action,
) -> bool {
    PermissionService::check_user_can(
        runner.context(),
        &CheckPermissionContext {
            user_id,
            site_id,
            page_reference: None,
        },
        Permission {
            resource_type: resource,
            resource_category: category_id.map(Reference::Id),
            action,
        },
    )
    .await
    .expect("Permission check returned an error")
}

#[must_use]
async fn batch_check<const N: usize>(
    runner: &TestRunner,
    user_id: Option<i64>,
    site_id: i64,
    perms: [(Resource, Option<i64>, Action); N],
) -> [bool; N] {
    let inputs = perms.map(|(resource, category_id, action)| Permission {
        resource_type: resource,
        resource_category: category_id.map(Reference::Id),
        action,
    });
    PermissionService::batch_check_user_can(
        runner.context(),
        &CheckPermissionContext {
            user_id,
            site_id,
            page_reference: None,
        },
        inputs,
    )
    .await
    .expect("Batch permission check returned an error")
}

#[tokio::test]
async fn check_user_can() {
    let runner = TestRunner::setup().await;
    let f = PermissionFixture::setup(&runner).await;

    let a = Some(f.user_a);
    let b = Some(f.user_b);
    let c = Some(f.user_c);
    let cat = Some(f.category_id);

    // Case: User with a role that grants the permission can exercise it

    // RoleA grants page:view and page:edit unscoped
    assert!(
        check(&runner, a, f.site_id, Resource::Page, None, Action::View).await,
        "user_a should pass page:view check"
    );
    assert!(
        check(&runner, a, f.site_id, Resource::Page, None, Action::Edit).await,
        "user_a should pass page:edit check"
    );

    // Case: User with no roles that grant a permission cannot exercise it

    // user_c has no roles at all
    assert!(
        !check(&runner, c, f.site_id, Resource::Page, None, Action::View).await,
        "user_c should fail page:view check"
    );
    assert!(
        !check(&runner, c, f.site_id, Resource::Page, None, Action::Edit).await,
        "user_c should fail page:edit check"
    );

    // user_b no view permission
    assert!(
        !check(&runner, b, f.site_id, Resource::Page, None, Action::View).await,
        "user_b should fail page:view check"
    );

    // Case: Permissions scoped to a category only apply within that category

    // user_b has page:edit permission scoped to the test category
    assert!(
        check(&runner, b, f.site_id, Resource::Page, cat, Action::Edit).await,
        "user_b: should pass page:edit check in test-category"
    );
    // unscoped edit should fail
    assert!(
        !check(&runner, b, f.site_id, Resource::Page, None, Action::Edit).await,
        "user_b: should fail page:edit without category"
    );
    // edit in other category should fail
    let other_cat = Some(f.other_category_id);
    assert!(
        !check(
            &runner,
            b,
            f.site_id,
            Resource::Page,
            other_cat,
            Action::Edit
        )
        .await,
        "user_b: should fail page:edit in other category"
    );

    // Since test category has scoped edit permission, user_a cannot edit it with _default edit permission
    assert!(
        !check(&runner, a, f.site_id, Resource::Page, cat, Action::Edit).await,
        "user_a: should fail page:edit check in test-category"
    );
}

#[tokio::test]
async fn batch_check_user_can() {
    let runner = TestRunner::setup().await;
    let f = PermissionFixture::setup(&runner).await;

    let a = Some(f.user_a);
    let b = Some(f.user_b);
    let c = Some(f.user_c);
    let cat = Some(f.category_id);

    // Case: user_a has both view and edit unscoped
    let [can_view, can_edit] = batch_check(
        &runner,
        a,
        f.site_id,
        [
            (Resource::Page, None, Action::View),
            (Resource::Page, None, Action::Edit),
        ],
    )
    .await;
    assert!(can_view, "user_a: batch should pass page:view");
    assert!(can_edit, "user_a: batch should pass page:edit");

    // Case: user_b has scoped edit but no view
    let [can_view, can_edit] = batch_check(
        &runner,
        b,
        f.site_id,
        [
            (Resource::Page, cat, Action::View),
            (Resource::Page, cat, Action::Edit),
        ],
    )
    .await;
    assert!(
        !can_view,
        "user_b: batch should fail page:view in test-category"
    );
    assert!(
        can_edit,
        "user_b: batch should pass page:edit in test-category"
    );

    // Case: User with no roles — all denied
    let [can_view, can_edit] = batch_check(
        &runner,
        c,
        f.site_id,
        [
            (Resource::Page, None, Action::View),
            (Resource::Page, None, Action::Edit),
        ],
    )
    .await;
    assert!(!can_view, "user_c: batch should fail page:view");
    assert!(!can_edit, "user_c: batch should fail page:edit");

    // Case: Batch and single check should return the same results
    let [batch_view, batch_edit] = batch_check(
        &runner,
        a,
        f.site_id,
        [
            (Resource::Page, None, Action::View),
            (Resource::Page, None, Action::Edit),
        ],
    )
    .await;
    assert_eq!(
        batch_view,
        check(&runner, a, f.site_id, Resource::Page, None, Action::View).await,
        "batch and single check differ on page:view"
    );
    assert_eq!(
        batch_edit,
        check(&runner, a, f.site_id, Resource::Page, None, Action::Edit).await,
        "batch and single check differ on page:edit"
    );
}

#[tokio::test]
async fn check_category_resolution() {
    let runner = TestRunner::setup().await;
    let f = PermissionFixture::setup(&runner).await;

    // Permission check should be able to resolve category name to ID
    assert!(
        PermissionService::check_user_can(
            runner.context(),
            &CheckPermissionContext {
                user_id: Some(f.user_b),
                site_id: f.site_id,
                page_reference: None
            },
            Permission {
                resource_type: Resource::Page,
                resource_category: Some(Reference::from(TEST_CATEGORY_NAME)),
                action: Action::Edit,
            },
        )
        .await
        .expect("Permission check returned an error"),
        "user_b should have page:edit permission for test-category"
    )
}

#[tokio::test]
async fn check_permission_endpoint() {
    let mut runner = TestRunner::setup().await;
    let f = PermissionFixture::setup(&runner).await;

    runner.set_request_context(RequestContext {
        user_id: Some(f.user_b),
        site_id: Some(f.site_id),
        page_reference: Some(Reference::Slug(std::borrow::Cow::Borrowed(
            "test-category:test-page",
        ))),
        ..Default::default()
    });
    let page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": f.site_id,
            "wikitext": "Test",
            "title": "Test Page",
            "alt_title": null,
            "slug": "test-category:test-page",
            "layout": null,
            "revision_comments": "",
            "user_id": f.user_b,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    // Check permissions for user_b via the endpoint, should allow
    runner.set_request_context(RequestContext {
        user_id: Some(f.user_b),
        site_id: Some(f.site_id),
        page_reference: Some(Reference::Id(page.page_id)),
        ..Default::default()
    });
    assert!(
        run_endpoint!(runner, page_edit_permission).can_edit,
        "user_b should have edit permission for page in test-category"
    );

    // Same test but with slug instead of page_id, should still work
    runner.set_request_context(RequestContext {
        user_id: Some(f.user_b),
        site_id: Some(f.site_id),
        page_reference: Some(Reference::Slug(std::borrow::Cow::Owned(page.slug.clone()))),
        ..Default::default()
    });
    assert!(
        run_endpoint!(runner, page_edit_permission).can_edit,
        "user_b should have edit permission for page in test-category"
    );

    // Check permissions for user_c via the endpoint, should deny due to no page permissions
    runner.set_request_context(RequestContext {
        user_id: Some(f.user_c),
        site_id: Some(f.site_id),
        page_reference: Some(Reference::Id(page.page_id)),
        ..Default::default()
    });
    assert!(
        !run_endpoint!(runner, page_edit_permission).can_edit,
        "user_c should NOT have edit permission for page in test-category"
    );

    // Same test but with slug instead of page_id, should still work
    runner.set_request_context(RequestContext {
        user_id: Some(f.user_c),
        site_id: Some(f.site_id),
        page_reference: Some(Reference::Slug(std::borrow::Cow::Owned(page.slug.clone()))),
        ..Default::default()
    });
    assert!(
        !run_endpoint!(runner, page_edit_permission).can_edit,
        "user_c should NOT have edit permission for page in test-category"
    );
}

#[tokio::test]
async fn role_update_permissions_and_get() {
    let runner = TestRunner::setup().await;
    let f = PermissionFixture::setup(&runner).await;

    const CATEGORY_NAME: &str = "TestCategory";
    const OTHER_CATEGORY_NAME: &str = "OtherCategory";

    // Create some categories with names
    let category_id =
        CategoryService::get_or_create(runner.context(), f.site_id, CATEGORY_NAME)
            .await
            .expect("Failed to create page category")
            .category_id;

    let other_category_id =
        CategoryService::get_or_create(runner.context(), f.site_id, OTHER_CATEGORY_NAME)
            .await
            .expect("Failed to create other page category")
            .category_id;

    let role = RoleService::create(
        runner.context(),
        InternalCreateRoleInput {
            site_id: f.site_id,
            name: str!("Test Role"),
            description: None,
            is_virtual: false,
            parent_role_id: None,
            creating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("Failed to create role");

    // Assign permissions with different resource categories
    // Using category names in the input to test that they get resolved correctly
    PermissionService::update_permissions_for_role(
        runner.context(),
        UpdateRolePermissionsInput {
            site_id: f.site_id,
            role_reference: Reference::Id(role.role_id),
            new_permissions: vec![
                Permission {
                    resource_type: Resource::Page,
                    resource_category: Some(Reference::Slug(CATEGORY_NAME.into())),
                    action: Action::View,
                },
                Permission {
                    resource_type: Resource::Page,
                    resource_category: Some(Reference::Slug(OTHER_CATEGORY_NAME.into())),
                    action: Action::Edit,
                },
            ],
            cascade_removals: false,
            updating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("Failed to update role permissions");

    // Get permissions with raw category IDs
    let perms = run_endpoint!(
        runner,
        get_role_permissions,
        json!({
            "site_id": f.site_id,
            "role_reference": role.role_id,
            "human_readable_categories": false,
        }),
    );

    assert_eq!(perms.len(), 2);
    let view_perm = perms
        .iter()
        .find(|p| p.action == Action::View)
        .expect("Expected to find view permission");
    let edit_perm = perms
        .iter()
        .find(|p| p.action == Action::Edit)
        .expect("Expected to find edit permission");

    // Assert that the resource categories were resolved to IDs
    assert_eq!(
        view_perm.resource_category,
        Some(Reference::Id(category_id))
    );
    assert_eq!(
        edit_perm.resource_category,
        Some(Reference::Id(other_category_id))
    );

    // Get permissions with human-readable categories
    let perms = run_endpoint!(
        runner,
        get_role_permissions,
        json!({
            "site_id": f.site_id,
            "role_reference": role.role_id,
            "human_readable_categories": true,
        }),
    );

    assert_eq!(perms.len(), 2);
    let view_perm = perms
        .iter()
        .find(|p| p.action == Action::View)
        .expect("Expected to find view permission");
    let edit_perm = perms
        .iter()
        .find(|p| p.action == Action::Edit)
        .expect("Expected to find edit permission");

    // Assert that the resource categories were resolved to human-readable slugs
    assert_eq!(
        view_perm.resource_category,
        Some(Reference::Slug(CATEGORY_NAME.into()))
    );
    assert_eq!(
        edit_perm.resource_category,
        Some(Reference::Slug(OTHER_CATEGORY_NAME.into()))
    );
}

#[tokio::test]
async fn get_decorated_permissions_for_role() {
    let runner = TestRunner::setup().await;
    let f = PermissionFixture::setup(&runner).await;
    let ctx = runner.context();

    // Parent role: page:view + page:edit
    let parent_id = create_role(ctx, f.site_id, "Parent", None).await;
    add_perms_to_role(
        ctx,
        f.site_id,
        parent_id,
        vec![
            Permission {
                resource_type: Resource::Page,
                resource_category: None,
                action: Action::View,
            },
            Permission {
                resource_type: Resource::Page,
                resource_category: None,
                action: Action::Edit,
            },
        ],
    )
    .await;

    // Child role: page:view only
    let child_id = create_role(ctx, f.site_id, "Child", Some(parent_id)).await;
    add_perms_to_role(
        ctx,
        f.site_id,
        child_id,
        vec![Permission {
            resource_type: Resource::Page,
            resource_category: None,
            action: Action::View,
        }],
    )
    .await;

    // Helper to find a permission in the list
    let find = |list: &Vec<DecoratedPermission<'static>>,
                resource: Resource,
                action: Action| {
        list.iter()
            .find(|d| {
                d.permission.resource_type == resource && d.permission.action == action
            })
            .unwrap_or_else(|| panic!("Permission {resource}:{action} not found"))
            .clone()
    };

    // Calling endpoint on child role
    let child_decorated = run_endpoint!(
        runner,
        get_decorated_role_permissions,
        json!({
            "site_id": f.site_id,
            "role_reference": child_id,
            "human_readable_categories": false,
        }),
    );

    // Page:View: active + removable
    let p = find(&child_decorated, Resource::Page, Action::View);
    assert!(
        p.active && p.removable && !p.addable,
        "child page:view: expected active+removable"
    );

    // Page:Edit: inactive + addable
    let p = find(&child_decorated, Resource::Page, Action::Edit);
    assert!(
        !p.active && p.addable && !p.removable,
        "child page:edit: expected inactive+addable"
    );

    // Page:Create: inactive, not addable
    let p = find(&child_decorated, Resource::Page, Action::Create);
    assert!(
        !p.active && !p.addable && !p.removable,
        "child page:create: expected inactive+locked"
    );

    // Calling endpoint on parent role
    let parent_decorated = run_endpoint!(
        runner,
        get_decorated_role_permissions,
        json!({
            "site_id": f.site_id,
            "role_reference": parent_id,
            "human_readable_categories": false,
        }),
    );

    // Page:View: active + not removable because child has it
    let p = find(&parent_decorated, Resource::Page, Action::View);
    assert!(
        p.active && !p.removable && !p.addable,
        "parent page:view: expected active+locked"
    );

    // Page:Edit: active + removable
    let p = find(&parent_decorated, Resource::Page, Action::Edit);
    assert!(
        p.active && p.removable && !p.addable,
        "parent page:edit: expected active+removable"
    );

    // Page:Create: inactive + addable as root role has no parent, so all base permissions are addable
    let p = find(&parent_decorated, Resource::Page, Action::Create);
    assert!(
        !p.active && p.addable && !p.removable,
        "parent page:create: expected inactive+addable"
    );

    // Check nonexistent permission type
    assert_panics!(|| find(&parent_decorated, Resource::Site, Action::Assign));
}
