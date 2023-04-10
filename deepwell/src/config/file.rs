/*
 * config/file.rs
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

use super::args;
use super::Config;
use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, process};
use tide::log::LevelFilter;

/// Structure representing a configuration file.
///
/// This differs from the `Config` struct because
/// it contains sub-sections which are used within
/// the TOML file which are then flattened before
/// being used during execution.
///
/// This also lets us customize certain parts of
/// how serialization and deserialization occur.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigFile {
    logger: Logger,
    server: Server,
    database: Database,
    security: Security,
    locale: Locale,
    job: Job,
    ftml: Ftml,
    user: User,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
struct Logger {
    enable: bool,
    level: LevelFilter,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
struct Server {
    address: SocketAddr,
    pid_file: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
struct Database {
    run_migrations: bool,
    run_seeder: bool,
    seeder_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
struct Security {
    authentication_fail_delay_ms: u64,
    session: Session,
    mfa: Mfa,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
struct Session {
    token_prefix: String,
    token_length: usize,
    duration_session_minutes: i64,
    duration_login_minutes: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
struct Mfa {
    recovery_code_count: usize,
    recovery_code_length: usize,
    time_step: u64,
    time_skew: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
struct Job {
    delay_ms: u64,
    prune_session_secs: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
struct Locale {
    path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
struct Ftml {
    render_timeout_ms: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
struct User {
    default_name_changes: u8,
    max_name_changes: u8,
}

impl ConfigFile {
    pub fn load(path: &Path) -> Result<(Self, String)> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config = toml::from_str(&contents)?;
        Ok((config, contents))
    }

    /// Deconstruct the `ConfigFile` and flatten it as a `Config` object.
    pub fn to_config(self, raw_toml: String) -> Config {
        let ConfigFile {
            logger:
                Logger {
                    enable: logger,
                    level: logger_level,
                },
            server: Server { address, mut pid_file },
            database:
                Database {
                    run_migrations,
                    run_seeder,
                    seeder_path,
                },
            security:
                Security {
                    authentication_fail_delay_ms,
                    session:
                        Session {
                            token_prefix,
                            token_length,
                            duration_session_minutes,
                            duration_login_minutes,
                        },
                    mfa:
                        Mfa {
                            recovery_code_count,
                            recovery_code_length,
                            time_step,
                            time_skew,
                        },
                },
            job:
                Job {
                    delay_ms,
                    prune_session_secs,
                },
            locale: Locale {
                path: localization_path,
            },
            ftml: Ftml { render_timeout_ms },
            user:
                User {
                    default_name_changes,
                    max_name_changes,
                },
        } = self;

        // Treats empty strings (which aren't valid paths anyways)
        // as null for the purpose of pid_file.
        if let Some(ref path) = pid_file {
            if path.as_os_str().is_empty() {
                pid_file = None;
            }
        }

        Config {
            raw_toml,
            logger,
            logger_level,
            address,
            pid_file,
            run_migrations,
            run_seeder,
            seeder_path,
            localization_path,
            render_timeout: Duration::from_millis(render_timeout_ms),
        }
    }
}
