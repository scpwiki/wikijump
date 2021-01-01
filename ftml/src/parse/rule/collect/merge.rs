/*
 * parse/rule/collect/merge.rs
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

/// Generic function to consume all tokens into a single string slice.
///
/// This is a subset of the functionality provided by `try_collect`,
/// as it specifically gathers all the extracted tokens into a string slice,
/// rather than considering them as special elements.
pub fn try_merge<'p, 'l, 'r, 't>(
    log: &'l slog::Logger,
    parser: &'p mut Parser<'l, 'r, 't>,
    rule: Rule,
    close_conditions: &[ParseCondition],
    invalid_conditions: &[ParseCondition],
) -> ParseResult<'r, 't, &'t str> {
    // Log try_merge() call
    info!(
        log,
        "Trying to consume tokens to merge into a single string",
    );

    // Iterate and collect the tokens to merge
    let result = try_collect(
        log,
        parser,
        rule,
        close_conditions,
        invalid_conditions,
        |log, parser| {
            trace!(log, "Ingesting token in string merge");

            ok!(parser.current(), parser.remaining())
        },
    )?;

    // Get the string slice associated with this list of tokens
    result.map_ok(|tokens| {
        match (tokens.first(), tokens.last()) {
            // Get first and last tokens for string slice
            (Some(first), Some(end)) => parser.full_text().slice(log, first, end),

            // Empty list of tokens, resultant slice must be empty
            (None, None) => "",

            // Impossible case, they should both be None if empty, Some(_) otherwise
            _ => unreachable!(),
        }
    })
}
