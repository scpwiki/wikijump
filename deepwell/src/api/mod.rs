/*
 * api/mod.rs
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

//! All routes for the API.
//!
//! No top-level routes should exist, any methods should be available under its respective API
//! version prefix to avoid future issues with backwards compatibility.
//!
//! This module should only contain definitions for the web server such as its routes, and
//! not any of the implementations themselves. Those should be in the `methods` module.

use crate::config::Config;
use crate::database;
use crate::locales::Localizations;
use crate::services::blob::spawn_magic_thread;
use crate::services::job::JobRunner;
use anyhow::Result;
use s3::bucket::Bucket;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

mod internal;

pub type ApiServerState = Arc<ServerState>;
pub type ApiServer = tide::Server<ApiServerState>;
pub type ApiRequest = tide::Request<ApiServerState>;
pub type ApiResponse = tide::Result;

#[derive(Debug)]
pub struct ServerState {
    pub config: Config,
    pub database: DatabaseConnection,
    pub localizations: Localizations,
    pub s3_bucket: Bucket,
}

pub async fn build_server_state(config: Config) -> Result<ApiServerState> {
    // Connect to database
    tide::log::info!("Connecting to PostgreSQL database");
    let database = database::connect(&config.database_url).await?;

    // Load localization data
    tide::log::info!("Loading localization data");
    let localizations = Localizations::open(&config.localization_path).await?;

    // Create S3 bucket
    tide::log::info!("Opening S3 bucket");

    let s3_bucket = Bucket::new(
        &config.s3_bucket,
        config.s3_region.clone(),
        config.s3_credentials.clone(),
    )?;

    // Return server state
    Ok(Arc::new(ServerState {
        config,
        database,
        localizations,
        s3_bucket,
    }))
}

pub fn build_server(state: ApiServerState) -> ApiServer {
    macro_rules! new {
        () => {
            tide::Server::with_state(Arc::clone(&state))
        };
    }

    // Start job executor task
    JobRunner::spawn(&state);

    // Start MIME evaluator thread
    spawn_magic_thread();

    // Create server and add routes
    let mut app = new!();
    app.at("/api").nest({
        let mut api = new!();
        api.at("/vI").nest(internal::build(new!()));
        api
    });

    app
}
