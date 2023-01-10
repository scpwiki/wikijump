/*
 * main.rs
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
mod constants;
mod database;
mod hash;
mod info;
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

    // Copy fields we need
    let socket_address = config.address;
    let run_migrations = config.run_migrations;
    let run_seeder = config.run_seeder;

    // Configure the logger
    if config.logger {
        tide::log::with_level(config.logger_level);
        tide::log::info!("Loaded server configuration:");
        config.log();

        color_backtrace::install();
    }

    // Run migrations, if enabled
    if run_migrations {
        database::migrate(&config.database_url).await?;
    }

    // Set up server state
    let app_state = api::build_server_state(config).await?;

    // Run seeder, if enabled
    if run_seeder {
        database::seed(&app_state).await?;
    }

    // Build and run server
    tide::log::info!("Building server and listening...");
    let app = api::build_server(app_state);
    app.listen(socket_address).await?;

    Ok(())
}
