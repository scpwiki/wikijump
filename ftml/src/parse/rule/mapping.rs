/*
 * parse/rule/mapping.rs
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

use super::{impls::*, Rule};
use crate::parse::token::{ExtractedToken, Token};
use enum_map::EnumMap;

// NOTE: Empty rule lists mean that it's an invalid token.
//       For instance, such as a ']]]' with no opening tag.
//       It will fall back to the text anyways, this communicates
//       intent better.

// TODO: Actually, we need to convert ending tokens into lookbacks
//       at current stack / context to try and end ongoing containers.

lazy_static! {
    pub static ref RULE_MAP: EnumMap<Token, Vec<Rule>> = {
        enum_map! {
            // Symbols
            Token::LeftBracket => vec![],
            Token::RightBracket => vec![],
            Token::Pipe => vec![RULE_TEXT],
            Token::LeftTag => vec![],
            Token::LeftTagSpecial => vec![],
            Token::RightTag => vec![],
            Token::LeftAnchor => vec![],
            Token::Equals => vec![RULE_TEXT],
            Token::DoubleDash => vec![],
            Token::TripleDash => vec![],
            Token::LineBreak => vec![],
            Token::ParagraphBreak => vec![],
            Token::Whitespace => vec![RULE_TEXT],
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
            Token::Identifier => vec![RULE_TEXT],
            Token::Email => vec![RULE_EMAIL],
            Token::Url => vec![RULE_URL],

            // Fallback
            Token::Text => vec![RULE_TEXT],
        }
    };
}

#[inline]
pub fn rules_for_token<'a>(extract: &'a ExtractedToken) -> &'a [Rule] {
    &RULE_MAP[extract.token]
}
