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

#![deny(missing_debug_implementations, unsafe_code)]

//! A library to parse Wikidot text and produce an abstract syntax tree (AST).
//!
//! This library aims to be a replacement of Wikidot's Text_Wiki
//! parser, which is presently a loose group of regular expressions
//! (with irregular Perl extensions). The aim is to provide an AST
//! while also maintaining the flexibility and lax parsing that
//! Wikidot permits.
//!
//! The overall flow is the following:
//!
//! * Run messy includer
//! * Run preprocessor
//! * Run tokenizer
//! * Run parser
//! * Run renderer
//!
//! Each step of the flow makes extensive use of Rust's
//! borrowing capabilities, ensuring that as few allocations
//! are performed as possible. Any strings which are unmodified
//! are passed by reference. Despite this, all of the exported
//! structures are both serializable and deserializable via
//! [`serde`].
//!
//! Rendering is performed by the trait [`Render`].
//! There are two main implementations of note,
//! [`TextRender`] and [`HtmlRender`], which render to
//! plain text and full HTML respectively.
//!
//! # Features
//! This crate has several features of note.
//!
//! By default the `ffi` and `log` features are enabled.
//! These enable support for FFI interfacing for the library
//! via [`cbindgen`] (with a slightly more limited interface),
//! and logging via [`slog`] respectively.
//!
//! If the `log` feature is enabled, then all calls requiring
//! a `Logger` are replaced with a stub, and all actual logging
//! calls are replaced with no-ops. Generally you want this
//! for very performance-sensitive contexts where logging is
//! simply not worth the overhead.
//!
//! # Targets
//! The library supports being compiled into WebAssembly.
//! (target `wasm32-unknown-unknown`, see [`wasm-pack`] for more information)
//!
//! This adds the feature `wasm-log`, which adds `slog` logging support via
//! `console.log()` calls to the browser's console. This is very useful for
//! debugging, but caveat emptor! This spams the console very hard and can cause
//! lag on some browsers. Do not enable in production.
//!
//! Additionally, disabling `log` as a feature compiles out all logging, similar
//! to the default target.
//!
//! Compiling to wasm also disables all FFI integration,
//! since these are inherently incompatible.
//!
//! # Bugs
//! If you discover any bugs or have any feature requests,
//! you can submit them via our Atlassian helpdesk [here](https://scuttle.atlassian.net/servicedesk/customer/portal/2).
//!
//! Alternatively, you can [get in touch with Wikijump developers directly](https://github.com/scpwiki/wikijump#readme).
//!
//! [`Render`]: ./render/trait.Render.html
//! [`TextRender`]: ./render/html/struct.HtmlRender.html
//! [`HtmlRender`]: ./render/text/struct.TextRender.html
//! [`serde`]: https://docs.rs/serde
//! [`cbindgen`]: https://docs.rs/cbindgen
//! [`slog`]: https://docs.rs/slog
//! [`wasm-pack`]: https://rustwasm.github.io/docs/wasm-pack/

// Only list crates which we want global macro imports.
// Rest are implicit based on Cargo.toml

#[macro_use]
extern crate cfg_if;

#[macro_use]
extern crate enum_map;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate maplit;

#[macro_use]
extern crate pest_derive;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate str_macro;

#[cfg(feature = "log")]
#[macro_use]
extern crate slog;

#[cfg(not(feature = "log"))]
#[macro_use]
extern crate slog_mock;

// Library top-level modules

#[cfg(test)]
mod test;

#[macro_use]
mod log;

#[macro_use]
mod macros;

mod non_empty_vec;
mod page_info;
mod preproc;
mod span_wrap;
mod text;
mod url;
mod user_info;
mod utf16;

#[cfg(feature = "ffi")]
#[cfg(not(target_arch = "wasm32"))]
pub mod ffi;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub mod includes;
pub mod info;
pub mod parsing;
pub mod render;
pub mod tokenizer;
pub mod tree;

#[cfg(test)]
#[cfg(feature = "log")]
pub use self::log::{build_logger, build_null_logger, build_terminal_logger};

pub use self::includes::include;
pub use self::page_info::PageInfo;
pub use self::parsing::parse;
pub use self::preproc::preprocess;
pub use self::tokenizer::{tokenize, Tokenization};
pub use self::user_info::UserInfo;
pub use self::utf16::Utf16IndexMap;

pub mod prelude {
    pub use super::includes::{include, Includer};
    pub use super::parsing::{parse, ParseResult, ParseWarning};
    pub use super::render::Render;
    pub use super::tokenizer::{tokenize, Tokenization};
    pub use super::tree::{Element, SyntaxTree};
    pub use super::{preprocess, PageInfo};
}
