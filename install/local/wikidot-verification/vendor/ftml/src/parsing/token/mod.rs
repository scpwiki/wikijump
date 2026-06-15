/*
 * parsing/token/mod.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2026 Wikijump Team
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

#[cfg(test)]
mod test;

mod lexer {
    // Since pest generates some code that clippy doesn't like
    #![allow(clippy::upper_case_acronyms, clippy::empty_docs)]

    // The actual parser definition, which we will re-export
    #[derive(Parser, Debug)]
    #[grammar = "parsing/lexer.pest"]
    pub struct TokenLexer;
}

use self::lexer::*;
use crate::utf16::Utf16IndexMap;
use pest::Parser;
use pest::iterators::Pair;
use std::ops::Range;
use strum_macros::IntoStaticStr;

/// Struct that represents a token in a specific text.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ExtractedToken<'a> {
    pub token: Token,
    pub slice: &'a str,
    pub span: Range<usize>,
}

impl ExtractedToken<'_> {
    /// Returns a new object with the same values, except with span refering to the byte indicies
    /// of the text if it were in UTF-16 rather than in UTF-8.
    #[must_use]
    pub fn to_utf16_indices(&self, map: &Utf16IndexMap) -> Self {
        // Copy fields
        let ExtractedToken { token, slice, span } = self.clone();

        // Map indices to UTF-16
        let start = map.get_index(span.start);
        let end = map.get_index(span.end);
        let span = start..end;

        // Output new ExtractedToken
        ExtractedToken { token, slice, span }
    }
}

/// Enum that represents the type of a parsed token. For a struct with additional context
/// surrounding the positioning and content of the token, see [`ExtractedToken`].
#[derive(
    Serialize, Deserialize, Enum, IntoStaticStr, Debug, Copy, Clone, PartialEq, Eq,
)]
#[serde(rename_all = "kebab-case")]
pub enum Token {
    //
    // Symbols
    //
    LeftBracket,
    LeftBracketAnchor,
    LeftBracketStar,
    RightBracket,
    LeftBlock,
    LeftBlockEnd,
    LeftBlockAnchor,
    LeftBlockStar,
    LeftMath,
    LeftParentheses,
    RightBlock,
    RightMath,
    RightParentheses,
    DoubleDash,
    TripleDash,
    DoubleTilde,
    LeftDoubleAngle,
    ClearFloatBoth,
    ClearFloatLeft,
    ClearFloatRight,
    Pipe,
    Equals,
    Colon,
    Underscore,
    Quote,
    Heading,

    //
    // Whitespace
    //
    LineBreak,
    ParagraphBreak,
    Whitespace,

    //
    // Formatting
    //
    Bold,
    Italics,
    Underline,
    Superscript,
    Subscript,
    LeftMonospace,
    RightMonospace,
    Color,
    Raw,
    LeftRaw,
    RightRaw,

    //
    // Lists
    //
    BulletItem,
    NumberedItem,

    //
    // Links
    //
    LeftLink,
    LeftLinkStar,
    RightLink,

    //
    // Tables
    //
    TableColumn,
    TableColumnRight,
    TableColumnCenter,
    TableColumnTitle,

    //
    // Text components
    //
    Identifier,
    Email,
    Url,
    Variable,
    DoubleQuote,
    EscapedDoubleQuote,
    EscapedBackslash,

    //
    // Miscellaneous
    //
    LeftComment,
    RightComment,
    InputStart,
    InputEnd,

    //
    // Catch-all case
    //
    Other,
}

impl Token {
    /// Extracts all tokens from the given text.
    /// # Errors
    /// Returns an error if something goes wrong with the parsing process. This will result in the
    /// only [`Token`] being a raw text containing all of the input.
    pub(crate) fn extract_all(text: &str) -> Vec<ExtractedToken<'_>> {
        debug!("Running lexer on input");

        match TokenLexer::parse(Rule::document, text) {
            Ok(pairs) => {
                debug!("Lexer produced pairs for processing");

                // Map pairs to tokens, and add a Token::InputStart at the beginning
                // Pest already adds a Token::InputEnd at the end
                let start = ExtractedToken {
                    token: Token::InputStart,
                    slice: "",
                    span: 0..0,
                };

                let mut tokens = vec![start];
                tokens.extend(pairs.map(Token::convert_pair));
                tokens
            }
            Err(error) => {
                // Return all of the input as one big raw text
                // and log this as an error, since it shouldn't be happening

                error!("Error while lexing input in pest: {error}");
                vec![ExtractedToken {
                    token: Token::Other,
                    slice: text,
                    span: 0..text.len(),
                }]
            }
        }
    }

    /// Converts a single [`Pair`] from pest into its corresponding [`ExtractedToken`].
    fn convert_pair(pair: Pair<Rule>) -> ExtractedToken {
        // Extract values from the Pair
        let rule = pair.as_rule();
        let slice = pair.as_str();
        let start = pair.as_span().start();
        let end = pair.as_span().end();
        let span = start..end;

        // Get matching Token.
        let token = Token::get_from_rule(rule);
        trace!("Converting pair '{:?}' into token {}", rule, token.name());

        ExtractedToken { token, slice, span }
    }

    /// Maps each pest [`Rule`] to its corresponding [`Token`].
    fn get_from_rule(rule: Rule) -> Token {
        match rule {
            // Symbols
            Rule::left_comment => Token::LeftComment,
            Rule::right_comment => Token::RightComment,
            Rule::left_bracket => Token::LeftBracket,
            Rule::left_bracket_anchor => Token::LeftBracketAnchor,
            Rule::left_bracket_star => Token::LeftBracketStar,
            Rule::right_bracket => Token::RightBracket,
            Rule::left_parens => Token::LeftParentheses,
            Rule::right_parens => Token::RightParentheses,
            Rule::left_block => Token::LeftBlock,
            Rule::left_block_end => Token::LeftBlockEnd,
            Rule::left_block_anchor => Token::LeftBlockAnchor,
            Rule::left_block_star => Token::LeftBlockStar,
            Rule::left_math => Token::LeftMath,
            Rule::right_block => Token::RightBlock,
            Rule::right_math => Token::RightMath,
            Rule::color => Token::Color,
            Rule::double_dash => Token::DoubleDash,
            Rule::triple_dash => Token::TripleDash,
            Rule::double_tilde => Token::DoubleTilde,
            Rule::left_double_angle => Token::LeftDoubleAngle,
            Rule::clear_float => Token::ClearFloatBoth,
            Rule::clear_float_left => Token::ClearFloatLeft,
            Rule::clear_float_right => Token::ClearFloatRight,
            Rule::pipe => Token::Pipe,
            Rule::colon => Token::Colon,
            Rule::underscore => Token::Underscore,
            Rule::equals => Token::Equals,
            Rule::quote => Token::Quote,
            Rule::heading => Token::Heading,

            // Whitespace
            Rule::line_break => Token::LineBreak,
            Rule::paragraph_break => Token::ParagraphBreak,
            Rule::space => Token::Whitespace,

            // Formatting
            Rule::bold => Token::Bold,
            Rule::italics => Token::Italics,
            Rule::underline => Token::Underline,
            Rule::superscript => Token::Superscript,
            Rule::subscript => Token::Subscript,
            Rule::left_monospace => Token::LeftMonospace,
            Rule::right_monospace => Token::RightMonospace,
            Rule::raw => Token::Raw,
            Rule::left_raw => Token::LeftRaw,
            Rule::right_raw => Token::RightRaw,

            // Lists
            Rule::bullet_item => Token::BulletItem,
            Rule::numbered_item => Token::NumberedItem,

            // Links
            Rule::left_link => Token::LeftLink,
            Rule::left_link_star => Token::LeftLinkStar,
            Rule::right_link => Token::RightLink,

            // Tables
            Rule::table_column => Token::TableColumn,
            Rule::table_column_right => Token::TableColumnRight,
            Rule::table_column_center => Token::TableColumnCenter,
            Rule::table_column_title => Token::TableColumnTitle,

            // Text components
            Rule::identifier => Token::Identifier,
            Rule::email => Token::Email,
            Rule::url => Token::Url,
            Rule::variable => Token::Variable,
            Rule::double_quote => Token::DoubleQuote,
            Rule::escaped_double_quote => Token::EscapedDoubleQuote,
            Rule::escaped_backslash => Token::EscapedBackslash,

            // Other
            Rule::other => Token::Other,
            Rule::EOI => Token::InputEnd,

            // Invalid
            Rule::document | Rule::token => {
                panic!("Received invalid pest rule: {rule:?}")
            }
        }
    }

    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }
}
