/*
 * render/html/element/style.rs
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
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};

pub fn render_style(ctx: &mut HtmlContext, input_css: &str) {
    let minify = ctx.settings().minify_css;

    let parser_options = ParserOptions {
        error_recovery: true,
        ..Default::default()
    };

    let print_options = PrinterOptions {
        minify,
        ..Default::default()
    };

    debug!("Parsing input CSS ({} bytes)", input_css.len());
    let stylesheet = StyleSheet::parse(input_css, parser_options)
        .expect("Produced error with recovery enabled");

    trace!("Rendering CSS into HTML (minify: {minify})");
    let output_css = match stylesheet.to_css(print_options) {
        Ok(output) => output.code,
        Err(error) => {
            error!("Problem outputting CSS from stylesheet: {error}");
            trace!("Input CSS:\n{input_css}");
            trace!("Parsed stylesheet:\n{stylesheet:#?}");
            return;
        }
    };

    ctx.html().style().inner(|ctx| {
        // SAFETY: The resultant CSS cannot contain HTML-escaping elements,
        //         as those are invalid and would not be retained during
        //         the parcel_css parsing process.
        ctx.push_raw_str(&output_css);
    });
}
