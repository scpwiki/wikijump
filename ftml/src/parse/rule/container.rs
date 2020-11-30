/*
 * parse/rule/container.rs
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

/// Helper code to parse tokens out to generate recursive containers.

use crate::parse::consume::consume;
use crate::parse::error::{ParseError, ParseErrorKind};
use crate::parse::rule::{Consumption, ConsumptionResult, Rule, TryConsumeFn};
use crate::parse::token::{ExtractedToken, Token};
use crate::tree::{Container, ContainerType, Element};

/// Generic function to parse upcoming tokens into a container.
///
/// The conditions for how to consume tokens are passed as arguments,
/// which are explained below.
///
/// Normal arguments (from `try_consume_fn`):
/// Obviously, the logger instance, and the current and upcoming tokens.
/// * `log`
/// * `extract`
/// * `remaining`
///
/// The rule we're parsing for:
/// * `rule`
///
/// The kind of container we're building:
/// Must match the parse rule.
/// * `container_type`
///
/// The tokens we expect at the opening and ending of this container:
/// This will perform an assertion that the current token matches the opening type.
/// * `open_token`
/// * `close_token`
///
/// The tokens we should abort the container on:
/// If one of these is the current token, we will return a consumption failure.
/// * `invalid_tokens`
///
/// The token pairs we should abort the container on:
/// Each of these is a tuple in the form `(previous_token, current_token)`,
/// if they ever are found adjacent during parsing, we will return a consumption failure.
/// * `invalid_token_pairs`
///
/// This will proceed until the closing token is found, at which point the completed
/// container element will be returned, or until an abort is found.
/// If the latter occurs, a `ParseError` is handed back and the parent will attempt the
/// next rule in the list, or the text fallback.
pub fn try_container<'t, 'r>(
    log: &slog::Logger,
    extract: &'r ExtractedToken<'t>,
    mut remaining: &'r [ExtractedToken<'t>],
    rule: Rule,
    container_type: ContainerType,
    open_token: Token,
    close_token: Token,
    invalid_tokens: &[Token],
    invalid_token_pairs: &[(Token, Token)],
) -> Consumption<'t, 'r> {
    // Log try_container() call
    let log = &log.new(slog_o!(
        "container-type" => str!(container_type.name()),
        "rule" => str!(rule.name()),
        "token" => str!(extract.token.name()),
        "slice" => str!(extract.slice),
        "span-start" => extract.span.start,
        "span-end" => extract.span.end,
        "remaining-len" => remaining.len(),
        "open-token" => open_token,
        "close-token" => close_token,
        "invalid-tokens-len" => invalid_tokens.len(),
        "invalid-token-pairs-len" => invalid_token_pairs.len(),
    ));

    info!(log, "Trying to consume tokens to produce container for {:?}", rule);

    // Ensure that we're on the right opening token
    assert_eq!(
        extract.token, open_token,
        "Current token does not match opener",
    );

    todo!()
}
