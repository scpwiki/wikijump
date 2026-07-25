/*
 * endpoints/blob.rs
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
use crate::hash::slice_to_blob_hash;
use crate::services::MutationAuthorization;
use crate::services::blob::{
    BlobMetadata, CancelBlobUpload, GetBlobOutput, HardDelete, HardDeleteOutput,
    StartBlobUpload, StartBlobUploadOutput,
};
use crate::types::Bytes;

/// Temporary endpoint to get any blob by hash.
/// Primarily for user avatars, which have no other
/// way of getting the data at the moment.
pub async fn blob_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<GetBlobOutput> {
    let hash: Bytes = parse!(params, Blob);

    let make_error = || Error::new("failed to get blob data", ErrorType::Blob);

    let data = BlobService::get(ctx, hash.as_ref())
        .await
        .or_raise(make_error)?;

    let BlobMetadata {
        mime,
        size,
        created_at,
    } = BlobService::get_metadata(ctx, hash.as_ref())
        .await
        .or_raise(make_error)?;

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
    } = parse!(params, Blob);
    MutationAuthorization::require_matching_actor(
        ctx,
        user_id,
        "cancel a pending blob upload",
    )?;

    BlobService::cancel_upload(ctx, user_id, &pending_blob_id)
        .await
        .or_raise(|| {
            Error::new(
                format!(
                    "failed to cancel a pending blob upload with ID {}",
                    pending_blob_id,
                ),
                ErrorType::Blob,
            )
        })
}

/// Starts a new upload by creating a pending blob.
pub async fn blob_upload(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<StartBlobUploadOutput> {
    info!("Creating new pending blob upload");
    let input: StartBlobUpload = parse!(params, Blob);
    MutationAuthorization::require_matching_actor(
        ctx,
        input.user_id,
        "start a blob upload",
    )?;

    BlobService::start_upload(ctx, input)
        .await
        .or_raise(|| Error::new("failed to start a blob upload", ErrorType::Blob))
}

pub async fn blob_blacklist_add(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    #[derive(Deserialize, Debug)]
    struct AddBlacklist {
        s3_hash: Bytes<'static>,
    }

    let make_error =
        || Error::new("failed to add a blob to the blacklist", ErrorType::Blob);

    let AddBlacklist { s3_hash } = params.parse().or_raise(make_error)?;
    let s3_hash = slice_to_blob_hash(s3_hash.as_ref());

    BlobService::add_blacklist(ctx, s3_hash)
        .await
        .or_raise(make_error)
}

pub async fn blob_blacklist_remove(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    #[derive(Deserialize, Debug)]
    struct RemoveBlacklist {
        s3_hash: Bytes<'static>,
    }

    let make_error = || {
        Error::new(
            "failed to remove a blob from the blacklist",
            ErrorType::Blob,
        )
    };

    let RemoveBlacklist { s3_hash } = params.parse().or_raise(make_error)?;
    let s3_hash = slice_to_blob_hash(s3_hash.as_ref());

    BlobService::remove_blacklist(ctx, s3_hash)
        .await
        .or_raise(make_error)
}

pub async fn blob_blacklist_check(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<bool> {
    #[derive(Deserialize, Debug)]
    struct HasBlacklist {
        s3_hash: Bytes<'static>,
    }

    let HasBlacklist { s3_hash } = parse!(params, Blob);
    let s3_hash = slice_to_blob_hash(s3_hash.as_ref());

    BlobService::on_blacklist(ctx, s3_hash).await.or_raise(|| {
        Error::new(
            "failed to check a blob against the blacklist",
            ErrorType::Blob,
        )
    })
}

pub async fn blob_hard_delete_preview(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<HardDeleteOutput> {
    #[derive(Deserialize, Debug)]
    struct HardDeletePreview {
        s3_hash: Bytes<'static>,
    }

    let HardDeletePreview { s3_hash } = parse!(params, Blob);

    BlobService::hard_delete_preview(ctx, s3_hash.as_ref())
        .await
        .or_raise(|| {
            Error::new("failed to preview a blob hard deletion", ErrorType::Blob)
        })
}

pub async fn blob_hard_delete_confirm(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<HardDeleteOutput> {
    let input: HardDelete = parse!(params, Blob);

    BlobService::hard_delete_all(ctx, input)
        .await
        .or_raise(|| Error::new("failed to hard delete a blob", ErrorType::Blob))
}
