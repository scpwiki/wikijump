/*
 * config/object.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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
use femme::LevelFilter;
use ftml::layout::Layout;
use std::env;
use std::net::SocketAddr;
use std::num::NonZeroU16;
use std::path::PathBuf;
use std::time::Duration as StdDuration;
use time::Duration as TimeDuration;

/// Primary configuration structure.
///
/// * See `config/file.rs` for the structure as parsed from disk.
/// * See `config.example.toml` for an explanation of all these fields.
#[derive(Debug, Clone)]
pub struct Config {
    /// The raw TOML data that was read on server load.
    pub raw_toml: String,

    /// The path where the above raw TOML data was read from.
    pub raw_toml_path: PathBuf,

    /// Whether the logger should be enabled or not.
    /// Also enables colorful backtraces.
    pub logger: bool,

    /// What log level to use during execution.
    pub logger_level: LevelFilter,

    /// The address the server will be hosted on.
    pub address: SocketAddr,

    /// The PID file (if any) to write to on boot.
    pub pid_file: Option<PathBuf>,

    /// The main domain to serve sites from.
    ///
    /// Always starts with a `.`
    pub main_domain: String,

    /// The main domain, but without a leading `.`
    pub main_domain_no_dot: String,

    /// The files domain to serve user-generated content from.
    ///
    /// Always starts with a `.`
    #[allow(dead_code)] // TEMP
    pub files_domain: String,

    /// The files domain, but without a leading `.`
    #[allow(dead_code)] // TEMP
    pub files_domain_no_dot: String,

    /// Whether to auto-restart on configuration file change.
    ///
    /// Currently watches:
    /// * Localization directory
    /// * Configuration file
    pub watch_files: bool,

    /// Whether to run the seeder on startup.
    /// This will only attempt to add the rows if the `user` table is empty.
    pub run_seeder: bool,

    /// The location where all the seeder files are kept.
    pub seeder_path: PathBuf,

    /// The location where all Fluent translation files are kept.
    pub localization_path: PathBuf,

    /// The duration to sleep after failed authentication attempts.
    pub authentication_fail_delay: StdDuration,

    /// Fixed prefix for all session tokens.
    pub session_token_prefix: String,

    /// Length of randomly-generated segment in session tokens.
    pub session_token_length: usize,

    /// How long normal sessions last before expiry.
    pub normal_session_duration: TimeDuration,

    /// How long restricted sessions last before expiry.
    pub restricted_session_duration: TimeDuration,

    /// The number of recovery codes to have per user.
    pub recovery_code_count: usize,

    /// Length of randomly-generated segment in recovery codes.
    pub recovery_code_length: usize,

    /// Length in seconds that each TOTP lasts.
    pub totp_time_step: u64,

    /// How much leniency should be allowed for TOTP.
    pub totp_time_skew: i64,

    /// The number of job workers to run in this process.
    pub job_workers: NonZeroU16,

    /// How many times to retry a job before deleting it anyways.
    pub job_max_attempts: u16,

    /// How long to sleep after finishing work on a job.
    pub job_work_delay: StdDuration,

    /// The minimum sleep time after polling an empty job queue.
    /// This uses exponential value starting at this value.
    pub job_min_poll_delay: StdDuration,

    /// The maximum sleep time after polling an empty job queue.
    /// This uses exponential value cappint out at this value.
    pub job_max_poll_delay: StdDuration,

    /// How often to run the "prune expired sessions" recurring job.
    pub job_prune_session: StdDuration,

    /// How often to run the "prune unused text" recurring job.
    pub job_prune_text: StdDuration,

    /// How often to run the "refill name change tokens" recurring job.
    pub job_name_change_refill: StdDuration,

    /// How often to run the "lift expired punishments" recurring job.
    pub job_lift_expired_punishments: StdDuration,

    /// Maximum run time for a render request.
    pub render_timeout: StdDuration,

    /// In what circumstances a page rerender should be skipped.
    ///
    /// A list of rerender job depths and durations. If any item in this
    /// list matches, then the rerender is skipped and subsequent rerender
    /// jobs are not proliferated.
    ///
    /// The condition means that the current job depth is equal or greater
    /// than the specified depth value, _and_ that the page revision was
    /// last updated in the duration value specified.
    ///
    /// If the duration value is `None`, then that check is skipped. This
    /// is specified in the configuration by placing a "0".
    pub rerender_skip: Vec<(u32, Option<TimeDuration>)>,

    /// The layout used when rendering direct messages.
    pub message_layout: Layout,

    /// The layout used by default when rendering a page.
    ///
    /// This only comes into effect if the page and site do not
    /// have a different layout set.
    pub default_page_layout: Layout,

    /// Prefix for "special pages". Default: `_`
    #[allow(dead_code)] // TEMP
    pub special_page_prefix: String,

    /// Page slug for the template page. Default: `_template`
    pub special_page_template: String,

    /// Page slug for pages that don't exist. Default: `_404`
    pub special_page_missing: String,

    /// Page slug for pages you don't have permission to see. Default: `_public`
    pub special_page_private: String,

    /// Page slug for when the user is banned, and the site disallows banned viewing. Default: `_ban`
    pub special_page_banned: String,

    /// Default name changes per user.
    pub default_name_changes: i16,

    /// Maximum name changes per user.
    pub maximum_name_changes: i16,

    /// How long until a user gets another name change token.
    /// `None` means that no name change tokens are refilled.
    pub refill_name_change: Option<StdDuration>,

    /// Minimum length of bytes in a username.
    pub minimum_name_bytes: usize,

    /// Length of randomly-generated portion of S3 presigned URLs.
    pub presigned_path_length: usize,

    /// How long S3 presigned URLs will last before expiry.
    pub presigned_expiry_secs: u32,

    /// Maximum size of the subject line allowed in a direct message.
    pub maximum_message_subject_bytes: usize,

    /// Maximum size of the wikitext body allowed in a direct message.
    pub maximum_message_body_bytes: usize,

    /// Maximum number of total recipients allowed in a direct message.
    pub maximum_message_recipients: usize,
}

impl Config {
    #[inline]
    pub fn load(path: PathBuf) -> Result<Self> {
        let (config_file, extra) = ConfigFile::load(path)?;
        let config = ConfigFile::into_config(config_file, extra);
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

        info!("Configuration details:");
        info!("Serving on {}", self.address);
        info!(
            "Auto-restart on config change: {}",
            bool_str(self.watch_files),
        );
        info!("Seeder: {}", bool_str(self.run_seeder));
        info!("Localization path: {}", self.localization_path.display());
        info!("Seeder path: {}", self.seeder_path.display());
        info!(
            "Current working directory: {}",
            env::current_dir()
                .expect("Cannot get current working directory")
                .display(),
        );
    }
}
