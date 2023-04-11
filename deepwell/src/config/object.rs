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

use super::file::ConfigFile;
use anyhow::Result;
use chrono::Duration as ChronoDuration;
use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tide::log::LevelFilter;

/// Primary configuration structure.
///
/// * See `config/file.rs` for the structure as parsed from disk.
/// * See `config.example.toml` for an explanation of all these fields.
#[derive(Debug, Clone)]
pub struct Config {
    /// The raw TOML data that was read on server load.
    pub raw_toml: String,

    /// Whether the logger should be enabled or not.
    /// Also enables colorful backtraces.
    pub logger: bool,

    /// What log level to use during execution.
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

    /// The duration to sleep after failed authentication attempts.
    pub authentication_fail_delay: Duration,

    /// Fixed prefix for all session tokens.
    pub session_token_prefix: String,

    /// Length of randomly-generated segment in session tokens.
    pub session_token_length: usize,

    /// How long normal sessions last before expiry.
    pub normal_session_duration: ChronoDuration,

    /// How long restricted sessions last before expiry.
    pub restricted_session_duration: ChronoDuration,

    /// The number of recovery codes to have per user.
    pub recovery_code_count: usize,

    /// Length of randomly-generated segment in recovery codes.
    pub recovery_code_length: usize,

    /// Length in seconds that each TOTP lasts.
    pub totp_time_step: u64,

    /// How much leniency should be allowed for TOTP.
    pub totp_time_skew: i64,

    /// How long to sleep in between job loops.
    pub job_delay: Duration,

    /// How often to run the "prune expired sessions" recurring job.
    pub job_prune_session_period: Duration,

    /// Maximum run time for a render request.
    pub render_timeout: Duration,

    /// Default name changes per user.
    pub default_name_changes: i16,

    /// Maximum name changes per user.
    pub max_name_changes: i16,

    /// How long until a user gets another name change token.
    pub refill_name_change: Duration,
}

impl Config {
    #[inline]
    pub fn load(path: &Path) -> Result<Self> {
        let (config_file, raw_toml) = ConfigFile::load(path)?;
        let config = ConfigFile::into_config(config_file, raw_toml);
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
