/*
 * services/link/mod.rs
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

//! This service updates the various kinds of backlinks supported in Wikijump.
//!
//! This includes "page connections" (a generic term for a relation between pages,
//! such as includes, links, redirects, etc), which can either be present or missing,
//! as well as external links, which are string URLs.
//!
//! Whenever a page is updated, a list of its backlinks is gathered by the parser,
//! which is then presented here for processing. A diff is needed:
//! * Any links no longer present are deleted.
//! * Any links not previously present are inesrted.
//! * Any links present but changed in quantity are updated.
//! * Else, the link is up-to-date and left alone.
//!
//! While the logic here is similar for each case, the slight differences in keys,
//! types, and tables make it hard to modularize. Instead, the logic is hopefully
//! clear enough to be acceptable when repeated over a few slightly distinct cases.

mod prelude {
    pub use super::super::prelude::*;
    pub use super::structs::*;
    pub use crate::web::ConnectionType;
}

#[macro_use]
mod macros;
mod service;
mod structs;

pub use self::service::LinkService;
pub use self::structs::*;
