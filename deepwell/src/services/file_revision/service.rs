/*
 * services/file_revision/service.rs
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
use crate::json_utils::string_list_to_json;
use crate::models::file_revision::{
    self, Entity as FileRevision, Model as FileRevisionModel,
};
use serde_json::json;

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
            file_id,
            page_id,
            user_id,
            comments,
            body,
        }: CreateFileRevision,
        previous: FileRevisionModel,
    ) -> Result<Option<CreateFileRevisionOutput>> {
        let txn = ctx.transaction();
        let revision_number = next_revision_number(&previous, &file_id, page_id);

        // Fields to create in the revision
        let mut changes = Vec::new();
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

        if let ProvidedValue::Set(new_name) = body.name {
            if name != new_name {
                changes.push("name");
                name = new_name;
            }
        }

        if let ProvidedValue::Set(new_blob) = body.blob {
            if s3_hash != new_blob.s3_hash || size_hint != new_blob.size_hint {
                changes.push("blob");
                s3_hash = new_blob.s3_hash.to_vec();
                size_hint = new_blob.size_hint;
            }
        }

        if let ProvidedValue::Set(new_mime_hint) = body.mime_hint {
            if mime_hint != new_mime_hint {
                changes.push("mime");
                mime_hint = new_mime_hint;
            }
        }

        if let ProvidedValue::Set(new_licensing) = body.licensing {
            if licensing != new_licensing {
                changes.push("licensing");
                licensing = new_licensing;
            }
        }

        // If nothing has changed, then don't create a new revision
        if changes.is_empty() {
            // TODO rerender page
            return Ok(None);
        }

        // Validate inputs
        if name.is_empty() || name.len() >= 256 {
            tide::log::error!("File name of invalid length: {}", name.len());
            return Err(Error::BadRequest);
        }

        if mime_hint.is_empty() {
            tide::log::error!("MIME type hint is empty");
            return Err(Error::BadRequest);
        }

        // TODO validate licensing field

        // Insert the new revision into the table
        let changes = string_list_to_json(&changes)?;
        let model = file_revision::ActiveModel {
            revision_type: Set(FileRevisionType::Create),
            revision_number: Set(0),
            file_id: Set(file_id.clone()),
            page_id: Set(page_id),
            user_id: Set(user_id),
            name: Set(name),
            s3_hash: Set(s3_hash.to_vec()),
            size_hint: Set(size_hint),
            mime_hint: Set(mime_hint),
            licensing: Set(licensing),
            changes: Set(json!(["name", "blob", "mime", "licensing"])),
            ..Default::default()
        };

        let FileRevisionModel { revision_id, .. } = model.insert(txn).await?;
        Ok(Some(CreateFileRevisionOutput {
            file_revision_id: revision_id,
            file_revision_number: revision_number,
        }))
    }

    /// Creates the first revision for a newly-uploaded file.
    ///
    /// See `RevisionService::create_first()`.
    ///
    /// # Panics
    /// If the given previous revision is for a different file or page, this method will panic.
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
            licensing,
            comments,
        }: CreateFirstFileRevision,
    ) -> Result<CreateFirstFileRevisionOutput> {
        let txn = ctx.transaction();

        todo!()
    }
}

fn next_revision_number(
    previous: &FileRevisionModel,
    file_id: &str,
    page_id: i64,
) -> i32 {
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
