/*
 * token.rs
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

//! Module for functionality related to the tokenization step in parsing.

use crate::{ExtractedToken, Token};

/// Take an input string and produce a list of tokens for consumption by the parser.
pub fn tokenize<'t>(log: &slog::Logger, text: &'t str) -> Tokenization<'t> {
    let log = &log.new(slog_o!("function" => "tokenize", "text" => str!(text)));

    info!(log, "Running lexer on text");
    let tokens = Token::extract_all(log, text);

    Tokenization { tokens, text }
}

/// Output of `tokenize()` to be consumed by `parse()`.
///
/// This is a wrapper struct around `Vec<ExtractedToken>` but which also
/// preserves the original input text along with it. This allows some
/// text operations (such as joining raw token slices) that would not
/// be possible otherwise.
///
/// Because it is internal, we can be sure that the passed `text` definitely
/// corresponds to the `tokens` produces by lexing.
#[derive(Debug)]
pub struct Tokenization<'t> {
    tokens: Vec<ExtractedToken<'t>>,
    text: &'t str,
}

impl<'t> Tokenization<'t> {
    #[inline]
    pub fn tokens(&self) -> &[ExtractedToken<'t>] {
        &self.tokens
    }

    #[inline]
    pub fn text(&self) -> &'t str {
        self.text
    }
}

impl<'t> Into<Vec<ExtractedToken<'t>>> for Tokenization<'t> {
    #[inline]
    fn into(self) -> Vec<ExtractedToken<'t>> {
        let Tokenization { tokens, .. } = self;

        tokens
    }
}
