/*
 * parse/rule/impls/color.rs
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

pub const RULE_COLOR: Rule = Rule {
    name: "color",
    try_consume_fn,
};

fn try_consume_fn<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> ParseResult<'r, 't, Element<'t>> {
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
        &[Token::ParagraphBreak, Token::LineBreak],
        &[],
    );

    // Return if failure, and get last token for try_container()
    let (color, extracted, remaining, mut all_exceptions) =
        try_consume_last!(remaining, consumption);

    debug!(
        log,
        "Retrieved color descriptor, now building container";
        "color" => color,
    );

    // Build color container
    let result = try_collect(
        log,
        (extracted, remaining, full_text),
        RULE_COLOR,
        &[Token::Color],
        &[Token::ParagraphBreak],
        &[],
        consume,
    );

    // Append errors, or return if failure
    let (elements, remaining, mut exceptions) = result?.into();

    // Add on new errors
    all_exceptions.append(&mut exceptions);

    // Return result
    let element = Element::Color {
        color: cow!(color),
        elements,
    };

    ok!(element, remaining, all_exceptions)
}
