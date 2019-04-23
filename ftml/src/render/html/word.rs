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

pub fn render_words<'a, I, W>(buffer: &mut String, words: I) -> Result<()>
where
    I: IntoIterator<Item = W>,
    W: AsRef<Word<'a>>,
{
    for word in words {
        render_word(buffer, word.as_ref())?;
    }

    Ok(())
}

// TODO remove this
#[allow(unused_variables)]
pub fn render_word(buffer: &mut String, word: &Word) -> Result<()> {
    match word {
        &Anchor {
            name,
            ref arguments,
        } => {
            buffer.push_str("<a");

            for (key, value) in arguments.iter() {
                write_tag_arg(buffer, key, value)?;
            }

            buffer.push_str("></a>\n");
        }
        &Bold { ref words } => {
            buffer.push_str("<b>");
            render_words(buffer, words)?;
            buffer.push_str("</b>");
        }
        &Button {} => unimplemented!(),
        &Collapsible {
            show_top,
            show_bottom,
            ref lines,
        } => unimplemented!(),
        &Color { color, ref words } => {
            buffer.push_str("<span style=\"color: ");
            escape_attr(buffer, color)?;
            buffer.push_str("\">");
            render_words(buffer, words)?;
            buffer.push_str("</span>");
        }
        &Date { timestamp, format } => unimplemented!(),
        &Email { contents } => {
            write!(buffer, "<a href=\"mailto:{}\">{}</a>", contents, contents)?;
        }
        &EquationReference { name } => unimplemented!(),
        &File { filename } => unimplemented!(),
        &Footnote { ref lines } => unimplemented!(),
        &FootnoteBlock => unimplemented!(),
        &Form { contents } => unimplemented!(),
        &Gallery => unimplemented!(),
        &Image {
            filename,
            float,
            direction,
            link,
            alt,
            title,
            width,
            height,
            style,
            class,
            size,
        } => {
            buffer.push_str("<img");

            // TODO adjust for other sources
            write_tag_arg(buffer, "src", filename)?;

            // TODO float

            if let Some(alt) = alt {
                write!(buffer, " alt={}", alt)?;
            }

            // TODO title

            if let Some(width) = width {
                write!(buffer, " width={}", width)?;
            }

            if let Some(height) = height {
                write!(buffer, " height={}", height)?;
            }

            if let Some(style) = style {
                write!(buffer, " style={}", style)?;
            }

            if let Some(class) = class {
                write!(buffer, " class={}", class)?;
            }

            if let Some(size) = size {
                write!(buffer, " size={}", size)?;
            }

            buffer.push_str("></img>");
        }
        &Italics { ref words } => {
            buffer.push_str("<i>");
            render_words(buffer, words)?;
            buffer.push_str("</i>");
        }
        &Link { page, anchor, text } => {
            buffer.push_str("<a");
            // TODO adjust for other sources
            write_tag_arg(buffer, "href", page)?;

            if let Some(anchor) = anchor {
                write_tag_arg(buffer, "name", anchor)?;
            }

            buffer.push('>');
            escape_html(buffer, text.unwrap_or(page))?;
            buffer.push_str("</a>");
        }
        &Math { expr } => unimplemented!(),
        &Module {
            name,
            ref arguments,
            contents,
        } => unimplemented!(),
        &Monospace { ref words } => {
            buffer.push_str("<tt>");
            render_words(buffer, words)?;
            buffer.push_str("</tt>");
        }
        &Note { ref lines } => unimplemented!(),
        &Raw { contents } => escape_html(buffer, contents)?,
        &Size { size, ref lines } => {
            write!(buffer, "<span style=\"size: {};\">", size)?;
            render_lines(buffer, lines)?;
            buffer.push_str("</span>");
        }
        &Span {
            id,
            class,
            style,
            ref lines,
        } => {
            buffer.push_str("<span");

            if let Some(id) = id {
                write!(buffer, " id={}", id)?;
            }

            if let Some(class) = class {
                write!(buffer, " class={}", class)?;
            }

            if let Some(style) = style {
                write!(buffer, " style={}", style)?;
            }

            buffer.push('>');
            render_lines(buffer, lines)?;
            buffer.push_str("</span>");
        }
        &Strikethrough { ref words } => {
            buffer.push_str("<strike>");
            render_words(buffer, words)?;
            buffer.push_str("</strike>");
        }
        &Subscript { ref words } => {
            buffer.push_str("<sub>");
            render_words(buffer, words)?;
            buffer.push_str("</sub>");
        }
        &Superscript { ref words } => {
            buffer.push_str("<sup>");
            render_words(buffer, words)?;
            buffer.push_str("</sup>");
        }
        &TabList { ref tabs } => unimplemented!(),
        &Text { contents } => escape_html(buffer, contents)?,
        &Underline { ref words } => {
            buffer.push_str("<u>");
            render_words(buffer, words)?;
            buffer.push_str("</u>");
        }
        &User {
            username,
            show_picture,
        } => unimplemented!(),
    }

    Ok(())
}
