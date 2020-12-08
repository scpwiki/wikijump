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
            Token::LeftBracket => vec![RULE_LINK_SINGLE, RULE_TEXT],
            Token::LeftBracketSpecial => vec![RULE_LINK_SINGLE_NEW_TAB],
            Token::RightBracket => vec![RULE_TEXT],
            Token::LeftTag => vec![RULE_TODO], // TODO
            Token::LeftTagAnchor => vec![RULE_TODO], // TODO
            Token::LeftTagSpecial => vec![RULE_TODO], // TODO
            Token::RightTag => vec![],
            Token::RightTagEnd => vec![],
            Token::LeftAnchor => vec![RULE_TODO], // TODO
            Token::DoubleDash => vec![RULE_STRIKETHROUGH, RULE_DASH],
            Token::TripleDash => vec![RULE_TODO], // TODO
            Token::ClearFloatNeutral => vec![RULE_TODO], // TODO
            Token::ClearFloatCenter => vec![RULE_TODO], // TODO
            Token::ClearFloatLeft => vec![RULE_TODO], // TODO
            Token::ClearFloatRight => vec![RULE_TODO], // TODO
            Token::Pipe => vec![RULE_TEXT],
            Token::Equals => vec![RULE_TODO, RULE_TEXT], // TODO
            Token::Quote => vec![RULE_TODO], // TODO
            Token::Heading => vec![RULE_TODO, RULE_TEXT], // TODO
            Token::LineBreak => vec![RULE_LINE_BREAK],
            Token::ParagraphBreak => vec![RULE_TODO], // TODO
            Token::Whitespace => vec![RULE_TEXT],

            // Formatting
            Token::Bold => vec![RULE_BOLD],
            Token::Italics => vec![RULE_ITALICS],
            Token::Underline => vec![RULE_UNDERLINE],
            Token::Superscript => vec![RULE_SUPERSCRIPT],
            Token::Subscript => vec![RULE_SUBSCRIPT],
            Token::LeftMonospace => vec![RULE_MONOSPACE],
            Token::RightMonospace => vec![],
            Token::Color => vec![RULE_COLOR],
            Token::Raw => vec![RULE_RAW],
            Token::LeftRaw => vec![RULE_RAW],
            Token::RightRaw => vec![],

            // Links
            Token::LeftLink => vec![RULE_TODO], // TODO
            Token::LeftLinkSpecial => vec![RULE_TODO], // TODO
            Token::RightLink => vec![],

            // Tables
            Token::TableColumn => vec![RULE_TODO], // TODO
            Token::TableColumnTitle => vec![RULE_TODO], // TODO

            // Alignment
            Token::RightAlignOpen => vec![RULE_TODO], // TODO
            Token::RightAlignClose => vec![],
            Token::LeftAlignOpen => vec![RULE_TODO], // TODO
            Token::LeftAlignClose => vec![],
            Token::CenterAlignOpen => vec![RULE_TODO], // TODO
            Token::CenterAlignClose => vec![],
            Token::JustifyAlignOpen => vec![RULE_TODO], // TODO
            Token::JustifyAlignClose => vec![],

            // Text components
            Token::Identifier => vec![RULE_TEXT],
            Token::Email => vec![RULE_EMAIL],
            Token::Url => vec![RULE_URL],
            Token::String => vec![RULE_TEXT],

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
