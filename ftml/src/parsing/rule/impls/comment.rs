/*
 * parsing/rule/impls/comment.rs
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

pub const RULE_COMMENT: Rule = Rule {
    name: "comment",
    position: LineRequirement::Any,
    try_consume_fn,
};

fn try_consume_fn<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(log, "Consuming tokens until end of comment");

    check_step(parser, Token::LeftComment)?;

    loop {
        let ExtractedToken {
            token,
            span: _span,
            slice: _slice,
        } = parser.current();

        debug!(
            log,
            "Received token inside comment";
            "token" => token,
            "slice" => _slice,
            "span" => SpanWrap::from(_span),
        );

        match token {
            // Hit the end of the comment, return
            Token::RightComment => {
                trace!(log, "Reached end of comment, returning");
                parser.step()?;
                return ok!(Elements::None);
            }

            // Hit the end of the input, abort
            Token::InputEnd => {
                trace!(log, "Reached end of input, aborting");

                return Err(parser.make_warn(ParseWarningKind::EndOfInput));
            }

            // Consume any other token
            _ => {
                trace!(log, "Token inside comment received. Discarding.");
                parser.step()?;
            }
        }
    }
}
