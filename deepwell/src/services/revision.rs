/*
 * services/revision.rs
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

// Helper structs
// TODO

#[derive(Deserialize, Debug)]
pub struct CreateRevisionInput {
    _page_id: i64,
    _site_id: i64,
    _user_id: i64,
    _comments: String,

    #[serde(default)]
    _wikitext: ProvidedValue<String>,

    #[serde(default)]
    _hidden: ProvidedValue<Vec<String>>,

    #[serde(default)]
    _title: ProvidedValue<String>,

    #[serde(default)]
    _alt_title: ProvidedValue<Option<String>>,

    #[serde(default)]
    _slug: ProvidedValue<String>,

    #[serde(default)]
    _tags: ProvidedValue<Vec<String>>,

    #[serde(default)]
    _metadata: ProvidedValue<serde_json::Value>,
}

#[derive(Serialize, Debug)]
pub struct CreateRevisionOutput {
    revision_id: i64,
    revision_number: i32,
}

// Service

#[derive(Debug)]
pub struct RevisionService;

impl RevisionService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        input: CreateRevisionInput,
    ) -> Result<Option<CreateRevisionOutput>> {
        let _todo = (ctx, input);

        todo!()
    }

    // TODO: add revision type
    pub async fn get_latest(_ctx: &ServiceContext<'_>, _page_id: i64, _site_id: i64) {}
}
