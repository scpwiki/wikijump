/*
 * test/mod.rs
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

mod prelude {
    pub use super::suite::*;
    pub use super::{
        ADMIN_USER_ID, ANONYMOUS_USER_ID, AUTOMATIC_USER_ID, REGULAR_USER_ID, WWW_SITE_ID,
    };
    pub use async_std_test::async_test;
    pub use serde_json::{json, Value as JsonValue};
    pub use tide::{Result, StatusCode};
}

pub const WWW_SITE_ID: i64 = 1;

pub const ADMIN_USER_ID: i64 = 1;
#[allow(dead_code)]
pub const AUTOMATIC_USER_ID: i64 = 2;
#[allow(dead_code)]
pub const ANONYMOUS_USER_ID: i64 = 3;
pub const REGULAR_USER_ID: i64 = 4;

mod locale;
mod misc;
mod page;
mod revision;
mod suite;
