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

#[derive(Debug, Copy, Clone)]
pub enum UpcomingTokens<'r, 't> {
    All {
        tokens: &'r [ExtractedToken<'t>],
    },
    Split {
        extracted: &'r ExtractedToken<'t>,
        remaining: &'r [ExtractedToken<'t>],
    },
}

impl<'r, 't> UpcomingTokens<'r, 't> {
    pub fn split(&self) -> Option<(&'r ExtractedToken<'t>, &'r [ExtractedToken<'t>])> {
        match self {
            UpcomingTokens::All { tokens } => tokens.split_first(),
            UpcomingTokens::Split {
                extracted,
                remaining,
            } => Some((extracted, remaining)),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            UpcomingTokens::All { tokens } => tokens.is_empty(),
            UpcomingTokens::Split { .. } => false,
        }
    }
}

impl<'r, 't> From<&'r [ExtractedToken<'t>]> for UpcomingTokens<'r, 't> {
    #[inline]
    fn from(tokens: &'r [ExtractedToken<'t>]) -> Self {
        UpcomingTokens::All { tokens }
    }
}

impl<'r, 't> From<(&'r ExtractedToken<'t>, &'r [ExtractedToken<'t>])>
    for UpcomingTokens<'r, 't>
{
    #[inline]
    fn from(
        (extracted, remaining): (&'r ExtractedToken<'t>, &'r [ExtractedToken<'t>]),
    ) -> Self {
        UpcomingTokens::Split {
            extracted,
            remaining,
        }
    }
}
