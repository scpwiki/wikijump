/*
 * methods/misc.rs
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

use super::prelude::*;
use crate::info;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

pub async fn ping(req: ApiRequest) -> ApiResponse {
    // Ensure the database is connected
    req.state()
        .database
        .execute(Statement::from_string(
            DatabaseBackend::Postgres,
            str!("SELECT 1"),
        ))
        .await?;

    // Seems good, respond to user
    Ok("Pong!".into())
}

pub async fn version(_: ApiRequest) -> ApiResponse {
    Ok(info::VERSION_WITH_NAME.as_str().into())
}

pub async fn full_version(_: ApiRequest) -> ApiResponse {
    Ok(info::FULL_VERSION_WITH_NAME.as_str().into())
}

pub async fn ratelimit_exempt(req: ApiRequest) -> ApiResponse {
    todo!()
}
