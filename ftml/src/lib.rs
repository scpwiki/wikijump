/*
 * lib.rs
 *
 * wikidot-html - Library to convert Wikidot syntax into HTML
 * Copyright (c) 2019 Ammon Smith for Project Foundation
 *
 * wikidot-html is available free of charge under the terms of the MIT
 * License. You are free to redistribute and/or modify it under those
 * terms. It is distributed in the hopes that it will be useful, but
 * WITHOUT ANY WARRANTY. See the LICENSE file for more details.
 *
 */

#![allow(unknown_lints, large_enum_variant, match_bool)]
#![deny(missing_debug_implementations)]

#[macro_use]
extern crate lazy_static;
extern crate regex;

mod error;
mod parse;
mod render;
mod tree;

pub use self::error::Error;
pub use self::parse::parse;
pub use self::render::render;
pub use self::tree::SyntaxTree;

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, Error>;

pub fn transform(text: &str) -> Result<String> {
    let tree = parse(text)?;
    let html = render(tree)?;
    Ok(html)
}

pub mod prelude {
    #![allow(unused_imports)]
    pub use super::{Error, Result, StdResult, SyntaxTree};
    pub use super::{parse, render, transform};
}
