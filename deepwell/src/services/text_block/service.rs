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

use super::prelude::*;
use crate::models::text_block::{
    self, Entity as TextBlockTable, Model as TextBlockModel,
};
use crate::types::TextBlockType;
use sea_orm::ActiveEnum;
use std::collections::HashSet;
use strum::IntoEnumIterator;

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
        use std::ops::Add;

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

        macro_rules! filename {
            ($index:expr) => {{
                format_filename!(buffer, page_id, $index, block_type);
                &buffer
            }};
        }

        // First, get the largest block index for this type.
        // This is needed for the step where we delete extraneous objects in S3.
        //
        // Consider a page which had 5 code blocks but now only has 2. Clearly,
        // we need to delete the last three (the other two will get overwritten
        // with the PutObject operation). So we fetch the maximum block index and
        // delete everything from index blocks.len() through max_index.

        let prev_max_index = Self::get_block_count(ctx, page_id, block_type)
            .await
            .or_raise(make_error)?;

        let max_index = blocks.len().add(1).try_into_i16().or_raise(make_error)?;

        // If there's no additional work for us, quit early

        if blocks.is_empty() && prev_max_index == 0 {
            debug!("Not inserting any blocks, no prior blocks to remove");
            return Ok(());
        }

        // As described above, we delete these extra blocks from S3.
        // If there are more or the same number of blocks now,
        // then this will do nothing.

        for index in max_index..=prev_max_index {
            let filename = filename!(index);
            debug!("Deleting now-out-of-range S3 text block {filename}");
            bucket.delete_object(filename).await.or_raise(make_error)?;
        }

        // Upload the new text blocks to S3.
        // This also replaces the existing S3 objects,
        // which is why we don't have to delete everything above.
        //
        // While we're at it, we can also create the models to be
        // inserted to the database.

        let max_index_usize = max_index.try_into_usize().or_raise(make_error)?;
        let mut models = Vec::with_capacity(max_index_usize);
        let mut previous_block_names = HashSet::with_capacity(max_index_usize);
        for (index, block) in blocks.iter().enumerate() {
            let &TextBlock {
                text,
                text_type,
                mime,
                mut name,
            } = block;

            // Text block indices are always 1-indexed
            let index = index.add(1).try_into_i16().or_raise(make_error)?;

            // Upload text block to S3
            let filename = filename!(index);
            debug!("Uploading new S3 text block {filename} ({mime})");
            bucket
                .put_object_with_content_type(filename, text.as_bytes(), mime)
                .await
                .or_raise(make_error)?;

            // Deny invalid block names
            if let Some(mut value) = name {
                value = value.trim();
                if !valid_block_name(&mut previous_block_names, value) {
                    name = None;
                }
            }

            models.push(text_block::ActiveModel {
                block_type: Set(block_type),
                page_id: Set(page_id),
                block_index: Set(index),
                block_name: Set(name.map(String::from)),
                text_type: Set(text_type.map(String::from)),
            });
        }

        // Then, delete the blocks from the database.
        //
        // This doesn't require us to know which need to be kept
        // because we're just INSERTing over all of it.

        let DeleteResult { rows_affected } = TextBlockTable::delete_many()
            .filter(
                Condition::all()
                    .add(text_block::Column::BlockType.eq(block_type))
                    .add(text_block::Column::PageId.eq(page_id)),
            )
            .exec(txn)
            .await
            .or_raise(make_error)?;

        debug_assert_eq!(
            rows_affected, prev_max_index as u64,
            "Deleted row count do not match previous maximum block index",
        );

        // Finally, insert the batch of new text block rows, then return.
        if !models.is_empty() {
            TextBlockTable::insert_many(models)
                .exec(txn)
                .await
                .or_raise(make_error)?;
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
        let index: Option<i16> = TextBlockTable::find()
            .select_only()
            .column(text_block::Column::BlockIndex)
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

        match index {
            None => Ok(None),
            Some(index) => {
                let mut s3_filename = String::new();
                format_filename!(s3_filename, page_id, index, block_type);
                Ok(Some(TextBlockIndex { index, s3_filename }))
            }
        }
    }

    /// Finds how many text blocks of a type exist for a page.
    async fn get_block_count(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        block_type: TextBlockType,
    ) -> Result<i16> {
        let txn = ctx.transaction();
        let count = TextBlockTable::find()
            .select_only()
            .column(text_block::Column::BlockIndex)
            .filter(
                Condition::all()
                    .add(text_block::Column::PageId.eq(page_id))
                    .add(text_block::Column::BlockType.eq(block_type)),
            )
            .order_by_desc(text_block::Column::BlockIndex)
            .limit(1)
            .into_tuple()
            .one(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to get {} text block count for page ID {}",
                        block_type_name(block_type),
                        page_id,
                    ),
                    ErrorType::TextBlock,
                )
            })?
            .unwrap_or(0);

        Ok(count)
    }

    /// Delete all hosted text blocks stored for a page.
    ///
    /// This is run when a page is deleted, since the page
    /// becomes inaccessible and storing this redundant information
    /// becomes unnecessary.
    pub async fn delete_blocks(ctx: &ServiceContext<'_>, page_id: i64) -> Result<()> {
        let txn = ctx.transaction();
        let bucket = ctx.s3_tblocks_bucket();
        let mut buffer = String::new();

        let make_error = || {
            Error::new(
                format!("failed to delete text blocks on page ID {}", page_id),
                ErrorType::TextBlock,
            )
        };

        // For each kind of text block type, find out how many
        // blocks exist and then delete the objects in S3.
        for block_type in TextBlockType::iter() {
            macro_rules! filename {
                ($index:expr) => {{
                    format_filename!(buffer, page_id, $index, block_type);
                    &buffer
                }};
            }

            let max_index = Self::get_block_count(ctx, page_id, block_type)
                .await
                .or_raise(make_error)?;

            for index in 1..=max_index {
                let filename = filename!(index);
                debug!("Deleting text block {filename}");
                bucket.delete_object(filename).await.or_raise(make_error)?;
            }
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
