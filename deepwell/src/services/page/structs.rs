/*
 * services/page/struct.rs
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
use crate::models::sea_orm_active_enums::PageRevisionType;
use crate::services::revision::CreateRevisionOutput;
use ftml::parsing::ParseWarning;
use sea_orm::entity::prelude::DateTimeWithTimeZone;
use serde_json::Value as JsonValue;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreatePage {
    pub wikitext: String,
    pub title: String,
    pub alt_title: Option<String>,
    pub slug: String,
    pub revision_comments: String,
    pub user_id: i64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreatePageOutput {
    pub page_id: i64,
    pub slug: String,
    pub revision_id: i64,
    pub parser_warnings: Vec<ParseWarning>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetPageOutput<'a> {
    pub page_id: i64,
    pub page_created_at: DateTimeWithTimeZone,
    pub page_updated_at: Option<DateTimeWithTimeZone>,
    pub page_deleted_at: Option<DateTimeWithTimeZone>,
    pub page_revision_count: i32,
    pub site_id: i64,
    pub page_category_id: i64,
    pub page_category_slug: &'a str,
    pub discussion_thread_id: Option<i64>,
    pub revision_id: i64,
    pub revision_type: PageRevisionType,
    pub revision_created_at: DateTimeWithTimeZone,
    pub revision_number: i32,
    pub revision_user_id: i64,
    pub wikitext: Option<String>,
    pub compiled_html: Option<String>,
    pub compiled_at: DateTimeWithTimeZone,
    pub compiled_generator: &'a str,
    pub revision_comments: &'a str,
    pub hidden_fields: &'a JsonValue, // TODO: replace with &[&str]
    pub title: &'a str,
    pub alt_title: Option<&'a str>,
    pub slug: &'a str,
    pub tags: &'a JsonValue, // TODO: replace with &[&str]
    pub rating: f64,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct EditPage {
    pub wikitext: ProvidedValue<String>,
    pub title: ProvidedValue<String>,
    pub alt_title: ProvidedValue<Option<String>>,
    pub tags: ProvidedValue<Vec<String>>,
    pub revision_comments: String,
    pub user_id: i64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EditPageOutput {
    revision_id: i64,
    revision_number: i32,
    parser_warnings: Option<Vec<ParseWarning>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MovePage {
    pub revision_comments: String,
    pub user_id: i64,
    // NOTE: slug field is a parameter, not in the body
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MovePageOutput {
    pub old_slug: String,
    pub new_slug: String,
    pub revision_id: i64,
    pub revision_number: i32,
    pub parser_warnings: Option<Vec<ParseWarning>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeletePage {
    pub revision_comments: String,
    pub user_id: i64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestorePage {
    pub revision_comments: String,
    pub user_id: i64,
    pub slug: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeletePageOutput {
    page_id: i64,
    revision_id: i64,
    revision_number: i32,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestorePageOutput {
    slug: String,
    revision_id: i64,
    revision_number: i32,
    parser_warnings: Vec<ParseWarning>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RollbackPage {
    pub revision_comments: String,
    pub user_id: i64,
}

impl From<CreateRevisionOutput> for EditPageOutput {
    #[inline]
    fn from(
        CreateRevisionOutput {
            revision_id,
            revision_number,
            parser_warnings,
        }: CreateRevisionOutput,
    ) -> EditPageOutput {
        EditPageOutput {
            revision_id,
            revision_number,
            parser_warnings,
        }
    }
}

impl From<(CreateRevisionOutput, i64)> for DeletePageOutput {
    #[inline]
    fn from(
        (
            CreateRevisionOutput {
                revision_id,
                revision_number,
                parser_warnings,
            },
            page_id,
        ): (CreateRevisionOutput, i64),
    ) -> DeletePageOutput {
        // There's no reason to rerender on page deletion
        debug_assert!(
            parser_warnings.is_none(),
            "Parser warnings from deleted page revision",
        );

        DeletePageOutput {
            page_id,
            revision_id,
            revision_number,
        }
    }
}

impl From<(CreateRevisionOutput, String)> for RestorePageOutput {
    #[inline]
    fn from(
        (
            CreateRevisionOutput {
                revision_id,
                revision_number,
                parser_warnings,
            },
            slug,
        ): (CreateRevisionOutput, String),
    ) -> RestorePageOutput {
        // We should always rerender on page restoration
        let parser_warnings =
            parser_warnings.expect("No parser warnings from deleted page revision");

        RestorePageOutput {
            slug,
            revision_id,
            revision_number,
            parser_warnings,
        }
    }
}
