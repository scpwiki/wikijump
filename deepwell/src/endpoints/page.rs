/*
 * endpoints/page.rs
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

use super::prelude::*;
mod output;
mod permissions;

use output::{build_page_deleted_output, build_page_file_output, build_page_output};
use permissions::{
    ensure_deleted_page_delete_permission_and_get_category_id,
    ensure_page_action_permission, ensure_page_create_permission,
    ensure_page_edit_permission, ensure_page_permission, ensure_page_view_permission,
};

use crate::models::file::Model as FileModel;
use crate::models::page::Model as PageModel;
use crate::services::file::{GetFileOutput, GetPageFiles};
use crate::services::forum_thread::ForumThreadService;
use crate::services::legacy_action::{LegacyActionService, SetLegacyActionTags};
use crate::services::membership::MembershipBrowserAction;
use crate::services::page::{
    CreatePage, CreatePageOutput, DeletePage, DeletePageOutput, EditPage, EditPageOutput,
    GetDeletedPageOutput, GetPageAnyDetails, GetPageOutput, GetPageReference,
    GetPageReferenceDetails, GetPageScoreOutput, GetPageSlug, MovePage, MovePageOutput,
    PageEditPermissionOutput, PageLifecycleIdentity, RestorePage, RestorePageOutput,
    RollbackPage, SetPageLayout,
};
use crate::services::page_draft::{PageDraftIdentity, PageDraftService, SavePageDraft};
use crate::services::page_query::PageQueryService;
use crate::services::page_revision::RerenderType;
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::render::{
    LegacyActionRegistry, LegacyBrowserAction, SiteChangesLoad, WikidotForumFeedKind,
    WikidotForumFeedOutput, WikidotForumModuleRequest, WikidotForumModuleResponse,
    WikidotListPagesFeedInput, WikidotListPagesFeedOutput,
    WikidotMembersListModuleResponse, WikidotSiteChangesFilter,
    WikidotSiteChangesModuleRequest, WikidotSiteChangesModuleResponse,
    wikidot_site_changes_empty_response,
};
use crate::services::settings::PageRatingVisibility;
use crate::services::{MutationAuthorization, SettingsService, TextService};
use crate::types::{
    Action, Bytes, FileOrder, PageDetails, PageId, Permission, Reference, RerenderDepth,
    Resource,
};
use crate::utils::get_category_name;
use ftml::data::UserInfo;
use futures::future::try_join_all;
use regex::Regex;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;
use wikidot_normalize::normalize;

static WIKIDOT_LIST_PAGES_SET_PAIR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)<span class="set (?P<name_class>[^"]+)"><span class="name">(?P<name>[^<]*)</span>\s*</span>\s*<span class="set (?P<value_class>[^"]+)"><span class="value">(?P<value>.*?)</span>\s*</span>"#,
    )
    .expect("Wikidot ListPages set-pair expression is valid")
});

#[derive(Deserialize)]
struct WikidotListPagesModuleInput {
    site_id: i64,
    module_body: String,
    parameters: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WikidotCategoriesPageListModuleInput {
    site_id: i64,
    category_id: i64,
}

#[derive(Deserialize)]
struct WikidotForumModuleInput {
    site_id: i64,
    module_name: String,
    parameters: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WikidotSiteChangesModuleInput {
    site_id: i64,
    page_id: Option<String>,
    page: String,
    perpage: String,
    category_id: Option<String>,
    options: String,
}

#[derive(Deserialize)]
struct WikidotMembersListModuleInput {
    site_id: i64,
    parameters: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct WikidotListPagesModuleOutput {
    pub body: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct WikidotCategoriesPageListModuleOutput {
    pub body: String,
}

#[derive(Deserialize)]
struct WikidotPagePreviewInput {
    site_id: i64,
    title: String,
    wikitext: String,
    #[serde(default)]
    syntax_only: bool,
}

#[derive(Deserialize)]
struct WikidotPageDiscussionInput {
    site_id: i64,
    page_id: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PageWatchersInput {
    site_id: i64,
    page_id: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PageWhoRatedInput {
    site_id: i64,
    page_id: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PageWhoRatedVote {
    pub user: UserInfo<'static>,
    pub value: i16,
}

#[derive(Clone, Debug, Serialize)]
pub struct WikidotPageDiscussionOutput {
    pub thread_id: i64,
    pub thread_unix_title: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct WikidotPagePreviewOutput {
    pub body: String,
    pub styles: Vec<String>,
    pub legacy_actions: Vec<LegacyBrowserAction>,
    pub membership_actions: Vec<MembershipBrowserAction>,
}

#[derive(Deserialize)]
struct WikidotLegacySetTagsInput {
    page_id: i64,
    last_revision_id: i64,
    action_index: usize,
    action_fingerprint: String,
    user_id: i64,
    ip_address: std::net::IpAddr,
}

pub async fn wikidot_page_preview(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<WikidotPagePreviewOutput> {
    let input: WikidotPagePreviewInput = parse!(params, Page);
    let output = if input.syntax_only {
        RenderService::render_wikidot_syntax_preview(
            ctx,
            input.site_id,
            &input.title,
            input.wikitext,
        )
        .await
    } else {
        RenderService::render_wikidot_page_preview(
            ctx,
            input.site_id,
            &input.title,
            input.wikitext,
        )
        .await
    }
    .or_raise(|| {
        Error::new(
            format!(
                "failed to render Wikidot page preview in site ID {}",
                input.site_id,
            ),
            ErrorType::Page,
        )
    })?;

    Ok(WikidotPagePreviewOutput {
        legacy_actions: LegacyActionRegistry::from_resource_requirements(
            &output.html_output.resource_requirements,
        )
        .browser_actions_for_wikidot_html(&output.html_output.body),
        // Preview reproduces the exact control DOM but has no immutable saved
        // page revision to authorize a membership transition against.
        membership_actions: Vec::new(),
        body: output.html_output.body,
        styles: output.html_output.styles,
    })
}

pub async fn wikidot_page_discussion_create(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<WikidotPageDiscussionOutput>> {
    let input: WikidotPageDiscussionInput = parse!(params, Page);
    let Some(observed_page) = PageService::get_direct_optional(ctx, input.page_id, false)
        .await
        .or_raise(|| Error::new("failed to load discussion page", ErrorType::Page))?
    else {
        return Ok(None);
    };
    if observed_page.site_id != input.site_id {
        return Ok(None);
    }

    let can_view = PermissionService::check_user_can_view_page(
        ctx,
        ctx.request().user_id,
        &observed_page,
    )
    .await
    .or_raise(|| {
        Error::new(
            "failed to check page discussion view permission",
            ErrorType::Permission,
        )
    })?;
    if !can_view {
        return Ok(None);
    }

    let discussion_settings = SettingsService::get_page_discussion_settings_for_update(
        ctx,
        observed_page.site_id,
        observed_page.page_category_id,
    )
    .await
    .or_raise(|| {
        Error::new(
            "failed to lock page discussion settings",
            ErrorType::SiteSettings,
        )
    })?;
    if !discussion_settings.enabled {
        return Ok(None);
    }

    let Some(page) =
        PageService::get_direct_optional_for_update(ctx, input.page_id, false)
            .await
            .or_raise(|| Error::new("failed to lock discussion page", ErrorType::Page))?
    else {
        return Ok(None);
    };
    if page.site_id != observed_page.site_id
        || page.page_category_id != observed_page.page_category_id
    {
        return Ok(None);
    }

    let revision = PageRevisionService::get_latest(ctx, page.site_id, page.page_id)
        .await
        .or_raise(|| {
            Error::new(
                "failed to load page discussion title",
                ErrorType::PageRevision,
            )
        })?;
    let thread = ForumThreadService::get_or_create_page_discussion(
        ctx,
        &page,
        crate::constants::ANONYMOUS_USER_ID,
        &revision.title,
    )
    .await
    .or_raise(|| {
        Error::new(
            "failed to create Wikidot page discussion",
            ErrorType::ForumThread,
        )
    })?;

    let mut thread_unix_title = thread.title;
    normalize(&mut thread_unix_title);
    Ok(Some(WikidotPageDiscussionOutput {
        thread_id: thread.forum_thread_id,
        thread_unix_title,
    }))
}

pub async fn wikidot_list_pages_module(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<WikidotListPagesModuleOutput> {
    let input: WikidotListPagesModuleInput = parse!(params, Page);
    let output = RenderService::render_wikidot_list_pages_module(
        ctx,
        input.site_id,
        input.module_body,
        &input.parameters,
    )
    .await
    .or_raise(|| {
        Error::new(
            format!(
                "failed to render Wikidot ListPages module in site ID {}",
                input.site_id,
            ),
            ErrorType::Page,
        )
    })?;

    Ok(WikidotListPagesModuleOutput {
        body: normalize_wikidot_list_pages_set_spacing(
            &normalize_wikidot_list_pages_set_pairs(&output.html_output.body),
        ),
    })
}

pub async fn wikidot_categories_page_list_module(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<WikidotCategoriesPageListModuleOutput> {
    let input: WikidotCategoriesPageListModuleInput = parse!(params, Page);
    if ctx
        .request()
        .site_id
        .is_some_and(|request_site_id| request_site_id != input.site_id)
    {
        return Err(Error::new(
            "Categories page-list site does not match the request context",
            ErrorType::PermissionDenied,
        )
        .into());
    }
    let body = RenderService::render_wikidot_categories_page_list_module(
        ctx,
        input.site_id,
        input.category_id,
        ctx.request().user_id,
    )
    .await
    .or_raise(|| Error::new("failed to render Categories page list", ErrorType::Page))?;
    Ok(WikidotCategoriesPageListModuleOutput { body })
}

pub async fn wikidot_site_changes_module(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<WikidotSiteChangesModuleResponse> {
    let input: WikidotSiteChangesModuleInput = parse!(params, Page);
    let not_ok = || WikidotSiteChangesModuleResponse {
        status: "not_ok".to_owned(),
        body: String::new(),
    };
    if ctx
        .request()
        .site_id
        .is_some_and(|request_site_id| request_site_id != input.site_id)
    {
        return Err(Error::new(
            "SiteChanges module site does not match the request context",
            ErrorType::PermissionDenied,
        )
        .into());
    }

    let Some(page) = wikidot_positive_decimal::<u32>(&input.page) else {
        return Ok(not_ok());
    };
    let (rows_per_page, category_id, filter) = match (input.page_id, input.category_id) {
        (Some(page_id), Some(category_id)) => {
            let Some(page_id) = wikidot_positive_decimal::<i64>(&page_id) else {
                return Ok(not_ok());
            };
            let Some(perpage) = site_changes_browser_perpage(&input.perpage) else {
                return Ok(not_ok());
            };
            let category_id = if category_id.is_empty() {
                None
            } else {
                let Some(category_id) = wikidot_positive_decimal::<i64>(&category_id)
                else {
                    return Ok(not_ok());
                };
                Some(category_id)
            };
            let Some(filter) =
                WikidotSiteChangesFilter::from_browser_options(&input.options)
            else {
                return Ok(not_ok());
            };

            let Some(host_page) =
                PageService::get_optional(ctx, input.site_id, Reference::Id(page_id))
                    .await
                    .or_raise(|| {
                        Error::new(
                            "failed to resolve SiteChanges host page",
                            ErrorType::Page,
                        )
                    })?
            else {
                return Ok(not_ok());
            };
            let can_view_host = PermissionService::check_user_can_view_page(
                ctx,
                ctx.request().user_id,
                &host_page,
            )
            .await
            .or_raise(|| {
                Error::new(
                    "failed to check SiteChanges host page visibility",
                    ErrorType::Permission,
                )
            })?;
            if !can_view_host {
                return Ok(not_ok());
            }
            let SiteChangesBrowserPerpage::Rows(rows_per_page) = perpage else {
                return Ok(wikidot_site_changes_empty_response());
            };
            (rows_per_page, category_id, filter)
        }
        (None, None) => {
            let rows_per_page = match input.perpage.as_str() {
                "20" => 20,
                "1000" => 1_000,
                malformed if wikidot_bounded_word_scalar(malformed) => {
                    return Ok(WikidotSiteChangesModuleResponse {
                        status: "ok".to_owned(),
                        body: "\tSorry, no revisions matching your criteria.".to_owned(),
                    });
                }
                _ => return Ok(not_ok()),
            };
            let Some(filter) =
                WikidotSiteChangesFilter::from_wikidot_py_options(&input.options)
            else {
                return Ok(not_ok());
            };
            (rows_per_page, None, filter)
        }
        _ => return Ok(not_ok()),
    };

    let outcome = RenderService::render_wikidot_site_changes_module(
        ctx,
        input.site_id,
        WikidotSiteChangesModuleRequest {
            page,
            rows_per_page,
            category_id,
            filter,
        },
    )
    .await
    .or_raise(|| {
        Error::new(
            format!(
                "failed to render Wikidot SiteChanges module in site ID {}",
                input.site_id,
            ),
            ErrorType::Page,
        )
    })?;
    match outcome {
        SiteChangesLoad::Complete(response) => Ok(response),
        SiteChangesLoad::Saturated => Ok(not_ok()),
    }
}

fn wikidot_bounded_word_scalar(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphabetic() || byte == b'-')
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SiteChangesBrowserPerpage {
    Empty,
    Rows(usize),
}

fn site_changes_browser_perpage(value: &str) -> Option<SiteChangesBrowserPerpage> {
    match value {
        "0" | "-1" => Some(SiteChangesBrowserPerpage::Empty),
        "1" => Some(SiteChangesBrowserPerpage::Rows(1)),
        "10" => Some(SiteChangesBrowserPerpage::Rows(10)),
        "20" => Some(SiteChangesBrowserPerpage::Rows(20)),
        "100" => Some(SiteChangesBrowserPerpage::Rows(100)),
        _ => None,
    }
}

fn wikidot_positive_decimal<T>(value: &str) -> Option<T>
where
    T: std::str::FromStr,
{
    let mut bytes = value.bytes();
    if !matches!(bytes.next(), Some(b'1'..=b'9'))
        || !bytes.all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    value.parse().ok()
}

pub async fn wikidot_forum_module(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<WikidotForumModuleResponse> {
    let input: WikidotForumModuleInput = parse!(params, Page);
    if ctx
        .request()
        .site_id
        .is_some_and(|request_site_id| request_site_id != input.site_id)
    {
        return Err(Error::new(
            "forum module site does not match the request context",
            ErrorType::PermissionDenied,
        )
        .into());
    }
    RenderService::render_wikidot_forum_module(
        ctx,
        input.site_id,
        WikidotForumModuleRequest {
            module_name: input.module_name,
            parameters: input.parameters,
        },
    )
    .await
    .or_raise(|| {
        Error::new(
            format!(
                "failed to render Wikidot forum module in site ID {}",
                input.site_id,
            ),
            ErrorType::Page,
        )
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WikidotForumFeedInput {
    site_id: i64,
    kind: String,
}

pub async fn wikidot_forum_feed(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<WikidotForumFeedOutput>> {
    let input: WikidotForumFeedInput = parse!(params, Page);
    if ctx
        .request()
        .site_id
        .is_some_and(|request_site_id| request_site_id != input.site_id)
    {
        return Err(Error::new(
            "forum feed site does not match the request context",
            ErrorType::PermissionDenied,
        )
        .into());
    }
    let kind = match input.kind.as_str() {
        "threads" => WikidotForumFeedKind::Threads,
        "posts" => WikidotForumFeedKind::Posts,
        _ => {
            return Err(
                Error::new("invalid forum feed kind", ErrorType::BadRequest).into()
            );
        }
    };
    RenderService::wikidot_forum_feed(ctx, input.site_id, kind).await
}

pub async fn wikidot_members_list_module(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<WikidotMembersListModuleResponse> {
    let input: WikidotMembersListModuleInput = parse!(params, Page);
    if ctx
        .request()
        .site_id
        .is_some_and(|request_site_id| request_site_id != input.site_id)
    {
        return Err(Error::new(
            "members list module site does not match the request context",
            ErrorType::PermissionDenied,
        )
        .into());
    }
    RenderService::render_wikidot_members_list_module(
        ctx,
        input.site_id,
        &input.parameters,
    )
    .await
    .or_raise(|| {
        Error::new(
            format!(
                "failed to render Wikidot MembersListModule in site ID {}",
                input.site_id,
            ),
            ErrorType::Page,
        )
    })
}

pub async fn wikidot_list_pages_feed(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<WikidotListPagesFeedOutput> {
    let input: WikidotListPagesFeedInput = parse!(params, Page);
    RenderService::render_wikidot_list_pages_feed(ctx, input)
        .await
        .or_raise(|| {
            Error::new("failed to render Wikidot ListPages feed", ErrorType::Page)
        })
}

fn normalize_wikidot_list_pages_set_spacing(body: &str) -> String {
    body.replace(
        r#"</span><span class="value">"#,
        r#"</span> <span class="value">"#,
    )
    .replace(
        r#"</span><span class="set "#,
        r#"</span> <span class="set "#,
    )
}

/// FTML renders adjacent inline spans as sibling nodes in this module shape.
/// wikidot.py's ListPages parser instead treats the name and value spans as one
/// `set` record, so restore that documented connector-only wire shape here.
fn normalize_wikidot_list_pages_set_pairs(body: &str) -> String {
    WIKIDOT_LIST_PAGES_SET_PAIR_REGEX
        .replace_all(body, |captures: &regex::Captures<'_>| {
            let name_class = captures
                .name("name_class")
                .expect("set-pair name class capture exists")
                .as_str();
            let value_class = captures
                .name("value_class")
                .expect("set-pair value class capture exists")
                .as_str();
            if name_class != value_class {
                return captures
                    .get(0)
                    .expect("set-pair full capture exists")
                    .as_str()
                    .to_owned();
            }
            format!(
                r#"<span class="set {name_class}"><span class="name">{}</span><span class="value">{}</span></span>"#,
                captures
                    .name("name")
                    .expect("set-pair name capture exists")
                    .as_str(),
                captures
                    .name("value")
                    .expect("set-pair value capture exists")
                    .as_str(),
            )
        })
        .into_owned()
}

#[cfg(test)]
mod wikidot_list_pages_set_pair_tests {
    use super::normalize_wikidot_list_pages_set_pairs;

    #[test]
    fn joins_split_name_and_value_records_after_an_empty_field() {
        let body = concat!(
            r#"<span class="set parent_fullname"><span class="name">parent_fullname</span></span> "#,
            r#"<span class="set comments"><span class="name">comments</span></span> "#,
            r#"<span class="set comments"><span class="value">79</span></span> "#,
            r#"<span class="set size"><span class="name">size</span><span class="value">30033</span></span>"#,
        );

        assert_eq!(
            normalize_wikidot_list_pages_set_pairs(body),
            concat!(
                r#"<span class="set parent_fullname"><span class="name">parent_fullname</span></span> "#,
                r#"<span class="set comments"><span class="name">comments</span><span class="value">79</span></span> "#,
                r#"<span class="set size"><span class="name">size</span><span class="value">30033</span></span>"#,
            ),
        );
    }
}

pub async fn page_create(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<CreatePageOutput> {
    let input: CreatePage = parse!(params, Page);
    info!("Creating new page in site ID {}", input.site_id);

    let actor_user_id = require_authenticated_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate page create actor", ErrorType::Page)
        })?;
    ensure_page_create_permission(ctx, input.site_id, &input.slug, actor_user_id)
        .await
        .or_raise(|| {
            Error::new("failed to check page create permission", ErrorType::Page)
        })?;

    PageService::create(ctx, input)
        .await
        .or_raise(|| Error::new("failed to create page", ErrorType::Page))
}

/// Persist the observed full-page editor draft state for the current actor.
pub async fn page_draft_save(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<bool> {
    let input: SavePageDraft = parse!(params, Page);
    PageDraftService::save(ctx, input)
        .await
        .map(|_| true)
        .or_raise(|| Error::new("failed to save page draft", ErrorType::Page))
}

/// Return whether the current actor owns the exact observed draft identity.
pub async fn page_draft_exists(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<bool> {
    let input: PageDraftIdentity = parse!(params, Page);
    PageDraftService::exists(ctx, input)
        .await
        .or_raise(|| Error::new("failed to check page draft", ErrorType::Page))
}

/// Remove the current actor's exact observed draft identity.
pub async fn page_draft_remove(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<bool> {
    let input: PageDraftIdentity = parse!(params, Page);
    PageDraftService::remove(ctx, input)
        .await
        .or_raise(|| Error::new("failed to remove page draft", ErrorType::Page))
}

pub async fn page_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<GetPageOutput>> {
    let GetPageReferenceDetails {
        site_id,
        page: reference,
        details,
    } = parse!(params, Page);

    let make_error = || Error::new("failed to get page", ErrorType::Page);

    if matches!(&reference, Reference::Slug(slug) if slug.trim_matches('/').eq_ignore_ascii_case("_admin"))
        && !PermissionService::check_user_can_access_admin(
            ctx,
            ctx.request().user_id,
            site_id,
        )
        .await
        .or_raise(|| {
            Error::new(
                "failed to check admin page permission",
                ErrorType::Permission,
            )
        })?
    {
        return Err(Error::new(
            "user does not have permission to view this page",
            ErrorType::PermissionDenied,
        )
        .into());
    }

    let page = PageService::get_optional(ctx, site_id, reference)
        .await
        .or_raise(make_error)?;

    match page {
        None => Ok(None),
        Some(page) => {
            if page.slug.eq_ignore_ascii_case("_admin")
                && !PermissionService::check_user_can_view_page(
                    ctx,
                    ctx.request().user_id,
                    &page,
                )
                .await
                .or_raise(|| {
                    Error::new(
                        "failed to check page view permission",
                        ErrorType::Permission,
                    )
                })?
            {
                return Err(Error::new(
                    "user does not have permission to view this page",
                    ErrorType::PermissionDenied,
                )
                .into());
            }
            build_page_output(ctx, page, details)
                .await
                .or_raise(make_error)
        }
    }
}

/// Returns whether the current request may view a page reference.
///
/// Unlike `page_get`, this endpoint intentionally exposes no page metadata and
/// treats a missing or hidden page as the same negative result. It is used by
/// public helper routes that must distinguish an existing target only after
/// applying the normal page-view permission boundary.
pub async fn page_view_permission(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<bool> {
    let GetPageReference {
        site_id,
        page: reference,
    } = parse!(params, Page);

    let Some(page) = PageService::get_optional(ctx, site_id, reference)
        .await
        .or_raise(|| {
            Error::new("failed to resolve page view permission", ErrorType::Page)
        })?
    else {
        return Ok(false);
    };

    PermissionService::check_user_can_view_page(ctx, ctx.request().user_id, &page)
        .await
        .or_raise(|| {
            Error::new(
                "failed to check page view permission",
                ErrorType::Permission,
            )
        })
}

/// Return display-name-only lifecycle identities for a viewable page.
///
/// Missing and hidden pages share the same null result. Identity resolution
/// happens only after the normal page-view permission check, and unavailable
/// identities remain null inside the projection rather than falling back to
/// internal IDs or slugs.
pub async fn page_lifecycle_identity(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<PageLifecycleIdentity>> {
    let GetPageReference {
        site_id,
        page: reference,
    } = parse!(params, Page);
    let make_error =
        || Error::new("failed to resolve page lifecycle identity", ErrorType::Page);

    let Some(page) = PageService::get_optional(ctx, site_id, reference)
        .await
        .or_raise(make_error)?
    else {
        return Ok(None);
    };
    let can_view =
        PermissionService::check_user_can_view_page(ctx, ctx.request().user_id, &page)
            .await
            .or_raise(make_error)?;
    if !can_view {
        return Ok(None);
    }

    PageService::get_lifecycle_identity(ctx, &page)
        .await
        .map(Some)
        .or_raise(make_error)
}

pub async fn page_watchers(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<UserInfo<'static>>> {
    let PageWatchersInput { site_id, page_id } = parse!(params, Page);
    let make_error = || {
        Error::new(
            format!("failed to list watchers for page ID {page_id} in site ID {site_id}"),
            ErrorType::PageWatchRelation,
        )
    };

    ensure_page_view_permission(ctx, site_id, page_id).await?;
    let user_ids = RelationService::get_active_page_watcher_ids(ctx, page_id)
        .await
        .or_raise(make_error)?;
    let mut watchers = Vec::with_capacity(user_ids.len());
    for user_id in user_ids {
        let user = UserService::get_optional(ctx, Reference::Id(user_id))
            .await
            .or_raise(make_error)?;
        let Some(identity) = user.and_then(|user| user.into_public_identity()) else {
            return Err(make_error().into());
        };
        watchers.push(identity);
    }
    watchers.sort_by_key(|identity| {
        (
            identity.user_name.to_lowercase(),
            identity.user_slug.to_lowercase(),
            identity.user_id,
        )
    });
    Ok(watchers)
}

pub async fn page_who_rated(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<PageWhoRatedVote>> {
    let PageWhoRatedInput { site_id, page_id } = parse!(params, Page);
    let make_error = || {
        Error::new(
            format!(
                "failed to list current ratings for page ID {page_id} in site ID {site_id}"
            ),
            ErrorType::PageVote,
        )
    };

    let page = get_page_who_rated_target(ctx, site_id, page_id).await?;
    let settings = SettingsService::get_page_rating_settings(
        ctx,
        page.site_id,
        page.page_category_id,
    )
    .await
    .or_raise(make_error)?;
    if settings.visibility == PageRatingVisibility::Anonymous {
        return Err(Error::new(
            "this category keeps individual page ratings anonymous",
            ErrorType::PermissionDenied,
        )
        .into());
    }

    let votes = VoteService::get_current_page_votes(
        ctx,
        page.page_id,
        settings.rating_type.vote_store_key(),
    )
    .await
    .or_raise(make_error)?;
    let user_ids = votes
        .iter()
        .map(|vote| vote.user_id)
        .collect::<BTreeSet<_>>();
    let mut identities = UserService::get_public_identities(ctx, &user_ids)
        .await
        .or_raise(make_error)?;
    let mut output = Vec::with_capacity(votes.len());
    for vote in votes {
        let Some(user) = identities.remove(&vote.user_id) else {
            return Err(make_error().into());
        };
        output.push(PageWhoRatedVote {
            user,
            value: vote.value,
        });
    }
    Ok(output)
}

async fn get_page_who_rated_target(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_id: i64,
) -> Result<PageModel> {
    let deny = || {
        Error::new(
            "page ratings are not available to this request",
            ErrorType::PermissionDenied,
        )
    };
    let Some(page) = PageService::get_optional(ctx, site_id, Reference::Id(page_id))
        .await
        .or_raise(|| Error::new("failed to resolve page ratings", ErrorType::PageVote))?
    else {
        return Err(deny().into());
    };
    let can_view =
        PermissionService::check_user_can_view_page(ctx, ctx.request().user_id, &page)
            .await
            .or_raise(|| {
                Error::new(
                    "failed to check page rating visibility",
                    ErrorType::Permission,
                )
            })?;
    if !can_view {
        return Err(deny().into());
    }
    Ok(page)
}

pub async fn page_get_direct(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<GetPageOutput>> {
    let GetPageAnyDetails {
        site_id,
        page_id,
        details,
        allow_deleted,
    } = parse!(params, Page);

    let make_error = || {
        Error::new(
            format!("failed to get page ID {} in site ID {}", page_id, site_id),
            ErrorType::Page,
        )
    };

    let page = PageService::get_direct_optional(ctx, page_id, allow_deleted)
        .await
        .or_raise(make_error)?;

    match page {
        None => Ok(None),
        Some(page) => {
            if page.slug.eq_ignore_ascii_case("_admin")
                && !PermissionService::check_user_can_view_page(
                    ctx,
                    ctx.request().user_id,
                    &page,
                )
                .await
                .or_raise(|| {
                    Error::new(
                        "failed to check page view permission",
                        ErrorType::Permission,
                    )
                })?
            {
                return Err(Error::new(
                    "user does not have permission to view this page",
                    ErrorType::PermissionDenied,
                )
                .into());
            }
            build_page_output(ctx, page, details)
                .await
                .or_raise(make_error)
        }
    }
}

pub async fn page_get_deleted(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<GetDeletedPageOutput>> {
    let GetPageSlug { site_id, slug } = parse!(params, Page);
    let slug2 = slug.clone();

    let make_error = || {
        Error::new(
            format!(
                "failed to get deleted page slug '{}' in site ID {}",
                slug2, site_id
            ),
            ErrorType::Page,
        )
    };
    let user_id = ctx.request().user_id().or_raise(|| {
        Error::new(
            "deleted page lookup requires an authenticated request context",
            ErrorType::PermissionDenied,
        )
    })?;

    let deleted_pages = PageService::get_deleted_by_slug(ctx, site_id, &slug)
        .await
        .or_raise(make_error)?;

    let mut result = Vec::new();
    for page in deleted_pages {
        let can_delete = PermissionService::check_user_can(
            ctx,
            &CheckPermissionContext {
                user_id: Some(user_id),
                site_id,
                page_reference: Some(Reference::Id(page.page_id)),
            },
            Permission {
                resource_type: Resource::Page,
                resource_category: Some(Reference::Id(page.page_category_id)),
                action: Action::Delete,
            },
        )
        .await
        .or_raise(make_error)?;

        if can_delete
            && let Some(page) = build_page_deleted_output(ctx, page)
                .await
                .or_raise(make_error)?
        {
            result.push(page);
        }
    }

    Ok(result)
}

pub async fn page_get_score(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<GetPageScoreOutput> {
    let GetPageReference {
        site_id,
        page: reference,
    } = parse!(params, Page);

    let make_error = || Error::new("failed to get page score", ErrorType::Page);

    let page_id = PageService::get_id(ctx, site_id, reference)
        .await
        .or_raise(make_error)?;

    ensure_page_view_permission(ctx, site_id, page_id)
        .await
        .or_raise(make_error)?;

    let score = ScoreService::score(ctx, page_id)
        .await
        .or_raise(make_error)?;

    Ok(GetPageScoreOutput { page_id, score })
}

pub async fn page_get_files(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<GetFileOutput>> {
    let GetPageFiles {
        page_id,
        site_id,
        deleted,
    } = parse!(params, Page);

    let make_error = || Error::new("failed to get files for page", ErrorType::Page);

    ensure_page_view_permission(ctx, site_id, page_id)
        .await
        .or_raise(make_error)?;

    let get_page_files = FileService::get_all(
        ctx,
        site_id,
        page_id,
        deleted.to_option().copied(),
        FileOrder::default(),
    )
    .await
    .or_raise(make_error)?
    .into_iter()
    .map(|file| build_page_file_output(ctx, file));

    let result = try_join_all(get_page_files)
        .await
        .or_raise(make_error)?
        .into_iter()
        .flatten()
        .collect();

    Ok(result)
}

pub async fn page_tags_select(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<String>> {
    PageQueryService::select_tags(ctx, parse!(params, Page))
        .await
        .or_raise(|| Error::new("failed to select page tags", ErrorType::Page))
}

pub async fn page_select(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<String>> {
    PageQueryService::select_pages(ctx, parse!(params, Page))
        .await
        .or_raise(|| Error::new("failed to select pages", ErrorType::Page))
}

pub async fn page_edit(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<EditPageOutput>> {
    let input: EditPage = parse!(params, Page);
    info!("Editing page {:?} in site ID {}", input.page, input.site_id);

    let actor_user_id = require_authenticated_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate page edit actor", ErrorType::Page)
        })?;
    ensure_page_edit_permission(ctx, input.site_id, input.page.clone(), actor_user_id)
        .await
        .or_raise(|| Error::new("failed to check edit permission", ErrorType::Page))?;

    PageService::edit(ctx, input)
        .await
        .or_raise(|| Error::new("failed to edit page", ErrorType::Page))
}

pub async fn wikidot_legacy_set_tags(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<EditPageOutput>> {
    let input: WikidotLegacySetTagsInput = parse!(params, Page);
    let site_id = ctx.request().site_id().or_raise(|| {
        Error::new(
            "set-tags requires a site request context",
            ErrorType::PermissionDenied,
        )
    })?;
    let actor_user_id = require_authenticated_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate set-tags actor", ErrorType::Page)
        })?;
    ensure_page_edit_permission(
        ctx,
        site_id,
        Reference::Id(input.page_id),
        actor_user_id,
    )
    .await
    .or_raise(|| Error::new("failed to check set-tags permission", ErrorType::Page))?;

    LegacyActionService::set_tags(
        ctx,
        SetLegacyActionTags {
            page_id: input.page_id,
            last_revision_id: input.last_revision_id,
            action_index: input.action_index,
            action_fingerprint: input.action_fingerprint,
            user_id: actor_user_id,
            ip_address: input.ip_address,
        },
    )
    .await
    .or_raise(|| Error::new("failed to apply set-tags action", ErrorType::Page))
}

pub async fn page_edit_permission(
    ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<PageEditPermissionOutput> {
    let can_edit = PageService::check_user_permission(ctx, Action::Edit)
        .await
        .or_raise(|| {
            Error::new("failed to check page edit permission", ErrorType::Page)
        })?;

    Ok(PageEditPermissionOutput { can_edit })
}

pub async fn page_delete(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<DeletePageOutput> {
    let input: DeletePage = parse!(params, Page);
    info!(
        "Deleting page {:?} in site ID {}",
        input.page, input.site_id,
    );

    let actor_user_id = require_authenticated_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate page delete actor", ErrorType::Page)
        })?;
    ensure_page_action_permission(
        ctx,
        input.site_id,
        input.page.clone(),
        actor_user_id,
        Action::Delete,
        "delete",
    )
    .await
    .or_raise(|| Error::new("failed to check page delete permission", ErrorType::Page))?;

    PageService::delete(ctx, input)
        .await
        .or_raise(|| Error::new("failed to delete page", ErrorType::Page))
}

pub async fn page_move(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<MovePageOutput> {
    let input: MovePage = parse!(params, Page);
    info!(
        "Moving page {:?} in site ID {} to {}",
        input.page, input.site_id, input.new_slug,
    );

    let actor_user_id = require_authenticated_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate page move actor", ErrorType::Page)
        })?;
    ensure_page_action_permission(
        ctx,
        input.site_id,
        input.page.clone(),
        actor_user_id,
        Action::Rename,
        "rename",
    )
    .await
    .or_raise(|| Error::new("failed to check page move permission", ErrorType::Page))?;
    ensure_page_create_permission(ctx, input.site_id, &input.new_slug, actor_user_id)
        .await
        .or_raise(|| {
            Error::new(
                "failed to check page move destination permission",
                ErrorType::Page,
            )
        })?;

    PageService::r#move(ctx, input)
        .await
        .or_raise(|| Error::new("failed to move page", ErrorType::Page))
}

pub async fn page_rerender(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let input: PageId = parse!(params, Page);
    let actor_user_id =
        MutationAuthorization::require_authenticated(ctx, "rerender a page")?;
    ensure_page_edit_permission(
        ctx,
        input.site_id,
        Reference::Id(input.page_id),
        actor_user_id,
    )
    .await
    .or_raise(|| {
        Error::new("failed to check page rerender permission", ErrorType::Page)
    })?;
    info!(
        "Re-rendering page ID {} in site ID {}",
        input.page_id, input.site_id,
    );
    PageRevisionService::rerender(
        ctx,
        input,
        RerenderDepth::default(),
        RerenderType::Full,
    )
    .await
    .or_raise(|| Error::new("failed to rerender page", ErrorType::Page))
}

pub async fn page_restore(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<RestorePageOutput> {
    let input: RestorePage = parse!(params, Page);

    info!(
        "Un-deleting page ID {} in site ID {}",
        input.site_id, input.page_id,
    );

    let actor_user_id = require_authenticated_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate page restore actor", ErrorType::Page)
        })?;
    let original_category_id = ensure_deleted_page_delete_permission_and_get_category_id(
        ctx,
        input.site_id,
        input.page_id,
        actor_user_id,
    )
    .await
    .or_raise(|| {
        Error::new("failed to check page restore permission", ErrorType::Page)
    })?;
    if let Some(ref slug) = input.slug {
        ensure_page_create_permission(ctx, input.site_id, slug, actor_user_id)
            .await
            .or_raise(|| {
                Error::new(
                    "failed to check page restore destination permission",
                    ErrorType::Page,
                )
            })?;
    } else {
        ensure_page_permission(
            ctx,
            input.site_id,
            None,
            Some(Reference::Id(original_category_id)),
            actor_user_id,
            Action::Create,
            "restore",
        )
        .await
        .or_raise(|| {
            Error::new(
                "failed to check page restore destination permission",
                ErrorType::Page,
            )
        })?;
    }

    PageService::restore(ctx, input)
        .await
        .or_raise(|| Error::new("failed to restore (undelete) page", ErrorType::Page))
}

pub async fn page_rollback(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<EditPageOutput>> {
    let input: RollbackPage = parse!(params, Page);

    info!(
        "Rolling back page {:?} in site ID {} to revision number {}",
        input.page, input.site_id, input.revision_number,
    );

    let actor_user_id = require_authenticated_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new(
                "failed to authenticate page rollback actor",
                ErrorType::Page,
            )
        })?;
    ensure_page_edit_permission(ctx, input.site_id, input.page.clone(), actor_user_id)
        .await
        .or_raise(|| {
            Error::new("failed to check page rollback permission", ErrorType::Page)
        })?;

    PageService::rollback(ctx, input)
        .await
        .or_raise(|| Error::new("failed to rollback page", ErrorType::Page))
}

pub async fn page_set_layout(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let input: SetPageLayout = parse!(params, Page);

    info!(
        "Setting layout override for page ID {} in site ID {} to layout {} (set by user ID {})",
        input.page_id,
        input.site_id,
        match input.layout {
            Some(layout) => layout.value(),
            None => "none (default)",
        },
        input.user_id,
    );

    let actor_user_id = require_authenticated_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate page layout actor", ErrorType::Page)
        })?;
    ensure_page_edit_permission(
        ctx,
        input.site_id,
        Reference::Id(input.page_id),
        actor_user_id,
    )
    .await
    .or_raise(|| Error::new("failed to check page layout permission", ErrorType::Page))?;

    PageService::set_layout(ctx, input)
        .await
        .or_raise(|| Error::new("failed to set layout for page", ErrorType::Page))
}

pub(super) fn require_authenticated_mutation_actor(
    ctx: &ServiceContext<'_>,
    attribution_user_id: i64,
) -> Result<i64> {
    require_matching_mutation_actor(ctx.request().user_id, attribution_user_id)
}

fn require_matching_mutation_actor(
    request_user_id: Option<i64>,
    attribution_user_id: i64,
) -> Result<i64> {
    let request_user_id = request_user_id.ok_or_raise(|| {
        Error::new(
            "page mutation requires an authenticated request context",
            ErrorType::PermissionDenied,
        )
    })?;

    if request_user_id == attribution_user_id {
        Ok(request_user_id)
    } else {
        Err(Error::new(
            "page mutation user does not match authenticated request user",
            ErrorType::PermissionDenied,
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SiteChangesBrowserPerpage, require_matching_mutation_actor,
        site_changes_browser_perpage,
    };
    use crate::services::render::wikidot_site_changes_empty_response;

    #[test]
    fn page_mutation_requires_matching_request_and_attribution_actors() {
        assert!(require_matching_mutation_actor(None, -1).is_err());
        assert_eq!(require_matching_mutation_actor(Some(-1), -1).unwrap(), -1);
        assert_eq!(require_matching_mutation_actor(Some(42), 42).unwrap(), 42);
        assert!(require_matching_mutation_actor(Some(-1), 42).is_err());
        assert!(require_matching_mutation_actor(Some(42), -1).is_err());
    }

    #[test]
    fn site_changes_browser_perpage_accepts_only_observed_positive_values() {
        for (value, expected) in [("1", 1), ("10", 10), ("20", 20), ("100", 100)] {
            assert_eq!(
                site_changes_browser_perpage(value),
                Some(SiteChangesBrowserPerpage::Rows(expected))
            );
        }
        for value in ["1.5", "5001", "9007199254740993", "not-a-number"] {
            assert_eq!(site_changes_browser_perpage(value), None);
        }
    }

    #[test]
    fn site_changes_browser_perpage_uses_existing_empty_response_for_zero_and_minus_one()
    {
        for value in ["0", "-1"] {
            assert_eq!(
                site_changes_browser_perpage(value),
                Some(SiteChangesBrowserPerpage::Empty)
            );
        }
        let response = wikidot_site_changes_empty_response();
        assert_eq!(response.status, "ok");
        assert_eq!(response.body, "Sorry, no revisions matching your criteria.");
    }
}
