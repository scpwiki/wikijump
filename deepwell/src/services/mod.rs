/*
 * services/mod.rs
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

//! The "services" module, providing low-level logical operations.
//!
//! Each service is named for a particular object or concept, and
//! provides several low-level methods for interacting with it.
//! This may be CRUD, or small operations which should be composed
//! into larger ones.
//!
//! As such, **all methods here are _not_ contained in transactions,**
//! the expectation is that the caller will use transactions when needed.
//! For methods which make multiple calls, they will assert that they
//! are currently in a transaction, if you are not then they will raise
//! an error.
//!
//! These methods are called as component operations either by other
//! services or by route implementations found in the `methods` module.

mod prelude {
    pub use super::context::ServiceContext;
    pub use super::error::*;
    pub use crate::utils::now;
    pub use crate::web::{ProvidedValue, Reference};
    pub use chrono::Utc;
    pub use sea_orm::{
        ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, JoinType,
        ModelTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
        Set,
    };
}

mod context;
mod error;

pub mod blob;
pub mod category;
pub mod file;
pub mod file_revision;
pub mod job;
pub mod link;
pub mod outdate;
pub mod page;
pub mod parent;
pub mod render;
pub mod revision;
pub mod score;
pub mod site;
pub mod text;
pub mod user;
pub mod vote;

use crate::api::ApiRequest;
use sea_orm::DatabaseConnection;

pub use self::blob::BlobService;
pub use self::category::CategoryService;
pub use self::context::ServiceContext;
pub use self::error::*;
pub use self::file::FileService;
pub use self::file_revision::FileRevisionService;
pub use self::job::JobService;
pub use self::link::LinkService;
pub use self::outdate::OutdateService;
pub use self::page::PageService;
pub use self::parent::ParentService;
pub use self::render::RenderService;
pub use self::revision::RevisionService;
pub use self::score::ScoreService;
pub use self::site::SiteService;
pub use self::text::TextService;
pub use self::user::UserService;
pub use self::vote::VoteService;

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
