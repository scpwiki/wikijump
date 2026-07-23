/*
 * endpoints/page_revision.rs
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
use crate::models::page_revision::Model as PageRevisionModel;
use crate::services::page::GetPageReference;
use crate::services::page_revision::{
    CountPageRevisions, GetPageRevisionDetails, GetPageRevisionRangeDetails,
    PageRevisionCountOutput, PageRevisionModelFiltered, UpdatePageRevisionDetails,
};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::{MutationAuthorization, TextService};
use crate::types::{Action, PageDetails, Permission, Reference, Resource};

pub async fn page_revision_count(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<PageRevisionCountOutput> {
    let GetPageReference {
        site_id,
        page: reference,
    } = parse!(params, PageRevision);

    let make_error =
        || Error::new("failed to get page revision count", ErrorType::PageRevision);

    let page_id = ensure_page_view_permission(ctx, site_id, reference)
        .await
        .or_raise(make_error)?;

    let revision_count =
        PageRevisionService::count(ctx, CountPageRevisions { site_id, page_id })
            .await
            .or_raise(make_error)?;

    Ok(PageRevisionCountOutput {
        revision_count,
        first_revision: 0,
        last_revision: revision_count.get() - 1,
    })
}

pub async fn page_revision_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<PageRevisionModelFiltered>> {
    let GetPageRevisionDetails { input, details } = parse!(params, PageRevision);

    let make_error =
        || Error::new("failed to get a page revision", ErrorType::PageRevision);

    ensure_page_view_permission(ctx, input.site_id, Reference::Id(input.page_id))
        .await
        .or_raise(make_error)?;

    let revision = PageRevisionService::get_optional(ctx, input)
        .await
        .or_raise(make_error)?;

    match revision {
        None => Ok(None),
        Some(revision) => {
            let revision = filter_and_populate_revision(ctx, revision, details)
                .await
                .or_raise(make_error)?;

            Ok(Some(revision))
        }
    }
}

pub async fn page_revision_edit(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<PageRevisionModelFiltered> {
    let UpdatePageRevisionDetails { input, details } = parse!(params, PageRevision);
    MutationAuthorization::require_matching_actor(
        ctx,
        input.user_id,
        "edit page revision visibility",
    )?;
    MutationAuthorization::require_permission(
        ctx,
        input.site_id,
        Some(Reference::Id(input.page_id)),
        Permission {
            resource_type: Resource::Page,
            resource_category: None,
            action: Action::Edit,
        },
        "edit page revision visibility",
    )
    .await?;

    info!(
        "Editing revision ID {} for page ID {} in site ID {}",
        input.revision_id, input.page_id, input.site_id,
    );

    let make_error =
        || Error::new("failed to edit a page revision", ErrorType::PageRevision);

    let revision_id = input.revision_id;
    PageRevisionService::update(ctx, input)
        .await
        .or_raise(make_error)?;
    let revision = PageRevisionService::get_direct(ctx, revision_id).await;
    let revision = raise_multiple!(revision; make_error);

    filter_and_populate_revision(ctx, revision, details)
        .await
        .or_raise(make_error)
}

pub async fn page_revision_range(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<PageRevisionModelFiltered>> {
    let GetPageRevisionRangeDetails { input, details } = parse!(params, PageRevision);

    let make_error = || {
        Error::new(
            "failed to get a range of page revisions",
            ErrorType::PageRevision,
        )
    };

    ensure_page_view_permission(ctx, input.site_id, Reference::Id(input.page_id))
        .await
        .or_raise(make_error)?;

    let revisions = PageRevisionService::get_range(ctx, input)
        .await
        .or_raise(make_error)?;

    filter_and_populate_revisions(ctx, revisions, details)
        .await
        .or_raise(make_error)
}

// Helper functions

async fn ensure_page_view_permission(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_reference: Reference<'_>,
) -> Result<i64> {
    let make_error = || {
        Error::new(
            "failed to check page view permission",
            ErrorType::Permission,
        )
    };

    let page = PageService::get(ctx, site_id, page_reference)
        .await
        .or_raise(make_error)?;

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
    .or_raise(make_error)?;

    if can_view {
        Ok(page.page_id)
    } else {
        Err(Error::new(
            "user does not have permission to view this page",
            ErrorType::PermissionDenied,
        )
        .into())
    }
}

async fn filter_and_populate_revision(
    ctx: &ServiceContext<'_>,
    model: PageRevisionModel,
    mut details: PageDetails,
) -> Result<PageRevisionModelFiltered> {
    let PageRevisionModel {
        revision_id,
        revision_type,
        created_at,
        updated_at,
        from_wikidot,
        revision_number,
        page_id,
        site_id,
        user_id,
        changes,
        wikitext_hash,
        compiled_body_html_hash,
        compiled_body_styles_hash,
        compiled_top_bar_html_hash,
        compiled_side_bar_html_hash,
        compiled_at,
        compiled_generator,
        comments,
        hidden,
        title,
        mut alt_title,
        slug,
        tags,
    } = model;

    let make_error = || {
        Error::new(
            "failed to filter and populate revision data",
            ErrorType::PageRevision,
        )
    };

    // Strip hidden fields
    let mut comments = Some(comments);
    let mut title = Some(title);
    // alt-title is already Option and we're not doubling up
    let mut slug = Some(slug);
    let mut tags = Some(tags);

    for field in &hidden {
        // TODO hidden fields aren't standardized yet
        match field.as_str() {
            "wikitext" => details.wikitext = false,
            "compiled" => details.compiled_html = false,
            "comments" => comments = None,
            "title" => title = None,
            "alt_title" => alt_title = None,
            "slug" => slug = None,
            "tags" => tags = None,
            _ => panic!("Unknown field name in hidden: {field}"),
        }
    }

    // Get text data, if requested
    let (
        wikitext,
        compiled_body_html,
        compiled_body_styles,
        compiled_top_bar_html,
        compiled_side_bar_html,
    ) = join!(
        TextService::get_conditional(ctx, details.wikitext, &wikitext_hash),
        TextService::get_conditional(
            ctx,
            details.compiled_html,
            &compiled_body_html_hash,
        ),
        TextService::get_conditional_option(
            ctx,
            details.compiled_html,
            &compiled_body_styles_hash,
        ),
        TextService::get_conditional_option(
            ctx,
            details.compiled_html,
            &compiled_top_bar_html_hash,
        ),
        TextService::get_conditional_option(
            ctx,
            details.compiled_html,
            &compiled_side_bar_html_hash,
        ),
    );

    let wikitext = wikitext.or_raise(make_error)?;
    let compiled_body_html = compiled_body_html.or_raise(make_error)?;
    let compiled_body_styles = compiled_body_styles.or_raise(make_error)?;
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
    let compiled_top_bar_html = compiled_top_bar_html.or_raise(make_error)?;
    let compiled_side_bar_html = compiled_side_bar_html.or_raise(make_error)?;

    Ok(PageRevisionModelFiltered {
        revision_id,
        revision_type,
        created_at,
        updated_at,
        from_wikidot,
        revision_number,
        page_id,
        site_id,
        user_id,
        changes,
        wikitext,
        compiled_body_html,
        compiled_body_styles,
        compiled_top_bar_html,
        compiled_side_bar_html,
        compiled_at,
        compiled_generator,
        comments,
        hidden,
        title,
        alt_title,
        slug,
        tags,
    })
}

async fn filter_and_populate_revisions(
    ctx: &ServiceContext<'_>,
    revisions: Vec<PageRevisionModel>,
    details: PageDetails,
) -> Result<Vec<PageRevisionModelFiltered>> {
    let mut f_revisions = Vec::new();

    let make_error = || {
        Error::new(
            "failed to populate a list of revisions",
            ErrorType::PageRevision,
        )
    };

    for revision in revisions {
        let f_revision = filter_and_populate_revision(ctx, revision, details)
            .await
            .or_raise(make_error)?;

        f_revisions.push(f_revision)
    }

    Ok(f_revisions)
}
