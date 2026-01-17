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
    RestorePage, RestorePageOutput, RollbackPage, SetPageLayout,
};
use crate::services::page_revision::RerenderType;
use crate::services::{Result, TextService};
use crate::types::{
    Bytes, FileOrder, PageDetails, PageIdGroup, Reference, RerenderDepth,
};
use futures::future::try_join_all;

pub async fn page_create(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<CreatePageOutput> {
    let input: CreatePage = params.parse()?;
    info!("Creating new page in site ID {}", input.site_id);
    PageService::create(ctx, input).await
}

pub async fn page_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<GetPageOutput>> {
    let GetPageReferenceDetails {
        site_id,
        page: reference,
        details,
    } = params.parse()?;

    info!("Getting page {reference:?} in site ID {site_id}");
    match PageService::get_optional(ctx, site_id, reference).await? {
        Some(page) => build_page_output(ctx, page, details).await,
        None => Ok(None),
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
    } = params.parse()?;

    info!("Getting page ID {page_id} in site ID {site_id}");
    match PageService::get_direct_optional(ctx, page_id, allow_deleted).await? {
        Some(page) => build_page_output(ctx, page, details).await,
        None => Ok(None),
    }
}

pub async fn page_get_deleted(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<GetDeletedPageOutput>> {
    let GetPageSlug { site_id, slug } = params.parse()?;

    info!("Getting deleted page {slug} in site ID {site_id}");
    let get_deleted_page = PageService::get_deleted_by_slug(ctx, site_id, &slug)
        .await?
        .into_iter()
        .map(|page| build_page_deleted_output(ctx, page));

    let result = try_join_all(get_deleted_page)
        .await?
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
    } = params.parse()?;

    info!("Getting score for page {reference:?} in site ID {site_id}");
    let page_id = PageService::get_id(ctx, site_id, reference).await?;
    let score = ScoreService::score(ctx, page_id).await?;
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
    } = params.parse()?;

    info!("Getting files for page ID {page_id} in site ID {site_id}");
    let get_page_files = FileService::get_all(
        ctx,
        site_id,
        page_id,
        deleted.to_option().copied(),
        FileOrder::default(),
    )
    .await?
    .into_iter()
    .map(|file| build_page_file_output(ctx, file));

    let result = try_join_all(get_page_files)
        .await?
        .into_iter()
        .flatten()
        .collect();

    Ok(result)
}

pub async fn page_edit(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<EditPageOutput>> {
    let input: EditPage = params.parse()?;
    info!("Editing page {:?} in site ID {}", input.page, input.site_id);
    PageService::edit(ctx, input).await
}

pub async fn page_delete(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<DeletePageOutput> {
    let input: DeletePage = params.parse()?;
    info!(
        "Deleting page {:?} in site ID {}",
        input.page, input.site_id,
    );
    PageService::delete(ctx, input).await
}

pub async fn page_move(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<MovePageOutput> {
    let input: MovePage = params.parse()?;
    info!(
        "Moving page {:?} in site ID {} to {}",
        input.page, input.site_id, input.new_slug,
    );
    PageService::r#move(ctx, input).await
}

pub async fn page_rerender(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let input: PageIdGroup = params.parse()?;
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
}

pub async fn page_restore(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<RestorePageOutput> {
    let input: RestorePage = params.parse()?;
    info!(
        "Un-deleting page ID {} in site ID {}",
        input.id.site_id, input.id.page_id,
    );
    PageService::restore(ctx, input).await
}

pub async fn page_rollback(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<EditPageOutput>> {
    let input: RollbackPage = params.parse()?;

    info!(
        "Rolling back page {:?} in site ID {} to revision number {}",
        input.page, input.site_id, input.revision_number,
    );

    PageService::rollback(ctx, input).await
}

pub async fn page_set_layout(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let input: SetPageLayout = params.parse()?;

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

    PageService::set_layout(ctx, input).await
}

async fn build_page_output(
    ctx: &ServiceContext<'_>,
    page: PageModel,
    details: PageDetails,
) -> Result<Option<GetPageOutput>> {
    // Get page revision
    let revision =
        PageRevisionService::get_latest(ctx, page.site_id, page.page_id).await?;

    // Get category slug from ID
    let category =
        CategoryService::get(ctx, page.site_id, Reference::from(page.page_category_id))
            .await?;

    // Get text data, if requested
    let (wikitext, compiled_body_html) = try_join!(
        TextService::get_conditional(ctx, details.wikitext, &revision.wikitext_hash),
        TextService::get_conditional(
            ctx,
            details.compiled_html,
            &revision.compiled_body_html_hash,
        ),
    )?;

    // Calculate score and determine layout
    let (rating, layout) = try_join!(
        ScoreService::score(ctx, page.page_id),
        SettingsService::get_layout(ctx, page.site_id, Some(page.page_id)),
    )?;

    // Build result struct
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
    // Get page revision
    let revision =
        PageRevisionService::get_latest(ctx, page.site_id, page.page_id).await?;

    // Calculate score and determine layout
    let rating = ScoreService::score(ctx, page.page_id).await?;

    // Build result struct
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
    // Get file revision
    let revision =
        FileRevisionService::get_latest(ctx, file.site_id, file.page_id, file.file_id)
            .await?;

    // Build result struct
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
