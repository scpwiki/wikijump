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
    CreateFileRevision, CreateFileRevisionBody, CreateFirstFileRevision,
    CreateResurrectionFileRevision, CreateTombstoneFileRevision, FileBlob,
};
use crate::services::filter::{FilterClass, FilterType};
use crate::services::{BlobService, FileRevisionService, FilterService};

#[derive(Debug)]
pub struct FileService;

impl FileService {
    /// Uploads a file and tracks it as a separate file entity.
    ///
    /// In the background, this stores the blob via content addressing,
    /// meaning that duplicates are not uploaded twice.
    #[allow(dead_code)] // TEMP
    pub async fn create(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        site_id: i64,
        CreateFile {
            revision_comments,
            name,
            user_id,
            licensing,
            bypass_filter,
        }: CreateFile,
        data: &[u8],
    ) -> Result<CreateFileOutput> {
        let txn = ctx.transaction();

        tide::log::info!(
            "Creating file with name '{}', content length {}",
            name,
            data.len(),
        );

        // Ensure row consistency
        Self::check_conflicts(ctx, page_id, &name, "create").await?;

        // Perform filter validation
        if !bypass_filter {
            Self::run_filter(ctx, site_id, Some(&name)).await?;
        }

        // Upload to S3, get derived metadata
        let CreateBlobOutput {
            hash,
            mime,
            size,
            created: _,
        } = BlobService::create(ctx, data).await?;

        // Add new file
        let model = file::ActiveModel {
            name: Set(name.clone()),
            page_id: Set(page_id),
            ..Default::default()
        };
        let file = model.insert(txn).await?;

        // Add new file revision
        let revision_output = FileRevisionService::create_first(
            ctx,
            CreateFirstFileRevision {
                site_id,
                page_id,
                file_id: file.file_id,
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
    #[allow(dead_code)] // TEMP
    pub async fn update(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        file_id: i64,
        UpdateFile {
            revision_comments,
            user_id,
            body,
            bypass_filter,
        }: UpdateFile,
    ) -> Result<Option<UpdateFileOutput>> {
        let txn = ctx.transaction();
        let last_revision =
            FileRevisionService::get_latest(ctx, page_id, file_id).await?;

        tide::log::info!("Updating file with ID {}", file_id);

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
            Self::check_conflicts(ctx, page_id, name, "update").await?;

            if !bypass_filter {
                Self::run_filter(ctx, site_id, Some(name)).await?;
            }
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
            file_id: Set(file_id),
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
                    name,
                    blob,
                    licensing,
                    ..Default::default()
                },
            },
            last_revision,
        )
        .await?;

        Ok(revision_output)
    }

