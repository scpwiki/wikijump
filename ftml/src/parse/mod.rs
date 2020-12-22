/*
 * parse/mod.rs
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

#[macro_use]
mod macros;

mod consume;
mod error;
mod paragraph;
mod result;
mod rule;
mod stack;
mod token;

use self::consume::{consume, Consumption, GenericConsumption};
use self::paragraph::{gather_paragraphs, process_consumption};
use self::stack::ParseStack;
use crate::tokenize::Tokenization;
use crate::tree::Element;
use crate::tree::SyntaxTree;

pub use self::error::{ParseError, ParseErrorKind, ParseException};
pub use self::result::ParseResult;
pub use self::token::{ExtractedToken, Token};

/// Parse through the given tokens and produce an AST.
///
/// This takes a list of `ExtractedToken` items produced by `tokenize()`.
pub fn parse<'r, 't>(
    log: &slog::Logger,
    tokenization: &'r Tokenization<'t>,
) -> ParseResult<SyntaxTree<'t>>
where
    'r: 't,
{
    // Set up variables
    let mut tokens = tokenization.tokens();
    let full_text = tokenization.full_text();

    // Logging setup
    let log = &log.new(slog_o!(
        "filename" => slog_filename!(),
        "lineno" => slog_lineno!(),
        "function" => "parse",
        "tokens-len" => tokens.len(),
    ));

    // Run through tokens until finished
    info!(log, "Running parser on tokens");

    let mut stack = ParseStack::new(log);

    while !tokens.is_empty() {
        let (extracted, remaining) = tokens
            .split_first() //
            .expect("Tokens list is empty");

        // Consume tokens to produce the next element
        let consumption = match extracted.token {
            // Avoid an unnecessary Token::Null and just exit
            Token::InputEnd => {
                debug!(log, "Hit the end of input, terminating token iteration");
                break;
            }

            // If we've hit a paragraph break, then finish the current paragraph.
            Token::ParagraphBreak => {
                debug!(
                    log,
                    "Hit a paragraph break, creating a new paragraph container",
                );
                stack.end_paragraph();
                continue;
            }

            // Produce consumption from this token pointer
            _ => {
                debug!(log, "Trying to consume tokens to produce element");
                consume(log, &extracted, remaining, full_text)
            }
        };

        process_consumption(log, consumption, &mut stack, &mut tokens);
    }

    info!(log, "Finished running parser, returning gathered elements");
    stack.into_syntax_tree()
}
