/*
 * services/settings/mod.rs
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

//! The "settings" service.
//!
//! This enables convenient access of the various object-associated settings
//! found throughout the database schema.
//!
//! For instance, a site may have a particular setting, which can be fetched
//! here, or a setting may be distributed across multiple levels, such as
//! a page's layout, which by default inherits from the level above it.

#[allow(unused_imports)]
mod prelude {
    pub use super::super::prelude::*;
    pub use super::structs::*;
}

mod service;
mod structs;

pub use self::service::SettingsService;
pub use self::structs::*;
