/*
 * parsing/collect/generic.rs
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
/// If one of the failures is activated, then this `ParseWarningKind`
/// will be returned. If `None` is provided, then `ParseWarningKind::RuleFailed` is used.
/// * `warn_kind`
///
/// The closure we should execute each time a token extraction is reached:
/// If the return value is `Err(_)` then collection is aborted and that warning
/// is bubbled up.
/// * `process`
///
/// This will proceed until a closing condition is found, an abort is found,
/// or the end of the input is reached.
///
/// It is up to the caller to save whatever result they need while running
/// in the closure.
///
/// The final token from the collection, one prior to the now-current token,
/// is returned.
pub fn collect<'p, 'r, 't, F>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
    _rule: Rule,
    close_conditions: &[ParseCondition],
    invalid_conditions: &[ParseCondition],
    warn_kind: Option<ParseWarningKind>,
    mut process: F,
) -> ParseResult<'r, 't, &'r ExtractedToken<'t>>
where
    F: FnMut(&Logger, &mut Parser<'r, 't>) -> ParseResult<'r, 't, ()>,
{
    // Log collect_until() call
    let log = {
        &log.new(slog_o!(
            "rule" => str!(_rule.name()),
            "token" => str!(parser.current().token.name()),
            "slice" => str!(parser.current().slice),
            "span" => SpanWrap::from(parser.current().span),
            "remaining-len" => parser.remaining().len(),
        ))
    };

    info!(log, "Trying to collect tokens for rule {:?}", _rule);

    let mut exceptions = Vec::new();
    let mut paragraph_safe = true;

    loop {
        // Check current token state to decide how to proceed.
        //
        // * End the collection, return elements
        // * Fail the collection, invalid token
        // * Continue the collection, consume to make a new element

        // See if the container has ended
        if parser.evaluate_any(close_conditions) {
            debug!(
                log,
                "Found ending condition, returning collected elements";
                "token" => parser.current().token,
            );

            let last = parser.current();
            if parser.current().token != Token::InputEnd {
                parser.step()?;
            }

            return ok!(paragraph_safe; last, exceptions);
        }

        // See if the container should be aborted
        if parser.evaluate_any(invalid_conditions) {
            debug!(
                log,
                "Found invalid token, aborting container attempt";
                "token" => parser.current().token,
            );

            return Err(
                parser.make_warn(warn_kind.unwrap_or(ParseWarningKind::RuleFailed))
            );
        }

        // See if we've hit the end
        if parser.current().token == Token::InputEnd {
            debug!(log, "Found end of input, aborting");

            return Err(parser.make_warn(ParseWarningKind::EndOfInput));
        }

        // Process token(s).
        let old_remaining = parser.remaining();
        process(log, parser)?.chain(&mut exceptions, &mut paragraph_safe);

        // If the pointer hasn't moved, we step one token.
        if parser.same_pointer(old_remaining) {
            parser.step()?;
        }
    }
}
