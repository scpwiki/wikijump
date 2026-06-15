/*
 * parsing/rule/impls/clear_float.rs
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
use crate::tree::ClearFloat;

pub const RULE_CLEAR_FLOAT: Rule = Rule {
    name: "clear-float",
    position: LineRequirement::StartOfLine,
    try_consume_fn,
};

fn try_consume_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    let current = parser.current();
    debug!("Consuming token to create a clear float");

    let clear_float = match current.token {
        Token::ClearFloatBoth => ClearFloat::Both,
        Token::ClearFloatLeft => ClearFloat::Left,
        Token::ClearFloatRight => ClearFloat::Right,
        _ => return Err(parser.make_err(ParseErrorKind::RuleFailed)),
    };

    // Optionally consume newline after
    parser.get_optional_line_break()?;

    ok!(Element::ClearFloat(clear_float))
}
