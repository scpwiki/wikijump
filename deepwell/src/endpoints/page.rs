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
use crate::models::file::Model as FileModel;
use crate::models::page::Model as PageModel;
use crate::services::file::{GetFileOutput, GetPageFiles};
use crate::services::page::{
    CreatePage, CreatePageOutput, DeletePage, DeletePageOutput, EditPage, EditPageOutput,
    GetDeletedPageOutput, GetPageAnyDetails, GetPageOutput, GetPageReference,
    GetPageReferenceDetails, GetPageScoreOutput, GetPageSlug, MovePage, MovePageOutput,
    PageEditPermissionOutput, RestorePage, RestorePageOutput, RollbackPage,
    SetPageLayout,
};
use crate::services::page_query::PageQueryService;
use crate::services::page_revision::RerenderType;
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::{MutationAuthorization, TextService};
use crate::types::{
    Action, Bytes, FileOrder, PageDetails, PageId, Permission, Reference, RerenderDepth,
    Resource,
};
use crate::utils::get_category_name;
use futures::future::try_join_all;
use regex::Regex;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::LazyLock;

static WIKIDOT_LIST_PAGES_SET_PAIR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)<span class="set (?P<name_class>[^"]+)"><span class="name">(?P<name>.*?)</span></span><span class="set (?P<value_class>[^"]+)"><span class="value">(?P<value>.*?)</span></span>"#,
    )
    .expect("Wikidot ListPages set-pair expression is valid")
});

#[derive(Deserialize)]
struct WikidotListPagesModuleInput {
    site_id: i64,
    module_body: String,
    parameters: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct WikidotListPagesModuleOutput {
    pub body: String,
}

#[derive(Deserialize)]
struct WikidotPagePreviewInput {
    site_id: i64,
    title: String,
    wikitext: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct WikidotPagePreviewOutput {
    pub body: String,
    pub styles: Vec<String>,
}

pub async fn wikidot_page_preview(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<WikidotPagePreviewOutput> {
    let input: WikidotPagePreviewInput = parse!(params, Page);
    let output = RenderService::render_wikidot_page_preview(
        ctx,
        input.site_id,
        &input.title,
        input.wikitext,
    )
    .await
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
        body: output.html_output.body,
        styles: output.html_output.styles,
    })
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

    let page = PageService::get_optional(ctx, site_id, reference)
        .await
        .or_raise(make_error)?;

    match page {
        None => Ok(None),
        Some(page) => build_page_output(ctx, page, details)
            .await
            .or_raise(make_error),
    }
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
        Some(page) => build_page_output(ctx, page, details)
            .await
            .or_raise(make_error),
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

    let get_deleted_page = PageService::get_deleted_by_slug(ctx, site_id, &slug)
        .await
        .or_raise(make_error)?
        .into_iter()
        .map(|page| build_page_deleted_output(ctx, page));

