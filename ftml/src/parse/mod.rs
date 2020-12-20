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
mod result;
mod rule;
mod stack;
mod token;

use self::consume::consume;
use self::rule::Consumption;
use self::stack::ParseStack;
use crate::tokenize::Tokenization;
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
    let mut stack = ParseStack::new();
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

    while !tokens.is_empty() {
        // Consume tokens to produce the next element
        let consumption = {
            let (extracted, remaining) = tokens
                .split_first() //
                .expect("Tokens list is empty");

            consume(log, extracted, remaining, full_text)
        };

        match consumption {
            Consumption::Success {
                item,
                remaining,
                exceptions,
            } => {
                debug!(log, "Tokens successfully consumed to produce element");

                // Update remaining tokens
                //
                // The new value is a subslice of tokens,
                // equivalent to &tokens[offset..] but without
                // needing to assert bounds.
                tokens = remaining;

                // Add the new element to the list
                stack.push_element(item);

                // Process exceptions
                for exception in exceptions {
                    match exception {
                        ParseException::Error(error) => stack.push_error(error),
                        ParseException::Style(style) => stack.push_style(style),
                    }
                }
            }
            Consumption::Failure { error } => {
                info!(
                    log,
                    "Token consumption failed, returned error";
                    "error-token" => error.token(),
                    "error-rule" => error.rule(),
                    "error-span-start" => error.span().start,
                    "error-span-end" => error.span().end,
                    "error-kind" => error.kind().name(),
                );

                // Append the error
                stack.push_error(error);
            }
        };
    }

    info!(log, "Finished running parser, returning gathered elements");
    stack.into_syntax_tree()
}
