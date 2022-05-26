/*
 * services/blob/mod.rs
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

//! The blob service, for interfacing with content-addressable S3 objects.
//!
//! This is essentially just a wrapper for how DEEPWELL interacts with S3.
//! Method implementations should instead work with the relevant concept
//! service instead, for instance the `FileService`.

mod prelude {
    pub use super::super::prelude::*;
    pub use super::mime_type;
    pub use super::structs::*;
    pub use crate::hash::{hash_to_hex, sha512_hash, Hash};
}

mod mime;
mod service;
mod structs;

pub use self::mime::{mime_type, spawn_magic_thread};
pub use self::service::BlobService;
pub use self::structs::*;
