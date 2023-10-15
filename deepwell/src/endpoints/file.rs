/*
 * endpoints/file.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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
    EditFile, EditFileOutput, GetFileDetails, GetFileOutput, UploadFile, UploadFileOutput,
};
use crate::services::Result;
use crate::web::{Bytes, FileDetails};

pub async fn file_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<GetFileOutput>> {
    let GetFileDetails { input, details } = params.parse()?;

    tide::log::info!(
        "Getting file {:?} from page ID {} in site ID {}",
        input.file,
        input.page_id,
        input.site_id,
    );

    // We cannot use get_id() because we need File for build_file_response().
    match FileService::get_optional(&ctx, input).await? {
        None => Ok(None),
        Some(file) => {
            let revision =
                FileRevisionService::get_latest(&ctx, file.page_id, file.file_id).await?;
            let output = build_file_response(&ctx, file, revision, details).await?;
            Ok(Some(output))
        }
    }
}

pub async fn file_upload(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<UploadFileOutput> {
    let input: UploadFile = params.parse()?;

    tide::log::info!(
        "Uploading file '{}' ({} bytes) to page ID {} in site ID {}",
        input.name,
        input.data.len(),
        input.page_id,
        input.site_id,
    );

    FileService::upload(ctx, input).await
}

pub async fn file_edit(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<EditFileOutput>> {
    let input: EditFile = params.parse()?;

    tide::log::info!(
        "Editing file ID {} in page ID {} in site ID {}",
        input.file_id,
        input.page_id,
        input.site_id,
    );

    FileService::edit(ctx, input).await
}

pub async fn file_delete(
    _ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<()> {
    todo!()
}

pub async fn file_move(
    _ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<()> {
    todo!()
}

pub async fn file_restore(
    _ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<()> {
    todo!()
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
