/*
 * parsing/paragraph/mod.rs
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

mod stack;

pub use self::stack::ParagraphStack;

use super::consume::consume;
use super::parser::Parser;
use super::prelude::*;
use super::rule::Rule;
use super::token::Token;
use crate::log::prelude::*;
use crate::tree::Elements;
use std::convert::TryInto;

/// Wrapper type to satisfy the issue with generic closure types.
///
/// Because `None` does not specify the type for `F`, we need to
/// tell the compiler it has a concrete type.
///
/// But since it's just `None`, it's not actually pointing to a function,
/// it's just clarifying what the `_` in `Option<_>` is.
pub const NO_CLOSE_CONDITION: Option<CloseConditionFn> = None;

type CloseConditionFn = fn(&mut Parser) -> Result<bool, ParseWarning>;

/// Function to iterate over tokens to produce elements in paragraphs.
///
/// Originally in `parse()`, but was moved out to allow paragraph
/// extraction deeper in code, such as in the `try_paragraph`
/// collection helper.
///
/// This does not necessarily produce a paragraph container.
/// It may produce multiple or none. Instead the logic iterates
/// and produces paragraphs or child elements as needed.
pub fn gather_paragraphs<'r, 't, F>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    rule: Rule,
    mut close_condition_fn: Option<F>,
) -> ParseResult<'r, 't, Vec<Element<'t>>>
where
    'r: 't,
    F: FnMut(&mut Parser<'r, 't>) -> Result<bool, ParseWarning>,
{
    info!(log, "Gathering paragraphs until ending");

    // Update parser rule
    parser.set_rule(rule);

    // Create paragraph stack
    let mut stack = ParagraphStack::new(log);

    loop {
        let (partials, mut exceptions, paragraph_safe) = match parser.current().token {
            Token::InputEnd => {
                if close_condition_fn.is_some() {
                    // There was a close condition, but it was not satisfied
                    // before the end of input.
                    //
                    // Pass a warning up the chain

                    warn!(log, "Hit the end of input, producing warning");

                    return Err(parser.make_warn(ParseWarningKind::EndOfInput));
                } else {
                    // Avoid an unnecessary Element::Null and just exit
                    // If there's no close condition, then this is not a warning

                    warn!(log, "Hit the end of input, terminating token iteration");

                    break;
                }
            }

            // If we've hit a paragraph break, then finish the current paragraph
            Token::ParagraphBreak => {
                info!(
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

            // Determine if we're ending the paragraph here,
            // or continuing with another element
            _ => {
                if let Some(ref mut close_condition_fn) = close_condition_fn {
                    if close_condition_fn(parser).unwrap_or(false) {
                        info!(
                            log,
                            "Hit closing condition for paragraphs, terminating token iteration",
                        );

                        break;
                    }
                }

                // Otherwise, produce consumption from this token pointer
                debug!(log, "Trying to consume tokens to produce element");
                consume(log, parser)
            }
        }?
        .into();

        debug!(log, "Tokens consumed to produce element");

        // Extract elements from partials
        let elements = try_from_partials!(parser, partials);

        // Add new elements to the list
        push_elements(&mut stack, elements, paragraph_safe);

        // Process exceptions
        stack.push_exceptions(&mut exceptions);
    }

    stack.into_result()
}

fn push_elements<'t>(
    stack: &mut ParagraphStack<'t>,
    elements: Elements<'t>,
    paragraph_safe: bool,
) {
    stack.reserve_elements(elements.len());

    for element in elements {
        // Don't add a line break if the paragraph is otherwise empty
        if stack.current_empty() && element == Element::LineBreak {
            continue;
        }

        stack.push_element(element, paragraph_safe);
    }
}
