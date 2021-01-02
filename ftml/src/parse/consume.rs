/*
 * parse/consume.rs
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

//! Module for look-ahead checking.
//!
//! This contains implementations of eager functions that try to interpret the
//! upcoming tokens as a particular object (e.g. seeing a `[[` and you see if it's a module).
//!
//! The parser is not disambiguous because any string of tokens can be interpreted
//! as raw text as a fallback, which is how Wikidot does it.

use super::prelude::*;
use super::rule::{impls::RULE_FALLBACK, rules_for_token};
use super::Parser;
use crate::span_wrap::SpanWrap;
use std::mem;

/// Main function that consumes tokens to produce a single element, then returns.
///
/// It will use the fallback if all rules, fail, so the only failure case is if
/// the end of the input is reached.
pub fn consume<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Element<'t>> {
    let log = &log.new(slog_o!(
        "token" => parser.current().token,
        "slice" => str!(parser.current().slice),
        "span" => SpanWrap::from(&parser.current().span),
        "remaining-len" => parser.remaining().len(),
    ));

    debug!(log, "Looking for valid rules");

    let mut all_exceptions = Vec::new();
    let current = parser.current();

    for &rule in rules_for_token(current) {
        info!(log, "Trying rule consumption for tokens"; "rule" => rule);

        parser.set_rule(rule);

        let old_remaining = parser.remaining();
        match rule.try_consume(log, parser) {
            Ok(output) => {
                debug!(log, "Rule matched, returning generated result"; "rule" => rule);

                // If the pointer hasn't moved, we step one token.
                if parser.same_pointer(old_remaining) {
                    parser.step()?;
                }

                // Explicitly drop exceptions
                //
                // We're returning the successful consumption
                // so these are going to be dropped as a previously
                // unsuccessful attempts.
                mem::drop(all_exceptions);

                return Ok(output);
            }
            Err(error) => {
                all_exceptions.push(ParseException::Error(error));
            }
        }
    }

    debug!(log, "All rules exhausted, using generic text fallback");
    let element = text!(current.slice);
    parser.step()?;

    // We should only carry styles over from *successful* consumptions
    trace!(log, "Removing non-errors from exceptions list");
    all_exceptions.retain(|exception| matches!(exception, ParseException::Error(_)));

    trace!(log, "Adding fallback error to exceptions list");
    all_exceptions.push(ParseException::Error(ParseError::new(
        ParseErrorKind::NoRulesMatch,
        RULE_FALLBACK,
        current,
    )));

    ok!(element, all_exceptions)
}
