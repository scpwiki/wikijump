/*
 * endpoints/file.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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
use crate::services::blob::BlobService;
use crate::services::file::{
    CreateFile, CreateFileOutput, DeleteFile, DeleteFileOutput, EditFile, EditFileOutput,
    GetFileDetails, GetFileOutput, MoveFile, MoveFileOutput, RestoreFile,
    RestoreFileOutput, RollbackFile,
};
use crate::services::Result;
use crate::types::{Bytes, FileDetails};

pub async fn file_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<GetFileOutput>> {
    let GetFileDetails { input, details } = params.parse()?;

    info!(
        "Getting file {:?} from page ID {} in site ID {}",
        input.file, input.page_id, input.site_id,
    );

    // We cannot use get_id() because we need File for build_file_response().
    match FileService::get_optional(ctx, input).await? {
        None => Ok(None),
        Some(file) => {
            let revision = FileRevisionService::get_latest(
                ctx,
                file.site_id,
                file.page_id,
                file.file_id,
            )
            .await?;

            let output = build_file_response(ctx, file, revision, details).await?;
            Ok(Some(output))
        }
    }
}

pub async fn file_create(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<CreateFileOutput> {
    let input: CreateFile = params.parse()?;

    info!(
        "Creating file on page ID {} in site ID {}",
        input.page_id, input.site_id,
    );

    FileService::create(ctx, input).await
}

pub async fn file_edit(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<EditFileOutput>> {
    let input: EditFile = params.parse()?;

    info!(
        "Editing file ID {} in page ID {} in site ID {}",
        input.file_id, input.page_id, input.site_id,
    );

    FileService::edit(ctx, input).await
}

pub async fn file_delete(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<DeleteFileOutput> {
    let input: DeleteFile = params.parse()?;

    info!(
        "Deleting file {:?} in page ID {} in site ID {}",
        input.file, input.page_id, input.site_id,
    );

    FileService::delete(ctx, input).await
}

pub async fn file_move(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<MoveFileOutput>> {
    let input: MoveFile = params.parse()?;

    info!(
        "Moving file ID {} from page ID {} to page ID {} in site ID {}",
        input.file_id, input.current_page_id, input.destination_page_id, input.site_id,
    );

    FileService::r#move(ctx, input).await
}

pub async fn file_restore(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<RestoreFileOutput> {
    let input: RestoreFile = params.parse()?;

    info!(
        "Restoring deleted file ID {} in page ID {} in site ID {}",
        input.file_id, input.page_id, input.site_id,
    );

    FileService::restore(ctx, input).await
}

pub async fn file_rollback(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<EditFileOutput>> {
    let input: RollbackFile = params.parse()?;

    info!(
        "Rolling back file {:?} in page ID {} in site ID {} to revision number {}",
        input.file, input.page_id, input.site_id, input.revision_number,
    );

    FileService::rollback(ctx, input).await
}

async fn build_file_response(
    ctx: &ServiceContext<'_>,
    file: FileModel,
    revision: FileRevisionModel,
    details: FileDetails,
) -> Result<GetFileOutput> {
    let data = BlobService::get_maybe(ctx, details.data, &revision.s3_hash).await?;
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
        mime: revision.mime_hint,
        size: revision.size_hint,
        licensing: revision.licensing,
        revision_comments: revision.comments,
        hidden_fields: revision.hidden,
    })
}
