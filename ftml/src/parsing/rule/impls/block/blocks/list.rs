/*
 * parsing/rule/impls/block/blocks/list.rs
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
use crate::parsing::strip_newlines;
use crate::tree::{ListItem, ListType};
use std::ops::{Deref, DerefMut};

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
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, PartialElements<'t>> {
    parse_list_block(
        (&BLOCK_UL, ListType::Bullet),
        log,
        parser,
        name,
        flag_star,
        flag_score,
        in_head,
    )
}

fn parse_ordered_block<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, PartialElements<'t>> {
    parse_list_block(
        (&BLOCK_OL, ListType::Numbered),
        log,
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
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, PartialElements<'t>> {
    info!(
        log,
        "Parsing list block";
        "block-rule" => block_rule.name,
        "list-type" => list_type.name(),
        "flag-score" => flag_score,
        "in-head" => in_head,
        "name" => name,
    );

    assert!(!flag_star, "List block doesn't allow star flag");
    assert_block_name(block_rule, name);

    // Enable flag for list interior.
    let mut parser = ParserWrap::new(parser, true);

    // "ul" means we wrap interpret as-is
    // "ul_" means we strip out any newlines or paragraph breaks
    let strip_line_breaks = flag_score;

    // Get attributes
    let arguments = parser.get_head_map(block_rule, in_head)?;
    let attributes = arguments.to_attribute_map();

    // Get body and convert into list form.
    let (mut partials, exceptions, _) =
        parser.get_body_elements(block_rule, false)?.into();

    let items = {
        let mut items = Vec::new();

        // Strip newlines, if desired
        if strip_line_breaks {
            strip_newlines(&mut elements);
        }

        // Empty lists aren't allowed
        if partials.is_empty() {
            return Err(parser.make_warn(ParseWarningKind::ListEmpty));
        }

        // Convert and extract partial list elements
        for partial in partials {
            match partial {
                // Ensure all elements of a list are only items, i.e. [[li]].
                PartialElement::ListItem(item) => items.push(*item),

                // Or sub-lists.
                PartialElement::Complete(Element::List {
                    ltype,
                    attributes,
                    items: sub_items,
                }) => {
                    let element = Element::List {
                        ltype,
                        attributes,
                        items: sub_items,
                    };
                    let item = ListItem::SubList(element);
                    items.push(item);
                }

                // Ignore "whitespace" elements
                PartialElement::Complete(element) if element.is_whitespace() => continue,

                // Other kinds of elements result in an exception.
                _ => return Err(parser.make_warn(ParseWarningKind::ListContainsNonItem)),
            }
        }

        items
    };

    let element = Element::List {
        ltype: list_type,
        items,
        attributes,
    };

    ok!(false; element, exceptions)
}

// List item

fn parse_list_item<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, PartialElements<'t>> {
    info!(
        log,
        "Parsing list item block";
        "block-rule" => BLOCK_LI.name,
        "flag-score" => flag_score,
        "in-head" => in_head,
        "name" => name,
    );

    assert!(!flag_star, "List item block doesn't allow star flag");
    assert_block_name(&BLOCK_LI, name);

    // This [[li]] is outside of a [[ol]] or [[ul]], which is not allowed.
    if !parser.in_list() {
        return Err(parser.make_warn(ParseWarningKind::ListItemOutsideList));
    }

    // Disable flag for list items.
    let mut parser = ParserWrap::new(parser, false);

    // "li" means we wrap interpret as-is
    // "li_" means we strip out any newlines or paragraph breaks
    let strip_line_breaks = flag_score;

    // Get attributes
    let arguments = parser.get_head_map(&BLOCK_LI, in_head)?;
    let attributes = arguments.to_attribute_map();

    // Get body elements
    let (mut elements, exceptions, _) =
        parser.get_body_elements(&BLOCK_LI, false)?.into();

    // Strip newlines, if desired
    if strip_line_breaks {
        strip_newlines(&mut elements);
    }

    let list_item = ListItem::Elements {
        elements,
        attributes,
    };
    let element = Element::ListItem(Box::new(list_item));

    ok!(false; element, exceptions)
}

// Helper

#[derive(Debug)]
struct ParserWrap<'p, 'r, 't> {
    value: bool,
    parser: &'p mut Parser<'r, 't>,
}

impl<'p, 'r, 't> ParserWrap<'p, 'r, 't> {
    #[inline]
    fn new(parser: &'p mut Parser<'r, 't>, value: bool) -> Self {
        parser.set_list_flag(value);

        ParserWrap { parser, value }
    }
}

impl<'r, 't> Deref for ParserWrap<'_, 'r, 't> {
    type Target = Parser<'r, 't>;

    #[inline]
    fn deref(&self) -> &Parser<'r, 't> {
        self.parser
    }
}

impl<'r, 't> DerefMut for ParserWrap<'_, 'r, 't> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Parser<'r, 't> {
        self.parser
    }
}

impl Drop for ParserWrap<'_, '_, '_> {
    fn drop(&mut self) {
        self.parser.set_list_flag(!self.value);
    }
}
