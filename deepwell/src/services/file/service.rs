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
use crate::hash::slice_to_blob_hash;
use crate::models::file::{self, Entity as File, Model as FileModel};
use crate::models::file_revision::{
    self, Entity as FileRevision, Model as FileRevisionModel,
};
use crate::models::sea_orm_active_enums::FileRevisionType;
use crate::services::blob::{FinalizeBlobUploadOutput, EMPTY_BLOB_HASH, EMPTY_BLOB_MIME};
use crate::services::file_revision::{
    CreateFileRevision, CreateFileRevisionBody, CreateFirstFileRevision,
    CreateResurrectionFileRevision, CreateTombstoneFileRevision, FileBlob,
    GetFileRevision,
};
use crate::services::filter::{FilterClass, FilterType};
use crate::services::{BlobService, FileRevisionService, FilterService};
use sea_orm::ActiveValue;

#[derive(Debug)]
pub struct FileService;

impl FileService {
    /// Creates a new file.
    ///
    /// Starts a file upload and tracks it as a distinct file entity.
    ///
    /// In the background, this stores the blob via content addressing,
    /// meaning that duplicates are not uploaded twice.
    pub async fn create(
        ctx: &ServiceContext<'_>,
        CreateFile {
            site_id,
            page_id,
            name,
            uploaded_blob_id,
            revision_comments,
            user_id,
            licensing,
            bypass_filter,
        }: CreateFile,
    ) -> Result<CreateFileOutput> {
        info!("Creating file with name '{}'", name);
        let txn = ctx.transaction();

        // Ensure row consistency
        Self::check_conflicts(ctx, page_id, &name, "create").await?;

        // Perform filter validation
        if !bypass_filter {
            Self::run_filter(ctx, site_id, Some(&name)).await?;
        }

        // Finish blob upload
        let FinalizeBlobUploadOutput {
            hash: s3_hash,
            mime: mime_hint,
            size: size_hint,
            created: blob_created,
        } = BlobService::finish_upload(ctx, user_id, &uploaded_blob_id).await?;

        // Add new file
        let model = file::ActiveModel {
            name: Set(name.clone()),
            site_id: Set(site_id),
            page_id: Set(page_id),
            ..Default::default()
        };
        let file = model.insert(txn).await?;

        FileRevisionService::create_first(
            ctx,
            CreateFirstFileRevision {
                page_id,
                site_id,
                file_id: file.file_id,
                user_id,
                name,
                s3_hash,
                size_hint,
                mime_hint,
                blob_created,
                licensing,
                revision_comments,
            },
        )
        .await
    }

