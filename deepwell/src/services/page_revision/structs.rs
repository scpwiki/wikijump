/*
 * services/page_revision/structs.rs
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

use super::prelude::*;
use crate::types::{FetchDirection, PageDetails, PageId, PageRevisionType};
use ftml::layout::Layout;
use ftml::parsing::ParseError;
use std::num::NonZeroI32;
use time::OffsetDateTime;

#[derive(Deserialize, Debug, Clone)]
pub struct CreatePageRevision {
    pub user_id: i64,
    pub comments: String,
    pub revision_type: PageRevisionType,

    #[serde(flatten)]
    pub body: CreatePageRevisionBody,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct CreatePageRevisionBody {
    pub wikitext: Maybe<String>,
    pub title: Maybe<String>,
    pub alt_title: Maybe<Option<String>>,
    pub slug: Maybe<String>,
    pub tags: Maybe<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreateFirstPageRevision {
    pub user_id: i64,
    pub comments: String,
    pub wikitext: String,
    pub title: String,
    pub alt_title: Option<String>,

    #[serde(default)]
    pub tags: Vec<String>,

    pub slug: String,
    pub layout: Option<Layout>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreateTombstonePageRevision {
    pub site_id: i64,
    pub page_id: i64,
    pub user_id: i64,
    pub comments: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreateResurrectionPageRevision {
    pub id: PageId,
    pub user_id: i64,
    pub comments: String,
    pub new_slug: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct CreatePageRevisionOutput {
    pub revision_id: i64,
    pub revision_number: i32,
    pub parser_errors: Option<Vec<ParseError>>,
}

#[derive(Serialize, Debug, Clone)]
pub struct CreateFirstPageRevisionOutput {
    pub revision_id: i64,
    pub parser_errors: Vec<ParseError>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetPageRevision {
    pub site_id: i64,
    pub page_id: i64,
    pub revision_number: i32,
}

#[derive(Debug, Clone)]
pub struct CountPageRevisions {
    pub site_id: i64,
    pub page_id: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetPageRevisionDetails {
    #[serde(flatten)]
    pub input: GetPageRevision,

    #[serde(default)]
    pub details: PageDetails,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdatePageRevision {
    pub site_id: i64,
    pub page_id: i64,
    pub revision_id: i64,
    pub user_id: i64,
    pub hidden: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdatePageRevisionDetails {
    #[serde(flatten)]
    pub input: UpdatePageRevision,

    #[serde(default)]
    pub details: PageDetails,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetPageRevisionRange {
    pub site_id: i64,
    pub page_id: i64,
    pub revision_number: i32,
    pub revision_direction: FetchDirection,
    pub limit: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetPageRevisionRangeDetails {
    #[serde(flatten)]
    pub input: GetPageRevisionRange,

    #[serde(default)]
    pub details: PageDetails,
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
#[derive(Serialize, Debug, Clone)]
pub struct PageRevisionCountOutput {
    pub revision_count: NonZeroI32,
    pub first_revision: i32,
    pub last_revision: i32,
}

#[derive(Serialize, Debug, Clone)]
pub struct PageRevisionModelFiltered {
    pub revision_id: i64,
    pub revision_type: PageRevisionType,

    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339::option")]
    pub updated_at: Option<OffsetDateTime>,
    pub from_wikidot: bool,
    pub revision_number: i32,
    pub page_id: i64,
    pub site_id: i64,
    pub user_id: i64,
    pub changes: Vec<String>,
    pub wikitext: Option<String>,
    pub compiled_body_html: Option<String>,
    pub compiled_body_styles: Option<Vec<String>>,
    pub compiled_top_bar_html: Option<String>,
    pub compiled_side_bar_html: Option<String>,

    #[serde(with = "time::serde::rfc3339")]
    pub compiled_at: OffsetDateTime,
    pub compiled_generator: String,
    pub comments: Option<String>,
    pub hidden: Vec<String>,
    pub title: Option<String>,
    pub alt_title: Option<String>,
    pub slug: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Encapsulates the kind of rerender job to be performed.
///
/// This affects both which fields are updated and how outdating is performed.
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum RerenderType {
    /// Rerender the page normally.
    ///
    /// This involves recompiling its main body source and navigation pages,
    /// as well as updating links and queueing any subsequent rerenders.
    #[serde(rename = "full")]
    #[default]
    Full,

    /// Rerender the page's navigation pages only.
    ///
    /// This rerenders the top and side bar navigation page data, if
    /// they are present for this page. Does not do any outdating.
    #[serde(rename = "nav")]
    NavigationOnly,
}
