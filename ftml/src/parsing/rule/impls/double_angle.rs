/*
 * parsing/rule/impls/double_angle.rs
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

pub const RULE_DOUBLE_ANGLE: Rule = Rule {
    name: "double-angle",
    try_consume_fn,
};

fn try_consume_fn<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    let token = parser.current().token;

    debug!(
        log,
        "Consuming token to create a left/right double angle quote";
        "token" => token,
    );

    match token {
        // « - LEFT-POINTING DOUBLE ANGLE QUOTATION MARK
        Token::LeftDoubleAngle => ok!(text!("\u{0ab}")),

        // » - RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK
        Token::RightDoubleAngle => ok!(text!("\u{0bb}")),

        // Invalid token for this rule
        _ => unreachable!(),
    }
}
