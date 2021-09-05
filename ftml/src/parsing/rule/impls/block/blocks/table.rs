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
use crate::tree::{AttributeMap, Table, TableItem, TableRow};

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

// Helper functions and macros

fn parse_block<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
    (block_rule, description): (&BlockRule, &str),
) -> ParseResult<'r, 't, (Vec<Element<'t>>, AttributeMap<'t>)> {
    debug!(
        log,
        "Parsing {} block",
        description;
        "in-head" => in_head,
        "name" => name,
    );

    assert!(
        !flag_star,
        "Block for {} doesn't allow star flag",
        description
    );
    assert!(
        !flag_score,
        "Block for {} doesn't allow score flag",
        description
    );
    assert_block_name(block_rule, name);

    // Get attributes
    let arguments = parser.get_head_map(block_rule, in_head)?;
    let attributes = arguments.to_attribute_map();

    // Get body elements
    let (elements, exceptions, _) = parser.get_body_elements(block_rule, false)?.into();

    // Item to return to the caller
    let result = (elements, attributes);

    ok!(false; result, exceptions)
}

macro_rules! extract_table_items {
    ($parser:expr, $elements:expr; $item_type:ident, $warning_kind:ident $(,)?) => {{
        let mut items = Vec::new();

        for element in $elements {
            match element {
                Element::TableItem(TableItem::$item_type(item)) => items.push(item),
                _ => return Err($parser.make_warn(ParseWarningKind::$warning_kind)),
            }
        }

        items
    }};
}

// Table block

fn parse_table<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    // The returned item is (elements, attributes).
    // Breaking apart a ParseSuccess yields (returned_item, elements, paragraph_safe).
    //
    // Ditto for other block rule functions.

    let ((elements, attributes), exceptions, _) = parse_block(
        log,
        parser,
        name,
        flag_star,
        flag_score,
        in_head,
        (&BLOCK_TABLE, "table block"),
    )?
    .into();

    let rows = extract_table_items!(parser, elements; Row, TableContainsNonRow);

    // Build and return table element
    let element = Element::Table(Table { rows, attributes });

    ok!(false; element, exceptions)
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
    let ((elements, attributes), exceptions, _) = parse_block(
        log,
        parser,
        name,
        flag_star,
        flag_score,
        in_head,
        (&BLOCK_TABLE_ROW, "table row"),
    )?
    .into();

    let cells = extract_table_items!(parser, elements; Cell, TableRowContainsNonCell);

    // Build and return table row
    let element = Element::TableItem(TableItem::Row(TableRow { cells, attributes }));

    ok!(false; element, exceptions)
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
    let ((elements, attributes), exceptions, _) = parse_block(
        log,
        parser,
        name,
        flag_star,
        flag_score,
        in_head,
        (&BLOCK_TABLE_CELL_REGULAR, "table cell (regular)"),
    )?
    .into();

    parse_cell(log, parser, elements, attributes, exceptions, false)
}

fn parse_cell_header<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    let ((elements, attributes), exceptions, _) = parse_block(
        log,
        parser,
        name,
        flag_star,
        flag_score,
        in_head,
        (&BLOCK_TABLE_CELL_HEADER, "table cell (header)"),
    )?
    .into();

    parse_cell(log, parser, elements, attributes, exceptions, true)
}

fn parse_cell<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    elements: Vec<Element<'t>>,
    attributes: AttributeMap<'t>,
    exceptions: Vec<ParseException<'t>>,
    header: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    todo!()
}
