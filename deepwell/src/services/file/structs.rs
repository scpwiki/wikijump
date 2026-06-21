/*
 * services/file/structs.rs
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

use crate::services::file_revision::{
    CreateFileRevisionOutput, CreateFirstFileRevisionOutput,
};
use crate::types::{Bytes, FileDetails, FileRevisionType, Maybe, Reference};
use serde_json::Value as JsonValue;
use std::net::IpAddr;
use time::OffsetDateTime;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateFile {
    pub site_id: i64,
    pub page_id: i64,
    pub name: String,
    pub uploaded_blob_id: String,
    pub revision_comments: String,
    pub user_id: i64,

    #[serde(default)]
    pub bypass_filter: bool,
    pub ip_address: IpAddr,

    /// Allows internal users to upload directly.
    /// When this is present, the value of `uploaded_blob_id` is ignored.
    ///
    /// Must always be `None` when called via API.
    #[serde(default, skip)]
    pub(crate) direct_upload: Option<Vec<u8>>,
}

pub type CreateFileOutput = CreateFirstFileRevisionOutput;

#[derive(Deserialize, Debug, Clone)]
pub struct GetFile<'a> {
    pub site_id: i64,
    pub page_id: i64,
    pub file: Reference<'a>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetFileDetails<'a> {
    #[serde(flatten)]
    pub input: GetFile<'a>,

    #[serde(default)]
    pub details: FileDetails,
}

#[derive(Serialize, Debug, Clone)]
pub struct GetFileOutput {
    pub file_id: i64,

    #[serde(with = "time::serde::rfc3339")]
    pub file_created_at: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339::option")]
    pub file_updated_at: Option<OffsetDateTime>,

    #[serde(with = "time::serde::rfc3339::option")]
    pub file_deleted_at: Option<OffsetDateTime>,
    pub page_id: i64,
    pub revision_id: i64,
    pub revision_type: FileRevisionType,

    #[serde(with = "time::serde::rfc3339")]
    pub revision_created_at: OffsetDateTime,
    pub revision_number: i32,
    pub revision_user_id: i64,
    pub name: String,
    pub data: Option<Bytes<'static>>,
    pub mime: String,
    pub size: i64,
    pub s3_hash: Bytes<'static>,
    pub revision_comments: String,
    pub hidden_fields: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetPageFiles {
    pub site_id: i64,
    pub page_id: i64,
    pub deleted: Maybe<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EditFile {
    pub site_id: i64,
    pub page_id: i64,
    pub file_id: i64,
    pub user_id: i64,
    pub last_revision_id: i64,
    pub revision_comments: String,

    #[serde(flatten)]
    pub body: EditFileBody,

    #[serde(default)]
    pub bypass_filter: bool,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct EditFileBody {
    pub name: Maybe<String>,
    pub uploaded_blob_id: Maybe<String>,

    /// Allows internal users to upload directly.
    /// When this is present, the value of `uploaded_blob_id` is ignored
    /// (even though it still must be set).
    ///
    /// Must always be `Unset` when called via API.
    #[serde(skip)]
    pub(crate) direct_upload: Maybe<Vec<u8>>,
}

pub type EditFileOutput = CreateFileRevisionOutput;

#[derive(Deserialize, Debug, Clone)]
pub struct MoveFile<'a> {
    pub revision_comments: String,
    pub site_id: i64,
    pub file_id: i64,
    pub user_id: i64,
    pub last_revision_id: i64,
    pub name: Option<String>,
    pub current_page_id: i64,
    pub destination_page: Reference<'a>,
}

pub type MoveFileOutput = CreateFileRevisionOutput;

#[derive(Deserialize, Debug, Clone)]
pub struct DeleteFile<'a> {
    pub last_revision_id: i64,
    pub revision_comments: String,
    pub site_id: i64,
    pub page_id: i64,
    pub file: Reference<'a>,
    pub user_id: i64,
}

#[derive(Serialize, Debug, Clone)]
pub struct DeleteFileOutput {
    pub file_id: i64,
    pub file_revision_id: i64,
    pub file_revision_number: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RestoreFile<'a> {
    pub revision_comments: String,
    pub new_page: Option<Reference<'a>>,
    pub new_name: Option<String>,
    pub site_id: i64,
    pub page_id: i64,
    pub file_id: i64,
    pub user_id: i64,
}

#[derive(Serialize, Debug, Clone)]
pub struct RestoreFileOutput {
    pub page_id: i64,
    pub file_id: i64,
    pub name: String,
    pub file_revision_id: i64,
    pub file_revision_number: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RollbackFile<'a> {
    pub site_id: i64,
    pub page_id: i64,
    pub file: Reference<'a>,
    pub last_revision_id: i64,
    pub revision_number: i32,
    pub revision_comments: String,
    pub user_id: i64,

    #[serde(default)]
    pub bypass_filter: bool,
    pub ip_address: IpAddr,
}