    let result = try_join_all(get_deleted_page)
        .await
        .or_raise(make_error)?
        .into_iter()
        .flatten()
        .collect();

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
    ensure_page_edit_permission(ctx, input.site_id, input.page.clone(), actor_user_id)
        .await
        .or_raise(|| {
            Error::new("failed to check page delete permission", ErrorType::Page)
        })?;

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
    ensure_page_edit_permission(ctx, input.site_id, input.page.clone(), actor_user_id)
        .await
        .or_raise(|| {
            Error::new("failed to check page move permission", ErrorType::Page)
        })?;
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
    let original_category_id = ensure_deleted_page_edit_permission(
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
    let request_user_id = ctx.request().user_id().or_raise(|| {
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

async fn ensure_page_create_permission(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    slug: &str,
    user_id: i64,
) -> Result<()> {
    let category_slug = get_category_name(slug);
    let category = CategoryService::get_optional(
        ctx,
        site_id,
        Reference::Slug(Cow::Borrowed(category_slug)),
    )
    .await
    .or_raise(|| {
        Error::new(
            "failed to load page category for create permission check",
            ErrorType::Permission,
        )
    })?;

    let resource_category = category.map(|category| Reference::Id(category.category_id));
    ensure_page_permission(
        ctx,
        site_id,
        None,
        resource_category,
        user_id,
        Action::Create,
        "create",
    )
    .await
}

pub(super) async fn ensure_page_edit_permission<'a>(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_reference: Reference<'a>,
    user_id: i64,
) -> Result<()> {
    let page = PageService::get(ctx, site_id, page_reference.clone())
        .await
        .or_raise(|| {
            Error::new(
                "failed to load page for edit permission check",
                ErrorType::Permission,
            )
        })?;

    ensure_page_permission(
        ctx,
        site_id,
        Some(Reference::Id(page.page_id)),
        Some(Reference::Id(page.page_category_id)),
        user_id,
        Action::Edit,
        "edit",
    )
    .await
}

async fn ensure_deleted_page_edit_permission(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_id: i64,
    user_id: i64,
) -> Result<i64> {
    let page = PageService::get_direct(ctx, page_id, true)
        .await
        .or_raise(|| {
            Error::new(
                "failed to load deleted page for edit permission check",
                ErrorType::Permission,
            )
        })?;

    if page.site_id != site_id {
        return Err(Error::new(
            "deleted page is not in the requested site",
            ErrorType::PermissionDenied,
        )
        .into());
    }

    ensure_page_permission(
        ctx,
        site_id,
        Some(Reference::Id(page.page_id)),
        Some(Reference::Id(page.page_category_id)),
        user_id,
        Action::Edit,
        "edit",
    )
    .await?;

    Ok(page.page_category_id)
}

async fn ensure_page_permission<'a>(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_reference: Option<Reference<'a>>,
    resource_category: Option<Reference<'a>>,
    user_id: i64,
    action: Action,
    action_name: &str,
) -> Result<()> {
    let can_mutate = PermissionService::check_user_can(
        ctx,
        &CheckPermissionContext {
            user_id: Some(user_id),
            site_id,
            page_reference,
        },
        Permission {
            resource_type: Resource::Page,
            resource_category,
            action,
        },
    )
    .await
    .or_raise(|| Error::new("failed to check page permission", ErrorType::Permission))?;

    if can_mutate {
        Ok(())
    } else {
        Err(Error::new(
            format!("user does not have permission to {action_name} this page"),
            ErrorType::PermissionDenied,
        )
        .into())
    }
}

async fn ensure_page_view_permission(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_id: i64,
) -> Result<()> {
    let page = PageService::get(ctx, site_id, Reference::Id(page_id))
        .await
        .or_raise(|| {
            Error::new(
                "failed to load page for view permission check",
                ErrorType::Permission,
            )
        })?;

    let can_view = PermissionService::check_user_can(
        ctx,
        &CheckPermissionContext {
            user_id: ctx.request().user_id,
            site_id,
            page_reference: Some(Reference::Id(page.page_id)),
        },
        Permission {
            resource_type: Resource::Page,
            resource_category: Some(Reference::Id(page.page_category_id)),
            action: Action::View,
        },
    )
    .await
    .or_raise(|| {
        Error::new(
            "failed to check page view permission",
            ErrorType::Permission,
        )
    })?;

    if can_view {
        Ok(())
    } else {
        Err(Error::new(
            "user does not have permission to view this page",
            ErrorType::PermissionDenied,
        )
        .into())
    }
}

async fn build_page_output(
    ctx: &ServiceContext<'_>,
    page: PageModel,
    details: PageDetails,
) -> Result<Option<GetPageOutput>> {
    let make_error = || Error::new("failed to build page output", ErrorType::Page);

    let revision = PageRevisionService::get_latest(ctx, page.site_id, page.page_id)
        .await
        .or_raise(make_error)?;

    let category =
        CategoryService::get(ctx, page.site_id, Reference::from(page.page_category_id))
            .await
            .or_raise(make_error)?;

    let (wikitext, compiled_body_html, compiled_body_styles) = join!(
        TextService::get_conditional(ctx, details.wikitext, &revision.wikitext_hash),
        TextService::get_conditional(
            ctx,
            details.compiled_html,
            &revision.compiled_body_html_hash,
        ),
        TextService::get_conditional_option(
            ctx,
            details.compiled_html,
            &revision.compiled_body_styles_hash,
        ),
    );
    let (wikitext, compiled_body_html, compiled_body_styles) =
        raise_multiple!(wikitext, compiled_body_html, compiled_body_styles; make_error);
    let compiled_body_styles = if details.compiled_html {
        Some(
            compiled_body_styles
                .map(|styles| serde_json::from_str(&styles))
                .transpose()
                .or_raise(make_error)?
                .unwrap_or_default(),
        )
    } else {
        None
    };

    let (rating, layout) = join!(
        ScoreService::score(ctx, page.page_id),
        SettingsService::get_layout(ctx, page.site_id, Some(page.page_id)),
    );
    let (rating, layout) = raise_multiple!(rating, layout; make_error);

    Ok(Some(GetPageOutput {
        page_id: page.page_id,
        page_created_at: page.created_at,
        page_updated_at: page.updated_at,
        page_deleted_at: page.deleted_at,
        page_revision_count: revision.revision_number + 1,
        site_id: page.site_id,
        page_category_id: category.category_id,
        page_category_slug: category.slug,
        discussion_thread_id: page.discussion_thread_id,
        revision_id: revision.revision_id,
        revision_type: revision.revision_type,
        revision_created_at: revision.created_at,
        revision_number: revision.revision_number,
        revision_user_id: revision.user_id,
        wikitext,
        compiled_body_html,
        compiled_body_styles,
        compiled_at: revision.compiled_at,
        compiled_generator: revision.compiled_generator,
        revision_comments: revision.comments,
        hidden_fields: revision.hidden,
        title: revision.title,
        alt_title: revision.alt_title,
        slug: revision.slug,
        tags: revision.tags,
        rating,
        layout,
    }))
}

async fn build_page_deleted_output(
    ctx: &ServiceContext<'_>,
    page: PageModel,
) -> Result<Option<GetDeletedPageOutput>> {
    let make_error = || {
        Error::new(
            "failed to build page output for a deleted page",
            ErrorType::Page,
        )
    };

    let revision = PageRevisionService::get_latest(ctx, page.site_id, page.page_id)
        .await
        .or_raise(make_error)?;

    let rating = ScoreService::score(ctx, page.page_id)
        .await
        .or_raise(make_error)?;

    Ok(Some(GetDeletedPageOutput {
        page_id: page.page_id,
        page_created_at: page.created_at,
        page_updated_at: page.updated_at,
        page_deleted_at: page.deleted_at.expect("Page should be deleted"),
        page_revision_count: revision.revision_number,
        site_id: page.site_id,
        discussion_thread_id: page.discussion_thread_id,
        hidden_fields: revision.hidden,
        title: revision.title,
        alt_title: revision.alt_title,
        slug: revision.slug,
        tags: revision.tags,
        rating,
    }))
}

async fn build_page_file_output(
    ctx: &ServiceContext<'_>,
    file: FileModel,
) -> Result<Option<GetFileOutput>> {
    let make_error = || {
        Error::new(
            "failed to build output for a file on a page",
            ErrorType::Page,
        )
    };

    let revision =
        FileRevisionService::get_latest(ctx, file.site_id, file.page_id, file.file_id)
            .await
            .or_raise(make_error)?;

    Ok(Some(GetFileOutput {
        file_id: file.file_id,
        file_created_at: file.created_at,
        file_updated_at: file.updated_at,
        file_deleted_at: file.deleted_at,
        page_id: file.page_id,
        revision_id: revision.revision_id,
        revision_type: revision.revision_type,
        revision_created_at: revision.created_at,
        revision_number: revision.revision_number,
        revision_user_id: revision.user_id,
        name: file.name,
        data: None,
        mime: revision.mime,
        size: revision.size,
        s3_hash: Bytes::from(revision.s3_hash),
        revision_comments: revision.comments,
        hidden_fields: revision.hidden,
    }))
}
