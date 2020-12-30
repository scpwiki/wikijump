/*
 * parse/condition.rs
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

use super::{Parser, Token};
use std::fmt::{self, Debug};

/// Represents a condition
#[derive(Copy, Clone)]
pub enum ParseCondition {
    CurrentToken { token: Token },
    TokenPair { previous: Token, current: Token },
    Function { f: ParseConditionFn },
}

pub type ParseConditionFn = for<'l, 'r, 't> fn(Parser<'l, 'r, 't>) -> bool;

impl Debug for ParseCondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseCondition::CurrentToken { token } => f
                .debug_struct("CurrentToken")
                .field("token", &token)
                .finish(),
            ParseCondition::TokenPair { previous, current } => f
                .debug_struct("TokenPair")
                .field("previous", &previous)
                .field("current", &current)
                .finish(),
            ParseCondition::Function { f: fn_pointer } => f
                .debug_struct("Function")
                .field("f", &(*fn_pointer as *const ()))
                .finish(),
        }
    }
}