    /// Edits a file, creating a new revision.
    pub async fn edit(
        ctx: &ServiceContext<'_>,
        EditFile {
            site_id,
            page_id,
            file_id,
            user_id,
            last_revision_id,
            revision_comments,
            bypass_filter,
            body,
        }: EditFile,
    ) -> Result<Option<EditFileOutput>> {
        info!("Editing file with ID {}", file_id);

        let txn = ctx.transaction();
        let last_revision =
            FileRevisionService::get_latest(ctx, site_id, page_id, file_id).await?;

        check_last_revision(&last_revision, last_revision_id)?;

        let EditFileBody {
            name,
            licensing,
            uploaded_blob_id,
        } = body;

        let mut new_name = ActiveValue::NotSet;

        // Verify name change
        //
        // If the name isn't changing, then we already verified this
        // when the file was originally created.
        if let Maybe::Set(ref name) = name {
            Self::check_conflicts(ctx, page_id, name, "update").await?;
            new_name = ActiveValue::Set(name.clone());

            if !bypass_filter {
                Self::run_filter(ctx, site_id, Some(name)).await?;
            }
        }

        // If a new file version was uploaded, then finalize.
        //
        // Get the blob struct for conditionally adding to
        // the CreateFileRevisionBody.
        let blob = match uploaded_blob_id {
            Maybe::Unset => Maybe::Unset,
            Maybe::Set(ref id) => {
                let FinalizeBlobUploadOutput {
                    hash: s3_hash,
                    mime: mime_hint,
                    size: size_hint,
                    created: blob_created,
                } = BlobService::finish_upload(ctx, user_id, id).await?;

                Maybe::Set(FileBlob {
                    s3_hash,
                    mime_hint,
                    size_hint,
                    blob_created,
                })
            }
        };

        // Update file metadata
        let model = file::ActiveModel {
            file_id: Set(file_id),
            name: new_name,
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
                revision_comments,
                revision_type: FileRevisionType::Regular,
                body: CreateFileRevisionBody {
                    name,
                    licensing,
                    blob,
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
            last_revision_id,
            revision_comments,
        }: MoveFile,
    ) -> Result<Option<MoveFileOutput>> {
        let txn = ctx.transaction();
        let last_revision =
            FileRevisionService::get_latest(ctx, site_id, current_page_id, file_id)
                .await?;

        check_last_revision(&last_revision, last_revision_id)?;

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
                revision_comments,
                revision_type: FileRevisionType::Move,
                body: CreateFileRevisionBody {
                    page_id: Maybe::Set(destination_page_id),
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
            last_revision_id,
            revision_comments,
            site_id,
            page_id,
            file: reference,
            user_id,
        }: DeleteFile<'_>,
    ) -> Result<DeleteFileOutput> {
        let txn = ctx.transaction();

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

        check_last_revision(&last_revision, last_revision_id)?;

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
        let txn = ctx.transaction();
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

    /// Rolls back a file to be the same as it was in a previous revision.
    /// It changes the file to have the exact state it had in a previous
    /// revision, regardless of any changes since.
    pub async fn rollback(
        ctx: &ServiceContext<'_>,
        RollbackFile {
            site_id,
            page_id,
            file: reference,
            last_revision_id,
            revision_number,
            revision_comments,
            user_id,
            bypass_filter,
        }: RollbackFile<'_>,
    ) -> Result<Option<EditFileOutput>> {
        let txn = ctx.transaction();

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

        // Get target revision and latest revision
        let get_revision_input = GetFileRevision {
            site_id,
            page_id,
            file_id,
            revision_number,
        };

        let (target_revision, last_revision) = try_join!(
            FileRevisionService::get(ctx, get_revision_input),
            FileRevisionService::get_latest(ctx, site_id, page_id, file_id),
        )?;

        // TODO Handle hidden fields, see https://scuttle.atlassian.net/browse/WJ-1285
        let _ = target_revision.hidden;

        // Check last revision ID
        check_last_revision(&last_revision, last_revision_id)?;

        // Extract fields from target revision
        let FileRevisionModel {
            name,
            s3_hash,
            mime_hint,
            size_hint,
            licensing,
            ..
        } = target_revision;

        let mut new_name = ActiveValue::NotSet;

        // Check name change
        if last_revision.name != name {
            Self::check_conflicts(ctx, page_id, &name, "rollback").await?;
            new_name = ActiveValue::Set(name.clone());

            if !bypass_filter {
                Self::run_filter(ctx, site_id, Some(&name)).await?;
            }
        }

        // Create new revision
        //
        // Copy the body of the target revision

        let blob = FileBlob {
            s3_hash: slice_to_blob_hash(&s3_hash),
            mime_hint,
            size_hint,
            // in a rollback, by definition the blob was already uploaded
            blob_created: false,
        };

        let revision_input = CreateFileRevision {
            site_id,
            page_id,
            file_id,
            user_id,
            revision_comments,
            revision_type: FileRevisionType::Rollback,
            body: CreateFileRevisionBody {
                name: Maybe::Set(name),
                blob: Maybe::Set(blob),
                licensing: Maybe::Set(licensing),
                page_id: Maybe::Unset, // rollbacks should never move files
            },
        };

        // Add new file revision
        let revision_output =
            FileRevisionService::create(ctx, revision_input, last_revision).await?;

        // Update file metadata
        let model = file::ActiveModel {
            file_id: Set(file_id),
            name: new_name,
            updated_at: Set(Some(now())),
            ..Default::default()
        };
        model.update(txn).await?;

        Ok(revision_output)
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        GetFile {
            site_id,
            page_id,
            file: reference,
        }: GetFile<'_>,
    ) -> Result<Option<FileModel>> {
        let txn = ctx.transaction();
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
                let txn = ctx.transaction();
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
        let txn = ctx.transaction();
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

/// Verifies that the `last_revision_id` argument is the most recent.
///
/// See the helper function with the same name in `services/page/service.rs`.
fn check_last_revision(
    last_revision_model: &FileRevisionModel,
    arg_last_revision_id: i64,
) -> Result<()> {
    if last_revision_model.revision_id != arg_last_revision_id {
        error!(
            "Latest revision ID in file table is {}, but user argument has ID {}",
            last_revision_model.revision_id, arg_last_revision_id,
        );

        return Err(Error::NotLatestRevisionId);
    }

    Ok(())
}
