/*
 * parse/ahead.rs
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

//! Module for look-ahead checking.
//!
//! This contains implementations of eager functions that try to interpret the
//! upcoming tokens as a particular object (e.g. seeing a `[[` and you see if it's a module).
//!
//! The parser is not disambiguous because any string of tokens can be interpreted
//! as raw text as a fallback, which is how Wikidot does it.

use super::stack::Stack;
use super::token::{ExtractedToken, Token};
use crate::tree::Element;

/// Main function which takes the current stack and upcoming tokens to attempt to match against it.
pub fn consume<'r, 'a>(
    log: &slog::Logger,
    stack: &'r mut Stack<'a>,
    extract: &ExtractedToken<'a>,
    next: &[ExtractedToken<'a>],
) {
    let ExtractedToken { token, slice, span } = extract;

    debug!(
        log,
        "Attempting to consume tokens in different look-aheads";
        "token" => token,
        "next-len" => next.len(),
        "stack-len" => stack.len(),
    );

    match token {
        Token::Identifier | Token::Text | Token::Whitespace => {
            stack.append(Element::Text(slice));
        }
        Token::LeftTag => try_block(),
        Token::LeftTagSpecial => try_special_block(),
        _ => todo!(),
    }

    todo!()
}

fn try_block() {
    todo!()
}

fn try_special_block() {
    todo!()
}
