/*
 * services/mod.rs
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

//! The "services" module, providing low-level logical operations.
//!
//! Each service is named for a particular object or concept, and
//! provides several low-level methods for interacting with it.
//! This may be CRUD, or small operations which should be composed
//! into larger ones.
//!
//! As such, _all methods here are not contained in transactions,_
//! the expectation is that the caller will use transactions when needed.
//! For methods which make multiple calls, they will assert that they
//! are currently in a transaction, if you are not then they will raise
//! an error.

#[macro_use]
mod macros;

mod prelude {
    pub use super::error::*;
    pub use crate::api::ApiServerState;
    pub use crate::types::Maybe;
    pub use crate::utils::now;
    pub use crate::web::ItemReference;
    pub use sea_orm::{
        ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set,
    };
}

mod error;

pub mod page;
pub mod user;

use self::page::PageService;
use self::user::UserService;
use crate::api::ApiRequest;

pub use self::error::*;

/// Extension trait to retrieve service objects from an `ApiRequest`.
pub trait RequestFetchService {
    fn page(&self) -> PageService;
    fn user(&self) -> UserService;
}

impl RequestFetchService for ApiRequest {
    #[inline]
    fn page(&self) -> PageService {
        self.into()
    }

    #[inline]
    fn user(&self) -> UserService {
        self.into()
    }
}
