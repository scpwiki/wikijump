/*
 * parse/rule/collect/generic.rs
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

use super::prelude::*;
use std::fmt::Debug;

/// Generic function to parse upcoming tokens until conditions are met.
///
/// Each handled token can then processed in some manner, in accordance
/// to the passed closure.
///
/// The conditions for how to consume tokens are passed as arguments,
/// which are explained below.
///
/// Normal arguments (from `try_consume_fn`):
/// Obviously, the logger instance, and the current and upcoming tokens.
/// * `log`
/// * `extracted`
/// * `remaining`
/// * `full_text`
///
/// The rule we're parsing for:
/// * `rule`
///
/// The tokens we should end iteration on:
/// If one of these is the current token, we will return a consumption success.
/// * `close_tokens`
///
/// The tokens we should abort on:
/// If one of these is the current token, we will return a consumption failure.
/// * `invalid_tokens`
///
/// The token pairs we should abort on:
/// Each of these is a tuple in the form `(previous_token, current_token)`,
/// if they ever are found adjacent during parsing, we will return a consumption failure.
/// * `invalid_token_pairs`
///
/// This will proceed until a closing token is found, at which point the completed
/// list of items will be returned, or until an abort is found.
///
/// If the latter occurs, a `ParseError` is handed back and the parent will attempt the
/// next rule in the list, or the text fallback.
pub fn try_collect<'t, 'r, F, T>(
    log: &slog::Logger,
    (extracted, mut remaining, full_text): (
        &'r ExtractedToken<'t>,
        &'r [ExtractedToken<'t>],
        FullText<'t>,
    ),
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
        FullText<'t>,
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
        match process(log, new_extracted, new_remaining, full_text) {
            GenericConsumption::Success {
                item,
                remaining: new_remaining,
                error,
            } => {
                debug!(
                    log,
                    "Adding newly produced item from token consumption";
                    "item" => format!("{:?}", item),
                    "remaining-len" => new_remaining.len(),
                    "has-error" => error.is_some(),
                );

                let error2 = &error;

                // Append new item
                collected.push(item);

                // Update token pointer
                remaining = new_remaining;
            }

            GenericConsumption::Failure { error } => {
                debug!(
                    log,
                    "Failed to produce item from consumption, bubbling up error",
                );

                return GenericConsumption::err(error);
            }
        }
    }

    // If we've exhausted tokens but didn't find an ending token, we must abort.
    //
    // I don't think this will be terribly common, given that Token::InputEnd exists
    // and terminates all token lists, but this logic needs to be here anyways.
    GenericConsumption::err(ParseError::new(ParseErrorKind::EndOfInput, rule, extracted))
}
