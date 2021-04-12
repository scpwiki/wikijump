/*
 * log.rs
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

//! Helper module to add filenames and newlines to slog output.
//!
//! See https://github.com/slog-rs/slog/issues/123

macro_rules! slog_filename {
    () => {
        slog::PushFnValue(|r: &slog::Record, ser: slog::PushFnValueSerializer| {
            ser.emit(r.file())
        })
    };
}

macro_rules! slog_lineno {
    () => {
        slog::PushFnValue(|r: &slog::Record, ser: slog::PushFnValueSerializer| {
            ser.emit(r.line())
        })
    };
}

#[cfg(test)]
#[allow(dead_code)]
mod loggers {
    #[inline]
    pub fn build_logger() -> slog::Logger {
        build_null_logger()
    }

    pub fn build_null_logger() -> slog::Logger {
        slog::Logger::root(slog::Discard, o!())
    }

    pub fn build_terminal_logger() -> slog::Logger {
        use sloggers::terminal::TerminalLoggerBuilder;
        use sloggers::types::Severity;
        use sloggers::Build;

        TerminalLoggerBuilder::new()
            .level(Severity::Trace)
            .build()
            .expect("Unable to initialize logger")
    }
}

#[cfg(test)]
pub use self::loggers::{build_logger, build_null_logger, build_terminal_logger};
