/*
 * services/filter/mod.rs
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

//! This service manages configurable filters.
//!
//! These encompass both platform filters (applies to all sites)
//! and site filters (applies only to a particular site).
//!
//! For instance, a user platform filter prevents a name from being
//! registered, where a user site filter would prevent the user from
//! joining.

#[allow(unused_imports)]
mod prelude {
    pub use super::super::prelude::*;
    pub use super::matcher::{FilterMatcher, FilterSummary};
    pub use super::structs::*;
}

mod matcher;
mod service;
mod structs;

pub use self::matcher::{FilterMatcher, FilterSummary};
pub use self::service::FilterService;
pub use self::structs::*;
