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

mod prelude {
    pub use crate::api::{ApiRequest, ApiResponse};
    pub use crate::services::{
        Error as ServiceError, PostTransactionToApiResponse, RequestFetchService,
    };
    pub use crate::web::{utils::error_response, HttpUnwrap, ItemReference};
    pub use chrono::prelude::*;
    pub use sea_orm::ConnectionTrait;
    pub use std::convert::TryFrom;
    pub use tide::{Body, Error as TideError, Request, Response, StatusCode};
}

pub mod locales;
pub mod misc;
pub mod page;
pub mod user;
