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
use deepwell::constants::ADMIN_USER_ID;
use deepwell::error::prelude::*;
use deepwell::services::RequestContext;
use deepwell::services::permission::PermissionService;
use deepwell::services::role::{
    GetUserRolesInput, GrantUserRoleInput, InternalCreateRoleInput, RoleService,
    UpdateRolePermissionsInput,
};
use deepwell::types::{Action, PageRevisionType, Permission, Reference, Resource};
use serde_json::json;

struct Issue5SiteFixture {
    slug: &'static str,
    name: &'static str,
    tagline: &'static str,
    description: &'static str,
    locale: &'static str,
    preferred_domain: &'static str,
    custom_domains: &'static [(&'static str, bool)],
    boundary_title: &'static str,
    boundary_wikitext: &'static str,
}

const ISSUE5_AI_TRANSLATION_FIXTURE: Issue5SiteFixture = Issue5SiteFixture {
    slug: "ai-translation",
    name: "AI Translation QA",
    tagline: "Editable translation QA",
    description: "Editable site for generated and user translation drafts that do not mirror real SCP-JP by default.",
    locale: "ja",
    preferred_domain: "ai-translation.localhost",
    custom_domains: &[
        ("ai-translation.wikijump.dev", false),
        ("ai-translation.localhost", false),
    ],
    boundary_title: "Boundary Check: AI Translation QA",
    boundary_wikitext: "== Issue 5 boundary check ==\nThis fixture ensures AI translation pages stay isolated from SCP-JP.",
};

const ISSUE5_EDITOR_ROLE_NAME: &str = "Issue5 Translation Editor";

const ISSUE5_SCP_JP_FIXTURE: Issue5SiteFixture = Issue5SiteFixture {
    slug: "scp-jp",
    name: "SCP-JP (Mirror)",
    tagline: "SCP-JP mirror placeholder",
    description: "Reserved mirror site for pages imported from real SCP-JP only.",
    locale: "ja",
    preferred_domain: "scp-jp.localhost",
    custom_domains: &[
        ("scp-jp.wikijump.dev", false),
        ("scp-jp.localhost", false),
    ],
    boundary_title: "Boundary Check: SCP-JP Mirror",
    boundary_wikitext: "== Issue 5 boundary check ==\nThis fixture ensures mirror pages stay isolated from editable AI translations.",
};

async fn ensure_issue5_sites(runner: &mut TestRunner) -> (i64, i64) {
    let ai_site_output = run_endpoint!(
        runner,
        site_get,
        json!({"site": ISSUE5_AI_TRANSLATION_FIXTURE.slug}),
    )
    .expect("Seeded ai-translation site not found");

    let scp_site_output = run_endpoint!(
        runner,
        site_get,
        json!({"site": ISSUE5_SCP_JP_FIXTURE.slug}),
    )
    .expect("Seeded scp-jp site not found");

    let ai_boundary_output = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": ai_site_output.site.site_id,
            "page": "boundary-check",
        }),
    )
    .expect("ai-translation boundary fixture missing");

    let scp_boundary_output = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": scp_site_output.site.site_id,
            "page": "boundary-check",
        }),
    )
    .expect("scp-jp boundary fixture missing");

    assert_eq!(ai_boundary_output.title, ISSUE5_AI_TRANSLATION_FIXTURE.boundary_title);
    assert_eq!(scp_boundary_output.title, ISSUE5_SCP_JP_FIXTURE.boundary_title);

    (ai_site_output.site.site_id, scp_site_output.site.site_id)
}

async fn issue5_site_user_id(runner: &mut TestRunner, site_slug: &str) -> i64 {
    let user = run_endpoint!(
        runner,
        user_get,
        json!({ "user": format!("site:{site_slug}") }),
    )
    .expect("Could not locate generated site owner user");

    user.user.user_id
}

