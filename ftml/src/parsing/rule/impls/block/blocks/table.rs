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
use crate::parsing::parser::TableParseState;
use crate::parsing::strip_whitespace;
use crate::tree::{AttributeMap, PartialElement, Table, TableCell, TableRow};
use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};

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
    exceptions: Vec<ParseException<'t>>,
}

fn parse_block<'r, 't>(
    log: &Logger,
    parser: &mut ParserWrap<'_, 'r, 't>,
    name: &str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
    (block_rule, description): (&BlockRule, &str),
) -> Result<ParsedBlock<'t>, ParseWarning>
where
    'r: 't,
    ParsedBlock<'t>: 't,
{
    info!(
        log,
        "Parsing {} block",
        description;
        "in-head" => in_head,
        "name" => name,
    );

    assert!(
        !flag_star,
        "Block for {} doesn't allow star flag",
        description,
    );
    assert!(
        !flag_score,
        "Block for {} doesn't allow score flag",
        description,
    );
    assert_block_name(block_rule, name);

    // Get attributes
    let arguments = parser.get_head_map(block_rule, in_head)?;
    let attributes = arguments.to_attribute_map();

    // Get body elements
    let (elements, exceptions, _) = parser.get_body_elements(block_rule, false)?.into();

    // Return result
    Ok(ParsedBlock {
        elements,
        attributes,
        exceptions,
    })
}

macro_rules! extract_table_items {
    ($parser:expr, $elements:expr; $table_item_type:ident, $warning_kind:ident $(,)?) => {{
        let mut items = Vec::new();

        for element in $elements {
            match element {
                // Append the next table item.
                Element::Partial(PartialElement::$table_item_type(item)) => {
                    items.push(item);
                }

                // Ignore internal whitespace.
                element if element.is_whitespace() => (),

                // Return a warning for anything else.
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
    // This [[table]] is not in a regular content context.
    if parser.table_flag() != TableParseState::Content {
        return Err(parser.make_warn(ParseWarningKind::RuleFailed));
    }

    let parser = &mut ParserWrap::new(parser, TableParseState::Table);

    // Get block contents.
    let ParsedBlock {
        elements,
        attributes,
        exceptions,
    } = parse_block(
        log,
        parser,
        name,
        flag_star,
        flag_score,
        in_head,
        (&BLOCK_TABLE, "table block"),
    )?;

    let rows = extract_table_items!(parser, elements; TableRow, TableContainsNonRow);

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
    // This [[row]] is outside a [[table]], which is not allowed.
    // It also cannot be inside another [[row]].
    if parser.table_flag() != TableParseState::Table {
        return Err(parser.make_warn(ParseWarningKind::TableRowOutsideTable));
    }

    let parser = &mut ParserWrap::new(parser, TableParseState::Row);

    // Get block contents.
    let ParsedBlock {
        elements,
        attributes,
        exceptions,
    } = parse_block(
        log,
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
    // This [[cell]]/[[hcell]] is outside a [[row]], which is not allowed.
    if parser.table_flag() != TableParseState::Row {
        return Err(parser.make_warn(ParseWarningKind::TableCellOutsideTable));
    }

    let parser = &mut ParserWrap::new(parser, TableParseState::Content);

    // Get block contents.
    let ParsedBlock {
        elements,
        attributes,
        exceptions,
    } = parse_block(
        log,
        parser,
        name,
        flag_star,
        flag_score,
        in_head,
        (&BLOCK_TABLE_CELL_REGULAR, "table cell (regular)"),
    )?;

    parse_cell(elements, attributes, exceptions, false)
}

fn parse_cell_header<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    // This [[cell]]/[[hcell]] is outside a [[row]], which is not allowed.
    if parser.table_flag() != TableParseState::Row {
        return Err(parser.make_warn(ParseWarningKind::TableCellOutsideTable));
    }

    let parser = &mut ParserWrap::new(parser, TableParseState::Content);

    // Get block contents.
    let ParsedBlock {
        elements,
        attributes,
        exceptions,
    } = parse_block(
        log,
        parser,
        name,
        flag_star,
        flag_score,
        in_head,
        (&BLOCK_TABLE_CELL_HEADER, "table cell (header)"),
    )?;

    parse_cell(elements, attributes, exceptions, true)
}

fn parse_cell<'r, 't>(
    mut elements: Vec<Element<'t>>,
    mut attributes: AttributeMap<'t>,
    exceptions: Vec<ParseException<'t>>,
    header: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    lazy_static! {
        static ref ONE: NonZeroU32 = NonZeroU32::new(1).unwrap();
    }

    // Remove leading and trailing whitespace
    strip_whitespace(&mut elements);

    // Extract column-span if specified via attributes.
    // If not specified, then the default.
    let column_span = match attributes.remove("colspan") {
        Some(value) => value.parse().unwrap_or(*ONE),
        None => *ONE,
    };

    let element = Element::Partial(PartialElement::TableCell(TableCell {
        header,
        column_span,
        align: None,
        elements,
        attributes,
    }));

    ok!(false; element, exceptions)
}

// Helper

#[derive(Debug)]
struct ParserWrap<'p, 'r, 't> {
    parser: &'p mut Parser<'r, 't>,
    old_value: TableParseState,
}

impl<'p, 'r, 't> ParserWrap<'p, 'r, 't> {
    #[inline]
    fn new(parser: &'p mut Parser<'r, 't>, new_value: TableParseState) -> Self {
        let old_value = parser.table_flag();
        parser.set_table_flag(new_value);

        ParserWrap { parser, old_value }
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
        self.parser.set_table_flag(self.old_value);
    }
}
