/*
 * parsing/rule/impls/color.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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
    position: LineRequirement::Any,
    try_consume_fn,
};

fn try_consume_fn<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(log, "Trying to create color container");

    check_step(parser, Token::Color)?;

    // The pattern for color is:
    // ## [color-style] | [text to be colored] ##

    // Gather the color name until the separator
    let color = collect_text(
        log,
        parser,
        RULE_COLOR,
        &[ParseCondition::current(Token::Pipe)],
        &[
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::current(Token::LineBreak),
        ],
        None,
    )?;

    debug!(
        log,
        "Retrieved color descriptor, now building container";
        "color" => color,
    );

    // Build color container
    let (elements, exceptions, paragraph_safe) = collect_consume(
        log,
        parser,
        RULE_COLOR,
        &[ParseCondition::current(Token::Color)],
        &[ParseCondition::current(Token::ParagraphBreak)],
        None,
    )?
    .into();

    // Return result
    let element = Element::Color {
        color: cow!(color),
        elements,
    };

    ok!(paragraph_safe; element, exceptions)
}
