/*
 * parsing/rule/impls/block/blocks/list.rs
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
use crate::parsing::{ParserWrap, strip_newlines};
use crate::tree::{AcceptsPartial, ListItem, ListType, PartialElement};

// Definitions

pub const BLOCK_UL: BlockRule = BlockRule {
    name: "block-list-unordered",
    accepts_names: &["ul"],
    accepts_star: false,
    accepts_score: true,
    accepts_newlines: true,
    parse_fn: parse_unordered_block,
};

pub const BLOCK_OL: BlockRule = BlockRule {
    name: "block-list-ordered",
    accepts_names: &["ol"],
    accepts_star: false,
    accepts_score: true,
    accepts_newlines: true,
    parse_fn: parse_ordered_block,
};

pub const BLOCK_LI: BlockRule = BlockRule {
    name: "block-list-item",
    accepts_names: &["li"],
    accepts_star: false,
    accepts_score: true,
    accepts_newlines: true,
    parse_fn: parse_list_item,
};

fn parse_unordered_block<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    parse_list_block(
        (&BLOCK_UL, ListType::Bullet),
        parser,
        name,
        flag_star,
        flag_score,
        in_head,
    )
}

fn parse_ordered_block<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    parse_list_block(
        (&BLOCK_OL, ListType::Numbered),
        parser,
        name,
        flag_star,
        flag_score,
        in_head,
    )
}

// List block

fn parse_list_block<'r, 't>(
    (block_rule, list_type): (&BlockRule, ListType),
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        "Parsing list block (name '{}', rule {}, list type {}, in-head {}, score {})",
        name,
        block_rule.name,
        list_type.name(),
        in_head,
        flag_score,
    );

    let parser = &mut ParserWrap::new(parser, AcceptsPartial::ListItem);

    assert!(!flag_star, "List block doesn't allow star flag");
    assert_block_name(block_rule, name);

    // "ul" means we wrap interpret as-is
    // "ul_" means we strip out any newlines or paragraph breaks
    let strip_line_breaks = flag_score;

    // Get attributes
    let arguments = parser.get_head_map(block_rule, in_head)?;
    let attributes = arguments.to_attribute_map(parser.settings());

    // Get body and convert into list form.
    let (mut elements, errors, _) = parser.get_body_elements(block_rule, false)?.into();

    let items = {
        let mut items = Vec::new();

        // Strip newlines, if desired
        if strip_line_breaks {
            strip_newlines(&mut elements);
        }

        // Empty lists aren't allowed
        if elements.is_empty() {
            return Err(parser.make_err(ParseErrorKind::ListEmpty));
        }

        // Convert and extract list elements
        for element in elements {
            match element {
                // Ensure all elements of a list are only items, i.e. [[li]].
                Element::Partial(PartialElement::ListItem(list_item)) => {
                    items.push(list_item);
                }

                // Or sub-lists.
                Element::List {
                    ltype,
                    attributes,
                    items: sub_items,
                } => {
                    let element = Box::new(Element::List {
                        ltype,
                        attributes,
                        items: sub_items,
                    });

                    items.push(ListItem::SubList { element });
                }

                // Ignore "whitespace" elements
                element if element.is_whitespace() => continue,

                // Other kinds of elements result in an exception.
                _ => return Err(parser.make_err(ParseErrorKind::ListContainsNonItem)),
            }
        }

        items
    };

    let element = Element::List {
        ltype: list_type,
        items,
        attributes,
    };

    ok!(false; element, errors)
}

// List item

fn parse_list_item<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        "Parsing list item block (name '{}', in-head {}, score {})",
        name, in_head, flag_score,
    );
    assert!(!flag_star, "List item block doesn't allow star flag");
    assert_block_name(&BLOCK_LI, name);

    // "li" means we wrap interpret as-is
    // "li_" means we strip out any newlines or paragraph breaks
    let strip_line_breaks = flag_score;

    // Get attributes
    let arguments = parser.get_head_map(&BLOCK_LI, in_head)?;
    let attributes = arguments.to_attribute_map(parser.settings());

    // Get body elements
    let (mut elements, errors, _) = parser.get_body_elements(&BLOCK_LI, false)?.into();

    // Strip newlines, if desired
    if strip_line_breaks {
        strip_newlines(&mut elements);
    }

    let element = Element::Partial(PartialElement::ListItem(ListItem::Elements {
        elements,
        attributes,
    }));

    ok!(false; element, errors)
}
