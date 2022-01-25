/*
 * services/job/structs.rs
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

use std::borrow::Cow;

pub const JOB_TYPE_RERENDER_PAGES: &str = "rerender_pages";

#[derive(Serialize, Deserialize, Debug)]
pub struct RerenderPage {
    pub site_id: i64,
    pub page_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RerenderPagesJobData {
    pub ids: Vec<RerenderPage>,
}
