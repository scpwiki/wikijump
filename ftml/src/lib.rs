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

#[macro_use]
extern crate lazy_static;
extern crate regex;

mod error;
mod parse;
mod render;
mod tree;
mod utils;

pub use self::error::Error;
pub use self::parse::parse;
pub use self::render::render;
pub use self::tree::SyntaxTree;
pub use self::utils::InPlaceReplace;

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, Error>;

pub fn transform(text: &str) -> Result<String> {
    let tree = parse(text)?;
    let html = render(tree)?;
    Ok(html)
}

pub mod prelude {
    #![allow(unused_imports)]
    pub use super::{Error, InPlaceReplace, Result, StdResult, SyntaxTree};
    pub use super::{parse, render, transform};
}
