/*
 * config/args.rs
 *
 * Wilson's Web Server - Serves a zoo of user-generated content
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

use crate::info;
use clap::{Arg, ArgAction, Command, value_parser};
use std::env;
use std::ffi::OsString;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Arguments {
    pub enable_trace: bool,
    pub enable_deepwell_check: bool,
    pub pid_file: Option<PathBuf>,
    pub address: SocketAddr,
}

impl Default for Arguments {
    fn default() -> Arguments {
        Arguments {
            enable_trace: true,
            enable_deepwell_check: true,
            pid_file: None,
            address: "[::]:3466".parse().unwrap(),
        }
    }
}

impl Arguments {
    pub fn parse() -> Self {
        Self::parse_from(env::args_os())
    }

    fn command() -> Command {
        Command::new("wws")
            .author(info::PKG_AUTHORS)
            .version(info::PKG_VERSION)
            .about(info::PKG_DESCRIPTION)
            .arg(
                Arg::new("disable-trace")
                    .short('q')
                    .long("quiet")
                    .long("disable-trace")
                    .action(ArgAction::SetTrue)
                    .help("Disable trace output."),
            )
            .arg(
                Arg::new("disable-deepwell-check")
                    .long("disable-deepwell-check")
                    .action(ArgAction::SetTrue)
                    .help("Disable checking DEEPWELL on start."),
            )
            .arg(
                Arg::new("pid-file")
                    .short('P')
                    .long("pid")
                    .long("pid-file")
                    .value_name("PATH")
                    .value_parser(value_parser!(PathBuf))
                    .help("The PID file to write to on boot."),
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
    }

    fn parse_from<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let mut matches = Self::command().get_matches_from(args);

        let mut args = Arguments::default();

        if matches.remove_one::<bool>("disable-trace") == Some(true) {
            args.enable_trace = false;
        }

        if matches.remove_one::<bool>("disable-deepwell-check") == Some(true) {
            args.enable_deepwell_check = false;
        }

        if let Some(value) = matches.remove_one::<PathBuf>("pid-file") {
            args.pid_file = Some(value);
        }

        if let Some(value) = matches.remove_one::<IpAddr>("host") {
            args.address.set_ip(value);
        }

        if let Some(value) = matches.remove_one::<u16>("port") {
            args.address.set_port(value);
        }

        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn defaults_match_documented_listener_settings() {
        let args = Arguments::parse_from(["wws"]);

        assert!(args.enable_trace);
        assert!(args.enable_deepwell_check);
        assert_eq!(args.pid_file, None);
        assert_eq!(args.address.ip(), IpAddr::V6(Ipv6Addr::UNSPECIFIED));
        assert_eq!(args.address.port(), 3466);
    }

    #[test]
    fn command_line_flags_override_defaults() {
        let args = Arguments::parse_from([
            "wws",
            "--disable-trace",
            "--disable-deepwell-check",
            "--pid-file",
            "/tmp/wws.pid",
            "--hostname",
            "127.0.0.1",
            "--port",
            "8080",
        ]);

        assert!(!args.enable_trace);
        assert!(!args.enable_deepwell_check);
        assert_eq!(args.pid_file, Some(PathBuf::from("/tmp/wws.pid")));
        assert_eq!(args.address.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(args.address.port(), 8080);
    }
}
