/*
 * parse/rule/collect/container.rs
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

//! Helper code to parse tokens out to generate recursive containers.

use super::prelude::*;
use crate::text::FullText;
use crate::tree::{Container, ContainerType, Element};

/// Generic function to consume tokens into a container.
///
/// This is a subset of the functionality provided by `try_collect`,
/// as it builds `Container`s specifically rather, but still permits
/// passing arguments to specify behavior and reduce boilerplate.
///
/// The arguments which differ from `collect_until` are listed:
/// See that function for full documentation, as the call here
/// mostly wraps it.
///
/// The kind of container we're building:
/// Must match the parse rule.
/// * `container_type`
///
/// The tokens we expect at the opening and ending of this container:
/// This will perform an assertion that the current token matches the opening type.
/// * `open_token`
/// * `close_token`
pub fn try_container<'t, 'r>(
    log: &slog::Logger,
    (extracted, remaining, full_text): (
        &'r ExtractedToken<'t>,
        &'r [ExtractedToken<'t>],
        FullText<'t>,
    ),
    (rule, container_type): (Rule, ContainerType<'t>),
    (open_token, close_token): (Token, Token),
    invalid_tokens: &[Token],
    invalid_token_pairs: &[(Token, Token)],
) -> Consumption<'t, 'r> {
    // Log try_container() call
    let log = &log.new(slog_o!(
        "container-type" => str!(container_type.name()),
        "open-token" => open_token,
    ));

    info!(
        log,
        "Trying to consume tokens to produce container for {:?}", rule,
    );

    // Ensure that we're on the right opening token
    assert_eq!(
        extracted.token, open_token,
        "Current token does not match opener",
    );

    // Actually iterate and collect
    let consumption = try_collect(
        log,
        (extracted, remaining, full_text),
        rule,
        &[close_token],
        invalid_tokens,
        invalid_token_pairs,
        consume,
    );

    // Package into a container
    consumption.map(|elements| Element::Container(Container::new(container_type, elements)))
}
