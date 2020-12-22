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

use super::consume::{consume, Consumption, GenericConsumption};
use super::rule::Rule;
use super::stack::ParagraphStack;
use super::token::Token;
use super::upcoming::UpcomingTokens;
use super::{ParseError, ParseErrorKind, ParseException};
use crate::text::FullText;
use crate::tree::Element;

/// Function to iterate over tokens to produce elements in paragraphs.
///
/// Originally in `parse()`, but was moved out to allow paragraph
/// extraction deeper in code, such as in the `try_paragraph`
/// collection helper.
///
/// Allows the caller to either pass in a full token list
/// (such as `parse()` starting at the beginning) or split
/// (such as `try_consume()`, in the middle of parsing).
///
/// See the `UpcomingTokens` enum for more information.
pub fn gather_paragraphs<'l, 'r, 't>(
    log: &'l slog::Logger,
    mut tokens: UpcomingTokens<'r, 't>,
    full_text: FullText<'t>,
    rule: Rule,
    close_tokens: &[Token],
    invalid_tokens: &[Token],
) -> GenericConsumption<'r, 't, Vec<Element<'t>>>
where
    'r: 't,
{
    info!(log, "Gathering paragraphs until ending");

    let mut stack = ParagraphStack::new(log);

    while let Some((extracted, remaining)) = tokens.split() {
        // Consume tokens to produce the next element
        let consumption = match extracted.token {
            // Avoid an unnecessary Token::Null and just exit
            Token::InputEnd => {
                if close_tokens.is_empty() {
                    debug!(log, "Hit the end of input, terminating token iteration");

                    break;
                } else {
                    debug!(log, "Hit the end of input, producing error");

                    return GenericConsumption::err(ParseError::new(
                        ParseErrorKind::EndOfInput,
                        rule,
                        extracted,
                    ));
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
                // we 'continue' here, skipping the usual consumption check.
                tokens.update(remaining);

                continue;
            }

            // Ending the paragraph prematurely due to the element ending
            token if close_tokens.contains(&token) => {
                debug!(
                    log,
                    "Hit closing token, returning consumption success";
                    "token" => token,
                );

                return stack.into_consumption(tokens.slice());
            }

            // Ending the paragraph prematurely due to an error
            token if invalid_tokens.contains(&token) => {
                debug!(
                    log,
                    "Hit failure token, returning consumption failure";
                    "token" => token,
                );

                return GenericConsumption::err(ParseError::new(
                    ParseErrorKind::RuleFailed,
                    rule,
                    extracted,
                ));
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
                remaining,
                exceptions,
            } => {
                debug!(log, "Tokens successfully consumed to produce element");

                // Update remaining tokens
                //
                // The new value is a subslice of tokens,
                // equivalent to &tokens[offset..] but without
                // needing to assert bounds.
                tokens.update(remaining);

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
        }
    }

    stack.into_consumption(tokens.slice())
}
