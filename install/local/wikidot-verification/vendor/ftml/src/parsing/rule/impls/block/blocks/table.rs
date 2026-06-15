/*
 * parsing/rule/impls/block/blocks/table.rs
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
use crate::parsing::{ParserWrap, strip_whitespace};
use crate::tree::{
    AcceptsPartial, AttributeMap, PartialElement, Table, TableCell, TableRow, TableType,
};
use std::num::NonZeroU32;

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
    name: "block-table-cell-regular",
    accepts_names: &["cell"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn: parse_cell_regular,
};

pub const BLOCK_TABLE_CELL_HEADER: BlockRule = BlockRule {
    name: "block-table-cell-header",
    accepts_names: &["hcell"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn: parse_cell_header,
};

// Helper functions and macros

#[derive(Debug)]
struct ParsedBlock<'t> {
    elements: Vec<Element<'t>>,
    attributes: AttributeMap<'t>,
    errors: Vec<ParseError>,
}

fn parse_block<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
    (block_rule, description): (&BlockRule, &str),
) -> Result<ParsedBlock<'t>, ParseError>
where
    'r: 't,
    ParsedBlock<'t>: 't,
{
    debug!("Parsing {description} block (name '{name}', in-head {in_head})");
    assert!(
        !flag_star,
        "Block for {description} doesn't allow star flag",
    );
    assert!(
        !flag_score,
        "Block for {description} doesn't allow score flag",
    );
    assert_block_name(block_rule, name);

    // Get attributes
    let arguments = parser.get_head_map(block_rule, in_head)?;
    let attributes = arguments.to_attribute_map(parser.settings());

    // Get body elements
    let (elements, errors, _) = parser.get_body_elements(block_rule, false)?.into();

    // Return result
    Ok(ParsedBlock {
        elements,
        attributes,
        errors,
    })
}

macro_rules! extract_table_items {
    ($parser:expr, $elements:expr; $table_item_type:ident, $error_kind:ident $(,)?) => {{
        let mut items = Vec::new();

        for element in $elements {
            match element {
                // Append the next table item.
                Element::Partial(PartialElement::$table_item_type(item)) => {
                    items.push(item);
                }

                // Ignore internal whitespace.
                element if element.is_whitespace() => (),

                // Return an error for anything else.
                _ => return Err($parser.make_err(ParseErrorKind::$error_kind)),
            }
        }

        items
    }};
}

// Table block

fn parse_table<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    let parser = &mut ParserWrap::new(parser, AcceptsPartial::TableRow);

    // Get block contents.
    let ParsedBlock {
        elements,
        attributes,
        errors,
    } = parse_block(
        parser,
        name,
        flag_star,
        flag_score,
        in_head,
        (&BLOCK_TABLE, "table block"),
    )?;

    let rows = extract_table_items!(parser, elements; TableRow, TableContainsNonRow);

    // Build and return table element
    let element = Element::Table(Table {
        rows,
        attributes,
        table_type: TableType::Advanced,
    });
    ok!(false; element, errors)
}

// Table row

fn parse_row<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    let parser = &mut ParserWrap::new(parser, AcceptsPartial::TableCell);

    // Get block contents.
    let ParsedBlock {
        elements,
        attributes,
        errors,
    } = parse_block(
        parser,
        name,
        flag_star,
        flag_score,
        in_head,
        (&BLOCK_TABLE_ROW, "table row"),
    )?;

    let cells =
        extract_table_items!(parser, elements; TableCell, TableRowContainsNonCell);

    // Build and return table row
    let element =
        Element::Partial(PartialElement::TableRow(TableRow { cells, attributes }));

    ok!(false; element, errors)
}

// Table cell

fn parse_cell_regular<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    // Get block contents.
    let ParsedBlock {
        elements,
        attributes,
        errors,
    } = parse_block(
        parser,
        name,
        flag_star,
        flag_score,
        in_head,
        (&BLOCK_TABLE_CELL_REGULAR, "table cell (regular)"),
    )?;

    parse_cell(elements, attributes, errors, false)
}

fn parse_cell_header<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    let parser = &mut ParserWrap::new(parser, AcceptsPartial::TableCell);

    // Get block contents.
    let ParsedBlock {
        elements,
        attributes,
        errors,
    } = parse_block(
        parser,
        name,
        flag_star,
        flag_score,
        in_head,
        (&BLOCK_TABLE_CELL_HEADER, "table cell (header)"),
    )?;

    parse_cell(elements, attributes, errors, true)
}

fn parse_cell<'r, 't>(
    mut elements: Vec<Element<'t>>,
    mut attributes: AttributeMap<'t>,
    errors: Vec<ParseError>,
    header: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    // Remove leading and trailing whitespace
    strip_whitespace(&mut elements);

    // Extract column-span if specified via attributes.
    // If not specified, then the default.
    let column_span = match attributes.remove("colspan") {
        Some(value) => value.parse().unwrap_or(NonZeroU32::new(1).unwrap()),
        None => NonZeroU32::new(1).unwrap(),
    };

    let element = Element::Partial(PartialElement::TableCell(TableCell {
        header,
        column_span,
        align: None,
        elements,
        attributes,
    }));

    ok!(false; element, errors)
}
