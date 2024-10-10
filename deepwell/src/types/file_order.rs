/*
 * types/file_order.rs
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

use crate::models::file;
use sea_orm::query::Order;

/// Describes what order files should be retrieved in.
///
/// It is composed of two components:
/// * `column`    -- The `FileOrderColumn` describing what column to order by.
/// * `direction` -- Whether the order should be ascending or descending. (See [`Order`])
///
/// [`Order`]: https://docs.rs/sea-orm/latest/sea_orm/query/enum.Order.html
#[derive(Debug, Clone, PartialEq)]
pub struct FileOrder {
    pub column: FileOrderColumn,
    pub direction: Order,
}

impl Default for FileOrder {
    #[inline]
    fn default() -> Self {
        FileOrder {
            column: FileOrderColumn::default(),
            direction: Order::Asc,
        }
    }
}

/// Describes what column that files should be ordered by.
#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum FileOrderColumn {
    /// Requests files in file ID order.
    /// This is the default.
    #[default]
    Id,

    /// Requests files in file creation order.
    /// For most purposes this is the same as `FileOrderColumn::Id`.
    Creation,

    /// Requests files in file update order.
    Update,

    /// Requests pages in page deletion order.
    Deletion,

    /// Requests files in file name order.
    Name,
}

impl FileOrderColumn {
    #[inline]
    pub fn into_column(self) -> file::Column {
        self.into()
    }
}

/// Conversion functions for `FileOrder` to a column.
impl From<FileOrderColumn> for file::Column {
    fn from(order: FileOrderColumn) -> file::Column {
        match order {
            FileOrderColumn::Id => file::Column::FileId,
            FileOrderColumn::Creation => file::Column::CreatedAt,
            FileOrderColumn::Update => file::Column::UpdatedAt,
            FileOrderColumn::Deletion => file::Column::DeletedAt,
            FileOrderColumn::Name => file::Column::Name,
        }
    }
}
