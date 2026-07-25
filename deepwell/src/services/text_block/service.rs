/*
 * services/text_block/service.rs
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

//! Manages the storage for hosted text blocks.
//!
//! This does _not_ have any methods for doing CRUD
//! on individual entries, since text blocks are only
//! updated when the underlying page is, which means
//! they all get replaced in one operation.
//!
//! The only other operation is page deletion, where
//! all the hosted text block data is removed.
//! (If a page is resurrected, then all this data gets
//! re-inserted as part of creating the new revision.)
//!
//! Additionally, for fetching text blocks, this is
//! done by wws by directly accessing S3 itself, so
//! nothing here is needed.

use super::structs::{TextBlock, TextBlockIndex};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt, StdResult};
use crate::models::text_block::{self, Entity as TextBlockTable};
use crate::services::ServiceContext;
use crate::types::TextBlockType;
use crate::utils::ConvertToI16;
use futures::{StreamExt, stream::FuturesUnordered};
use sea_orm::{
    ColumnTrait, Condition, DeleteResult, EntityTrait, QueryFilter, QuerySelect, Set,
};
use std::collections::HashSet;

/// Keep each SeaORM insert well below PostgreSQL's 65,535 bind-parameter limit.
const TEXT_BLOCK_INSERT_BATCH_SIZE: usize = 1_000;

/// Bound concurrent object-store writes while avoiding one network round trip per block.
const TEXT_BLOCK_UPLOAD_CONCURRENCY: usize = 16;

/// Write out the S3 filename for this hosted text block.
///
/// This allows reusing a buffer, since we need to write out several
/// and don't need the string other than for running the S3 operation.
///
/// They all take the format of `<PAGE ID>_<BLOCK TYPE>_<BLOCK INDEX>`.
/// These are always 1-indexed, since that's how they're addressed via URL.
/// To give some examples of filenames, consider we have a page with ID
/// 12345 which has 2 code blocks and 3 html blocks. The objects in the bucket
/// would be:
///
/// * `12345_code_1`
/// * `12345_code_2`
/// * `12345_html_1`
/// * `12345_html_2`
/// * `12345_html_3`
macro_rules! format_filename {
    ($buffer:expr, $page_id:expr, $index:expr, $block_type:expr $(,)?) => {{
        let buffer = &mut $buffer;
        let page_id = $page_id;
        let index = $index;
        let block_type = block_type_name($block_type);

        buffer.clear();
        assert_ne!(index, 0, "Text block indices must be 1-indexed!");
        str_write!(buffer, "{page_id}_{block_type}_{index}");
    }};
}

#[derive(Debug)]
pub struct TextBlockService;

impl TextBlockService {
    /// Validates both hosted text block collections for a page.
    ///
    /// Render preflights both counts together before writing either collection
    /// to S3, so an invalid later collection cannot leave objects from the
    /// earlier collection behind when the database transaction rolls back.
    pub(crate) fn validate_page_block_counts(
        html_count: usize,
        code_count: usize,
    ) -> StdResult<(), std::num::TryFromIntError> {
        max_text_block_index(html_count)?;
        max_text_block_index(code_count)?;
        Ok(())
    }

    /// Replaces the text blocks associated with this page with the ones given.
    ///
    /// This is to be run after ftml returns the lists of code and html blocks
    /// found in the source, which will replace the existing text block data
    /// to be replaced.
    pub async fn add_blocks(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        block_type: TextBlockType,
        blocks: &[TextBlock<'_>],
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to insert {} new {} text blocks for page ID {}",
                    blocks.len(),
                    block_type_name(block_type),
                    page_id,
                ),
                ErrorType::TextBlock,
            )
        };

        info!(
            "Inserting {} {} text blocks for page ID {}",
            blocks.len(),
            block_type_name(block_type),
            page_id,
        );

        let txn = ctx.transaction();
        let bucket = ctx.s3_tblocks_bucket();
        let mut buffer = String::new();

        // First, get the stored filenames for this block type.
        // After the new blocks are uploaded, old filenames not present in the
        // replacement batch are removed from S3.
        let previous_filenames =
            Self::get_block_s3_filenames(ctx, page_id, Some(block_type))
                .await
                .or_raise(make_error)?;

        // If there's no additional work for us, quit early

        if blocks.is_empty() && previous_filenames.is_empty() {
            debug!("Not inserting any blocks, no prior blocks to remove");
            return Ok(());
        }

        // Validate the maximum 1-indexed block index before uploading
        // anything to S3. S3 writes are external side effects and are not
        // rolled back with the database transaction, so all deterministic
        // range errors must happen before the first upload.
        let max_index = max_text_block_index(blocks.len()).or_raise(make_error)?;

        // Prepare and validate the complete replacement before starting external
        // writes. The object-store uploads then run with bounded concurrency so a
        // page containing many hosted blocks does not pay one network round trip
        // after another.
        let mut uploads = Vec::with_capacity(blocks.len());
        let mut models = Vec::with_capacity(blocks.len());
        let mut new_filenames = HashSet::with_capacity(blocks.len());
        let mut previous_block_names = HashSet::with_capacity(blocks.len());
        for (index, block) in (1..=max_index).zip(blocks.iter()) {
            let &TextBlock {
                text,
                text_type,
                mime,
                mut name,
            } = block;

            // Deny invalid block names
            if let Some(mut value) = name {
                value = value.trim();
                if !valid_block_name(&mut previous_block_names, value) {
                    name = None;
                }
            }

            format_filename!(buffer, page_id, index, block_type);
            let filename = buffer.clone();
            uploads.push((filename.clone(), text, mime));

            models.push(text_block::ActiveModel {
                block_type: Set(block_type),
                page_id: Set(page_id),
                block_index: Set(index),
                s3_filename: Set(filename.clone()),
                block_name: Set(name.map(String::from)),
                text_type: Set(text_type.map(String::from)),
            });
            new_filenames.insert(filename);
        }

        let mut pending_uploads = FuturesUnordered::new();
        for (filename, text, mime) in &uploads {
            pending_uploads.push(async move {
                debug!("Uploading new S3 text block {filename} ({mime})");
                bucket
                    .put_object_with_content_type(filename, text.as_bytes(), mime)
                    .await
            });
            if pending_uploads.len() == TEXT_BLOCK_UPLOAD_CONCURRENCY {
                while let Some(result) = pending_uploads.next().await {
                    result.or_raise(make_error)?;
                }
            }
        }
        while let Some(result) = pending_uploads.next().await {
            result.or_raise(make_error)?;
        }

        // Then, delete the blocks from the database.
        //
        // This doesn't require us to know which need to be kept
        // because we're just INSERTing over all of it.

        let DeleteResult { rows_affected, .. } = TextBlockTable::delete_many()
            .filter(
                Condition::all()
                    .add(text_block::Column::BlockType.eq(block_type))
                    .add(text_block::Column::PageId.eq(page_id)),
            )
            .exec(txn)
            .await
            .or_raise(make_error)?;

        debug_assert_eq!(
            rows_affected,
            previous_filenames.len() as u64,
            "Deleted row count does not match previous text block filename count",
        );

        // Finally, insert the new text block rows in bounded batches. SeaORM
        // emits one statement per insert_many call, so an unbounded batch can
        // exceed PostgreSQL's bind limit after the S3 uploads have succeeded.
        for range in text_block_insert_ranges(models.len()) {
            TextBlockTable::insert_many(models[range].iter().cloned())
                .exec(txn)
                .await
                .or_raise(make_error)?;
        }

        for filename in &previous_filenames {
            if !new_filenames.contains(filename) {
                debug!("Deleting replaced S3 text block {filename}");
                bucket.delete_object(filename).await.or_raise(make_error)?;
            }
        }

        Ok(())
    }

    /// Gets the index and associated S3 name for a block accessed via name/alias.
    pub async fn get_block_index(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        block_type: TextBlockType,
        name: &str,
    ) -> Result<Option<TextBlockIndex>> {
        info!(
            "Looking for a {} text block on page ID {} with name '{}'",
            block_type_name(block_type),
            page_id,
            name,
        );

        let make_error = || {
            Error::new(
                format!(
                    "failed to find a {} text block on page ID {} with name '{}'",
                    block_type_name(block_type),
                    page_id,
                    name,
                ),
                ErrorType::TextBlock,
            )
        };

        let txn = ctx.transaction();
        let block: Option<(i16, String)> = TextBlockTable::find()
            .select_only()
            .column(text_block::Column::BlockIndex)
            .column(text_block::Column::S3Filename)
            .filter(
                Condition::all()
                    .add(text_block::Column::PageId.eq(page_id))
                    .add(text_block::Column::BlockType.eq(block_type))
                    .add(text_block::Column::BlockName.eq(name)),
            )
            .into_tuple()
            .one(txn)
            .await
            .or_raise(make_error)?;

        match block {
            None => Ok(None),
            Some((index, s3_filename)) => Ok(Some(TextBlockIndex { index, s3_filename })),
        }
    }

    /// Gets the index and associated S3 name for a block accessed via numeric index.
    pub async fn get_block_by_index(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        block_type: TextBlockType,
        index: i16,
    ) -> Result<Option<TextBlockIndex>> {
        info!(
            "Looking for a {} text block on page ID {} with index {}",
            block_type_name(block_type),
            page_id,
            index,
        );

        let make_error = || {
            Error::new(
                format!(
                    "failed to find a {} text block on page ID {} with index {}",
                    block_type_name(block_type),
                    page_id,
                    index,
                ),
                ErrorType::TextBlock,
            )
        };

        let txn = ctx.transaction();
        let s3_filename: Option<String> = TextBlockTable::find()
            .select_only()
            .column(text_block::Column::S3Filename)
            .filter(
                Condition::all()
                    .add(text_block::Column::PageId.eq(page_id))
                    .add(text_block::Column::BlockType.eq(block_type))
                    .add(text_block::Column::BlockIndex.eq(index)),
            )
            .into_tuple()
            .one(txn)
            .await
            .or_raise(make_error)?;

        match s3_filename {
            None => Ok(None),
            Some(s3_filename) => Ok(Some(TextBlockIndex { index, s3_filename })),
        }
    }

    /// Finds how many text blocks of a type exist for a page.
    async fn get_block_s3_filenames(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        block_type: Option<TextBlockType>,
    ) -> Result<Vec<String>> {
        let txn = ctx.transaction();
        let mut condition = Condition::all().add(text_block::Column::PageId.eq(page_id));
        if let Some(block_type) = block_type {
            condition = condition.add(text_block::Column::BlockType.eq(block_type));
        }

        TextBlockTable::find()
            .select_only()
            .column(text_block::Column::S3Filename)
            .filter(condition)
            .into_tuple()
            .all(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to get text block S3 filenames for page ID {page_id}"
                    ),
                    ErrorType::TextBlock,
                )
            })
    }

    /// Delete all hosted text blocks stored for a page.
    ///
    /// This is run when a page is deleted, since the page
    /// becomes inaccessible and storing this redundant information
    /// becomes unnecessary.
    pub async fn delete_blocks(ctx: &ServiceContext<'_>, page_id: i64) -> Result<()> {
        let txn = ctx.transaction();
        let bucket = ctx.s3_tblocks_bucket();
        let make_error = || {
            Error::new(
                format!("failed to delete text blocks on page ID {}", page_id),
                ErrorType::TextBlock,
            )
        };

        let filenames = Self::get_block_s3_filenames(ctx, page_id, None)
            .await
            .or_raise(make_error)?;

        for filename in &filenames {
            debug!("Deleting text block {filename}");
            bucket.delete_object(filename).await.or_raise(make_error)?;
        }

        // Now that S3 is cleared out, we can delete all the
        // database rows in one sweep.
        TextBlockTable::delete_many()
            .filter(text_block::Column::PageId.eq(page_id))
            .exec(txn)
            .await
            .or_raise(make_error)?;

        Ok(())
    }
}

fn block_type_name(block_type: TextBlockType) -> &'static str {
    match block_type {
        TextBlockType::Html => "html",
        TextBlockType::Code => "code",
    }
}

/// Returns the largest 1-indexed text block index for this block count.
fn max_text_block_index(count: usize) -> StdResult<i16, std::num::TryFromIntError> {
    count.try_into_i16()
}

fn text_block_insert_ranges(
    count: usize,
) -> impl Iterator<Item = std::ops::Range<usize>> {
    (0..count)
        .step_by(TEXT_BLOCK_INSERT_BATCH_SIZE)
        .map(move |start| start..(start + TEXT_BLOCK_INSERT_BATCH_SIZE).min(count))
}

/// Ensures that this name can be used to reference a block.
fn valid_block_name<'n>(previous: &mut HashSet<&'n str>, name: &'n str) -> bool {
    // To prevent shenanigans with excessively-long block aliases.
    const MAX_BLOCK_NAME_LEN: usize = 32;

    if name.is_empty() {
        warn!("Empty block name passed, rejecting");
        return false;
    }

    let char_len = name.chars().count();
    if char_len > MAX_BLOCK_NAME_LEN {
        warn!("Block name '{name}' is too long ({char_len} > {MAX_BLOCK_NAME_LEN})");
        return false;
    }

    if name.chars().all(|c| c.is_ascii_digit()) {
        warn!("Numeric block name '{name}' passed, rejecting");
        return false;
    }

    if previous.contains(name) {
        warn!("Block name '{name}' has already been used for this page, rejecting");
        return false;
    }

    // Now that all checks have passed, add this as one of the already-used names.
    previous.insert(name);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Iterable;

    #[test]
    fn text_block_count_and_index_range_fit_i16_boundaries() {
        assert_eq!(max_text_block_index(0).unwrap(), 0);
        assert_eq!(max_text_block_index(i16::MAX as usize).unwrap(), i16::MAX,);
        assert!(max_text_block_index(i16::MAX as usize + 1).is_err());

        for count in [0, 1, i16::MAX as usize] {
            let max_index = max_text_block_index(count).unwrap();
            let indices = 1..=max_index;
            let expected_first = (count > 0).then_some(1);
            let expected_last = (count > 0).then_some(max_index);

            assert_eq!(indices.clone().count(), count);
            assert_eq!(indices.clone().next(), expected_first);
            assert_eq!(indices.clone().next_back(), expected_last);
        }
    }

    #[test]
    fn page_block_counts_are_prevalidated_together() {
        let max = i16::MAX as usize;

        assert!(TextBlockService::validate_page_block_counts(max, max).is_ok());
        assert!(TextBlockService::validate_page_block_counts(max + 1, 1).is_err());
        assert!(TextBlockService::validate_page_block_counts(1, max + 1).is_err());
    }

    #[test]
    fn insert_batch_stays_below_postgres_bind_limit() {
        let bound_columns_per_row = text_block::Column::iter().count();
        let bound_parameters = TEXT_BLOCK_INSERT_BATCH_SIZE * bound_columns_per_row;

        assert!(bound_parameters <= u16::MAX as usize);
    }

    #[test]
    fn object_store_upload_concurrency_is_bounded() {
        assert!((1..=32).contains(&TEXT_BLOCK_UPLOAD_CONCURRENCY));
    }

    #[test]
    fn combined_overflow_preflight_has_zero_fake_s3_side_effects() {
        let max = i16::MAX as usize;

        for (html_count, code_count) in [(max + 1, 1), (1, max + 1)] {
            let mut fake_s3_uploads = Vec::new();
            let validation =
                TextBlockService::validate_page_block_counts(html_count, code_count);
            if validation.is_ok() {
                fake_s3_uploads.extend(0..html_count);
                fake_s3_uploads.extend(0..code_count);
            }

            assert!(validation.is_err());
            assert!(fake_s3_uploads.is_empty());
        }
    }

    #[test]
    fn one_thousand_and_one_blocks_keep_indices_across_two_insert_batches() {
        let count = TEXT_BLOCK_INSERT_BATCH_SIZE + 1;
        let max_index = max_text_block_index(count).unwrap();
        let indices = (1..=max_index).collect::<Vec<_>>();
        let batches = text_block_insert_ranges(count).collect::<Vec<_>>();

        assert_eq!(indices.len(), count);
        assert_eq!(indices[0], 1);
        assert_eq!(indices[999], 1_000);
        assert_eq!(indices[1_000], 1_001);
        assert_eq!(batches, [0..1_000, 1_000..1_001]);
    }

    #[test]
    fn induced_second_batch_failure_stops_before_later_batches() {
        let count = TEXT_BLOCK_INSERT_BATCH_SIZE * 2 + 1;
        let mut attempted = Vec::new();
        let result: StdResult<(), &'static str> = text_block_insert_ranges(count)
            .enumerate()
            .try_for_each(|(batch, range)| {
                attempted.push(range);
                if batch == 1 {
                    return Err("induced second-batch database failure");
                }
                Ok(())
            });

        assert_eq!(result, Err("induced second-batch database failure"));
        assert_eq!(attempted, [0..1_000, 1_000..2_000]);
        assert!(!attempted.contains(&(2_000..2_001)));
    }
}
