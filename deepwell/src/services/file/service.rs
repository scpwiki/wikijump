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
use crate::services::blob::CreateBlobOutput;
use crate::services::file_revision::{
    CreateFileRevision, CreateFileRevisionBody, CreateFirstFileRevision, FileBlob,
};
use crate::services::{BlobService, FileRevisionService};

#[derive(Debug)]
pub struct FileService;

impl FileService {
    /// Uploads a file and tracks it as a separate file entity.
    ///
    /// In the background, this stores the blob via content addressing,
    /// meaning that duplicates are not uploaded twice.
    pub async fn create(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        CreateFile {
            revision_comments,
            name,
            site_id,
            user_id,
            licensing,
        }: CreateFile,
        data: &[u8],
    ) -> Result<CreateFileOutput> {
        let txn = ctx.transaction();

        tide::log::info!(
            "Creating file with name '{}', content length {}",
            name,
            data.len(),
        );

        Self::check_conflicts(ctx, page_id, &name).await?;

        // Upload to S3, get derived metadata
        let CreateBlobOutput {
            hash,
            mime,
            size,
            created: _,
        } = BlobService::create(ctx, data).await?;

        // Add new file
        let file_id = ctx.cuid()?;

        let model = file::ActiveModel {
            file_id: Set(file_id.clone()),
            name: Set(name.clone()),
            page_id: Set(page_id),
            ..Default::default()
        };
        model.insert(txn).await?;

        // Add new file revision
        let revision_output = FileRevisionService::create_first(
            ctx,
            CreateFirstFileRevision {
                site_id,
                page_id,
                file_id: file_id.clone(),
                user_id,
                name,
                s3_hash: hash,
                size_hint: size,
                mime_hint: mime,
                licensing,
                comments: revision_comments,
            },
        )
        .await?;

        Ok(revision_output)
    }

    /// Updates a file, including the ability to upload a new version.
    pub async fn update(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        file_id: String,
        UpdateFile {
            revision_comments,
            user_id,
            body,
        }: UpdateFile,
    ) -> Result<Option<UpdateFileOutput>> {
        let txn = ctx.transaction();
        let previous = FileRevisionService::get_latest(ctx, page_id, &file_id).await?;

        tide::log::info!("Updating file with ID '{}'", file_id);

        // Process inputs

        let UpdateFileBody {
            name,
            data,
            licensing,
        } = body;

        // Verify name change
        //
        // If the name isn't changing, then we already verified this
        // when the file was originally created.
        if let ProvidedValue::Set(ref name) = name {
            Self::check_conflicts(ctx, page_id, name).await?;
        }

        // Upload to S3, get derived metadata
        let blob = match data {
            ProvidedValue::Unset => ProvidedValue::Unset,
            ProvidedValue::Set(bytes) => {
                let CreateBlobOutput {
                    hash,
                    mime,
                    size,
                    created: _,
                } = BlobService::create(ctx, &bytes).await?;

                ProvidedValue::Set(FileBlob {
                    s3_hash: hash,
                    size_hint: size,
                    mime_hint: mime,
                })
            }
        };

        // Make database changes

        // Update file metadata
        let model = file::ActiveModel {
            file_id: Set(file_id.clone()),
            updated_at: Set(Some(now())),
            ..Default::default()
        };
        model.update(txn).await?;

        // Add new file revision
        let revision_output = FileRevisionService::create(
            ctx,
            CreateFileRevision {
                site_id,
                page_id,
                file_id,
                user_id,
                comments: revision_comments,
                body: CreateFileRevisionBody {
                    page_id: ProvidedValue::Unset, // For moving to a different page
                    name,
                    blob,
                    licensing,
                },
            },
            previous,
        )
        .await?;

        Ok(revision_output)
    }

    /// Moves a file from from one page to another.
    ///
    /// Note: This is called `rename` and not `move` because
    ///       the latter is a reserved word in Rust.
    pub async fn rename(
        ctx: &ServiceContext<'_>,
        page_id: String,
        input: MoveFile,
    ) -> Result<FileModel> {
        let txn = ctx.transaction();

        let MoveFile {
            revision_comments,
            user_id,
            current_page_id,
            new_page_id,
        } = input;

        todo!()
    }

    /// Deletes this file.
    ///
    /// Like other deletions throughout Wikijump, this is a soft deletion.
    /// It marks the files as deleted but retains the contents, permitting it
    /// to be easily reverted.
    pub async fn delete(
        ctx: &ServiceContext<'_>,
        file_id: String,
        input: DeleteFile,
    ) -> Result<FileModel> {
        let txn = ctx.transaction();

        let DeleteFile {
            revision_comments,
            site_id,
            page_id,
            user_id,
        } = input;

        // Ensure file exists
        if !Self::exists(ctx, &file_id).await? {
            return Err(Error::NotFound);
        }

        // Set deletion flag
        let model = file::ActiveModel {
            file_id: Set(file_id.clone()),
            deleted_at: Set(Some(now())),
            ..Default::default()
        };
        let file = model.update(txn).await?;

        // Add new file revision
        // TODO

        Ok(file)
    }

    // TODO
    /// Restores a deleted file.
    ///
    /// This undeletes a file, moving it from the deleted sphere to the specified location.
    #[allow(dead_code)]
    pub async fn restore(_ctx: &ServiceContext<'_>, _file_id: String) -> Result<()> {
        todo!()
    }

    /// Gets an uploaded file that has been, including its contents if requested.
    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        file_id: &str,
        blob: bool,
    ) -> Result<Option<GetFileOutput>> {
        todo!()
    }

    /// Gets an uploaded file, failing if it does not exists.
    pub async fn get(
        ctx: &ServiceContext<'_>,
        file_id: &str,
        blob: bool,
    ) -> Result<GetFileOutput> {
        match Self::get_optional(ctx, file_id, blob).await? {
            Some(file) => Ok(file),
            None => Err(Error::NotFound),
        }
    }

    pub async fn exists(ctx: &ServiceContext<'_>, file_id: &str) -> Result<bool> {
        Self::get_optional(ctx, file_id, false)
            .await
            .map(|file| file.is_some())
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

    /// Checks to see if a file already exists at the name specified.
    ///
    /// If so, this method fails with `Error::Conflict`. Otherwise it returns nothing.
    async fn check_conflicts(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        name: &str,
    ) -> Result<()> {
        let txn = ctx.transaction();

        let result = File::find()
            .filter(
                Condition::all()
                    .add(file::Column::Name.eq(name))
                    .add(file::Column::PageId.eq(page_id))
                    .add(file::Column::DeletedAt.is_null()),
            )
            .one(txn)
            .await?;

        match result {
            None => Ok(()),
            Some(file) => {
                tide::log::error!(
                    "File {} with name '{}' already exists on page ID {}, cannot create",
                    file.file_id,
                    name,
                    page_id,
                );

                Err(Error::Conflict)
            }
        }
    }
}
