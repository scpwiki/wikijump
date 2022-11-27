/*
 * services/system_filter/mod.rs
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

//! This service manages system-wide filters.
//!
//! These apply to all sites or across sites, such as filters
//! which prevent users with certain names from being registered.

mod prelude {
    pub use super::super::prelude::*;
    pub use super::structs::*;
}

mod service;
mod structs;

pub use self::service::SystemFilterService;
pub use self::structs::*;
