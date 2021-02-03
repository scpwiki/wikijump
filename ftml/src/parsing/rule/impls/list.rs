/*
 * parsing/rule/impls/list.rs
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
use crate::parsing::{process_depths, DepthItem, DepthList};
use crate::span_wrap::SpanWrap;
use crate::tree::{ListItem, ListType};

pub const RULE_BULLET_LIST: Rule = Rule {
    name: "bullet-list",
    try_consume_fn,
};

pub const RULE_NUMBERED_LIST: Rule = Rule {
    name: "numbered-list",
    try_consume_fn,
};

fn try_consume_fn<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Element<'t>> {
    // We don't know the list type(s) yet, so just log that we're starting
    debug!(log, "Parsing a list");

    assert!(
        parser.current().token == Token::InputStart
            || parser.current().token == Token::LineBreak,
        "Starting token for list is not start of input or newline",
    );
    parser.step()?;

    let mut top_list_type = None;

    // Produce a depth list with elements
    let mut depths = Vec::new();
    let mut exceptions = Vec::new();

    loop {
        let current = parser.current();
        let depth = match current.token {
            // Count the number of spaces for its depth
            Token::Whitespace => {
                let spaces = parser.current().slice;
                parser.step()?;

                // Since these are only ASCII spaces a byte count is fine
                spaces.len()
            }

            // No depth, just the bullet
            Token::BulletItem | Token::NumberedItem => 0,

            // Invalid token, bail
            _ => {
                debug!(
                    log,
                    "Didn't find correct bullet token or couldn't determine list depth, ending list iteration";
                    "token" => current.token,
                    "slice" => current.slice,
                    "span" => SpanWrap::from(&current.span),
                );

                break;
            }
        };

        // Check that we're processing a bullet, and get the type
        let (list_type, rule) = {
            let current = parser.current();
            let bullet_token = parser.current().token;

            match get_list_type(bullet_token) {
                Some((list_type, rule)) => (list_type, rule),
                None => {
                    debug!(
                        log,
                        "Didn't find bullet token, couldn't determine list type, ending list iteration";
                        "token" => current.token,
                        "slice" => current.slice,
                        "span" => SpanWrap::from(&current.span),
                    );

                    break;
                }
            }
        };
        parser.step()?;

        // TODO: for now, until we generate lists based on item type
        top_list_type = Some(list_type);

        debug!(
            log,
            "Parsing listen item";
            "rule" => rule,
            "list-type" => list_type.name(),
        );

        // For now, always expect whitespace after the bullet
        let current = parser.current();
        if current.token != Token::Whitespace {
            debug!(
                log,
                "Didn't find whitespace after bullet token, ending list iteration";
                "token" => current.token,
                "slice" => current.slice,
                "span" => SpanWrap::from(&current.span),
            );

            break;
        }
        parser.step()?;

        // Parse elements until we hit the end of the line
        let elements = collect_consume(
            log,
            parser,
            rule,
            &[
                ParseCondition::current(Token::LineBreak),
                ParseCondition::current(Token::InputEnd),
            ],
            &[ParseCondition::current(Token::ParagraphBreak)],
            None,
        )?
        .chain(&mut exceptions);

        // Append bullet line
        depths.push((depth, elements));
    }

    // Our rule is in another castle
    if depths.is_empty() {
        return Err(parser.make_warn(ParseWarningKind::RuleFailed));
    }

    // Build a tree structure from our depths list
    let depth_list = process_depths(depths);
    // NOTE unwrap is safe since we check depths.is_empty(), which means at least one iteration
    let element = build_list_element(depth_list, top_list_type.unwrap());

    ok!(element, exceptions)
}

fn build_list_element(list: DepthList<Vec<Element>>, ltype: ListType) -> Element {
    let build_item = |item| match item {
        DepthItem::Item(elements) => ListItem::Elements(elements),
        DepthItem::List(list) => ListItem::SubList(build_list_element(list, ltype)),
    };

    let items = list.into_iter().map(build_item).collect();

    // Return the Element::List object
    Element::List { ltype, items }
}

const fn get_list_type(token: Token) -> Option<(ListType, Rule)> {
    match token {
        Token::BulletItem => Some((ListType::Bullet, RULE_BULLET_LIST)),
        Token::NumberedItem => Some((ListType::Numbered, RULE_NUMBERED_LIST)),
        _ => None,
    }
}
