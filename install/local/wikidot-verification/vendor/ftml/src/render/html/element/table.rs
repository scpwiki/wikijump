/*
 * render/html/element/table.rs
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
use crate::tree::{Table, TableType};
use std::num::NonZeroU32;

pub fn render_table(ctx: &mut HtmlContext, table: &Table) {
    debug!("Rendering table");

    let mut column_span_buf = String::new();
    let value_one = NonZeroU32::new(1).unwrap();
    let layout = ctx.layout();

    let table_class = match (layout, table.table_type) {
        (Layout::Wikidot, TableType::Simple) => "wiki-content-table",
        (Layout::Wikidot, TableType::Advanced) => "",
        (Layout::Wikijump, TableType::Simple) => "wj-table wj-table-simple",
        (Layout::Wikijump, TableType::Advanced) => "wj-table wj-table-advanced",
    };

    let table_attributes = if table_class.is_empty() {
        attr!(;; &table.attributes)
    } else {
        attr!(
            "class" => table_class;;
            &table.attributes,
        )
    };

    // Full table
    ctx.html().table().attr(table_attributes).inner(|ctx| {
        ctx.html().tbody().inner(|ctx| {
            // Each row
            for row in &table.rows {
                ctx.html()
                    .tr()
                    .attr(attr!(;; &row.attributes))
                    .inner(|ctx| {
                        // Each cell in a row
                        for cell in &row.cells {
                            let elements: &[Element] = &cell.elements;

                            if cell.column_span > value_one {
                                // SAFETY: The NonZeroU32 type has no possible values which
                                //         can lead to an XSS when converted directly to a
                                //         string.
                                //
                                //         Also, reusable buffer cleared before each use.
                                column_span_buf.clear();
                                str_write!(column_span_buf, "{}", cell.column_span);
                            }

                            let attributes = match (cell.align, layout) {
                                (Some(align), Layout::Wikidot) => attr!(
                                    // Add column span if not default (1)
                                    "colspan" => &column_span_buf;
                                        if cell.column_span > value_one,

                                    // Add alignment if specified
                                    "style" => align.wd_html_style();;

                                    // Add remaining attributes
                                    &cell.attributes,
                                ),

                                (Some(align), Layout::Wikijump) => attr!(
                                    "colspan" => &column_span_buf;
                                        if cell.column_span > value_one,
                                    "class" => align.wj_html_class();;
                                    &cell.attributes,
                                ),

                                (None, _) => attr!(
                                    "colspan" => &column_span_buf;
                                        if cell.column_span > value_one;;
                                    &cell.attributes,
                                ),
                            };

                            ctx.html()
                                .table_cell(cell.header)
                                .attr(attributes)
                                .contents(elements);
                        }
                    });
            }
        });
    });
}
