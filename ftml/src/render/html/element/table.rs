/*
 * render/html/element/table.rs
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
use crate::tree::Table;
use std::num::NonZeroU32;

pub fn render_table(log: &Logger, ctx: &mut HtmlContext, table: &Table) {
    debug!(log, "Rendering table");

    let mut column_span_buf = String::new();
    let value_one = NonZeroU32::new(1).unwrap();

    // Full table
    ctx.html()
        .table()
        .attr(attr!(;; &table.attributes))
        .contents(|ctx| {
            ctx.html()
                .tbody()
                .contents(|ctx| {
                    // Each row
                    for row in &table.rows {
                        ctx.html() //
                            .tr()
                            .attr(attr!(;; &row.attributes))
                            .contents(|ctx| {
                                // Each cell in a row
                                for cell in &row.cells {
                                    let elements: &[Element] = &cell.elements;
                                    let align_class = match cell.align {
                                        Some(align) => align.html_class(),
                                        None => "",
                                    };

                                    if cell.column_span > value_one {
                                        column_span_buf.clear();
                                        str_write!(&mut column_span_buf, "{}", cell.column_span);
                                    }

                                    ctx.html()
                                        .table_cell(cell.header)
                                        .attr(attr!(
                                            // Add column span if not default (1)
                                            "colspan" => &column_span_buf;
                                                if cell.column_span > value_one,

                                            // Add alignment if specified
                                            "class" => align_class;
                                                if cell.align.is_some();;

                                            &cell.attributes,
                                        ))
                                        .inner(log, &elements);
                                }
                            });
                    }
                });
        });
}
