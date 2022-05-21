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
use crate::hash::{sha512_hash, Hash};

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
        let hash = sha512_hash(data);

        todo!()
    }

    pub async fn get_blob_optional(
        ctx: &ServiceContext<'_>,
        hash: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        todo!()
    }

    pub async fn blob_exists(ctx: &ServiceContext<'_>, hash: &[u8]) -> Result<bool> {
        // TODO
        // *NOT* get_blob_optional.is_some(), we make a separate S3 call for HEAD
        todo!()
    }

    #[inline]
    pub async fn get_blob(ctx: &ServiceContext<'_>, hash: &[u8]) -> Result<Vec<u8>> {
        match Self::get_blob_optional(ctx, hash).await? {
            Some(string) => Ok(string),
            None => Err(Error::NotFound),
        }
    }
}

// TODO
