/*
 * services/render/structs.rs
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

use crate::hash::TextHash;
use ftml::parsing::ParseError;
use ftml::render::html::HtmlOutput;
use time::OffsetDateTime;

#[derive(Serialize, Debug)]
pub struct RenderOutput {
    pub html_output: HtmlOutput,
    pub errors: Vec<ParseError>,
    pub compiled_hash: TextHash,

    #[serde(with = "time::serde::rfc3339")]
    pub compiled_at: OffsetDateTime,
    pub compiled_generator: String,
}

#[derive(Serialize, Debug)]
pub struct RenderPageOutput {
    pub html_output: HtmlOutput,
    pub errors: Vec<ParseError>,
    pub compiled_body_html_hash: TextHash,
    pub compiled_body_styles_hash: TextHash,
    pub compiled_top_bar_html_hash: Option<TextHash>,
    pub compiled_side_bar_html_hash: Option<TextHash>,

    #[serde(with = "time::serde::rfc3339")]
    pub compiled_at: OffsetDateTime,
    pub compiled_generator: String,
}

#[derive(Deserialize, Debug)]
pub struct WikidotListPagesFeedInput {
    pub site_id: i64,
    pub pagetype: Option<String>,
    pub category: Option<String>,
    pub tags: Option<String>,
    pub parent: Option<String>,
    pub created_by: Option<String>,
    pub rating: Option<String>,
    pub range: Option<String>,
}

#[derive(Clone, Serialize, Debug)]
pub struct WikidotListPagesFeedOutput {
    pub items: Vec<WikidotListPagesFeedItem>,
}

#[derive(Clone, Serialize, Debug)]
pub struct WikidotListPagesFeedItem {
    pub slug: String,
    pub title: String,

    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    pub body_html: String,
    pub created_by_html: String,
}
