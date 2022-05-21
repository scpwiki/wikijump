/*
 * main.rs
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

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

//! A web server to expose Wikijump operations via a versioned REST API.

#[macro_use]
extern crate futures;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate str_macro;

#[macro_use]
mod macros;

mod api;
mod config;
mod database;
mod hash;
mod info;
mod json_utils;
mod locales;
mod methods;
mod models;
mod services;
mod utils;
mod web;

use self::config::Config;
use anyhow::Result;

#[async_std::main]
async fn main() -> Result<()> {
    // Load the configuration so we can set up
    let config = Config::load();
    let socket_address = config.address;

    // Configure the logger
    if config.logger {
        tide::log::with_level(config.logger_level);
        tide::log::info!("Loaded server configuration:");
        config.log();

        color_backtrace::install();
    }

    // Run migrations, if enabled
    if config.run_migrations {
        database::migrate(&config.database_url).await?;
    }

    // Build server and run
    let app = api::build_server(config).await?;

    tide::log::info!("Built server. Listening...");
    app.listen(socket_address).await?;

    Ok(())
}
