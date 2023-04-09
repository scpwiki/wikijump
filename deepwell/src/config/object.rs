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

use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, process};
use tide::log::LevelFilter;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
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

    /// Whether to run migrations on startup.
    pub run_migrations: bool,

    /// Whether to run the seeder on startup.
    /// This will only attempt to add the rows if the `user` table is empty.
    pub run_seeder: bool,

    /// The location where all Fluent translation files are kept.
    pub localization_path: PathBuf,

    /// The location where all the seeder files are kept.
    pub seeder_path: PathBuf,

    /// How long to allow a render job to run before terminating it.
    ///
    /// This is to ensure that a parser bug or malicious input cannot
    /// crash or freeze the backend. This value should not be too
    /// aggressive, but still not extremely long.
    pub render_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            logger: true,
            logger_level: LevelFilter::Info,
            address: "[::]:2747".parse().unwrap(),
            run_migrations: true,
            run_seeder: true,
            localization_path: PathBuf::from("../locales"),
            seeder_path: PathBuf::from("seeder"),
            render_timeout: Duration::from_millis(2000),
        }
    }
}

impl Config {
    pub fn parse_args() -> Self {
        todo!()
    }

    pub fn load(path: &Path) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn dump(&self) -> Result<String> {
        tide::log::info!("Dumping current TOML configuration to string");

        let output = toml::to_string_pretty(self)?;
        Ok(output)
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
