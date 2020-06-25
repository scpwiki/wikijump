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

extern crate chrono;
extern crate either;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;
extern crate percent_encoding;
extern crate pest;

#[macro_use]
extern crate pest_derive;
extern crate regex;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate serde_repr;

#[macro_use]
extern crate str_macro;

#[macro_use]
extern crate thiserror;

#[macro_use]
extern crate tinyvec;

#[cfg(test)]
extern crate serde_json;

mod enums;
mod error;
pub mod handle;
mod info;
mod parse;
mod preproc;

pub use self::error::{Error, RemoteError};
pub use self::handle::Handle;
pub use self::info::{PageInfo, PageInfoOwned};
pub use self::parse::{parse, SyntaxTree};
pub use self::preproc::preprocess;

pub mod prelude {
    pub use super::{handle, parse, preprocess};
    pub use super::{Error, PageInfo, PageInfoOwned, Result, StdResult, SyntaxTree};
}

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, Error>;
pub type RemoteResult<T> = StdResult<T, RemoteError>;
