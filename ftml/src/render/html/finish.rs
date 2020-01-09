/*
 * render/html/finish.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019-2020 Ammon Smith
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
    ctx.write_footnote_block(|ctx| {
        ctx.buffer().insert_str(
            0,
            stringify!(
                "<div class=\"footnotes-footer\">",
                "<ul type=\"1\" class=\"footnotes-footer\">",
            ),
        );
        ctx.push_raw_str("</ul></div");

        Ok(())
    })?;

    // If a footnote block hasn't been placed yet, add one.
    if ctx.footnotes().needs_block() {
        Word::FootnoteBlock.render(ctx)?;
    }

    // Replace footnote block placeholders
    ctx.substitute_footnote_block();

    // Add final newline
    ctx.push_raw('\n');

    Ok(())
}
