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

use super::prelude::*;
use crate::models::blob_pending::{
    self, Entity as BlobPending, Model as BlobPendingModel,
};
use crate::utils::assert_is_csprng;
use cuid2::cuid;
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use s3::request_trait::ResponseData;
use s3::serde_types::HeadObjectResult;
use std::str;
use time::format_description::well_known::Rfc2822;
use time::{Duration, OffsetDateTime};

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
    /// This is the start to the upload process for any kind of file.
    ///
    /// # Returns
    /// The generated presign URL, which can be uploaded to.
    pub async fn start_upload(
        ctx: &ServiceContext<'_>,
        StartBlobUpload { user_id, blob_size }: StartBlobUpload,
    ) -> Result<StartBlobUploadOutput> {
        info!("Creating upload by {user_id} with promised length {blob_size}");
        let config = ctx.config();
        let txn = ctx.transaction();

        // Convert expected length integer type, then check it
        let blob_size = i64::try_from(blob_size).map_err(|_| Error::BlobTooBig)?;
        if blob_size > config.maximum_blob_size {
            error!(
                "Blob proposed to upload is too big ({} > {})",
                blob_size, config.maximum_blob_size,
            );

            return Err(Error::BlobTooBig);
        }

        // Generate primary key and random S3 path
        let pending_blob_id = cuid();
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

        info!("Creating presign upload URL for blob at path {s3_path} with primary key {pending_blob_id}");

        // Create presign URL
        let bucket = ctx.s3_bucket();
        let presign_url =
            bucket.presign_put(&s3_path, config.presigned_expiry_secs, None)?;

        // Get timestamps
        let created_at = now();
        let expires_at = created_at
            .checked_add(Duration::seconds(i64::from(config.presigned_expiry_secs)))
            .expect("getting expiration timestamp overflowed");

        // Add pending blob entry
        let model = blob_pending::ActiveModel {
            external_id: Set(pending_blob_id),
            expected_length: Set(blob_size),
            s3_path: Set(s3_path),
            presign_url: Set(presign_url),
            created_by: Set(user_id),
            created_at: Set(created_at),
            expires_at: Set(expires_at),
            ..Default::default()
        };

        let BlobPendingModel {
            external_id: pending_blob_id,
            presign_url,
            ..
        } = model.insert(txn).await?;

        debug!("New presign upload URL will last until {expires_at}");

        Ok(StartBlobUploadOutput {
            pending_blob_id,
            presign_url,
            expires_at,
        })
    }

    async fn get_pending_blob_path(
        ctx: &ServiceContext<'_>,
        user_id: i64,
        pending_blob_id: &str,
    ) -> Result<PendingBlob> {
        let txn = ctx.transaction();
        let row = BlobPending::find_by_id(pending_blob_id).one(txn).await?;
        let BlobPendingModel {
            s3_path,
            s3_hash,
            created_by,
            expected_length,
            ..
        } = match row {
            Some(pending) => pending,
            None => return Err(Error::BlobNotFound),
        };

        if user_id != created_by {
            error!("User mismatch, user ID {user_id} is attempting to use blob uploaded by {created_by}");
            return Err(Error::BlobWrongUser);
        }

        Ok(PendingBlob {
            s3_path,
            expected_length,
            moved_hash: s3_hash,
        })
    }

    pub async fn cancel_upload(
        ctx: &ServiceContext<'_>,
        user_id: i64,
        pending_blob_id: &str,
    ) -> Result<()> {
        info!("Cancelling upload for blob for pending ID {pending_blob_id}");
        let txn = ctx.transaction();
        let PendingBlob { s3_path, .. } =
            Self::get_pending_blob_path(ctx, user_id, pending_blob_id).await?;

        BlobPending::delete_by_id(pending_blob_id).exec(txn).await?;

        if Self::head(ctx, &s3_path).await?.is_some() {
            let bucket = ctx.s3_bucket();
            bucket.delete_object(&s3_path).await?;
        }

        Ok(())
    }

    /// Helper function to do the actual "move" step of blob finalization.
    /// This is where, after uploading to the presign URL, the S3 object is
    /// then moved to its permanent location with a hashed name.
    async fn move_uploaded(
        ctx: &ServiceContext<'_>,
        pending_blob_id: &str,
        s3_path: &str,
        expected_length: usize,
    ) -> Result<FinalizeBlobUploadOutput> {
        let bucket = ctx.s3_bucket();
        let txn = ctx.transaction();

        debug!("Download uploaded blob from S3 uploads to get metadata");
        let response = bucket.get_object(s3_path).await?;
        let data: Vec<u8> = match response.status_code() {
            200 => response.into(),
            404 => {
                error!("No blob uploaded at presign path {s3_path}");
                return Err(Error::BlobNotUploaded);
            }
            _ => {
                error!("Unable to retrieve uploaded blob at {s3_path} from S3");
                let error = s3_error(&response, "finalizing uploaded blob")?;
                return Err(error);
            }
        };

        if expected_length != data.len() {
            error!(
                "Expected blob length of {} bytes, instead found {} uploaded. Deleting pending.",
                expected_length,
                data.len(),
            );
            bucket.delete_object(&s3_path).await?;
            return Err(Error::BlobSizeMismatch);
        }

        // Special handling for empty blobs
        if data.is_empty() {
            debug!("File being created is empty, special case");
            return Ok(FinalizeBlobUploadOutput {
                hash: EMPTY_BLOB_HASH,
                mime: str!(EMPTY_BLOB_MIME),
                size: 0,
                created: false,
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

                // TODO: Should we ever update the mime type?
                //       In case of changing file formats, etc.

                // Content-Type header should be returned
                let mime = result.content_type.ok_or(Error::S3Response)?;

                Ok(FinalizeBlobUploadOutput {
                    hash,
                    mime,
                    size,
                    created: false,
                })
            }

            // Blob doesn't exist, "move" it
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
                    _ => s3_error(&response, "creating final S3 blob")?,
                }
            }
        };
        bucket.delete_object(&s3_path).await?;

        // Update pending blob with hash
        let model = blob_pending::ActiveModel {
            external_id: Set(str!(pending_blob_id)),
            s3_hash: Set(Some(hash.to_vec())),
            ..Default::default()
        };
        model.update(txn).await?;

        // Return
        result
    }

    pub async fn finish_upload(
        ctx: &ServiceContext<'_>,
        user_id: i64,
        pending_blob_id: &str,
    ) -> Result<FinalizeBlobUploadOutput> {
        info!("Finishing upload for blob for pending ID {pending_blob_id}");

        let PendingBlob {
            s3_path,
            expected_length,
            moved_hash,
        } = Self::get_pending_blob_path(ctx, user_id, pending_blob_id).await?;

        let output = match moved_hash {
            // Need to move from pending to main hash area
            None => {
                let expected_length = expected_length
                    .try_into()
                    .map_err(|_| Error::BlobSizeMismatch)?;

                Self::move_uploaded(ctx, pending_blob_id, &s3_path, expected_length)
                    .await?
            }

            // Already moved
            Some(hash_vec) => {
                let BlobMetadata { mime, size, .. } =
                    Self::get_metadata(ctx, &hash_vec).await?;

                debug_assert_eq!(expected_length, size);

                let mut hash = [0; 64];
                hash.copy_from_slice(&hash_vec);

                FinalizeBlobUploadOutput {
                    hash,
                    mime,
                    size,
                    created: false,
                }
            }
        };

        // Return result based on blob status
        Ok(output)
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

    #[allow(dead_code)] // TEMP
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
        path: &str,
    ) -> Result<Option<HeadObjectResult>> {
        let bucket = ctx.s3_bucket();
        let (result, status) = bucket.head_object(path).await?;

        match status {
            200 | 204 => Ok(Some(result)),
            404 => Ok(None),
            _ => s3_error(&ResponseData::new(vec![], status), "heading S3 blob"),
        }
    }

    #[allow(dead_code)] // TEMP
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

#[derive(Debug)]
struct PendingBlob {
    s3_path: String,
    expected_length: i64,
    moved_hash: Option<Vec<u8>>,
}
