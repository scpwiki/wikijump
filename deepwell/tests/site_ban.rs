/*
 * tests/site_ban.rs
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

use self::common::{TestRunner, latest_audit_event};
use deepwell::constants::{
    ADMIN_USER_ID, SAMPLE_USER_ID, SYSTEM_USER_ID, UNKNOWN_USER_ID,
};
use deepwell::error::prelude::*;
use deepwell::models::relation::{self, Entity as Relation};
use deepwell::services::RelationService;
use deepwell::services::role::{InternalCreateRoleInput, RoleService};
use deepwell::types::RelationType;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde_json::json;

const SITE_SLUG: &str = "test";

async fn test_site_id(runner: &TestRunner) -> i64 {
    run_endpoint!(runner, site_get, json!({ "site": SITE_SLUG }))
        .expect("Seeded test site not found")
        .site
        .site_id
}

async fn clear_site_ban(runner: &TestRunner, site_id: i64, user_id: i64) {
    let current = run_endpoint!(
        runner,
        site_ban_get,
        json!({
            "site_id": site_id,
            "user_id": user_id,
        }),
    );

    if current.is_some() {
        run_endpoint!(
            runner,
            site_ban_remove,
            json!({
                "site_id": site_id,
                "user_id": user_id,
                "removed_by": ADMIN_USER_ID,
                "reason": "Test site ban removal",
                "ip_address": common::IP_ADDRESS,
            }),
        );
    }
}

async fn clear_site_membership(runner: &TestRunner, site_id: i64, user_id: i64) {
    let current = run_endpoint!(
        runner,
        membership_get,
        json!({
            "site_id": site_id,
            "user_id": user_id,
        }),
    );

    if current.is_some() {
        run_endpoint!(
            runner,
            membership_remove,
            json!({
                "site_id": site_id,
                "user_id": user_id,
                "removed_by": ADMIN_USER_ID,
            }),
        );
    }
}

#[tokio::test]
async fn lifecycle_membership_blocking_and_audit() {
    let runner = TestRunner::setup().await;
    let site_id = test_site_id(&runner).await;
    let user_id = SAMPLE_USER_ID;

    const REASON: &str = "site-ban integration test";

    clear_site_ban(&runner, site_id, user_id).await;
    clear_site_membership(&runner, site_id, user_id).await;

    let role = RoleService::create(
        runner.context(),
        InternalCreateRoleInput {
            site_id,
            name: String::from("Site Ban Test Role"),
            description: None,
            is_virtual: false,
            parent_role_id: None,
            creating_user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("Failed to create site-ban test role");

    run_endpoint!(
        runner,
        grant_role_to_user,
        json!({
            "site_id": site_id,
            "user_id": user_id,
            "role_id": role.role_id,
            "assigning_user_id": ADMIN_USER_ID,
            "expires_at": null,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    // A malformed request must fail before creating a ban.
    let error = run_endpoint_err!(
        runner,
        site_ban_set,
        json!({
            "site_id": site_id,
            "user_id": user_id,
            "metadata": {
                "banned_until": null,
                "reason": "missing IP address",
            },
            "created_by": ADMIN_USER_ID,
        }),
    );

    assert_contains_error!(error, ErrorType::SiteBanRelation);

    let ban = run_endpoint!(
        runner,
        site_ban_get,
        json!({
            "site_id": site_id,
            "user_id": user_id,
        }),
    );
    assert!(ban.is_none(), "Malformed request created a site ban");

    // Add the user as a member before banning them.
    run_endpoint!(
        runner,
        membership_set,
        json!({
            "site_id": site_id,
            "user_id": user_id,
            "metadata": {
                "accepted": {
                    "cause": "accepted",
                    "user_id": ADMIN_USER_ID,
                },
            },
            "created_by": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let membership = run_endpoint!(
        runner,
        membership_get,
        json!({
            "site_id": site_id,
            "user_id": user_id,
        }),
    );
    assert!(membership.is_some(), "Site membership was not created");

    // Creating the ban must remove the existing membership.
    run_endpoint!(
        runner,
        site_ban_set,
        json!({
            "site_id": site_id,
            "user_id": user_id,
            "metadata": {
                "banned_until": null,
                "reason": REASON,
            },
            "created_by": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let ban = run_endpoint!(
        runner,
        site_ban_get,
        json!({
            "site_id": site_id,
            "user_id": user_id,
        }),
    )
    .expect("Site ban was not created");

    assert_eq!(ban.from_id, user_id);
    assert_eq!(ban.dest_id, site_id);
    assert_eq!(ban.metadata["banned_until"], json!(null));
    assert_eq!(ban.metadata["reason"], json!(REASON));

    let membership = run_endpoint!(
        runner,
        membership_get,
        json!({
            "site_id": site_id,
            "user_id": user_id,
        }),
    );
    assert!(
        membership.is_none(),
        "Banning the user did not remove their site membership",
    );

    let roles = run_endpoint!(
        runner,
        get_user_roles,
        json!({
            "site_id": site_id,
            "user_id": user_id,
        }),
    );

    assert!(
        roles.iter().all(|item| item.role_id != role.role_id),
        "Banning the user did not remove their site role",
    );

    // A banned user must not be able to become a member again.
    let error = run_endpoint_err!(
        runner,
        membership_set,
        json!({
            "site_id": site_id,
            "user_id": user_id,
            "metadata": {
                "accepted": {
                    "cause": "accepted",
                    "user_id": ADMIN_USER_ID,
                },
            },
            "created_by": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    assert_contains_error!(error, ErrorType::SiteBannedUser);

    let membership = run_endpoint!(
        runner,
        membership_get,
        json!({
            "site_id": site_id,
            "user_id": user_id,
        }),
    );
    assert!(
        membership.is_none(),
        "Failed membership attempt left a partial relation",
    );

    // A banned user must not receive a new site role.
    let error = run_endpoint_err!(
        runner,
        grant_role_to_user,
        json!({
            "site_id": site_id,
            "user_id": user_id,
            "role_id": role.role_id,
            "assigning_user_id": ADMIN_USER_ID,
            "expires_at": null,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    assert_contains_error!(error, ErrorType::SiteBannedUser);

    let roles = run_endpoint!(
        runner,
        get_user_roles,
        json!({
            "site_id": site_id,
            "user_id": user_id,
        }),
    );

    assert!(
        roles.iter().all(|item| item.role_id != role.role_id),
        "Failed role-grant attempt restored the removed role",
    );

    // Verify the ban creation audit event.
    let create_event =
        latest_audit_event(&runner, "site_ban.create", site_id, user_id).await;

    assert_eq!(create_event.ip_address, common::IP_ADDRESS.to_string());
    assert_eq!(create_event.user_id, Some(user_id));
    assert_eq!(create_event.site_id, Some(site_id));
    assert_eq!(create_event.extra_id_1, Some(ADMIN_USER_ID));
    assert_eq!(create_event.extra_string_1.as_deref(), Some(REASON));
    assert_eq!(create_event.extra_string_2, None);

    // Removing the ban must soft-delete it and add another audit event.
    let removed = run_endpoint!(
        runner,
        site_ban_remove,
        json!({
            "site_id": site_id,
            "user_id": user_id,
            "removed_by": ADMIN_USER_ID,
            "reason": "Test site ban removal",
            "ip_address": common::IP_ADDRESS,
        }),
    );

    assert_eq!(removed.deleted_by, Some(ADMIN_USER_ID));
    assert!(removed.deleted_at.is_some());

    let ban = run_endpoint!(
        runner,
        site_ban_get,
        json!({
            "site_id": site_id,
            "user_id": user_id,
        }),
    );
    assert!(ban.is_none(), "Removed site ban is still active");

    let remove_event =
        latest_audit_event(&runner, "site_ban.remove", site_id, user_id).await;

    assert_eq!(remove_event.ip_address, common::IP_ADDRESS.to_string());
    assert_eq!(remove_event.user_id, Some(user_id));
    assert_eq!(remove_event.site_id, Some(site_id));
    assert_eq!(remove_event.extra_id_1, Some(ADMIN_USER_ID));
    assert_eq!(
        remove_event.extra_string_1.as_deref(),
        Some("Test site ban removal")
    );
    assert_eq!(remove_event.extra_string_2, None);
}

#[tokio::test]
async fn expiration_cleanup_preserves_future_and_permanent_bans() {
    let runner = TestRunner::setup().await;
    let site_id = test_site_id(&runner).await;
    let user_id = UNKNOWN_USER_ID;

    clear_site_ban(&runner, site_id, user_id).await;
    clear_site_membership(&runner, site_id, user_id).await;

    // Banning a non-member must succeed.
    run_endpoint!(
        runner,
        site_ban_set,
        json!({
            "site_id": site_id,
            "user_id": user_id,
            "metadata": {
                "banned_until": "2000-01-01",
                "reason": "expired site-ban test",
            },
            "created_by": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let lifted = RelationService::lift_expired_site_bans(runner.context())
        .await
        .expect("Failed to lift expired site bans");

    assert!(lifted >= 1, "Expired site ban was not lifted");

    let ban = run_endpoint!(
        runner,
        site_ban_get,
        json!({
            "site_id": site_id,
            "user_id": user_id,
        }),
    );
    assert!(ban.is_none(), "Expired site ban is still active");

    let expired_relation = Relation::find()
        .filter(relation::Column::RelationType.eq(RelationType::SiteBan))
        .filter(relation::Column::DestId.eq(site_id))
        .filter(relation::Column::FromId.eq(user_id))
        .order_by_desc(relation::Column::RelationId)
        .one(runner.context().transaction())
        .await
        .expect("Unable to query expired site-ban relation")
        .expect("Expired site-ban relation was not found");

    assert_eq!(expired_relation.deleted_by, Some(SYSTEM_USER_ID));
    assert!(
        expired_relation.deleted_at.is_some(),
        "Expired site ban was not soft-deleted",
    );
    let expiry_event =
        latest_audit_event(&runner, "site_ban.remove", site_id, user_id).await;

    assert_eq!(expiry_event.ip_address, "::1");
    assert_eq!(expiry_event.user_id, Some(user_id));
    assert_eq!(expiry_event.site_id, Some(site_id));
    assert_eq!(expiry_event.extra_id_1, Some(SYSTEM_USER_ID));
    assert_eq!(
        expiry_event.extra_string_1.as_deref(),
        Some("Site ban expired")
    );
    assert_eq!(expiry_event.extra_string_2, None);

    // A future ban must survive the cleanup operation.
    run_endpoint!(
        runner,
        site_ban_set,
        json!({
            "site_id": site_id,
            "user_id": user_id,
            "metadata": {
                "banned_until": "2999-01-01",
                "reason": "future site-ban test",
            },
            "created_by": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    RelationService::lift_expired_site_bans(runner.context())
        .await
        .expect("Failed to run cleanup for future site ban");

    let future_ban = run_endpoint!(
        runner,
        site_ban_get,
        json!({
            "site_id": site_id,
            "user_id": user_id,
        }),
    )
    .expect("Future site ban was incorrectly lifted");

    assert_eq!(future_ban.metadata["banned_until"], json!("2999-01-01"));

    run_endpoint!(
        runner,
        site_ban_remove,
        json!({
            "site_id": site_id,
            "user_id": user_id,
            "removed_by": ADMIN_USER_ID,
            "reason": "Test site ban removal",
            "ip_address": common::IP_ADDRESS,
        }),
    );

    // A permanent ban must also survive cleanup.
    run_endpoint!(
        runner,
        site_ban_set,
        json!({
            "site_id": site_id,
            "user_id": user_id,
            "metadata": {
                "banned_until": null,
                "reason": "permanent site-ban test",
            },
            "created_by": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    RelationService::lift_expired_site_bans(runner.context())
        .await
        .expect("Failed to run cleanup for permanent site ban");

    let permanent_ban = run_endpoint!(
        runner,
        site_ban_get,
        json!({
            "site_id": site_id,
            "user_id": user_id,
        }),
    )
    .expect("Permanent site ban was incorrectly lifted");

    assert_eq!(permanent_ban.metadata["banned_until"], json!(null));
}
