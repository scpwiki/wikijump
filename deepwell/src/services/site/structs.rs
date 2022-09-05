/*
 * services/site/structs.rs
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

#[derive(Deserialize, Debug)]
pub struct CreateSite {
    pub slug: String,
    pub name: String,
    pub subtitle: String,
    pub description: String,
    pub locale: String, // Strum instead, maybe?
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSiteOutput {
    pub site_id: i64,
    pub slug: String,
}