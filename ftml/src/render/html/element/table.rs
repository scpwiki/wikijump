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

pub fn render_table(log: &Logger, ctx: &mut HtmlContext, table: &Table) {
    debug!(log, "Rendering table");

    ctx.html()
        .table()
        .attr_map(&table.attributes)
        .contents(|ctx| {
            for row in table.rows {
                ctx.html() //
                    .tr()
                    .attr_map(&row.attributes)
                    .contents(|ctx| {
                        for cell in row.cells {
                            let elements: &[Element] = &cell.elements;

                            ctx.html()
                                .table_cell(cell.header)
                                .attr_map(&cell.attributes)
                                .inner(&log, &elements);
                        }
                    });
            }
        });
}
