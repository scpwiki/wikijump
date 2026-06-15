/*
 * parsing/rule/impls/double_angle.rs
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

pub const RULE_DOUBLE_ANGLE: Rule = Rule {
    name: "double-angle",
    position: LineRequirement::Any,
    try_consume_fn,
};

fn try_consume_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    let current = parser.current();
    debug!(
        "Consuming token '{}', to create a left/right double angle quote",
        current.token.name(),
    );

    match current.token {
        // « - LEFT-POINTING DOUBLE ANGLE QUOTATION MARK
        Token::LeftDoubleAngle => ok!(text!("\u{0ab}")),

        // » - RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK
        Token::Quote if current.slice == ">>" => ok!(text!("\u{0bb}")),

        // Some other series of ">"s in a line
        Token::Quote => Err(parser.make_err(ParseErrorKind::RuleFailed)),

        // Invalid token for this rule
        _ => unreachable!(),
    }
}
