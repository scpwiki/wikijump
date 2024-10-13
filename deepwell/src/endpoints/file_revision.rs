/*
 * endpoints/file_revision.rs
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
use crate::hash::slice_to_blob_hash;
use crate::models::file_revision::Model as FileRevisionModel;
use crate::services::file::GetFile;
use crate::services::file_revision::{
    FileRevisionCountOutput, GetFileRevision, GetFileRevisionRange, HardDelete,
    HardDeleteOutput, HardDeletionStats, UpdateFileRevision,
};
use crate::types::Bytes;

pub async fn file_revision_count(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<FileRevisionCountOutput> {
    let GetFile {
        site_id,
        page_id,
        file: file_reference,
    } = params.parse()?;

    info!("Getting latest revision for file ID {page_id} in site ID {site_id}",);

    let file_id = FileService::get_id(ctx, site_id, file_reference).await?;
    let revision_count = FileRevisionService::count(ctx, page_id, file_id).await?;

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
    let input: GetFileRevision = params.parse()?;

    info!(
        "Getting file revision {} for file ID {} on page ID {}",
        input.revision_number, input.file_id, input.page_id,
    );

    FileRevisionService::get_optional(ctx, input).await
}

pub async fn file_revision_range(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<FileRevisionModel>> {
    let input: GetFileRevisionRange = params.parse()?;
    FileRevisionService::get_range(ctx, input).await
}

pub async fn file_revision_edit(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<FileRevisionModel> {
    let input: UpdateFileRevision = params.parse()?;

    info!(
        "Editing file revision ID {} for file ID {} on page {}",
        input.revision_id, input.file_id, input.page_id,
    );

    FileRevisionService::update(ctx, input).await
}

pub async fn file_hard_delete_list(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<HardDeletionStats> {
    let input: Bytes = params.parse()?;
    let s3_hash = slice_to_blob_hash(input.as_ref());
    FileRevisionService::hard_delete_list(ctx, s3_hash).await
}

pub async fn file_hard_delete(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<HardDeleteOutput> {
    let input: HardDelete = params.parse()?;
    FileRevisionService::hard_delete_all(ctx, input).await
}
