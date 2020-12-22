/*
 * parse/upcoming.rs
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

use super::token::ExtractedToken;

// Local type aliases to make this module less messy.
type Token<'r, 't> = &'r ExtractedToken<'t>;
type TokenSlice<'r, 't> = &'r [ExtractedToken<'t>];

#[derive(Debug, Copy, Clone)]
pub enum UpcomingTokens<'r, 't> {
    All {
        tokens: TokenSlice<'r, 't>,
    },
    Split {
        extracted: Token<'r, 't>,
        remaining: TokenSlice<'r, 't>,
    },
}

impl<'r, 't> UpcomingTokens<'r, 't> {
    /// Get the next pair of `(current_token, next_tokens)`.
    pub fn split(&self) -> Option<(Token<'r, 't>, TokenSlice<'r, 't>)> {
        match self {
            UpcomingTokens::All { tokens } => tokens.split_first(),
            UpcomingTokens::Split {
                extracted,
                remaining,
            } => Some((extracted, remaining)),
        }
    }

    /// Get the remaining tokens slice, for adjusting the pointer.
    pub fn slice(&self) -> TokenSlice<'r, 't> {
        match self {
            UpcomingTokens::All { tokens } => tokens,
            UpcomingTokens::Split {
                extracted: _,
                remaining,
            } => remaining,
        }
    }
}

impl<'r, 't> From<TokenSlice<'r, 't>> for UpcomingTokens<'r, 't> {
    #[inline]
    fn from(tokens: &'r [ExtractedToken<'t>]) -> Self {
        UpcomingTokens::All { tokens }
    }
}

impl<'r, 't> From<(Token<'r, 't>, TokenSlice<'r, 't>)> for UpcomingTokens<'r, 't> {
    #[inline]
    fn from((extracted, remaining): (Token<'r, 't>, TokenSlice<'r, 't>)) -> Self {
        UpcomingTokens::Split {
            extracted,
            remaining,
        }
    }
}
