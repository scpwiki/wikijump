/*
 * parse/rule/impls/block/impls/mod.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Ammon Smith
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

mod prelude {
    pub use super::super::{BlockParser, BlockRule};
    pub use crate::parse::collect::*;
    pub use crate::parse::condition::ParseCondition;
    pub use crate::parse::prelude::*;
    pub use crate::parse::{ParseError, Token};
    pub use crate::tree::Element;
}

// TODO
mod code;
mod div;

pub use self::code::BLOCK_CODE;
pub use self::div::BLOCK_DIV;
