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
use chrono::{DateTime, FixedOffset};

pub fn render_date(
    log: &Logger,
    ctx: &mut HtmlContext,
    datetime: DateTime<FixedOffset>,
    date_format: Option<&str>,
    hover: bool,
) {
    let date_format = date_format.unwrap_or(DEFAULT_DATETIME_FORMAT);

    todo!()
}
