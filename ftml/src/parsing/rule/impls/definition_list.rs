/*
 * parsing/rule/impls/definition_list.rs
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
use crate::parsing::Token;
use std::borrow::Cow;

type DefinitionItem<'t> = (Cow<'t, str>, Cow<'t, str>);

pub const RULE_DEFINITION_LIST: Rule = Rule {
    name: "definition-list",
    position: LineRequirement::StartOfLine,
    try_consume_fn: parse_definition_list,
};

pub const RULE_DEFINITION_LIST_SKIP_NEWLINE: Rule = Rule {
    name: "definition-list-skip-newline",
    position: LineRequirement::Any,
    try_consume_fn: skip_newline,
};

fn skip_newline<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    info!(log, "Seeing if we skip due to an upcoming definition list");

    let current = parser.current().token;
    let second = parser.look_ahead(0).map(|extract| extract.token);
    let third = parser.look_ahead(1).map(|extract| extract.token);

    match (current, second, third) {
        // It looks like a definition list is upcoming
        (Token::LineBreak, Some(Token::Colon), Some(Token::Whitespace)) => ok!(Elements::None),

        // Anything else
        _ => Err(parser.make_warn(ParseWarningKind::RuleFailed)),
    }
}

fn parse_definition_list<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    info!(log, "Trying to create a definition list");

    let mut items = Vec::new();

    while let Some(item) = parse_item(log, parser)? {
        items.push(item);
    }

    // Definition list must have at least one pair
    if items.is_empty() {
        return Err(parser.make_warn(ParseWarningKind::RuleFailed));
    }

    // Build and return element
    ok!(Element::DefinitionList(items))
}

fn parse_item<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> Result<Option<DefinitionItem<'t>>, ParseWarning>
where
    'r: 't,
{
    debug!(log, "Trying to parse a definition list item pair");

    // The pattern for a definition list row is:
    // : key : value \n

    // Ensure the start of the line
    if !parser.start_of_line() {
        return Err(parser.make_warn(ParseWarningKind::RuleFailed));
    }

    // Ensure that it matches expected token state
    if !matches!(
        parser.next_two_tokens(),
        (Token::Colon, Some(Token::Whitespace)),
    ) {
        return Ok(None);
    }

    parser.step_n(2)?;

    // Gather key text until colon
    let key = collect_text(
        log,
        parser,
        RULE_DEFINITION_LIST,
        &[ParseCondition::token_pair(Token::Whitespace, Token::Colon)],
        &[
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::current(Token::LineBreak),
        ],
        None,
    )?;

    parser.step_n(2)?;

    // Gather value text until end of line
    let value = collect_text(
        log,
        parser,
        RULE_DEFINITION_LIST,
        &[
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::current(Token::LineBreak),
            ParseCondition::current(Token::InputEnd),
        ],
        &[],
        None,
    )?;

    let key = cow!(key.trim());
    let value = cow!(value.trim());

    Ok(Some((key, value)))
}
