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

use crate::enums::Alignment;
use self::Line::*;
use super::prelude::*;

pub fn render_lines<'a, I, L> (buffer: &mut String, lines: I) -> Result<()>
where
    I: IntoIterator<Item = L>,
    L: AsRef<Line<'a>>,
{
    for line in lines {
        render_line(buffer, line.as_ref())?;
        write!(buffer, " <br>")?;
    }

    Ok(())
}

pub fn render_line(buffer: &mut String, line: &Line) -> Result<()> {
    match line {
        &Align { alignment, ref lines } => {
            write!(buffer, r#"<div style="text-align: {};">\n"#, alignment)?;
            render_lines(buffer, lines)?;
            buffer.push_str("</div>");
        },
        &Center { ref words } => {
            buffer.push_str(r#"<div style="text-align: center;">\n"#);
            render_words(buffer, words)?;
            buffer.push_str("</div>");
        },
        &ClearFloat { direction } => {
            let style = match direction {
                Some(Alignment::Left) => "left",
                Some(Alignment::Right) => "right",
                Some(direction) => panic!("Invalid case for ClearFloat: {:?}", direction),
                None => "both",
            };

            write!(buffer, r#"<div style="clear: {};"></div>"#, style)?;
            // TODO verify this ^^^
            unimplemented!()
        },
        &CodeBlock { ref language, ref contents } => {
            // TODO add language highlighting
            let _ = language;

            buffer.push_str("<code>\n");
            escape_html(buffer, contents)?;
            buffer.push_str("</code>\n");
        },
        &Div { id, class, style, ref lines } => {
            buffer.push_str("<div");

            if let Some(id) = id {
                write_tag_arg(buffer, "id", Some(id))?;
            }

            if let Some(class) = class {
                write_tag_arg(buffer, "class", Some(class))?;
            }

            if let Some(style) = style {
                write_tag_arg(buffer, "style", Some(style))?;
            }

            buffer.push_str(">\n");
            render_lines(buffer, lines)?;
            buffer.push_str("\n</div>");
        },
        &Heading { level, ref words } => {
            write!(buffer, "<{}>", level)?;
            render_words(buffer, words)?;
            write!(buffer, "</{}>\n", level)?;
        },
        &HorizontalLine => buffer.push_str("<hr>\n"),
        &Html { contents } => buffer.push_str(contents),
        &Iframe { url, args } => {
            unimplemented!()
        },
        &IfTags { ref required, ref prohibited, ref lines } => {
            // Not sure what the approach on this should be
            unimplemented!()
        },
        &List { style, depth, ref items } => {
            // TODO will need to collect nearby entries for depth
            let _ = depth;

            write!(buffer, "<{}>\n", style)?;
            for item in items {
                buffer.push_str("<li> ");
                render_line(buffer, item)?;
                buffer.push_str(" </li>\n");
            }
            write!(buffer, "</{}>", style)?;
        },
        &Math { label, id, latex_env, expr } => {
            // TODO do LaTeX rendering
            unimplemented!()
        },
        &Table { ref rows } => {
            buffer.push_str("<table>\n");
            for row in rows {
                let (start_tag, end_tag) = match row.title {
                    true => ("<th> ", " </th>\n"),
                    false => ("<td> ", " </td>\n"),
                };

                buffer.push_str("<tr>\n");
                for column in &row.columns {
                    buffer.push_str(start_tag);
                    render_words(buffer, column)?;
                    buffer.push_str(end_tag);
                }
                buffer.push_str("</tr>\n");
            }
            buffer.push_str("</table>\n");
        },
        &TableOfContents { } => {
            // TODO
            unimplemented!()
        },
        QuoteBlock { id, class, style, ref lines } => {
            buffer.push_str("<blockquote");

            if let Some(id) = id {
                write_tag_arg(buffer, "id", Some(id))?;
            }

            if let Some(class) = class {
                write_tag_arg(buffer, "class", Some(class))?;
            }

            if let Some(style) = style {
                write_tag_arg(buffer, "style", Some(style))?;
            }

            buffer.push_str(">\n");
            render_lines(buffer, lines)?;
            buffer.push_str("\n</blockquote>");
        },
        Words { centered, ref words } => {
            if *centered {
                buffer.push_str(r#"<div style="text-align: center;"> "#);
            }

            render_words(buffer, words)?;

            if *centered {
                buffer.push_str(" </div>");
            }
        },
    }

    Ok(())
}
