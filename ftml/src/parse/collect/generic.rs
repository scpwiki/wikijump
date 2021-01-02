/*
 * parse/collect/generic.rs
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

/// Generic function to parse upcoming tokens until conditions are met.
///
/// Each handled token can then processed in some manner, in accordance
/// to the passed closure.
///
/// The conditions for how to consume tokens are passed as arguments,
/// which are explained below.
///
/// Logger instance and mutable parser reference:
/// * `log`
/// * `parser`
///
/// The rule we're parsing for:
/// * `rule`
///
/// The conditions we should end iteration on:
/// If one of these is true, we will return success.
/// * `close_conditions`
///
/// The conditions we should abort on:
/// If one of these is true, we will return failure.
/// * `invalid_conditions`
///
/// The closure we should execute each time a token extraction is reached:
/// If the return value is `Err(_)` then collection is aborted and that error
/// is bubbled up.
/// * `process`
///
/// This will proceed until a closing condition is found, an abort is found,
/// or the end of the input is reached.
///
/// It is up to the caller to save whatever result they need while running
/// in the closure.
pub fn collect<'p, 'r, 't, F>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
    rule: Rule,
    close_conditions: &[ParseCondition],
    invalid_conditions: &[ParseCondition],
    mut process: F,
) -> ParseResult<'r, 't, ()>
where
    F: FnMut(&slog::Logger, &mut Parser<'r, 't>) -> ParseResult<'r, 't, ()>,
{
    /// Tokens that are always considered invalid, and will fail the rule.
    ///
    /// This behaves as if all of these tokens have associated
    /// `ParseCondition::CurrentToken` rules in `invalid_conditions`.
    const ALWAYS_INVALID: &[Token] = &[Token::InputEnd];

    // Log collect_until() call
    let log = &log.new(slog_o!(
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

    let mut exceptions = Vec::new();
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
            );

            parser.step()?;
            return ok!((), exceptions);
        }

        // See if the container should be aborted
        if parser.evaluate_any(invalid_conditions)
            || ALWAYS_INVALID.contains(&parser.current().token)
        {
            debug!(
                log,
                "Found invalid token, aborting container attempt";
                "token" => parser.current().token,
            );

            return Err(parser.make_error(ParseErrorKind::RuleFailed));
        }

        // Process token(s).
        let old_remaining = parser.remaining();
        process(log, parser)?.chain(&mut exceptions);

        // If the pointer hasn't moved, we step one token.
        if parser.same_pointer(old_remaining) {
            parser.step()?;
        }
    }
}
