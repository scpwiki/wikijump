/*
 * render/html/word.rs
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

use percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};
use self::Word::*;
use super::prelude::*;

macro_rules! percent_encode {
    ($input:expr) => ( percent_encode($input.as_ref(), DEFAULT_ENCODE_SET) )
}

pub fn render_words<'a, I, W>(output: &mut HtmlOutput, words: I) -> Result<()>
where
    I: IntoIterator<Item = W>,
    W: AsRef<Word<'a>>,
{
    for word in words {
        render_word(output, word.as_ref())?;
    }

    Ok(())
}

// TODO remove this
#[allow(unused_variables)]
pub fn render_word(output: &mut HtmlOutput, word: &Word) -> Result<()> {
    match word {
        &Anchor {
            href,
            name,
            id,
            class,
            style,
            target,
            ref words,
        } => {
            output.push_str("<a");

            if let Some(href) = href {
                write!(output.html, " href=\"{}\"", percent_encode!(href))?;
            }

            if let Some(name) = name {
                write!(output.html, " name=\"{}\"", name)?;
            }

            if let Some(id) = id {
                write!(output.html, " id=\"{}\"", id)?;
            }

            if let Some(class) = class {
                write!(output.html, " class=\"{}\"", class)?;
            }

            if let Some(style) = style {
                write!(output.html, " style=\"{}\"", style)?;
            }

            if let Some(target) = target {
                write!(output.html, " target=\"{}\"", target)?;
            }

            output.push('>');
            render_words(output, words)?;
            output.push_str("</a>");
        }
        &Link { href, target, text } => {
            write!(output.html, "<a href=\"{}\"", percent_encode!(href))?;

            if let Some(target) = target {
                write!(output.html, " target=\"{}\"", target)?;
            }

            // TODO fetch title of the page
            let text = match text {
                Some("") => "<page title here>",
                Some(text) => text,
                None => href,
            };

            output.push('>');
            escape_html(output, text)?;
            output.push_str("</a>");
        }
        &Bold { ref words } => {
            output.push_str("<b>");
            render_words(output, words)?;
            output.push_str("</b>");
        }
        &Color { color, ref words } => {
            output.push_str("<span style=\"color: ");
            escape_attr(output, color)?;
            output.push_str("\">");
            render_words(output, words)?;
            output.push_str("</span>");
        }
        &Date { timestamp, format } => unimplemented!(),
        &Email { address, text } => {
            write!(output.html, "<a href=\"mailto:{}\">", address)?;
            escape_html(output, text.unwrap_or(address))?;
            output.push_str("</a>");
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
            output.push_str("<div class=\"image-container\"");
            if let Some(align) = direction {
                write!(output.html, " style=\"text-align: {};\"", align)?;
            }
            output.push_str("><img");

            // TODO adjust for other sources
            write_tag_arg(output, "src", filename)?;

            // TODO float

            if let Some(alt) = alt {
                write!(output.html, " alt={}", alt)?;
            }

            // TODO title

            if let Some(width) = width {
                write!(output.html, " width={}", width)?;
            }

            if let Some(height) = height {
                write!(output.html, " height={}", height)?;
            }

            if let Some(style) = style {
                write!(output.html, " style={}", style)?;
            }

            if let Some(class) = class {
                write!(output.html, " class={}", class)?;
            }

            if let Some(size) = size {
                write!(output.html, " size={}", size)?;
            }

            output.push_str("></img></div>");
        }
        &Italics { ref words } => {
            output.push_str("<i>");
            render_words(output, words)?;
            output.push_str("</i>");
        }
        &Math { expr } => unimplemented!(),
        &Module {
            name,
            ref arguments,
            contents,
        } => unimplemented!(), // TODO switch to ctx vs output and add module listing
        &Monospace { ref words } => {
            output.push_str("<tt>");
            render_words(output, words)?;
            output.push_str("</tt>");
        }
        &Note { ref lines } => unimplemented!(),
        &Raw { contents } => escape_html(output, contents)?,
        &Size { size, ref lines } => {
            write!(output.html, "<span style=\"size: {};\">", size)?;
            render_lines(output, lines)?;
            output.push_str("</span>");
        }
        &Span {
            id,
            class,
            style,
            ref lines,
        } => {
            output.push_str("<span");

            if let Some(id) = id {
                write!(output.html, " id={}", id)?;
            }

            if let Some(class) = class {
                write!(output.html, " class={}", class)?;
            }

            if let Some(style) = style {
                write!(output.html, " style={}", style)?;
            }

            output.push('>');
            render_lines(output, lines)?;
            output.push_str("</span>");
        }
        &Strikethrough { ref words } => {
            output.push_str("<strike>");
            render_words(output, words)?;
            output.push_str("</strike>");
        }
        &Subscript { ref words } => {
            output.push_str("<sub>");
            render_words(output, words)?;
            output.push_str("</sub>");
        }
        &Superscript { ref words } => {
            output.push_str("<sup>");
            render_words(output, words)?;
            output.push_str("</sup>");
        }
        &TabList { ref tabs } => unimplemented!(),
        &Text { contents } => escape_html(output, contents)?,
        &Underline { ref words } => {
            output.push_str("<u>");
            render_words(output, words)?;
            output.push_str("</u>");
        }
        &User {
            username,
            show_picture,
        } => unimplemented!(),
    }

    Ok(())
}
