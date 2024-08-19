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
use s3::request_trait::ResponseData;
use s3::serde_types::HeadObjectResult;
use std::str;
use time::format_description::well_known::Rfc2822;
use time::OffsetDateTime;

/// Hash for empty blobs.
///
/// Even though it is not the SHA-512 hash, for simplicity we treat the hash
/// value with all zeroes to be the blob address for the empty blob.
/// This empty file is not actually stored in S3 but instead is a "virtual file",
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

#[derive(Debug)]
pub struct BlobService;

impl BlobService {
    /// Creates a blob with this data, if it does not already exist.
    pub async fn create<B: AsRef<[u8]>>(
        ctx: &ServiceContext,
        data: B,
    ) -> Result<CreateBlobOutput> {
        let data = data.as_ref();
        info!("Creating blob (length {})", data.len());

        // Special handling for empty blobs
        if data.is_empty() {
            debug!("File being created is empty, special case");
            return Ok(CreateBlobOutput {
                hash: EMPTY_BLOB_HASH,
                mime: str!(EMPTY_BLOB_MIME),
                size: 0,
            });
        }

        // Upload blob
        let bucket = ctx.s3_bucket();
        let hash = sha512_hash(data);
        let hex_hash = blob_hash_to_hex(&hash);

        // Convert size to correct integer type
        let size: i64 = data.len().try_into().expect("Buffer size exceeds i64");

        match Self::head(ctx, &hex_hash).await? {
            // Blob exists, copy metadata and return that
            Some(result) => {
                debug!("Blob with hash {hex_hash} already exists");

                // Content-Type header should be passed in
                let mime = result.content_type.ok_or(Error::S3Response)?;

                Ok(CreateBlobOutput { hash, mime, size })
            }

            // Blob doesn't exist, insert it
            None => {
                debug!("Blob with hash {hex_hash} to be created");

                // Determine MIME type for the new file
                let mime = ctx.mime().get_mime_type(data.to_vec()).await?;

                // Put into S3
                let response = bucket
                    .put_object_with_content_type(&hex_hash, data, &mime)
                    .await?;

                // We assume all unexpected statuses are errors, even if 1XX or 2XX
                match response.status_code() {
                    200 => Ok(CreateBlobOutput { hash, mime, size }),
                    _ => s3_error(&response, "creating S3 blob"),
                }
            }
        }
    }

    pub async fn get_optional(
        ctx: &ServiceContext,
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
    pub async fn get(ctx: &ServiceContext, hash: &[u8]) -> Result<Vec<u8>> {
        find_or_error!(Self::get_optional(ctx, hash), Blob)
    }

    pub async fn get_metadata_optional(
        ctx: &ServiceContext,
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
    pub async fn get_metadata(ctx: &ServiceContext, hash: &[u8]) -> Result<BlobMetadata> {
        find_or_error!(Self::get_metadata_optional(ctx, hash), Blob)
    }

    pub async fn exists(ctx: &ServiceContext, hash: &[u8]) -> Result<bool> {
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
        ctx: &ServiceContext,
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
        ctx: &ServiceContext,
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

    pub async fn hard_delete(ctx: &ServiceContext, hash: &[u8]) -> Result<()> {
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
