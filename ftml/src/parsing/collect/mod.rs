/*
 * parsing/collect/mod.rs
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

//! Module which contains functions to iterate through tokens and output according to rules.
//!
//! The main function here is `collect()`, which is a generic procedure to perform
//! some action over tokens, finishing or abortion when it reaches certain tokens.
//!
//! The other functions are extensions of `collect()` which perform more specific functions,
//! but still with the same customization.

mod prelude {
    pub use super::collect;
    pub use crate::log::prelude::*;
    pub use crate::parsing::condition::ParseCondition;
    pub use crate::parsing::consume::consume;
    pub use crate::parsing::exception::{ParseWarning, ParseWarningKind};
    pub use crate::parsing::parser::Parser;
    pub use crate::parsing::prelude::*;
    pub use crate::parsing::rule::Rule;
    pub use crate::parsing::token::{ExtractedToken, Token};
    pub use crate::text::FullText;
}

mod consume;
mod container;
mod generic;
mod text;

pub use self::consume::{collect_consume, collect_consume_keep};
pub use self::container::collect_container;
pub use self::generic::collect;
pub use self::text::{collect_text, collect_text_keep};
