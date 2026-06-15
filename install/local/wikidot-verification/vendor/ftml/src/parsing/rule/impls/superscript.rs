/*
 * parsing/rule/impls/superscript.rs
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

pub const RULE_SUPERSCRIPT: Rule = Rule {
    name: "superscript",
    position: LineRequirement::Any,
    try_consume_fn,
};

fn try_consume_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Trying to create superscript container");
    assert_step(parser, Token::Superscript)?;
    collect_container(
        parser,
        RULE_SUPERSCRIPT,
        ContainerType::Superscript,
        &[ParseCondition::current(Token::Superscript)],
        &[
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::token_pair(Token::Superscript, Token::Whitespace),
            ParseCondition::token_pair(Token::Whitespace, Token::Superscript),
        ],
        None,
    )
}
