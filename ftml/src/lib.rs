/*
 * lib.rs
 *
 * wikidot-html - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith for Project Foundation
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

#![allow(unknown_lints, large_enum_variant, match_bool)]
#![deny(missing_debug_implementations)]

//! A library to convert Wikidot text source into HTML.
//!
//! Essentially a rewrite of Wikidot's Text_Wiki module, with
//! the intention for better modular integration and standalone
//! servicing.
//!
//! In the future we may even be able to improve the parsing to
//! be better than repeated applications of regular expression
//! substitutions.
//!
//! The main goal of this project is backwards-compatibility: if
//! there is an article on the SCP Wiki which uses a piece of syntax,
//! we intend to support it (or convince the author to change it).
//! Thus, every parsing or rendering rule should have tests, and
//! a dedicated battery of test articles and their HTML outputs
//! are test for any new version.
//!
//! Additionally, as this library matures, features not found within
//! Wikidot's Text_Wiki may be added. These will be clearly documented
//! and a flag will be added to run in legacy mode. However, these
//! new modules or capabilities will hopefully be useful going forward.
//!
//! This crate also provides an executable to convert files from
//! the command-line. See that file for usage documentation.

#[macro_use]
extern crate lazy_static;
extern crate regex;

mod error;
mod parse;
mod render;

pub use self::error::Error;
pub use self::parse::{parse, ParseState, Token};
pub use self::render::render;

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, Error>;

pub fn transform<I: Into<String>>(text: I) -> Result<String> {
    let state = parse(text.into())?;
    let html = render(state)?;
    Ok(html)
}

pub mod prelude {
    #![allow(unused_imports)]
    pub use super::{Error, ParseState, Result, StdResult, Token};
    pub use super::{parse, render, transform};
}
