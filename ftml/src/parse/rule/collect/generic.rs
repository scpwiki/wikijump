/*
 * parse/rule/collect/generic.rs
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
pub fn try_collect<'p, 'l, 'r, 't, F, T>(
    parser: &'p mut Parser<'l, 'r, 't>,
    rule: Rule,
    close_conditions: &[ParseCondition],
    invalid_conditions: &[ParseCondition],
    mut process: F,
) -> ParseResult<'r, 't, Vec<T>>
where
    F: FnMut(&slog::Logger, &'p mut Parser<'l, 'r, 't>) -> ParseResult<'r, 't, T>,
    T: Debug,
{
    /// Tokens that are always considered invalid, and will fail the rule.
    ///
    /// This behaves as if all of these tokens have associated
    /// `ParseCondition::CurrentToken` rules in `invalid_conditions`.
    const ALWAYS_INVALID: &[Token] = &[Token::InputEnd];

    // Log collect_until() call
    let log = &parser.log().new(slog_o!(
        "rule" => str!(rule.name()),
        "token" => str!(parser.current().token.name()),
        "slice" => str!(parser.current().slice),
        "span-start" => parser.current().span.start,
        "span-end" => parser.current().span.end,
        "remaining-len" => parser.remaining().len(),
        "close-conditions" => format!("{:?}", close_conditions),
        "invalid-conditions" => format!("{:?}", invalid_conditions),
    ));

    info!(log, "Trying to collect tokens for rule {:?}", rule);

    let mut collected = Vec::new();
    let mut all_exc = Vec::new();

    loop {
        // Check current token state to decide how to proceed.
        //
        // * End the container, return elements
        // * Fail the container, invalid token
        // * Continue the container, consume to make a new element

        // See if the container has ended
        if parser.evaluate_any(close_conditions) {
            debug!(
                log,
                "Found ending condition, returning collected elements";
                "token" => parser.current().token,
                "collected" => format!("{:?}", collected),
            );

            return ok!(collected, parser.remaining(), all_exc);
        }

        // See if the container should be aborted
        if parser.evaluate_any(invalid_conditions)
            || ALWAYS_INVALID.contains(&parser.current().token)
        {
            debug!(
                log,
                "Found invalid token, aborting container attempt";
                "token" => parser.current().token,
                "collected" => format!("{:?}", collected),
            );

            return Err(parser.make_error(ParseErrorKind::RuleFailed));
        }

        // Process token(s).
        let (item, _, mut exceptions) = process(log, parser)?.into();

        debug!(
            log,
            "Adding newly produced item from token consumption";
            "item" => format!("{:?}", item),
            "remaining-len" => parser.remaining().len(),
        );

        // Append new item
        collected.push(item);

        // Append new exceptions
        all_exc.append(&mut exceptions);
    }
}
