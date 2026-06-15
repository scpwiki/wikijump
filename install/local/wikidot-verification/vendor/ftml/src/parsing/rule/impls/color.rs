/*
 * parsing/rule/impls/color.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2026 Wikijump Team
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
use regex::Regex;
use std::borrow::Cow;
use std::sync::LazyLock;

static HEX_COLOR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([a-fA-F0-9]{3}|[a-fA-F0-9]{6})$").unwrap());

pub const RULE_COLOR: Rule = Rule {
    name: "color",
    position: LineRequirement::Any,
    try_consume_fn,
};

fn try_consume_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Trying to create color container");
    assert_step(parser, Token::Color)?;

    // The pattern for color is:
    // ## [color-style] | [text to be colored] ##

    // Gather the color name until the separator
    let color = collect_text(
        parser,
        RULE_COLOR,
        &[ParseCondition::current(Token::Pipe)],
        &[
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::current(Token::LineBreak),
        ],
        None,
    )?;

    trace!("Retrieved color descriptor, now building container ('{color}')");

    // Build color container
    let (elements, errors, paragraph_safe) = collect_consume(
        parser,
        RULE_COLOR,
        &[ParseCondition::current(Token::Color)],
        &[ParseCondition::current(Token::ParagraphBreak)],
        None,
    )?
    .into();

    // Return result
    let element = Element::Color {
        color: hexify_color(color),
        elements,
    };

    ok!(paragraph_safe; element, errors)
}

/// Prefix with `#`, if needed.
///
/// Normally we pass the color as-is, such as `blue` or `rgb(10, 12, 14)`,
/// but if a hex specification is passed, and it doesn't already begin with
/// `#`, then one should be prepended.
fn hexify_color(color: &str) -> Cow<'_, str> {
    if HEX_COLOR.is_match(color) {
        Cow::Owned(format!("#{color}"))
    } else {
        Cow::Borrowed(color)
    }
}
