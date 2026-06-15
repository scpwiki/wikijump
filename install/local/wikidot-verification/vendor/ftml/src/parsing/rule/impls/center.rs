/*
 * parsing/rule/impls/center.rs
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
use crate::tree::Alignment;

pub const RULE_CENTER: Rule = Rule {
    name: "center",
    position: LineRequirement::StartOfLine,
    try_consume_fn,
};

fn try_consume_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Trying to create centered container");

    // Check that the rule has "= "
    macro_rules! next {
        ($token:expr) => {{
            let token = parser.current().token;
            if token != $token {
                return Err(parser.make_err(ParseErrorKind::RuleFailed));
            }

            parser.step()?;
        }};
    }

    next!(Token::Equals);
    next!(Token::Whitespace);

    // Collect contents
    collect_container(
        parser,
        RULE_CENTER,
        ContainerType::Align(Alignment::Center),
        &[
            ParseCondition::current(Token::LineBreak),
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::current(Token::InputEnd),
        ],
        &[],
        None,
    )
}
