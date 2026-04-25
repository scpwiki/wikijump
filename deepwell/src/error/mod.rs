/*
 * error/mod.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
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

pub mod prelude {
    pub use super::{Error, ErrorType, ExnError, Result, StdError, StdResult};
    pub use crate::types::EnumConversionError;
    pub use exn::{OptionExt, ResultExt};
}

mod convert;
mod error_type;
mod object;

pub use self::{convert::*, error_type::ErrorType, object::Error};
pub use exn::{Exn, Result as ExnResult};
pub use std::error::Error as StdError;

pub type ExnError = Exn<Error>;
pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = ExnResult<T, Error>;
