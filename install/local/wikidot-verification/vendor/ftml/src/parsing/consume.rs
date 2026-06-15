/*
 * parsing/consume.rs
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

//! Module for look-ahead checking.
//!
//! This contains implementations of eager functions that try to interpret the
//! upcoming tokens as a particular object (e.g. seeing a `[[` and you see if it's a module).
//!
//! The parser is not disambiguous because any string of tokens can be interpreted
//! as raw text as a fallback, which is how Wikidot does it.

use super::Parser;
use super::prelude::*;
use super::rule::{get_rules_for_token, impls::RULE_FALLBACK};
use std::mem;

const STACK_RED_ZONE: usize = 128 * 1024;
const STACK_GROW_SIZE: usize = 2 * 1024 * 1024;

/// Main function that consumes tokens to produce a single element, then returns.
///
/// It will use the fallback if all rules, fail, so the only failure case is if
/// the end of the input is reached.
pub fn consume<'r, 't>(parser: &mut Parser<'r, 't>) -> ParseResult<'r, 't, Elements<'t>> {
    stacker::maybe_grow(STACK_RED_ZONE, STACK_GROW_SIZE, || consume_inner(parser))
}

fn consume_inner<'r, 't>(
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        "Running consume attempt (token {}, slice {:?})",
        parser.current().token.name(),
        parser.current().slice,
    );

    // Incrementing recursion depth
    // Will fail if we're too many layers in
    parser.depth_increment()?;

    trace!("Looking for valid rules");
    let mut all_errors = Vec::new();
    let current = parser.current();

    for &rule in get_rules_for_token(current) {
        trace!("Trying rule consumption for tokens (rule {})", rule.name());

        let old_remaining = parser.remaining();
        let footnote_count = parser.footnote_count();
        match rule.try_consume(parser) {
            Ok(output) => {
                debug!("Rule {} matched, returning generated result", rule.name());

                // If the pointer hasn't moved, we step one token.
                if parser.same_pointer(old_remaining) {
                    parser.step()?;
                }

                // Explicitly drop errors
                //
                // We're returning the successful consumption
                // so these are going to be dropped as a previously
                // unsuccessful attempts.
                mem::drop(all_errors);

                // Decrement recursion depth
                parser.depth_decrement();

                return Ok(output);
            }
            Err(error) => {
                warn!("Rule failed, returning error: '{}'", error.kind().name());
                // Rollback footnotes added during failed rule attempt
                parser.truncate_footnotes(footnote_count);
                all_errors.push(error);
            }
        }
    }

    warn!("All rules exhausted, using generic text fallback");
    let element = text!(current.slice);
    parser.step()?;

    // If we've hit the recursion limit, just bail
    if let Some(error) = all_errors.last()
        && error.kind() == ParseErrorKind::RecursionDepthExceeded
    {
        error!("Found recursion depth error, failing");
        return Err(error.clone());
    }

    // Add fallback error to errors list
    all_errors.push(ParseError::new(
        ParseErrorKind::NoRulesMatch,
        RULE_FALLBACK,
        current,
    ));

    // Decrement recursion depth
    parser.depth_decrement();

    ok!(element, all_errors)
}
