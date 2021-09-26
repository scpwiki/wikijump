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
use crate::render::DEFAULT_DATETIME_FORMAT;
use chrono::{DateTime, FixedOffset, Utc};

pub fn render_date(
    log: &Logger,
    ctx: &mut HtmlContext,
    datetime: DateTime<FixedOffset>,
    date_format: Option<&str>,
    hover: bool,
) {
    // Get date format to use
    let date_format = date_format.unwrap_or(DEFAULT_DATETIME_FORMAT);

    // Get time since / until the given datetime
    let delta_seconds = datetime.timestamp() - Utc::now().timestamp();

    // Get attribute values
    let timestamp = &str!(datetime.timestamp());
    let delta = &str!(delta_seconds);
    let (space, hover_class) = if hover {
        (" ", "wj-date-hover")
    } else {
        ("", "")
    };

    // Format datetime
    let formatted_datetime = str!(datetime.format(date_format));

    // Build HTML elements
    ctx.html()
        .span()
        .attr(attr!(
            "is" => "wj-date",
            "class" => "wj-date" space hover_class,
            "data-format" => date_format,
            "data-timestamp" => timestamp,
            "data-delta" => delta,
        ))
        .inner(log, formatted_datetime);
}
