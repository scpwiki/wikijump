/*
 * config/object.rs
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

use super::file::ConfigFile;
use crate::error::prelude::*;
use femme::LevelFilter;
use ftml::layout::Layout;
use std::env;
use std::net::SocketAddr;
use std::num::NonZeroU16;
use std::path::PathBuf;
use std::time::Duration as StdDuration;
use time::Duration as TimeDuration;

/// The primary configuration structure for the DEEPWELL server.
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
    pub files_domain: String,

    /// The files domain, but without a leading `.`
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

    /// Whether to enable sqlx statement logging.
    pub sqlx_logging: bool,

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

    /// Length in digits of a TOTP code.
    pub totp_digits: u32,

    /// Length in seconds that each TOTP lasts.
    pub totp_time_step: u64,

    /// Signed seconds added to the server timestamp before TOTP verification.
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

    /// How often to run the "prune pending uploads" recurring job.
    pub job_prune_uploads: StdDuration,

    /// How often to run the "prune unused text" recurring job.
    pub job_prune_text: StdDuration,

    /// How often to run the "refill name change tokens" recurring job.
    pub job_name_change_refill: StdDuration,

    /// How often to run the "lift expired punishments" recurring job.
    pub job_lift_expired_punishments: StdDuration,

    /// Maximum run time for the preprocessing steps of wikitext.
    pub preprocess_timeout: StdDuration,

    /// Maximum run time for parsing and rendering wikitext.
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

    /// Prefix for bluprint pages. Default: `_`
    #[allow(dead_code)] // TEMP
    pub blueprint_page_prefix: String,

    /// Page slug for the template page. Default: `_template`
    pub blueprint_page_template: String,

    /// Page slug for pages that don't exist. Default: `_404`
    pub blueprint_page_missing: String,

    /// Page slug for pages you don't have permission to see. Default: `_public`
    pub blueprint_page_private: String,

    /// Page slug for when the user is banned, and the site disallows banned viewing. Default: `_ban`
    pub blueprint_page_banned: String,

    /// Default name changes per user.
    pub default_name_changes: i16,

    /// Maximum name changes per user.
    pub maximum_name_changes: i16,

    /// How long until a user gets another name change token.
    /// `None` means that no name change tokens are refilled.
    pub refill_name_change: Option<StdDuration>,

    /// Minimum length of bytes in a username.
    pub minimum_name_bytes: usize,

    /// Minimum length of chars in a username.
    pub minimum_name_chars: usize,

    /// Whether to use a mocked version of MailCheck for testing purposes.
    pub mock_mailcheck: bool,

    /// The email address to use when sending typical automation emails.
    pub automation_email: String,

    /// The email address to use when sending notification emails.
    pub notification_email: String,

    /// The email address to use when sending newsletter emails.
    pub newsletter_email: String,

    /// Length of randomly-generated portion of S3 presigned URLs.
    pub presigned_path_length: usize,

    /// How long S3 presigned URLs will last before expiry.
    pub presigned_expiry_secs: u32,

    /// Maximum size of a blob globally.
    pub maximum_blob_size: i64,

    /// Maximum size of a user's avatar image.
    pub maximum_avatar_size: i64,

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
        let (config_file, extra) = ConfigFile::load(path).or_raise(|| {
            Error::new("failed to load configuration data", ErrorType::ConfigSetup)
        })?;

        let config = ConfigFile::into_config(config_file, extra);
        Ok(config)
    }

    /// Caps operational logging below dependency traces that can expose authentication bearers.
    pub fn operational_logger_level(&self) -> LevelFilter {
        match self.logger_level {
            LevelFilter::Trace => LevelFilter::Debug,
            level => level,
        }
    }

    pub fn log(&self) {
        #[inline]
        fn bool_str(value: bool) -> &'static str {
            if value { "enabled" } else { "disabled" }
        }

        info!("Configuration details:");
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

    /// Configuration file for integration testing.
    #[allow(dead_code)]
    pub fn integration_testing() -> Self {
        Config {
            raw_toml: str!("[[ BUILT-IN ]]"),
            raw_toml_path: PathBuf::from("/dev/null"),
            logger: true,
            logger_level: LevelFilter::Debug,
            address: "[::1]:2747".parse().unwrap(),
            pid_file: None,
            main_domain: str!(".wikijump.com"),
            main_domain_no_dot: str!("wikijump.com"),
            files_domain: str!(".wjfiles.com"),
            files_domain_no_dot: str!("wjfiles.com"),
            watch_files: false,
            run_seeder: false,
            sqlx_logging: true,
            seeder_path: PathBuf::from("seeder"),
            localization_path: PathBuf::from("../locales"),
            authentication_fail_delay: StdDuration::from_millis(1),
            session_token_prefix: str!("wj:"),
            session_token_length: 64,
            normal_session_duration: time::Duration::minutes(30),
            restricted_session_duration: time::Duration::minutes(5),
            recovery_code_count: 4,
            recovery_code_length: 8,
            totp_digits: 6,
            totp_time_step: 30,
            totp_time_skew: 1,
            job_workers: NonZeroU16::new(2).unwrap(),
            job_max_attempts: 1,
            job_work_delay: StdDuration::from_millis(1),
            job_min_poll_delay: StdDuration::from_millis(1),
            job_max_poll_delay: StdDuration::from_millis(500),
            job_prune_session: StdDuration::from_secs(60),
            job_prune_uploads: StdDuration::from_secs(60),
            job_prune_text: StdDuration::from_secs(60),
            job_name_change_refill: StdDuration::from_secs(60),
            job_lift_expired_punishments: StdDuration::from_secs(60),
            preprocess_timeout: StdDuration::from_millis(1_000),
            render_timeout: StdDuration::from_millis(9_000),
            rerender_skip: vec![
                (1, Some(time::Duration::milliseconds(100))),
                (5, Some(time::Duration::milliseconds(500))),
                (10, None),
            ],
            message_layout: Layout::Wikijump,
            default_page_layout: Layout::Wikidot,
            blueprint_page_prefix: str!("_"),
            blueprint_page_template: str!("_template"),
            blueprint_page_missing: str!("_404"),
            blueprint_page_private: str!("_public"),
            blueprint_page_banned: str!("_ban"),
            default_name_changes: 2,
            maximum_name_changes: 2,
            refill_name_change: None,
            minimum_name_bytes: 4,
            minimum_name_chars: 2,
            mock_mailcheck: true,
            automation_email: str!("noreply@wikijump.com"),
            notification_email: str!("notifications@wikijump.com"),
            newsletter_email: str!("newsletter@wikijump.com"),
            presigned_path_length: 4,
            presigned_expiry_secs: 60,
            maximum_blob_size: 512 * 1024,
            maximum_avatar_size: 250 * 1024,
            maximum_message_subject_bytes: 128,
            maximum_message_body_bytes: 10_000,
            maximum_message_recipients: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operational_logger_level_caps_trace_at_debug() {
        let mut config = Config::integration_testing();
        config.logger_level = LevelFilter::Trace;
        assert_eq!(config.operational_logger_level(), LevelFilter::Debug);

        config.logger_level = LevelFilter::Info;
        assert_eq!(config.operational_logger_level(), LevelFilter::Info);
    }
}
