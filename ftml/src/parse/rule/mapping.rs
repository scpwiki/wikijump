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
    /// Mapping of all tokens to the rules they possibly correspond with.
    ///
    /// This is the first tokens that could consistute the given rule,
    /// in order of precedence.
    ///
    /// An empty list means that this is a special token that shouldn't be used
    /// in this manner. It will of course fall back to interpreting this token
    /// as text, but will also produce an error for the user.
    pub static ref RULE_MAP: EnumMap<Token, Vec<Rule>> = {
        enum_map! {
            // Symbols
            Token::LeftBracket => vec![RULE_TODO], // TODO
            Token::RightBracket => vec![RULE_TODO], // TODO
            Token::LeftTag => vec![RULE_TODO], // TODO
            Token::LeftTagAnchor => vec![RULE_TODO], // TODO
            Token::LeftTagSpecial => vec![RULE_TODO], // TODO
            Token::RightTag => vec![RULE_TODO], // TODO
            Token::RightTagEnd => vec![RULE_TODO], // TODO
            Token::LeftAnchor => vec![RULE_TODO], // TODO
            Token::DoubleDash => vec![RULE_TODO], // TODO
            Token::TripleDash => vec![RULE_TODO], // TODO
            Token::Pipe => vec![RULE_TODO], // TODO
            Token::Equals => vec![RULE_TODO], // TODO
            Token::Quote => vec![RULE_TODO], // TODO
            Token::Heading => vec![RULE_TODO], // TODO
            Token::LineBreak => vec![RULE_LINE_BREAK],
            Token::ParagraphBreak => vec![RULE_TODO], // TODO
            Token::Whitespace => vec![RULE_TEXT],

            // Formatting
            Token::Bold => vec![RULE_BOLD],
            Token::Italics => vec![RULE_ITALICS],
            Token::Underline => vec![RULE_TODO], // TODO
            Token::Superscript => vec![RULE_TODO], // TODO
            Token::Subscript => vec![RULE_TODO], // TODO
            Token::LeftMonospace => vec![RULE_TODO], // TODO
            Token::RightMonospace => vec![RULE_TODO], // TODO
            Token::Color => vec![RULE_TODO], // TODO
            Token::Raw => vec![RULE_RAW],
            Token::LeftRaw => vec![RULE_RAW],
            Token::RightRaw => vec![RULE_TODO], // TODO

            // Links
            Token::LeftLink => vec![RULE_TODO], // TODO
            Token::RightLink => vec![RULE_TODO], // TODO

            // Tables
            Token::TableColumn => vec![RULE_TODO], // TODO
            Token::TableColumnTitle => vec![RULE_TODO], // TODO

            // Alignment
            Token::RightAlignOpen => vec![RULE_TODO], // TODO
            Token::RightAlignClose => vec![RULE_TODO], // TODO
            Token::LeftAlignOpen => vec![RULE_TODO], // TODO
            Token::LeftAlignClose => vec![RULE_TODO], // TODO
            Token::CenterAlignOpen => vec![RULE_TODO], // TODO
            Token::CenterAlignClose => vec![RULE_TODO], // TODO
            Token::JustifyAlignOpen => vec![RULE_TODO], // TODO
            Token::JustifyAlignClose => vec![RULE_TODO], // TODO

            // Text components
            Token::Identifier => vec![RULE_TEXT],
            Token::Email => vec![RULE_EMAIL],
            Token::Url => vec![RULE_URL],
            Token::String => vec![RULE_TODO], // TODO

            // Miscellaneous
            Token::LeftComment => vec![RULE_COMMENT],
            Token::RightComment => vec![RULE_TODO], // TODO
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
