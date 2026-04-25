/*
 * main.rs
 *
 * Wilson's Web Server - Serves a zoo of user-generated content
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

//! A server to handle incoming web requests.
//!
//! Depending on the hostname, requests are routed to either framerail
//! or given to logic to serve wjfiles data.

#[macro_use]
extern crate str_macro;

#[macro_use]
extern crate tracing;

#[macro_use]
mod macros;

mod attachment;
mod cache;
mod config;
mod deepwell;
mod error;
mod fetch;
mod handler;
mod info;
mod language;
mod path;
mod range;
mod route;
mod state;
mod trace;

use self::{
    config::load_config, route::build_router, state::build_server_state,
    trace::setup_tracing,
};
use anyhow::Result;
use std::{fs::File, io::Write, process};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let (config, secrets) = load_config();

    // Set up tracing
    if config.enable_trace {
        setup_tracing();
    }

    // Write PID file
    if let Some(ref path) = config.pid_file {
        debug!(pid = process::id(), "Writing PID file");
        let mut file = File::create(path)?;
        writeln!(&mut file, "{}", process::id())?;
    }

    // Connect to services, build server state and then run
    let state = build_server_state(config.enable_deepwell_check, secrets).await?;
    let router = build_router(state);
    let app = router.into_make_service();

    // Begin listening
    info!(
        address = str!(config.address),
        "Listening to connections...",
    );

    let listener = TcpListener::bind(config.address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
