/*
 * services/error.rs
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

use crate::api::ApiResponse;
use sea_orm::error::DbErr;
use thiserror::Error as ThisError;
use tide::{Error as TideError, StatusCode};

pub use std::error::Error as StdError;

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(DbErr),

    #[error("The request conflicts with data already present")]
    Conflict,

    #[error("The requested data was not found")]
    NotFound,
}

// Error conversions (most are automatic, see #[from])

impl From<DbErr> for Error {
    fn from(error: DbErr) -> Error {
        match error {
            DbErr::RecordNotFound(_) => Error::NotFound,
            _ => Error::Database(error),
        }
    }
}

// Conversion for web responses

impl From<Error> for ApiResponse {
    fn from(error: Error) -> ApiResponse {
        let tide_error = match error {
            Error::Database(inner) => {
                TideError::new(StatusCode::InternalServerError, inner)
            }
            Error::Conflict => TideError::from_str(StatusCode::Conflict, ""),
            Error::NotFound => TideError::from_str(StatusCode::NotFound, ""),
        };

        Err(tide_error)
    }
}
