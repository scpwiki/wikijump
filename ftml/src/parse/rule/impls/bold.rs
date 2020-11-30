/*
 * parse/rule/impls/bold.rs
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

// TODO once this is implemented, genericize into a helper function

use super::prelude::*;

pub const RULE_BOLD: Rule = Rule {
    name: "bold",
    try_consume_fn,
};

fn try_consume_fn<'t, 'r>(
    log: &slog::Logger,
    extract: &'r ExtractedToken<'t>,
    mut remaining: &'r [ExtractedToken<'t>],
) -> Consumption<'t, 'r> {
    debug_assert_eq!(extract.token, Token::Bold, "Current token is not bold");

    debug!(log, "Trying to consume tokens until we find ending bold");

    let mut elements = Vec::new();
    while let Some((new_extract, new_remaining)) = remaining.split_first() {
        // Update token slice
        remaining = new_remaining;

        // Check token for how to proceed
        match new_extract.token {
            // End of bold, send result
            Token::Bold => {
                trace!(log, "Found ending bold, returning collected elements"; "elements-len" => elements.len());

                let container = Container::new(ContainerType::Bold, elements);
                let element = Element::Container(container);

                return Consumption::ok(element, remaining);
            }

            // Cases where we should abort
            Token::LineBreak | Token::InputEnd => {
                trace!(log, "Found invalid token, failing rule");

                return Consumption::err(ParseError::new(
                    ParseErrorKind::RuleFailed,
                    RULE_BOLD,
                    new_extract,
                ));
            }

            // Other, attempt to consume separately
            _ => {
                let consumption = consume(log, new_extract, new_remaining);
                match consumption.result {
                    ConsumptionResult::Success {
                        element,
                        remaining: new_remaining,
                    } => {
                        trace!(log, "Adding new element from token consumption attempt");
                        elements.push(element);
                        remaining = new_remaining;
                    }
                    ConsumptionResult::Failure => {
                        trace!(log, "Bubbling up error from token consumption attempt");
                        return consumption;
                    }
                }
            }
        }
    }

    Consumption::err(ParseError::new(
        ParseErrorKind::RuleFailed,
        RULE_BOLD,
        extract,
    ))
}
