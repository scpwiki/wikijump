/*
 * parse/consume.rs
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

use super::rule::{Rule, RuleResult};
use super::token::{ExtractedToken, Token};
use enum_map::EnumMap;

lazy_static! {
    static ref PATTERN_MAP: EnumMap<Token, Vec<Rule>> = {
        enum_map! {
            // Symbols
            Token::LeftBracket => vec![],
            Token::RightBracket => vec![],
            Token::Pipe => vec![],
            Token::LeftTag => vec![],
            Token::LeftTagSpecial => vec![],
            Token::RightTag => vec![],
            Token::LeftAnchor => vec![],
            Token::Equals => vec![],
            Token::DoubleDash => vec![],
            Token::TripleDash => vec![],
            Token::LineBreak => vec![],
            Token::ParagraphBreak => vec![],
            Token::Whitespace => vec![],
            Token::Bold => vec![],
            Token::Italics => vec![],
            Token::Underline => vec![],
            Token::Superscript => vec![],
            Token::Subscript => vec![],
            Token::LeftMonospace => vec![],
            Token::RightMonospace => vec![],
            Token::Color => vec![],

            // Formatting
            Token::Raw => vec![],
            Token::LeftRaw => vec![],
            Token::RightRaw => vec![],

            // Links
            Token::LeftLink => vec![],
            Token::RightLink => vec![],

            // Tables
            Token::TableColumn => vec![],
            Token::TableColumnTitle => vec![],

            // Alignment
            Token::RightAlignOpen => vec![],
            Token::RightAlignClose => vec![],
            Token::LeftAlignOpen => vec![],
            Token::LeftAlignClose => vec![],
            Token::CenterAlignOpen => vec![],
            Token::CenterAlignClose => vec![],
            Token::JustifyAlignOpen => vec![],
            Token::JustifyAlignClose => vec![],

            // Text components
            Token::Identifier => vec![],
            Token::Email => vec![],
            Token::Url => vec![],

            // Fallback
            Token::Text => vec![],
        }
    };
}

/// Main function that consumes tokens to produce a single element, then returns.
pub fn consume<'a>(
    log: &slog::Logger,
    extract: &ExtractedToken<'a>,
    next: &[ExtractedToken<'a>],
) -> RuleResult<'a> {
    let ExtractedToken { token, slice, span } = extract;

    debug!(
        log,
        "Attempting to consume tokens in different look-aheads";
        "token" => token,
        "next-len" => next.len(),
    );

    // TODO match on token, get pattern attempts

    todo!()
}
