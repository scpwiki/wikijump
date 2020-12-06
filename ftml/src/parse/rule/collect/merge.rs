/*
 * parse/rule/collect/merge.rs
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

/// Generic function to consume all tokens into a single string slice.
///
/// This is a subset of the functionality provided by `try_collect`,
/// as it specifically gathers all the extracted tokens into a string slice,
/// rather than considering them as special elements.
pub fn try_merge<'t, 'r>(
    log: &slog::Logger,
    (extracted, remaining, full_text): (
        &'r ExtractedToken<'t>,
        &'r [ExtractedToken<'t>],
        FullText<'t>,
    ),
    rule: Rule,
    close_tokens: &[Token],
    invalid_tokens: &[Token],
    invalid_token_pairs: &[(Token, Token)],
) -> GenericConsumption<'t, 'r, &'t str> {
    // Log try_merge() call
    info!(log, "Trying to consume tokens to merge into a single string");

    let tokens = try_collect(
        log,
        (extracted, remaining, full_text),
        rule,
        close_tokens,
        invalid_tokens,
        invalid_token_pairs,
        |log, extracted, remaining, _full_text| {
            trace!(log, "Ingesting token in string merge");

            GenericConsumption::ok(extracted, remaining)
        },
    );

    println!("tokens {:#?}", tokens);

    todo!()
}
