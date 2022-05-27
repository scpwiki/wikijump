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
use crate::models::sea_orm_active_enums::RevisionType;
use crate::services::blob::CreateBlobOutput;
use crate::services::revision::CreateFileRevision;
use crate::services::{BlobService, RevisionService};

#[derive(Debug)]
pub struct FileService;

impl FileService {
    /// Uploads a file and tracks it as a separate file entity.
    ///
    /// In the background, this stores the blob via content addressing,
    /// meaning that duplicates are not uploaded twice.
    pub async fn create(
        ctx: &ServiceContext<'_>,
        input: CreateFile,
        data: &[u8],
    ) -> Result<CreateFileOutput> {
        let txn = ctx.transaction();

        tide::log::info!(
            "Creating file with name '{}', content length {}",
            input.name,
            data.len(),
        );

        let CreateFile {
            revision_comments,
            name,
            site_id,
            page_id,
            user_id,
            licensing,
        } = input;

        // Check file doesn't already exist
        {
            let result = File::find()
                .filter(
                    Condition::all()
                        .add(file::Column::Name.eq(name.as_str()))
                        .add(file::Column::PageId.eq(page_id))
                        .add(file::Column::DeletedAt.is_null()),
                )
                .one(txn)
                .await?;

            if let Some(file) = result {
                tide::log::error!(
                    "File {} with name '{}' already exists on page ID {}, cannot create",
                    file.file_id,
                    name,
                    page_id,
                );

                return Err(Error::Conflict);
            }
        }

        // Upload to S3, get derived metadata
        let CreateBlobOutput { hash, mime, .. } = BlobService::create(ctx, data).await?;

        // Insert into database
        let file_id = ctx.cuid()?;
        let size_hint: i64 = data.len().try_into().expect("Buffer size exceeds i64");

        let model = file::ActiveModel {
            file_id: Set(file_id.clone()),
            name: Set(name),
            s3_hash: Set(Some(hash.to_vec())),
            user_id: Set(user_id),
            page_id: Set(page_id),
            size_hint: Set(size_hint),
            mime_hint: Set(mime),
            licensing: Set(licensing),
            ..Default::default()
        };
        let file = model.insert(txn).await?;

        // Add new page revision
        let previous = RevisionService::get_latest(ctx, site_id, page_id).await?;
        let revision = RevisionService::create_file_revision(
            ctx,
            CreateFileRevision {
                site_id,
                page_id,
                user_id,
                file_id,
                file_change: RevisionType::FileCreate,
                comments: revision_comments,
            },
            previous,
        )
        .await?;

        Ok(CreateFileOutput { file, revision })
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
