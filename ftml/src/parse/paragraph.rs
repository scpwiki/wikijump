/*
 * parse/paragraph.rs
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

use crate::text::FullText;
use crate::tree::Element;
use super::consume::consume;
use super::rule::{GenericConsumption, Consumption};
use super::stack::ParseStack;
use super::token::{ExtractedToken, Token};
use super::ParseException;

/// Function to iterate over tokens to produce elements in paragraphs.
///
/// Originally in `parse()`, but was moved out to allow paragraph
/// extraction deeper in code, such as in the `try_paragraph`
/// collection helper.
pub fn gather_paragraphs<'l, 'r, 't>(
    log: &'l slog::Logger,
    mut extracted: &'r ExtractedToken<'t>,
    mut remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> ParseStack<'l, 't> {
    info!(log, "Gathering paragraphs until ending");

    let mut stack = ParseStack::new(log);

    while !remaining.is_empty() {
        // TODO: figure out how to iterate
        let x = ();

        // Consume tokens to produce the next element
        let consumption: GenericConsumption<'r, 't, Element<'t>> = match extracted.token {
            // Avoid an unnecessary Token::Null and just exit
            Token::InputEnd => {
                debug!(log, "Hit the end of input, terminating token iteration");
                break;
            }

            // If we've hit a paragraph break, then finish the current paragraph.
            Token::ParagraphBreak => {
                debug!(log, "Hit a paragraph break, creating a new paragraph container");
                stack.end_paragraph();
                continue;
            }

            // Produce consumption from this token pointer
            _ => {
                debug!(log, "Trying to consume tokens to produce element");
                consume(log, &extracted, remaining, full_text)
            }
        };

        match consumption {
            Consumption::Success {
                item,
                remaining: new_remaining,
                exceptions,
            } => {
                debug!(log, "Tokens successfully consumed to produce element");

                // Update remaining tokens
                //
                // The new value is a subslice of tokens,
                // equivalent to &tokens[offset..] but without
                // needing to assert bounds.
                remaining = new_remaining;

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

    stack
}
