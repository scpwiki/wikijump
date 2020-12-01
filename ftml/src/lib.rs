/*
 * lib.rs
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

#![deny(missing_debug_implementations)]
#![forbid(unsafe_code)]

//! A library to parse Wikidot text and produce an abstract syntax tree (AST).
//!
//! This library aims to be a replacement of Wikidot's Text_Wiki
//! parser, which is presently a loose group of regular expressions
//! (with irregular Perl extensions). The aim is to provide an AST
//! while also maintaining the flexibility and lax parsing that
//! Wikidot permits.

#[macro_use]
extern crate cfg_if;

#[macro_use]
extern crate enum_map;

#[macro_use]
extern crate lazy_static;
extern crate pest;

#[macro_use]
extern crate pest_derive;
extern crate regex;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate slog;

#[macro_use]
extern crate str_macro;
extern crate strum;
extern crate strum_macros;

cfg_if! {
    if #[cfg(test)] {
        extern crate serde_json;
        extern crate slog_bunyan;
        extern crate sloggers;
    }
}

pub mod data;
pub mod tree;

mod enums;
mod parse;
mod preproc;
mod token;

pub use self::parse::{parse, ExtractedToken, ParseError, ParseErrorKind, ParseResult, Token};
pub use self::preproc::preprocess;
pub use self::token::tokenize;

pub mod prelude {
    pub use super::tree::{Element, SyntaxTree};
    pub use super::{data, parse, preprocess, tokenize};
}

#[cfg(test)]
#[inline]
fn build_logger() -> slog::Logger {
    build_console_logger()
}

#[cfg(test)]
#[allow(dead_code)]
fn build_console_logger() -> slog::Logger {
    use sloggers::terminal::TerminalLoggerBuilder;
    use sloggers::types::Severity;
    use sloggers::Build;

    TerminalLoggerBuilder::new()
        .level(Severity::Trace)
        .build()
        .expect("Unable to initialize logger")
}

#[cfg(test)]
#[allow(dead_code)]
fn build_json_logger() -> slog::Logger {
    use slog::Drain;
    use std::io;
    use std::sync::Mutex;

    // For writing to a file:
    // + .add_default_keys()
    // + .set_pretty(false)
    let drain = slog_bunyan::with_name("ftml", io::stdout())
        .set_newlines(true)
        .set_pretty(true)
        .set_flush(true)
        .build();

    slog::Logger::root(Mutex::new(drain).fuse(), o!("env" => "test"))
}
