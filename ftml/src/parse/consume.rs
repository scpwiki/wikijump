/*
 * parse/consume.rs
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

//! Module for look-ahead checking.
//!
//! This contains implementations of eager functions that try to interpret the
//! upcoming tokens as a particular object (e.g. seeing a `[[` and you see if it's a module).
//!
//! The parser is not disambiguous because any string of tokens can be interpreted
//! as raw text as a fallback, which is how Wikidot does it.

use super::rule::{rules_for_token, RuleResult};
use super::stack::Stack;
use super::token::ExtractedToken;
use crate::tree::Element;

/// Main function that consumes tokens to produce a single element, then returns.
pub fn consume<'a>(
    log: &slog::Logger,
    _stack: &mut Stack,
    extract: &ExtractedToken<'a>,
    next: &[ExtractedToken<'a>],
) -> RuleResult<'a> {
    let ExtractedToken { token, slice, .. } = extract;
    let log = &log.new(slog_o!(
        "token" => str!(token.name()),
        "slice" => str!(slice),
        "next-len" => next.len(),
    ));

    debug!(log, "Looking for valid rules");

    for rule in rules_for_token(extract) {
        debug!(log, "Trying rule look ahead"; "rule" => rule);

        if let Some(result) = rule.try_consume(log, extract, next) {
            debug!(log, "Rule matched, returning generated result"; "rule" => rule);

            return result;
        }
    }

    debug!(log, "All rules exhausted, using generic text fallback");

    // Convert this token to text
    RuleResult {
        offset: 1,
        element: Element::Text(slice),
    }
}
