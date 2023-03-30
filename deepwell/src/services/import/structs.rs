/*
 * services/import/structs.rs
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

use crate::utils::DateTimeWithTimeZone;
use chrono::NaiveDate;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportUser {
    pub user_id: i64,
    pub created_at: DateTimeWithTimeZone,
    pub name: String,
    pub slug: String,
    pub email: String,
    pub locale: String,
    pub avatar: Option<Vec<u8>>,
    pub real_name: Option<String>,
    pub gender: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub location: Option<String>,
    pub biography: Option<String>,
    pub user_page: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportSite {
    pub site_id: i64,
    pub created_at: DateTimeWithTimeZone,
    pub name: String,
    pub slug: String,
    pub locale: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportPage {
    pub page_id: i64,
    pub site_id: i64,
    pub created_at: DateTimeWithTimeZone,
    pub slug: String,
    pub discussion_thread_id: Option<i64>,
}
