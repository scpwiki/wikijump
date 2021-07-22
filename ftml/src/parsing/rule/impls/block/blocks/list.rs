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
use crate::tree::{ListItem, ListType};

// Definitions

pub const BLOCK_UL: BlockRule = BlockRule {
    name: "block-ul",
    accepts_names: &["ul"],
    accepts_star: false,
    accepts_score: true,
    accepts_newlines: true,
    parse_fn: parse_unordered_block,
};

pub const BLOCK_OL: BlockRule = BlockRule {
    name: "block-ol",
    accepts_names: &["ol"],
    accepts_star: false,
    accepts_score: true,
    accepts_newlines: true,
    parse_fn: parse_ordered_block,
};

pub const BLOCK_LI: BlockRule = BlockRule {
    name: "block-li",
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
) -> ParseResult<'r, 't, Elements<'t>> {
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
) -> ParseResult<'r, 't, Elements<'t>> {
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
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        log,
        "Parsing list block";
        "block-rule" => block_rule.name,
        "list_type" => list_type.name(),
        "in-head" => in_head,
        "name" => name,
    );

    assert!(!flag_star, "List block doesn't allow star flag");
    assert_block_name(block_rule, name);

    let arguments = parser.get_head_map(block_rule, in_head)?;
    let attributes = arguments.to_attribute_map();

    // TODO
    let exceptions = vec![];
    let element = Element::List {
        ltype: list_type,
        items: vec![],
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
) -> ParseResult<'r, 't, Elements<'t>> {
    todo!()
}
