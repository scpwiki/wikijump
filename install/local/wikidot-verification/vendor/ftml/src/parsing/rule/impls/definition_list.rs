/*
 * parsing/rule/impls/definition_list.rs
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

fn skip_newline<'r, 't>(
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Seeing if we skip due to an upcoming definition list");

    match parser.next_three_tokens() {
        // It looks like a definition list is upcoming
        (Token::LineBreak, Some(Token::Colon), Some(Token::Whitespace)) => {
            ok!(Elements::None)
        }

        // Anything else
        _ => Err(parser.make_err(ParseErrorKind::RuleFailed)),
    }
}

fn parse_definition_list<'r, 't>(
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Trying to create a definition list");

    let mut items = Vec::new();
    let mut errors = Vec::new();
    let mut _paragraph_safe = false;

    // Definition list needs at least one item
    let (item, at_end) = parse_item(parser)?.chain(&mut errors, &mut _paragraph_safe);

    items.push(item);

    // Collect remainder, halting if there's a failure
    if !at_end {
        loop {
            let sub_parser = &mut parser.clone();

            match parse_item(sub_parser) {
                Ok(success) => {
                    trace!("Retrieved definition list item");

                    let (item, at_end) = success.chain(&mut errors, &mut _paragraph_safe);

                    items.push(item);
                    parser.update(sub_parser);

                    if at_end {
                        break;
                    }
                }
                Err(error) => {
                    warn!(
                        "Failed to get the next definition list item, ending iteration: {error:?}"
                    );
                    break;
                }
            }
        }
    }

    // Build and return element
    ok!(Element::DefinitionList(items))
}

fn parse_item<'r, 't>(
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, (DefinitionListItem<'t>, bool)> {
    trace!("Trying to parse a definition list item pair");

    let mut errors = Vec::new();
    let mut _paragraph_safe = false;

    // The pattern for a definition list row is:
    // : key : value \n

    // Ensure the start of the line
    if !parser.start_of_line() {
        return Err(parser.make_err(ParseErrorKind::RuleFailed));
    }

    // Ensure that it matches expected token state
    if !matches!(
        parser.next_two_tokens(),
        (Token::Colon, Some(Token::Whitespace)),
    ) {
        return Err(parser.make_err(ParseErrorKind::RuleFailed));
    }

    parser.step_n(2)?;

    // Gather key elements until colon
    let start_token = parser.current();
    let mut key_elements = collect_consume(
        parser,
        RULE_DEFINITION_LIST,
        &[ParseCondition::token_pair(Token::Whitespace, Token::Colon)],
        &[
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::current(Token::LineBreak),
        ],
        None,
    )?
    .chain(&mut errors, &mut _paragraph_safe);
    let end_token = parser.current();

    strip_whitespace(&mut key_elements);
    parser.step_n(2)?;

    // Gather key wikitext
    let key_string = parser
        .full_text()
        .slice_partial(start_token, end_token)
        .trim();

    // Gather value elements until end of line
    let (mut value_elements, last) = collect_consume_keep(
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
    .chain(&mut errors, &mut _paragraph_safe);

    // Some ending tokens designate a definite end
    let should_break = match last.token {
        Token::ParagraphBreak | Token::InputEnd => true,
        Token::LineBreak => false,
        _ => panic!("Invalid close token: {}", last.token.name()),
    };

    strip_whitespace(&mut value_elements);

    // Build and return
    let item = DefinitionListItem {
        key_string: cow!(key_string),
        key_elements,
        value_elements,
    };

    ok!(false; (item, should_break), errors)
}
