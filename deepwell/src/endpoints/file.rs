/*
 * endpoints/file.rs
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
use crate::models::file_revision::Model as FileRevisionModel;
use crate::services::file::{
    CreateFile, CreateFileOutput, DeleteFile, DeleteFileOutput, EditFile, EditFileOutput,
    GetFileDetails, GetFileOutput, MoveFile, MoveFileOutput, RestoreFile,
    RestoreFileOutput, RollbackFile,
};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::{BlobService, FileRevisionService};
use crate::types::{Action, Bytes, FileDetails, Permission, Reference, Resource};

pub async fn file_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<GetFileOutput>> {
    let GetFileDetails { input, details } = parse!(params, File);

    let make_error = || Error::new("failed to get file", ErrorType::File);

    ensure_parent_page_view_permission(ctx, input.site_id, input.page_id)
        .await
        .or_raise(make_error)?;

    // We cannot use get_id() because we need File for build_file_response().
    let file = FileService::get_optional(ctx, input)
        .await
        .or_raise(make_error)?;

    match file {
        None => Ok(None),
        Some(file) => {
            let revision = FileRevisionService::get_latest(
                ctx,
                file.site_id,
                file.page_id,
                file.file_id,
            )
            .await
            .or_raise(make_error)?;

            let output = build_file_response(ctx, file, revision, details)
                .await
                .or_raise(make_error)?;

            Ok(Some(output))
        }
    }
}

pub async fn file_create(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<CreateFileOutput> {
    let input: CreateFile = parse!(params, File);

    info!(
        "Creating file on page ID {} in site ID {}",
        input.page_id, input.site_id,
    );

    let actor_user_id = require_authenticated_file_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate file create actor", ErrorType::File)
        })?;
    ensure_parent_page_permission(
        ctx,
        input.site_id,
        input.page_id,
        Some(actor_user_id),
        Action::Edit,
        "edit this file's parent page",
    )
    .await
    .or_raise(|| Error::new("failed to check file create permission", ErrorType::File))?;

    FileService::create(ctx, input)
        .await
        .or_raise(|| Error::new("failed to create file", ErrorType::File))
}

pub async fn file_edit(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<EditFileOutput>> {
    let input: EditFile = parse!(params, File);

    info!(
        "Editing file ID {} in page ID {} in site ID {}",
        input.file_id, input.page_id, input.site_id,
    );

    let actor_user_id = require_authenticated_file_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate file edit actor", ErrorType::File)
        })?;
    ensure_parent_page_permission(
        ctx,
        input.site_id,
        input.page_id,
        Some(actor_user_id),
        Action::Edit,
        "edit this file's parent page",
    )
    .await
    .or_raise(|| Error::new("failed to check file edit permission", ErrorType::File))?;

    FileService::edit(ctx, input)
        .await
        .or_raise(|| Error::new("failed to edit file", ErrorType::File))
}

pub async fn file_delete(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<DeleteFileOutput> {
    let input: DeleteFile = parse!(params, File);

    info!(
        "Deleting file {:?} in page ID {} in site ID {}",
        input.file, input.page_id, input.site_id,
    );

    let actor_user_id = require_authenticated_file_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate file delete actor", ErrorType::File)
        })?;
    ensure_parent_page_permission(
        ctx,
        input.site_id,
        input.page_id,
        Some(actor_user_id),
        Action::Edit,
        "edit this file's parent page",
    )
    .await
    .or_raise(|| Error::new("failed to check file delete permission", ErrorType::File))?;

    FileService::delete(ctx, input)
        .await
        .or_raise(|| Error::new("failed to delete file", ErrorType::File))
}

pub async fn file_move(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<MoveFileOutput>> {
    let input: MoveFile = parse!(params, File);

    info!(
        "Moving file ID {} from page ID {} to page {:?} in site ID {}",
        input.file_id, input.current_page_id, input.destination_page, input.site_id,
    );

    let actor_user_id = require_authenticated_file_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate file move actor", ErrorType::File)
        })?;
    ensure_parent_page_permission(
        ctx,
        input.site_id,
        input.current_page_id,
        Some(actor_user_id),
        Action::Edit,
        "edit this file's current parent page",
    )
    .await
    .or_raise(|| {
        Error::new(
            "failed to check file current-page move permission",
            ErrorType::File,
        )
    })?;
    let destination =
        PageService::get(ctx, input.site_id, input.destination_page.clone())
            .await
            .or_raise(|| {
                Error::new(
                    "failed to load file destination page for move permission check",
                    ErrorType::File,
                )
            })?;
    ensure_parent_page_permission(
        ctx,
        input.site_id,
        destination.page_id,
        Some(actor_user_id),
        Action::Edit,
        "edit this file's destination page",
    )
    .await
    .or_raise(|| {
        Error::new(
            "failed to check file destination-page move permission",
            ErrorType::File,
        )
    })?;

    FileService::r#move(ctx, input)
        .await
        .or_raise(|| Error::new("failed to move file", ErrorType::File))
}

pub async fn file_restore(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<RestoreFileOutput> {
    let input: RestoreFile = parse!(params, File);

    info!(
        "Restoring deleted file ID {} in page ID {} in site ID {}",
        input.file_id, input.page_id, input.site_id,
    );

    let actor_user_id = require_authenticated_file_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate file restore actor", ErrorType::File)
        })?;
    ensure_parent_page_permission(
        ctx,
        input.site_id,
        input.page_id,
        Some(actor_user_id),
        Action::Edit,
        "edit this file's parent page",
    )
    .await
    .or_raise(|| {
        Error::new("failed to check file restore permission", ErrorType::File)
    })?;
    if let Some(ref new_page) = input.new_page {
        let destination = PageService::get(ctx, input.site_id, new_page.clone())
            .await
            .or_raise(|| {
                Error::new(
                    "failed to load file restore destination page for permission check",
                    ErrorType::File,
                )
            })?;
        ensure_parent_page_permission(
            ctx,
            input.site_id,
            destination.page_id,
            Some(actor_user_id),
            Action::Edit,
            "edit this file's restore destination page",
        )
        .await
        .or_raise(|| {
            Error::new(
                "failed to check file restore destination-page permission",
                ErrorType::File,
            )
        })?;
    }

    FileService::restore(ctx, input)
        .await
        .or_raise(|| Error::new("failed to restore file", ErrorType::File))
}

pub async fn file_rollback(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<EditFileOutput>> {
    let input: RollbackFile = parse!(params, File);

    info!(
        "Rolling back file {:?} in page ID {} in site ID {} to revision number {}",
        input.file, input.page_id, input.site_id, input.revision_number,
    );

    let actor_user_id = require_authenticated_file_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new(
                "failed to authenticate file rollback actor",
                ErrorType::File,
            )
        })?;
    ensure_parent_page_permission(
        ctx,
        input.site_id,
        input.page_id,
        Some(actor_user_id),
        Action::Edit,
        "edit this file's parent page",
    )
    .await
    .or_raise(|| {
        Error::new("failed to check file rollback permission", ErrorType::File)
    })?;

    FileService::rollback(ctx, input)
        .await
        .or_raise(|| Error::new("failed to rollback file", ErrorType::File))
}

fn require_authenticated_file_mutation_actor(
    ctx: &ServiceContext<'_>,
    attribution_user_id: i64,
) -> Result<i64> {
    let request_user_id = ctx.request().user_id().or_raise(|| {
        Error::new(
            "file mutation requires an authenticated request context",
            ErrorType::PermissionDenied,
        )
    })?;

    if request_user_id == attribution_user_id {
        Ok(request_user_id)
    } else {
        Err(Error::new(
            "file mutation user does not match authenticated request user",
            ErrorType::PermissionDenied,
        )
        .into())
    }
}

pub(super) async fn ensure_parent_page_view_permission(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_id: i64,
) -> Result<()> {
    ensure_parent_page_permission(
        ctx,
        site_id,
        page_id,
        ctx.request().user_id,
        Action::View,
        "view this file's parent page",
    )
    .await
}

async fn ensure_parent_page_permission(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_id: i64,
    user_id: Option<i64>,
    action: Action,
    action_name: &str,
) -> Result<()> {
    let make_error = || {
        Error::new(
            "failed to check parent page permission",
            ErrorType::Permission,
        )
    };

    let page = PageService::get(ctx, site_id, Reference::Id(page_id))
        .await
        .or_raise(make_error)?;

    let can_access = PermissionService::check_user_can(
        ctx,
        &CheckPermissionContext {
            user_id,
            site_id,
            page_reference: Some(Reference::Id(page.page_id)),
        },
        Permission {
            resource_type: Resource::Page,
            resource_category: Some(Reference::Id(page.page_category_id)),
            action,
        },
    )
    .await
    .or_raise(make_error)?;

    if can_access {
        Ok(())
    } else {
        Err(Error::new(
            format!("user does not have permission to {action_name}"),
            ErrorType::PermissionDenied,
        )
        .into())
    }
}

async fn build_file_response(
    ctx: &ServiceContext<'_>,
    file: FileModel,
    revision: FileRevisionModel,
    details: FileDetails,
) -> Result<GetFileOutput> {
    let data = BlobService::fetch_if_requested(ctx, details.data, &revision.s3_hash)
        .await
        .or_raise(|| Error::new("failed to build a file response", ErrorType::File))?;

    Ok(GetFileOutput {
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
        data: data.map(Bytes::from),
        mime: revision.mime,
        size: revision.size,
        s3_hash: Bytes::from(revision.s3_hash),
        revision_comments: revision.comments,
        hidden_fields: revision.hidden,
    })
}
