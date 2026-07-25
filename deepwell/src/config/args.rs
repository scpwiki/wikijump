/*
 * config/args.rs
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

use super::Config;
use crate::error::Result;
use crate::info;
use clap::builder::{BoolishValueParser, NonEmptyStringValueParser};
use clap::{Arg, ArgAction, Command, value_parser};
use std::env;
use std::ffi::OsString;
use std::net::IpAddr;
use std::path::PathBuf;
use std::process;

pub fn parse_args() -> Config {
    parse_args_from(env::args_os(), Config::load)
}

fn command() -> Command {
    Command::new("DEEPWELL")
        .author(info::PKG_AUTHORS)
        .version(info::VERSION.as_str())
        .long_version(info::FULL_VERSION.as_str())
        .about(info::PKG_DESCRIPTION)
        .arg(
            Arg::new("disable-log")
                .short('q')
                .long("quiet")
                .long("disable-log")
                .action(ArgAction::SetTrue)
                .help("Disable logging output."),
        )
        .arg(
            Arg::new("log-level")
                .short('l')
                .long("log")
                .long("log-level")
                .value_name("LEVEL")
                .value_parser(NonEmptyStringValueParser::new())
                .action(ArgAction::Set)
                .help("What logging level to use; trace requests are capped at debug."),
        )
        .arg(
            Arg::new("host")
                .short('H')
                .long("host")
                .long("hostname")
                .value_name("HOST")
                .value_parser(value_parser!(IpAddr))
                .action(ArgAction::Set)
                .help("What host to listen on."),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .value_parser(value_parser!(u16))
                .action(ArgAction::Set)
                .help("What port to listen on."),
        )
        .arg(
            Arg::new("watch-config")
                .short('w')
                .long("watch")
                .action(ArgAction::SetTrue)
                .help("Whether to auto-restart when configuration or localization files change."),
        )
        .arg(
            Arg::new("run-seeder")
                .short('S')
                .long("seeder")
                .long("run-seeder")
                .value_name("BOOLEAN")
                .value_parser(BoolishValueParser::new())
                .action(ArgAction::Set)
                .help("Whether to run the seeder on server startup."),
        )
        .arg(
            Arg::new("seeder-path")
                .long("seed")
                .value_parser(value_parser!(PathBuf))
                .value_name("PATH")
                .help("The path to read seeder data from."),
        )
        .arg(
            Arg::new("localization-path")
                .short('L')
                .long("localizations")
                .value_parser(value_parser!(PathBuf))
                .value_name("PATH")
                .help("The path to read translation files from."),
        )
        .arg(
            Arg::new("config-file")
                .value_parser(value_parser!(PathBuf))
                .action(ArgAction::Set)
                .required(true)
                .help("The configuration file to use for this DEEPWELL instance."),
        )
}

fn parse_args_from<I, T, F>(args: I, load_config: F) -> Config
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
    F: FnOnce(PathBuf) -> Result<Config>,
{
    let mut matches = command().get_matches_from(args);

    // Read configuration from path

    let config_path = matches
        .remove_one::<PathBuf>("config-file")
        .expect("Required argument not provided");

    let mut config = match load_config(config_path) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Unable to load configuration from file: {error}");
            process::exit(1);
        }
    };

    // Process remaining arguments and modify config

    if matches.remove_one::<bool>("disable-log") == Some(true) {
        config.logger = false;
    }

    if let Some(value) = matches.remove_one::<String>("log-level") {
        match value.parse() {
            Ok(level) => config.logger_level = level,
            Err(error) => {
                eprintln!("Invalid logging level: {value} ({error})");
                process::exit(1);
            }
        }
    }

    if let Some(value) = matches.remove_one::<IpAddr>("host") {
        config.address.set_ip(value);
    }

    if let Some(value) = matches.remove_one::<u16>("port") {
        config.address.set_port(value);
    }

    if matches.remove_one::<bool>("watch-config") == Some(true) {
        config.watch_files = true;
    }

    if let Some(value) = matches.remove_one::<bool>("run-seeder") {
        config.run_seeder = value;
    }

    if let Some(value) = matches.remove_one::<PathBuf>("localization-path") {
        config.localization_path = value;
    }

    if let Some(value) = matches.remove_one::<PathBuf>("seeder-path") {
        config.seeder_path = value;
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use femme::LevelFilter;
    use std::net::{IpAddr, Ipv4Addr};

    fn parse_test_args(args: &[&str]) -> Config {
        parse_args_from(args.iter().copied(), |path| {
            assert_eq!(path, PathBuf::from("config.toml"));
            Ok(Config::integration_testing())
        })
    }

    #[test]
    fn command_line_flags_override_config_file_values() {
        let config = parse_test_args(&[
            "deepwell",
            "--disable-log",
            "--log-level",
            "warn",
            "--hostname",
            "127.0.0.1",
            "--port",
            "8080",
            "--watch",
            "--run-seeder",
            "true",
            "--localizations",
            "../test-locales",
            "--seed",
            "../test-seeder",
            "config.toml",
        ]);

        assert!(!config.logger);
        assert_eq!(config.logger_level, LevelFilter::Warn);
        assert_eq!(config.address.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(config.address.port(), 8080);
        assert!(config.watch_files);
        assert!(config.run_seeder);
        assert_eq!(config.localization_path, PathBuf::from("../test-locales"));
        assert_eq!(config.seeder_path, PathBuf::from("../test-seeder"));
    }

    #[test]
    fn short_flags_are_accepted() {
        let config = parse_test_args(&[
            "deepwell",
            "-q",
            "-l",
            "debug",
            "-H",
            "127.0.0.1",
            "-p",
            "9090",
            "-w",
            "-S",
            "false",
            "-L",
            "../locale-short",
            "config.toml",
        ]);

        assert!(!config.logger);
        assert_eq!(config.logger_level, LevelFilter::Debug);
        assert_eq!(config.address.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(config.address.port(), 9090);
        assert!(config.watch_files);
        assert!(!config.run_seeder);
        assert_eq!(config.localization_path, PathBuf::from("../locale-short"));
    }
}
