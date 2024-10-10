/*
 * types/page_order.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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
use sea_orm::query::Order;

/// Describes what order pages should be retrieved in.
///
/// It is composed of two components:
/// * `column`    -- The `PageOrderColumn` describing what column to order by.
/// * `direction` -- Whether the order should be ascending or descending. (See [`Order`])
///
/// [`Order`]: https://docs.rs/sea-orm/latest/sea_orm/query/enum.Order.html
#[derive(Debug, Clone, PartialEq)]
pub struct PageOrder {
    pub column: PageOrderColumn,
    pub direction: Order,
}

impl Default for PageOrder {
    #[inline]
    fn default() -> Self {
        PageOrder {
            column: PageOrderColumn::default(),
            direction: Order::Asc,
        }
    }
}

/// Describes what column that pages should be ordered by.
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PageOrderColumn {
    /// Requests pages in page ID order.
    /// This is the default.
    #[default]
    Id,

    /// Requests pages in page creation order.
    /// For most purposes this is the same as `PageOrderColumn::Id`.
    Creation,

    /// Requests pages in page update order.
    Update,

    /// Requests pages in page deletion order.
    Deletion,

    /// Requests pages in slug order.
    Slug,
}

impl PageOrderColumn {
    #[inline]
    pub fn into_column(self) -> page::Column {
        self.into()
    }
}

/// Conversion functions for `PageOrder` to a column.
impl From<PageOrderColumn> for page::Column {
    fn from(order: PageOrderColumn) -> page::Column {
        match order {
            PageOrderColumn::Id => page::Column::PageId,
            PageOrderColumn::Creation => page::Column::CreatedAt,
            PageOrderColumn::Update => page::Column::UpdatedAt,
            PageOrderColumn::Deletion => page::Column::DeletedAt,
            PageOrderColumn::Slug => page::Column::Slug,
        }
    }
}
