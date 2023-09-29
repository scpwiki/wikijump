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
#![allow(clippy::large_enum_variant)]

//! A web server to expose Wikijump operations via an internal REST API.

#[macro_use]
extern crate futures;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate str_macro;

#[macro_use]
mod macros;

#[cfg(feature = "watch")]
mod watch;

mod api;
mod config;
mod constants;
mod database;
mod endpoints;
mod hash;
mod info;
mod locales;
mod models;
mod services;
mod utils;
mod web;

#[cfg(feature = "notify")]
use self::watch::setup_autorestart;

use self::config::SetupConfig;
use anyhow::Result;
use cfg_if::cfg_if;
use std::fs::File;
use std::io::Write;
use std::process;

#[async_std::main]
async fn main() -> Result<()> {
    // Load the configuration so we can set up
    let SetupConfig { secrets, config } = SetupConfig::load();

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

    // Write PID file, if enabled
    if let Some(ref path) = config.pid_file {
        tide::log::info!(
            "Writing process ID ({}) to {}",
            process::id(),
            path.display(),
        );

        let mut file = File::create(path)?;
        writeln!(&mut file, "{}", process::id())?;
    }

    // Set up restart-on-config change (if feature enabled)
    #[cfg(feature = "watch")]
    let _watcher;

    if config.watch_files {
        cfg_if! {
            if #[cfg(feature = "watch")] {
                _watcher = setup_autorestart(&config)?;
            } else {
                tide::log::error!("The --watch-files option requires the 'watch' feature");
                process::exit(1);
            }
        }
    }

    // Run migrations, if enabled
    if run_migrations {
        database::migrate(&secrets.database_url).await?;
    }

    // Set up server state
    let app_state = api::build_server_state(config, secrets).await?;

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
