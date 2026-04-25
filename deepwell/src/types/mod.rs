/*
 * types/mod.rs
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

#![allow(unused_imports)]

mod array;
mod bytes;
mod conversion_error;
mod enums;
mod fetch_direction;
mod file_details;
mod file_order;
mod maybe;
mod page_details;
mod page_id;
mod page_order;
mod permissions;
mod reference;
mod rerender_depth;

pub use self::{
    array::ArrayLength,
    bytes::Bytes,
    conversion_error::{EnumConversionError, parse_layout},
    enums::*,
    fetch_direction::FetchDirection,
    file_details::FileDetails,
    file_order::FileOrder,
    maybe::Maybe,
    page_details::PageDetails,
    page_id::PageId,
    page_order::PageOrder,
    permissions::Permission,
    reference::Reference,
    rerender_depth::RerenderDepth,
};
