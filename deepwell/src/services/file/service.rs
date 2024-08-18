/*
 * services/file/service.rs
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
    pub async fn upload(
        ctx: &ServiceContext<'_>,
        UploadFile {
            site_id,
            page_id,
            name,
            revision_comments,
            user_id,
            data,
            licensing,
            bypass_filter,
        }: UploadFile,
    ) -> Result<UploadFileOutput> {
        let txn = ctx.seaorm_transaction();

        info!(
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
        let CreateBlobOutput { hash, mime, size } =
            BlobService::create(ctx, &data).await?;

        // Add new file
        let model = file::ActiveModel {
            name: Set(name.clone()),
            site_id: Set(site_id),
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

    /// Edits a file, including the ability to upload a new version.
    pub async fn edit(
        ctx: &ServiceContext<'_>,
        EditFile {
            site_id,
            page_id,
            file_id,
            user_id,
            revision_comments,
            bypass_filter,
            body,
        }: EditFile,
    ) -> Result<Option<EditFileOutput>> {
        info!("Editing file with ID {}", file_id);

        let txn = ctx.seaorm_transaction();
        let last_revision =
            FileRevisionService::get_latest(ctx, site_id, page_id, file_id).await?;

        let EditFileBody {
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
                let CreateBlobOutput { hash, mime, size } =
                    BlobService::create(ctx, &bytes).await?;

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
    pub async fn r#move(
        ctx: &ServiceContext<'_>,
        MoveFile {
            name,
            site_id,
            current_page_id,
            destination_page_id,
            file_id,
            user_id,
            revision_comments,
        }: MoveFile,
    ) -> Result<Option<MoveFileOutput>> {
        let txn = ctx.seaorm_transaction();
        let last_revision =
            FileRevisionService::get_latest(ctx, site_id, current_page_id, file_id)
                .await?;

        // Get destination filename
        let name = name.unwrap_or_else(|| last_revision.name.clone());

        info!(
            "Moving file with ID {} from page ID {} to {}",
            file_id, current_page_id, destination_page_id,
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
    pub async fn delete(
        ctx: &ServiceContext<'_>,
        DeleteFile {
            revision_comments,
            site_id,
            page_id,
            file: reference,
            user_id,
        }: DeleteFile<'_>,
    ) -> Result<DeleteFileOutput> {
        let txn = ctx.seaorm_transaction();

        // Ensure file exists
        let FileModel { file_id, .. } = Self::get(
            ctx,
            GetFile {
                site_id,
                page_id,
                file: reference,
            },
        )
        .await?;

        let last_revision =
            FileRevisionService::get_latest(ctx, site_id, page_id, file_id).await?;

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
    pub async fn restore(
        ctx: &ServiceContext<'_>,
        RestoreFile {
            new_page_id,
            new_name,
            site_id,
            page_id,
            file_id,
            user_id,
            revision_comments,
        }: RestoreFile,
    ) -> Result<RestoreFileOutput> {
        let txn = ctx.seaorm_transaction();
        let file = Self::get_direct(ctx, file_id, true).await?;
        let new_page_id = new_page_id.unwrap_or(page_id);
        let new_name = new_name.unwrap_or(file.name);

        // Do page checks:
        // - Page is correct
        // - File is deleted
        // - Name doesn't already exist

        if file.page_id != page_id {
            warn!("File's page ID and passed page ID do not match");
            return Err(Error::FileNotFound);
        }

        if file.deleted_at.is_none() {
            warn!("File requested to be restored is not currently deleted");
            return Err(Error::FileNotDeleted);
        }

        Self::check_conflicts(ctx, page_id, &new_name, "restore").await?;

        let last_revision =
            FileRevisionService::get_latest(ctx, site_id, page_id, file_id).await?;

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
        GetFile {
            site_id,
            page_id,
            file: reference,
        }: GetFile<'_>,
    ) -> Result<Option<FileModel>> {
        let txn = ctx.seaorm_transaction();
        let file = {
            let condition = match reference {
                Reference::Id(id) => file::Column::FileId.eq(id),
                Reference::Slug(name) => file::Column::Name.eq(name),
            };

            File::find()
                .filter(
                    Condition::all()
                        .add(condition)
                        .add(file::Column::SiteId.eq(site_id))
                        .add(file::Column::PageId.eq(page_id))
                        .add(file::Column::DeletedAt.is_null()),
                )
                .one(txn)
                .await?
        };

        Ok(file)
    }

    #[inline]
    pub async fn get(ctx: &ServiceContext<'_>, input: GetFile<'_>) -> Result<FileModel> {
        find_or_error!(Self::get_optional(ctx, input), File)
    }

    /// Gets the file ID from a reference, looking up if necessary.
    ///
    /// Convenience method since this is much more common than the optional
    /// case, and we don't want to perform a redundant check for site existence
    /// later as part of the actual query.
    pub async fn get_id(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        reference: Reference<'_>,
    ) -> Result<i64> {
        match reference {
            Reference::Id(id) => Ok(id),
            Reference::Slug(name) => {
                let txn = ctx.seaorm_transaction();
                let result: Option<(i64,)> = File::find()
                    .select_only()
                    .column(file::Column::FileId)
                    .filter(
                        Condition::all()
                            .add(file::Column::PageId.eq(page_id))
                            .add(file::Column::Name.eq(name))
                            .add(file::Column::DeletedAt.is_null()),
                    )
                    .into_tuple()
                    .one(txn)
                    .await?;

                match result {
                    Some(tuple) => Ok(tuple.0),
                    None => Err(Error::FileNotFound),
                }
            }
        }
    }

    pub async fn get_direct_optional(
        ctx: &ServiceContext<'_>,
        file_id: i64,
        allow_deleted: bool,
    ) -> Result<Option<FileModel>> {
        let txn = ctx.seaorm_transaction();
        let file = File::find()
            .filter(file::Column::FileId.eq(file_id))
            .one(txn)
            .await?;

        if let Some(ref file) = file {
            if !allow_deleted && file.deleted_at.is_some() {
                // If we're not looking for deleted files, then skip.
                return Ok(None);
            }
        }

        Ok(file)
    }

    #[inline]
    pub async fn get_direct(
        ctx: &ServiceContext<'_>,
        file_id: i64,
        allow_deleted: bool,
    ) -> Result<FileModel> {
        find_or_error!(Self::get_direct_optional(ctx, file_id, allow_deleted), File)
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
    pub async fn hard_delete_all(_ctx: &ServiceContext<'_>, _file_id: i64) -> Result<()> {
        // TODO find hash. update all files with the same hash
        // TODO if hash == 00000 then error
        // TODO add to audit log
        // TODO hard delete BlobService

        todo!()
    }

    /// Checks to see if a file already exists at the name specified.
    ///
    /// If so, this method fails with `Error::FileExists`. Otherwise it returns nothing.
    async fn check_conflicts(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        name: &str,
        action: &str,
    ) -> Result<()> {
        let txn = ctx.seaorm_transaction();

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
                error!(
                    "File {} with name {} already exists on page ID {}, cannot {}",
                    file.file_id, name, page_id, action,
                );

                Err(Error::FileExists)
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
        info!("Checking file data against filters...");

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
