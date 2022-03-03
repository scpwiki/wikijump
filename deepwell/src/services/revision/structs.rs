/*
 * services/revision/structs.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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
use ftml::parsing::ParseWarning;
use std::num::NonZeroI32;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRevision {
    pub user_id: i64,
    pub comments: String,

    #[serde(flatten)]
    pub body: CreateRevisionBody,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CreateRevisionBody {
    pub wikitext: ProvidedValue<String>,
    pub title: ProvidedValue<String>,
    pub alt_title: ProvidedValue<Option<String>>,
    pub slug: ProvidedValue<String>,
    pub tags: ProvidedValue<Vec<String>>,
    pub metadata: ProvidedValue<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFirstRevision {
    pub user_id: i64,
    pub comments: String,
    pub wikitext: String,
    pub title: String,
    pub alt_title: Option<String>,
    pub slug: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRevisionOutput {
    pub revision_id: i64,
    pub revision_number: i32,
    pub parser_warnings: Option<Vec<ParseWarning>>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFirstRevisionOutput {
    pub revision_id: i64,
    pub parser_warnings: Vec<ParseWarning>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRevision {
    pub user_id: i64,
    pub hidden: Vec<String>,
}

/// Information about the revisions currently associated with a page.
///
/// A lot of this information is not strictly necessary:
/// * The first revision number is always `0`.
/// * The last revision number is always the total count of revisions minus one.
///
/// However it's convenient so that we don't have to do these calculations inline
/// in other places, and also so that API consumers have the relevant information.
#[derive(Serialize, Debug)]
pub struct RevisionCountOutput {
    pub revision_count: NonZeroI32,
    pub first_revision: i32,
    pub last_revision: i32,
}
