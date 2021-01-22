/*
 * logger.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

use crate::info::CI_PLATFORM;
use slog::Drain;
use sloggers::terminal::TerminalLoggerBuilder;
use sloggers::types::Severity;
use sloggers::Build;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Mutex;

pub fn build(log_file: &Path, log_level: Severity) -> slog::Logger {
    let json_file = OpenOptions::new()
        .append(true)
        .truncate(false)
        .create(true)
        .open(log_file)
        .expect("Unable to create log file");

    let json_drain = slog_bunyan::with_name("ftml", json_file)
        .add_default_keys()
        .set_newlines(true)
        .set_pretty(false)
        .set_flush(true)
        .build();

    let console_drain = TerminalLoggerBuilder::new()
        .level(log_level)
        .build()
        .expect("Unable to initialize logger");

    let drain = slog::Duplicate(json_drain, console_drain);
    let env = match CI_PLATFORM {
        Some(_) => "ci",
        None => "server",
    };

    slog::Logger::root(Mutex::new(drain).fuse(), o!("env" => env))
}
