/*
 * render/html/word.rs
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

use self::Word::*;
use super::prelude::*;

pub fn render_words<'a, I, W> (buffer: &mut String, words: I) -> Result<()>
where
    I: IntoIterator<Item = W>,
    W: AsRef<Word<'a>>,
{
    for word in words {
        render_word(buffer, word.as_ref())?;
    }

    Ok(())
}

pub fn render_word(buffer: &mut String, word: &Word) -> Result<()> {
    match word {
        &Anchor { name, ref arguments } => {
            buffer.push_str("<a");

            for (key, value) in arguments.iter() {
                write_tag_arg(buffer, key, Some(value))?;
            }

            buffer.push_str("></a>\n");
        },
        &Bold { ref words } => {
            buffer.push_str("<b>");
            render_words(buffer, words)?;
            buffer.push_str("</b>");
        },
        &Collapsible { show_top, show_bottom, ref lines } => {
            unimplemented!()
        },
        &Color { color, ref words } => {
            buffer.push_str("<span style=\"color: ");
            escape_attr(buffer, color)?;
            buffer.push_str("\">");
            render_words(buffer, words)?;
            buffer.push_str("</span>");
        },
        &Date { timestamp, format } => {
            unimplemented!()
        },
        &Email { contents } => {
            write!(buffer, "<a href=\"mailto:{}\">{}</a>", contents, contents)?;
        },
        &EquationReference { name } => {
            unimplemented!()
        },
        &File { filename } => {
            unimplemented!()
        },
        &Footnote { ref lines } => {
            unimplemented!()
        },
        &FootnoteBlock => {
            unimplemented!()
        },
        &Form { contents } => {
            unimplemented!()
        },
        &Gallery => {
            unimplemented!()
        },
        &Image { filename, float, direction, link, alt, ref title, width, height, style, class, size } => {
            buffer.push_str("<img");

            // TODO adjust for other sources
            write_tag_arg(buffer, "src", Some(filename))?;

            // TODO float

            if let Some(alt) = alt {
                write_tag_arg(buffer, "alt", Some(alt))?;
            }

            // TODO title

            if let Some(width) = width {
                write_tag_arg(buffer, "width", Some(width))?;
            }

            if let Some(height) = height {
                write_tag_arg(buffer, "height", Some(height))?;
            }

            if let Some(style) = style {
                write_tag_arg(buffer, "style", Some(style))?;
            }

            if let Some(class) = class {
                write_tag_arg(buffer, "class", Some(class))?;
            }

            if let Some(size) = size {
                write_tag_arg(buffer, "size", Some(size))?;
            }

            buffer.push_str("></img>");
        },

        _ => panic!("Word case not implemented yet!"),
    }

    Ok(())
}
