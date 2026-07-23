/*
 * services/category/structs.rs
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

use crate::services::settings::{
    PageRatingPermission, PageRatingType, PageRatingVisibility,
};
use crate::types::{Maybe, Reference};
use std::net::IpAddr;

#[derive(Deserialize, Debug, Clone)]
pub struct GetCategory<'a> {
    pub site: Reference<'a>,
    pub category: Reference<'a>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateCategory<'a> {
    pub site: Reference<'a>,
    pub category: Reference<'a>,
    pub user_id: i64,
    #[serde(flatten)]
    pub body: UpdateCategoryBody,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct UpdateCategoryBody {
    pub top_bar_page: Maybe<Option<String>>,
    pub side_bar_page: Maybe<Option<String>>,
    pub template_page_id: Maybe<Option<i64>>,
    pub license: Maybe<Option<String>>,
    pub license_other: Maybe<Option<String>>,
    pub rating_enabled: Maybe<Option<bool>>,
    pub rating_permission: Maybe<Option<PageRatingPermission>>,
    pub rating_visibility: Maybe<Option<PageRatingVisibility>>,
    pub rating_type: Maybe<Option<PageRatingType>>,
}
