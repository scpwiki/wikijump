/*
 * tokenize.rs
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

use crate::text::FullText;
use crate::{ExtractedToken, Token};

#[derive(Debug, Clone)]
pub struct Tokenization<'t> {
    tokens: Vec<ExtractedToken<'t>>,
    full_text: FullText<'t>,
}

impl<'t> Tokenization<'t> {
    #[inline]
    pub fn tokens<'r>(&'r self) -> &'r [ExtractedToken<'t>] {
        &self.tokens
    }

    #[inline]
    pub(crate) fn full_text(&self) -> FullText<'t> {
        self.full_text
    }
}

impl<'t> Into<Vec<ExtractedToken<'t>>> for Tokenization<'t> {
    #[inline]
    fn into(self) -> Vec<ExtractedToken<'t>> {
        self.tokens
    }
}

/// Take an input string and produce a list of tokens for consumption by the parser.
pub fn tokenize<'t>(log: &slog::Logger, text: &'t str) -> Tokenization<'t> {
    let log = &log.new(slog_o!(
        "filename" => slog_filename!(),
        "lineno" => slog_lineno!(),
        "function" => "tokenize",
        "text" => str!(text),
    ));

    info!(log, "Running lexer on text");

    let tokens = Token::extract_all(log, text);
    let full_text = FullText::new(text);

    Tokenization { tokens, full_text }
}
