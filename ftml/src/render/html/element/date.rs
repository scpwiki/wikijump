/*
 * render/html/element/date.rs
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
use crate::tree::Date;

pub fn render_date(
    log: &Logger,
    ctx: &mut HtmlContext,
    date: Date,
    date_format: Option<&str>,
    hover: bool,
) {
    // Get attribute values
    let timestamp = str!(date.timestamp());
    let delta = str!(date.time_since());
    let (space, hover_class) = if hover {
        (" ", "wj-date-hover")
    } else {
        ("", "")
    };

    // Format datetime
    let formatted_datetime = str!(date.format(date_format));

    // Build HTML elements
    ctx.html()
        .span()
        .attr(attr!(
            "is" => "wj-date",
            "class" => "wj-date" space hover_class,
            "data-format" => date_format.unwrap_or_else(|| date.default_format_string()),
            "data-iso" => &date.to_rfc3339(),
            "data-timestamp" => &timestamp,
            "data-delta" => &delta,
        ))
        .inner(log, formatted_datetime);
}
