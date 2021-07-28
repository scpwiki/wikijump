/*
 * parsing/rule/impls/header.rs
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
use std::convert::TryInto;

pub const RULE_HEADER: Rule = Rule {
    name: "header",
    try_consume_fn,
};

fn try_consume_fn<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(log, "Trying to create header container");

    // Helper to ensure the current token is expected
    macro_rules! step {
        ($token:expr) => {{
            let current = parser.current();
            if current.token != $token {
                return Err(parser.make_warn(ParseWarningKind::RuleFailed));
            }

            parser.step()?;
            current
        }};
    }

    // Assert first tokens match rule
    check_step_multiple(parser, &[Token::LineBreak, Token::InputStart])?;

    // Get header depth
    let heading_level = step!(Token::Heading)
        .slice
        .len()
        .try_into()
        .expect("Received invalid heading length token slice");

    // Step over whitespace
    step!(Token::Whitespace);

    // Collect contents until newline
    collect_container(
        log,
        parser,
        RULE_HEADER,
        ContainerType::Header(heading_level),
        &[
            ParseCondition::current(Token::InputEnd),
            ParseCondition::current(Token::LineBreak),
            ParseCondition::current(Token::ParagraphBreak),
        ],
        &[],
        None,
    )
}
