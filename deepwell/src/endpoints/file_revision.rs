/*
 * endpoints/file_revision.rs
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

use super::file::ensure_parent_page_view_permission;
use super::prelude::*;
use crate::models::file_revision::Model as FileRevisionModel;
use crate::services::MutationAuthorization;
use crate::services::file::GetFile;
use crate::services::file_revision::{
    CountFileRevisions, FileRevisionCountOutput, GetFileRevision, GetFileRevisionRange,
    UpdateFileRevision,
};
use crate::types::{Action, Permission, Reference, Resource};

pub async fn file_revision_count(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<FileRevisionCountOutput> {
    let input: GetFile<'_> = parse!(params, FileRevision);
    let site_id = input.site_id;
    let page_id = input.page_id;

    let make_error = || {
        Error::new(
            "failed to get count of file revisions",
            ErrorType::FileRevision,
        )
    };

    ensure_parent_page_view_permission(ctx, site_id, page_id)
        .await
        .or_raise(make_error)?;

    let file_id = FileService::get_id(ctx, input).await.or_raise(make_error)?;

    let revision_count = FileRevisionService::count(
        ctx,
        CountFileRevisions {
            site_id,
            page_id,
            file_id,
        },
    )
    .await
    .or_raise(make_error)?;

    Ok(FileRevisionCountOutput {
        revision_count,
        first_revision: 0,
        last_revision: revision_count.get() - 1,
    })
}

pub async fn file_revision_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<FileRevisionModel>> {
    let input: GetFileRevision = parse!(params, FileRevision);
    ensure_parent_page_view_permission(ctx, input.site_id, input.page_id)
        .await
        .or_raise(|| {
            Error::new(
                "failed to check file revision parent-page visibility",
                ErrorType::FileRevision,
            )
        })?;

    FileRevisionService::get_optional(ctx, input)
        .await
        .or_raise(|| Error::new("failed to get file revision", ErrorType::FileRevision))
}

pub async fn file_revision_range(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<FileRevisionModel>> {
    let input: GetFileRevisionRange = parse!(params, FileRevision);
    ensure_parent_page_view_permission(ctx, input.site_id, input.page_id)
        .await
        .or_raise(|| {
            Error::new(
                "failed to check file revision range parent-page visibility",
                ErrorType::FileRevision,
            )
        })?;

    FileRevisionService::get_range(ctx, input)
        .await
        .or_raise(|| {
            Error::new(
                "failed to get range of file revisions",
                ErrorType::FileRevision,
            )
        })
}

pub async fn file_revision_edit(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<FileRevisionModel> {
    let input: UpdateFileRevision = parse!(params, FileRevision);
    MutationAuthorization::require_matching_actor(
        ctx,
        input.user_id,
        "edit file revision visibility",
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
        "edit file revision visibility",
    )
    .await?;

    info!(
        "Editing file revision ID {} for file ID {} on page {}",
        input.revision_id, input.file_id, input.page_id,
    );

    FileRevisionService::update(ctx, input)
        .await
        .or_raise(|| Error::new("failed to edit file revision", ErrorType::FileRevision))
}