async fn ensure_issue5_editor_permission(
    runner: &mut TestRunner,
    site_id: i64,
    user_id: i64,
    page_category_id: i64,
) {
    let ctx = runner.context();

    let issue5_editor_role = if let Some(role) = RoleService::get_optional(
        ctx,
        site_id,
        Reference::Slug(std::borrow::Cow::Borrowed(ISSUE5_EDITOR_ROLE_NAME)),
    )
    .await
    .expect("Failed to look up Issue 5 translation editor role")
    {
        role
    } else {
        RoleService::create(
            ctx,
            InternalCreateRoleInput {
                site_id,
                name: ISSUE5_EDITOR_ROLE_NAME.to_string(),
                description: None,
                is_virtual: false,
                parent_role_id: None,
                creating_user_id: ADMIN_USER_ID,
                ip_address: common::IP_ADDRESS,
            },
        )
        .await
        .expect("Failed to create Issue 5 translation editor role")
    };

    PermissionService::update_permissions_for_role(
        ctx,
        UpdateRolePermissionsInput {
            site_id,
            role_reference: Reference::Id(issue5_editor_role.role_id),
            new_permissions: vec![
                Permission {
                    resource_type: Resource::Page,
                    resource_category: Some(Reference::Id(page_category_id)),
                    action: Action::View,
                },
                Permission {
                    resource_type: Resource::Page,
                    resource_category: Some(Reference::Id(page_category_id)),
                    action: Action::Edit,
                },
            ],
            cascade_removals: false,
            updating_user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("Failed to grant issue5 editor permissions to role");

    let has_role = RoleService::get_all_roles_for_user_and_site(
        ctx,
        GetUserRolesInput {
            site_id,
            user_id: Some(user_id),
            page_reference: None,
        },
    )
    .await
    .expect("Failed to check issue5 editor role assignments")
    .iter()
    .any(|role| role.role_id == issue5_editor_role.role_id);

    if !has_role {
        RoleService::grant_role_to_user(
            ctx,
            GrantUserRoleInput {
                site_id,
                user_id,
                role_id: issue5_editor_role.role_id,
                assigning_user_id: ADMIN_USER_ID,
                expires_at: None,
                ip_address: common::IP_ADDRESS,
            },
        )
        .await
        .expect("Failed to grant issue5 editor role to site user");
    }
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

#[tokio::test]
async fn issue5_seeded_sites_are_distinct() {
    let mut runner = TestRunner::setup().await;
    let (ai_site_id, scp_site_id) = ensure_issue5_sites(&mut runner).await;

    let ai_site_output = run_endpoint!(runner, site_get, json!({"site": "ai-translation"}))
        .expect("ai-translation site not found");
    let scp_site_output = run_endpoint!(runner, site_get, json!({"site": "scp-jp"}))
        .expect("scp-jp site not found");

    assert_eq!(ai_site_output.site.slug, "ai-translation");
    assert_eq!(scp_site_output.site.slug, "scp-jp");
    assert_eq!(ai_site_output.site.site_id, ai_site_id);
    assert_eq!(scp_site_output.site.site_id, scp_site_id);
    assert_ne!(
        ai_site_output.site.site_id,
        scp_site_output.site.site_id,
        "Site IDs are identical across boundary sites"
    );
    assert!(
        ai_site_output.site.preferred_domain.is_some(),
        "ai-translation site should have a preferred domain"
    );
    assert_eq!(
        ai_site_output.site.preferred_domain,
        Some("ai-translation.localhost".to_owned())
    );
    assert!(
        ai_site_output.domains.iter().any(|domain| domain.domain == "ai-translation.wikijump.dev"),
        "ai-translation should include wikijump.dev domain"
    );
    assert!(
        ai_site_output.domains.iter().any(|domain| domain.domain == "ai-translation.localhost"),
        "ai-translation should include localhost domain"
    );
    assert!(
        ai_site_output.domains.iter().all(|domain| domain.domain != "scp-jp.wikijump.dev"),
        "ai-translation should not share scp-jp domain"
    );
    assert!(
        scp_site_output.site.preferred_domain.is_some(),
        "scp-jp site should have a preferred domain"
    );
    assert_eq!(
        scp_site_output.site.preferred_domain,
        Some("scp-jp.localhost".to_owned())
    );
    assert!(
        scp_site_output
            .domains
            .iter()
            .any(|domain| domain.domain == "scp-jp.wikijump.dev"),
        "scp-jp should include wikijump.dev domain"
    );
    assert!(
        scp_site_output
            .domains
            .iter()
            .any(|domain| domain.domain == "scp-jp.localhost"),
        "scp-jp should include localhost domain"
    );
    assert!(
        scp_site_output.domains.iter().all(|domain| domain.domain != "ai-translation.wikijump.dev"),
        "scp-jp should not share ai-translation domain"
    );
}

#[tokio::test]
async fn issue5_seeding_is_idempotent() {
    let mut runner = TestRunner::setup().await;

    let (ai_site_id, scp_site_id) = ensure_issue5_sites(&mut runner).await;
    let (ai_site_id_repeat, scp_site_id_repeat) = ensure_issue5_sites(&mut runner).await;

    assert_eq!(ai_site_id, ai_site_id_repeat);
    assert_eq!(scp_site_id, scp_site_id_repeat);
}

#[tokio::test]
async fn issue5_ai_translation_is_editable() {
    let mut runner = TestRunner::setup().await;
    let (ai_site_id, _) = ensure_issue5_sites(&mut runner).await;
    let ai_site_user_id = issue5_site_user_id(&mut runner, ISSUE5_AI_TRANSLATION_FIXTURE.slug).await;

    const PAGE_SLUG: &str = "issue5-editable-smoke";

    let site_id = ai_site_id;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ai_site_user_id),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(PAGE_SLUG.into())),
    });

    let output = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "This page belongs to an editable local translation site.",
            "title": "Issue 5 editable smoke",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": null,
            "revision_comments": "created draft",
            "user_id": ai_site_user_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(output.slug, PAGE_SLUG);
    assert!(output.parser_errors.is_empty());
    let page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
        }),
    )
    .expect("Cannot find created page");
    ensure_issue5_editor_permission(
        &mut runner,
        site_id,
        ai_site_user_id,
        page.page_category_id,
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
    .expect("Cannot find created page");
    assert_eq!(page.site_id, site_id);
    assert_eq!(page.slug, PAGE_SLUG);
    assert_eq!(page.title, "Issue 5 editable smoke");
    assert_eq!(page.revision_type, PageRevisionType::Create);

    let output = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
            "last_revision_id": page.revision_id,
            "revision_comments": "Edit translation draft",
            "user_id": ai_site_user_id,
            "wikitext": "Edited draft body for local translation work.",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("No revision created");
    assert_eq!(output.revision_number, 1);
    assert!(output.revision_id > page.revision_id);

    let updated_page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
        }),
    )
    .expect("Cannot find edited page");
    assert_eq!(updated_page.revision_number, 1);
    assert_eq!(updated_page.site_id, site_id);
}

