/*
 * parsing/rule/impls/table.rs
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
use crate::tree::{Alignment, Table, TableCell, TableRow};
use std::num::NonZeroU32;

#[derive(Debug)]
struct TableCellStart {
    align: Option<Alignment>,
    header: bool,
    column_span: NonZeroU32,
}

pub const RULE_TABLE: Rule = Rule {
    name: "table",
    position: LineRequirement::StartOfLine,
    try_consume_fn,
};

fn try_consume_fn<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(log, "Trying to parse simple table");

    let mut rows = Vec::new();
    let mut exceptions = Vec::new();
    let mut _paragraph_break = false;

    'table: loop {
        debug!(log, "Parsing next table row");

        let mut cells = Vec::new();

        // Loop for each cell in the row
        'row: loop {
            debug!(log, "Parsing next table cell"; "cells" => cells.len());

            let mut elements = Vec::new();
            let TableCellStart {
                align,
                header,
                column_span,
            } = parse_cell_start(parser)?;

            // Loop for each element in the cell
            'cell: loop {
                debug!(log, "Parsing next element"; "elements" => elements.len());

                match next_two_tokens(parser) {
                    // Special case:
                    //
                    // If there is "_\n" next, then treat this as a newline insertion.
                    // Since normally a newline will end the row, but we want a <br>
                    // in the cell contents.
                    (
                        Token::Underscore,
                        Some(Token::LineBreak | Token::ParagraphBreak),
                    ) => {
                        trace!(log, "Handling newline escape in table");

                        elements.push(Element::LineBreak);
                        parser.step_n(2)?;
                    }

                    // End the cell or row
                    (
                        Token::TableColumn
                        | Token::TableColumnTitle
                        | Token::TableColumnLeft
                        | Token::TableColumnCenter
                        | Token::TableColumnRight,
                        Some(next),
                    ) => {
                        trace!(log, "Ending cell, row, or table"; "next-token" => next.name());

                        match next {
                            // End the table entirely, there's a newline in between,
                            // or it's the end of input.
                            Token::ParagraphBreak | Token::InputEnd => {
                                if next != Token::InputEnd {
                                    parser.step()?;
                                }

                                rows.push(TableRow {
                                    cells,
                                    attributes: AttributeMap::new(),
                                });

                                break 'table;
                            }

                            // Only end the row, continue the table.
                            Token::LineBreak => {
                                parser.step()?;

                                cells.push(TableCell {
                                    elements,
                                    header,
                                    column_span,
                                    align,
                                    attributes: AttributeMap::new(),
                                });

                                break 'row;
                            }

                            // Otherwise, the cell is finished, and we proceed to the next one.
                            _ => break 'cell,
                        }
                    }

                    // Ignore leading whitespace
                    (Token::Whitespace, _) if elements.is_empty() => {
                        trace!(log, "Ignoring leading whitespace");

                        parser.step()?;
                        continue 'cell;
                    }

                    // Ignore trailing whitespace
                    (
                        Token::Whitespace,
                        Some(
                            Token::TableColumn
                            | Token::TableColumnTitle
                            | Token::TableColumnLeft
                            | Token::TableColumnCenter
                            | Token::TableColumnRight,
                        ),
                    ) => {
                        trace!(log, "Ignoring trailing whitespace");

                        parser.step()?;
                        continue 'cell;
                    }

                    // Invalid tokens
                    (Token::LineBreak | Token::ParagraphBreak | Token::InputEnd, _) => {
                        trace!(log, "Invalid termination tokens in table, failing");

                        return Err(parser.make_warn(ParseWarningKind::RuleFailed));
                    }

                    // Consume tokens like normal
                    _ => {
                        trace!(log, "Consuming cell contents as elements");

                        let new_elements = consume(log, parser)?
                            .chain(&mut exceptions, &mut _paragraph_break);

                        elements.extend(new_elements);
                    }
                }
            }

            // Build table cell
            cells.push(TableCell {
                elements,
                header,
                column_span,
                align,
                attributes: AttributeMap::new(),
            });
        }

        // Build table row
        rows.push(TableRow {
            cells,
            attributes: AttributeMap::new(),
        });
    }

    // Build table
    let table = Table {
        rows,
        attributes: AttributeMap::new(),
    };

    ok!(false; Element::Table(table), exceptions)
}

/// Parse out the cell settings from the start.
///
/// Cells have a few settings, such as alignment, and most importantly
/// here, their span, which is specified by having multiple
/// `Token::TableColumn` (`||`) adjacent together.
fn parse_cell_start(parser: &mut Parser) -> Result<TableCellStart, ParseWarning> {
    let mut span = 1;

    let (align, header) = loop {
        match parser.current().token {
            // Style cases, terminal
            Token::TableColumnTitle => break (None, true),
            Token::TableColumnLeft => break (Some(Alignment::Left), false),
            Token::TableColumnCenter => break (Some(Alignment::Center), false),
            Token::TableColumnRight => break (Some(Alignment::Right), false),

            // Regular column, iterate to see if it has a span
            Token::TableColumn => {
                span += 1;
                parser.step()?;
            }

            // Regular column, terminal
            _ if span > 1 => break (None, false),

            // No span depth, just an invalid token
            _ => return Err(parser.make_warn(ParseWarningKind::RuleFailed)),
        }
    };

    Ok(TableCellStart {
        align,
        header,
        column_span: NonZeroU32::new(span).unwrap(),
    })
}

fn next_two_tokens(parser: &Parser) -> (Token, Option<Token>) {
    let first = parser.current().token;
    let second = parser.look_ahead(0).map(|next| next.token);

    (first, second)
}
