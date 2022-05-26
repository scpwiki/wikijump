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
use crate::models::file::{self, Entity as File, Model as FileModel};

#[derive(Debug)]
pub struct FileService;

impl FileService {
    /// Uploads a file and tracks it as a separate file entity.
    ///
    /// In the background, this stores the blob via content addressing,
    /// meaning that duplicates are not uploaded twice.
    pub async fn create(ctx: &ServiceContext<'_>, data: &[u8]) -> Result<()> {
        // TODO insert into file

        todo!()
    }

    /// Updates metadata associated with this file.
    pub async fn update(ctx: &ServiceContext<'_>, file_id: &str) -> Result<()> {
        // TODO update file, updated_at

        todo!()
    }

    /// Deletes this file.
    ///
    /// Like other deletions throughout Wikijump, this is a soft deletion.
    /// It marks the files as deleted but retains the contents, permitting it
    /// to be easily reverted.
    pub async fn delete(ctx: &ServiceContext<'_>, file_id: &str) -> Result<()> {
        // TODO update deleted_at in file

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

    /// Hard deletes this file and all duplicates.
    ///
    /// This is a very powerful method and needs to be used carefully.
    /// It should only be accessible to platform staff.
    ///
    /// As opposed to normal soft deletions, this method will completely
    /// remove a file from Wikijump. The file rows will be deleted themselves,
    /// and will cascade to any places where file IDs are used.
    ///
    /// This method should only be used very rarely to clear content such
    /// as severe copyright violations, abuse content, or comply with court orders.
    pub async fn hard_delete_all(ctx: &ServiceContext<'_>, file_id: &str) -> Result<()> {
        // TODO find hash. update all files with the same hash
        // TODO add to audit log
        // TODO hard delete BlobService

        todo!()
    }
}
