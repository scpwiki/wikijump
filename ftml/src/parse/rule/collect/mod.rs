/*
 * parse/rule/collect/mod.rs
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

//! Module which contains functions to iterate through tokens and output according to rules.
//!
//! The main function here is `try_collect()`, which is a generic procedure to perform
//! some action over tokens, finishing or abortion when it reaches certain tokens.

mod prelude {
    pub use super::try_collect;
    pub use crate::parse::consume::consume;
    pub use crate::parse::error::{ParseError, ParseErrorKind};
    pub use crate::parse::rule::{Consumption, GenericConsumption, Rule};
    pub use crate::parse::token::{ExtractedToken, Token};
    pub use crate::text::FullText;
}

mod container;
mod generic;
mod last;
mod merge;
mod paragraph;

pub use self::container::try_container;
pub use self::generic::try_collect;
pub use self::last::last_before_slice;
pub use self::merge::try_merge;
pub use self::paragraph::try_paragraph;
