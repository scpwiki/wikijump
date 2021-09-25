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
use crate::parsing::{Token, strip_whitespace};
use crate::tree::DefinitionListItem;

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

    match parser.next_three_tokens() {
        // It looks like a definition list is upcoming
        (Token::LineBreak, Some(Token::Colon), Some(Token::Whitespace)) => {
            ok!(Elements::None)
        }

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
    let mut exceptions = Vec::new();
    let mut _paragraph_safe = false;

    // Definition list needs at least one item
    let item = parse_item(log, parser)?
        .chain(&mut exceptions, &mut _paragraph_safe);

    items.push(item);

    // Collect remainder, halting if there's a failure
    loop {
        let sub_parser = &mut parser.clone();

        match parse_item(log, sub_parser) {
            Ok(success) => {
                debug!(log, "Retrieved definition list item");

                let item = success.chain(&mut exceptions, &mut _paragraph_safe);

                items.push(item);
                parser.update(sub_parser);
            }
            Err(warn) => {
                warn!(
                    log,
                    "Failed to get the next definition list item, ending iteration";
                    "warning" => format!("{:#?}", warn),
                );

                break;
            }
        }
    }

    // Build and return element
    ok!(Element::DefinitionList(items))
}

fn parse_item<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, DefinitionListItem<'t>> {
    debug!(log, "Trying to parse a definition list item pair");

    let mut exceptions = Vec::new();
    let mut _paragraph_safe = false;

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
        return Err(parser.make_warn(ParseWarningKind::RuleFailed));
    }

    parser.step_n(2)?;

    // Gather key text until colon
    let mut key = collect_consume(
        log,
        parser,
        RULE_DEFINITION_LIST,
        &[ParseCondition::token_pair(Token::Whitespace, Token::Colon)],
        &[
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::current(Token::LineBreak),
        ],
        None,
    )?
    .chain(&mut exceptions, &mut _paragraph_safe);

    strip_whitespace(&mut key);
    parser.step_n(2)?;

    // Gather value text until end of line
    let mut value = collect_consume(
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
    )?
    .chain(&mut exceptions, &mut _paragraph_safe);

    strip_whitespace(&mut value);

    ok!(false; DefinitionListItem { key, value }, exceptions)
}
