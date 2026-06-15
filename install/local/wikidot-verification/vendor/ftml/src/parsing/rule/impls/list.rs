/*
 * parsing/rule/impls/list.rs
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
use crate::parsing::{DepthItem, DepthList, process_depths};
use crate::tree::{AttributeMap, ListItem, ListType};

const MAX_LIST_DEPTH: usize = 20;

const fn get_list_type(token: Token) -> Option<ListType> {
    match token {
        Token::BulletItem => Some(ListType::Bullet),
        Token::NumberedItem => Some(ListType::Numbered),
        _ => None,
    }
}

pub const RULE_LIST: Rule = Rule {
    name: "list",
    position: LineRequirement::StartOfLine,
    try_consume_fn,
};

fn try_consume_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    // We don't know the list type(s) yet, so just log that we're starting
    debug!("Parsing a list");

    // Context variables
    let mut depths = Vec::new();
    let mut errors = Vec::new();

    // Blockquotes are always paragraph-unsafe,
    // but we need this binding for chain().
    let mut paragraph_safe = false;

    // Produce a depth list with elements
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
                warn!(
                    "Didn't find correct bullet token or couldn't determine list depth, ending list iteration"
                );
                break;
            }
        };

        // Check that the depth isn't obscenely deep, to avoid DOS attacks via stack overflow.
        if depth > MAX_LIST_DEPTH {
            warn!(
                "List item has a depth {depth} greater than the maximum ({MAX_LIST_DEPTH})! Failing"
            );
            return Err(parser.make_err(ParseErrorKind::ListDepthExceeded));
        }

        // Check that we're processing a bullet, and get the type
        let current = parser.current();
        let list_type = match get_list_type(current.token) {
            Some(ltype) => ltype,
            None => {
                trace!(
                    "Didn't find bullet token, couldn't determine list type, ending list iteration"
                );
                break;
            }
        };
        parser.step()?;

        trace!("Parsing list item '{}'", list_type.name());

        // For now, always expect whitespace after the bullet
        let current = parser.current();
        if current.token != Token::Whitespace {
            warn!("Didn't find whitespace after bullet token, ending list iteration");
            break;
        }
        parser.step()?;

        // Parse elements until we hit the end of the line
        let elements = collect_consume(
            parser,
            RULE_LIST,
            &[
                ParseCondition::current(Token::LineBreak),
                ParseCondition::current(Token::ParagraphBreak),
                ParseCondition::current(Token::InputEnd),
            ],
            &[],
            None,
        )?
        .chain(&mut errors, &mut paragraph_safe);

        // Empty list lines are ignored
        if elements.is_empty() {
            trace!("Skipping empty list line");
            continue;
        }

        // Append list line
        depths.push((depth, list_type, elements));
    }

    // This list has no rows, so the rule fails
    if depths.is_empty() {
        return Err(parser.make_err(ParseErrorKind::RuleFailed));
    }

    let depth_lists = process_depths(ListType::Generic, depths);
    let elements: Vec<Element> = depth_lists
        .into_iter()
        .map(|(ltype, depth_list)| build_list_element(ltype, depth_list))
        .collect();

    ok!(paragraph_safe; elements, errors)
}

fn build_list_element(
    top_ltype: ListType,
    list: DepthList<ListType, Vec<Element>>,
) -> Element {
    let build_item = |item| match item {
        DepthItem::Item(elements) => ListItem::Elements {
            elements,
            attributes: AttributeMap::new(),
        },
        DepthItem::List(ltype, list) => ListItem::SubList {
            element: Box::new(build_list_element(ltype, list)),
        },
    };

    let items = list.into_iter().map(build_item).collect();
    let attributes = AttributeMap::new();

    // Return the Element::List object
    Element::List {
        ltype: top_ltype,
        items,
        attributes,
    }
}
