/*
 * render/html/finish.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith for Project Foundation
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

/// Function for any rendering which takes place after the rest
/// of the document has been completed.
pub fn render_finish(ctx: &mut HtmlContext) -> Result<()> {
    // Finish footnote block text
    if ctx.footnotes().has_footnotes() {
        ctx.write_footnote_block(|ctx| {
            ctx.insert_str(
                0,
                stringify!(
                    "<div class=\"footnotes-footer\">",
                    "<ul type=\"1\" class=\"footnotes-footer\">",
                ),
            );
            ctx.push_str("</ul></div");

            Ok(())
        })?;
    }

    // Replace footnote block placeholders
    ctx.substitute_footnote_block();

    // If a footnote block hasn't been placed yet, add one.
    if ctx.footnotes().needs_render() {
        render_word(ctx, &Word::FootnoteBlock)?;
    }

    Ok(())
}
