/*
 * services/blob/service.rs
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

use super::structs::{
    BlobMetadata, FinalizeBlobUploadOutput, HardDelete, HardDeleteOutput,
    StartBlobUpload, StartBlobUploadOutput,
};
use crate::constants::{ADMIN_USER_ID, SYSTEM_USER_ID};
use crate::error::prelude::{Error, ErrorType, OptionExt, Result, ResultExt};
use crate::hash::slice_to_blob_hash;
use crate::hash::{BlobHash, blob_hash_to_hex, sha512_hash};
use crate::models::blob_blacklist::{self, Entity as BlobBlacklist};
use crate::models::blob_pending::{
    self, Entity as BlobPending, Model as BlobPendingModel,
};
use crate::models::file_revision::{self, Entity as FileRevision};
use crate::models::user::{self, Entity as User};
use crate::services::ServiceContext;
use crate::services::file::{DeleteFile, FileService};
use crate::utils::assert_is_csprng;
use crate::utils::{ConvertToI64, ConvertToU64, ConvertToUsize, now};
use bytes::Bytes;
use cuid2::cuid;
use paste::paste;
use rand::distr::{Alphanumeric, SampleString};
use s3::request::request_trait::ResponseData;
use s3::serde_types::HeadObjectResult;
use sea_orm::prelude::Value;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
    Set,
};
use sea_orm::{DatabaseBackend, FromQueryResult, Statement, TransactionTrait};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::str;
use std::sync::Arc;
use time::format_description::well_known::Rfc2822;
use time::{Duration, OffsetDateTime};

/// How many samples to provide when providing hard deletion stats.
const SAMPLE_COUNT: u16 = 10;

/// Hash for empty blobs.
///
/// Even though it is not the SHA-512 hash, for simplicity we treat the hash
/// value with all zeroes to be the blob address for the empty blob.
/// This empty blob is not actually stored in S3 but instead is a "virtual blob",
/// considered to have always been present in `BlobService`.
pub const EMPTY_BLOB_HASH: BlobHash = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
];

/// MIME type for empty blobs.
pub const EMPTY_BLOB_MIME: &str = "inode/x-empty; charset=binary";

/// Created UNIX timestamp for empty blobs.
///
/// Timestamp is 2019/01/18 at midnight, the date of the first Wikijump commit.
pub const EMPTY_BLOB_TIMESTAMP: i64 = 1547769600;

/// The subdirectory in the S3 bucket where all pending uploads are kept.
pub const PRESIGN_DIRECTORY: &str = "uploads";

#[derive(Debug)]
pub struct BlobService;

impl BlobService {
    // File-related operations

    /// Creates an S3 presign URL to allow an end user to upload a blob.
    /// This is the start to the upload process for any kind of file.
    ///
    /// # Returns
    /// The generated presign URL, which can be uploaded to.
    pub async fn start_upload(
        ctx: &ServiceContext<'_>,
        StartBlobUpload { user_id, blob_size }: StartBlobUpload,
    ) -> Result<StartBlobUploadOutput> {
        info!("Creating upload by {user_id} with promised length {blob_size}");
        let config = ctx.config();
        let txn = ctx.transaction();

        // Convert expected length integer type, then check it
        let blob_size = i64::try_from(blob_size).or_raise(|| Error::new(
            format!(
                "failed to create pending blob upload, size integer could not be converted: {}",
                blob_size,
            ),
            ErrorType::BlobTooBig,
        ))?;

        if blob_size > config.maximum_blob_size {
            error!(
                "Blob proposed to upload is too big ({} > {})",
                blob_size, config.maximum_blob_size,
            );
            bail!(Error::new(
                format!(
                    "failed to create pending blob upload, proposed file size is too large: ({} > {} bytes)",
                    blob_size, config.maximum_blob_size,
                ),
                ErrorType::BlobTooBig,
            ));
        }

        let make_error =
            || Error::new("failed to create pending blob upload", ErrorType::Blob);

        // Generate primary key and random S3 path
        let pending_blob_id = cuid();
        let s3_path = {
            let mut path = format!("{PRESIGN_DIRECTORY}/");

            {
                let mut rng = rand::rng();
                assert_is_csprng(&rng);
                Alphanumeric.append_string(
                    &mut rng,
                    &mut path,
                    config.presigned_path_length,
                );
            }

            path
        };

        info!(
            "Creating presign upload URL for blob at path {} with primary key {}",
            s3_path, pending_blob_id,
        );

        // Create presign URL
        let bucket = ctx.s3_files_bucket();
        let presign_url = bucket
            .presign_put(&s3_path, config.presigned_expiry_secs, None, None)
            .await
            .or_raise(make_error)?;

        // Get timestamps
        let created_at = now();
        let expires_at = created_at
            .checked_add(Duration::seconds(i64::from(config.presigned_expiry_secs)))
            .ok_or_raise(make_error)?;

        // Add pending blob entry
        let model = blob_pending::ActiveModel {
            external_id: Set(pending_blob_id),
            expected_length: Set(blob_size),
            s3_path: Set(s3_path),
            presign_url: Set(presign_url),
            created_by: Set(user_id),
            created_at: Set(created_at),
            expires_at: Set(expires_at),
            ..Default::default()
        };

        let BlobPendingModel {
            external_id: pending_blob_id,
            presign_url,
            ..
        } = model.insert(txn).await.or_raise(make_error)?;

        debug!("New presign upload URL will last until {expires_at}");

        Ok(StartBlobUploadOutput {
            pending_blob_id,
            presign_url,
            expires_at,
        })
    }

    async fn get_pending_blob_path(
        ctx: &ServiceContext<'_>,
        user_id: i64,
        pending_blob_id: &str,
    ) -> Result<PendingBlob> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!(
                    "failed to get pending blob path for ID '{}' (from user ID {})",
                    pending_blob_id, user_id,
                ),
                ErrorType::Blob,
            )
        };

        let row = BlobPending::find_by_id(pending_blob_id)
            .one(txn)
            .await
            .or_raise(make_error)?;

        let BlobPendingModel {
            s3_path,
            s3_hash,
            created_by,
            expected_length,
            ..
        } = match row {
            Some(pending) => pending,
            None => bail!(Error::new("blob does not exist", ErrorType::BlobNotFound)),
        };

        if user_id != created_by {
            error!(
                "User mismatch, user ID {} is attempting to use blob uploaded by {}",
                user_id, created_by,
            );
            bail!(Error::new(
                format!(
                    "failed to get blob, user mismatch (user ID {} is requested, but uploaded by user ID {}",
                    user_id, created_by,
                ),
                ErrorType::BlobWrongUser,
            ));
        }

        Ok(PendingBlob {
            s3_path,
            expected_length,
            moved_hash: s3_hash,
        })
    }

    pub async fn cancel_upload(
        ctx: &ServiceContext<'_>,
        user_id: i64,
        pending_blob_id: &str,
    ) -> Result<()> {
        info!("Cancelling upload for blob for pending ID {pending_blob_id}");
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!(
                    "failed to cancel pending blob for ID '{}' (from user ID {})",
                    pending_blob_id, user_id,
                ),
                ErrorType::Blob,
            )
        };

        let PendingBlob { s3_path, .. } =
            Self::get_pending_blob_path(ctx, user_id, pending_blob_id)
                .await
                .or_raise(make_error)?;

        BlobPending::delete_by_id(pending_blob_id)
            .exec(txn)
            .await
            .or_raise(make_error)?;

        if Self::head(ctx, &s3_path)
            .await
            .or_raise(make_error)?
            .is_some()
        {
            let bucket = ctx.s3_files_bucket();
            bucket.delete_object(&s3_path).await.or_raise(make_error)?;
        }

        Ok(())
    }

    /// Helper function to do the actual "move" step of blob finalization.
    /// This is where, after uploading to the presign URL, the S3 object is
    /// then moved to its permanent location with a hashed name.
    // NOTE: Because S3 changes cannot be rolled back on error, we are
    //       creating a separate transaction here so that `blob_pending`
    //       changes are persistent even if the outer request fails.
    async fn move_uploaded(
        ctx: &ServiceContext<'_>,
        pending_blob_id: &str,
        pending_blob_user_id: i64,
        s3_path: &str,
        expected_length: usize,
    ) -> Result<FinalizeBlobUploadOutput> {
        let state = ctx.state();
        let db_state = Arc::clone(&state);

        let make_error =
            || Error::new("failed to move uploaded blob to final", ErrorType::Blob);

        // Produce temporary context in a new transaction
        let txn = db_state.database.begin().await.or_raise(make_error)?;

        let inner_ctx = ServiceContext::new(&state, &txn);
        let result = Self::move_uploaded_inner(
            &inner_ctx,
            pending_blob_id,
            pending_blob_user_id,
            s3_path,
            expected_length,
        )
        .await;

        // Commit separate transaction, recording a move (if it occurred)
        txn.commit().await.or_raise(make_error)?;
        result
    }

    async fn move_uploaded_inner(
        ctx: &ServiceContext<'_>,
        pending_blob_id: &str,
        pending_blob_user_id: i64,
        s3_path: &str,
        expected_length: usize,
    ) -> Result<FinalizeBlobUploadOutput> {
        let bucket = ctx.s3_files_bucket();
        let txn = ctx.transaction();

        let make_error =
            || Error::new("failed to move uploaded blob to final", ErrorType::Blob);

        debug!("Download uploaded blob from S3 uploads to get metadata");
        let response = bucket.get_object(s3_path).await.or_raise(make_error)?;

        let data: Vec<u8> = match response.status_code() {
            200 => response.into(),
            404 => {
                error!("No blob uploaded at presign path {s3_path}");
                bail!(Error::new(
                    format!("no blob uploaded at presign path {}", s3_path),
                    ErrorType::BlobNotUploaded,
                ));
            }
            _ => {
                error!("Unable to retrieve uploaded blob at {s3_path} from S3");
                bail!(s3_error(&response, "finalizing uploaded blob"));
            }
        };

        if expected_length != data.len() {
            error!(
                "Expected blob length of {} bytes, instead found {} uploaded. Deleting pending.",
                expected_length,
                data.len(),
            );
            bucket.delete_object(&s3_path).await.or_raise(make_error)?;
            bail!(Error::new(
                format!(
                    "expected blob length of {} bytes, instead found a blob of {} bytes uploaded",
                    expected_length,
                    data.len(),
                ),
                ErrorType::BlobSizeMismatch {
                    expected: expected_length,
                    actual: data.len()
                },
            ));
        }

        // Special handling for empty blobs
        if data.is_empty() {
            debug!("File being created is empty, special case");
            return Ok(FinalizeBlobUploadOutput {
                s3_hash: EMPTY_BLOB_HASH,
                mime: str!(EMPTY_BLOB_MIME),
                size: 0,
                created: false,
            });
        }

        debug!("Updating blob metadata in database and S3");

        // If the blob exists, then just delete the uploaded one.
        //
        // If it doesn't, then we need to move it. However, within S3
        // we cannot "move" objects, we have to upload and delete the original.
        //
        // In either case, we delete the blob at the temporary upload location.

        let result = Self::direct_upload(ctx, data).await.or_raise(make_error)?;

        bucket.delete_object(&s3_path).await.or_raise(make_error)?;

        // Check that new blob is not blacklisted
        if Self::on_blacklist(ctx, result.s3_hash)
            .await
            .or_raise(make_error)?
        {
            let hex_hash = blob_hash_to_hex(&result.s3_hash);
            error!(
                "Newly-uploaded blob {} is blacklisted (hash {})",
                pending_blob_id, hex_hash,
            );

            // Cancel this pending upload, what they're trying to store shouldn't be on here
            Self::cancel_upload(ctx, pending_blob_user_id, pending_blob_id)
                .await
                .or_raise(make_error)?;

            // Finally, return error
            bail!(Error::new(
                "cannot upload blob, contents are blacklisted",
                ErrorType::BlobBlacklisted(result.s3_hash)
            ));
        }

        // Update pending blob with hash
        let model = blob_pending::ActiveModel {
            external_id: Set(str!(pending_blob_id)),
            s3_hash: Set(Some(result.s3_hash.to_vec())),
            ..Default::default()
        };
        model.update(txn).await.or_raise(make_error)?;

        // Return
        Ok(result)
    }

    /// Takes a blob and uploads it to its final destination in S3.
    ///
    /// This is used in the above `move_uploaded_inner()` method to
    /// "move" the S3 blob. This is done by uploading to the final
    /// destination, then afterwards, deleting the blob at the temporary
    /// upload location.
    pub(crate) async fn direct_upload(
        ctx: &ServiceContext<'_>,
        data: Vec<u8>,
    ) -> Result<FinalizeBlobUploadOutput> {
        let bucket = ctx.s3_files_bucket();

        let make_error =
            || Error::new("failed to perform direct upload", ErrorType::Blob);

        // Get hash for blob
        let s3_hash = sha512_hash(&data);
        let hex_hash = blob_hash_to_hex(&s3_hash);

        // Convert size to correct integer type
        let size = data.len().try_into_i64().or_raise(make_error)?;

        match Self::head(ctx, &hex_hash).await.or_raise(make_error)? {
            Some(result) => {
                debug!("Blob with hash {hex_hash} already exists");

                // TODO: Should we ever update the mime type?
                //       In case of changing file formats, etc.

                // Content-Type header should be returned
                let mime = result.content_type.ok_or_raise(|| {
                    Error::new(
                        format!("blob with hash {} already exists in S3", hex_hash),
                        ErrorType::BlobBackend,
                    )
                })?;

                Ok(FinalizeBlobUploadOutput {
                    s3_hash,
                    mime,
                    size,
                    created: false,
                })
            }
            None => {
                debug!("Blob with hash {hex_hash} to be created");

                // Determine MIME type for the new blob
                let mime = ctx
                    .mime()
                    .get_mime_type(data.clone())
                    .await
                    .or_raise(make_error)?;

                // Upload S3 object
                let response = bucket
                    .put_object_with_content_type(&hex_hash, &data, &mime)
                    .await
                    .or_raise(make_error)?;

                // We assume all unexpected statuses are errors, even if 1XX or 2XX
                match response.status_code() {
                    200 => Ok(FinalizeBlobUploadOutput {
                        s3_hash,
                        mime,
                        size,
                        created: true,
                    }),
                    _ => bail!(s3_error(&response, "creating finalized S3 blob")),
                }
            }
        }
    }

    pub async fn finish_upload(
        ctx: &ServiceContext<'_>,
        user_id: i64,
        pending_blob_id: &str,
    ) -> Result<FinalizeBlobUploadOutput> {
        info!("Finishing upload for blob for pending ID {pending_blob_id}");

        let make_error = || {
            Error::new(
                format!("failed to finalize blob upload for '{}'", pending_blob_id),
                ErrorType::Blob,
            )
        };

        let PendingBlob {
            s3_path,
            expected_length,
            moved_hash,
        } = Self::get_pending_blob_path(ctx, user_id, pending_blob_id)
            .await
            .or_raise(make_error)?;

        let output = match moved_hash {
            // Need to move from pending to main hash area
            None => {
                let expected_length =
                    expected_length.try_into_usize().or_raise(make_error)?;

                Self::move_uploaded(
                    ctx,
                    pending_blob_id,
                    user_id,
                    &s3_path,
                    expected_length,
                )
                .await
                .or_raise(make_error)?
            }

            // Already moved
            Some(hash_vec) => {
                let BlobMetadata { mime, size, .. } = Self::get_metadata(ctx, &hash_vec)
                    .await
                    .or_raise(make_error)?;

                debug_assert_eq!(expected_length, size);

                FinalizeBlobUploadOutput {
                    s3_hash: slice_to_blob_hash(&hash_vec),
                    mime,
                    size,
                    created: false,
                }
            }
        };

        // Return result based on blob status
        Ok(output)
    }

    // Prune operations

    /// Deletes all expired pending blobs from the database and S3.
    pub async fn prune(ctx: &ServiceContext<'_>) -> Result<()> {
        let txn = ctx.transaction();
        let bucket = ctx.s3_files_bucket();
        info!("Pruning expired pending blobs from database and S3");

        let make_error =
            || Error::new("failed to prune expired pending blobs", ErrorType::Blob);

        // Fetch all expired pending blobs
        let pending_blobs = BlobPending::find()
            .select_only()
            .column(blob_pending::Column::ExternalId)
            .column(blob_pending::Column::S3Path)
            .filter(blob_pending::Column::ExpiresAt.lte(now()))
            .into_tuple::<(String, String)>()
            .all(txn)
            .await
            .or_raise(make_error)?;

        // Delete from the S3 bucket
        for (_, s3_path) in &pending_blobs {
            // Only try to delete if the object exists,
            // ignore missing objects.
            if Self::exists(ctx, s3_path).await.or_raise(make_error)? {
                bucket.delete_object(&s3_path).await.or_raise(make_error)?;
            }
        }

        // Delete from the database
        let blob_ids = pending_blobs.into_iter().map(|(id, _)| id);

        BlobPending::delete_many()
            .filter(blob_pending::Column::ExternalId.is_in(blob_ids))
            .exec(txn)
            .await
            .or_raise(make_error)?;

        Ok(())
    }

    // Hard-deletion operations

    /// Does a dry run on a blob hard deletion, showing what would have been changed.
    /// This method does not mutate any data.
    pub async fn hard_delete_preview(
        ctx: &ServiceContext<'_>,
        s3_hash: &[u8],
    ) -> Result<HardDeleteOutput> {
        Self::require_platform_staff(ctx, "preview blob hard deletion")?;
        let s3_hash = slice_to_blob_hash(s3_hash);

        Self::hard_delete_inner(ctx, HardDeleteInner::DryRun { s3_hash })
            .await
            .or_raise(|| Error::new("failed to preview hard deletion", ErrorType::Blob))
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
        HardDelete { s3_hash }: HardDelete,
    ) -> Result<HardDeleteOutput> {
        let user_id = Self::require_platform_staff(ctx, "confirm blob hard deletion")?;
        let s3_hash = slice_to_blob_hash(s3_hash.as_ref());

        Self::hard_delete_inner(ctx, HardDeleteInner::Commit { s3_hash, user_id })
            .await
            .or_raise(|| Error::new(
                format!(
                    "failed to hard delete all instances of a blob, performed by user ID {}",
                    user_id,
                ),
                ErrorType::Blob,
            ))
    }

    fn require_platform_staff(ctx: &ServiceContext<'_>, action: &str) -> Result<i64> {
        let user_id = ctx.request().user_id().or_raise(|| {
            Error::new(
                format!("{action} requires an admin request context"),
                ErrorType::PermissionDenied,
            )
        })?;

        if user_id != ADMIN_USER_ID {
            bail!(Error::new(
                format!("{action} requires an admin request context"),
                ErrorType::PermissionDenied,
            ));
        }

        Ok(user_id)
    }

    /// Inner implementation, which runs the hard deletion procedure but may not actually delete.
    ///
    /// By running the actual deletion system, we can verify that the predicted set to delete is in
    /// fact what will be deleted in a run.
    async fn hard_delete_inner(
        ctx: &ServiceContext<'_>,
        input: HardDeleteInner,
    ) -> Result<HardDeleteOutput> {
        let txn = ctx.transaction();
        let (s3_hash, deleter_user_id) = match input {
            HardDeleteInner::Commit { s3_hash, user_id } => (s3_hash, Some(user_id)),
            HardDeleteInner::DryRun { s3_hash } => (s3_hash, None),
        };
        // NOTE: Instead of an explicit "dry_run" variable, the value of "is real run"
        //       or "is dry run" is derived from whether "deleter_user_id" is None or not.
        //       If there's no user ID to record as responsible, it must necessarily
        //       be a dry run.

        let make_error = || {
            Error::new(
                format!(
                    "failed to perform hard deletion operation for blob '{}'",
                    blob_hash_to_hex(&s3_hash),
                ),
                ErrorType::Blob,
            )
        };

        Self::check_hash_not_empty(s3_hash).or_raise(make_error)?;

        let mut revisions = SamplerCounter::new();
        let mut files = SamplerCounter::new();
        let mut pages = SamplerCounter::new();
        let mut sites = SamplerCounter::new();
        let mut total_files_deleted = 0;

        match deleter_user_id {
            None => info!(
                "Checking result of hard deletion of all blobs matching hash {}",
                blob_hash_to_hex(&s3_hash),
            ),
            Some(user_id) => {
                info!(
                    "Hard deleting all blobs matching hash {} (done by user ID {})",
                    blob_hash_to_hex(&s3_hash),
                    user_id,
                );
                // TODO add to audit log
            }
        }

        // Get all latest file revisions with this hash, which we then delete
        // so it's no longer the most recent revision (which we can't hide).

        #[derive(Debug, FromQueryResult)]
        struct LatestFileRevision {
            site_id: i64,
            page_id: i64,
            file_id: i64,
            revision_id: i64,
        }

        let query = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            str!("
                SELECT
                    f.site_id AS site_id,
                    f.page_id AS page_id,
                    f.file_id AS file_id,
                    r1.revision_id AS revision_id
                FROM file AS f
                JOIN file_revision AS r1
                    ON f.file_id = r1.file_id
                LEFT OUTER JOIN file_revision AS r2
                    ON (f.file_id = r2.file_id AND r1.revision_number < r2.revision_number)
                WHERE r2.revision_id IS NULL
                AND r1.s3_hash = $1
                AND f.deleted_at IS NULL
            "),
            [Value::from(s3_hash.to_vec())],
        );

        {
            let latest_revisions = FileRevision::find()
                .from_raw_sql(query)
                .into_model::<LatestFileRevision>()
                .all(txn)
                .await
                .or_raise(make_error)?;

            for LatestFileRevision {
                site_id,
                page_id,
                file_id,
                revision_id,
            } in latest_revisions
            {
                total_files_deleted += 1;

                if deleter_user_id.is_some() {
                    // Only do deletions when running for real
                    FileService::delete_with_erased_s3_hash(
                        ctx,
                        DeleteFile {
                            site_id,
                            page_id,
                            file: file_id.into(),
                            last_revision_id: revision_id,
                            revision_comments: format!(
                                "Hard delete {}",
                                blob_hash_to_hex(&s3_hash)
                            ),
                            user_id: SYSTEM_USER_ID,
                        },
                    )
                    .await
                    .or_raise(make_error)?;
                }
            }
        }

        // Go through all the revisions with the matching S3 hash and delete / hide it
        {
            let mut results = FileRevision::find()
                .filter(file_revision::Column::S3Hash.eq(s3_hash.as_slice()))
                .paginate(txn, 20);

            while let Some(revs) = results.fetch_and_next().await.or_raise(make_error)? {
                for rev in revs {
                    revisions.add(rev.revision_id);
                    files.add(rev.file_id);
                    pages.add(rev.page_id);
                    sites.add(rev.site_id);

                    if deleter_user_id.is_some() {
                        // Amend 'hidden' to add 's3_hash'
                        let hidden = {
                            let column = str!("s3_hash"); // avoid double-allocating String
                            let mut hidden = rev.hidden;
                            if !hidden.contains(&column) {
                                hidden.push(column);
                                hidden.sort();
                            }
                            hidden
                        };

                        // Run UPDATE
                        let model = file_revision::ActiveModel {
                            revision_id: Set(rev.revision_id),
                            s3_hash: Set(EMPTY_BLOB_HASH.to_vec()),
                            hidden: Set(hidden),
                            ..Default::default()
                        };

                        model.update(txn).await.or_raise(make_error)?;
                    }
                }
            }
        }

        // Update all users using this blob to remove this as a profile picture
        // (But first getting a set of sample records)

        let sample_user_ids: Vec<i64> = User::find()
            .select_only()
            .column(user::Column::UserId)
            .filter(user::Column::AvatarS3Hash.eq(s3_hash.as_slice()))
            .limit(u64::from(SAMPLE_COUNT))
            .into_tuple()
            .all(txn)
            .await
            .or_raise(make_error)?;

        let total_users: u64 = match deleter_user_id {
            Some(_) => {
                // Mutate, update all users and get count
                let model = user::ActiveModel {
                    avatar_s3_hash: Set(None),
                    ..Default::default()
                };

                User::update_many()
                    .set(model)
                    .filter(user::Column::AvatarS3Hash.eq(s3_hash.as_slice()))
                    .exec(txn)
                    .await
                    .or_raise(make_error)?
                    .rows_affected
            }
            None => {
                // Read-only, just get the count
                User::find()
                    .select_only()
                    .column_as(user::Column::UserId.count(), "count")
                    .filter(user::Column::AvatarS3Hash.eq(s3_hash.as_slice()))
                    .into_tuple::<i64>() // Postgres cannot return u64 as a column type
                    .one(txn)
                    .await
                    .or_raise(make_error)?
                    .expect("No results from COUNT aggregate query")
                    .try_into_u64()
                    .or_raise(make_error)?
            }
        };

        if deleter_user_id.is_some() {
            // Delete and blacklist the hash, nobody should be uploading new versions
            // Only do so if we are actually mutating.
            let (result1, result2) = join!(
                BlobService::add_blacklist(ctx, s3_hash),
                BlobService::hard_delete(ctx, &s3_hash),
            );
            raise_multiple!(result1, result2; make_error);
        }

        // Finish counting and sampling
        let (total_revisions, sample_revision_ids) = revisions.finish();
        let (total_files, sample_file_ids) = files.finish();
        let (total_pages, sample_page_ids) = pages.finish();
        let (total_sites, sample_site_ids) = sites.finish();

        Ok(HardDeleteOutput {
            total_revisions,
            total_files,
            total_files_deleted,
            total_pages,
            total_sites,
            total_users,
            sample_revision_ids,
            sample_file_ids,
            sample_page_ids,
            sample_site_ids,
            sample_user_ids,
        })
    }

    // Blacklist operations

    /// Verifies that the blob hash to blacklist is not the static empty blob.
    ///
    /// After all it makes no sense to blacklist empty files, and doing so
    /// would cause some issues internally.
    pub(crate) fn check_hash_not_empty(hash: BlobHash) -> Result<()> {
        if hash == EMPTY_BLOB_HASH {
            error!("Cannot hard delete the empty blob");
            bail!(Error::new(
                "cannot hard delete the empty blob",
                ErrorType::BadRequest,
            ));
        }

        Ok(())
    }

    /// Checks that a blob hash is not presently used anywhere in the system.
    ///
    /// If this is the case, then a hard deletion is needed instead to purge
    /// this object from data storage.
    pub(crate) async fn check_hash_in_use(
        ctx: &ServiceContext<'_>,
        hash: BlobHash,
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to check whether blob hash '{}' is in use anywhere",
                    blob_hash_to_hex(&hash),
                ),
                ErrorType::Blob,
            )
        };

        let txn = ctx.transaction();
        let count = FileRevision::find()
            .select_only()
            .column_as(file_revision::Column::RevisionId.count(), "count")
            .filter(file_revision::Column::S3Hash.eq(hash.as_slice()))
            .into_tuple::<i64>()
            .one(txn)
            .await
            .or_raise(make_error)?
            .expect("No results from COUNT aggregate query");

        if count > 0 {
            error!(
                "Cannot blacklist a blob that is currently in use (found {count} uses)"
            );
            bail!(Error::new(
                format!(
                    "cannot blacklist a blob that is currently in use (found {} uses)",
                    count,
                ),
                ErrorType::BlobCannotBlacklistExisting
            ));
        }

        debug!("Found no current uses of blob to be blacklisted");
        Ok(())
    }

    pub async fn add_blacklist(ctx: &ServiceContext<'_>, hash: BlobHash) -> Result<()> {
        let created_by = Self::require_platform_staff(ctx, "add a blob blacklist entry")?;
        info!("Adding hash {} to blacklist", blob_hash_to_hex(&hash));

        let make_error =
            || Error::new("failed to add blob to blacklist", ErrorType::Blob);
        Self::check_hash_not_empty(hash).or_raise(make_error)?;
        Self::check_hash_in_use(ctx, hash)
            .await
            .or_raise(make_error)?;

        debug_assert_ne!(
            hash, EMPTY_BLOB_HASH,
            "Empty blob hash passed to add_blacklist()",
        );

        if Self::on_blacklist(ctx, hash).await.or_raise(make_error)? {
            debug!("Already blacklisted, skipping");
            return Ok(());
        }

        let txn = ctx.transaction();
        let model = blob_blacklist::ActiveModel {
            s3_hash: Set(hash.to_vec()),
            created_by: Set(created_by),
            ..Default::default()
        };
        model.insert(txn).await.or_raise(make_error)?;
        Ok(())
    }

    pub async fn remove_blacklist(
        ctx: &ServiceContext<'_>,
        hash: BlobHash,
    ) -> Result<()> {
        Self::require_platform_staff(ctx, "remove a blob blacklist entry")?;
        info!("Removing hash {} to blacklist", blob_hash_to_hex(&hash));

        let txn = ctx.transaction();
        BlobBlacklist::delete_by_id(hash.to_vec())
            .exec(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to remove blob '{}' from blacklist",
                        blob_hash_to_hex(&hash),
                    ),
                    ErrorType::Blob,
                )
            })?;

        Ok(())
    }

    pub async fn on_blacklist(ctx: &ServiceContext<'_>, hash: BlobHash) -> Result<bool> {
        info!(
            "Checking if hash {} is on blacklist",
            blob_hash_to_hex(&hash),
        );

        let make_error = || {
            Error::new(
                format!(
                    "failed to check if blob '{}' is on the blacklist",
                    blob_hash_to_hex(&hash),
                ),
                ErrorType::Blob,
            )
        };

        let txn = ctx.transaction();
        let exists = BlobBlacklist::find()
            .filter(blob_blacklist::Column::S3Hash.eq(hash.as_slice()))
            .one(txn)
            .await
            .or_raise(make_error)?
            .is_some();

        Ok(exists)
    }

    // Getters

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        hash: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        // Special handling for empty blobs
        if hash == EMPTY_BLOB_HASH {
            debug!("Returning the empty blob");
            return Ok(Some(Vec::new()));
        }

        // Retrieve blob from S3
        let bucket = ctx.s3_files_bucket();
        let hex_hash = blob_hash_to_hex(hash);
        let response = bucket
            .get_object(&hex_hash)
            .await
            .or_raise(|| Error::new("failed to get blob", ErrorType::Blob))?;

        match response.status_code() {
            200 => Ok(Some(response.into())),
            404 => Ok(None),
            _ => bail!(s3_error(&response, "fetching S3 blob")),
        }
    }

    #[inline]
    pub async fn get(ctx: &ServiceContext<'_>, hash: &[u8]) -> Result<Vec<u8>> {
        find_or_error!(Self::get_optional(ctx, hash), "blob", Blob)
    }

    pub async fn get_metadata_optional(
        ctx: &ServiceContext<'_>,
        hash: &[u8],
    ) -> Result<Option<BlobMetadata>> {
        let hex_hash = blob_hash_to_hex(hash);
        let make_error = || {
            Error::new(
                format!("failed to get blob metadata for '{}'", hex_hash),
                ErrorType::Blob,
            )
        };

        // Special handling for empty blobs
        if hash == EMPTY_BLOB_HASH {
            return Ok(Some(BlobMetadata {
                mime: str!(EMPTY_BLOB_MIME),
                size: 0,
                created_at: OffsetDateTime::from_unix_timestamp(EMPTY_BLOB_TIMESTAMP)
                    .or_raise(make_error)?,
            }));
        }

        // Retrieve metadata from S3
        match Self::head(ctx, &hex_hash).await.or_raise(make_error)? {
            None => Ok(None),
            Some(result) => {
                let make_error = || {
                    let mut error = make_error();
                    error.error_type = ErrorType::BlobBackend;
                    error
                };

                // Headers should be passed in
                let size = result.content_length.ok_or_raise(make_error)?;
                let mime = result.content_type.ok_or_raise(make_error)?;
                let created_at = {
                    let timestamp = result.last_modified.ok_or_raise(make_error)?;
                    OffsetDateTime::parse(&timestamp, &Rfc2822).or_raise(make_error)?
                };

                Ok(Some(BlobMetadata {
                    mime,
                    size,
                    created_at,
                }))
            }
        }
    }

    #[inline]
    pub async fn get_metadata(
        ctx: &ServiceContext<'_>,
        hash: &[u8],
    ) -> Result<BlobMetadata> {
        find_or_error!(Self::get_metadata_optional(ctx, hash), "blob", Blob)
    }

    /// Possibly retrieve blob contents, if a flag is set.
    ///
    /// This utility conditionally retrieves the
    /// text given by the specified hash only
    /// if the flag `requested` is true.
    /// Otherwise, it does no action, returning `None`.
    pub async fn fetch_if_requested(
        ctx: &ServiceContext<'_>,
        requested: bool,
        hash: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        if requested {
            let data = Self::get(ctx, hash).await.or_raise(|| {
                Error::new("failed to conditionally get blob data", ErrorType::Blob)
            })?;

            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    async fn head(
        ctx: &ServiceContext<'_>,
        path: &str,
    ) -> Result<Option<HeadObjectResult>> {
        let bucket = ctx.s3_files_bucket();
        let (result, status) = bucket.head_object(path).await.or_raise(|| {
            Error::new(
                format!("failed to HEAD existence of S3 object '{}'", path),
                ErrorType::Blob,
            )
        })?;

        match status {
            200 | 204 => Ok(Some(result)),
            404 => Ok(None),
            _ => {
                let response = ResponseData::new(Bytes::new(), status, HashMap::new());
                bail!(s3_error(&response, "heading S3 blob"));
            }
        }
    }

    async fn exists(ctx: &ServiceContext<'_>, path: &str) -> Result<bool> {
        let head = Self::head(ctx, path)
            .await
            .or_raise(|| Error::new("failed to check blob existence", ErrorType::Blob))?;

        Ok(head.is_some())
    }

    pub async fn hard_delete(ctx: &ServiceContext<'_>, hash: &[u8]) -> Result<()> {
        // Special handling for empty blobs
        //
        // Being virtual, having always existed, they cannot be deleted.
        // So this is a no-op.
        if hash == EMPTY_BLOB_HASH {
            debug!("Ignoring attempt to hard delete the empty blob");
            return Ok(());
        }

        // Delete from S3
        let bucket = ctx.s3_files_bucket();
        let hex_hash = blob_hash_to_hex(hash);

        let response = bucket
            .delete_object(&hex_hash)
            .await
            .or_raise(|| Error::new("failed to hard delete blob", ErrorType::Blob))?;

        match response.status_code() {
            204 => Ok(()),
            _ => bail!(s3_error(&response, "hard-deleting S3 blob")),
        }
    }
}

/// Helper method to parse out an S3 error response and print the message (if any).
#[must_use]
fn s3_error(response: &ResponseData, action: &str) -> Error {
    let error_message = match str::from_utf8(response.bytes()) {
        Ok("") => "(no content)",
        Ok(m) => m,
        Err(_) => "(invalid UTF-8)",
    };

    error!(
        "Error while {} (HTTP {}): {}",
        action,
        response.status_code(),
        error_message,
    );
    Error::new(
        format!(
            "failed {}, (HTTP {}): {}",
            action,
            response.status_code(),
            error_message,
        ),
        ErrorType::BlobBackend,
    )
}

#[derive(Debug)]
enum HardDeleteInner {
    Commit { s3_hash: BlobHash, user_id: i64 },
    DryRun { s3_hash: BlobHash },
}

#[derive(Debug)]
struct PendingBlob {
    s3_path: String,
    expected_length: i64,
    moved_hash: Option<Vec<u8>>,
}

/// Helper struct to produce a count of items and a sample list.
#[derive(Debug)]
struct SamplerCounter<T: Hash + Ord + Eq> {
    items: HashSet<T>,
}

impl<T: Hash + Ord + Eq> SamplerCounter<T> {
    #[inline]
    fn new() -> Self {
        SamplerCounter {
            items: HashSet::new(),
        }
    }

    fn add(&mut self, item: T) {
        self.items.insert(item);
    }

    fn finish(self) -> (usize, Vec<T>) {
        let count = self.items.len();
        let samples = {
            let mut samples = self
                .items
                .into_iter()
                .take(usize::from(SAMPLE_COUNT))
                .collect::<Vec<_>>();

            samples.sort();
            samples
        };

        (count, samples)
    }
}

// Helper functions to assist rustc in disambiguating error typing.
