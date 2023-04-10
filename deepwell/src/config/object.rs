/*
 * config/object.rs
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

use super::args;
use super::file::ConfigFile;
use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, process};
use tide::log::LevelFilter;

/// Primary configuration structure.
///
/// See `config/file.rs` for an explanation of the
/// structure that is parsed from disk.
#[derive(Debug, Clone)]
pub struct Config {
    /// The raw TOML data that was read on server load.
    pub raw_toml: String,

    /// Whether the logger should be enabled or not.
    /// Also enables colorful backtraces.
    pub logger: bool,

    /// What log level to use during execution.
    ///
    /// One of:
    /// * `off`
    /// * `error`
    /// * `warn`
    /// * `info`
    /// * `debug`
    /// * `trace`
    pub logger_level: LevelFilter,

    /// The address the server will be hosted on.
    pub address: SocketAddr,

    /// The PID file (if any) to write to on boot.
    pub pid_file: Option<PathBuf>,

    /// Whether to run migrations on startup.
    pub run_migrations: bool,

    /// Whether to run the seeder on startup.
    /// This will only attempt to add the rows if the `user` table is empty.
    pub run_seeder: bool,

    /// The location where all the seeder files are kept.
    pub seeder_path: PathBuf,

    /// The location where all Fluent translation files are kept.
    pub localization_path: PathBuf,

    /// How long to allow a render job to run before terminating it.
    ///
    /// This is to ensure that a parser bug or malicious input cannot
    /// crash or freeze the backend. This value should not be too
    /// aggressive, but still not extremely long.
    pub render_timeout: Duration,
}

/*
TODO remove
impl Default for Config {
    fn default() -> Self {
        Config {
            logger: true,
            logger_level: LevelFilter::Info,
            address: "[::]:2747".parse().unwrap(),
            pid_file: None,
            run_migrations: true,
            run_seeder: true,
            seeder_path: PathBuf::from("seeder"),
            localization_path: PathBuf::from("../locales"),
            render_timeout: Duration::from_millis(2000),
        }
    }
}
*/

impl Config {
    #[inline]
    pub fn load(path: &Path) -> Result<Self> {
        let (config_file, raw_toml) = ConfigFile::load(path)?;
        let config = ConfigFile::to_config(config_file, raw_toml);
        Ok(config)
    }

    pub fn log(&self) {
        #[inline]
        fn bool_str(value: bool) -> &'static str {
            if value {
                "enabled"
            } else {
                "disabled"
            }
        }

        tide::log::info!("Configuration details:");
        tide::log::info!("Serving on {}", self.address);
        tide::log::info!("Migrations: {}", bool_str(self.run_migrations));
        tide::log::info!("Seeder: {}", bool_str(self.run_seeder));
        tide::log::info!("Localization path: {}", self.localization_path.display());
        tide::log::info!("Seeder path: {}", self.seeder_path.display());
        tide::log::info!(
            "Current working directory: {}",
            env::current_dir()
                .expect("Cannot get current working directory")
                .display(),
        );
    }
}
