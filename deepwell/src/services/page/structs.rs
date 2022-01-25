/*
 * services/page/struct.rs
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
use crate::services::revision::CreateRevisionOutput;
use ftml::parsing::ParseWarning;

#[derive(Deserialize, Debug)]
pub struct CreatePage {
    pub wikitext: String,
    pub title: String,
    pub alt_title: Option<String>,
    pub slug: String,
    pub revision_comments: String,
    pub user_id: i64,
}

#[derive(Serialize, Debug)]
pub struct CreatePageOutput {
    pub page_id: i64,
    pub slug: String,
    pub revision_id: i64,
    pub parser_warnings: Vec<ParseWarning>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct EditPage {
    pub wikitext: ProvidedValue<String>,
    pub title: ProvidedValue<String>,
    pub alt_title: ProvidedValue<Option<String>>,
    pub tags: ProvidedValue<Vec<String>>,
    pub revision_comments: String,
    pub user_id: i64,
}

#[derive(Serialize, Debug)]
pub struct EditPageOutput {
    revision_id: i64,
    revision_number: i32,
    parser_warnings: Option<Vec<ParseWarning>>,
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
