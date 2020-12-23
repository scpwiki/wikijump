/*
 * log.rs
 *
 * ftml - Library to parse Wikidot code
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

use crate::info::CI_PLATFORM;
use slog::Drain;
use sloggers::terminal::TerminalLoggerBuilder;
use sloggers::types::Severity;
use sloggers::Build;
use std::io;
use std::sync::Mutex;

pub fn build_logger() -> slog::Logger {
    let console_drain = TerminalLoggerBuilder::new()
        .level(Severity::Trace)
        .build()
        .expect("Unable to initialize logger");

    let json_drain = slog_bunyan::with_name("ftml", io::stdout())
        .add_default_keys()
        .set_newlines(true)
        .set_pretty(false)
        .set_flush(true)
        .build();

    let drain = slog::Duplicate(console_drain, json_drain);
    let env = match CI_PLATFORM {
        Some(_) => "ci",
        None => "server",
    };

    slog::Logger::root(Mutex::new(drain).fuse(), o!("env" => env))
}
