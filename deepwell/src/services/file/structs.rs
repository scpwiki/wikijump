/*
 * services/file/structs.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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
};
use crate::utils::DateTimeWithTimeZone;
use crate::web::{ProvidedValue, Reference};
use serde_json::Value as JsonValue;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFile {
    pub revision_comments: String,
    pub name: String,
    pub user_id: i64,
    pub licensing: JsonValue, // TODO

    #[serde(default)]
    pub bypass_filter: bool,
}

pub type CreateFileOutput = CreateFirstFileRevisionOutput;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFile<'a> {
    pub site_id: i64,
    pub page_id: i64,
    pub file: Reference<'a>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFile {
    pub revision_comments: String,
    pub user_id: i64,

    #[serde(flatten)]
    pub body: UpdateFileBody,

    #[serde(default)]
    pub bypass_filter: bool,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateFileBody {
    pub name: ProvidedValue<String>,
    pub data: ProvidedValue<Vec<u8>>,
    pub licensing: ProvidedValue<serde_json::Value>,
}

pub type UpdateFileOutput = CreateFileRevisionOutput;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MoveFile {
    pub revision_comments: String,
    pub user_id: i64,
    pub name: Option<String>,
    pub current_page_id: i64,
    pub destination_page_id: i64,
}

pub type MoveFileOutput = CreateFileRevisionOutput;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFileOutput<'a> {
    pub file_id: i64,
    pub file_created_at: DateTimeWithTimeZone,
    pub file_updated_at: Option<DateTimeWithTimeZone>,
    pub file_deleted_at: Option<DateTimeWithTimeZone>,
    pub page_id: i64,
    pub revision_id: i64,
    pub revision_type: FileRevisionType,
    pub revision_created_at: DateTimeWithTimeZone,
    pub revision_number: i32,
    pub revision_user_id: i64,
    pub name: &'a str,
    pub data: Option<Vec<u8>>,
    pub mime: &'a str,
    pub size: i64,
    pub licensing: &'a JsonValue,
    pub revision_comments: &'a str,
    pub hidden_fields: &'a [String],
}

#[derive(Debug)]
pub struct DeleteFile {
    pub revision_comments: String,
    pub site_id: i64,
    pub user_id: i64,
}

#[derive(Debug)]
pub struct RestoreFile {
    pub revision_comments: String,
    pub new_page_id: Option<i64>,
    pub new_name: Option<String>,
    pub site_id: i64,
    pub user_id: i64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFileOutput {
    pub file_id: i64,
    pub file_revision_id: i64,
    pub file_revision_number: i32,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreFileOutput {
    pub page_id: i64,
    pub file_id: i64,
    pub name: String,
    pub file_revision_id: i64,
    pub file_revision_number: i32,
}
