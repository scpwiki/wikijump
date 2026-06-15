/*
 * parsing/collect/generic.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2026 Wikijump Team
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
/// Mutable parser reference:
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
/// If one of the failures is activated, then this `ParseErrorKind`
/// will be returned. If `None` is provided, then `ParseErrorKind::RuleFailed` is used.
/// * `error_kind`
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
///
/// The final token from the collection, one prior to the now-current token,
/// is returned.
pub fn collect<'p, 'r, 't, F>(
    parser: &'p mut Parser<'r, 't>,
    rule: Rule,
    close_conditions: &[ParseCondition],
    invalid_conditions: &[ParseCondition],
    error_kind: Option<ParseErrorKind>,
    mut process: F,
) -> ParseResult<'r, 't, &'r ExtractedToken<'t>>
where
    F: FnMut(&mut Parser<'r, 't>) -> ParseResult<'r, 't, ()>,
{
    debug!("Trying to collect tokens for rule {}", rule.name());

    let mut errors = Vec::new();
    let mut paragraph_safe = true;

    loop {
        // Check current token state to decide how to proceed.
        //
        // * End the collection, return elements
        // * Fail the collection, invalid token
        // * Continue the collection, consume to make a new element

        // See if the container has ended
        if parser.evaluate_any(close_conditions) {
            trace!(
                "Found ending condition, returning collected elements (token {})",
                parser.current().token.name(),
            );

            let last = parser.current();
            if parser.current().token != Token::InputEnd {
                parser.step()?;
            }

            return ok!(paragraph_safe; last, errors);
        }

        // See if the container should be aborted
        if parser.evaluate_any(invalid_conditions) {
            trace!(
                "Found invalid token, aborting container attempt (token {})",
                parser.current().token.name(),
            );

            return Err(parser.make_err(error_kind.unwrap_or(ParseErrorKind::RuleFailed)));
        }

        // See if we've hit the end
        if parser.current().token == Token::InputEnd {
            trace!("Found end of input, aborting");
            return Err(parser.make_err(ParseErrorKind::EndOfInput));
        }

        // Process token(s).
        let old_remaining = parser.remaining();
        process(parser)?.chain(&mut errors, &mut paragraph_safe);

        // If the pointer hasn't moved, we step one token.
        if parser.same_pointer(old_remaining) {
            parser.step()?;
        }
    }
}