    /// Moves a file from from one page to another.
    #[allow(dead_code)] // TEMP
    pub async fn r#move(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        file_id: i64,
        input: MoveFile,
    ) -> Result<Option<MoveFileOutput>> {
        let txn = ctx.transaction();

        let MoveFile {
            revision_comments,
            user_id,
            name,
            current_page_id,
            destination_page_id,
        } = input;

        let last_revision =
            FileRevisionService::get_latest(ctx, current_page_id, file_id).await?;

        // Get destination filename
        let name = name.unwrap_or_else(|| last_revision.name.clone());

        tide::log::info!(
            "Moving file with ID {} from page ID {} to {} ",
            file_id,
            current_page_id,
            destination_page_id,
        );

        // Ensure there isn't a file with this name on the destination page
        Self::check_conflicts(ctx, destination_page_id, &name, "move").await?;

        // Update file metadata
        let model = file::ActiveModel {
            file_id: Set(file_id),
            updated_at: Set(Some(now())),
            name: Set(name),
            page_id: Set(destination_page_id),
            ..Default::default()
        };
        model.update(txn).await?;

        // Add new file revision
        let revision_output = FileRevisionService::create(
            ctx,
            CreateFileRevision {
                site_id,
                page_id: current_page_id,
                file_id,
                user_id,
                comments: revision_comments,
                body: CreateFileRevisionBody {
                    page_id: ProvidedValue::Set(destination_page_id),
                    ..Default::default()
                },
            },
            last_revision,
        )
        .await?;

        Ok(revision_output)
    }

    /// Deletes this file.
    ///
    /// Like other deletions throughout Wikijump, this is a soft deletion.
    /// It marks the files as deleted but retains the contents, permitting it
    /// to be easily reverted.
    #[allow(dead_code)] // TEMP
    pub async fn delete(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        reference: CuidReference<'_>,
        input: DeleteFile,
    ) -> Result<DeleteFileOutput> {
        let txn = ctx.transaction();

        let DeleteFile {
            revision_comments,
            site_id,
            user_id,
        } = input;

        // Ensure file exists
        let FileModel { file_id, .. } = Self::get(ctx, page_id, reference).await?;

        let last_revision =
            FileRevisionService::get_latest(ctx, page_id, file_id).await?;

        // Create tombstone revision
        // This outdates the page, etc
        let output = FileRevisionService::create_tombstone(
            ctx,
            CreateTombstoneFileRevision {
                site_id,
                page_id,
                file_id,
                user_id,
                comments: revision_comments,
            },
            last_revision,
        )
        .await?;

        // Set deletion flag
        let model = file::ActiveModel {
            file_id: Set(file_id),
            deleted_at: Set(Some(now())),
            ..Default::default()
        };
        model.update(txn).await?;

        Ok(DeleteFileOutput {
            file_id,
            file_revision_id: output.file_revision_id,
            file_revision_number: output.file_revision_number,
        })
    }

    /// Restores a deleted file.
    ///
    /// This undeletes a file, moving it from the deleted sphere to the specified location.
    #[allow(dead_code)] // TEMP
    pub async fn restore(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        file_id: i64,
        input: RestoreFile,
    ) -> Result<RestoreFileOutput> {
        let txn = ctx.transaction();

        let RestoreFile {
            revision_comments,
            new_page_id,
            new_name,
            site_id,
            user_id,
        } = input;

        let file = Self::get_direct(ctx, file_id).await?;
        let new_page_id = new_page_id.unwrap_or(page_id);
        let new_name = new_name.unwrap_or(file.name);

        // Do page checks:
        // - Page is correct
        // - File is deleted
        // - Name doesn't already exist

        if file.page_id != page_id {
            tide::log::warn!("File's page ID and passed page ID do not match");
            return Err(Error::NotFound);
        }

        if file.deleted_at.is_none() {
            tide::log::warn!("File requested to be restored is not currently deleted");
            return Err(Error::BadRequest);
        }

        Self::check_conflicts(ctx, page_id, &new_name, "restore").await?;

        let last_revision =
            FileRevisionService::get_latest(ctx, page_id, file_id).await?;

        // Create resurrection revision
        // This outdates the page, etc
        let output = FileRevisionService::create_resurrection(
            ctx,
            CreateResurrectionFileRevision {
                site_id,
                page_id,
                file_id,
                user_id,
                new_page_id,
                new_name: new_name.clone(),
                comments: revision_comments,
            },
            last_revision,
        )
        .await?;

        // Set deletion flag
        let model = file::ActiveModel {
            file_id: Set(file_id),
            deleted_at: Set(None),
            ..Default::default()
        };
        model.update(txn).await?;

        Ok(RestoreFileOutput {
            page_id,
            file_id,
            name: new_name,
            file_revision_id: output.file_revision_id,
            file_revision_number: output.file_revision_number,
        })
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        reference: CuidReference<'_>,
    ) -> Result<Option<FileModel>> {
        let txn = ctx.transaction();
        let file = {
            let condition = match reference {
                CuidReference::Id(id) => file::Column::FileId.eq(id),
                CuidReference::Name(name) => file::Column::Name.eq(name),
            };

            File::find()
                .filter(
                    Condition::all()
                        .add(condition)
                        .add(file::Column::PageId.eq(page_id)),
                )
                .one(txn)
                .await?
        };

        Ok(file)
    }

    #[inline]
    pub async fn get(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        reference: CuidReference<'_>,
    ) -> Result<FileModel> {
        find_or_error(Self::get_optional(ctx, page_id, reference)).await
    }

    pub async fn get_direct_optional(
        ctx: &ServiceContext<'_>,
        file_id: i64,
    ) -> Result<Option<FileModel>> {
        let txn = ctx.transaction();
        let file = File::find()
            .filter(file::Column::FileId.eq(file_id))
            .one(txn)
            .await?;

        Ok(file)
    }

    #[inline]
    pub async fn get_direct(ctx: &ServiceContext<'_>, file_id: i64) -> Result<FileModel> {
        find_or_error(Self::get_direct_optional(ctx, file_id)).await
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
    #[allow(dead_code)] // TEMP
    pub async fn hard_delete_all(_ctx: &ServiceContext<'_>, _file_id: i64) -> Result<()> {
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
        action: &str,
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
                    "File {} with name {} already exists on page ID {}, cannot {}",
                    file.file_id,
                    name,
                    page_id,
                    action,
                );

                Err(Error::Conflict)
            }
        }
    }

    /// This runs the regular expression-based text filters against a file's name.
    ///
    /// It does not check the file's contents, as that is a binary blob.
    /// Such a hash filter would need to be implemented through a separate system.
    async fn run_filter(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        name: Option<&str>,
    ) -> Result<()> {
        tide::log::info!("Checking file data against filters...");

        let filter_matcher = FilterService::get_matcher(
            ctx,
            FilterClass::PlatformAndSite(site_id),
            FilterType::Forum,
        )
        .await?;

        if let Some(name) = name {
            filter_matcher.verify(ctx, name).await?;
        }

        Ok(())
    }
}
