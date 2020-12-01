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

use super::rule::{impls::RULE_FALLBACK, rules_for_token, Consumption};
use super::token::ExtractedToken;
use super::{ParseError, ParseErrorKind};
use crate::tree::Element;

/// Main function that consumes tokens to produce a single element, then returns.
pub fn consume<'t, 'r>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
) -> Consumption<'t, 'r> {
    let ExtractedToken { token, slice, span } = extracted;
    let log = &log.new(slog_o!(
        "token" => str!(token.name()),
        "slice" => str!(slice),
        "span-start" => span.start,
        "span-end" => span.end,
        "remaining-len" => remaining.len(),
    ));

    debug!(log, "Looking for valid rules");

    for rule in rules_for_token(extracted) {
        info!(log, "Trying rule consumption for tokens"; "rule" => rule);

        let consumption = rule.try_consume(log, extracted, remaining);
        if consumption.is_success() {
            debug!(log, "Rule matched, returning generated result"; "rule" => rule);

            return consumption;
        }

        // Discard invalid consumption
    }

    debug!(log, "All rules exhausted, using generic text fallback");

    let element = Element::Text(slice);
    let error = ParseError::new(ParseErrorKind::NoRulesMatch, RULE_FALLBACK, extracted);

    Consumption::warn(element, remaining, error)
}
