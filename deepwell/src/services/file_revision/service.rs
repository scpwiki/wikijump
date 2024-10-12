/*
 * services/file_revision/service.rs
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
use crate::hash::{blob_hash_to_hex, BlobHash};
use crate::models::file_revision::{
    self, Entity as FileRevision, Model as FileRevisionModel,
};
use crate::models::{file, page, site};
use crate::services::blob::{FinalizeBlobUploadOutput, EMPTY_BLOB_HASH, EMPTY_BLOB_MIME};
use crate::services::{BlobService, OutdateService, PageService};
use crate::types::{Bytes, FetchDirection};
use futures::TryStreamExt;
use once_cell::sync::Lazy;
use sea_orm::prelude::*;
use std::num::NonZeroI32;

pub const MAXIMUM_FILE_NAME_LENGTH: usize = 256;

/// The changes for the first revision.
/// The first revision is always considered to have changed everything.
///
/// See `services/page_revision/service.rs`.
static ALL_CHANGES: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        str!("page"),
        str!("name"),
        str!("blob"),
        str!("mime"),
        str!("licensing"),
    ]
});

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

        // Replace with debug_assert_matches! when stablized
        debug_assert!(
            matches!(
                revision_type,
                FileRevisionType::Regular | FileRevisionType::Rollback,
            ),
            "Invalid revision type for standard revision creation",
        );

        // Fields to create in the revision
        let mut changes = Vec::new();
        let mut blob_created = Maybe::Unset;
        let FileRevisionModel {
            mut name,
            mut s3_hash,
            mut mime_hint,
            mut size_hint,
            mut licensing,
            ..
        } = previous;

        // Update fields from input
        //
        // We check the values so that the only listed "changes"
        // are those that actually are different.

        if let Maybe::Set(new_page_id) = body.page_id {
            if page_id != new_page_id {
                changes.push(str!("page"));
                page_id = new_page_id;
            }
        }

        if let Maybe::Set(new_name) = body.name {
            if name != new_name {
                changes.push(str!("name"));
                name = new_name;
            }
        }

        if let Maybe::Set(new_blob) = body.blob {
            if s3_hash != new_blob.s3_hash
                || size_hint != new_blob.size_hint
                || mime_hint != new_blob.mime_hint
            {
                changes.push(str!("blob"));
                s3_hash = new_blob.s3_hash.to_vec();
                size_hint = new_blob.size_hint;
                mime_hint = new_blob.mime_hint;
                blob_created = Maybe::Set(new_blob.blob_created);
            }
        }

        if let Maybe::Set(new_licensing) = body.licensing {
            if licensing != new_licensing {
                changes.push(str!("licensing"));
                licensing = new_licensing;
            }
        }

        // If nothing has changed, then don't create a new revision
        // Also don't rerender the page, this isn't an edit.
        if changes.is_empty() {
            debug!("No changes in file, performing no action");
            return Ok(None);
        }

        // Validate inputs
        if name.is_empty() {
            error!("File name is empty");
            return Err(Error::FileNameEmpty);
        }

        if name.len() >= MAXIMUM_FILE_NAME_LENGTH {
            error!(
                "File name of invalid length: {} > {}",
                name.len(),
                MAXIMUM_FILE_NAME_LENGTH,
            );
            return Err(Error::FileNameTooLong);
        }

        if mime_hint.is_empty() {
            error!("MIME type hint is empty");
            return Err(Error::FileMimeEmpty);
        }

        // TODO validate licensing field

        // Run outdater
        let page_slug = Self::get_page_slug(ctx, site_id, page_id).await?;
        OutdateService::process_page_edit(ctx, site_id, page_id, &page_slug, 0).await?;

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
            size_hint: Set(size_hint),
            mime_hint: Set(mime_hint),
            licensing: Set(licensing),
            changes: Set(changes),
            comments: Set(revision_comments),
            hidden: Set(vec![]),
            ..Default::default()
        };

        let FileRevisionModel { revision_id, .. } = model.insert(txn).await?;
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
            size_hint,
            mime_hint,
            blob_created,
            licensing,
            revision_comments,
        }: CreateFirstFileRevision,
    ) -> Result<CreateFirstFileRevisionOutput> {
        let txn = ctx.transaction();

        // Run outdater
        let page_slug = Self::get_page_slug(ctx, site_id, page_id).await?;
        OutdateService::process_page_displace(ctx, site_id, page_id, &page_slug, 0)
            .await?;

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
            mime_hint: Set(mime_hint),
            size_hint: Set(size_hint),
            licensing: Set(licensing),
            changes: Set(ALL_CHANGES.clone()),
            comments: Set(revision_comments),
            hidden: Set(vec![]),
            ..Default::default()
        };

        let FileRevisionModel { revision_id, .. } = model.insert(txn).await?;
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
            comments,
        }: CreateTombstoneFileRevision,
        previous: FileRevisionModel,
    ) -> Result<CreateFileRevisionOutput> {
        let txn = ctx.transaction();
        let revision_number = next_revision_number(&previous, page_id, file_id);

        let FileRevisionModel {
            name,
            s3_hash,
            mime_hint,
            size_hint,
            licensing,
            ..
        } = previous;

        // Run outdater
        let page_slug = Self::get_page_slug(ctx, site_id, page_id).await?;
        OutdateService::process_page_edit(ctx, site_id, page_id, &page_slug, 0).await?;

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
            mime_hint: Set(mime_hint),
            size_hint: Set(size_hint),
            licensing: Set(licensing),
            changes: Set(vec![]),
            comments: Set(comments),
            hidden: Set(vec![]),
            ..Default::default()
        };

        let FileRevisionModel { revision_id, .. } = model.insert(txn).await?;
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
            comments,
        }: CreateResurrectionFileRevision,
        previous: FileRevisionModel,
    ) -> Result<CreateFileRevisionOutput> {
        let txn = ctx.transaction();
        let revision_number = next_revision_number(&previous, old_page_id, file_id);

        let FileRevisionModel {
            name: old_name,
            s3_hash,
            mime_hint,
            size_hint,
            licensing,
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
        let new_page_slug = Self::get_page_slug(ctx, site_id, new_page_id).await?;
        OutdateService::process_page_edit(ctx, site_id, new_page_id, &new_page_slug, 0)
            .await?;

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
            mime_hint: Set(mime_hint),
            size_hint: Set(size_hint),
            licensing: Set(licensing),
            changes: Set(changes),
            comments: Set(comments),
            hidden: Set(vec![]),
            ..Default::default()
        };

        let FileRevisionModel { revision_id, .. } = model.insert(txn).await?;
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

        let txn = ctx.transaction();
        let latest = Self::get_latest(ctx, site_id, page_id, file_id).await?;
        if revision_id == latest.revision_id {
            warn!("Attempting to edit latest revision, denying request");
            return Err(Error::CannotHideLatestRevision);
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
        let revision = model.update(txn).await?;
        Ok(revision)
    }

    /// Lists information about a blob which is being considered for hard deletion.
    /// This method does not mutate any data.
    pub async fn hard_delete_list(
        ctx: &ServiceContext<'_>,
        s3_hash: BlobHash,
    ) -> Result<HardDeletionStats> {
        const SAMPLE_COUNT: u64 = 10;

        let txn = ctx.transaction();
        let s3_hash = s3_hash.as_slice();

        // Get total count of affected revisions
        let total_revisions = FileRevision::find()
            .select_only()
            .expr(Expr::col(file_revision::Column::RevisionId).count())
            .filter(file_revision::Column::S3Hash.eq(s3_hash))
            .one(txn)
            .await?;

        // Get count of affected files
        let total_files = FileRevision::find()
            .select_only()
            .expr(Expr::col(file_revision::Column::FileId).count_distinct())
            .filter(file_revision::Column::S3Hash.eq(s3_hash))
            .one(txn)
            .await?;

        // Get count of affected pages
        let total_pages = FileRevision::find()
            .select_only()
            .expr(Expr::col(file_revision::Column::PageId).count_distinct())
            .filter(file_revision::Column::S3Hash.eq(s3_hash))
            .one(txn)
            .await?;

        // Get count of affected sites
        let total_sites = FileRevision::find()
            .select_only()
            .expr(Expr::col(file_revision::Column::SiteId).count_distinct())
            .filter(file_revision::Column::S3Hash.eq(s3_hash))
            .one(txn)
            .await?;

        // Get sample filenames
        let sample_files = FileRevision::find()
            .join(JoinType::RightJoin, file_revision::Relation::File.def())
            .select_only()
            .expr(Expr::col(file::Column::Name))
            .filter(file_revision::Column::S3Hash.eq(s3_hash))
            .limit(SAMPLE_COUNT)
            .all(txn)
            .await?;

        // Get sample page slugs
        let sample_pages = FileRevision::find()
            .join(JoinType::RightJoin, file_revision::Relation::Page.def())
            .select_only()
            .expr(Expr::col(page::Column::Slug))
            .filter(file_revision::Column::S3Hash.eq(s3_hash))
            .limit(SAMPLE_COUNT)
            .all(txn)
            .await?;

        // Get sample site slugs
        let sample_sites = FileRevision::find()
            .join(JoinType::RightJoin, file_revision::Relation::Site.def())
            .select_only()
            .expr(Expr::col(site::Column::Slug))
            .filter(file_revision::Column::S3Hash.eq(s3_hash))
            .limit(SAMPLE_COUNT)
            .all(txn)
            .await?;

        todo!()
    }

    /// Hard deletes the specified blob and all duplicates.
    ///
    /// This is a very powerful method and needs to be used carefully.
    /// It should only be accessible to platform staff.
    ///
    /// As opposed to normal soft deletions, this method will completely
    /// remove a file from Wikijump. The data will be entirely purged
    /// and the data will be replaced with the blank file.
    ///
    /// This method should only be used very rarely to clear content such
    /// as severe copyright violations, abuse content, or comply with court orders.
    pub async fn hard_delete_all(
        ctx: &ServiceContext<'_>,
        s3_hash: BlobHash,
        user_id: i64,
    ) -> Result<u64> {
        let txn = ctx.transaction();

        if s3_hash == EMPTY_BLOB_HASH {
            error!("Cannot hard delete the empty blob");
            return Err(Error::BadRequest);
        }

        info!(
            "Hard deleting all blobs matching hash {} (done by user ID {})",
            blob_hash_to_hex(&s3_hash),
            user_id,
        );

        // TODO add to audit log

        // We can't use SeaORM to do an update_many because we have to modify the 'hidden'
        // column to hide the data. But, there's a chance that the column is already blocked,
        // which requires manual adjustment.
        //
        // Instead, we use a stream to get rows and then update each one.

        let mut revisions_affected = 0;
        let mut file_revisions = FileRevision::find()
            .filter(file_revision::Column::S3Hash.eq(EMPTY_BLOB_HASH.to_vec()))
            .stream(txn)
            .await?;

        while let Some(file_revision) = file_revisions.try_next().await? {
            debug!("Updating file revision {}", file_revision.revision_id);

            // Reuse the buffer from the model
            let s3_hash = {
                let mut buffer = file_revision.s3_hash;
                buffer.copy_from_slice(&EMPTY_BLOB_HASH);
                buffer
            };

            // Add 's3_hash' to hidden (deleting the blob data)
            // Then make sure to maintain normal invariants
            let hidden = {
                let mut hidden = file_revision.hidden;
                let field = str!("s3_hash");
                if !hidden.contains(&field) {
                    hidden.push(field);
                }
                hidden.sort();
                hidden
            };

            let model = file_revision::ActiveModel {
                s3_hash: Set(s3_hash),
                hidden: Set(hidden),
                ..Default::default()
            };
            model.update(txn).await?;
            revisions_affected += 1;
        }

        // Delete and blacklist the hash, nobody should be uploading new versions
        try_join!(
            BlobService::add_blacklist(ctx, s3_hash, user_id),
            BlobService::hard_delete(ctx, &s3_hash),
        )?;

        Ok(revisions_affected)
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

        let txn = ctx.transaction();
        let revision = FileRevision::find()
            .filter(
                Condition::all()
                    .add(file_revision::Column::SiteId.eq(site_id))
                    .add(file_revision::Column::PageId.eq(page_id))
                    .add(file_revision::Column::FileId.eq(file_id)),
            )
            .order_by_desc(file_revision::Column::RevisionNumber)
            .one(txn)
            .await?
            .ok_or(Error::FileRevisionNotFound)?;

        Ok(revision)
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
            .await?;

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
        find_or_error!(Self::get_optional(ctx, input), FileRevision)
    }

    /// Counts the number of revisions for a file.
    ///
    /// See `RevisionService::count()`.
    pub async fn count(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        file_id: i64,
    ) -> Result<NonZeroI32> {
        let txn = ctx.transaction();
        let row_count = FileRevision::find()
            .filter(
                Condition::all()
                    .add(file_revision::Column::PageId.eq(page_id))
                    .add(file_revision::Column::FileId.eq(file_id)),
            )
            .count(txn)
            .await?;

        // We store revision_number in INT, which is i32.
        // So even though this row count is usize, it
        // should always fit inside an i32.
        let row_count = i32::try_from(row_count)
            .expect("Revision row count greater than revision_number integer size");

        // All pages have at least one revision, so if there are none
        // that means this page does not exist, and we should return an error.
        match NonZeroI32::new(row_count) {
            Some(count) => Ok(count),
            None => Err(Error::FileNotFound),
        }
    }

    /// Gets a range of revisions for a file.
    ///
    /// See `RevisionService::get_range()`.
    pub async fn get_range(
        ctx: &ServiceContext<'_>,
        GetFileRevisionRange {
            page_id,
            file_id,
            revision_number,
            revision_direction,
            limit,
        }: GetFileRevisionRange,
    ) -> Result<Vec<FileRevisionModel>> {
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
        let revisions = FileRevision::find()
            .filter(
                Condition::all()
                    .add(file_revision::Column::PageId.eq(page_id))
                    .add(file_revision::Column::FileId.eq(file_id))
                    .add(revision_condition),
            )
            .order_by_asc(file_revision::Column::RevisionNumber)
            .limit(limit)
            .all(txn)
            .await?;

        Ok(revisions)
    }

    async fn get_page_slug(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
    ) -> Result<String> {
        let page = PageService::get(ctx, site_id, Reference::Id(page_id)).await?;
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
