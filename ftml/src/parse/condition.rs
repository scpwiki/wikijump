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

use super::error::ParseError;
use super::parser::Parser;
use super::token::Token;
use std::fmt::{self, Debug};

/// The closure being evaluated for a custom parse condition.
///
/// This returns a copy of the parse state for the function to explore.
///
/// For convenience, it returns `ParseResult` instead of plain boolean for convenience.
/// Any `Err(_)` case is interpreted as `false`.
pub type ParseConditionFn =
    dyn for<'r, 't> Fn(&mut Parser<'r, 't>) -> Result<bool, ParseError>;

/// Represents a condition on a parse state.
///
/// It takes a parser state and determines if it matches
/// the condition described by this structure, returning
/// a boolean as appropriate.
#[derive(Copy, Clone)]
pub enum ParseCondition<'f> {
    CurrentToken { token: Token },
    TokenPair { current: Token, next: Token },
    Function { f: &'f ParseConditionFn },
}

impl<'f> ParseCondition<'f> {
    #[inline]
    pub fn current(token: Token) -> Self {
        ParseCondition::CurrentToken { token }
    }

    #[inline]
    pub fn token_pair(current: Token, next: Token) -> Self {
        ParseCondition::TokenPair { current, next }
    }

    #[inline]
    pub fn function(f: &'f ParseConditionFn) -> Self {
        ParseCondition::Function { f }
    }
}

impl Debug for ParseCondition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseCondition::CurrentToken { token } => f
                .debug_struct("CurrentToken")
                .field("token", &token)
                .finish(),
            ParseCondition::TokenPair { current, next } => f
                .debug_struct("TokenPair")
                .field("current", &current)
                .field("next", &next)
                .finish(),
            ParseCondition::Function { f: fn_pointer } => f
                .debug_struct("Function")
                .field("f", &(fn_pointer as *const ParseConditionFn))
                .finish(),
        }
    }
}
