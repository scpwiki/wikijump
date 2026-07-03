/*
 * error/object.rs
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

use super::ErrorType;
use serde_json::Value as JsonValue;
use std::error::Error as StdError;
use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
    pub error_type: ErrorType,
}

impl StdError for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:04}] {}", self.code(), self.message)
    }
}

impl Error {
    #[inline]
    pub fn new<S: Into<String>>(message: S, error_type: ErrorType) -> Self {
        Error {
            message: message.into(),
            error_type,
        }
    }

    /// Returns a unique integer code for this type of error.
    ///
    /// See `ErrorType::code()` for details.
    #[inline]
    pub fn code(&self) -> i32 {
        self.error_type.code()
    }

    /// Returns a basic summary of what this error is meant to represent.
    ///
    /// See `ErrorType::summary()` for details.
    #[inline]
    pub fn summary(&self) -> &'static str {
        self.error_type.summary()
    }

    /// Returns auxiliary data for this error.
    ///
    /// See `ErrorType::data()` for details.
    #[inline]
    pub fn data(&self) -> JsonValue {
        self.error_type.data()
    }
}

#[test]
fn error_object_delegates_to_error_type() {
    let error = Error::new("missing page", ErrorType::PageNotFound);

    assert_eq!(error.message, "missing page");
    assert_eq!(error.code(), ErrorType::PageNotFound.code());
    assert_eq!(error.summary(), ErrorType::PageNotFound.summary());
    assert_eq!(error.data(), ErrorType::PageNotFound.data());
    assert_eq!(error.to_string(), "[2005] missing page");
}
