/*
 * parse/rule/collect.rs
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

use crate::parse::rule::{Consumption, ConsumptionResult, Rule};
use crate::parse::token::{ExtractedToken, Token};

/// Generic function to parse through tokens until conditions are met.
///
/// This is even more generic than `try_container`, as it doesn't produce
/// a specific sub-element when done. It's more designed to remove the boilerplate
/// of extracted token iteration by providing common notions and abilities.
pub fn collect_until<'t, 'r>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    mut remaining: &'r [ExtractedToken<'t>],
    rule: Rule,
    close_tokens: &[Token],
    invalid_tokens: &[Token],
    invalid_token_pairs: &[(Token, Token)],
) -> Consumption<'t, 'r> {
    // Log collect_until() call
    let log = &log.new(slog_o!(
        "rule" => str!(rule.name()),
        "token" => str!(extracted.token.name()),
        "slice" => str!(extracted.slice),
        "span-start" => extracted.span.start,
        "span-end" => extracted.span.end,
        "remaining-len" => remaining.len(),
        "close-tokens" => format!("{:?}", close_tokens),
        "invalid-tokens-len" => format!("{:?}", invalid_tokens),
        "invalid-token-pairs-len" => format!("{:?}", invalid_token_pairs),
    ));

    info!(log, "Trying to collect tokens for rule {:?}", rule);

    todo!()
}