#[tokio::test]
async fn issue5_ai_translation_pages_do_not_leak_to_scp_jp() {
    let mut runner = TestRunner::setup().await;
    let (ai_site_id, scp_site_id) = ensure_issue5_sites(&mut runner).await;

    const AI_SITE_SLUG: &str = "ai-translation";
    const PAGE_SLUG: &str = "issue5-local-only-draft-page";

    let ai_output = run_endpoint!(runner, site_get, json!({"site": AI_SITE_SLUG}))
        .expect("Seeded site not found");
    let scp_output = run_endpoint!(runner, site_get, json!({"site": "scp-jp"}))
        .expect("Seeded site not found");
    assert_eq!(ai_output.site.site_id, ai_site_id);
    assert_eq!(scp_output.site.site_id, scp_site_id);

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(ai_site_id),
        page_reference: Some(Reference::Slug(PAGE_SLUG.into())),
    });

    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": ai_site_id,
            "wikitext": "This draft only belongs in ai-translation.",
            "title": "Issue 5 local draft",
            "alt_title": null,
            "slug": PAGE_SLUG,
            "layout": null,
            "revision_comments": "local-only draft",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let ai_page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": ai_site_id,
            "page": PAGE_SLUG,
        }),
    )
    .expect("Draft missing from ai-translation");
    assert_eq!(ai_page.site_id, ai_site_id);

    let scp_page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": scp_site_id,
            "page": PAGE_SLUG,
        }),
    );
    assert!(scp_page.is_none(), "Draft unexpectedly found in scp-jp");
}

#[tokio::test]
async fn issue5_same_slug_does_not_mix_between_ai_translation_and_scp_jp() {
    let mut runner = TestRunner::setup().await;
    let (ai_site_id, scp_site_id) = ensure_issue5_sites(&mut runner).await;

    let ai_site_output = run_endpoint!(runner, site_get, json!({"site": "ai-translation"}))
        .expect("Seeded ai-translation not found");
    let scp_site_output = run_endpoint!(runner, site_get, json!({"site": "scp-jp"}))
        .expect("Seeded scp-jp not found");
    assert_eq!(ai_site_output.site.site_id, ai_site_id);
    assert_eq!(scp_site_output.site.site_id, scp_site_id);

    let ai_boundary_page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": ai_site_output.site.site_id,
            "page": "boundary-check",
        }),
    )
    .expect("ai-translation boundary fixture missing");
    let scp_boundary_page = run_endpoint!(
        runner,
        page_get,
        json!({
            "site_id": scp_site_output.site.site_id,
            "page": "boundary-check",
        }),
    )
    .expect("scp-jp boundary fixture missing");

    assert_eq!(ai_boundary_page.slug, "boundary-check");
    assert_eq!(scp_boundary_page.slug, "boundary-check");
    assert_ne!(
        ai_boundary_page.page_id,
        scp_boundary_page.page_id,
        "Boundary pages should never reuse page IDs across sites"
    );
    assert_ne!(
        ai_boundary_page.site_id,
        scp_boundary_page.site_id,
        "Boundary pages should be isolated by site id"
    );
    assert_ne!(
        ai_boundary_page.revision_id,
        scp_boundary_page.revision_id,
        "Boundary pages should not share revision IDs"
    );
    assert_ne!(
        ai_boundary_page.title,
        scp_boundary_page.title,
        "Boundary fixtures should remain boundary-specific"
    );
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
