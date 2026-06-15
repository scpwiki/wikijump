/*
 * render/html/element/clear_float.rs
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
use crate::tree::ClearFloat;

pub fn render_clear_float(ctx: &mut HtmlContext, clear_float: ClearFloat) {
    let attributes = match ctx.layout() {
        Layout::Wikidot => attr!("style" => clear_float.wd_html_style()),
        Layout::Wikijump => {
            attr!("class" => "wj-clear-float " clear_float.wj_html_class())
        }
    };
    ctx.html().div().attr(attributes);
}
