/*
 * config.rs
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

use crate::info;
use clap::{Arg, Command};
use dotenv::dotenv;
use ref_map::*;
use s3::{creds::Credentials, region::Region};
use std::env;
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::process;
use std::time::Duration;
use tide::log::LevelFilter;

const MIN_SECRET_LENGTH: usize = 64;

#[derive(Debug, Clone)]
pub struct Config {
    /// Whether the logger should be enabled or not.
    /// Also enables colorful backtraces.
    ///
    /// Can be set using environment variable `ENABLE_LOGGER`.
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
    ///
    /// Can be set using environment variable `LOGGER_LEVEL`.
    pub logger_level: LevelFilter,

    /// The address the server will be hosted on.
    ///
    /// Can be set using environment variables `SERVER_HOST` and `SERVER_PORT`.
    pub address: SocketAddr,

    /// The URL of the PostgreSQL database to connect to.
    ///
    /// Can be set using environment variable `DATABASE_URL`.
    pub database_url: String,

    /// Whether to run migrations on startup.
    ///
    /// Can be set using environment variable `RUN_MIGRATIONS`.
    pub run_migrations: bool,

    /// Whether to run the seeder on startup.
    /// This will only attempt to add the rows if the `user` table is empty.
    ///
    /// Can be set using environment variable `RUN_SEEDER`.
    pub run_seeder: bool,

    /// The name of the S3 bucket that file blobs are kept in.
    /// The bucket must already exist prior to program invocation.
    ///
    /// Can be set using environment variable `S3_BUCKET`.
    pub s3_bucket: String,

    /// The region to use for S3.
    ///
    /// Can be set using environment variable `S3_AWS_REGION` if standard,
    /// or `S3_REGION_NAME` and `S3_CUSTOM_ENDPOINT` if custom.
    pub s3_region: Region,

    /// The credentials to use for S3.
    ///
    /// Can be set using environment variable `S3_ACCESS_KEY_ID` and `S3_SECRET_ACCESS_KEY`.
    ///
    /// Alternatively you can have it read from the AWS credentials file.
    /// The profile to read from can be set in the `AWS_PROFILE_NAME` environment variable.
    pub s3_credentials: Credentials,

    /// The location where all Fluent translation files are kept.
    ///
    /// Can be set using environment variable `LOCALIZATION_PATH`.
    pub localization_path: PathBuf,

    /// The location where all the seeder files are kept.
    ///
    /// Can be set using environment variable `SEEDER_PATH`.
    pub seeder_path: PathBuf,

    /// The number of requests allowed per IP per minute.
    ///
    /// Can be set using environment variable `RATE_LIMIT_PER_MINUTE`.
    pub rate_limit_per_minute: NonZeroU32,

    /// The secret to bypass the rate-limit.
    /// An empty value means to disable bypassing.
    /// If a value is specified, the secret must be at least 64 bytes long.
    ///
    /// Set using environment variable `RATE_LIMIT_SECRET`.
    pub rate_limit_secret: String,

    /// How long to allow a render job to run before terminating it.
    ///
    /// This is to ensure that a parser bug or malicious input cannot
    /// crash or freeze the backend. This value should not be too
    /// aggressive, but still not extremely long.
    ///
    /// Set using environment variable `RENDER_TIMEOUT_MS`.
    pub render_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            logger: true,
            logger_level: LevelFilter::Info,
            address: "[::]:2747".parse().unwrap(),
            database_url: str!("postgres://localhost"),
            run_migrations: true,
            run_seeder: true,
            s3_bucket: String::new(),
            s3_region: Region::Custom {
                region: String::new(),
                endpoint: String::new(),
            },
            s3_credentials: Credentials {
                access_key: None,
                secret_key: None,
                security_token: None,
                session_token: None,
                expiration: None,
            },
            localization_path: PathBuf::from("../locales"),
            seeder_path: PathBuf::from("seeder"),
            rate_limit_per_minute: NonZeroU32::new(20).unwrap(),
            rate_limit_secret: String::new(),
            render_timeout: Duration::from_millis(2000),
        }
    }
}

fn read_env(config: &mut Config) {
    dotenv().ok();

    if let Ok(value) = env::var("ENABLE_LOGGER") {
        if value.eq_ignore_ascii_case("true") {
            config.logger = true;
        } else if value.eq_ignore_ascii_case("false") {
            config.logger = false;
        } else {
            eprintln!("ENABLE_LOGGER variable is not a valid boolean value");
            process::exit(1);
        }
    }

    if let Ok(value) = env::var("LOGGER_LEVEL") {
        match get_log_level(&value) {
            Some(level) => config.logger_level = level,
            None => {
                eprintln!("LOGGER_LEVEL variable does not have a valid logging level");
                process::exit(1);
            }
        }
    }

    if let Ok(value) = env::var("SERVER_HOST") {
        match value.parse() {
            Ok(host) => config.address.set_ip(host),
            Err(_) => {
                eprintln!("SERVER_HOST variable is not a valid hostname");
                process::exit(1);
            }
        }
    }

    if let Ok(value) = env::var("SERVER_PORT") {
        match value.parse() {
            Ok(port) => config.address.set_port(port),
            Err(_) => {
                eprintln!("SERVER_PORT variable is not a valid port");
                process::exit(1);
            }
        }
    }

    if let Ok(value) = env::var("DATABASE_URL") {
        config.database_url = value;
    }

    if let Ok(value) = env::var("RUN_MIGRATIONS") {
        match value.parse() {
            Ok(run) => config.run_migrations = run,
            Err(_) => {
                eprintln!("RUN_MIGRATIONS variable is not a valid boolean");
                process::exit(1);
            }
        }
    }

    if let Ok(value) = env::var("RUN_SEEDER") {
        match value.parse() {
            Ok(run) => config.run_seeder = run,
            Err(_) => {
                eprintln!("RUN_SEEDER variable is not a valid boolean");
                process::exit(1);
            }
        }
    }

    if let Ok(value) = env::var("S3_BUCKET") {
        config.s3_bucket = value;
    }

    if let Ok(value) = env::var("S3_AWS_REGION") {
        match value.parse() {
            Ok(region) => config.s3_region = region,
            Err(_) => {
                eprintln!("S3_AWS_REGION variable is not a valid AWS region ID");
                process::exit(1);
            }
        }
    } else {
        let region = env::var("S3_REGION_NAME");
        let endpoint = env::var("S3_CUSTOM_ENDPOINT");

        if let (Ok(region), Ok(endpoint)) = (region, endpoint) {
            config.s3_region = Region::Custom { region, endpoint };
        }
    }

    if let Ok(credentials) = Credentials::from_env_specific(
        Some("S3_ACCESS_KEY_ID"),
        Some("S3_SECRET_ACCESS_KEY"),
        None,
        None,
    ) {
        // Try to read from environment
        // Reads from S3_ACCESS_KEY_ID and S3_SECRET_ACCESS_KEY
        config.s3_credentials = credentials;
    } else {
        // Try to read from profile
        let profile_name = env::var("AWS_PROFILE_NAME").ok();
        let profile_name = profile_name.ref_map(|s| s.as_str());

        config.s3_credentials = match Credentials::from_profile(profile_name) {
            Ok(credentials) => credentials,
            Err(error) => {
                eprintln!("Unable to read AWS credentials file: {}", error);
                process::exit(1);
            }
        };
    }

    if let Some(value) = env::var_os("LOCALIZATION_PATH") {
        config.localization_path = PathBuf::from(value);
    }

    if let Some(value) = env::var_os("SEEDER_PATH") {
        config.seeder_path = PathBuf::from(value);
    }

    if let Ok(value) = env::var("RATE_LIMIT_PER_MINUTE") {
        match value.parse() {
            Ok(rate_limit) => config.rate_limit_per_minute = rate_limit,
            Err(_) => {
                eprintln!("RATE_LIMIT_PER_MINUTE variable is not a valid integer");
                process::exit(1);
            }
        }
    }

    if let Ok(value) = env::var("RATE_LIMIT_SECRET") {
        if value.len() < MIN_SECRET_LENGTH {
            eprintln!(
                "RATE_LIMIT_SECRET value too short (must be at least {MIN_SECRET_LENGTH} bytes long)",
            );
            process::exit(1);
        }

        config.rate_limit_secret = value;
    }

    if let Ok(value) = env::var("RENDER_TIMEOUT_MS") {
        match value.parse() {
            Ok(ms) => config.render_timeout = Duration::from_millis(ms),
            Err(_) => {
                eprintln!(
                    "RENDER_TIMEOUT_MS variable is not a valid number of milliseconds",
                );
                process::exit(1);
            }
        }
    }
}

fn parse_args(config: &mut Config) {
    let matches = Command::new("DEEPWELL")
        .author(info::PKG_AUTHORS)
        .version(info::VERSION.as_str())
        .long_version(info::FULL_VERSION.as_str())
        .about(info::PKG_DESCRIPTION)
        .arg(
            Arg::new("disable-log")
                .short('q')
                .long("quiet")
                .long("disable-log")
                .help("Disable logging output."),
        )
        .arg(
            Arg::new("log-level")
                .short('l')
                .long("log")
                .long("log-level")
                .takes_value(true)
                .value_name("LEVEL")
                .help("What logging level to use."),
        )
        .arg(
            Arg::new("host")
                .short('h')
                .long("host")
                .long("hostname")
                .takes_value(true)
                .value_name("HOST")
                .help("What host to listen on."),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .takes_value(true)
                .value_name("PORT")
                .help("What port to listen on."),
        )
        .arg(
            Arg::new("database-url")
                .short('d')
                .long("db")
                .long("database")
                .takes_value(true)
                .value_name("URI")
                .help("The URL of the PostgreSQL database to connect to."),
        )
        .arg(
            Arg::new("run-migrations")
                .short('M')
                .long("migrate")
                .long("run-migrations")
                .takes_value(true)
                .value_name("BOOLEAN")
                .help("Whether to run migrations on server startup."),
        )
        .arg(
            Arg::new("run-seeder")
                .short('S')
                .long("seeder")
                .long("run-seeder")
                .takes_value(true)
                .value_name("BOOLEAN")
                .help("Whether to run the seeder on server startup."),
        )
        .arg(
            Arg::new("s3-bucket")
                .short('B')
                .long("bucket")
                .long("s3-bucket")
                .takes_value(true)
                .value_name("NAME")
                .help("The name of the S3 bucket where uploaded file blobs are kept."),
        )
        .arg(
            Arg::new("aws-region")
                .long("aws-region")
                .takes_value(true)
                .value_name("NAME")
                .help("The name of the standard AWS region to use for AWS calls. Conflicts with --s3-region."),
        )
        .arg(
            Arg::new("s3-region")
                .long("s3-region")
                .takes_value(true)
                .value_name("NAME")
                .help("The name of the custom region to use, if not AWS. Conflicts with --aws-region."),
        )
        .arg(
            Arg::new("s3-endpoint")
                .long("s3-endpoint")
                .takes_value(true)
                .value_name("URL")
                .help("The endpoint to contact for S3 calls, if not AWS. Requires --s3-region."),
        )
        .arg(
            Arg::new("localization-path")
                .short('L')
                .long("localizations")
                .takes_value(true)
                .value_name("PATH")
                .help("The path to read translation files from."),
        )
        .arg(
            Arg::new("seeder-path")
                .long("seed")
                .takes_value(true)
                .value_name("PATH")
                .help("The path to read seeder data from."),
        )
        .arg(
            Arg::new("ratelimit-min")
                .short('r')
                .long("requests-per-minute")
                .takes_value(true)
                .value_name("COUNT")
                .help("How many requests are allowed per IP address per minute."),
        )
        .arg(
            Arg::new("render-timeout")
                .long("render-timeout")
                .takes_value(true)
                .value_name("MS")
                .help("How long in milliseconds to allow render jobs to run before terminating them."),
        )
        .get_matches();

    // Parse arguments and modify config
    if matches.is_present("disable-log") {
        config.logger = false;
    }

    if let Some(value) = matches.value_of("log-level") {
        match get_log_level(value) {
            Some(level) => config.logger_level = level,
            None => {
                eprintln!("Invalid logging level: {value}");
                process::exit(1);
            }
        }
    }

    if let Some(value) = matches.value_of("host") {
        match value.parse() {
            Ok(host) => config.address.set_ip(host),
            Err(_) => {
                eprintln!("Invalid IP address: {value}");
                process::exit(1);
            }
        }
    }

    if let Some(value) = matches.value_of("port") {
        match value.parse() {
            Ok(port) => config.address.set_port(port),
            Err(_) => {
                eprintln!("Invalid port number: {value}");
                process::exit(1);
            }
        }
    }

    if let Some(value) = matches.value_of("run-migrations") {
        match value.parse() {
            Ok(run) => config.run_migrations = run,
            Err(_) => {
                eprintln!("Invalid boolean value for migrations: {value}");
                process::exit(1);
            }
        }
    }

    if let Some(value) = matches.value_of("run-seeder") {
        match value.parse() {
            Ok(run) => config.run_seeder = run,
            Err(_) => {
                eprintln!("Invalid boolean value for seeder: {value}");
                process::exit(1);
            }
        }
    }

    if let Some(bucket) = matches.value_of("s3-bucket") {
        config.s3_bucket = bucket.into();
    }

    match (
        matches.value_of("aws-region"),
        matches.value_of("s3-region"),
        matches.value_of("s3-endpoint"),
    ) {
        // Using AWS
        (Some(value), None, None) => match value.parse() {
            Ok(region) => config.s3_region = region,
            Err(_) => {
                eprintln!("Invalid standard AWS region name: {value}");
                process::exit(1);
            }
        },

        // Using a custom endpoint
        (None, Some(region), Some(endpoint)) => {
            let region = str!(region);
            let endpoint = str!(endpoint);
            config.s3_region = Region::Custom { region, endpoint };
        }

        // Don't specify anything via arguments, use environment variables instead
        (None, None, None) => (),

        // Conflicting options passed
        _ => {
            eprintln!("Conflicting arguments, you must specify either --aws-region OR --s3-region and --s3-endpoint, not both.");
            process::exit(1);
        }
    }

    if let Some(localization_path) = matches.value_of_os("localization-path") {
        config.localization_path = localization_path.into();
    }

    if let Some(seeder_path) = matches.value_of_os("seeder-path") {
        config.seeder_path = seeder_path.into();
    }

    if let Some(value) = matches.value_of("ratelimit-min") {
        match value.parse() {
            Ok(value) => config.rate_limit_per_minute = value,
            Err(_) => {
                eprintln!("Invalid number of requests per minute: {value}");
                process::exit(1);
            }
        }
    }

    if let Some(value) = matches.value_of("render-timeout") {
        match value.parse() {
            Ok(ms) => config.render_timeout = Duration::from_millis(ms),
            Err(_) => {
                eprintln!("Invalid millisecond timeout for render calls: {value}");
                process::exit(1);
            }
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let mut config = Config::default();
        read_env(&mut config);
        parse_args(&mut config);
        config
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
        tide::log::info!(
            "Rate limit (per minute): {} requests",
            self.rate_limit_per_minute,
        );
        tide::log::info!(
            "Rate limit bypass: {}",
            bool_str(!self.rate_limit_secret.is_empty()),
        );
    }
}

fn get_log_level(value: &str) -> Option<LevelFilter> {
    const LEVELS: [(&str, LevelFilter); 10] = [
        ("off", LevelFilter::Off),
        ("err", LevelFilter::Error),
        ("error", LevelFilter::Error),
        ("warn", LevelFilter::Warn),
        ("warning", LevelFilter::Warn),
        ("info", LevelFilter::Info),
        ("information", LevelFilter::Info),
        ("debug", LevelFilter::Debug),
        ("trace", LevelFilter::Trace),
        ("all", LevelFilter::Trace),
    ];

    for &(name, level) in &LEVELS {
        if value.eq_ignore_ascii_case(name) {
            return Some(level);
        }
    }

    None
}
