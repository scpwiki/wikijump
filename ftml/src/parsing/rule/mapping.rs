/*
 * parsing/rule/mapping.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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
use crate::parsing::token::{ExtractedToken, Token};
use enum_map::EnumMap;

lazy_static! {
    /// Mapping of all tokens to the rules they possibly correspond with.
    ///
    /// This is the first tokens that could consistute the given rule,
    /// in order of precedence.
    ///
    /// An empty list means that this is a special token that shouldn't be used
    /// in this manner. It will of course fall back to interpreting this token
    /// as text, but will also produce a warning for the user.
    pub static ref RULE_MAP: EnumMap<Token, Vec<Rule>> = {
        enum_map! {
            // Symbols
            Token::LeftBracket => vec![RULE_LINK_SINGLE, RULE_TEXT],
            Token::LeftBracketAnchor => vec![RULE_LINK_ANCHOR],
            Token::LeftBracketStar => vec![RULE_LINK_SINGLE_NEW_TAB],
            Token::RightBracket => vec![RULE_TEXT],
            Token::LeftBlock => vec![RULE_BLOCK],
            Token::LeftBlockEnd => vec![],
            Token::LeftBlockStar => vec![RULE_BLOCK_STAR],
            Token::RightBlock => vec![],
            Token::DoubleDash => vec![RULE_STRIKETHROUGH, RULE_DASH],
            Token::TripleDash => vec![RULE_HORIZONTAL_RULE],
            Token::LeftDoubleAngle => vec![RULE_DOUBLE_ANGLE],
            Token::ClearFloatNeutral => vec![RULE_TODO], // TODO
            Token::ClearFloatCenter => vec![RULE_TODO], // TODO
            Token::ClearFloatLeft => vec![RULE_TODO], // TODO
            Token::ClearFloatRight => vec![RULE_TODO], // TODO
            Token::Pipe => vec![RULE_TEXT],
            Token::Equals => vec![RULE_TEXT],
            Token::Underscore => vec![RULE_TEXT],
            Token::Quote => vec![RULE_DOUBLE_ANGLE, RULE_TEXT],
            Token::Heading => vec![RULE_TEXT],
            Token::ParagraphBreak => vec![RULE_LINE_BREAK_PARAGRAPH],
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

            // Lists
            Token::BulletItem => vec![RULE_TEXT],
            Token::NumberedItem => vec![RULE_TEXT],

            // Links
            Token::LeftLink => vec![RULE_LINK_TRIPLE],
            Token::LeftLinkStar => vec![RULE_LINK_TRIPLE_NEW_TAB],
            Token::RightLink => vec![],

            // Tables
            Token::TableColumn => vec![RULE_TODO], // TODO
            Token::TableColumnTitle => vec![RULE_TODO], // TODO

            // Text components
            Token::Identifier => vec![RULE_TEXT],
            Token::Email => vec![RULE_EMAIL],
            Token::Url => vec![RULE_URL],
            Token::Variable => vec![RULE_TEXT],
            Token::String => vec![RULE_TEXT],

            // Input boundaries
            Token::LineBreak => vec![
                // Start-of-line rules
                RULE_CENTER,
                RULE_HEADER,
                RULE_BLOCKQUOTE,
                RULE_LIST,

                // Consume newline for blocks
                RULE_BLOCK_SKIP,

                // Normal rule handler
                RULE_LINE_BREAK,
            ],
            Token::InputStart => vec![
                // Start-of-line rules
                RULE_CENTER,
                RULE_HEADER,
                RULE_BLOCKQUOTE,
                RULE_LIST,

                // Normal rule handler
                RULE_NULL,
            ],
            Token::InputEnd => vec![RULE_NULL],

            // Miscellaneous
            Token::LeftComment => vec![RULE_COMMENT],
            Token::RightComment => vec![],

            // Fallback
            Token::Other => vec![RULE_TEXT],
        }
    };
}

#[inline]
pub fn get_rules_for_token(current: &ExtractedToken) -> &'static [Rule] {
    &RULE_MAP[current.token]
}
