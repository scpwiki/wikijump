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
    DeleteFile, DeleteFileOutput, EditFile, EditFileOutput, FinishFileCreation,
    FinishFileCreationOutput, GetBlobOutput, GetFileDetails, GetFileOutput, MoveFile,
    MoveFileOutput, RestoreFile, RestoreFileOutput, StartFileCreation,
    StartFileCreationOutput,
};
use crate::services::Result;
use crate::web::{Bytes, FileDetails};

/// Temporary endpoint to get any blob by hash.
/// Primarily for user avatars, which have no other
/// way of getting the data at the moment.
pub async fn blob_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<GetBlobOutput> {
    info!("Getting blob for S3 hash");
    let hash: Bytes = params.parse()?;
    let data = BlobService::get(ctx, hash.as_ref()).await?;
    let metadata = BlobService::get_metadata(ctx, hash.as_ref()).await?;

    let output = GetBlobOutput {
        data,
        mime: metadata.mime,
        size: metadata.size,
    };
    Ok(output)
}

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

pub async fn file_create_start(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<StartFileCreationOutput> {
    let input: StartFileCreation = params.parse()?;

    info!(
        "Starting file upload '{}' to page ID {} in site ID {}",
        input.name, input.page_id, input.site_id,
    );

    FileService::start_new_upload(ctx, input).await
}

pub async fn file_create_finish(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<FinishFileCreationOutput> {
    let input: FinishFileCreation = params.parse()?;

    info!(
        "Finishing file upload (pending blob ID {} for file ID {} in page ID {} in site ID {}",
        input.pending_blob_id,
        input.file_id,
        input.page_id,
        input.site_id,
    );

    FileService::finish_new_upload(ctx, input).await
}

// TODO
pub async fn file_edit_start(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    todo!()
}

// TODO
pub async fn file_edit_finish(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    todo!()
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

pub async fn file_hard_delete(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let file_id: i64 = params.one()?;

    info!(
        "Hard deleting file ID {file_id} and all duplicates, including underlying data",
    );

    FileService::hard_delete_all(ctx, file_id).await
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
