/*
 * config.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2020 Ammon Smith
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

use crate::{info, logger};
use clap::{App, Arg};
use sloggers::types::Severity;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::process;

const DEFAULT_PORT: &str = "3865";
const DEFAULT_LOG_LEVEL: &str = "debug";

#[derive(Debug, Clone)]
pub struct Config {
    pub log_level: Severity,
    pub log_file: PathBuf,
    pub address: SocketAddr,
}

impl Config {
    #[cold]
    pub fn parse_args() -> Self {
        let matches = App::new("ftml")
            .version(&**info::VERSION)
            .author("Wikijump Team")
            .about("REST server to parse and render Wikidot text.")
            .max_term_width(110)
            .arg(
                Arg::with_name("info")
                    .long("info-only")
                    .help("Print information then exit."),
            )
            .arg(
                Arg::with_name("port")
                    .short("p")
                    .long("port")
                    .value_name("PORT")
                    .default_value(DEFAULT_PORT)
                    .help("The port to be used by the server."),
            )
            .arg(
                Arg::with_name("ipv4_only")
                    .short("4")
                    .long("ipv4")
                    .help("Only host the server on IPv4."),
            )
            .arg(
                Arg::with_name("log_file")
                    .short("l")
                    .long("log-file")
                    .value_name("FILE")
                    .default_value("ftml.log")
                    .help("The log file to write formatted entries to."),
            )
            .arg(
                Arg::with_name("log_level")
                    .short("L")
                    .long("log-level")
                    .value_name("LEVEL")
                    .default_value(DEFAULT_LOG_LEVEL)
                    .help("Log level to be use when running the server."),
            )
            .get_matches();

        // Process settings
        let port = matches
            .value_of("port")
            .expect("No port argument set")
            .parse::<u16>()
            .expect("Invalid port number");

        let host = match matches.value_of("ipv4_only") {
            Some(_) => Ipv4Addr::UNSPECIFIED.into(),
            None => Ipv6Addr::UNSPECIFIED.into(),
        };

        let address = SocketAddr::new(host, port);

        let log_file = matches
            .value_of_os("log_file")
            .expect("No log file argument set")
            .to_os_string()
            .into();

        let log_level = {
            let value = matches
                .value_of("log_level")
                .expect("No log level argument set");

            get_log_level(value).expect("Invalid log level value")
        };

        // If info-only, then print and quit
        if matches.is_present("info") {
            let path = Path::new("/dev/null");
            let log = logger::build(path, log_level);
            info::print(&log, address);
            process::exit(0);
        }

        Config {
            log_file,
            log_level,
            address,
        }
    }
}

fn get_log_level(value: &str) -> Option<Severity> {
    const LOG_LEVELS: [(&str, Severity); 6] = [
        ("trace", Severity::Trace),
        ("debug", Severity::Debug),
        ("info", Severity::Info),
        ("warning", Severity::Warning),
        ("error", Severity::Error),
        ("critical", Severity::Critical),
    ];

    for (name, level) in &LOG_LEVELS {
        if name.eq_ignore_ascii_case(value) {
            return Some(*level);
        }
    }

    None
}
