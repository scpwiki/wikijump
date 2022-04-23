/*
 * services/revision/structs.rs
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
use crate::json_utils::json_to_string_list;
use crate::models::page_revision::Model as PageRevisionModel;
use ftml::parsing::ParseWarning;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde_json::Value as JsonValue;
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
/// * If there is only one revision, then the first and last revision numbers are the same.
///
/// However it's convenient so that we don't have to do these calculations inline
/// in other places, and also so that API consumers have the relevant information.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RevisionCountOutput {
    pub revision_count: NonZeroI32,
    pub first_revision: i32,
    pub last_revision: i32,
}

#[derive(Serialize, Debug)]
pub struct PageRevisionModelFiltered {
    pub revision_id: i64,
    pub created_at: DateTimeWithTimeZone,
    pub revision_number: i32,
    pub page_id: i64,
    pub site_id: i64,
    pub user_id: i64,
    pub changes: Vec<String>,
    pub wikitext_hash: Option<Vec<u8>>,
    pub compiled_hash: Option<Vec<u8>>,
    pub compiled_at: DateTimeWithTimeZone,
    pub compiled_generator: String,
    pub comments: Option<String>,
    pub hidden: Vec<String>,
    pub title: Option<String>,
    pub alt_title: Option<String>,
    pub slug: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<JsonValue>,
}

impl From<PageRevisionModel> for PageRevisionModelFiltered {
    fn from(model: PageRevisionModel) -> PageRevisionModelFiltered {
        let PageRevisionModel {
            revision_id,
            created_at,
            revision_number,
            page_id,
            site_id,
            user_id,
            changes,
            wikitext_hash,
            compiled_hash,
            compiled_at,
            compiled_generator,
            comments,
            hidden,
            title,
            mut alt_title,
            slug,
            tags,
            metadata,
        } = model;

        // Convert string list fields
        let changes = json_to_string_list(&changes);
        let hidden = json_to_string_list(&hidden);
        let tags = json_to_string_list(&tags);

        // Strip hidden fields
        let mut wikitext_hash = Some(wikitext_hash);
        let mut compiled_hash = Some(compiled_hash);
        let mut comments = Some(comments);
        let mut title = Some(title);
        // alt-title is already Option and we're not doubling up
        let mut slug = Some(slug);
        let mut tags = Some(tags);
        let mut metadata = Some(metadata);

        for field in &hidden {
            // TODO hidden fields aren't standardized yet
            match field.as_str() {
                "wikitext" => wikitext_hash = None,
                "compiled" => compiled_hash = None,
                "comments" => comments = None,
                "title" => title = None,
                "alt_title" => alt_title = None,
                "slug" => slug = None,
                "tags" => tags = None,
                "metadata" => metadata = None,
                _ => panic!("Unknown field name in hidden: {}", field),
            }
        }

        PageRevisionModelFiltered {
            revision_id,
            created_at,
            revision_number,
            page_id,
            site_id,
            user_id,
            changes,
            wikitext_hash,
            compiled_hash,
            compiled_at,
            compiled_generator,
            comments,
            hidden,
            title,
            alt_title,
            slug,
            tags,
            metadata,
        }
    }
}
