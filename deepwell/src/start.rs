/*
 * start.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
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

//! Entrypoint to the server-based execution mode of deepwell.

#[cfg(feature = "notify")]
use crate::watch::setup_autorestart;

use crate::config::SetupConfig;
use crate::error::prelude::*;
use crate::{api, database};
use cfg_if::cfg_if;
use std::fs::File;
use std::io::Write;
use std::process;

fn start_logging(config: &crate::config::Config) {
    let logger_level = config.operational_logger_level();
    femme::with_level(logger_level);
    if config.logger_level == femme::LevelFilter::Trace {
        warn!(
            "Trace logging is disabled because dependency traces can expose authentication bearers"
        );
    }
}

pub async fn start() -> Result<()> {
    let SetupConfig { secrets, config } = SetupConfig::load().await;
    let address = config.address;
    let run_seeder = config.run_seeder;

    let make_error = || {
        Error::new(
            format!("failed to start deepwell server on {address} (seeder {run_seeder})"),
            ErrorType::ApplicationStart,
        )
    };

    if config.logger {
        start_logging(&config);
        info!("Loaded server configuration:");
        config.log();

        color_backtrace::install();
    }

    if let Some(ref path) = config.pid_file {
        info!(
            "Writing process ID ({}) to {}",
            process::id(),
            path.display(),
        );

        let mut file = File::create(path).or_raise(make_error)?;
        writeln!(&mut file, "{}", process::id()).or_raise(make_error)?;
    }

    #[cfg(feature = "watch")]
    let _watcher;

    if config.watch_files {
        cfg_if! {
            if #[cfg(feature = "watch")] {
                _watcher = setup_autorestart(&config).or_raise(make_error)?;
            } else {
                error!("The --watch-files option requires the 'watch' feature");
                process::exit(1);
            }
        }
    }

    let app_state = api::build_server_state(config, secrets)
        .await
        .or_raise(make_error)?;

    if run_seeder {
        database::seed(&app_state).await.or_raise(make_error)?;
    }

    info!("Building server...");
    let server = api::build_server(app_state).await.or_raise(make_error)?;

    info!("Listening to connections on {address}...");
    server.stopped().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_logging_caps_trace_at_debug() {
        let mut config = crate::config::Config::integration_testing();
        config.logger_level = femme::LevelFilter::Trace;

        start_logging(&config);

        assert_eq!(log::max_level(), femme::LevelFilter::Debug);
    }
}
