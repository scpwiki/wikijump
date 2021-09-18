/*
 * parsing/rule/impls/bold.rs
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

pub const RULE_BOLD: Rule = Rule {
    name: "bold",
    position: LineRequirement::Any,
    try_consume_fn,
};

fn try_consume_fn<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, PartialElements<'t>> {
    info!(log, "Trying to create bold (strong) container");

    check_step(parser, Token::Bold)?;

    collect_container(
        log,
        parser,
        RULE_BOLD,
        ContainerType::Bold,
        &[ParseCondition::current(Token::Bold)],
        &[
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::token_pair(Token::Bold, Token::Whitespace),
            ParseCondition::token_pair(Token::Whitespace, Token::Bold),
        ],
        None,
    )
}
