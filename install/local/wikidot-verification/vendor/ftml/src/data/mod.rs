/*
 * data/mod.rs
 *
 * ftml - Library to parse Wikidot text
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

//! This module defines POD (plain old data) structs.

mod backlinks;
mod karma;
mod page_info;
mod page_ref;
mod score;
mod user_info;

pub use self::backlinks::Backlinks;
pub use self::karma::KarmaLevel;
pub use self::page_info::PageInfo;
pub use self::page_ref::{PageRef, PageRefParseError};
pub use self::score::ScoreValue;
pub use self::user_info::UserInfo;
