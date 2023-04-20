/*
 * render/html/element/date.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2023 Wikijump Team
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
use crate::tree::DateItem;

pub fn render_date(
    ctx: &mut HtmlContext,
    date: DateItem,
    date_format: Option<&str>,
    hover: bool,
) {
    // TEMP
    if date_format.is_some() {
        warn!("Time format passed, feature currently not supported!");
    }

    // Get attribute values
    let timestamp = str!(date.timestamp());
    let delta = str!(date.time_since());
    let (space, hover_class) = if hover {
        (" ", "wj-date-hover")
    } else {
        ("", "")
    };

    // Format datetime
    // TODO handle error
    let formatted_datetime = match date.format() {
        Ok(datetime) => datetime,
        Err(error) => {
            error!("Error formatting date into string: {error}");
            str!("<ERROR>")
        }
    };

    // Build HTML elements
    ctx.html()
        .span()
        .attr(attr!(
            "class" => "wj-date" space hover_class,
            "data-timestamp" => &timestamp,
            "data-delta" => &delta,
        ))
        .contents(formatted_datetime);
}
