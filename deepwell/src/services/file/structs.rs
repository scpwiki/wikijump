/*
 * services/file/structs.rs
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

use crate::models::file::Model as FileModel;
use crate::services::revision::CreateRevisionOutput;

#[derive(Debug)]
pub struct CreateFile {
    pub revision_comments: String,
    pub name: String,
    pub site_id: i64,
    pub page_id: i64,
    pub user_id: i64,
    pub licensing: serde_json::Value, // TODO
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileOutput {
    #[serde(flatten)]
    pub file: FileModel,

    #[serde(flatten)]
    pub revision: CreateRevisionOutput,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFileOutput {
    #[serde(flatten)]
    pub file: FileModel,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct DeleteFile {
    pub revision_comments: String,
    pub site_id: i64,
    pub page_id: i64,
    pub user_id: i64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFileOutput {
    #[serde(flatten)]
    pub file: FileModel,

    #[serde(flatten)]
    pub revision: CreateRevisionOutput,
}
