/*
 * services/file_revision/service.rs
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
use crate::hash::{BlobHash, blob_hash_to_hex, slice_to_blob_hash};
use crate::models::file_revision::{
    self, Entity as FileRevision, Model as FileRevisionModel,
};
use crate::models::{file, page, site};
use crate::services::blob::{EMPTY_BLOB_HASH, EMPTY_BLOB_MIME, FinalizeBlobUploadOutput};
use crate::services::{BlobService, OutdateService, PageService};
use crate::types::{Bytes, FetchDirection, RerenderDepth};
use sea_orm::FromQueryResult;
use sea_orm::prelude::*;
use std::num::NonZeroI32;
use std::sync::LazyLock;

/// The changes for the first revision.
/// The first revision is always considered to have changed everything.
///
/// See `services/page_revision/service.rs`.
static ALL_CHANGES: LazyLock<Vec<String>> =
    LazyLock::new(|| vec![str!("page"), str!("name"), str!("blob"), str!("mime")]);

#[derive(Debug)]
pub struct FileRevisionService;

impl FileRevisionService {
    /// Creates a new revision on an existing file.
    ///
    /// See `RevisionService::create()`.
    ///
    /// # Panics
    /// If the given previous revision is for a different file or page, this method will panic.
    pub async fn create(
        ctx: &ServiceContext<'_>,
        CreateFileRevision {
            site_id,
            mut page_id,
            file_id,
            user_id,
            revision_comments,
            revision_type,
            body,
        }: CreateFileRevision,
        previous: FileRevisionModel,
    ) -> Result<Option<CreateFileRevisionOutput>> {
        let txn = ctx.transaction();
        let revision_number = next_revision_number(&previous, page_id, file_id);

        let original_page_id = page_id;
        let make_error = || {
            Error::new(
                format!(
                    "failed to create new file revision on file ID {} on page ID {} in site ID {} by user ID {}",
                    file_id, original_page_id, site_id, user_id,
                ),
                ErrorType::FileRevision,
            )
        };

        // Replace with debug_assert_matches! when stablized.
        // This should correspond to each use of FileRevisionService::create() in FileService.
        debug_assert!(
            matches!(
                revision_type,
                FileRevisionType::Regular
                    | FileRevisionType::Move
                    | FileRevisionType::Rollback,
            ),
            "Invalid revision type for standard revision creation",
        );

        // Fields to create in the revision
        let mut changes = Vec::new();
        let mut blob_created = Maybe::Unset;
        let FileRevisionModel {
            mut name,
            mut s3_hash,
            mut mime,
            mut size,
            ..
        } = previous;

        // Update fields from input
        //
        // We check the values so that the only listed "changes"
        // are those that actually are different.

        if let Maybe::Set(new_page_id) = body.page_id
            && page_id != new_page_id
        {
            changes.push(str!("page"));
            page_id = new_page_id;
        }

        if let Maybe::Set(new_name) = body.name
            && name != new_name
        {
            changes.push(str!("name"));
            name = new_name;
        }

        if let Maybe::Set(new_blob) = body.blob
            && (s3_hash != new_blob.s3_hash
                || size != new_blob.size
                || mime != new_blob.mime)
        {
            changes.push(str!("blob"));
            s3_hash = new_blob.s3_hash.to_vec();
            size = new_blob.size;
            mime = new_blob.mime;
            blob_created = Maybe::Set(new_blob.blob_created);
        }

        // If nothing has changed, then don't create a new revision
        // Also don't rerender the page, this isn't an edit.
        if changes.is_empty() {
            debug!("No changes in file, performing no action");
            return Ok(None);
        }

        // Validate inputs
        // (Note that filename checks are done in FileService)

        if mime.is_empty() {
            error!("MIME type is empty");
            bail!(Error::new(
                "cannot create file revision, no MIME type specified",
                ErrorType::FileMimeEmpty
            ));
        }

        // Run outdater
        let page_slug = Self::get_page_slug(ctx, site_id, page_id)
            .await
            .or_raise(make_error)?;

        OutdateService::process_page_edit(
            ctx,
            site_id,
            page_id,
            &page_slug,
            RerenderDepth::default(),
        )
        .await
        .or_raise(make_error)?;

        // Insert the new revision into the table
        let model = file_revision::ActiveModel {
            revision_type: Set(revision_type),
            revision_number: Set(revision_number),
            file_id: Set(file_id),
            page_id: Set(page_id),
            site_id: Set(site_id),
            user_id: Set(user_id),
            name: Set(name),
            s3_hash: Set(s3_hash.to_vec()),
            size: Set(size),
            mime: Set(mime),
            changes: Set(changes),
            comments: Set(revision_comments),
            hidden: Set(vec![]),
            ..Default::default()
        };

        let FileRevisionModel { revision_id, .. } =
            model.insert(txn).await.or_raise(make_error)?;
        Ok(Some(CreateFileRevisionOutput {
            file_revision_id: revision_id,
            file_revision_number: revision_number,
            blob_created,
        }))
    }

    /// Creates the first revision for an already-uploaded file.
    ///
    /// See `RevisionService::create_first()`.
    pub async fn create_first(
        ctx: &ServiceContext<'_>,
        CreateFirstFileRevision {
            page_id,
            site_id,
            file_id,
            user_id,
            name,
            s3_hash,
            size,
            mime,
            blob_created,
            revision_comments,
        }: CreateFirstFileRevision,
    ) -> Result<CreateFirstFileRevisionOutput> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!(
                    "failed to create first file revision on file ID {} on page ID {} in site ID {} by user ID {}",
                    file_id, page_id, site_id, user_id,
                ),
                ErrorType::FileRevision,
            )
        };

        // Run outdater
        let page_slug = Self::get_page_slug(ctx, site_id, page_id)
            .await
            .or_raise(make_error)?;

        OutdateService::process_page_displace(
            ctx,
            site_id,
            page_id,
            &page_slug,
            RerenderDepth::default(),
        )
        .await
        .or_raise(make_error)?;

        // Insert the first revision into the table
        let model = file_revision::ActiveModel {
            revision_type: Set(FileRevisionType::Create),
            revision_number: Set(0),
            file_id: Set(file_id),
            page_id: Set(page_id),
            site_id: Set(site_id),
            user_id: Set(user_id),
            name: Set(name),
            s3_hash: Set(s3_hash.to_vec()),
            mime: Set(mime),
            size: Set(size),
            changes: Set(ALL_CHANGES.clone()),
            comments: Set(revision_comments),
            hidden: Set(vec![]),
            ..Default::default()
        };

        let FileRevisionModel { revision_id, .. } =
            model.insert(txn).await.or_raise(make_error)?;

        Ok(CreateFirstFileRevisionOutput {
            file_id,
            file_revision_id: revision_id,
            blob_created,
        })
    }

    /// Creates a revision marking a page as deleted.
    ///
    /// This revision is called a "tombstone" in that
    /// its only purpose is to mark that the file has been deleted.
    ///
    /// See `RevisionService::create_tombstone()`.
    ///
    /// # Panics
    /// If the given previous revision is for a different file or page, this method will panic.
    pub async fn create_tombstone(
        ctx: &ServiceContext<'_>,
        CreateTombstoneFileRevision {
            site_id,
            page_id,
            file_id,
            user_id,
            revision_comments,
            erase_s3_hash,
        }: CreateTombstoneFileRevision,
        previous: FileRevisionModel,
    ) -> Result<CreateFileRevisionOutput> {
        let txn = ctx.transaction();
        let revision_number = next_revision_number(&previous, page_id, file_id);

        let make_error = || {
            Error::new(
                format!(
                    "failed to create tombstone file revision on file ID {} on page ID {} in site ID {} by user ID {} (erase S3 hash {})",
                    file_id, page_id, site_id, user_id, erase_s3_hash,
                ),
                ErrorType::FileRevision,
            )
        };

        let mut hidden = Vec::new();
        let FileRevisionModel {
            name,
            mut s3_hash,
            mime,
            size,
            ..
        } = previous;

        if erase_s3_hash {
            // Replace S3 hash for tombstone revision with empty data
            s3_hash.copy_from_slice(&EMPTY_BLOB_HASH);

            // Also block the s3_hash column for this revision
            hidden.push(str!("s3_hash"));
        }

        // Run outdater
        let page_slug = Self::get_page_slug(ctx, site_id, page_id)
            .await
            .or_raise(make_error)?;

        OutdateService::process_page_edit(
            ctx,
            site_id,
            page_id,
            &page_slug,
            RerenderDepth::default(),
        )
        .await
        .or_raise(make_error)?;

        // Insert the tombstone revision into the table
        let model = file_revision::ActiveModel {
            revision_type: Set(FileRevisionType::Delete),
            revision_number: Set(revision_number),
            file_id: Set(file_id),
            page_id: Set(page_id),
            site_id: Set(site_id),
            user_id: Set(user_id),
            name: Set(name),
            s3_hash: Set(s3_hash),
            mime: Set(mime),
            size: Set(size),
            changes: Set(vec![]),
            comments: Set(revision_comments),
            hidden: Set(hidden),
            ..Default::default()
        };

        let FileRevisionModel { revision_id, .. } =
            model.insert(txn).await.or_raise(make_error)?;

        Ok(CreateFileRevisionOutput {
            file_revision_id: revision_id,
            file_revision_number: revision_number,
            blob_created: Maybe::Unset,
        })
    }

    /// Creates a revision marking a pages as restored (i.e., undeleted).
    ///
    /// Similar to `create_tombstone`, this method creates
    /// a revision whose only purpose is to mark that the page
    /// has been restored.
    ///
    /// Note that page parenting information is removed during deletion
    /// and is not restored here.
    ///
    /// Remember that, like `create_first()`, this method assumes
    /// the caller has already verified that undeleting the page here
    /// will not cause conflicts.
    ///
    /// See `RevisionService::create_tombstone()`.
    ///
    /// # Panics
    /// If the given previous revision is for a different file or page, this method will panic.
    pub async fn create_resurrection(
        ctx: &ServiceContext<'_>,
        CreateResurrectionFileRevision {
            site_id,
            page_id: old_page_id,
            file_id,
            user_id,
            new_page_id,
            new_name,
            revision_comments,
        }: CreateResurrectionFileRevision,
        previous: FileRevisionModel,
    ) -> Result<CreateFileRevisionOutput> {
        let txn = ctx.transaction();
        let revision_number = next_revision_number(&previous, old_page_id, file_id);

        let make_error = || {
            Error::new(
                format!(
                    "failed to create resurrection file revision on file ID {} on old page ID {} in site ID {} to new page ID {} by user ID {}",
                    file_id, old_page_id, site_id, new_page_id, user_id,
                ),
                ErrorType::FileRevision,
            )
        };

        let FileRevisionModel {
            name: old_name,
            s3_hash,
            mime,
            size,
            ..
        } = previous;

        let changes = {
            let mut changes = vec![];

            if old_page_id != new_page_id {
                changes.push(str!("page"));
            }

            if old_name != new_name {
                changes.push(str!("name"));
            }

            changes
        };

        // Run outdater
        let new_page_slug = Self::get_page_slug(ctx, site_id, new_page_id)
            .await
            .or_raise(make_error)?;

        OutdateService::process_page_edit(
            ctx,
            site_id,
            new_page_id,
            &new_page_slug,
            RerenderDepth::default(),
        )
        .await
        .or_raise(make_error)?;

        // Insert the resurrection revision into the table
        let model = file_revision::ActiveModel {
            revision_type: Set(FileRevisionType::Undelete),
            revision_number: Set(revision_number),
            file_id: Set(file_id),
            page_id: Set(new_page_id),
            site_id: Set(site_id),
            user_id: Set(user_id),
            name: Set(new_name),
            s3_hash: Set(s3_hash),
            mime: Set(mime),
            size: Set(size),
            changes: Set(changes),
            comments: Set(revision_comments),
            hidden: Set(vec![]),
            ..Default::default()
        };

        let FileRevisionModel { revision_id, .. } =
            model.insert(txn).await.or_raise(make_error)?;

        Ok(CreateFileRevisionOutput {
            file_revision_id: revision_id,
            file_revision_number: revision_number,
            blob_created: Maybe::Unset,
        })
    }

    /// Modifies an existing file revision.
    ///
    /// Revisions are immutable entries in an append-only log.
    /// However, the `hidden` column can be updated to "delete"
    /// revisions (wholly or partially) to cover spam and abuse.
    pub async fn update(
        ctx: &ServiceContext<'_>,
        UpdateFileRevision {
            site_id,
            page_id,
            file_id,
            revision_id,
            user_id,
            hidden,
        }: UpdateFileRevision,
    ) -> Result<FileRevisionModel> {
        // The latest file revision cannot be hidden, because
        // the file, its name, contents, etc are exposed.
        // It should be reverted first, and then it can be hidden.

        let make_error = || {
            Error::new(
                format!(
                    "failed to update file revision ID {} on file ID {} on page ID {} in site ID {} by user ID {}",
                    revision_id, file_id, page_id, site_id, user_id,
                ),
                ErrorType::FileRevision,
            )
        };

        let txn = ctx.transaction();
        let latest = Self::get_latest(ctx, site_id, page_id, file_id)
            .await
            .or_raise(make_error)?;

        if revision_id == latest.revision_id {
            warn!("Attempting to edit latest revision, denying request");
            bail!(Error::new(
                "cannot edit latest file revision",
                ErrorType::CannotHideLatestRevision,
            ));
        }

        // TODO: record revision edit in audit log
        let _ = user_id;

        // Update the revision

        let model = file_revision::ActiveModel {
            revision_id: Set(revision_id),
            hidden: Set(hidden),
            ..Default::default()
        };

        // Update and return
        let revision = model.update(txn).await.or_raise(make_error)?;
        Ok(revision)
    }

    /// Get the latest revision for this file.
    ///
    /// See `RevisionService::get_latest()`.
    pub async fn get_latest(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        file_id: i64,
    ) -> Result<FileRevisionModel> {
        // NOTE: There is no optional variant of this method,
        //       since all extant files must have at least one revision.

        let make_error = || {
            Error::new(
                format!(
                    "failed to get latest file revision for file ID {} on page ID {} in site ID {}",
                    file_id, page_id, site_id,
                ),
                ErrorType::FileRevision,
            )
        };

        let txn = ctx.transaction();
        let revision_opt = FileRevision::find()
            .filter(
                Condition::all()
                    .add(file_revision::Column::SiteId.eq(site_id))
                    .add(file_revision::Column::PageId.eq(page_id))
                    .add(file_revision::Column::FileId.eq(file_id)),
            )
            .order_by_desc(file_revision::Column::RevisionNumber)
            .one(txn)
            .await
            .or_raise(make_error)?;

        match revision_opt {
            Some(revision) => Ok(revision),
            None => bail!(Error::new(
                format!(
                    "no latest file revision for file ID {} on page ID {} in site ID {}",
                    file_id, page_id, site_id,
                ),
                ErrorType::FileRevisionNotFound,
            )),
        }
    }

    /// Get the given revision for a file.
    ///
    /// See `RevisionService::get_optional()`.
    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        GetFileRevision {
            site_id,
            page_id,
            file_id,
            revision_number,
        }: GetFileRevision,
    ) -> Result<Option<FileRevisionModel>> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to get file revision number {} on file ID {} in page ID {} on site ID {}",
                    revision_number, file_id, page_id, site_id,
                ),
                ErrorType::FileRevision,
            )
        };

        let txn = ctx.transaction();
        let revision = FileRevision::find()
            .filter(
                Condition::all()
                    .add(file_revision::Column::SiteId.eq(site_id))
                    .add(file_revision::Column::PageId.eq(page_id))
                    .add(file_revision::Column::FileId.eq(file_id))
                    .add(file_revision::Column::RevisionNumber.eq(revision_number)),
            )
            .one(txn)
            .await
            .or_raise(make_error)?;

        Ok(revision)
    }

    /// Gets the given revision for a file, failing if it doesn't exist.
    ///
    /// See `RevisionService::get()`.
    #[inline]
    #[allow(dead_code)]
    pub async fn get(
        ctx: &ServiceContext<'_>,
        input: GetFileRevision,
    ) -> Result<FileRevisionModel> {
        find_or_error!(
            Self::get_optional(ctx, input),
            "file revision",
            FileRevision
        )
    }

    /// Counts the number of revisions for a file.
    ///
    /// See `RevisionService::count()`.
    pub async fn count(
        ctx: &ServiceContext<'_>,
        CountFileRevisions {
            site_id,
            page_id,
            file_id,
        }: CountFileRevisions,
    ) -> Result<NonZeroI32> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to get file revision count on file ID {} in page ID {} on site ID {}",
                    file_id, page_id, site_id,
                ),
                ErrorType::FileRevision,
            )
        };

        let txn = ctx.transaction();
        let row_count = FileRevision::find()
            .filter(
                Condition::all()
                    .add(file_revision::Column::SiteId.eq(site_id))
                    .add(file_revision::Column::PageId.eq(page_id))
                    .add(file_revision::Column::FileId.eq(file_id)),
            )
            .count(txn)
            .await
            .or_raise(make_error)?;

        // We store revision_number in INT, which is i32.
        // So even though this row count is usize, it
        // should always fit inside an i32.
        let row_count = i32::try_from(row_count)
            .expect("Revision row count greater than revision_number integer size");

        // All pages have at least one revision, so if there are none
        // that means this page does not exist, and we should return an error.
        match NonZeroI32::new(row_count) {
            Some(count) => Ok(count),
            None => bail!(Error::new(
                format!(
                    "cannot get file revision count for file ID {} in page ID {} on site ID {}",
                    file_id, page_id, site_id,
                ),
                ErrorType::FileNotFound
            )),
        }
    }

    /// Gets a range of revisions for a file.
    ///
    /// See `RevisionService::get_range()`.
    pub async fn get_range(
        ctx: &ServiceContext<'_>,
        GetFileRevisionRange {
            file_id,
            revision_number,
            revision_direction,
            limit,
        }: GetFileRevisionRange,
    ) -> Result<Vec<FileRevisionModel>> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to get {} file revisions from number {} in file ID {} (max {})",
                    revision_direction.name(),
                    revision_number,
                    file_id,
                    limit,
                ),
                ErrorType::FileRevision,
            )
        };

        let revision_condition = {
            use file_revision::Column::RevisionNumber;

            // Allow specifying "-1" to mean "the most recent revision",
            // otherwise keep as-is.
            let revision_number = if revision_number >= 0 {
                revision_number
            } else {
                i32::MAX
            };

            // Get correct database condition based on requested ordering
            match revision_direction {
                FetchDirection::Before => RevisionNumber.lte(revision_number),
                FetchDirection::After => RevisionNumber.gte(revision_number),
            }
        };

        let txn = ctx.transaction();
        let mut query = FileRevision::find().filter(
            Condition::all()
                .add(file_revision::Column::FileId.eq(file_id))
                .add(revision_condition),
        );

        query = match revision_direction {
            FetchDirection::Before => {
                query.order_by_desc(file_revision::Column::RevisionNumber)
            }
            FetchDirection::After => {
                query.order_by_asc(file_revision::Column::RevisionNumber)
            }
        };

        let revisions = query.limit(limit).all(txn).await.or_raise(make_error)?;

        Ok(revisions)
    }

    async fn get_page_slug(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
    ) -> Result<String> {
        let page = PageService::get(ctx, site_id, Reference::Id(page_id))
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to get page slug for page ID {} in site ID {}",
                        page_id, site_id,
                    ),
                    ErrorType::FileRevision,
                )
            })?;

        Ok(page.slug)
    }
}

fn next_revision_number(previous: &FileRevisionModel, page_id: i64, file_id: i64) -> i32 {
    // Check for basic consistency
    assert_eq!(
        previous.file_id, file_id,
        "Previous revision has an inconsistent file ID",
    );
    assert_eq!(
        previous.page_id, page_id,
        "Previous revision has an inconsistent page ID",
    );

    // Get the new revision number
    previous.revision_number + 1
}
