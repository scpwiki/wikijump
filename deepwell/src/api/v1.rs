/*
 * api/v1.rs
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

//! Routes for version 1 of the API.
//!
//! Because it is not yet stabilized, it is a stub.

use super::ApiServer;
use crate::web::utils::error_response;
use tide::StatusCode;

pub fn build() -> ApiServer {
    let mut app = tide::new();
    app.at("/*").all(|_| async {
        error_response(
            StatusCode::NotImplemented,
            "API v1 has not yet been stablized",
        )
    });
    app
}
