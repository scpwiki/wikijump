/*
 * parsing/rule/impls/underscore_line_break.rs
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

pub const RULE_UNDERSCORE_LINE_BREAK: Rule = Rule {
    name: "underscore-line-break",
    position: LineRequirement::Any,
    try_consume_fn,
};

fn try_consume_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Trying to parse underscore line break");

    // These can start in two ways:
    // Either a space, or start of line.
    //
    // Put in a regex-like syntax, we want:
    // " "? ~ "_" ~ "\n"
    //
    // So if it's not newline-based, we step to get onto the underscore.
    match parser.next_two_tokens() {
        (Token::Whitespace, Some(Token::Underscore)) => {
            parser.step()?;
        }
        (Token::Underscore, Some(_)) if parser.start_of_line() => (),
        _ => return Err(parser.make_err(ParseErrorKind::RuleFailed)),
    }

    // Now the current token should be underscore, then newline.
    if !matches!(
        parser.next_two_tokens(),
        (
            Token::Underscore,
            Some(Token::LineBreak | Token::ParagraphBreak),
        ),
    ) {
        return Err(parser.make_err(ParseErrorKind::RuleFailed));
    }

    // Since we know where we are, we can step over them, then be done.
    parser.step_n(2)?;

    ok!(Element::LineBreak)
}
