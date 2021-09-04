/*
 * parsing/rule/impls/block/blocks/table.rs
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

pub const BLOCK_TABLE: BlockRule = BlockRule {
    name: "block-table",
    accepts_names: &["table"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn: parse_table,
};

pub const BLOCK_TABLE_ROW: BlockRule = BlockRule {
    name: "block-table-row",
    accepts_names: &["row"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn: parse_row,
};

pub const BLOCK_TABLE_CELL_REGULAR: BlockRule = BlockRule {
    name: "block-table-row",
    accepts_names: &["cell"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn: parse_cell_regular,
};

pub const BLOCK_TABLE_CELL_HEADER: BlockRule = BlockRule {
    name: "block-table-row",
    accepts_names: &["hcell"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn: parse_cell_header,
};

// Table block

fn parse_table<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        log,
        "Parsing table block";
        "in-head" => in_head,
        "name" => name,
    );

    assert!(!flag_star, "Table block doesn't allow star flag");
    assert!(!flag_score, "Table block doesn't allow score flag");
    assert_block_name(&BLOCK_TABLE, name);

    todo!();
}

// Table row

fn parse_row<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        log,
        "Parsing table row block";
        "in-head" => in_head,
        "name" => name,
    );

    assert!(!flag_star, "Table row block doesn't allow star flag");
    assert!(!flag_score, "Table row block doesn't allow score flag");
    assert_block_name(&BLOCK_TABLE_ROW, name);

    // Get attributes
    let arguments = parser.get_head_map(&BLOCK_TABLE_ROW, in_head)?;
    let attributes = arguments.to_attribute_map();

    // Get body, a list of cells
    let (elements, exceptions, _) =
        parser.get_body_elements(&BLOCK_TABLE_ROW, false)?.into();

    // TODO element
    todo!()
}

// Table cell

fn parse_cell_regular<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    todo!();

    parse_cell(log, parser, &BLOCK_TABLE_CELL_REGULAR, false)
}

fn parse_cell_header<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    todo!();

    parse_cell(log, parser, &BLOCK_TABLE_CELL_HEADER, true)
}

fn parse_cell<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    block_rule: &BlockRule,
    header: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    todo!()
}
