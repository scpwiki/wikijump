/*
 * services/blob/service.rs
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

// TEMP, until https://scuttle.atlassian.net/browse/WJ-1032
#![allow(dead_code)]

use super::prelude::*;
use crate::models::blob_pending::{
    self, Entity as BlobPending, Model as BlobPendingModel,
};
use crate::utils::assert_is_csprng;
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use s3::request_trait::ResponseData;
use s3::serde_types::HeadObjectResult;
use std::str;
use time::format_description::well_known::Rfc2822;
use time::OffsetDateTime;

/// Hash for empty blobs.
///
/// Even though it is not the SHA-512 hash, for simplicity we treat the hash
/// value with all zeroes to be the blob address for the empty blob.
/// This empty blob is not actually stored in S3 but instead is a "virtual blob",
/// considered to have always been present in `BlobService`.
pub const EMPTY_BLOB_HASH: BlobHash = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
];

/// MIME type for empty blobs.
pub const EMPTY_BLOB_MIME: &str = "inode/x-empty; charset=binary";

/// Created UNIX timestamp for empty blobs.
///
/// Timestamp is 2019/01/18 at midnight, the date of the first Wikijump commit.
pub const EMPTY_BLOB_TIMESTAMP: i64 = 1547769600;

/// The subdirectory in the S3 bucket where all pending uploads are kept.
pub const PRESIGN_DIRECTORY: &str = "uploads";

#[derive(Debug)]
pub struct BlobService;

impl BlobService {
    /// Creates an S3 presign URL to allow an end user to upload a blob.
    ///
    /// Also adds an entry for the pending blob upload (`blob_pending`),
    /// so it can be used by the main `blob` table.
    ///
    /// # Returns
    /// The generated presign URL that can be uploaded to.
    pub async fn create_upload(ctx: &ServiceContext<'_>) -> Result<BlobPendingModel> {
        let config = ctx.config();
        let txn = ctx.transaction();

        // Generate random S3 path
        let s3_path = {
            let mut path = format!("{PRESIGN_DIRECTORY}/");

            {
                let mut rng = thread_rng();
                assert_is_csprng(&rng);
                Alphanumeric.append_string(
                    &mut rng,
                    &mut path,
                    config.presigned_path_length,
                );
            }

            path
        };
        info!("Creating presign upload URL for blob at path {s3_path}");

        // Create presign URL
        let bucket = ctx.s3_bucket();
        let presign_url =
            bucket.presign_put(&s3_path, config.presigned_expiry_secs, None)?;

        // Add pending blob entry
        let model = blob_pending::ActiveModel {
            s3_path: Set(s3_path),
            presign_url: Set(presign_url),
            ..Default::default()
        };
        let output = model.insert(txn).await?;
        Ok(output)
    }

    pub async fn finish_upload(
        ctx: &ServiceContext<'_>,
        pending_blob_id: i64,
    ) -> Result<FinalizeBlobUploadOutput> {
        info!("Finishing upload for blob for pending blob ID {pending_blob_id}");
        let bucket = ctx.s3_bucket();
        let txn = ctx.transaction();

        debug!("Getting pending blob info");
        let row = BlobPending::find()
            .filter(blob_pending::Column::PendingBlobId.eq(pending_blob_id))
            .one(txn)
            .await?;

        let BlobPendingModel { s3_path, .. } = match row {
            Some(pending) => pending,
            None => return Err(Error::GeneralNotFound),
        };

        debug!("Download uploaded blob from S3 uploads to get metadata");
        let response = bucket.get_object(&s3_path).await?;
        let data: Vec<u8> = match response.status_code() {
            200 => response.into(),
            _ => {
                error!("Cannot find blob at presign path {s3_path}");
                return Err(Error::FileNotUploaded);
            }
        };

        debug!("Deleting pending blob");
        BlobPending::delete_by_id(pending_blob_id).exec(txn).await?;

        // Special handling for empty blobs
        if data.is_empty() {
            debug!("File being created is empty, special case");
            return Ok(FinalizeBlobUploadOutput {
                hash: EMPTY_BLOB_HASH,
                mime: str!(EMPTY_BLOB_MIME),
                size: 0,
            });
        }

        debug!("Updating blob metadata in database and S3");

        // Convert size to correct integer type
        let size: i64 = data.len().try_into().expect("Buffer size exceeds i64");

        let hash = sha512_hash(&data);
        let hex_hash = blob_hash_to_hex(&hash);

        // If the blob exists, then just delete the uploaded one.
        //
        // If it doesn't, then we need to move it. However, within
        // S3 we cannot "move" objects, we have to upload and delete the original.

        let result = match Self::head(ctx, &hex_hash).await? {
            // Blob exists, copy metadata and return that
            Some(result) => {
                debug!("Blob with hash {hex_hash} already exists");

                // Content-Type header should be returned
                let mime = result.content_type.ok_or(Error::S3Response)?;

                Ok(FinalizeBlobUploadOutput {
                    hash,
                    mime,
                    size,
                    created: false,
                })
            }

            // Blob doesn't exist, move it from uploaded
            None => {
                debug!("Blob with hash {hex_hash} to be created");

                // Determine MIME type for the new blob
                let mime = ctx.mime().get_mime_type(data.to_vec()).await?;

                // Upload S3 object to final destination
                let response = bucket
                    .put_object_with_content_type(&hex_hash, &data, &mime)
                    .await?;

                // We assume all unexpected statuses are errors, even if 1XX or 2XX
                match response.status_code() {
                    200 => Ok(FinalizeBlobUploadOutput {
                        hash,
                        mime,
                        size,
                        created: true,
                    }),
                    _ => s3_error(&response, "creating final S3 blob"),
                }
            }
        };

        // Delete uploaded version, in either case
        bucket.delete_object(&s3_path).await?;

        // Return result based on blob status
        result
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        hash: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        // Special handling for empty blobs
        if hash == EMPTY_BLOB_HASH {
            debug!("Returning the empty blob");
            return Ok(Some(Vec::new()));
        }

        // Retrieve blob from S3
        let bucket = ctx.s3_bucket();
        let hex_hash = blob_hash_to_hex(hash);
        let response = bucket.get_object(&hex_hash).await?;
        match response.status_code() {
            200 => Ok(Some(response.into())),
            404 => Ok(None),
            _ => s3_error(&response, "fetching S3 blob"),
        }
    }

    #[inline]
    pub async fn get(ctx: &ServiceContext<'_>, hash: &[u8]) -> Result<Vec<u8>> {
        find_or_error!(Self::get_optional(ctx, hash), Blob)
    }

    pub async fn get_metadata_optional(
        ctx: &ServiceContext<'_>,
        hash: &[u8],
    ) -> Result<Option<BlobMetadata>> {
        // Special handling for empty blobs
        if hash == EMPTY_BLOB_HASH {
            return Ok(Some(BlobMetadata {
                mime: str!(EMPTY_BLOB_MIME),
                size: 0,
                created_at: OffsetDateTime::from_unix_timestamp(EMPTY_BLOB_TIMESTAMP)
                    .unwrap(),
            }));
        }

        // Retrieve metadata from S3
        let hex_hash = blob_hash_to_hex(hash);
        match Self::head(ctx, &hex_hash).await? {
            None => Ok(None),
            Some(result) => {
                // Headers should be passed in
                let size = result.content_length.ok_or(Error::S3Response)?;
                let mime = result.content_type.ok_or(Error::S3Response)?;
                let created_at = {
                    let timestamp = result.last_modified.ok_or(Error::S3Response)?;

                    OffsetDateTime::parse(&timestamp, &Rfc2822)
                        .map_err(|_| Error::S3Response)?
                };

                Ok(Some(BlobMetadata {
                    mime,
                    size,
                    created_at,
                }))
            }
        }
    }

    #[inline]
    pub async fn get_metadata(
        ctx: &ServiceContext<'_>,
        hash: &[u8],
    ) -> Result<BlobMetadata> {
        find_or_error!(Self::get_metadata_optional(ctx, hash), Blob)
    }

    pub async fn exists(ctx: &ServiceContext<'_>, hash: &[u8]) -> Result<bool> {
        // Special handling for the empty blob
        if hash == EMPTY_BLOB_HASH {
            debug!("Checking existence of the empty blob");
            return Ok(true);
        }

        // Fetch existence from S3
        let hex_hash = blob_hash_to_hex(hash);
        let result = Self::head(ctx, &hex_hash).await?;
        Ok(result.is_some())
    }

    /// Possibly retrieve blob contents, if a flag is set.
    ///
    /// This utility conditionally retrieves the
    /// text given by the specified hash only
    /// if the flag `should_fetch` is true.
    /// Otherwise, it does no action, returning `None`.
    pub async fn get_maybe(
        ctx: &ServiceContext<'_>,
        should_fetch: bool,
        hash: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        if should_fetch {
            let data = Self::get(ctx, hash).await?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    async fn head(
        ctx: &ServiceContext<'_>,
        hex_hash: &str,
    ) -> Result<Option<HeadObjectResult>> {
        let bucket = ctx.s3_bucket();
        let (result, status) = bucket.head_object(hex_hash).await?;

        match status {
            200 | 204 => Ok(Some(result)),
            404 => Ok(None),
            _ => s3_error(&ResponseData::new(vec![], status), "heading S3 blob"),
        }
    }

    pub async fn hard_delete(ctx: &ServiceContext<'_>, hash: &[u8]) -> Result<()> {
        // Special handling for empty blobs
        //
        // Being virtual, having always existed, they cannot be deleted.
        // So this is a no-op.
        if hash == EMPTY_BLOB_HASH {
            debug!("Ignoring attempt to hard delete the empty blob");
            return Ok(());
        }

        // Delete from S3
        let bucket = ctx.s3_bucket();
        let hex_hash = blob_hash_to_hex(hash);

        let response = bucket.delete_object(&hex_hash).await?;
        match response.status_code() {
            204 => Ok(()),
            _ => s3_error(&response, "hard-deleting S3 blob"),
        }
    }
}

/// Helper method to parse out an S3 error response and print the message (if any).
fn s3_error<T>(response: &ResponseData, action: &str) -> Result<T> {
    let error_message = match str::from_utf8(response.bytes()) {
        Ok("") => "(no content)",
        Ok(m) => m,
        Err(_) => "(invalid UTF-8)",
    };

    error!(
        "Error while {} (HTTP {}): {}",
        action,
        response.status_code(),
        error_message,
    );

    // TODO replace with S3 backend-specific error
    Err(Error::S3Response)
}
