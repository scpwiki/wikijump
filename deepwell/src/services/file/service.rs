/*
 * services/file/service.rs
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
use crate::hash::{hash_to_hex, sha512_hash, Hash};
use std::str;

#[derive(Debug)]
pub struct FileService;

impl FileService {
    /// Uploads a file and tracks it as a separate file entity.
    ///
    /// In the background, this stores the blob via content addressing,
    /// meaning that duplicates are not uploaded twice.
    pub async fn create(ctx: &ServiceContext<'_>) -> Result<()> {
        todo!()
    }

    /// Gets an uploaded file that has been, including its contents if requested.
    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        file_id: &str,
        blob: bool,
    ) -> Result<Option<()>> {
        todo!()
    }

    /// Gets an uploaded file, failing if it does not exists.
    pub async fn get(ctx: &ServiceContext<'_>, file_id: &str, blob: bool) -> Result<()> {
        match Self::get_optional(ctx, file_id, blob).await? {
            Some(file) => Ok(file),
            None => Err(Error::NotFound),
        }
    }

    /// Creates a blob with this data, if it does not already exist.
    pub async fn create_blob(ctx: &ServiceContext<'_>, data: &[u8]) -> Result<Hash> {
        let bucket = ctx.s3_bucket();
        let hash = sha512_hash(data);
        let hex_hash = hash_to_hex(&hash);

        // TODO insert into file_blob table

        let (return_data, status) = bucket.put_object(&hex_hash, data).await?;

        // We assume all unexpected statuses are errors, even if 1XX or 2XX
        match status {
            200 => Ok(hash),
            _ => s3_error(&return_data, status, "creating S3 blob"),
        }
    }

    pub async fn get_blob_optional(
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

    pub async fn blob_exists(ctx: &ServiceContext<'_>, hash: &[u8]) -> Result<bool> {
        let bucket = ctx.s3_bucket();
        let hex_hash = hash_to_hex(hash);
        let (data, status) = bucket.get_object(&hex_hash).await?;

        // TODO check from file_blob table, don't call S3

        match status {
            200 | 204 => Ok(true),
            404 => Ok(false),
            _ => s3_error(&data, status, "fetching S3 blob"),
        }
    }

    #[inline]
    pub async fn get_blob(ctx: &ServiceContext<'_>, hash: &[u8]) -> Result<Vec<u8>> {
        match Self::get_blob_optional(ctx, hash).await? {
            Some(string) => Ok(string),
            None => Err(Error::NotFound),
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
