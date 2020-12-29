/*
 * parse/parser.rs
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

use super::prelude::*;
use crate::tokenize::Tokenization;

#[derive(Debug, Clone)]
pub struct Parser<'l, 'r, 't> {
    log: &'l slog::Logger,
    tokens: UpcomingTokens<'r, 't>,
    full_text: FullText<'t>,
}

impl<'l, 'r, 't> Parser<'l, 'r, 't> {
    pub(crate) fn new(log: &'l slog::Logger, tokenization: &'r Tokenization<'t>) -> Self {
        let tokens = UpcomingTokens::from(tokenization.tokens());
        let full_text = tokenization.full_text();

        Parser {
            log,
            tokens,
            full_text,
        }
    }

    #[inline]
    pub fn tokens(&self) -> UpcomingTokens<'r, 't> {
        self.tokens
    }

    #[inline]
    pub fn full_text(&self) -> FullText<'t> {
        self.full_text
    }
}
