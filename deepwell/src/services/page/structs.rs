/*
 * services/page/struct.rs
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
use crate::services::page_revision::CreatePageRevisionOutput;
use crate::services::score::ScoreValue;
use crate::types::{PageDetails, PageId, PageRevisionType};
use ftml::layout::Layout;
use ftml::parsing::ParseError;
use std::net::IpAddr;
use time::OffsetDateTime;

#[derive(Deserialize, Debug, Clone)]
pub struct CreatePage {
    pub site_id: i64,
    pub wikitext: String,
    pub title: String,
    pub alt_title: Option<String>,

    #[serde(default)]
    pub tags: Vec<String>,

    pub slug: String,
    pub layout: Option<Layout>,
    pub revision_comments: String,
    pub user_id: i64,

    #[serde(default)]
    pub bypass_filter: bool,

    pub ip_address: IpAddr,
}

#[derive(Serialize, Debug, Clone)]
pub struct CreatePageOutput {
    pub page_id: i64,
    pub slug: String,
    pub revision_id: i64,
    pub parser_errors: Vec<ParseError>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetPageReference<'a> {
    pub site_id: i64,
    pub page: Reference<'a>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetPageReferenceDetails<'a> {
    pub site_id: i64,
    pub page: Reference<'a>,

    #[serde(default)]
    pub details: PageDetails,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetPageSlug {
    pub site_id: i64,
    pub slug: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetPageAnyDetails {
    pub site_id: i64,
    pub page_id: i64,

    #[serde(default)]
    pub allow_deleted: bool,

    #[serde(default)]
    pub details: PageDetails,
}

#[derive(Serialize, Debug, Clone)]
pub struct GetPageOutput {
    pub page_id: i64,

    #[serde(with = "time::serde::rfc3339")]
    pub page_created_at: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339::option")]
    pub page_updated_at: Option<OffsetDateTime>,

    #[serde(with = "time::serde::rfc3339::option")]
    pub page_deleted_at: Option<OffsetDateTime>,
    pub page_revision_count: i32,
    pub site_id: i64,
    pub page_category_id: i64,
    pub page_category_slug: String,
    pub discussion_thread_id: Option<i64>,
    pub revision_id: i64,
    pub revision_type: PageRevisionType,

    #[serde(with = "time::serde::rfc3339")]
    pub revision_created_at: OffsetDateTime,
    pub revision_number: i32,
    pub revision_user_id: i64,
    pub wikitext: Option<String>,
    pub compiled_body_html: Option<String>,
    pub compiled_body_styles: Option<Vec<String>>,

    #[serde(with = "time::serde::rfc3339")]
    pub compiled_at: OffsetDateTime,
    pub compiled_generator: String,
    pub revision_comments: String,
    pub hidden_fields: Vec<String>,
    pub title: String,
    pub alt_title: Option<String>,
    pub slug: String,
    pub tags: Vec<String>,
    pub rating: ScoreValue,
    pub layout: Layout,
}

#[derive(Serialize, Debug, Clone)]
pub struct GetDeletedPageOutput {
    pub page_id: i64,

    #[serde(with = "time::serde::rfc3339")]
    pub page_created_at: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339::option")]
    pub page_updated_at: Option<OffsetDateTime>,

    #[serde(with = "time::serde::rfc3339")]
    pub page_deleted_at: OffsetDateTime,
    pub page_revision_count: i32,
    pub site_id: i64,
    pub discussion_thread_id: Option<i64>,
    pub hidden_fields: Vec<String>,
    pub title: String,
    pub alt_title: Option<String>,
    pub slug: String,
    pub tags: Vec<String>,
    pub rating: ScoreValue,
}

#[derive(Serialize, Debug, Clone)]
pub struct GetPageScoreOutput {
    pub page_id: i64,
    pub score: ScoreValue,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EditPage<'a> {
    pub site_id: i64,
    pub page: Reference<'a>,
    pub last_revision_id: i64,
    pub revision_comments: String,
    pub user_id: i64,

    #[serde(flatten)]
    pub body: EditPageBody,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct EditPageBody {
    pub wikitext: Maybe<String>,
    pub title: Maybe<String>,
    pub alt_title: Maybe<Option<String>>,
    pub tags: Maybe<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MovePage<'a> {
    pub site_id: i64,
    pub page: Reference<'a>,
    pub last_revision_id: i64,
    pub new_slug: String,
    pub revision_comments: String,
    pub user_id: i64,
    // NOTE: slug field is a parameter, not in the body
    pub ip_address: IpAddr,
}

#[derive(Serialize, Debug, Clone)]
pub struct MovePageOutput {
    pub old_slug: String,
    pub new_slug: String,
    pub revision_id: i64,
    pub revision_number: i32,
    pub parser_errors: Option<Vec<ParseError>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DeletePage<'a> {
    pub site_id: i64,
    pub page: Reference<'a>,
    pub last_revision_id: i64,
    pub revision_comments: String,
    pub user_id: i64,
    pub ip_address: IpAddr,
}

#[derive(Serialize, Debug, Clone)]
pub struct DeletePageOutput {
    page_id: i64,
    revision_id: i64,
    revision_number: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RestorePage {
    pub site_id: i64,
    pub page_id: i64,
    pub revision_comments: String,
    pub user_id: i64,
    pub slug: Option<String>,
    pub ip_address: IpAddr,
}

#[derive(Serialize, Debug, Clone)]
pub struct RestorePageOutput {
    slug: String,
    revision_id: i64,
    revision_number: i32,
    parser_errors: Vec<ParseError>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RollbackPage<'a> {
    pub site_id: i64,
    pub page: Reference<'a>,
    pub last_revision_id: i64,
    pub revision_number: i32,
    pub revision_comments: String,
    pub user_id: i64,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct SetPageLayout {
    pub site_id: i64,
    pub page_id: i64,
    pub layout: Option<Layout>,
    pub user_id: i64,
    pub ip_address: IpAddr,
}

pub type EditPageOutput = CreatePageRevisionOutput;

impl From<(CreatePageRevisionOutput, i64)> for DeletePageOutput {
    #[inline]
    fn from(
        (
            CreatePageRevisionOutput {
                revision_id,
                revision_number,
                parser_errors,
            },
            page_id,
        ): (CreatePageRevisionOutput, i64),
    ) -> DeletePageOutput {
        // There's no reason to rerender on page deletion
        debug_assert!(
            parser_errors.is_none(),
            "Parser errors from deleted page revision",
        );

        DeletePageOutput {
            page_id,
            revision_id,
            revision_number,
        }
    }
}

impl From<(CreatePageRevisionOutput, String)> for RestorePageOutput {
    #[inline]
    fn from(
        (
            CreatePageRevisionOutput {
                revision_id,
                revision_number,
                parser_errors,
            },
            slug,
        ): (CreatePageRevisionOutput, String),
    ) -> RestorePageOutput {
        // We should always rerender on page restoration
        let parser_errors =
            parser_errors.expect("No parser warnings from deleted page revision");

        RestorePageOutput {
            slug,
            revision_id,
            revision_number,
            parser_errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_page_output_uses_tombstone_revision_metadata() {
        let output = DeletePageOutput::from((
            CreatePageRevisionOutput {
                revision_id: 50,
                revision_number: 4,
                parser_errors: None,
            },
            20,
        ));

        assert_eq!(output.page_id, 20);
        assert_eq!(output.revision_id, 50);
        assert_eq!(output.revision_number, 4);
    }

    #[test]
    fn restore_page_output_requires_rerender_parser_errors() {
        let output = RestorePageOutput::from((
            CreatePageRevisionOutput {
                revision_id: 51,
                revision_number: 5,
                parser_errors: Some(Vec::new()),
            },
            str!("restored-page"),
        ));

        assert_eq!(output.slug, "restored-page");
        assert_eq!(output.revision_id, 51);
        assert_eq!(output.revision_number, 5);
        assert!(output.parser_errors.is_empty());
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct PageEditPermissionOutput {
    pub can_edit: bool,
}
