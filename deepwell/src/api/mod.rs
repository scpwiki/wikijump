/*
 * api/mod.rs
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

//! All routes for the API.
//!
//! No top-level routes should exist, any methods should be available under its respective API
//! version prefix to avoid future issues with backwards compatibility.

use crate::config::Config;
use crate::web::ratelimit::GovernorMiddleware;
use tide::Server;

mod v0;
mod v1;

pub type ApiServerContext = ();
pub type ApiServer = Server<ApiServerContext>;

pub fn build_server(config: &Config) -> ApiServer {
    let mut app = tide::new();
    app.at("/api")
        .with(GovernorMiddleware::per_minute(
            config.rate_limit_per_minute,
            &config.rate_limit_secret,
        ))
        .nest({
            let mut api = tide::new();
            api.at("/v0").nest(v0::build());
            api.at("/v1").nest(v1::build());
            api
        });
    app
}
