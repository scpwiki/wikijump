/*
 * main.rs
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

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

//! A web server to expose Wikijump operations via a versioned REST API.

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde;

mod api;
mod config;
mod locales;
mod methods;
mod services;
mod web;

use self::config::Config;
use anyhow::Result;

#[async_std::main]
async fn main() -> Result<()> {
    let config = Config::load();
    let socket_address = config.address;

    if config.logger {
        tide::log::start();
        tide::log::info!("Loaded server configuration");
    }

    let app = api::build_server(config).await?;
    app.listen(socket_address).await?;

    Ok(())
}
