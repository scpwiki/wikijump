/*
 * web/page_order.rs
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

use crate::models::page;

/// Describes what order pages should be retrieved in.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PageOrder {
    /// Requests pages in page ID order.
    /// This is the default.
    Id,

    /// Requests pages in page creation order.
    /// For most purposes this is the same as `PageOrder::Id`.
    Creation,

    /// Requests pages in page update order.
    /// For most purposes this is the same as `PageOrder::Id`.
    Update,

    /// Requests pages in slug order.
    Slug,
}

impl Default for PageOrder {
    #[inline]
    fn default() -> Self {
        PageOrder::Id
    }
}

/// Conversion functions for PageOrder to a column.
impl From<PageOrder> for page::Column {
    fn from(order: PageOrder) -> page::Column {
        match order {
            PageOrder::Id => page::Column::PageId,
            PageOrder::Creation => page::Column::CreatedAt,
            PageOrder::Update => page::Column::UpdatedAt,
            PageOrder::Slug => page::Column::Slug,
        }
    }
}
