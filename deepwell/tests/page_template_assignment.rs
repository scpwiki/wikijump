/*
 * tests/page_template_assignment.rs
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
use deepwell::constants::{ADMIN_USER_ID, SAMPLE_USER_ID, SYSTEM_USER_ID};
use deepwell::error::ErrorType;
use deepwell::license::License;
use deepwell::models::page::{Entity as PageTable, Model as PageModel};
use deepwell::services::RequestContext;
use deepwell::services::SessionService;
use deepwell::services::category::CategoryService;
use deepwell::services::page::CreatePageOutput;
use deepwell::services::page::{CreatePage, PageService};
use deepwell::services::permission::{CheckPermissionContext, PermissionService};
use deepwell::services::role::{
    GrantUserRoleInput, InternalCreateRoleInput, RoleService, UpdateRolePermissionsInput,
};
use deepwell::services::session::CreateSession;
use deepwell::services::site::{CreateSite, SiteService};
use deepwell::services::view::{
    GetArticleViewOutput, GetPageViewOutput, PageTemplateSummary,
};
use deepwell::types::{Action, Permission, Reference, Resource};
use ftml::layout::Layout;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, Set};
use serde_json::json;
use std::borrow::Cow;
use std::collections::BTreeMap;

fn set_page_actor(runner: &mut TestRunner, site_id: i64, slug: &str) {
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Owned(slug.to_owned()))),
    });
}

async fn create_page(
    runner: &mut TestRunner,
    site_id: i64,
    slug: &str,
    wikitext: &str,
) -> CreatePageOutput {
    set_page_actor(runner, site_id, slug);
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": wikitext,
            "title": slug,
            "alt_title": null,
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "page template assignment fixture",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    )
}

async fn create_template_fixture_page(
    runner: &TestRunner,
    site_id: i64,
    slug: &str,
    wikitext: &str,
) {
    PageService::create(
        runner.context(),
        CreatePage {
            site_id,
            wikitext: wikitext.to_owned(),
            title: slug.to_owned(),
            alt_title: None,
            tags: Vec::new(),
            slug: slug.to_owned(),
            layout: Some(Layout::Wikidot),
            revision_comments: String::from("component notice template fixture"),
            user_id: SYSTEM_USER_ID,
            bypass_filter: true,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("component template fixture page should be created");
}

async fn create_component_notice_fixture_site(
    runner: &TestRunner,
    slug: &str,
    notice: &str,
) -> i64 {
    let site = SiteService::create(
        runner.context(),
        CreateSite {
            slug: slug.to_owned(),
            name: format!("{slug} fixture"),
            tagline: String::new(),
            description: String::from("Isolated component notice template fixture"),
            default_page: None,
            layout: Some(Layout::Wikidot),
            license: License::CcBySa40,
            locale: String::from("en"),
            ip_address: common::IP_ADDRESS,
        },
        None,
    )
    .await
    .expect("isolated component notice fixture site should be created");
    let site_id = site.site_id;
    let everyone_role = RoleService::create(
        runner.context(),
        InternalCreateRoleInput {
            site_id,
            name: String::from("everyone"),
            description: Some(String::from("View access for fixture site visitors")),
            is_virtual: true,
            parent_role_id: None,
            creating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("fixture site should have an everyone role");
    PermissionService::update_permissions_for_role(
        runner.context(),
        UpdateRolePermissionsInput {
            site_id,
            role_reference: Reference::Id(everyone_role.role_id),
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
    .expect("fixture site should grant everyone permission to view pages");

    create_template_fixture_page(
        runner,
        site_id,
        "component:_template",
        &format!(
            "[[div class=\"warning-top-box\"]]\n+* NOTICE:\n{notice}\n[[/div]]\n%%content%%"
        ),
    )
    .await;
    create_template_fixture_page(
        runner,
        site_id,
        "component:probe",
        "ORIGINAL COMPONENT BODY",
    )
    .await;
    create_template_fixture_page(
        runner,
        site_id,
        "article",
        "[[include component:probe]]",
    )
    .await;

    site_id
}

async fn page_view_source_and_html(
    runner: &TestRunner,
    site_id: i64,
    slug: &str,
) -> (String, String) {
    match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": { "slug": slug, "extra": "" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            wikitext,
            compiled_body_html,
            ..
        } => (wikitext, compiled_body_html),
        other => panic!("expected fixture page view, got {other:?}"),
    }
}

async fn mark_page_imported(runner: &TestRunner, page_id: i64) -> PageModel {
    let page = PageTable::find_by_id(page_id)
        .one(runner.context().transaction())
        .await
        .expect("imported page lookup should not fail")
        .expect("imported page should exist");
    let mut page = page.into_active_model();
    page.from_wikidot = Set(true);
    page.update(runner.context().transaction())
        .await
        .expect("page should be marked as imported")
}

async fn anonymous_article_html_and_cache_key(
    runner: &TestRunner,
    site_id: i64,
    slug: &str,
) -> (String, String) {
    match run_endpoint!(
        runner,
        article_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": { "slug": slug, "extra": "" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetArticleViewOutput {
            page:
                GetPageViewOutput::Found {
                    compiled_body_html, ..
                },
            article_page_cache_key: Some(cache_key),
            ..
        } => (compiled_body_html, cache_key),
        other => panic!("expected cacheable anonymous article view, got {other:?}"),
    }
}

async fn missing_page_template(
    runner: &TestRunner,
    site_id: i64,
    slug: &str,
    extra: &str,
    session_token: Option<&str>,
) -> (Option<String>, Vec<PageTemplateSummary>, Option<i64>) {
    match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": slug, "extra": extra },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Missing {
            new_page_wikitext,
            page_templates,
            selected_template_page_id,
            ..
        } => (new_page_wikitext, page_templates, selected_template_page_id),
        other => panic!("expected a missing-page view, got {other:?}"),
    }
}

async fn grant_category_permission(
    runner: &TestRunner,
    site_id: i64,
    category_id: i64,
    role_name: &str,
    action: Action,
    user_ids: &[i64],
) {
    let role = RoleService::create(
        runner.context(),
        InternalCreateRoleInput {
            site_id,
            name: role_name.to_owned(),
            description: None,
            is_virtual: false,
            parent_role_id: None,
            creating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("category permission role should be created");
    PermissionService::update_permissions_for_role(
        runner.context(),
        UpdateRolePermissionsInput {
            site_id,
            role_reference: Reference::Id(role.role_id),
            new_permissions: vec![Permission {
                resource_type: Resource::Page,
                resource_category: Some(Reference::Id(category_id)),
                action,
            }],
            cascade_removals: false,
            updating_user_id: SYSTEM_USER_ID,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("category permission should be updated");
    for &user_id in user_ids {
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
        .expect("category permission role should be granted");
    }
}

#[tokio::test]
async fn category_page_template_prefills_new_page_source_and_can_be_cleared() {
    const TEMPLATE_SOURCE: &str = "ORACLE-TEMPLATE-BEGIN\nTitle: %%title%%\nContent: %%content%%\nORACLE-TEMPLATE-END";

    let mut runner = TestRunner::setup().await;
    let site_id = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site
        .site_id;
    let admin_session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "deepwell page template assignment test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");
    let template = create_page(
        &mut runner,
        site_id,
        "template:page-template-assignment",
        TEMPLATE_SOURCE,
    )
    .await;
    let ordinary = create_page(
        &mut runner,
        site_id,
        "ordinary-page-template-source",
        "not a template",
    )
    .await;
    let alternate_template = create_page(
        &mut runner,
        site_id,
        "template:page-template-assignment-alternate",
        "ALTERNATE-TEMPLATE-SOURCE",
    )
    .await;
    let category = CategoryService::get_or_create(
        runner.context(),
        site_id,
        "page-template-assignment-target",
    )
    .await
    .expect("target category should be created");
    let template_category =
        CategoryService::get(runner.context(), site_id, Reference::from("template"))
            .await
            .expect("template category should exist");
    let sample_session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: SAMPLE_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "deepwell page template permission test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("registered non-member session should be created");
    let (no_create_source, no_create_templates, no_create_template_page_id) =
        missing_page_template(
            &runner,
            site_id,
            "page-template-assignment-target:no-create-page",
            "/edit/true",
            Some(&sample_session_token),
        )
        .await;
    assert_eq!(no_create_source, None);
    assert!(no_create_templates.is_empty());
    assert_eq!(no_create_template_page_id, None);
    grant_category_permission(
        &runner,
        site_id,
        category.category_id,
        "page-template-assignment-creators",
        Action::Create,
        &[ADMIN_USER_ID, SAMPLE_USER_ID],
    )
    .await;
    grant_category_permission(
        &runner,
        site_id,
        template_category.category_id,
        "page-template-assignment-template-viewers",
        Action::View,
        &[ADMIN_USER_ID],
    )
    .await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });

    let rejected = run_endpoint_err!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": ordinary.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_contains_error!(rejected, ErrorType::PageCategory);

    let assigned = run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(assigned.template_page_id, Some(template.page_id));
    let (initial_source, page_templates, selected_template_page_id) =
        missing_page_template(
            &runner,
            site_id,
            "page-template-assignment-target:new-page",
            "/edit/true",
            Some(&admin_session_token),
        )
        .await;
    assert_eq!(initial_source.as_deref(), Some(TEMPLATE_SOURCE));
    assert_eq!(selected_template_page_id, Some(template.page_id));
    assert!(page_templates.iter().any(|candidate| {
        candidate.page_id == template.page_id && candidate.wikitext == TEMPLATE_SOURCE
    }));

    let forced_extra = format!("/edit/true/t/{}", alternate_template.page_id);
    let (forced_source, _, forced_template_page_id) = missing_page_template(
        &runner,
        site_id,
        "page-template-assignment-target:forced-page",
        &forced_extra,
        Some(&admin_session_token),
    )
    .await;
    assert_eq!(forced_source.as_deref(), Some("ALTERNATE-TEMPLATE-SOURCE"));
    assert_eq!(forced_template_page_id, Some(alternate_template.page_id));

    let (anonymous_source, anonymous_templates, anonymous_template_page_id) =
        missing_page_template(
            &runner,
            site_id,
            "page-template-assignment-target:anonymous-page",
            "/edit/true",
            None,
        )
        .await;
    assert_eq!(anonymous_source, None);
    assert!(anonymous_templates.is_empty());
    assert_eq!(anonymous_template_page_id, None);

    let sample_user_can_create = PermissionService::check_user_can(
        runner.context(),
        &CheckPermissionContext {
            user_id: Some(SAMPLE_USER_ID),
            site_id,
            page_reference: None,
        },
        Permission {
            resource_type: Resource::Page,
            resource_category: Some(Reference::Id(category.category_id)),
            action: Action::Create,
        },
    )
    .await
    .expect("sample-user create permission should be checked");
    assert!(sample_user_can_create);
    let (no_view_source, no_view_templates, no_view_template_page_id) =
        missing_page_template(
            &runner,
            site_id,
            "page-template-assignment-target:no-template-view-page",
            "/edit/true",
            Some(&sample_session_token),
        )
        .await;
    assert_eq!(no_view_source, None);
    assert!(no_view_templates.is_empty());
    assert_eq!(no_view_template_page_id, None);

    let default_category =
        CategoryService::get(runner.context(), site_id, Reference::from("_default"))
            .await
            .expect("seeded default category should exist");
    let assigned_default = run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": default_category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(assigned_default.template_page_id, Some(template.page_id));
    let (default_source, _, default_template_page_id) = missing_page_template(
        &runner,
        site_id,
        "page-template-assignment-default-page",
        "/edit/true",
        Some(&admin_session_token),
    )
    .await;
    assert_eq!(default_source.as_deref(), Some(TEMPLATE_SOURCE));
    assert_eq!(default_template_page_id, Some(template.page_id));

    let cleared = run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": null,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(cleared.template_page_id, None);
    assert_eq!(
        missing_page_template(
            &runner,
            site_id,
            "page-template-assignment-target:another-page",
            "/edit/true",
            Some(&admin_session_token),
        )
        .await
        .0,
        None,
    );

    let cleared_default = run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": default_category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": null,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(cleared_default.template_page_id, None);
}

#[tokio::test]
async fn page_view_exposes_category_data_form_definition_in_template_order() {
    const CATEGORY: &str = "data-form-create-flow";
    const TEMPLATE_SOURCE: &str = concat!(
        "[[form]]\n",
        "fields:\n",
        "  name:\n",
        "    label: Name\n",
        "    type: text\n",
        "  choice:\n",
        "    label: Choice\n",
        "    type: select\n",
        "    values:\n",
        "      a: Alpha\n",
        "      b: Beta\n",
        "    default: b\n",
        "[[/form]]",
    );

    let mut runner = TestRunner::setup().await;
    let site_id = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site
        .site_id;
    let session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "deepwell data-form create-flow test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");
    let template = create_page(
        &mut runner,
        site_id,
        "data-form-create-flow:_template",
        TEMPLATE_SOURCE,
    )
    .await;
    let category = CategoryService::get_or_create(runner.context(), site_id, CATEGORY)
        .await
        .expect("data-form target category should be created");
    grant_category_permission(
        &runner,
        site_id,
        category.category_id,
        "data-form-create-flow-creators",
        Action::Create,
        &[ADMIN_USER_ID],
    )
    .await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    for extra in ["", "/edit"] {
        match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": session_token,
                "route": {
                    "slug": "data-form-create-flow:_template",
                    "extra": extra
                },
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                data_form: None,
                wikitext,
                compiled_body_html,
                ..
            } => {
                assert_eq!(wikitext, TEMPLATE_SOURCE);
                assert!(!compiled_body_html.contains("form-table"));
            }
            other => {
                panic!("category template must retain ordinary view/edit: {other:?}")
            }
        }
    }

    let definition = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": {
                "slug": "data-form-create-flow:example",
                "extra": "/edit/true"
            },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Missing {
            data_form: Some(data_form),
            ..
        } => data_form.definition,
        other => panic!("expected missing data-form page definition, got {other:?}"),
    };

    assert_eq!(
        definition
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        ["name", "choice"],
        "Wikidot emits form-fields in category-template field order",
    );
    assert!(definition.default_layout);
    assert_eq!(definition.fields[0].label, "Name");
    assert_eq!(definition.fields[0].field_type.as_deref(), Some("text"));
    assert_eq!(definition.fields[0].default_value, None);
    assert_eq!(definition.fields[1].label, "Choice");
    assert_eq!(definition.fields[1].field_type.as_deref(), Some("select"));
    assert_eq!(definition.fields[1].default_value.as_deref(), Some("b"));
    assert_eq!(
        definition.fields[1]
            .values
            .iter()
            .map(|value| (value.value.as_str(), value.label.as_str()))
            .collect::<Vec<_>>(),
        [("a", "Alpha"), ("b", "Beta")],
    );

    create_page(
        &mut runner,
        site_id,
        "data-form-create-flow:saved",
        "name: 'Probe Name'\nchoice: a",
    )
    .await;
    let edit_definition = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": {
                "slug": "data-form-create-flow:saved",
                "extra": "/edit"
            },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: Some(data_form),
            ..
        } => data_form,
        other => panic!("expected saved data-form page definition, got {other:?}"),
    };
    assert_eq!(edit_definition.definition, definition);
    assert_eq!(
        edit_definition.values,
        BTreeMap::from([
            ("choice".to_owned(), "a".to_owned()),
            ("name".to_owned(), "Probe Name".to_owned()),
        ]),
    );

    match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {
                "slug": "data-form-create-flow:saved",
                "extra": "/edit"
            },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: None, ..
        } => {}
        other => panic!("view-only actors must not receive editor metadata: {other:?}"),
    }

    let rendered_html = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {
                "slug": "data-form-create-flow:saved",
                "extra": ""
            },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: None,
            compiled_body_html,
            ..
        } => compiled_body_html,
        other => panic!("expected rendered data-form page, got {other:?}"),
    };
    assert!(
        rendered_html.contains(concat!(
            "<table class=\"form-table\">",
            "<tbody><tr class=\"form-row\">",
            "<td class=\"form-labels\"><span class=\"form-label\">Name</span></td>",
            "<td class=\"form-values\"><span>Probe Name</span></td>",
            "</tr>",
            "<tr class=\"form-row\">",
            "<td class=\"form-labels\"><span class=\"form-label\">Choice</span></td>",
            "<td class=\"form-values\"><span>Alpha</span></td>",
            "</tr>",
            "</tbody></table>",
        )),
        "saved data-form fields must render with Wikidot's table DOM:\n{rendered_html}",
    );

    for (slug, source) in [
        (
            "data-form-create-flow:legacy",
            "name: 'Probe Name'\nlegacy: x\nchoice: a",
        ),
        (
            "data-form-create-flow:blank-line",
            "name: 'Probe Name'\n\nchoice: a",
        ),
    ] {
        create_page(&mut runner, site_id, slug, source).await;
        for extra in ["", "/edit"] {
            match run_endpoint!(
                runner,
                page_view,
                json!({
                    "site_id": site_id,
                    "session_token": session_token,
                    "route": { "slug": slug, "extra": extra },
                    "locales": ["en-US", "en"],
                }),
            ) {
                GetPageViewOutput::Found {
                    data_form: None,
                    wikitext,
                    compiled_body_html,
                    ..
                } => {
                    assert_eq!(wikitext, source);
                    assert!(!compiled_body_html.contains("form-table"));
                }
                other => panic!("unrecognized stored source must fail closed: {other:?}"),
            }
        }
    }

    match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": {
                "slug": "data-form-create-flow:saved",
                "extra": "/p/2"
            },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: None,
            compiled_body_html,
            ..
        } => assert!(!compiled_body_html.contains("form-table")),
        other => panic!("non-bare routes must preserve route rendering: {other:?}"),
    }
}

#[tokio::test]
async fn page_view_and_edit_round_trip_date_field_scalars_and_options() {
    const CATEGORY: &str = "data-form-date-field-public";
    const TEMPLATE_SLUG: &str = "data-form-date-field-public:_template";
    const PAGE_SLUG: &str = "data-form-date-field-public:date";
    const TEMPLATE_SOURCE: &str = concat!(
        "[[form]]\n",
        "fields:\n",
        "  date:\n",
        "    label: Date value\n",
        "    width: 24\n",
        "    type: date\n",
        "    options:\n",
        "      dateFormat: 'mm/dd/yy'\n",
        "      showOn: button\n",
        "[[/form]]",
    );

    let mut runner = TestRunner::setup().await;
    let site_id = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site
        .site_id;
    let session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "deepwell data-form date-field test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");
    let template =
        create_page(&mut runner, site_id, TEMPLATE_SLUG, TEMPLATE_SOURCE).await;
    let category = CategoryService::get_or_create(runner.context(), site_id, CATEGORY)
        .await
        .expect("date-form category should be created");
    grant_category_permission(
        &runner,
        site_id,
        category.category_id,
        "data-form-date-field-creators",
        Action::Create,
        &[ADMIN_USER_ID],
    )
    .await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let definition = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": PAGE_SLUG, "extra": "/edit/true" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Missing {
            data_form: Some(data_form),
            ..
        } => data_form.definition,
        other => {
            panic!("date field must be exposed by the public create view: {other:?}")
        }
    };
    let date = definition.field("date").expect("date field definition");
    assert!(definition.supports_observed_create_edit());
    assert_eq!(date.width, 24);
    assert_eq!(date.options["dateFormat"], json!("mm/dd/yy"));
    assert_eq!(date.options["showOn"], json!("button"));

    let page = create_page(&mut runner, site_id, PAGE_SLUG, "date: 02/29/2023").await;
    match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": PAGE_SLUG, "extra": "/edit" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: Some(data_form),
            ..
        } => {
            assert_eq!(data_form.values["date"], "02/29/2023");
        }
        other => {
            panic!("date field must round-trip through the public edit view: {other:?}")
        }
    }

    set_page_actor(&mut runner, site_id, PAGE_SLUG);
    run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": PAGE_SLUG,
            "last_revision_id": page.revision_id,
            "revision_comments": "round trip leap date",
            "user_id": ADMIN_USER_ID,
            "wikitext": "date: 02/29/2024",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("date field edit should save through the public page seam");
    match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": PAGE_SLUG, "extra": "/edit" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: Some(data_form),
            wikitext,
            ..
        } => {
            assert_eq!(wikitext, "date: 02/29/2024");
            assert_eq!(data_form.values["date"], "02/29/2024");
        }
        other => {
            panic!("saved date field must reload through the public edit view: {other:?}")
        }
    }
}

#[tokio::test]
async fn component_notice_templates_are_site_scoped_and_include_pure() {
    const SOURCE: &str = "ORIGINAL COMPONENT BODY";
    const INCLUDE_SOURCE: &str = "[[include component:probe]]";
    const NOTICE_A: &str = "SITE A COMPONENT NOTICE";
    const NOTICE_B: &str = "SITE B COMPONENT NOTICE";

    let runner = TestRunner::setup().await;
    let site_a = create_component_notice_fixture_site(
        &runner,
        "component-notice-site-a",
        NOTICE_A,
    )
    .await;
    let site_b = create_component_notice_fixture_site(
        &runner,
        "component-notice-site-b",
        NOTICE_B,
    )
    .await;

    for (site_id, own_notice, other_notice) in
        [(site_a, NOTICE_A, NOTICE_B), (site_b, NOTICE_B, NOTICE_A)]
    {
        let (component_source, component_html) =
            page_view_source_and_html(&runner, site_id, "component:probe").await;
        assert_eq!(component_source, SOURCE);
        assert!(component_html.contains("warning-top-box"));
        assert!(component_html.contains(own_notice));
        assert!(!component_html.contains(other_notice));
        assert!(component_html.contains(SOURCE));

        let (article_source, article_html) =
            page_view_source_and_html(&runner, site_id, "article").await;
        assert_eq!(article_source, INCLUDE_SOURCE);
        assert!(article_html.contains(SOURCE));
        assert!(!article_html.contains("warning-top-box"));
        assert!(!article_html.contains(own_notice));
        assert!(!article_html.contains(other_notice));
    }
}

#[tokio::test]
async fn page_view_exposes_file_data_form_create_control_without_write_lifecycle() {
    const CATEGORY: &str = "data-form-file-field-public";
    const TEMPLATE_SLUG: &str = "data-form-file-field-public:_template";
    const PAGE_SLUG: &str = "data-form-file-field-public:target";
    const TEMPLATE_SOURCE: &str = concat!(
        "[[form]]\n",
        "fields:\n",
        "  document:\n",
        "    type: file\n",
        "    label: Upload document\n",
        "[[/form]]",
    );

    let mut runner = TestRunner::setup().await;
    let site_id = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site
        .site_id;
    let session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "deepwell data-form file-field test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");
    let template =
        create_page(&mut runner, site_id, TEMPLATE_SLUG, TEMPLATE_SOURCE).await;
    let category = CategoryService::get_or_create(runner.context(), site_id, CATEGORY)
        .await
        .expect("file-form category should be created");
    grant_category_permission(
        &runner,
        site_id,
        category.category_id,
        "data-form-file-field-creators",
        Action::Create,
        &[ADMIN_USER_ID],
    )
    .await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": PAGE_SLUG, "extra": "/edit/true" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Missing {
            data_form: Some(data_form),
            ..
        } => {
            assert!(data_form.definition.supports_observed_editor());
            assert!(!data_form.definition.supports_observed_create_edit());
            assert_eq!(
                data_form
                    .definition
                    .field("document")
                    .and_then(|field| field.field_type.as_deref()),
                Some("file"),
            );
            assert!(data_form.values.is_empty());
        }
        other => {
            panic!("file field must expose the observed create-form boundary: {other:?}")
        }
    }
}

#[tokio::test]
async fn assigned_data_form_template_edit_invalidates_warm_imported_article_cache() {
    const CATEGORY: &str = "data-form-template-cache-lifecycle";
    const TEMPLATE_SLUG: &str = "data-form-template-cache-lifecycle:_template";
    const PAGE_SLUG: &str = "data-form-template-cache-lifecycle:saved";
    const TEMPLATE_SOURCE_A: &str = concat!(
        "[[form]]\n",
        "fields:\n",
        "  name:\n",
        "    label: Lifecycle label A\n",
        "    type: text\n",
        "[[/form]]",
    );
    const TEMPLATE_SOURCE_B: &str = concat!(
        "[[form]]\n",
        "fields:\n",
        "  name:\n",
        "    label: Lifecycle label B\n",
        "    type: text\n",
        "[[/form]]",
    );

    let mut runner = TestRunner::setup().await;
    let site_id = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site
        .site_id;
    let template =
        create_page(&mut runner, site_id, TEMPLATE_SLUG, TEMPLATE_SOURCE_A).await;
    let category = CategoryService::get_or_create(runner.context(), site_id, CATEGORY)
        .await
        .expect("data-form lifecycle category should be created");
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let page =
        create_page(&mut runner, site_id, PAGE_SLUG, "name: 'Lifecycle value'").await;
    mark_page_imported(&runner, page.page_id).await;
    runner.set_request_context(RequestContext::default());

    let (cold_html, cold_cache_key) =
        anonymous_article_html_and_cache_key(&runner, site_id, PAGE_SLUG).await;
    assert!(cold_html.contains("Lifecycle label A"));
    assert!(cold_html.contains("Lifecycle value"));

    let (warm_html, warm_cache_key) =
        anonymous_article_html_and_cache_key(&runner, site_id, PAGE_SLUG).await;
    assert_eq!(warm_html, cold_html);
    assert_eq!(warm_cache_key, cold_cache_key);

    set_page_actor(&mut runner, site_id, TEMPLATE_SLUG);
    run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": TEMPLATE_SLUG,
            "last_revision_id": template.revision_id,
            "revision_comments": "replace lifecycle data-form label",
            "user_id": ADMIN_USER_ID,
            "wikitext": TEMPLATE_SOURCE_B,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("template edit should create a revision");
    runner.set_request_context(RequestContext::default());

    let (edited_html, edited_cache_key) =
        anonymous_article_html_and_cache_key(&runner, site_id, PAGE_SLUG).await;
    assert!(
        edited_html.contains("Lifecycle label B"),
        "saved template edits must replace warm anonymous imported-page output:\n{edited_html}",
    );
    assert!(!edited_html.contains("Lifecycle label A"));
    assert!(edited_html.contains("Lifecycle value"));
    assert_ne!(
        edited_cache_key, cold_cache_key,
        "assigned template revisions must participate in the Deepwell article cache identity",
    );
}

#[tokio::test]
async fn data_form_template_deletion_requires_actual_form_removal() {
    const CATEGORY: &str = "data-form-template-deletion";
    const TEMPLATE_SLUG: &str = "data-form-template-deletion:_template";
    const PAGE_SLUG: &str = "data-form-template-deletion:saved";
    const FORMLESS_PAGE_SLUG: &str = "data-form-template-deletion:formless";
    const SAVED_SOURCE: &str = "name: 'Retained value'";
    const FORMLESS_SOURCE: &str = "name: 'Formless value'";
    const ACTIVE_TEMPLATE_SOURCE: &str = concat!(
        "[[form]]\n",
        "fields:\n",
        "  name:\n",
        "    label: Retained label\n",
        "    type: text\n",
        "[[/form]]",
    );
    const COMMENTED_TEMPLATE_SOURCE: &str = concat!(
        "[!--\n",
        "[[form]]\n",
        "fields:\n",
        "  name:\n",
        "    label: Retained label\n",
        "    type: text\n",
        "[[/form]]\n",
        "--]",
    );
    const RENAMED_TEMPLATE_SOURCE: &str = concat!(
        "[[x-form]]\n",
        "fields:\n",
        "  name:\n",
        "    label: Retained label\n",
        "    type: text\n",
        "[[/x-form]]",
    );
    const RESTORED_TEMPLATE_SOURCE: &str = concat!(
        "[[form]]\n",
        "fields:\n",
        "  name:\n",
        "    label: Restored label\n",
        "    type: text\n",
        "[[/form]]",
    );

    let mut runner = TestRunner::setup().await;
    let site_id = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site
        .site_id;
    let session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "deepwell data-form template deletion test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");
    let template =
        create_page(&mut runner, site_id, TEMPLATE_SLUG, ACTIVE_TEMPLATE_SOURCE).await;
    let category = CategoryService::get_or_create(runner.context(), site_id, CATEGORY)
        .await
        .expect("data-form deletion category should be created");
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    create_page(&mut runner, site_id, PAGE_SLUG, SAVED_SOURCE).await;

    match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": PAGE_SLUG, "extra": "/edit" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: Some(data_form),
            wikitext,
            ..
        } => {
            assert_eq!(wikitext, SAVED_SOURCE);
            assert_eq!(data_form.values["name"], "Retained value");
        }
        other => {
            panic!("active category template must expose the data-form editor: {other:?}")
        }
    }

    set_page_actor(&mut runner, site_id, TEMPLATE_SLUG);
    let commented = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": TEMPLATE_SLUG,
            "last_revision_id": template.revision_id,
            "revision_comments": "comment data-form template without deleting it",
            "user_id": ADMIN_USER_ID,
            "wikitext": COMMENTED_TEMPLATE_SOURCE,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("commented template edit should create a revision");

    match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": PAGE_SLUG, "extra": "/edit" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: Some(data_form),
            wikitext,
            ..
        } => {
            assert_eq!(wikitext, SAVED_SOURCE);
            assert_eq!(data_form.values["name"], "Retained value");
        }
        other => panic!(
            "commenting out [[form]] must not delete the category data form: {other:?}"
        ),
    }

    set_page_actor(&mut runner, site_id, TEMPLATE_SLUG);
    let removed = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": TEMPLATE_SLUG,
            "last_revision_id": commented.revision_id,
            "revision_comments": "rename data-form construct to delete it",
            "user_id": ADMIN_USER_ID,
            "wikitext": RENAMED_TEMPLATE_SOURCE,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("renamed template edit should create a revision");

    match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": PAGE_SLUG, "extra": "/edit" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: None,
            wikitext,
            ..
        } => assert_eq!(wikitext, SAVED_SOURCE),
        other => {
            panic!("renaming [[form]] must restore the ordinary page editor: {other:?}")
        }
    }
    match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": "data-form-template-deletion:new", "extra": "/edit/true" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Missing {
            data_form: None, ..
        } => {}
        other => {
            panic!("renamed [[form]] must disable the generated create editor: {other:?}")
        }
    }

    create_page(&mut runner, site_id, FORMLESS_PAGE_SLUG, FORMLESS_SOURCE).await;
    match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": FORMLESS_PAGE_SLUG, "extra": "/edit" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: None,
            wikitext,
            ..
        } => assert_eq!(wikitext, FORMLESS_SOURCE),
        other => panic!(
            "a page created while the form is absent must use the ordinary editor: {other:?}"
        ),
    }

    set_page_actor(&mut runner, site_id, TEMPLATE_SLUG);
    run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": TEMPLATE_SLUG,
            "last_revision_id": removed.revision_id,
            "revision_comments": "restore data-form construct on the same template page",
            "user_id": ADMIN_USER_ID,
            "wikitext": RESTORED_TEMPLATE_SOURCE,
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("restored template edit should create a revision");

    for (slug, expected_value) in [
        (PAGE_SLUG, "Retained value"),
        (FORMLESS_PAGE_SLUG, "Formless value"),
    ] {
        match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": session_token,
                "route": { "slug": slug, "extra": "/edit" },
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                data_form: Some(data_form),
                ..
            } => assert_eq!(data_form.values["name"], expected_value),
            other => panic!(
                "restoring [[form]] on the same template page must re-enable the generated editor for {slug}: {other:?}"
            ),
        }
    }

    match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": "data-form-template-deletion:restored-new", "extra": "/edit/true" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Missing {
            data_form: Some(data_form),
            ..
        } => assert_eq!(
            data_form
                .definition
                .field("name")
                .map(|field| field.label.as_str()),
            Some("Restored label"),
        ),
        other => panic!(
            "restored [[form]] must re-enable the generated create editor: {other:?}"
        ),
    }
}

#[tokio::test]
async fn page_view_exposes_live_text_and_select_control_contract() {
    const CATEGORY: &str = "data-form-control-contract";
    const TEMPLATE_SOURCE: &str = concat!(
        "[[form]]\n",
        "fields:\n",
        "  plain:\n",
        "    label: Plain text\n",
        "    width: 0\n",
        "    default: \"**bold** #hash\"\n",
        "  multi:\n",
        "    label: Multi line\n",
        "    type: text\n",
        "    width: 50\n",
        "    height: 3\n",
        "  matched:\n",
        "    label: Matched text\n",
        "    type: text\n",
        "    hint: enter a color like \\#468259\n",
        "    match: /^ok-[0-9]+$/\n",
        "    match-error: Use ok- followed by digits\n",
        "    join: true\n",
        "    before: PRE\n",
        "    after: POST\n",
        "  missing_values:\n",
        "    label: Missing values\n",
        "    type: select\n",
        "  empty_values:\n",
        "    label: Empty values\n",
        "    type: select\n",
        "    values:\n",
        "  select_one:\n",
        "    label: Select one\n",
        "    type: select\n",
        "    hint: ignored select hint\n",
        "    before: PRE\n",
        "    after: POST\n",
        "    values:\n",
        "      a: Alpha\n",
        "  select_five:\n",
        "    label: Select five\n",
        "    type: select\n",
        "    values:\n",
        "      0: Zero\n",
        "      1: One\n",
        "      2: Two\n",
        "      3: Three\n",
        "      4: Four\n",
        "    default: 4\n",
        "  reserved:\n",
        "    label: Reserved labels\n",
        "    type: select\n",
        "    values:\n",
        "      no_value: No\n",
        "      yes_value: Yes\n",
        "      false_value: False\n",
        "      true_value: True\n",
        "  quoted:\n",
        "    label: Quoted labels\n",
        "    type: select\n",
        "    values:\n",
        "      no_value: \"No\"\n",
        "      yes_value: \"Yes\"\n",
        "      false_value: \"False\"\n",
        "      true_value: \"True\"\n",
        "[[/form]]",
    );
    const SAVED_SOURCE: &str = r#"plain: 'O''Brien: # [x] \ slash "quote"'
multi: "first \"quoted\"\nsecond 'single' \\ end"
matched: ok-42
missing_values: null
empty_values: null
select_one: a
select_five: '2'
reserved: no_value
quoted: false_value"#;

    let mut runner = TestRunner::setup().await;
    let site_id = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site
        .site_id;
    let session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "deepwell data-form text/select controls test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");
    let template = create_page(
        &mut runner,
        site_id,
        "data-form-control-contract:_template",
        TEMPLATE_SOURCE,
    )
    .await;
    let category = CategoryService::get_or_create(runner.context(), site_id, CATEGORY)
        .await
        .expect("data-form target category should be created");
    grant_category_permission(
        &runner,
        site_id,
        category.category_id,
        "data-form-control-contract-creators",
        Action::Create,
        &[ADMIN_USER_ID],
    )
    .await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let definition = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": {
                "slug": "data-form-control-contract:new",
                "extra": "/edit/true"
            },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Missing {
            data_form: Some(data_form),
            ..
        } => data_form.definition,
        other => {
            panic!("expected live-backed data-form control definition, got {other:?}")
        }
    };

    assert_eq!(
        definition
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        [
            "plain",
            "multi",
            "matched",
            "missing_values",
            "empty_values",
            "select_one",
            "select_five",
            "reserved",
            "quoted",
        ],
        "select fields without usable values stay in storage order",
    );
    let plain = definition.field("plain").expect("plain field");
    assert_eq!(plain.field_type.as_deref(), Some("text"));
    assert_eq!(plain.width, 1);
    assert_eq!(plain.height, 1);
    assert_eq!(plain.default_value.as_deref(), Some("**bold** #hash"));
    let multi = definition.field("multi").expect("multiline field");
    assert_eq!(multi.width, 50);
    assert_eq!(multi.height, 3);
    let matched = definition.field("matched").expect("matched field");
    assert_eq!(matched.width, 40);
    assert_eq!(matched.height, 1);
    assert_eq!(matched.hint, "enter a color like \\#468259");
    assert_eq!(matched.match_pattern.as_deref(), Some("/^ok-[0-9]+$/"));
    assert_eq!(
        matched.match_error.as_deref(),
        Some("Use ok- followed by digits"),
    );
    assert!(matched.join);
    assert_eq!(matched.before, "PRE");
    assert_eq!(matched.after, "POST");
    let select_one = definition.field("select_one").expect("one-value select");
    assert_eq!(select_one.hint, "ignored select hint");
    assert_eq!(select_one.before, "PRE");
    assert_eq!(select_one.after, "POST");
    assert_eq!(select_one.values.len(), 1,);
    assert_eq!(
        definition
            .field("select_five")
            .expect("five-value select")
            .values
            .len(),
        5,
    );
    assert_eq!(
        definition
            .field("reserved")
            .expect("unquoted reserved labels")
            .values
            .iter()
            .map(|value| value.label.as_str())
            .collect::<Vec<_>>(),
        ["No", "Yes"],
    );
    assert_eq!(
        definition
            .field("quoted")
            .expect("quoted reserved labels")
            .values
            .iter()
            .map(|value| value.label.as_str())
            .collect::<Vec<_>>(),
        ["No", "Yes", "False", "True"],
    );

    create_page(
        &mut runner,
        site_id,
        "data-form-control-contract:saved",
        SAVED_SOURCE,
    )
    .await;
    let editor = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": {
                "slug": "data-form-control-contract:saved",
                "extra": "/edit"
            },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: Some(data_form),
            ..
        } => data_form,
        other => panic!("expected stored live-backed data-form values, got {other:?}"),
    };
    assert_eq!(editor.definition, definition);
    assert_eq!(
        editor.values,
        BTreeMap::from([
            ("matched".to_owned(), "ok-42".to_owned()),
            ("missing_values".to_owned(), String::new()),
            ("empty_values".to_owned(), String::new()),
            (
                "multi".to_owned(),
                "first \"quoted\"\nsecond 'single' \\ end".to_owned(),
            ),
            (
                "plain".to_owned(),
                "O'Brien: # [x] \\ slash \"quote\"".to_owned(),
            ),
            ("quoted".to_owned(), "false_value".to_owned()),
            ("reserved".to_owned(), "no_value".to_owned()),
            ("select_five".to_owned(), "2".to_owned()),
            ("select_one".to_owned(), "a".to_owned()),
        ]),
    );

    let rendered_html = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {
                "slug": "data-form-control-contract:saved",
                "extra": ""
            },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: None,
            compiled_body_html,
            ..
        } => compiled_body_html,
        other => panic!("expected saved data-form public rendering, got {other:?}"),
    };
    for expected in [
        r#"<table class="form-table">"#,
        r#"<tr class="form-row">"#,
        r#"<td class="form-labels"><span class="form-label">Plain text</span></td>"#,
        r#"<td class="form-values"><span>"#,
    ] {
        assert!(
            rendered_html.contains(expected),
            "saved data-form HTML must retain the live-observed CSS hook {expected:?}: {rendered_html}",
        );
    }
    assert!(
        !rendered_html.contains("row-1") && !rendered_html.contains("field-plain"),
        "default text/select saved output must not invent documentation-only row/value hooks absent from the live capture: {rendered_html}",
    );

    for (slug, source) in [
        (
            "data-form-control-contract:unquoted-number",
            SAVED_SOURCE.replace("select_five: '2'", "select_five: 2"),
        ),
        (
            "data-form-control-contract:unsafe-plain-text",
            SAVED_SOURCE.replace(
                r#"plain: 'O''Brien: # [x] \ slash "quote"'"#,
                "plain: unquoted text",
            ),
        ),
    ] {
        create_page(&mut runner, site_id, slug, &source).await;
        match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": session_token,
                "route": {
                    "slug": slug,
                    "extra": "/edit"
                },
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                data_form: None,
                wikitext,
                ..
            } => assert_eq!(wikitext, source),
            other => panic!("non-canonical stored scalars must fail closed: {other:?}"),
        }
    }

    const UNSUPPORTED_SELECT_PROPERTY_CATEGORY: &str =
        "data-form-unsupported-select-property";
    let unsupported_template = create_page(
        &mut runner,
        site_id,
        "data-form-unsupported-select-property:_template",
        concat!(
            "[[form]]\n",
            "fields:\n",
            "  choice:\n",
            "    label: Choice\n",
            "    type: select\n",
            "    width: 40\n",
            "    values:\n",
            "      a: Alpha\n",
            "[[/form]]",
        ),
    )
    .await;
    let unsupported_category = CategoryService::get_or_create(
        runner.context(),
        site_id,
        UNSUPPORTED_SELECT_PROPERTY_CATEGORY,
    )
    .await
    .expect("unsupported-property category should be created");
    grant_category_permission(
        &runner,
        site_id,
        unsupported_category.category_id,
        "data-form-unsupported-select-property-creators",
        Action::Create,
        &[ADMIN_USER_ID],
    )
    .await;
    run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": unsupported_category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": unsupported_template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": {
                "slug": "data-form-unsupported-select-property:new",
                "extra": "/edit/true"
            },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Missing {
            data_form: None, ..
        } => {}
        other => panic!("text-only properties on selects must fail closed: {other:?}"),
    }

    const DIMENSION_CATEGORY: &str = "data-form-dimension-boundaries";
    let dimension_template = create_page(
        &mut runner,
        site_id,
        "data-form-dimension-boundaries:_template",
        concat!(
            "[[form]]\n",
            "fields:\n",
            "  omitted:\n",
            "    label: Omitted\n",
            "  empty:\n",
            "    label: Empty\n",
            "    width:\n",
            "    height:\n",
            "  nonnumeric:\n",
            "    label: Nonnumeric\n",
            "    width: nope\n",
            "    height: nope\n",
            "  zero:\n",
            "    label: Zero\n",
            "    width: 0\n",
            "    height: 0\n",
            "  negative:\n",
            "    label: Negative\n",
            "    width: -1\n",
            "    height: -1\n",
            "  one:\n",
            "    label: One\n",
            "    height: 1\n",
            "  two:\n",
            "    label: Two\n",
            "    height: 2\n",
            "[[/form]]",
        ),
    )
    .await;
    let dimension_category =
        CategoryService::get_or_create(runner.context(), site_id, DIMENSION_CATEGORY)
            .await
            .expect("dimension-boundary category should be created");
    grant_category_permission(
        &runner,
        site_id,
        dimension_category.category_id,
        "data-form-dimension-boundary-creators",
        Action::Create,
        &[ADMIN_USER_ID],
    )
    .await;
    run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": dimension_category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": dimension_template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    let dimensions = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": {
                "slug": "data-form-dimension-boundaries:new",
                "extra": "/edit/true"
            },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Missing {
            data_form: Some(data_form),
            ..
        } => data_form
            .definition
            .fields
            .into_iter()
            .map(|field| (field.name, field.width, field.height))
            .collect::<Vec<_>>(),
        other => panic!("expected dimension-boundary definition, got {other:?}"),
    };
    assert_eq!(
        dimensions,
        [
            ("omitted".to_owned(), 40, 1),
            ("empty".to_owned(), 40, 1),
            ("nonnumeric".to_owned(), 40, 1),
            ("zero".to_owned(), 1, 1),
            ("negative".to_owned(), 1, 1),
            ("one".to_owned(), 40, 1),
            ("two".to_owned(), 40, 2),
        ],
    );
}

#[tokio::test]
async fn empty_text_values_survive_public_create_edit_view_and_listpages_lifecycle() {
    const CATEGORY: &str = "data-form-empty-text-lifecycle";
    const TEMPLATE_SLUG: &str = "data-form-empty-text-lifecycle:_template";
    const TARGET_SLUG: &str = "data-form-empty-text-lifecycle:saved";
    const EMPTY_SOURCE: &str = "explicit: ''\nimplicit: ''\nchoice: null";
    const TEMPLATE_SOURCE: &str = concat!(
        "[[form]]\n",
        "fields:\n",
        "  explicit:\n",
        "    label: Explicit text\n",
        "    type: text\n",
        "  implicit:\n",
        "    label: Implicit text\n",
        "  choice:\n",
        "    label: Choice\n",
        "    type: select\n",
        "    values:\n",
        "      a: Alpha\n",
        "[[/form]]",
    );

    let mut runner = TestRunner::setup().await;
    let site_id = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site
        .site_id;
    let session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "deepwell empty text data-form lifecycle test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");
    let template =
        create_page(&mut runner, site_id, TEMPLATE_SLUG, TEMPLATE_SOURCE).await;
    let category = CategoryService::get_or_create(runner.context(), site_id, CATEGORY)
        .await
        .expect("data-form target category should be created");
    grant_category_permission(
        &runner,
        site_id,
        category.category_id,
        "data-form-empty-text-lifecycle-creators",
        Action::Create,
        &[ADMIN_USER_ID],
    )
    .await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let page = create_page(&mut runner, site_id, TARGET_SLUG, EMPTY_SOURCE).await;
    match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": TARGET_SLUG, "extra": "/edit" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: Some(data_form),
            wikitext,
            ..
        } => {
            assert_eq!(wikitext, EMPTY_SOURCE);
            assert_eq!(
                data_form.values,
                BTreeMap::from([
                    ("choice".to_owned(), String::new()),
                    ("explicit".to_owned(), String::new()),
                    ("implicit".to_owned(), String::new()),
                ]),
            );
        }
        other => {
            panic!("expected canonical empty values immediately after create: {other:?}")
        }
    }

    set_page_actor(&mut runner, site_id, TARGET_SLUG);
    let populated = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": TARGET_SLUG,
            "last_revision_id": page.revision_id,
            "revision_comments": "populate data-form fields",
            "user_id": ADMIN_USER_ID,
            "wikitext": "explicit: alpha\nimplicit: beta\nchoice: a",
            "ip_address": common::IP_ADDRESS,
        }),
    )
    .expect("populated data-form edit should create a revision");
    let emptied = run_endpoint!(
        runner,
        page_edit,
        json!({
            "site_id": site_id,
            "page": TARGET_SLUG,
            "last_revision_id": populated.revision_id,
            "revision_comments": "empty data-form text fields",
            "user_id": ADMIN_USER_ID,
            "wikitext": EMPTY_SOURCE,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(
        emptied.is_some(),
        "empty data-form edit should create a revision"
    );

    let editor = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": TARGET_SLUG, "extra": "/edit" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: Some(data_form),
            ..
        } => data_form,
        other => {
            panic!("expected empty text values in the data-form editor, got {other:?}")
        }
    };
    assert_eq!(
        editor.values,
        BTreeMap::from([
            ("choice".to_owned(), String::new()),
            ("explicit".to_owned(), String::new()),
            ("implicit".to_owned(), String::new()),
        ]),
    );

    let rendered_html = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": { "slug": TARGET_SLUG, "extra": "" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: None,
            compiled_body_html,
            ..
        } => compiled_body_html,
        other => panic!("expected rendered empty data-form page, got {other:?}"),
    };
    assert!(rendered_html.contains("Explicit text"), "{rendered_html}");
    assert!(rendered_html.contains("Implicit text"), "{rendered_html}");
    assert!(!rendered_html.contains("alpha"), "{rendered_html}");
    assert!(!rendered_html.contains("beta"), "{rendered_html}");

    let index_source = concat!(
        "[[module ListPages category=\"data-form-empty-text-lifecycle\" name=\"saved\" separate=\"no\" wrapper=\"no\"]]\n",
        "EMPTY-TEXT-BEGIN|%%form_data{explicit}%%|%%form_raw{implicit}%%|EMPTY-TEXT-END\n",
        "[[/module]]",
    );
    create_page(
        &mut runner,
        site_id,
        "data-form-empty-text-lifecycle-index",
        index_source,
    )
    .await;
    let listpages_html = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {
                "slug": "data-form-empty-text-lifecycle-index",
                "extra": ""
            },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected rendered ListPages page, got {other:?}"),
    };
    assert!(
        listpages_html.contains("EMPTY-TEXT-BEGIN|||EMPTY-TEXT-END"),
        "ListPages must resolve canonical empty text values:\n{listpages_html}",
    );

    for (slug, source) in [
        (
            "data-form-empty-text-lifecycle:text-null",
            "explicit: null\nimplicit: ''\nchoice: null",
        ),
        (
            "data-form-empty-text-lifecycle:select-empty-string",
            "explicit: ''\nimplicit: ''\nchoice: ''",
        ),
    ] {
        create_page(&mut runner, site_id, slug, source).await;
        match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": session_token,
                "route": { "slug": slug, "extra": "/edit" },
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                data_form: None,
                wikitext,
                ..
            } => assert_eq!(wikitext, source),
            other => panic!("noncanonical empty scalar must fail closed: {other:?}"),
        }
    }
}

#[tokio::test]
async fn page_view_exposes_live_checkbox_and_wiki_contract() {
    const CATEGORY: &str = "data-form-checkbox-wiki-contract";
    const TEMPLATE_SOURCE: &str = concat!(
        "[[form]]\n",
        "fields:\n",
        "  enabled:\n",
        "    label: Enabled\n",
        "    type: checkbox\n",
        "    default: 1\n",
        "    before: PRE\n",
        "    after: POST\n",
        "  details:\n",
        "    label: Details\n",
        "    type: wiki\n",
        "    default: \"**Default**\"\n",
        "    hint: enter wiki \\#source\n",
        "    before: \"**Before**\"\n",
        "    after: \"//After//\"\n",
        "    match: /^never$/\n",
        "    match-error: ignored\n",
        "[[/form]]",
    );
    const SAVED_SOURCE: &str = "enabled: '1'\ndetails: \"**Bold**\\n[[[start|Home]]]\"";

    let mut runner = TestRunner::setup().await;
    let site_id = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site
        .site_id;
    let session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "deepwell checkbox/wiki data-form contract test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");
    let template = create_page(
        &mut runner,
        site_id,
        "data-form-checkbox-wiki-contract:_template",
        TEMPLATE_SOURCE,
    )
    .await;
    let category = CategoryService::get_or_create(runner.context(), site_id, CATEGORY)
        .await
        .expect("data-form target category should be created");
    grant_category_permission(
        &runner,
        site_id,
        category.category_id,
        "data-form-checkbox-wiki-contract-creators",
        Action::Create,
        &[ADMIN_USER_ID],
    )
    .await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let definition = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": {
                "slug": "data-form-checkbox-wiki-contract:new",
                "extra": "/edit/true"
            },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Missing {
            data_form: Some(data_form),
            ..
        } => data_form.definition,
        other => panic!("expected checkbox/wiki create definition, got {other:?}"),
    };
    let enabled = definition.field("enabled").expect("checkbox field");
    assert_eq!(enabled.default_value.as_deref(), Some("1"));
    let details = definition.field("details").expect("wiki field");
    assert_eq!((details.width, details.height), (40, 2));
    assert_eq!(details.hint, "enter wiki \\#source");
    assert_eq!(details.match_pattern, None);
    assert_eq!(details.match_error, None);

    create_page(
        &mut runner,
        site_id,
        "data-form-checkbox-wiki-contract:saved",
        SAVED_SOURCE,
    )
    .await;
    let editor = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": {
                "slug": "data-form-checkbox-wiki-contract:saved",
                "extra": "/edit"
            },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: Some(data_form),
            ..
        } => data_form,
        other => panic!("expected checkbox/wiki edit definition, got {other:?}"),
    };
    assert_eq!(
        editor.values,
        BTreeMap::from([
            (
                "details".to_owned(),
                "**Bold**\n[[[start|Home]]]".to_owned()
            ),
            ("enabled".to_owned(), "1".to_owned()),
        ]),
    );

    let rendered_html = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {
                "slug": "data-form-checkbox-wiki-contract:saved",
                "extra": ""
            },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected checkbox/wiki rendered page, got {other:?}"),
    };
    assert!(
        rendered_html.contains("<span>PRE 1 POST</span>"),
        "{rendered_html}",
    );
    assert!(
        rendered_html.contains(r#"<div class="form-value field-details">"#,)
            && rendered_html.contains("<p><strong>Bold</strong><br>")
            && rendered_html.contains(r#"<a class="newpage" href="/start">Home</a>"#),
        "wiki field values must compile through the Wikidot renderer:\n{rendered_html}",
    );
    assert!(
        rendered_html
            .contains(r#"<p><span style="white-space: pre-wrap;">**Before**</span></p>"#,)
            && rendered_html.contains(
                r#"<p><span style="white-space: pre-wrap;">//After//</span></p>"#,
            ),
        "wiki affixes must stay literal while the stored value is parsed:\n{rendered_html}",
    );
}

#[tokio::test]
async fn pagepath_round_trips_fullnames_and_projects_only_resolved_visible_nodes() {
    const FORM_CATEGORY: &str = "data-form-pagepath-contract";
    const TREE_CATEGORY: &str = "data-form-pagepath-tree";
    const TEMPLATE_SLUG: &str = "data-form-pagepath-contract:_template";
    const ROOT_SLUG: &str = "data-form-pagepath-tree:_root";
    const ALPHA_SLUG: &str = "data-form-pagepath-tree:alpha";
    const BETA_SLUG: &str = "data-form-pagepath-tree:beta";
    const TEMPLATE_SOURCE: &str = concat!(
        "[[form]]\n",
        "fields:\n",
        "  origin:\n",
        "    label: Origin\n",
        "    type: pagepath\n",
        "    category: data-form-pagepath-tree\n",
        "    max-level: 3\n",
        "[[/form]]",
    );

    let mut runner = TestRunner::setup().await;
    let site_id = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site
        .site_id;
    let session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "deepwell pagepath contract test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");

    let template =
        create_page(&mut runner, site_id, TEMPLATE_SLUG, TEMPLATE_SOURCE).await;
    let form_category =
        CategoryService::get_or_create(runner.context(), site_id, FORM_CATEGORY)
            .await
            .expect("data-form pagepath category should be created");
    grant_category_permission(
        &runner,
        site_id,
        form_category.category_id,
        "data-form-pagepath-contract-creators",
        Action::Create,
        &[ADMIN_USER_ID],
    )
    .await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": form_category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    for (slug, body) in [
        (ROOT_SLUG, "root"),
        (ALPHA_SLUG, "[[module Backlinks]]"),
        (BETA_SLUG, "beta"),
    ] {
        create_page(&mut runner, site_id, slug, body).await;
    }
    for (parent, child) in [(ROOT_SLUG, ALPHA_SLUG), (ALPHA_SLUG, BETA_SLUG)] {
        set_page_actor(&mut runner, site_id, child);
        run_endpoint!(
            runner,
            parent_set,
            json!({
                "site_id": site_id,
                "parent": parent,
                "child": child,
            }),
        )
        .expect("pagepath tree relationship should be created");
    }

    let create_editor = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": "data-form-pagepath-contract:new", "extra": "/edit/true" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Missing {
            data_form: Some(data_form),
            ..
        } => data_form,
        other => {
            panic!("pagepath create view must expose the generated editor: {other:?}")
        }
    };
    let origin = create_editor
        .definition
        .field("origin")
        .expect("pagepath field");
    assert_eq!(origin.field_type.as_deref(), Some("pagepath"));
    assert_eq!(origin.pagepath_category.as_deref(), Some(TREE_CATEGORY));
    assert_eq!(origin.pagepath_max_level, Some(3));
    assert_eq!(
        create_editor.pagepaths["origin"],
        vec![
            deepwell::services::DataFormPagepathNode {
                fullname: ROOT_SLUG.to_owned(),
                name: "_root".to_owned(),
                parent: None,
            },
            deepwell::services::DataFormPagepathNode {
                fullname: ALPHA_SLUG.to_owned(),
                name: "alpha".to_owned(),
                parent: Some(ROOT_SLUG.to_owned()),
            },
            deepwell::services::DataFormPagepathNode {
                fullname: BETA_SLUG.to_owned(),
                name: "beta".to_owned(),
                parent: Some(ALPHA_SLUG.to_owned()),
            },
        ],
    );

    for (case, stored, expected_visible) in [
        ("existing", ALPHA_SLUG, Some("alpha")),
        ("missing", "data-form-pagepath-tree:missing", None),
        ("cross-category", "data-form-pagepath-outside:alpha", None),
    ] {
        let slug = format!("{FORM_CATEGORY}:{case}");
        let source = format!("origin: '{stored}'");
        create_page(&mut runner, site_id, &slug, &source).await;

        let editor = match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": session_token,
                "route": { "slug": slug, "extra": "/edit" },
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                data_form: Some(data_form),
                wikitext,
                ..
            } => {
                assert_eq!(wikitext, source);
                data_form
            }
            other => panic!("pagepath edit view must preserve {case} storage: {other:?}"),
        };
        assert_eq!(editor.values["origin"], stored);
        assert_eq!(
            editor.pagepaths["origin"],
            create_editor.pagepaths["origin"]
        );

        let rendered = match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": null,
                "route": { "slug": slug, "extra": "" },
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                compiled_body_html, ..
            } => compiled_body_html,
            other => panic!("pagepath saved view must render {case}: {other:?}"),
        };
        assert!(rendered.contains("Origin"), "{rendered}");
        assert!(
            !rendered.contains(stored),
            "raw pagepath fullname leaked: {rendered}"
        );
        match expected_visible {
            Some(value) => assert!(
                rendered.contains(&format!("<span>{value}</span>")),
                "{rendered}"
            ),
            None => assert!(rendered.contains("<span></span>"), "{rendered}"),
        }
        if case == "existing" {
            let backlinks = match run_endpoint!(
                runner,
                page_view,
                json!({
                    "site_id": site_id,
                    "session_token": null,
                    "route": { "slug": ALPHA_SLUG, "extra": "" },
                    "locales": ["en-US", "en"],
                }),
            ) {
                GetPageViewOutput::Found {
                    compiled_body_html, ..
                } => compiled_body_html,
                other => panic!("pagepath target backlinks must render: {other:?}"),
            };
            assert!(
                backlinks.contains("/data-form-pagepath-contract:existing"),
                "pagepath storage must create the Wikidot backlink relation: {backlinks}",
            );
        }
    }
}

#[tokio::test]
async fn custom_data_form_layout_resolves_pagepath_data_but_preserves_form_raw() {
    const FORM_CATEGORY: &str = "data-form-pagepath-layout";
    const TEMPLATE_SLUG: &str = "data-form-pagepath-layout:_template";
    const TARGET_SLUG: &str = "data-form-pagepath-layout-tree:alpha";
    const TEMPLATE_SOURCE: &str = concat!(
        "DISPLAY=%%form_data{origin}%% RAW=%%form_raw{origin}%%\n",
        "====\n",
        "[[form]]\n",
        "fields:\n",
        "  origin:\n",
        "    label: Origin\n",
        "    type: pagepath\n",
        "    category: data-form-pagepath-layout-tree\n",
        "[[/form]]",
    );

    let mut runner = TestRunner::setup().await;
    let site_id = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site
        .site_id;
    let template =
        create_page(&mut runner, site_id, TEMPLATE_SLUG, TEMPLATE_SOURCE).await;
    let category =
        CategoryService::get_or_create(runner.context(), site_id, FORM_CATEGORY)
            .await
            .expect("custom pagepath layout category should exist");
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    create_page(&mut runner, site_id, TARGET_SLUG, "alpha").await;
    create_page(
        &mut runner,
        site_id,
        "data-form-pagepath-layout:saved",
        &format!("origin: '{TARGET_SLUG}'"),
    )
    .await;

    let rendered = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": { "slug": "data-form-pagepath-layout:saved", "extra": "" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("custom pagepath layout must render: {other:?}"),
    };
    assert!(
        rendered.contains(&format!("DISPLAY=alpha RAW={TARGET_SLUG}")),
        "{rendered}",
    );
}

#[tokio::test]
async fn custom_data_form_layout_remains_editable_and_renders_form_variables() {
    let mut runner = TestRunner::setup().await;
    let site_id = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site
        .site_id;
    let session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "deepwell custom data-form layout test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });

    const CATEGORY: &str = "data-form-custom-layout";
    let template = create_page(
        &mut runner,
        site_id,
        "data-form-custom-layout:_template",
        concat!(
            "[[module css]]\n",
            ".urgent { color: red; font-weight: bold; }\n",
            "[[/module]]\n",
            "[[span class=\"%%form_raw{priority}%%\"]]%%form_data{priority}%%[[/span]]\n",
            "External: *%%form_data{website}%%\n",
            "Internal: [[[%%form_data{target}%%]]]\n",
            "====\n",
            "[[form]]\n",
            "fields:\n",
            "  priority:\n",
            "    label: Priority\n",
            "    type: select\n",
            "    values:\n",
            "      normal: Normal\n",
            "      urgent: Urgent\n",
            "  website:\n",
            "    label: Website\n",
            "    type: url\n",
            "  target:\n",
            "    label: Target\n",
            "    type: text\n",
            "[[/form]]",
        ),
    )
    .await;
    let category = CategoryService::get_or_create(runner.context(), site_id, CATEGORY)
        .await
        .expect("data-form category should be created");
    grant_category_permission(
        &runner,
        site_id,
        category.category_id,
        "custom-data-form-creators",
        Action::Create,
        &[ADMIN_USER_ID],
    )
    .await;
    run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let create_definition = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": "data-form-custom-layout:new", "extra": "/edit/true" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Missing {
            data_form: Some(data_form),
            ..
        } => data_form.definition,
        other => panic!("expected custom-layout create form, got {other:?}"),
    };
    assert!(!create_definition.default_layout);

    create_page(
        &mut runner,
        site_id,
        "data-form-custom-layout:saved",
        "priority: urgent\nwebsite: example.com/alpha\ntarget: missing-target",
    )
    .await;

    let editor = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": "data-form-custom-layout:saved", "extra": "/edit" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: Some(data_form),
            ..
        } => data_form,
        other => panic!("expected custom-layout edit form, got {other:?}"),
    };
    assert_eq!(
        editor.values,
        BTreeMap::from([
            ("priority".to_owned(), "urgent".to_owned()),
            ("target".to_owned(), "missing-target".to_owned()),
            ("website".to_owned(), "example.com/alpha".to_owned()),
        ]),
    );

    let (rendered_html, rendered_styles) = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": { "slug": "data-form-custom-layout:saved", "extra": "" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            compiled_body_html,
            compiled_body_styles,
            ..
        } => (compiled_body_html, compiled_body_styles),
        other => panic!("expected custom-layout saved page, got {other:?}"),
    };
    assert!(
        rendered_html.contains(r#"<span class="urgent">Urgent</span>"#),
        "form_raw must retain the stored select key while form_data emits its label: {rendered_html}",
    );
    assert!(
        rendered_html.contains(r#"href="http://example.com/alpha" target="_blank""#,),
        "url form_data must use the documented default http scheme and leading-star new-window link semantics: {rendered_html}",
    );
    assert!(
        rendered_html.contains(r#"class="newpage" href="/missing-target""#),
        "text form_data inside ordinary internal-link syntax must use normal link resolution: {rendered_html}",
    );
    assert!(
        rendered_styles
            .iter()
            .any(|css| css.contains(".urgent { color: red; font-weight: bold; }")),
        "custom-layout CSS must flow through the ordinary page/site style channel: {rendered_styles:?}",
    );
    assert!(!rendered_html.contains("[[form]]"));
    assert!(!rendered_html.contains("%%form_data"));
    assert!(!rendered_html.contains("%%form_raw"));
}

#[tokio::test]
async fn data_form_youtube_raw_wiki_executes_only_inside_html_block() {
    const CATEGORY: &str = "data-form-youtube-contract";
    const TEMPLATE_SLUG: &str = "data-form-youtube-contract:_template";
    const TARGET_SLUG: &str = "data-form-youtube-contract:saved";
    const RAW: &str = r#"<iframe width="560" height="315" src="https://www.youtube.com/embed/dQw4w9WgXcQ" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>"#;
    const HTML_BLOCK_BOUNDARY_CASES: &[(&str, bool)] = &[
        (
            r#"<iframe width="320" height="180" src="https://example.invalid/reordered"></iframe>"#,
            true,
        ),
        (
            r#"<iframe src="https://example.invalid/extra" width="320" height="180" allowfullscreen="allowfullscreen"></iframe>"#,
            true,
        ),
        (
            r#"<iframe src="http://example.invalid/http" width="320" height="180"></iframe>"#,
            true,
        ),
        (RAW, true),
        (
            r#"<iframe src='https://example.invalid/single-quotes' width='321' height='181'></iframe>"#,
            true,
        ),
        (
            r#"<iframe src="https://example.invalid/class" width="320" height="180" class="probe-frame"></iframe>"#,
            true,
        ),
        (
            r#"<iframe src="https://example.invalid/style" width="320" height="180" style="border:0"></iframe>"#,
            true,
        ),
        (
            r#"<iframe src="https://example.invalid/scrolling" width="320" height="180" scrolling="no"></iframe>"#,
            true,
        ),
        (
            r#"<iframe src="https://example.invalid/align" width="320" height="180" align="left"></iframe>"#,
            true,
        ),
        (
            r#"<iframe src="https://example.invalid/sandbox" width="320" height="180" sandbox="allow-scripts"></iframe>"#,
            true,
        ),
        (
            r#"<iframe src="https://example.invalid/loading" width="320" height="180" loading="lazy"></iframe>"#,
            true,
        ),
        (
            r#"<iframe src="https://example.invalid/src-only"></iframe>"#,
            true,
        ),
        (r#"<iframe title="probe"></iframe>"#, true),
        ("<iframe></iframe>", false),
        (r#"<iframe data-wj-probe="x"></iframe>"#, false),
        (
            r#"<iframe src="https://example.invalid/data" width="320" height="180" data-wj-probe="kept-or-dropped"></iframe>"#,
            false,
        ),
        (
            r#"<iframe src="https://example.invalid/onload" width="320" height="180" onload="return false"></iframe>"#,
            false,
        ),
        (
            r#"<iframe src="https://example.invalid/title" width="320" height="180" title="1 > 0"></iframe>"#,
            false,
        ),
        (
            r#"<iframe src="https://example.invalid/title-lt" width="320" height="180" title="1 < 2"></iframe>"#,
            false,
        ),
        (
            r#"before <iframe src="https://example.invalid/adjacent" width="320" height="180"></iframe> after"#,
            false,
        ),
        (
            r#"<iframe src="https://example.invalid/one" width="320" height="180"></iframe><iframe src="https://example.invalid/two" width="320" height="180"></iframe>"#,
            false,
        ),
        (
            r#"<iframe src="https://example.invalid/fallback" width="320" height="180">fallback</iframe>"#,
            false,
        ),
        (
            r#"<!--before--><iframe src="https://example.invalid/comment" width="320" height="180"></iframe><!--after-->"#,
            false,
        ),
    ];
    const TEMPLATE_SOURCE: &str = concat!(
        "[[html]]\n",
        "%%form_raw{video}%%\n",
        "[[/html]]\n",
        "[[div class=\"form-data-control\"]]\n",
        "%%form_data{video}%%\n",
        "[[/div]]\n",
        "[[div class=\"form-raw-outside\"]]\n",
        "%%form_raw{video}%%\n",
        "[[/div]]\n",
        "====\n",
        "[[form]]\n",
        "fields:\n",
        "  video:\n",
        "    label: Video\n",
        "    type: wiki\n",
        "[[/form]]",
    );

    let mut runner = TestRunner::setup().await;
    let site_id = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site
        .site_id;
    let session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "deepwell data-form YouTube contract test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");
    let template =
        create_page(&mut runner, site_id, TEMPLATE_SLUG, TEMPLATE_SOURCE).await;
    let category = CategoryService::get_or_create(runner.context(), site_id, CATEGORY)
        .await
        .expect("data-form YouTube category should be created");
    grant_category_permission(
        &runner,
        site_id,
        category.category_id,
        "data-form-youtube-contract-creators",
        Action::Create,
        &[ADMIN_USER_ID],
    )
    .await;
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    create_page(
        &mut runner,
        site_id,
        TARGET_SLUG,
        &format!("video: '{RAW}'"),
    )
    .await;
    let editor = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": { "slug": TARGET_SLUG, "extra": "/edit" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            data_form: Some(data_form),
            ..
        } => data_form,
        other => panic!("expected data-form YouTube edit view, got {other:?}"),
    };
    assert_eq!(editor.values.get("video").map(String::as_str), Some(RAW));

    let rendered = match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": { "slug": TARGET_SLUG, "extra": "" },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected data-form YouTube saved view, got {other:?}"),
    };
    assert_eq!(
        rendered.matches("<iframe").count(),
        1,
        "only form_raw inside the authored HTML block may execute an iframe: {rendered}",
    );
    assert!(
        rendered.contains(r#"src="https://www.youtube.com/embed/dQw4w9WgXcQ""#)
            && rendered.contains(r#"width="560""#)
            && rendered.contains(r#"height="315""#)
            && rendered.contains(r#"title="YouTube video player""#)
            && rendered.contains(r#"referrerpolicy="strict-origin-when-cross-origin""#)
            && rendered.contains("allowfullscreen"),
        "the live-observed iframe attributes must survive the saved data-form render: {rendered}",
    );
    for class_name in ["form-data-control", "form-raw-outside"] {
        let marker = format!(r#"<div class="{class_name}">"#);
        let start = rendered
            .find(&marker)
            .unwrap_or_else(|| panic!("missing {class_name} wrapper: {rendered}"));
        let tail = &rendered[start + marker.len()..];
        let end = tail
            .find("</div>")
            .unwrap_or_else(|| panic!("unterminated {class_name} wrapper: {rendered}"));
        let body = &tail[..end];
        assert!(
            body.contains("&lt;iframe") && !body.contains("<iframe"),
            "{class_name} must render the raw source as inert wiki text: {body}",
        );
    }

    for (index, &(value, direct)) in HTML_BLOCK_BOUNDARY_CASES.iter().enumerate() {
        let slug = format!("data-form-youtube-html-boundary-{}", index + 1);
        create_page(
            &mut runner,
            site_id,
            &slug,
            &format!("[[html]]\n{value}\n[[/html]]"),
        )
        .await;
        let rendered = match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": null,
                "route": { "slug": &slug, "extra": "" },
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                compiled_body_html, ..
            } => compiled_body_html,
            other => panic!("expected data-form YouTube boundary view, got {other:?}"),
        };
        assert_eq!(
            rendered.matches("<iframe").count(),
            1,
            "each observed HTML-block boundary case must emit one direct iframe or one hosted wrapper: {rendered}",
        );
        assert_eq!(
            !rendered.contains(r#"class="html-block-iframe""#),
            direct,
            "saved [[html]] direct-iframe boundary mismatch for {value}: {rendered}",
        );
    }
}

#[tokio::test]
async fn page_view_exposes_live_hidden_password_static_url_scalar_contract() {
    let mut runner = TestRunner::setup().await;
    let site_id = run_endpoint!(runner, site_get, json!({ "site": "test" }))
        .expect("seeded test site should exist")
        .site
        .site_id;
    let session_token = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "deepwell scalar data-form contract test".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("admin session should be created");
    runner.set_request_context(RequestContext {
        user_id: Some(ADMIN_USER_ID),
        ..Default::default()
    });
    let cases = [
        (
            "hidden",
            "[[form]]\nfields:\n  scalar:\n    label: Hidden <scalar>\n    type: hidden\n    value: HIDDEN_CONFIGURED_ALPHA\n[[/form]]",
            "scalar: HIDDEN_CONFIGURED_ALPHA",
            Some("HIDDEN_CONFIGURED_ALPHA"),
        ),
        (
            "password",
            "[[form]]\nfields:\n  scalar:\n    label: Password scalar\n    type: password\n[[/form]]",
            "scalar: NONSECRET_PASSWORD_ALPHA",
            None,
        ),
        (
            "static",
            "[[form]]\nfields:\n  scalar:\n    label: Static scalar\n    type: static\n    value: 'STATIC **BOLD** ALPHA'\n[[/form]]",
            "null",
            Some("STATIC **BOLD** ALPHA"),
        ),
        (
            "url",
            "[[form]]\nfields:\n  scalar:\n    label: URL scalar\n    type: url\n[[/form]]",
            "scalar: example.com/alpha",
            None,
        ),
    ];

    for (field_type, template_source, saved_source, configured_value) in cases {
        let category_name = format!("data-form-scalar-{field_type}");
        let template_slug = format!("{category_name}:_template");
        let template =
            create_page(&mut runner, site_id, &template_slug, template_source).await;
        let category =
            CategoryService::get_or_create(runner.context(), site_id, &category_name)
                .await
                .expect("data-form target category should be created");
        grant_category_permission(
            &runner,
            site_id,
            category.category_id,
            &format!("{category_name}-creators"),
            Action::Create,
            &[ADMIN_USER_ID],
        )
        .await;
        run_endpoint!(
            runner,
            category_update,
            json!({
                "site": site_id,
                "category": category.category_id,
                "user_id": ADMIN_USER_ID,
                "template_page_id": template.page_id,
                "ip_address": common::IP_ADDRESS,
            }),
        );

        let create_slug = format!("{category_name}:new");
        let definition = match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": session_token,
                "route": { "slug": create_slug, "extra": "/edit/true" },
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Missing {
                data_form: Some(data_form),
                ..
            } => data_form.definition,
            other => panic!("expected {field_type} create definition, got {other:?}"),
        };
        assert_eq!(definition.fields.len(), 1);
        let scalar = definition.field("scalar").expect("scalar field");
        assert_eq!(scalar.field_type.as_deref(), Some(field_type));
        assert_eq!(scalar.configured_value.as_deref(), configured_value);

        let saved_slug = format!("{category_name}:saved");
        create_page(&mut runner, site_id, &saved_slug, saved_source).await;
        let editor = match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": session_token,
                "route": { "slug": saved_slug, "extra": "/edit" },
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                data_form: Some(data_form),
                ..
            } => data_form,
            other => panic!("expected {field_type} edit definition, got {other:?}"),
        };
        let expected_value = match field_type {
            "hidden" => "HIDDEN_CONFIGURED_ALPHA",
            "password" => "NONSECRET_PASSWORD_ALPHA",
            "static" => "STATIC **BOLD** ALPHA",
            "url" => "example.com/alpha",
            _ => unreachable!(),
        };
        assert_eq!(
            editor.values.get("scalar").map(String::as_str),
            Some(expected_value)
        );

        let rendered_html = match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": null,
                "route": { "slug": saved_slug, "extra": "" },
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                data_form: None,
                compiled_body_html,
                ..
            } => compiled_body_html,
            other => panic!("expected {field_type} rendered page, got {other:?}"),
        };
        match field_type {
            "hidden" => assert!(
                rendered_html.contains("Hidden &lt;scalar&gt;")
                    && rendered_html.contains("HIDDEN_CONFIGURED_ALPHA"),
                "{rendered_html}",
            ),
            "password" => assert!(
                rendered_html.contains("************************")
                    && !rendered_html.contains("NONSECRET_PASSWORD_ALPHA"),
                "password display must be masked: {rendered_html}",
            ),
            "static" => assert!(
                rendered_html.contains(r#"<div class="form-value field-scalar">"#)
                    && rendered_html.contains("<strong>BOLD</strong>"),
                "static configured wiki must use the trusted renderer: {rendered_html}",
            ),
            "url" => assert!(
                rendered_html.contains(
                    r#"<a href="http://example.com/alpha">http://example.com/alpha</a>"#,
                ),
                "bare URL display must normalize without changing source: {rendered_html}",
            ),
            _ => unreachable!(),
        }
    }

    let unsupported_template = create_page(
        &mut runner,
        site_id,
        "data-form-scalar-unsupported:_template",
        "[[form]]\nfields:\n  scalar:\n    type: hidden\n[[/form]]",
    )
    .await;
    let unsupported_category = CategoryService::get_or_create(
        runner.context(),
        site_id,
        "data-form-scalar-unsupported",
    )
    .await
    .expect("unsupported data-form category should be created");
    grant_category_permission(
        &runner,
        site_id,
        unsupported_category.category_id,
        "data-form-scalar-unsupported-creators",
        Action::Create,
        &[ADMIN_USER_ID],
    )
    .await;
    run_endpoint!(
        runner,
        category_update,
        json!({
            "site": site_id,
            "category": unsupported_category.category_id,
            "user_id": ADMIN_USER_ID,
            "template_page_id": unsupported_template.page_id,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    match run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": session_token,
            "route": {
                "slug": "data-form-scalar-unsupported:new",
                "extra": "/edit/true"
            },
            "locales": ["en-US", "en"],
        }),
    ) {
        GetPageViewOutput::Missing {
            data_form: None, ..
        } => {}
        other => {
            panic!("hidden fields without configured values must fail closed: {other:?}")
        }
    }
}
