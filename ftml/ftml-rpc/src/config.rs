/*
 * config.rs
 *
 * ftml-rpc - RPC server to convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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

use log::LevelFilter;
use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

const DEFAULT_PORT: u16 = 3865;
const DEFAULT_KEEP_ALIVE: usize = 20;
const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Info;

// Structopt argument parsing

#[derive(Debug, StructOpt)]
#[structopt(
    name = "ftml-rpc",
    about = "RPC server to convert Wikidot code to HTML"
)]
struct Options {
    /// Logging level to use.
    #[structopt(short, long)]
    level: Option<LevelFilter>,

    /// Configuration file.
    #[structopt(name = "CONFIG_FILE", parse(from_os_str))]
    config_file: PathBuf,
}

// Configuration objects

#[derive(Debug, Clone)]
pub struct Config {
    // Network
    pub hostname: String,
    pub http_address: SocketAddr,
    // Server settings
    pub log_level: LevelFilter,
}

impl Config {
    #[cold]
    pub fn parse_args() -> Self {
        let opts = Options::from_args();
        let mut config: Self = ConfigFile::read(&opts.config_file).into();
        if let Some(level) = opts.level {
            config.log_level = level;
        }

        config
    }
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Debug)]
struct App {
    log_level: Option<String>,
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Debug)]
struct Network {
    hostname: String,
    use_ipv6: bool,
    port: Option<u16>,
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Debug)]
struct ConfigFile {
    app: App,
    network: Network,
}

impl ConfigFile {
    #[cold]
    fn read(path: &Path) -> Self {
        let mut file = File::open(path).expect("Unable to open config file");
        let mut contents = String::new();
        let _ = file
            .read_to_string(&mut contents)
            .expect("Unable to read config file");

        let obj: Self = toml::from_str(&contents).expect("Unable to parse TOML in config file");

        obj
    }

    #[cold]
    fn parse_log_level(log_level: Option<&str>) -> LevelFilter {
        const LEVELS: [(&str, LevelFilter); 9] = [
            ("", DEFAULT_LOG_LEVEL),
            ("off", LevelFilter::Off),
            ("none", LevelFilter::Off),
            ("trace", LevelFilter::Trace),
            ("debug", LevelFilter::Debug),
            ("warn", LevelFilter::Warn),
            ("warning", LevelFilter::Warn),
            ("err", LevelFilter::Error),
            ("error", LevelFilter::Error),
        ];

        let log_level = match log_level {
            Some(ref log_level) => log_level,
            None => return DEFAULT_LOG_LEVEL,
        };

        for (text, level) in &LEVELS {
            if log_level.eq_ignore_ascii_case(text) {
                return *level;
            }
        }

        panic!("No such log level for '{}'", log_level);
    }
}

impl Into<Config> for ConfigFile {
    #[cold]
    fn into(self) -> Config {
        let ConfigFile { app, network } = self;

        let Network {
            hostname,
            use_ipv6,
            port,
        } = network;

        let ip_address = if use_ipv6 {
            IpAddr::V6(Ipv6Addr::UNSPECIFIED)
        } else {
            IpAddr::V4(Ipv4Addr::UNSPECIFIED)
        };

        let http_address = SocketAddr::new(ip_address, port.unwrap_or(DEFAULT_PORT));
        let log_level = app.log_level.as_ref().map(|s| s.as_ref());

        Config {
            hostname,
            http_address,
            log_level: Self::parse_log_level(log_level),
        }
    }
}
