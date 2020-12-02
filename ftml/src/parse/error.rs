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

use super::{rule::Rule, ExtractedToken, Token};
use std::ops::Range;
use strum_macros::IntoStaticStr;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct ParseError {
    token: Token,
    rule: &'static str,
    span: Range<usize>,
    kind: ParseErrorKind,
}

impl ParseError {
    #[inline]
    pub fn new(kind: ParseErrorKind, rule: Rule, extracted: &ExtractedToken) -> Self {
        let token = extracted.token;
        let span = Range::clone(&extracted.span);
        let rule = rule.name();

        ParseError {
            token,
            rule,
            span,
            kind,
        }
    }

    #[inline]
    pub fn token(&self) -> Token {
        self.token
    }

    #[inline]
    pub fn rule(&self) -> &'static str {
        self.rule
    }

    #[inline]
    pub fn span(&self) -> Range<usize> {
        Range::clone(&self.span)
    }

    #[inline]
    pub fn kind(&self) -> ParseErrorKind {
        self.kind
    }
}

#[derive(Serialize, Deserialize, IntoStaticStr, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ParseErrorKind {
    /// The self-enforced recursion limit has been passed, giving up.
    RecursionDepthExceeded,

    /// Attempting to match this rule failed, falling back to try an alternate.
    RuleFailed,

    /// Attempting to process this rule failed because the end of input was reached.
    EndOfInput,

    /// Temporary rule denoting that this syntactical construction isn't implemented yet.
    NotImplemented,

    /// No rules match for these tokens, returning as plain text.
    NoRulesMatch,
}

impl ParseErrorKind {
    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }
}
