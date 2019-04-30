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

pub fn render_lines<'a, I, L>(output: &mut HtmlOutput, lines: I) -> Result<()>
where
    L: AsRef<Line<'a>>,
    I: IntoIterator<Item = L>,
    I::IntoIter: ExactSizeIterator,
{
    let lines = lines.into_iter();
    let len = lines.len();

    for (i, line) in lines.enumerate() {
        render_line(output, line.as_ref())?;

        if i < len - 1 {
            write!(output.html, " <br>")?;
        }
    }

    Ok(())
}

// TODO remove this
#[allow(unused_variables)]
pub fn render_line(output: &mut HtmlOutput, line: &Line) -> Result<()> {
    match line {
        &Align {
            alignment,
            ref lines,
        } => {
            write!(output.html, r#"<div style="text-align: {};">\n"#, alignment)?;
            render_lines(output, lines)?;
            output.push_str("</div>");
        }
        &Center { ref words } => {
            output.push_str(r#"<div style="text-align: center;">\n"#);
            render_words(output, words)?;
            output.push_str("</div>");
        }
        &ClearFloat { direction } => {
            let style = match direction {
                Some(Alignment::Left) => "left",
                Some(Alignment::Right) => "right",
                Some(direction) => panic!("Invalid case for ClearFloat: {:?}", direction),
                None => "both",
            };

            write!(output.html, r#"<div style="clear: {}; height: 0;"></div>"#, style)?;
        }
        &CodeBlock {
            ref language,
            ref contents,
        } => {
            // TODO add language highlighting
            let _ = language;

            output.push_str("<code>\n");
            escape_html(output, contents)?;
            output.push_str("</code>\n");
        }
        &Div {
            id,
            class,
            style,
            ref lines,
        } => {
            output.push_str("<div");

            if let Some(id) = id {
                write!(output.html, " id={}", id)?;
            }

            if let Some(class) = class {
                write!(output.html, " class={}", class)?;
            }

            if let Some(style) = style {
                write!(output.html, " style={}", style)?;
            }

            output.push_str(">\n");
            render_lines(output, lines)?;
            output.push_str("\n</div>");
        }
        &Heading { level, ref words } => {
            write!(output.html, "<{}>", level)?;
            render_words(output, words)?;
            write!(output.html, "</{}>\n", level)?;
        }
        &HorizontalLine => output.push_str("<hr>\n"),
        &Html { contents } => output.push_str(contents),
        &Iframe { url, args } => unimplemented!(),
        &IfTags {
            ref required,
            ref prohibited,
            ref lines,
        } => {
            // Not sure what the approach on this should be
            unimplemented!()
        }
        &Javascript { contents } => {
            write!(output.html, "<script>\n{}\n</script>", contents)?;
        }
        &List {
            style,
            depth,
            ref items,
        } => {
            // TODO will need to collect nearby entries for depth
            let _ = depth;

            write!(output.html, "<{}>\n", style)?;
            for item in items {
                output.push_str("<li> ");
                render_line(output, item)?;
                output.push_str(" </li>\n");
            }
            write!(output.html, "</{}>", style)?;
        }
        &Math {
            label,
            id,
            latex_env,
            expr,
        } => {
            // TODO do LaTeX rendering
            // use mathjax library
            unimplemented!()
        }
        &Newlines { count } => {
            for _ in 0..count {
                output.push_str("<br>");
            }

            output.push('\n');
        }
        &Table { ref rows } => {
            output.push_str("<table>\n");
            for row in rows {
                let (start_tag, end_tag) = match row.title {
                    true => ("<th>", "</th>\n"),
                    false => ("<td>", "</td>\n"),
                };

                output.push_str("<tr>\n");
                for column in &row.columns {
                    output.push_str(start_tag);
                    render_words(output, column)?;
                    output.push_str(end_tag);
                }
                output.push_str("</tr>\n");
            }
            output.push_str("</table>\n");
        }
        &TableOfContents {} => {
            // TODO
            unimplemented!()
        }
        QuoteBlock {
            id,
            class,
            style,
            ref lines,
        } => {
            output.push_str("<blockquote");

            if let Some(id) = id {
                write!(output.html, " id={}", id)?;
            }

            if let Some(class) = class {
                write!(output.html, " class={}", class)?;
            }

            if let Some(style) = style {
                write!(output.html, " style={}", style)?;
            }

            output.push_str(">\n");
            render_lines(output, lines)?;
            output.push_str("\n</blockquote>");
        }
        Words {
            centered,
            ref words,
        } => {
            if *centered {
                output.push_str(r#"<div style="text-align: center;"> "#);
            }

            render_words(output, words)?;

            if *centered {
                output.push_str(" </div>");
            }
        }
    }

    Ok(())
}
