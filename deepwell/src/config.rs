/*
 * config.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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
use clap::{App, Arg};
use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::process;

const MIN_SECRET_LENGTH: usize = 64;

#[derive(Debug, Clone)]
pub struct Config {
    /// Whether the logger should be enabled or not.
    ///
    /// Can be set using environment variable `DEEPWELL_LOGGER`.
    pub logger: bool,

    /// The address the server will be hosted on.
    ///
    /// Can be set using environment variables `DEEPWELL_HOST` and `DEEPWELL_PORT`.
    pub address: SocketAddr,

    /// The URL of the PostgreSQL database to connect to.
    ///
    /// Can be set using environment variable `DEEPWELL_DATABASE_URL`.
    pub database_url: String,

    /// Whether to run migrations on startup.
    ///
    /// Can be set using environment variable `DEEPWELL_RUN_MIGRATIONS`.
    pub run_migrations: bool,

    /// The location where all gettext translation files are kept.
    ///
    /// Can be set using environment variable `DEEPWELL_LOCALIZATION_PATH`.
    pub localization_path: PathBuf,

    /// The number of requests allowed per IP per minute.
    ///
    /// Can be set using environment variable `DEEPWELL_RATE_LIMIT_PER_MINUTE`.
    pub rate_limit_per_minute: NonZeroU32,

    /// The secret to bypass the rate-limit.
    /// An empty value means to disable bypassing.
    /// If a value is specified, the secret must be at least 64 bytes long.
    ///
    /// Set using environment variable `DEEPWELL_RATE_LIMIT_SECRET`.
    pub rate_limit_secret: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            logger: true,
            address: "[::]:2747".parse().unwrap(),
            database_url: String::new(),
            run_migrations: true,
            localization_path: PathBuf::from("../locales/out"),
            rate_limit_per_minute: NonZeroU32::new(20).unwrap(),
            rate_limit_secret: String::new(),
        }
    }
}

fn read_env(config: &mut Config) {
    dotenv().ok();

    if let Ok(value) = env::var("DEEPWELL_LOGGER") {
        if value.eq_ignore_ascii_case("true") {
            config.logger = true;
        } else if value.eq_ignore_ascii_case("false") {
            config.logger = false;
        } else {
            eprintln!("DEEPWELL_LOGGER variable is not a valid boolean value");
            process::exit(1);
        }
    }

    if let Ok(value) = env::var("DEEPWELL_HOST") {
        match value.parse() {
            Ok(host) => config.address.set_ip(host),
            Err(_) => {
                eprintln!("DEEPWELL_ADDRESS_HOST variable is not a valid hostname");
                process::exit(1);
            }
        }
    }

    if let Ok(value) = env::var("DEEPWELL_PORT") {
        match value.parse() {
            Ok(port) => config.address.set_port(port),
            Err(_) => {
                eprintln!("DEEPWELL_ADDRESS_PORT variable is not a valid port");
                process::exit(1);
            }
        }
    }

    if let Ok(value) = env::var("DEEPWELL_DATABASE_URL") {
        config.database_url = value;
    }

    if let Ok(value) = env::var("DEEPWELL_RUN_MIGRATIONS") {
        match value.parse() {
            Ok(run) => config.run_migrations = run,
            Err(_) => {
                eprintln!("DEEPWELL_RUN_MIGRATIONS variable is not a valid boolean");
                process::exit(1);
            }
        }
    }

    if let Some(value) = env::var_os("DEEPWELL_LOCALIZATION_PATH") {
        config.localization_path = PathBuf::from(value);
    }

    if let Ok(value) = env::var("DEEPWELL_RATE_LIMIT_PER_MINUTE") {
        match value.parse() {
            Ok(rate_limit) => config.rate_limit_per_minute = rate_limit,
            Err(_) => {
                eprintln!(
                    "DEEPWELL_RATE_LIMIT_PER_MINUTE variable is not a valid integer",
                );
                process::exit(1);
            }
        }
    }

    if let Ok(value) = env::var("DEEPWELL_RATE_LIMIT_SECRET") {
        if value.len() < MIN_SECRET_LENGTH {
            eprintln!(
                "DEEPWELL_RATE_LIMIT_SECRET value too short (must be at least {} bytes long)",
                MIN_SECRET_LENGTH,
            );
            process::exit(1);
        }

        config.rate_limit_secret = value;
    }
}

fn parse_args(config: &mut Config) {
    let matches = App::new("DEEPWELL")
        .author(info::PKG_AUTHORS)
        .version(info::VERSION.as_str())
        .long_version(info::FULL_VERSION.as_str())
        .about(info::PKG_DESCRIPTION)
        .arg(
            Arg::with_name("disable-log")
                .short("q")
                .long("quiet")
                .long("disable-log")
                .help("Disable logging output."),
        )
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .long("hostname")
                .takes_value(true)
                .value_name("host")
                .help("What host to listen on."),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true)
                .value_name("port")
                .help("What port to listen on."),
        )
        .arg(
            Arg::with_name("database-url")
                .short("d")
                .long("db")
                .long("database")
                .takes_value(true)
                .value_name("uri")
                .help("The URL of the PostgreSQL database to connect to."),
        )
        .arg(
            Arg::with_name("run-migrations")
                .short("M")
                .long("migrate")
                .long("run-migrations")
                .takes_value(true)
                .value_name("true / false")
                .help("Whether to run migrations on server startup."),
        )
        .arg(
            Arg::with_name("localization-path")
                .short("L")
                .long("localizations")
                .takes_value(true)
                .value_name("path")
                .help("The path to read translation files from."),
        )
        .arg(
            Arg::with_name("ratelimit-min")
                .short("r")
                .long("requests-per-minute")
                .takes_value(true)
                .value_name("count")
                .help("How many requests are allowed per IP address per minute."),
        )
        .get_matches();

    // Parse arguments and modify config
    if matches.is_present("disable-log") {
        config.logger = false;
    }

    if let Some(value) = matches.value_of("host") {
        match value.parse() {
            Ok(host) => config.address.set_ip(host),
            Err(_) => {
                eprintln!("Invalid IP address: {}", value);
                process::exit(1);
            }
        }
    }

    if let Some(value) = matches.value_of("port") {
        match value.parse() {
            Ok(port) => config.address.set_port(port),
            Err(_) => {
                eprintln!("Invalid port number: {}", value);
                process::exit(1);
            }
        }
    }

    if let Some(value) = matches.value_of("run-migrations") {
        match value.parse() {
            Ok(run) => config.run_migrations = run,
            Err(_) => {
                eprintln!("Invalid boolean value for migrations: {}", value);
                process::exit(1);
            }
        }
    }

    if let Some(localization_path) = matches.value_of_os("localization-path") {
        config.localization_path = localization_path.into();
    }

    if let Some(value) = matches.value_of("ratelimit-min") {
        match value.parse() {
            Ok(value) => config.rate_limit_secret = value,
            Err(_) => {
                eprintln!("Invalid number of requests per minute: {}", value);
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

        tide::log::info!("Serving on {}", self.address);
        tide::log::info!("Migrations: {}", bool_str(self.run_migrations));
        tide::log::info!("Localization path: {}", self.localization_path.display());
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
