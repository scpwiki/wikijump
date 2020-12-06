/*
 * parse/rule/impls/color.rs
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

pub const RULE_COLOR: Rule = Rule {
    name: "color",
    try_consume_fn,
};

fn try_consume_fn<'t, 'r>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> Consumption<'t, 'r> {
    debug!(log, "Trying to create color container");

    assert_eq!(
        extracted.token,
        Token::Color,
        "Current token isn't color marker",
    );

    // The pattern for color is:
    // ## [color-style] | [text to be colored] ##

    // Gather the color name until the separator
    let consumption = try_merge(
        log,
        (extracted, remaining, full_text),
        RULE_COLOR,
        &[Token::Pipe],
        &[Token::ParagraphBreak, Token::LineBreak, Token::InputEnd],
        &[],
    );

    // Return if failure
    let (color, remaining, errors) = match consumption {
        GenericConsumption::Failure { error } => return GenericConsumption::err(error),
        GenericConsumption::Success {
            item,
            remaining,
            errors,
        } => (item, remaining, errors),
    };

    debug!(
        log,
        "Retrieved color descriptor, now building container";
        "color" => color,
    );

    // Build color container
    try_container(
        log,
        (extracted, remaining, full_text),
        (RULE_COLOR, ContainerType::Color(color)),
        (Token::Pipe, Token::Color),
        &[Token::ParagraphBreak, Token::InputEnd],
        &[],
    )
}
