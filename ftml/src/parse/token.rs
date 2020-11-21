/*
 * parse/token.rs
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

use pest::error::Error as PestError;
use pest::Parser;
use pest::iterators::Pair;
use std::ops::Range;
use strum_macros::IntoStaticStr;

#[derive(Debug, Copy, Clone, Parser)]
#[grammar = "parse/lexer.pest"]
struct TokenLexer;

type LexerError = PestError<Rule>;

#[derive(Debug, Clone, PartialEq)]
pub struct ExtractedToken<'a> {
    pub token: Token,
    pub slice: &'a str,
    pub span: Range<usize>,
}

#[derive(Enum, IntoStaticStr, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Token {
    //
    // Symbols
    //
    LeftBracket,
    RightBracket,
    Pipe,
    LeftTag,
    LeftTagSpecial,
    RightTag,
    LeftAnchor,
    Equals,
    DoubleDash,
    TripleDash,
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
    // Links
    //
    LeftLink,
    RightLink,

    //
    // Tables
    //
    TableColumn,
    TableColumnTitle,

    //
    // Alignment
    //
    RightAlignOpen,
    RightAlignClose,
    LeftAlignOpen,
    LeftAlignClose,
    CenterAlignOpen,
    CenterAlignClose,
    JustifyAlignOpen,
    JustifyAlignClose,

    //
    // Text components
    //
    Identifier,
    Email,
    Url,

    //
    // Catch-all case
    //
    Other,
}

impl Token {
    pub fn extract_all<'a>(logger: &slog::Logger, text: &'a str) -> Vec<ExtractedToken<'a>> {
        debug!(logger, "Running lexer on input");

        match TokenLexer::parse(Rule::document, text) {
            Ok(pairs) => {
                info!(logger, "Lexer produced pairs for processing");

                pairs.filter_map(|pair| Token::convert_pair(logger, pair)).collect()
            }
            Err(error) => {
                error!(logger, "Error while lexing input in pest: {}", error);

                // TODO better handling lol
                // Return all of the input as one big raw text

                vec![ExtractedToken {
                    token: Token::Other,
                    slice: text,
                    span: 0..text.len(),
                }]
            }
        }
    }

    /// Converts a single `Pair` from pest into its corresponding `ExtractedToken`.
    fn convert_pair<'a>(logger: &slog::Logger, pair: Pair<'a, Rule>) -> Option<ExtractedToken<'a>> {
        let rule = pair.as_rule();
        let slice = pair.as_str();
        let start = pair.as_span().start();
        let end = pair.as_span().end();

        // Get matching Token, if any.
        // (Returns if we're skipping this Pair)
        let token = match Token::get_from_rule(rule) {
            Some(token) => token,
            None => return None,
        };

        debug!(
            logger,
            "Converting pair '{:?}' into token", rule;
            "token" => token.name(),
            "slice" => pair.as_str(),
            "span-start" => start,
            "span-end" => end,
        );

        let span = start..end;
        Some(ExtractedToken {
            token,
            slice,
            span,
        })
    }

    /// Mapping of a pest `Rule` to its corresponding `Token` enum.
    fn get_from_rule(rule: Rule) -> Option<Token> {
        let token = match rule {
            // Symbols
            Rule::left_bracket => Token::LeftBracket,
            Rule::right_bracket => Token::RightBracket,
            Rule::pipe => Token::Pipe,
            _ => todo!(),
        };

        Some(token)
    }

    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }
}

impl slog::Value for Token {
    fn serialize(
        &self,
        _: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, self.name())
    }
}

#[test]
fn test_tokens() {
    let logger = crate::build_logger();

    macro_rules! test {
        ($input:expr, $expected:expr,) => {{
            let result = Token::extract_all(&logger, $input);

            assert_eq!(
                result, $expected,
                "Extracted tokens (left) from lexer did not match expected (right)",
            );
        }};
    }

    // Test cases:

    test!(
        "text",
        vec![ExtractedToken {
            token: Token::Identifier,
            slice: "text",
            span: 0..4,
        }],
    );

    test!(
        "-- doubleDash",
        vec![
            ExtractedToken {
                token: Token::DoubleDash,
                slice: "--",
                span: 0..2,
            },
            ExtractedToken {
                token: Token::Whitespace,
                slice: " ",
                span: 2..3,
            },
            ExtractedToken {
                token: Token::Identifier,
                slice: "doubleDash",
                span: 3..13,
            },
        ],
    );

    test!(
        "--doubleDash",
        vec![
            ExtractedToken {
                token: Token::DoubleDash,
                slice: "--",
                span: 0..2,
            },
            ExtractedToken {
                token: Token::Identifier,
                slice: "doubleDash",
                span: 2..12,
            },
        ],
    );
}
