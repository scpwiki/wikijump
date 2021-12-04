/*
 * methods/mod.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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

mod defaults {
    //! Helper module containing functions which return values.
    //!
    //! This is used in the [`#[serde(default = "function_name")]`](https://serde.rs/field-attrs.html)
    //! attribute to define optional parameter values.

    #[inline]
    pub const fn bool_true() -> bool {
        true
    }
}

mod prelude {
    pub use super::defaults::*;
    pub use crate::api::{ApiRequest, ApiResponse};
    pub use crate::types::*;
    pub use tide::{Body, Request};
}

pub mod user;
