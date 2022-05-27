/*
 * services/blob/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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
use s3::serde_types::HeadObjectResult;
use std::str;

#[derive(Debug)]
pub struct BlobService;

impl BlobService {
    /// Creates a blob with this data, if it does not already exist.
    pub async fn create(
        ctx: &ServiceContext<'_>,
        data: &[u8],
    ) -> Result<CreateBlobOutput> {
        let bucket = ctx.s3_bucket();
        let hash = sha512_hash(data);
        let hex_hash = hash_to_hex(&hash);

        match Self::head(ctx, &hex_hash).await? {
            // Blob exists, copy metadata and return that
            Some(result) => {
                // Content-Type header should be passed in
                let mime = result.content_type.ok_or(Error::RemoteOperationFailed)?;

                Ok(CreateBlobOutput {
                    hash,
                    mime,
                    created: false,
                })
            }

            // Blob doesn't exist, insert it
            None => {
                // Determine MIME type for the new file
                let mime = mime_type(data.to_vec()).await?;

                // Put into S3
                let (return_data, status) = bucket
                    .put_object_with_content_type(&hex_hash, data, &mime)
                    .await?;

                // We assume all unexpected statuses are errors, even if 1XX or 2XX
                match status {
                    200 => Ok(CreateBlobOutput {
                        hash,
                        mime,
                        created: true,
                    }),
                    _ => s3_error(&return_data, status, "creating S3 blob"),
                }
            }
        }
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        hash: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        let bucket = ctx.s3_bucket();
        let hex_hash = hash_to_hex(hash);
        let (data, status) = bucket.get_object(&hex_hash).await?;

        // TODO read from file_blob table
        // TODO change return to FileBlob type

        match status {
            200 => Ok(Some(data)),
            404 => Ok(None),
            _ => s3_error(&data, status, "fetching S3 blob"),
        }
    }

    #[inline]
    pub async fn get(ctx: &ServiceContext<'_>, hash: &[u8]) -> Result<Vec<u8>> {
        match Self::get_optional(ctx, hash).await? {
            Some(string) => Ok(string),
            None => Err(Error::NotFound),
        }
    }

    pub async fn exists(ctx: &ServiceContext<'_>, hash: &[u8]) -> Result<bool> {
        let hex_hash = hash_to_hex(hash);
        let result = Self::head(ctx, &hex_hash).await?;
        Ok(result.is_some())
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
            _ => s3_error(&[], status, "heading S3 blob"),
        }
    }

    pub async fn hard_delete(ctx: &ServiceContext<'_>, hash: &[u8]) -> Result<()> {
        let bucket = ctx.s3_bucket();
        let hex_hash = hash_to_hex(hash);

        let (_, status) = bucket.delete_object(&hex_hash).await?;

        match status {
            204 => Ok(()),
            _ => s3_error(&[], status, "hard-deleting S3 blob"),
        }
    }
}

/// Helper method to parse out an S3 error response and print the message (if any)
fn s3_error<T>(data: &[u8], status: u16, action: &str) -> Result<T> {
    let error_message = match str::from_utf8(data) {
        Ok(m) if m.is_empty() => "(no content)",
        Ok(m) => m,
        Err(_) => "(invalid UTF-8)",
    };

    tide::log::error!(
        "Error while {} (HTTP {}): {}",
        action,
        status,
        error_message,
    );

    Err(Error::RemoteOperationFailed)
}
