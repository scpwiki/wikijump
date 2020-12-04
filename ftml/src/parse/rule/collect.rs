/*
 * parse/rule/collect.rs
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

use crate::parse::error::{ParseError, ParseErrorKind};
use crate::parse::rule::{GenericConsumption, GenericConsumptionResult, Rule};
use crate::parse::token::{ExtractedToken, Token};
use std::fmt::Debug;

/// Generic function to parse through tokens until conditions are met.
///
/// This is even more generic than `try_container`, as it doesn't produce
/// a specific sub-element when done. It's more designed to remove the boilerplate
/// of extracted token iteration by providing common notions and abilities.
pub fn collect_until<'t, 'r, F, T>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    mut remaining: &'r [ExtractedToken<'t>],
    rule: Rule,
    close_tokens: &[Token],
    invalid_tokens: &[Token],
    invalid_token_pairs: &[(Token, Token)],
    mut process: F,
) -> GenericConsumption<'r, 't, Vec<T>>
where
    F: FnMut(
        &slog::Logger,
        &'r ExtractedToken<'t>,
        &'r [ExtractedToken<'t>],
    ) -> GenericConsumption<'r, 't, T>,
    T: Debug,
{
    // Log collect_until() call
    let log = &log.new(slog_o!(
        "rule" => str!(rule.name()),
        "token" => str!(extracted.token.name()),
        "slice" => str!(extracted.slice),
        "span-start" => extracted.span.start,
        "span-end" => extracted.span.end,
        "remaining-len" => remaining.len(),
        "close-tokens" => format!("{:?}", close_tokens),
        "invalid-tokens-len" => format!("{:?}", invalid_tokens),
        "invalid-token-pairs-len" => format!("{:?}", invalid_token_pairs),
    ));

    info!(log, "Trying to collect tokens for rule {:?}", rule);

    let mut collected = Vec::new();
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

            return GenericConsumption::err(ParseError::new(
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
        if close_tokens.contains(&current_token) {
            debug!(
                log,
                "Found ending token, returning collected elements";
                "token" => current_token,
                "collected" => format!("{:?}", collected),
            );

            return GenericConsumption::ok(collected, remaining);
        }

        // See if the container should be aborted
        if invalid_tokens.contains(&current_token) {
            debug!(
                log,
                "Found invalid token, aborting container attempt";
                "token" => current_token,
                "collected" => format!("{:?}", collected),
            );

            return GenericConsumption::err(ParseError::new(
                ParseErrorKind::RuleFailed,
                rule,
                new_extracted,
            ));
        }

        // Process token(s).
        let consumption = process(log, new_extracted, new_remaining);
        match consumption.result {
            GenericConsumptionResult::Success {
                element,
                remaining: new_remaining,
            } => {
                debug!(
                    log,
                    "Adding newly produced item from token consumption";
                    "item" => format!("{:?}", element),
                    "remaining-len" => new_remaining.len(),
                );

                // Append new item
                collected.push(element);

                // Update token pointer
                remaining = new_remaining;
            }

            GenericConsumptionResult::Failure => {
                debug!(
                    log,
                    "Failed to produce item from consumption, bubbling up error",
                );

                return GenericConsumption::err(
                    consumption
                        .error
                        .expect("Token consumption attemption did not produce an error"),
                );
            }
        }
    }

    // If we've exhausted tokens but didn't find an ending token, we must abort.
    //
    // I don't think this will be terribly common, given that Token::InputEnd exists
    // and terminates all token lists, but this logic needs to be here anyways.
    GenericConsumption::err(ParseError::new(ParseErrorKind::EndOfInput, rule, extracted))
}
