/*
 * endpoints/blob.rs
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
use crate::services::blob::{
    BlobMetadata, CancelBlobUpload, GetBlobOutput, StartBlobUpload, StartBlobUploadOutput,
};
use crate::services::Result;
use crate::web::Bytes;

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

    let BlobMetadata {
        mime,
        size,
        created_at,
    } = BlobService::get_metadata(ctx, hash.as_ref()).await?;

    Ok(GetBlobOutput {
        data,
        mime,
        size,
        created_at,
    })
}

/// Cancel a started upload by removing the pending blob.
pub async fn blob_cancel(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    info!("Cancelling a pending blob upload");

    let CancelBlobUpload {
        user_id,
        pending_blob_id,
    } = params.parse()?;

    BlobService::cancel_upload(ctx, user_id, &pending_blob_id).await
}

/// Starts a new upload by creating a pending blob.
pub async fn blob_upload(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<StartBlobUploadOutput> {
    info!("Creating new pending blob upload");
    let input: StartBlobUpload = params.parse()?;
    BlobService::start_upload(ctx, input).await
}
