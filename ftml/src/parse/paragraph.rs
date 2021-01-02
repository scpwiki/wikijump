/*
 * parse/paragraph.rs
 *
 * ftml - Library to parse Wikidot text
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

use super::condition::ParseCondition;
use super::consume::consume;
use super::parser::Parser;
use super::prelude::*;
use super::rule::Rule;
use super::stack::ParagraphStack;
use super::token::Token;

/// Function to iterate over tokens to produce elements in paragraphs.
///
/// Originally in `parse()`, but was moved out to allow paragraph
/// extraction deeper in code, such as in the `try_paragraph`
/// collection helper.
pub fn gather_paragraphs<'r, 't>(
    log: &slog::Logger,
    parser: &mut Parser<'r, 't>,
    rule: Rule,
    close_conditions: &[ParseCondition],
    invalid_conditions: &[ParseCondition],
) -> ParseResult<'t, Vec<Element<'t>>>
where
    'r: 't,
{
    info!(log, "Gathering paragraphs until ending");

    // Update parser rule
    parser.set_rule(rule);

    // Build paragraph stack
    let mut stack = ParagraphStack::new(log);

    loop {
        // Consume tokens to produce the next element
        let result = match parser.current().token {
            // Avoid an unnecessary Token::Null and just exit
            Token::InputEnd => {
                if close_conditions.is_empty() {
                    debug!(log, "Hit the end of input, terminating token iteration");

                    break;
                } else {
                    debug!(log, "Hit the end of input, producing error");

                    return Err(parser.make_error(ParseErrorKind::EndOfInput));
                }
            }

            // If we've hit a paragraph break, then finish the current paragraph
            Token::ParagraphBreak => {
                debug!(
                    log,
                    "Hit a paragraph break, creating a new paragraph container",
                );

                // Paragraph break -- end the paragraph and start a new one!
                stack.end_paragraph();

                // We must manually bump up this pointer because
                // we 'continue' here, skipping the usual pointer update.
                parser.step()?;
                continue;
            }

            // Ending the paragraph prematurely due to the element ending
            _ if parser.evaluate_any(close_conditions) => {
                debug!(
                    log,
                    "Hit closing condition, returning parsing success";
                    "token" => parser.current().token,
                );

                return stack.into_result();
            }

            // Ending the paragraph prematurely due to an error
            _ if parser.evaluate_any(invalid_conditions) => {
                debug!(
                    log,
                    "Hit failure condition, returning parsing failure";
                    "token" => parser.current().token,
                );

                return Err(parser.make_error(ParseErrorKind::RuleFailed));
            }

            // Produce consumption from this token pointer
            _ => {
                debug!(log, "Trying to consume tokens to produce element");
                consume(log, parser)
            }
        };

        match result {
            Ok(ParseSuccess {
                item,
                mut exceptions,
            }) => {
                debug!(log, "Tokens successfully consumed to produce element");

                // Add the new element to the list
                stack.push_element(item);

                // Process exceptions
                stack.push_exceptions(&mut exceptions);
            }
            Err(error) => {
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
        }
    }

    stack.into_result()
}
