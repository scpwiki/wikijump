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
    accepts_newlines: false,
    parse_fn: parse_ul_block,
};

pub const BLOCK_OL: BlockRule = BlockRule {
    name: "block-ol",
    accepts_names: &["ol"],
    accepts_star: false,
    accepts_score: true,
    accepts_newlines: false,
    parse_fn: parse_ol_block,
};

fn parse_ul_block<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    parse_list_block(ListType::Bullet, log, parser, name, flag_star, flag_score, in_head)
}

fn parse_ol_block<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    parse_list_block(ListType::Numbered, log, parser, name, flag_star, flag_score, in_head)
}

// List block

fn parse_list_block<'r, 't>(
    list_type: ListType,
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    todo!()
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
