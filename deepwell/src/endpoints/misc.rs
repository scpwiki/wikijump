/*
 * endpoints/misc.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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
use wikidot_normalize::normalize;

pub async fn ping(req: ApiRequest) -> ApiResponse {
    tide::log::info!("Ping request");

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
    tide::log::info!("Getting DEEPWELL version");
    Ok(info::VERSION.as_str().into())
}

pub async fn full_version(_: ApiRequest) -> ApiResponse {
    tide::log::info!("Getting DEEPWELL version (full)");
    Ok(info::FULL_VERSION.as_str().into())
}

pub async fn hostname(_: ApiRequest) -> ApiResponse {
    tide::log::info!("Getting DEEPWELL hostname");
    Ok(info::HOSTNAME.as_str().into())
}

pub async fn config_dump(req: ApiRequest) -> ApiResponse {
    tide::log::info!("Dumping raw DEEPWELL configuration for debugging");
    let toml_config = &req.state().config.raw_toml;
    let mut body = Body::from_string(str!(toml_config));
    body.set_mime("text/toml;charset=utf-8");
    Ok(body.into())
}

pub async fn normalize_method(req: ApiRequest) -> ApiResponse {
    let input = req.param("input")?;
    tide::log::info!("Running normalize as utility web method: {input}");

    let mut value = str!(input);
    normalize(&mut value);
    Ok(value.into())
}
