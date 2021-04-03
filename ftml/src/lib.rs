/*
 * lib.rs
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

#[macro_use]
extern crate maplit;
extern crate pest;

#[macro_use]
extern crate pest_derive;
extern crate ref_map;
extern crate regex;

#[macro_use]
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate slog;

#[macro_use]
extern crate str_macro;
extern crate strum;
extern crate strum_macros;
extern crate unicase;
extern crate void;
extern crate wikidot_normalize;

cfg_if! {
    if #[cfg(test)] {
        extern crate sloggers;
    }
}

#[cfg(test)]
mod test;

#[macro_use]
mod log;

#[macro_use]
mod macros;

mod non_empty_vec;
mod preproc;
mod span_wrap;
mod text;

pub mod data;
pub mod includes;
pub mod parsing;
pub mod render;
pub mod tokenizer;
pub mod tree;

#[cfg(test)]
pub use self::log::{build_logger, build_null_logger, build_terminal_logger};

pub use self::includes::include;
pub use self::parsing::parse;
pub use self::preproc::preprocess;
pub use self::tokenizer::{tokenize, Tokenization};

pub mod prelude {
    pub use super::includes::{include, Includer};
    pub use super::parsing::{parse, ParseResult, ParseWarning};
    pub use super::render::Render;
    pub use super::tokenizer::{tokenize, Tokenization};
    pub use super::tree::{Element, SyntaxTree};
    pub use super::{data, preprocess};
}
