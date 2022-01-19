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

//! Definitions of methods invoked by different
//!
//! This module contains functions defining various routes used by the web server,
//! so that they can be referred to and reused by name.
//!
//! The module should not contain any core business logic of its own, but should
//! be simple wrappers around the various service methods exposed by structures
//! in the `services` module.

mod prelude {
    pub use crate::api::{ApiRequest, ApiResponse};
    pub use crate::services::{
        Error as ServiceError, LinkService, PageService, PostTransactionToApiResponse,
        RenderService, RequestFetchService, RevisionService, ServiceContext, SiteService,
        TextService, UserService,
    };
    pub use crate::web::{utils::error_response, HttpUnwrap, Reference};
    pub use chrono::prelude::*;
    pub use sea_orm::ConnectionTrait;
    pub use std::convert::TryFrom;
    pub use tide::{Body, Error as TideError, Request, Response, StatusCode};

    pub fn exists_status(exists: bool) -> ApiResponse {
        if exists {
            Ok(Response::new(StatusCode::NoContent))
        } else {
            Ok(Response::new(StatusCode::NotFound))
        }
    }
}

pub mod locales;
pub mod misc;
pub mod page;
pub mod revision;
pub mod text;
pub mod user;
