/*
 * parse/rule/collect/mod.rs
 *
 * ftml - Library to parse Wikidot text
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

//! Module which contains functions to iterate through tokens and output according to rules.
//!
//! The main function here is `try_collect()`, which is a generic procedure to perform
//! some action over tokens, finishing or abortion when it reaches certain tokens.

mod prelude {
    pub use super::collect;
    pub use crate::parse::condition::ParseCondition;
    pub use crate::parse::consume::consume;
    pub use crate::parse::error::{ParseError, ParseErrorKind};
    pub use crate::parse::parser::Parser;
    pub use crate::parse::prelude::*;
    pub use crate::parse::rule::Rule;
    pub use crate::parse::token::{ExtractedToken, Token};
    pub use crate::text::FullText;
}

mod consume;
mod container;
mod generic;
mod merge;

pub use self::consume::collect_consume;
pub use self::container::collect_container;
pub use self::generic::collect;
pub use self::merge::collect_merge;
