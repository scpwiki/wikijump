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

lazy_static! {
    pub static ref RULE_MAP: EnumMap<Token, Vec<Rule>> = {
        enum_map! {
            // Symbols
            Token::LeftBracket => vec![],
            Token::RightBracket => vec![],
            Token::LeftTag => vec![],
            Token::LeftTagAnchor => vec![],
            Token::LeftTagSpecial => vec![],
            Token::RightTag => vec![],
            Token::RightTagEnd => vec![],
            Token::LeftAnchor => vec![],
            Token::DoubleDash => vec![],
            Token::TripleDash => vec![],
            Token::Pipe => vec![],
            Token::Equals => vec![],
            Token::Quote => vec![],
            Token::Heading => vec![],
            Token::LineBreak => vec![RULE_LINE_BREAK],
            Token::ParagraphBreak => vec![],
            Token::Whitespace => vec![RULE_TEXT],

            // Formatting
            Token::Bold => vec![RULE_BOLD],
            Token::Italics => vec![RULE_ITALICS],
            Token::Underline => vec![],
            Token::Superscript => vec![],
            Token::Subscript => vec![],
            Token::LeftMonospace => vec![],
            Token::RightMonospace => vec![],
            Token::Color => vec![],
            Token::Raw => vec![RULE_RAW],
            Token::LeftRaw => vec![RULE_RAW],
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
            Token::String => vec![],

            // Miscellaneous
            Token::LeftComment => vec![RULE_COMMENT],
            Token::RightComment => vec![],
            Token::InputEnd => vec![RULE_NULL],

            // Fallback
            Token::Other => vec![RULE_TEXT],
        }
    };
}

#[inline]
pub fn rules_for_token(extracted: &ExtractedToken) -> &'static [Rule] {
    &RULE_MAP[extracted.token]
}
