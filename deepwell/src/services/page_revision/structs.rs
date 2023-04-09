/*
 * services/page_revision/structs.rs
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

use super::prelude::*;
use crate::models::sea_orm_active_enums::PageRevisionType;
use crate::utils::DateTimeWithTimeZone;
use crate::web::FetchDirection;
use ftml::parsing::ParseError;
use std::num::NonZeroI32;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreatePageRevision {
    pub user_id: i64,
    pub comments: String,

    #[serde(flatten)]
    pub body: CreatePageRevisionBody,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CreatePageRevisionBody {
    pub wikitext: ProvidedValue<String>,
    pub title: ProvidedValue<String>,
    pub alt_title: ProvidedValue<Option<String>>,
    pub slug: ProvidedValue<String>,
    pub tags: ProvidedValue<Vec<String>>,
}

#[derive(Debug)]
pub struct CreateFirstPageRevision {
    pub user_id: i64,
    pub comments: String,
    pub wikitext: String,
    pub title: String,
    pub alt_title: Option<String>,
    pub slug: String,
}

#[derive(Debug)]
pub struct CreateTombstonePageRevision {
    pub site_id: i64,
    pub page_id: i64,
    pub user_id: i64,
    pub comments: String,
}

#[derive(Debug)]
pub struct CreateResurrectionPageRevision {
    pub site_id: i64,
    pub page_id: i64,
    pub user_id: i64,
    pub comments: String,
    pub new_slug: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreatePageRevisionOutput {
    pub revision_id: i64,
    pub revision_number: i32,
    pub parser_errors: Option<Vec<ParseError>>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFirstPageRevisionOutput {
    pub revision_id: i64,
    pub parser_errors: Vec<ParseError>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetPageRevision {
    pub site_id: i64,
    pub page_id: i64,
    pub revision_id: i32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePageRevision {
    pub user_id: i64,
    pub hidden: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetPageRevisionRange {
    pub site_id: i64,
    pub page_id: i64,
    pub revision_number: i32,
    pub revision_direction: FetchDirection,
    pub revision_limit: u64,
}

/// Information about the revisions currently associated with a page.
///
/// A lot of this information is not strictly necessary:
/// * The first revision number is always `0`.
/// * The last revision number is always the total count of revisions minus one.
/// * If there is only one revision, then the first and last revision numbers are the same.
///
/// However it's convenient to avoid having to do these calculations inline
/// in other places, and also so that API consumers have the relevant information.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageRevisionCountOutput {
    pub revision_count: NonZeroI32,
    pub first_revision: i32,
    pub last_revision: i32,
}

#[derive(Serialize, Debug)]
pub struct PageRevisionModelFiltered {
    pub revision_id: i64,
    pub revision_type: PageRevisionType,
    pub created_at: DateTimeWithTimeZone,
    pub from_wikidot: bool,
    pub revision_number: i32,
    pub page_id: i64,
    pub site_id: i64,
    pub user_id: i64,
    pub changes: Vec<String>,
    pub wikitext: Option<String>,
    pub compiled_html: Option<String>,
    pub compiled_at: DateTimeWithTimeZone,
    pub compiled_generator: String,
    pub comments: Option<String>,
    pub hidden: Vec<String>,
    pub title: Option<String>,
    pub alt_title: Option<String>,
    pub slug: Option<String>,
    pub tags: Option<Vec<String>>,
}
