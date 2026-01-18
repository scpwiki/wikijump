/*
 * services/file/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
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
use crate::services::blob::{EMPTY_BLOB_HASH, EMPTY_BLOB_MIME, FinalizeBlobUploadOutput};
use crate::services::file_revision::{
    CreateFileRevision, CreateFileRevisionBody, CreateFirstFileRevision,
    CreateResurrectionFileRevision, CreateTombstoneFileRevision, FileBlob,
    GetFileRevision,
};
use crate::services::filter::{FilterClass, FilterType};
use crate::services::{BlobService, FileRevisionService, FilterService, PageService};
use crate::types::FileOrder;
use crate::utils::trim_spaces_in_place;
use sea_orm::ActiveValue;

pub const MAXIMUM_FILE_NAME_LENGTH: usize = 256;

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
            mut name,
            uploaded_blob_id,
            direct_upload,
            revision_comments,
            user_id,
            bypass_filter,
        }: CreateFile,
    ) -> Result<CreateFileOutput> {
        info!("Creating file with name '{name}'");
        let txn = ctx.transaction();

        // Verify filename is valid
        check_file_name(&mut name)?;

        // Ensure row consistency
        Self::check_conflicts(ctx, page_id, &name, "create").await?;

        // Perform filter validation
        if !bypass_filter {
            Self::run_filter(ctx, site_id, Some(&name)).await?;
        }

        // Finish blob upload
        let FinalizeBlobUploadOutput {
            s3_hash,
            mime,
            size,
            created: blob_created,
        } = match direct_upload {
            None => {
                // Normal path, finish upload of blob from user
                BlobService::finish_upload(ctx, user_id, &uploaded_blob_id).await?
            }
            Some(data) => {
                // Special path, used only internally to directly upload a blob,
                // for instance in the seeder
                //
                // This should always be None when called from API users
                BlobService::direct_upload(ctx, data).await?
            }
        };

        // Add new file
        let model = file::ActiveModel {
            name: Set(name.clone()),
            site_id: Set(site_id.0),
            page_id: Set(page_id.0),
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
                size,
                mime,
                blob_created,
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
        info!("Editing file with ID {file_id}");

        let txn = ctx.transaction();
        let last_revision =
            FileRevisionService::get_latest(ctx, site_id, page_id, file_id).await?;

        check_last_revision(&last_revision, last_revision_id)?;

        let EditFileBody {
            mut name,
            uploaded_blob_id,
            direct_upload,
        } = body;

        let mut new_name = ActiveValue::NotSet;

        // Verify name change
        //
        // If the name isn't changing, then we already verified this
        // when the file was originally created.
        if let Maybe::Set(ref mut name) = name {
            check_file_name(name)?;
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
                    s3_hash,
                    mime,
                    size,
                    created: blob_created,
                } = match direct_upload {
                    Maybe::Unset => {
                        // Normal path, finish upload of blob from user
                        BlobService::finish_upload(ctx, user_id, id).await?
                    }
                    Maybe::Set(data) => {
                        // Special path, used only internally to directly upload a blob
                        // See FileService::create()
                        BlobService::direct_upload(ctx, data).await?
                    }
                };

                Maybe::Set(FileBlob {
                    s3_hash,
                    mime,
                    size,
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
            destination_page,
            file_id,
            user_id,
            last_revision_id,
            revision_comments,
        }: MoveFile<'_>,
    ) -> Result<Option<MoveFileOutput>> {
        let txn = ctx.transaction();
        let last_revision =
            FileRevisionService::get_latest(ctx, site_id, current_page_id, file_id)
                .await?;

        check_last_revision(&last_revision, last_revision_id)?;

        // Get destination page id
        let destination_page_id =
            PageService::get_id(ctx, site_id, destination_page).await?;

        // Get destination filename
        let mut name = name.unwrap_or_else(|| last_revision.name.clone());

        info!(
            "Moving file with ID {file_id} from page ID {current_page_id} to {destination_page_id}"
        );

        // Verify filename is valid
        check_file_name(&mut name)?;

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
        input: DeleteFile<'_>,
    ) -> Result<DeleteFileOutput> {
        Self::delete_inner(ctx, input, false).await
    }

    /// Deletes this file, erasing its S3 hash in the tombstone revision.
    ///
    /// This is used as part of the hard deletion implementation, in the step
    /// prior to erasing and hiding the S3 hash in all affected files.
    pub async fn delete_with_erased_s3_hash(
        ctx: &ServiceContext<'_>,
        input: DeleteFile<'_>,
    ) -> Result<DeleteFileOutput> {
        Self::delete_inner(ctx, input, true).await
    }

    /// Performs a file deletion.
    ///
    /// Contains a flag for determining if the S3 hash of the file being deleted should be wiped,
    /// as part of the hard deletion implementation.
    async fn delete_inner(
        ctx: &ServiceContext<'_>,
        DeleteFile {
            last_revision_id,
            revision_comments,
            site_id,
            page_id,
            file: reference,
            user_id,
        }: DeleteFile<'_>,
        erase_s3_hash: bool,
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
                revision_comments,
                erase_s3_hash,
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
            new_page,
            new_name,
            site_id,
            page_id,
            file_id,
            user_id,
            revision_comments,
        }: RestoreFile<'_>,
    ) -> Result<RestoreFileOutput> {
        let txn = ctx.transaction();
        let file = Self::get_direct(ctx, file_id, true).await?;
        let new_page_id =
            PageService::get_id(ctx, site_id, new_page.unwrap_or(Reference::Id(page_id)))
                .await?;
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
                revision_comments,
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

        // Check last revision ID
        check_last_revision(&last_revision, last_revision_id)?;

        // Extract fields from target revision
        let FileRevisionModel {
            name,
            s3_hash,
            mime,
            size,
            hidden,
            ..
        } = target_revision;

        let hide_name = hidden.iter().any(|field| field == "name");
        let hide_s3_hash = hidden.iter().any(|field| field == "s3_hash");
        let hide_mime = hidden.iter().any(|field| field == "mime");
        let hide_size = hidden.iter().any(|field| field == "size");

        let mut new_name = ActiveValue::NotSet;

        // Check name change
        if !hide_name && last_revision.name != name {
            Self::check_conflicts(ctx, page_id, &name, "rollback").await?;
            new_name = ActiveValue::Set(name.clone());

            if !bypass_filter {
                Self::run_filter(ctx, site_id, Some(&name)).await?;
            }
        }

        // Create new revision
        //
        // Copy the body of the target revision

        let blob = if hide_s3_hash || hide_mime || hide_size {
            Maybe::Unset
        } else {
            Maybe::Set(FileBlob {
                s3_hash: slice_to_blob_hash(&s3_hash),
                mime,
                size,
                // in a rollback, by definition the blob was already uploaded
                blob_created: false,
            })
        };

        let name_body = if hide_name {
            Maybe::Unset
        } else {
            Maybe::Set(name)
        };

        let revision_input = CreateFileRevision {
            site_id,
            page_id,
            file_id,
            user_id,
            revision_comments,
            revision_type: FileRevisionType::Rollback,
            body: CreateFileRevisionBody {
                name: name_body,
                blob,
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

    /// Gets all files on a page, with potential conditions.
    ///
    /// The `deleted` argument:
    /// * If it is `Some(true)`, then it only returns pages which have been deleted.
    /// * If it is `Some(false)`, then it only returns pages which are extant.
    /// * If it is `None`, then it returns all pages regardless of deletion status.
    // TODO add pagination
    pub async fn get_all(
        ctx: &ServiceContext<'_>,
        site_id: SiteId,
        page_id: PageId,
        deleted: Option<bool>,
        order: FileOrder,
    ) -> Result<Vec<FileModel>> {
        let txn = ctx.transaction();
        let deleted_condition = match deleted {
            Some(true) => Some(file::Column::DeletedAt.is_not_null()),
            Some(false) => Some(file::Column::DeletedAt.is_null()),
            None => None,
        };

        let files = File::find()
            .filter(
                Condition::all()
                    .add(file::Column::SiteId.eq(site_id))
                    .add(file::Column::PageId.eq(page_id))
                    .add_option(deleted_condition),
            )
            .order_by(order.column.into_column(), order.direction)
            .all(txn)
            .await?;

        Ok(files)
    }

    /// Gets the file ID from a reference, looking up if necessary.
    ///
    /// Convenience method since this is much more common than the optional
    /// case, and we don't want to perform a redundant check for site existence
    /// later as part of the actual query.
    pub async fn get_id(
        ctx: &ServiceContext<'_>,
        page_id: PageId,
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

        if let Some(ref file) = file
            && !allow_deleted
            && file.deleted_at.is_some()
        {
            // If we're not looking for deleted files, then skip.
            return Ok(None);
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

    /// Checks to see if a file already exists at the name specified.
    ///
    /// If so, this method fails with `Error::FileExists`. Otherwise it returns nothing.
    async fn check_conflicts(
        ctx: &ServiceContext<'_>,
        page_id: PageId,
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
        site_id: SiteId,
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

/// Verifies that this filename is valid.
///
/// This helper function is generally read-only, but if
/// it finds a name which has leading or trailing whitespace,
/// then it trims that off in-place.
fn check_file_name(name: &mut String) -> Result<()> {
    // Removes leading or trailing whitespace
    trim_spaces_in_place(name);
    debug!("Trimmed file name: '{name}'");

    // Disallow empty filenames
    if name.is_empty() {
        error!("File name is empty");
        return Err(Error::FileNameEmpty);
    }

    // Limit filename length
    if name.len() >= MAXIMUM_FILE_NAME_LENGTH {
        error!(
            "File name of invalid length: {} > {}",
            name.len(),
            MAXIMUM_FILE_NAME_LENGTH,
        );
        return Err(Error::FileNameTooLong {
            length: name.len(),
            maximum: MAXIMUM_FILE_NAME_LENGTH,
        });
    }

    // Makes sure there aren't any control characters or slashes.
    //
    // Rust considers null bytes, newlines, tabs and the various unprintables to be 'control'.
    // See https://doc.rust-lang.org/stable/std/primitive.char.html#method.is_control
    if name
        .chars()
        .any(|c| c.is_control() || c == '/' || c == '\\')
    {
        error!("File name contains control characters or slashes");
        return Err(Error::FileNameInvalidCharacters);
    }

    // Looks good
    Ok(())
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
