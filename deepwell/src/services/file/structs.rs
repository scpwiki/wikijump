/*
 * services/file/structs.rs
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

use crate::models::sea_orm_active_enums::FileRevisionType;
use crate::services::file_revision::{
    CreateFileRevisionOutput, CreateFirstFileRevisionOutput,
    FinishFileRevisionUploadOutput,
};
use crate::web::{Bytes, FileDetails, ProvidedValue, Reference};
use serde_json::Value as JsonValue;
use time::OffsetDateTime;

#[derive(Deserialize, Debug, Clone)]
pub struct StartFileCreation {
    pub site_id: i64,
    pub page_id: i64,
    pub name: String,
    pub revision_comments: String,
    pub user_id: i64,
    pub licensing: JsonValue, // TODO

    #[serde(default)]
    pub bypass_filter: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct StartFileCreationOutput {
    pub pending_blob_id: i64,
    pub presign_url: String,
    pub file_revision_id: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FinishFileCreation {
    pub site_id: i64,
    pub page_id: i64,
    pub file_id: i64,
    pub pending_blob_id: i64,
}

pub type FinishFileCreationOutput = FinishFileRevisionUploadOutput;

#[derive(Deserialize, Debug, Clone)]
pub struct UploadFileEdit {
    pub site_id: i64,
    pub page_id: i64,
    pub file_id: i64,
    pub user_id: i64,
    pub revision_comments: String,
}

pub type UploadFileEditOutput = CreateFileRevisionOutput;

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
    pub file_created_at: OffsetDateTime,
    pub file_updated_at: Option<OffsetDateTime>,
    pub file_deleted_at: Option<OffsetDateTime>,
    pub page_id: i64,
    pub revision_id: i64,
    pub revision_type: FileRevisionType,
    pub revision_created_at: OffsetDateTime,
    pub revision_number: i32,
    pub revision_user_id: i64,
    pub name: String,
    pub data: Option<Bytes<'static>>,
    pub mime: String,
    pub size: i64,
    pub licensing: JsonValue,
    pub revision_comments: String,
    pub hidden_fields: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct GetBlobOutput {
    pub data: Vec<u8>,
    pub mime: String,
    pub size: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EditFile {
    pub site_id: i64,
    pub page_id: i64,
    pub file_id: i64,
    pub user_id: i64,
    pub revision_comments: String,

    #[serde(flatten)]
    pub body: EditFileBody,

    #[serde(default)]
    pub bypass_filter: bool,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct EditFileBody {
    pub name: ProvidedValue<String>,
    pub licensing: ProvidedValue<serde_json::Value>,
}

pub type EditFileOutput = CreateFileRevisionOutput;

#[derive(Deserialize, Debug, Clone)]
pub struct MoveFile {
    pub revision_comments: String,
    pub site_id: i64,
    pub file_id: i64,
    pub user_id: i64,
    pub name: Option<String>,
    pub current_page_id: i64,
    pub destination_page_id: i64,
}

pub type MoveFileOutput = CreateFileRevisionOutput;

#[derive(Deserialize, Debug, Clone)]
pub struct DeleteFile<'a> {
    pub revision_comments: String,
    pub site_id: i64,
    pub page_id: i64,
    pub file: Reference<'a>,
    pub user_id: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RestoreFile {
    pub revision_comments: String,
    pub new_page_id: Option<i64>,
    pub new_name: Option<String>,
    pub site_id: i64,
    pub page_id: i64,
    pub file_id: i64,
    pub user_id: i64,
}

#[derive(Serialize, Debug, Clone)]
pub struct DeleteFileOutput {
    pub file_id: i64,
    pub file_revision_id: i64,
    pub file_revision_number: i32,
}

#[derive(Serialize, Debug, Clone)]
pub struct RestoreFileOutput {
    pub page_id: i64,
    pub file_id: i64,
    pub name: String,
    pub file_revision_id: i64,
    pub file_revision_number: i32,
}
