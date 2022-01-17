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

mod prelude {
    pub use super::context::ServiceContext;
    pub use super::error::*;
    pub use crate::utils::{now, now_naive};
    pub use crate::web::{ProvidedValue, Reference};
    pub use sea_orm::{
        ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait,
        PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
    };
}

mod context;
mod error;

pub mod link;
pub mod page;
pub mod render;
pub mod revision;
pub mod text;
pub mod user;

use crate::api::ApiRequest;
use sea_orm::DatabaseConnection;

pub use self::context::ServiceContext;
pub use self::error::*;
pub use self::link::LinkService;
pub use self::page::PageService;
pub use self::render::RenderService;
pub use self::revision::RevisionService;
pub use self::text::TextService;
pub use self::user::UserService;

/// Extension trait to retrieve service objects from an `ApiRequest`.
pub trait RequestFetchService {
    fn database(&self) -> &DatabaseConnection;
}

impl RequestFetchService for ApiRequest {
    #[inline]
    fn database(&self) -> &DatabaseConnection {
        &self.state().database
    }
}
