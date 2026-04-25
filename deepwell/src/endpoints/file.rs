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
use crate::{
    models::{file::Model as FileModel, file_revision::Model as FileRevisionModel},
    services::{
        BlobService, FileRevisionService,
        file::{
            CreateFile, CreateFileOutput, DeleteFile, DeleteFileOutput, EditFile,
            EditFileOutput, GetFileDetails, GetFileOutput, MoveFile, MoveFileOutput,
            RestoreFile, RestoreFileOutput, RollbackFile,
        },
    },
    types::{Bytes, FileDetails},
};

pub async fn file_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<GetFileOutput>> {
    let GetFileDetails { input, details } = parse!(params, File);

    info!(
        "Getting file {:?} from page ID {} in site ID {}",
        input.file, input.page_id, input.site_id,
    );

    let make_error = || Error::new("failed to get file", ErrorType::File);

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

    FileService::rollback(ctx, input)
        .await
        .or_raise(|| Error::new("failed to rollback file", ErrorType::File))
}

async fn build_file_response(
    ctx: &ServiceContext<'_>,
    file: FileModel,
    revision: FileRevisionModel,
    details: FileDetails,
) -> Result<GetFileOutput> {
    let data = BlobService::get_maybe(ctx, details.data, &revision.s3_hash)
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
