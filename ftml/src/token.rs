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
pub fn tokenize<'t>(log: &slog::Logger, text: &'t str) -> Vec<ExtractedToken<'t>> {
    let log = &log.new(slog_o!("function" => "tokenize", "text" => str!(text)));

    info!(log, "Running lexer on text");
    Token::extract_all(log, text)
}
