/*
 * tests/site.rs
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
use deepwell::constants::SYSTEM_USER_ID;
use deepwell::error::prelude::*;
use deepwell::license::License;
use deepwell::services::RequestContext;
use deepwell::services::permission::PermissionService;
use deepwell::services::role::{
    GrantUserRoleInput, InternalCreateRoleInput, RoleService, UpdateRolePermissionsInput,
};
use deepwell::services::site::{CreateSite, SiteService};
use deepwell::services::user::{CreateUser, UserService};
use deepwell::types::{Action, Permission, Reference, Resource, UserType};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use str_macro::str;

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_n() -> u64 {
    FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

async fn create_user(runner: &TestRunner, n: u64, label: &str) -> i64 {
    UserService::create(
        runner.context(),
        CreateUser {
            user_type: UserType::Regular,
            name: format!("Site Test {n} {label}"),
            email: format!("site-test-{n}-{label}@email.com"),
            locales: vec![str!("en")],
            password: String::from("password"),
            bypass_filter: true,
            bypass_email_verification: true,
            override_user_id: None,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("failed to create test user")
    .user_id
}

async fn grant_site_edit(runner: &TestRunner, site_id: i64, user_id: i64, n: u64) {
    let role = RoleService::create(
        runner.context(),
        InternalCreateRoleInput {
            site_id,
            name: format!("Site Editor {n}"),
            description: None,
            is_virtual: false,
            parent_role_id: None,
            creating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("failed to create site editor role");

    PermissionService::update_permissions_for_role(
        runner.context(),
        UpdateRolePermissionsInput {
            site_id,
            role_reference: Reference::Id(role.role_id),
            new_permissions: vec![Permission {
                resource_type: Resource::Site,
                resource_category: None,
                action: Action::Edit,
            }],
            cascade_removals: false,
            updating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("failed to grant site edit permission to role");

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
    .expect("failed to grant site editor role to user");
}

async fn create_site(runner: &TestRunner, n: u64) -> i64 {
    SiteService::create(
        runner.context(),
        CreateSite {
            slug: format!("site-update-permission-{n}"),
            name: format!("Site update permission {n}"),
            tagline: String::new(),
            description: format!("Site update permission {n}"),
            default_page: None,
            layout: None,
            license: License::CcBySa40,
            locale: String::from("en"),
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("failed to create test site")
    .site_id
}

#[tokio::test]
async fn site_update_without_request_actor_does_not_reveal_site_existence() {
    let runner = TestRunner::setup().await;

    let error = run_endpoint_err!(
        runner,
        site_update,
        json!({
            "site": -1_i64,
            "user_id": SYSTEM_USER_ID,
            "name": "Anonymous site rename",
            "ip_address": common::IP_ADDRESS,
        }),
    );

    assert_contains_error!(error, ErrorType::PermissionDenied);
}

#[tokio::test]
async fn site_update_without_site_edit_does_not_reveal_missing_site_ids() {
    let mut runner = TestRunner::setup().await;
    let n = next_n();
    let user_id = create_user(&runner, n, "probe").await;
    runner.set_request_context(RequestContext {
        user_id: Some(user_id),
        ..Default::default()
    });

    let error = run_endpoint_err!(
        runner,
        site_update,
        json!({
            "site": -1_i64,
            "user_id": SYSTEM_USER_ID,
            "name": "Probe site rename",
            "ip_address": common::IP_ADDRESS,
        }),
    );

    assert_contains_error!(error, ErrorType::PermissionDenied);
}

#[tokio::test]
async fn site_update_requires_site_edit_permission() {
    let mut runner = TestRunner::setup().await;
    let n = next_n();
    let site_id = create_site(&runner, n).await;
    let user_id = create_user(&runner, n, "unauthorized").await;
    runner.set_request_context(RequestContext {
        user_id: Some(user_id),
        ..Default::default()
    });

    let error = run_endpoint_err!(
        runner,
        site_update,
        json!({
            "site": site_id,
            "user_id": SYSTEM_USER_ID,
            "name": "Unauthorized site rename",
            "ip_address": common::IP_ADDRESS,
        }),
    );

    assert_contains_error!(error, ErrorType::PermissionDenied);

    let site = run_endpoint!(runner, site_get, json!({ "site": site_id }))
        .expect("test site should still exist");
    assert_eq!(site.site.name, format!("Site update permission {n}"));
}

#[tokio::test]
async fn site_update_allows_users_with_site_edit_permission() {
    let mut runner = TestRunner::setup().await;
    let n = next_n();
    let site_id = create_site(&runner, n).await;
    let user_id = create_user(&runner, n, "editor").await;
    grant_site_edit(&runner, site_id, user_id, n).await;
    runner.set_request_context(RequestContext {
        user_id: Some(user_id),
        ..Default::default()
    });

    let updated = run_endpoint!(
        runner,
        site_update,
        json!({
            "site": site_id,
            "user_id": SYSTEM_USER_ID,
            "name": "Authorized site rename",
            "ip_address": common::IP_ADDRESS,
        }),
    );

    assert_eq!(updated.site_id, site_id);
    assert_eq!(updated.name, "Authorized site rename");
}
