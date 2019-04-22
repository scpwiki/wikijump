/*
 * render/html/line.rs
 *
 * wikidot-html - Convert Wikidot code to HTML
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

use self::Line::*;
use super::prelude::*;

pub fn render_line(buffer: &mut String, line: &Line) -> fmt::Result {
    match line {
        &Align { alignment, ref lines } => {
            write!(buffer, r#"<div style="text-align: {};">\n"#, alignment.style())?;
            render_lines(buffer, lines)?;
            buffer.push_str("</div>");
        },
        &Center { ref words } => {
            buffer.push_str(r#"<div style="text-align: center;">\n"#);
            render_words(buffer, words)?;
            buffer.push_str("</div>");
        },

        _ => panic!("Render rule for {:?} not implemented yet", line),
    }

    Ok(())
}

pub fn render_lines<'a, I: IntoIterator<Item = &'a Line<'a>>>(buffer: &mut String, lines: I) -> fmt::Result {
    for line in lines {
        render_line(buffer, line)?;
        write!(buffer, " <br>")?;
    }

    Ok(())
}
