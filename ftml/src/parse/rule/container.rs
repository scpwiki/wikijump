/*
 * parse/rule/container.rs
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

//! Helper code to parse tokens out to generate recursive containers.

use crate::parse::consume::consume;
use crate::parse::error::{ParseError, ParseErrorKind};
use crate::parse::rule::{Consumption, ConsumptionResult, Rule};
use crate::parse::token::{ExtractedToken, Token};
use crate::tree::{Container, ContainerType, Element};

/// Generic function to parse upcoming tokens into a container.
///
/// The conditions for how to consume tokens are passed as arguments,
/// which are explained below.
///
/// Normal arguments (from `try_consume_fn`):
/// Obviously, the logger instance, and the current and upcoming tokens.
/// * `log`
/// * `extracted`
/// * `remaining`
///
/// The rule we're parsing for:
/// * `rule`
///
/// The kind of container we're building:
/// Must match the parse rule.
/// * `container_type`
///
/// The tokens we expect at the opening and ending of this container:
/// This will perform an assertion that the current token matches the opening type.
/// * `open_token`
/// * `close_token`
///
/// The tokens we should abort the container on:
/// If one of these is the current token, we will return a consumption failure.
/// * `invalid_tokens`
///
/// The token pairs we should abort the container on:
/// Each of these is a tuple in the form `(previous_token, current_token)`,
/// if they ever are found adjacent during parsing, we will return a consumption failure.
/// * `invalid_token_pairs`
///
/// This will proceed until the closing token is found, at which point the completed
/// container element will be returned, or until an abort is found.
/// If the latter occurs, a `ParseError` is handed back and the parent will attempt the
/// next rule in the list, or the text fallback.
pub fn try_container<'t, 'r>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    mut remaining: &'r [ExtractedToken<'t>],
    (rule, container_type): (Rule, ContainerType),
    (open_token, close_token): (Token, Token),
    invalid_tokens: &[Token],
    invalid_token_pairs: &[(Token, Token)],
) -> Consumption<'t, 'r> {
    // Log try_container() call
    let log = &log.new(slog_o!(
        "container-type" => str!(container_type.name()),
        "rule" => str!(rule.name()),
        "token" => str!(extracted.token.name()),
        "slice" => str!(extracted.slice),
        "span-start" => extracted.span.start,
        "span-end" => extracted.span.end,
        "remaining-len" => remaining.len(),
        "open-token" => open_token,
        "close-token" => close_token,
        "invalid-tokens-len" => invalid_tokens.len(),
        "invalid-token-pairs-len" => invalid_token_pairs.len(),
    ));

    info!(
        log,
        "Trying to consume tokens to produce container for {:?}", rule,
    );

    // Ensure that we're on the right opening token
    assert_eq!(
        extracted.token, open_token,
        "Current token does not match opener",
    );

    // Begin building up the child elements
    let mut elements = Vec::new();
    let mut prev_token = extracted.token;

    while let Some((new_extracted, new_remaining)) = remaining.split_first() {
        let current_token = new_extracted.token;

        // Check previous and current tokens
        let pair = &(prev_token, current_token);
        if invalid_token_pairs.contains(&pair) {
            debug!(
                log,
                "Found invalid (previous, current) token combination, failing rule";
                "prev-token" => prev_token,
                "current-token" => current_token,
            );

            return Consumption::err(ParseError::new(
                ParseErrorKind::RuleFailed,
                rule,
                new_extracted,
            ));
        }

        // Update the state variables
        //
        // * "remaining" is updated in case we return
        // * "prev_token" is updated as it's only checked above
        remaining = new_remaining;
        prev_token = current_token;

        // "prev_token" should *not* be used underneath here.
        // To enforce this, we shadow the variable name:
        #[allow(unused_variables)]
        let prev_token = ();

        // Check current token to decide how to proceed.
        //
        // * End the container, return elements
        // * Fail the container, invalid token
        // * Continue the container, consume to make a new element

        // See if the container has ended
        if current_token == close_token {
            debug!(
                log,
                "Found ending token, returning collected elements";
                "elements-len" => elements.len(),
            );

            let container = Container::new(container_type, elements);
            let element = Element::Container(container);

            return Consumption::ok(element, remaining);
        }

        // See if the container should be aborted
        if invalid_tokens.contains(&current_token) {
            debug!(
                log,
                "Found invalid token, aborting container attempt";
                "token" => current_token,
                "elements-len" => elements.len(),
            );

            return Consumption::err(ParseError::new(
                ParseErrorKind::RuleFailed,
                rule,
                new_extracted,
            ));
        }

        // Consume tokens to produce a new element
        let consumption = consume(log, new_extracted, new_remaining);
        match consumption.result {
            ConsumptionResult::Success {
                element,
                remaining: new_remaining,
            } => {
                debug!(
                    log,
                    "Adding newly produced element from token consumption";
                    "element" => element.name(),
                    "remaining-len" => new_remaining.len(),
                );

                // Add new element to container
                elements.push(element);

                // Update token pointer
                remaining = new_remaining;
            }

            ConsumptionResult::Failure => {
                debug!(
                    log,
                    "Failed to produce token from consumption, bubbling up error",
                );

                return consumption;
            }
        }
    }

    // If we've exhausted tokens but didn't find an ending token, we must abort.
    //
    // I don't think this will be terribly common, given that Token::InputEnd exists
    // and terminates all token lists, but this logic needs to be here anyways.
    Consumption::err(ParseError::new(ParseErrorKind::RuleFailed, rule, extracted))
}
