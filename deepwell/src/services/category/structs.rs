/*
 * services/category/structs.rs
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

use crate::models::page_category::Model as PageCategoryModel;
use crate::web::Reference;
use time::OffsetDateTime;

#[derive(Deserialize, Debug, Clone)]
pub struct GetCategory<'a> {
    pub site: Reference<'a>,
    pub category: Reference<'a>,
}

#[derive(Serialize, Debug, Clone)]
pub struct CategoryOutput {
    category_id: i64,
    created_at: OffsetDateTime,
    updated_at: Option<OffsetDateTime>,
    site_id: i64,
    slug: String,
}

impl From<PageCategoryModel> for CategoryOutput {
    #[inline]
    fn from(model: PageCategoryModel) -> CategoryOutput {
        let PageCategoryModel {
            category_id,
            created_at,
            updated_at,
            site_id,
            slug,
        } = model;

        CategoryOutput {
            category_id,
            created_at,
            updated_at,
            site_id,
            slug,
        }
    }
}
