/*
 * utils/mod.rs
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

//! Eclectic module containing various utilities, grouped by type.

mod category;
mod error;
mod json;
mod locale;
mod string;
mod tide;
mod time;
mod user;

pub use self::category::*;
pub use self::error::*;
pub use self::json::*;
pub use self::locale::*;
pub use self::string::*;
pub use self::tide::*;
pub use self::time::*;
pub use self::user::*;
