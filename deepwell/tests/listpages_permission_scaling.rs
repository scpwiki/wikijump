/*
 * tests/listpages_permission_scaling.rs
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

//! Scaling evidence for the batched ListPages view-permission path.
//!
//! Dimension: `R` = rows scanned by the ListPages render scan, bounded by
//! `MAX_LISTPAGES_RENDER_SCAN_ROWS` (50_000).
//!
//! Before, each row cost a fixed handful of sequential database round trips:
//! the site ban probe, the explicit-role join, the virtual-role scan, the
//! membership probe, the page lookup, the attribution query and the
//! category-scope probe. That is `Theta(R)` *checks* each costing `Theta(1)`
//! round trips, so the round-trip count is `Theta(R)` with a large constant.
//!
//! After, a scan costs one attribution query, one permission derivation per
//! distinct site, and one category probe per distinct `(site, category)`, plus
//! the individually-checked pages the viewer authored. Round trips are therefore
//! `O(1 + C + A)` in the number of distinct categories `C` and authored pages `A`,
//! both independent of `R`.
//!
//! This test measures wall clock across a doubling series and asserts both the
//! flatness of the new path and the agreement of the two paths at every size.

#[macro_use]
mod common;

use self::common::TestRunner;
use deepwell::constants::SYSTEM_USER_ID;
use deepwell::license::License;
use deepwell::services::ServiceContext;
use deepwell::services::category::CategoryService;
use deepwell::services::page::{CreatePage, PageService};
use deepwell::services::permission::{CheckPermissionContext, PermissionService};
use deepwell::services::render::{ViewablePageRef, view_decisions_for_scanned_pages};
use deepwell::services::role::{
    GrantUserRoleInput, InternalCreateRoleInput, RoleService, UpdateRolePermissionsInput,
};
use deepwell::services::site::{CreateSite, SiteService};
use deepwell::services::user::{CreateUser, UserService};
use deepwell::types::{Action, Permission, Reference, Resource, UserType};
use ftml::layout::Layout;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use str_macro::str;
use uuid::Uuid;

/// A viewer with a site-wide page:view grant, so every row is viewable and the
/// measurement is about query volume rather than about denials.
async fn create_viewing_user(ctx: &ServiceContext<'_>, site_id: i64, tag: &str) -> i64 {
    let user_id = UserService::create(
        ctx,
        CreateUser {
            user_type: UserType::Regular,
            name: format!("Perm Scale {tag}"),
            email: format!("perm-scale-{tag}@email.com"),
            locales: vec![str!("en")],
            password: String::from("password-fixture"),
            bypass_filter: true,
            bypass_email_verification: true,
            override_user_id: None,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("scaling user should be created")
    .user_id;

    let role_id = RoleService::create(
        ctx,
        InternalCreateRoleInput {
            site_id,
            name: format!("PermScaleRole{tag}"),
            description: None,
            is_virtual: false,
            parent_role_id: None,
            creating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("scaling role should be created")
    .role_id;

    PermissionService::update_permissions_for_role(
        ctx,
        UpdateRolePermissionsInput {
            site_id,
            role_reference: Reference::Id(role_id),
            new_permissions: vec![Permission {
                resource_type: Resource::Page,
                resource_category: None,
                action: Action::View,
            }],
            cascade_removals: false,
            updating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("scaling role permissions should be set");

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
    .expect("scaling role should be granted");

    user_id
}

async fn create_scaling_site(
    runner: &TestRunner,
    category_count: usize,
    page_count: usize,
) -> (i64, i64, Vec<ViewablePageRef>) {
    let ctx = runner.context();
    let tag = Uuid::new_v4().simple().to_string();

    let site = SiteService::create(
        ctx,
        CreateSite {
            slug: format!("perm-scale-{tag}"),
            name: "Permission scaling site".to_owned(),
            tagline: String::new(),
            description: "Permission scaling site".to_owned(),
            default_page: None,
            layout: None,
            license: License::CcBySa40,
            locale: String::from("en"),
            ip_address: common::IP_ADDRESS,
        },
        None,
    )
    .await
    .expect("scaling site should be created");
    let site_id = site.site_id;

    let mut categories = Vec::with_capacity(category_count);
    for i in 0..category_count {
        categories.push(
            CategoryService::get_or_create(ctx, site_id, &format!("scale-cat-{tag}-{i}"))
                .await
                .expect("scaling category should be created")
                .category_id,
        );
    }

    let viewer = create_viewing_user(ctx, site_id, &tag).await;

    let mut pages = Vec::with_capacity(page_count);
    for i in 0..page_count {
        let page_id = PageService::create(
            ctx,
            CreatePage {
                site_id,
                wikitext: String::from("scaling fixture"),
                title: format!("scale {i}"),
                alt_title: None,
                tags: Vec::new(),
                slug: format!("perm-scale-page-{tag}-{i}"),
                layout: Some(Layout::Wikidot),
                revision_comments: String::from("permission scaling"),
                user_id: SYSTEM_USER_ID,
                bypass_filter: true,
                ip_address: common::IP_ADDRESS,
            },
        )
        .await
        .expect("scaling page should be created")
        .page_id;

        // Pages are spread across the categories so the category memo has a
        // realistic number of distinct keys to resolve.
        let category_id = if categories.is_empty() {
            None
        } else {
            Some(categories[i % categories.len()])
        };

        pages.push(ViewablePageRef {
            page_id,
            site_id,
            page_category_id: category_id,
        });
    }

    (site_id, viewer, pages)
}

async fn reference_decisions(
    ctx: &ServiceContext<'_>,
    viewer: Option<i64>,
    pages: &[ViewablePageRef],
) -> Vec<bool> {
    let mut decisions = Vec::with_capacity(pages.len());
    for page in pages {
        decisions.push(
            PermissionService::check_user_can(
                ctx,
                &CheckPermissionContext {
                    user_id: viewer,
                    site_id: page.site_id,
                    page_reference: Some(Reference::Id(page.page_id)),
                },
                Permission {
                    resource_type: Resource::Page,
                    resource_category: page.page_category_id.map(Reference::Id),
                    action: Action::View,
                },
            )
            .await
            .expect("reference decision should resolve"),
        );
    }
    decisions
}

async fn time_reference(
    ctx: &ServiceContext<'_>,
    viewer: Option<i64>,
    pages: &[ViewablePageRef],
) -> Duration {
    let start = Instant::now();
    let decisions = reference_decisions(ctx, viewer, pages).await;
    let elapsed = start.elapsed();
    assert!(
        decisions.iter().all(|can_view| *can_view),
        "every fixture page should be viewable by the scaling viewer"
    );
    elapsed
}

async fn time_batched(
    ctx: &ServiceContext<'_>,
    viewer: Option<i64>,
    pages: &[ViewablePageRef],
    cache: &mut BTreeMap<(i64, Option<i64>), bool>,
) -> Duration {
    let start = Instant::now();
    let decisions = view_decisions_for_scanned_pages(ctx, viewer, pages, cache)
        .await
        .expect("batched decisions should resolve");
    let elapsed = start.elapsed();
    assert_eq!(
        decisions.len(),
        pages.len(),
        "batched path must answer every row"
    );
    assert!(
        decisions.iter().all(|can_view| *can_view),
        "every fixture page should be viewable by the scaling viewer"
    );
    elapsed
}

#[tokio::test]
async fn view_permission_scaling_is_flat_in_row_count() {
    // Four distinct categories, so the batched path's per-category work is a
    // visible constant rather than something that hides inside R.
    const CATEGORIES: usize = 4;
    let series = [10usize, 20, 40, 80, 160];

    let mut rows: Vec<(usize, Duration, Duration)> = Vec::new();

    for &r in &series {
        // A fresh runner per round keeps each fixture in its own short
        // transaction. Writing a few hundred pages inside one long-lived
        // transaction on a shared database trips Postgres HOT-chain pruning.
        let runner = TestRunner::setup().await;
        let ctx = runner.context();
        let (_site_id, viewer, pages) = create_scaling_site(&runner, CATEGORIES, r).await;

        // Warm the connection pool and the query plan cache for this shape, so
        // the first timed round is not paying one-off costs.
        let mut warm_cache: BTreeMap<(i64, Option<i64>), bool> = BTreeMap::new();
        time_batched(ctx, Some(viewer), &pages[..r.min(8)], &mut warm_cache).await;

        let reference = time_reference(ctx, Some(viewer), &pages).await;

        // A fresh cache per round: the render scan may start cold, and the
        // category probe cost must not be attributed to an earlier size.
        let mut cache: BTreeMap<(i64, Option<i64>), bool> = BTreeMap::new();
        let batched = time_batched(ctx, Some(viewer), &pages, &mut cache).await;

        // Correctness at every size: the two paths must agree row for row.
        let expected = reference_decisions(ctx, Some(viewer), &pages).await;
        let actual =
            view_decisions_for_scanned_pages(ctx, Some(viewer), &pages, &mut cache)
                .await
                .expect("verification pass should resolve");
        assert_eq!(expected, actual, "paths disagree at R={r}");

        rows.push((r, reference, batched));
    }

    let mut report = String::new();
    report.push_str("\nR      reference(ms)  batched(ms)  speedup\n");
    for (r, reference, batched) in &rows {
        report.push_str(&format!(
            "{r:<6} {:>13.2}  {:>11.2}  {:>6.1}x\n",
            reference.as_secs_f64() * 1000.0,
            batched.as_secs_f64() * 1000.0,
            reference.as_secs_f64() / batched.as_secs_f64().max(f64::MIN_POSITIVE),
        ));
    }
    eprintln!("{report}");

    // The batched path must be flat in R. Growing R by 16x while the batched
    // cost stays within a small constant factor is the observable signature of
    // round trips going from Theta(R) to O(1 + C + A).
    let (r_small, _, batched_small) = rows[0];
    let (r_large, _, batched_large) = *rows.last().expect("series should not be empty");
    assert_eq!(r_large, r_small * 16, "series should span 16x");

    let growth =
        batched_large.as_secs_f64() / batched_small.as_secs_f64().max(f64::MIN_POSITIVE);
    eprintln!(
        "batched growth over {r_small}->{r_large} rows: {growth:.2}x \
         (a per-row path would be near 16x)"
    );
    assert!(
        growth < 6.0,
        "batched path scaled {growth:.1}x while rows grew 16x, so it is not flat"
    );

    // The reference path is the thing being replaced: it must still be the slow
    // one, otherwise this test is not measuring anything.
    let (_, reference_large, batched_large) =
        *rows.last().expect("series should not be empty");
    assert!(
        reference_large > batched_large * 2,
        "expected the per-row path to cost several times more at R={r_large}"
    );
}
