/*
 * parse/error.rs
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

use super::{rule::Rule, Token};
use std::ops::Range;
use strum_macros::IntoStaticStr;

#[derive(Debug, Clone)]
pub struct ParseError {
    token: Token,
    rule: &'static str,
    span: Range<usize>,
    kind: ParseErrorKind,
}

impl ParseError {
    #[inline]
    pub fn new(token: Token, rule: Rule, span: Range<usize>, kind: ParseErrorKind) -> Self {
        let rule = rule.name();

        ParseError {
            token,
            rule,
            span,
            kind,
        }
    }
}

#[derive(IntoStaticStr, Debug, Copy, Clone)]
pub enum ParseErrorKind {
    /// The self-enforced recursion limit has been passed, giving up.
    RecursionDepthExceeded,

    /// No rules match for these tokens, returning as plain text.
    NoRulesMatch,
}

impl ParseErrorKind {
    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }
}
