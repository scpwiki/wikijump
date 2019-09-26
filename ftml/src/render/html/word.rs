/*
 * render/html/word.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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
use super::module;
use super::prelude::*;
use crate::enums::{Alignment, LinkText, PageInfoField};
use arrayvec::ArrayVec;
use std::borrow::Cow;

// TODO: change these to be tenant-specific
const DEFAULT_HEADER: &str = "SCP Foundation";
const DEFAULT_SUBHEADER: &str = "Secure, Contain, Protect";

const MAILTO_EMAILS: bool = false;

impl<'a, 'w> ComponentRender for &'a [Word<'w>] {
    fn render(&self, ctx: &mut HtmlContext) -> Result<()> {
        for word in *self {
            word.render(ctx)?;
        }

        Ok(())
    }
}

impl<'a, 'w> ComponentRender for &'a Vec<Word<'w>> {
    #[inline]
    fn render(&self, ctx: &mut HtmlContext) -> Result<()> {
        self.as_slice().render(ctx)
    }
}

// TODO remove this lint
#[allow(unused_variables)]
impl<'w> ComponentRender for Word<'w> {
    fn render(&self, ctx: &mut HtmlContext) -> Result<()> {
        match self {
            &Anchor {
                ref href,
                ref name,
                ref id,
                ref class,
                ref style,
                ref target,
                ref words,
            } => {
                let mut html = ctx.html().a();

                if let Some(href) = href {
                    html.attr_fmt("href", |ctx| {
                        write!(ctx, "{}", percent_encode_url(href))?;
                        Ok(())
                    })?;
                }

                if let Some(name) = name {
                    html.attr("name", &[name]);
                }

                if let Some(id) = id {
                    html.attr("id", &[id]);
                }

                if let Some(class) = class {
                    html.attr("class", &[class]);
                }

                if let Some(style) = style {
                    html.attr("style", &[style]);
                }

                if let Some(target) = target {
                    html.attr_fmt("target", |ctx| {
                        write!(ctx, "{}", target)?;
                        Ok(())
                    })?;
                }

                html.inner(&words)?;
            }
            &Link { href, target, text } => {
                let text = match text {
                    LinkText::Article => &ctx.info().title,
                    LinkText::Text(text) => text,
                    LinkText::Url => href,
                };

                let mut html = ctx.html().a();

                html.attr_fmt("href", |ctx| {
                    write!(ctx, "{}", percent_encode_url(href))?;
                    Ok(())
                })?;

                if let Some(target) = target {
                    html.attr("target", &[target.style()]);
                }

                html.inner(&text)?;
            }
            &Bold { ref words } => {
                ctx.html().b().inner(&words)?;
            }
            &Color { color, ref words } => {
                ctx.html()
                    .span()
                    .attr("style", &["color: ", color])
                    .inner(&words)?;
            }
            &Css { style } => ctx.add_style(style),
            &Date { timestamp, format } => {
                use chrono::prelude::*;

                // For now timestamp can only be seconds since the epoch.
                // In the future we may want to extend this to also include ISO timestamps and such.

                let date = NaiveDateTime::from_timestamp(timestamp, 0);
                let format = format.unwrap_or("%e");
                let (format, hover) = if format.ends_with("|agohover") {
                    let new_len = format.len() - "|agohover".len();

                    (&format[..new_len], true)
                } else {
                    (format, false)
                };

                // TODO actually add the hover thing
                let _ = hover;

                write!(ctx, "{}", date.format(format))?;
            }
            &Email { address, text } => {
                if MAILTO_EMAILS {
                    ctx.html()
                        .a()
                        .attr("href", &["mailto:", address])
                        .inner(&text.unwrap_or(address))?;
                } else {
                    ctx.push_escaped(address);
                }
            }
            &EquationReference { name } => {
                // TODO
                return Err(Error::StaticMsg(
                    "Rendering for equation references are not implemented yet",
                ));
            }
            &File {
                filename,
                text,
                target,
            } => {
                let text = match text {
                    Some("") | None => filename,
                    Some(text) => text,
                };

                let mut html = ctx.html().a();
                html.attr_fmt("href", |ctx| {
                    write!(ctx, "{}", percent_encode_url(filename))?;
                    Ok(())
                })?;

                if let Some(target) = target {
                    html.attr("target", &[target.style()]);
                }

                html.inner(&text)?;
            }
            &Footnote { ref paragraphs } => {
                // TODO add javascript
                let number = ctx.footnotes_mut().incr();

                {
                    let mut html = ctx.html().sup();
                    html.attr("class", &["footnoteref"]);
                    html.contents(|ctx| {
                        let mut html = ctx.html().a();

                        html.attr("class", &["footnoteref"]);

                        html.attr_fmt("id", |ctx| {
                            write!(ctx, "footnote-{}", number)?;
                            Ok(())
                        })?;

                        html.attr_fmt("onclick", |ctx| {
                            write!(ctx, "scrollToFootnote('footnote-{}')", number)?;
                            Ok(())
                        })?;

                        html.contents(|ctx| {
                            write!(ctx, "{}", number)?;
                            Ok(())
                        })?;

                        Ok(())
                    })?;
                }

                ctx.write_footnote_block(|ctx| {
                    ctx.html().li().inner(&paragraphs)?;
                    Ok(())
                })?;
            }
            &FootnoteBlock => {
                // TODO
                ctx.footnotes_mut().set_block(true);
                ctx.push_raw_str("\0footnote-block\0");
            }
            &Form { contents } => {
                // TODO
                return Err(Error::StaticMsg(
                    "Rendering for forms is not implemented yet",
                ));
            }
            &Gallery => {
                // TODO
                return Err(Error::StaticMsg(
                    "Rendering for galleries is not implemented yet",
                ));
            }
            &Image {
                filename,
                float,
                direction,
                link,
                ref alt,
                ref title,
                ref width,
                ref height,
                ref style,
                ref class,
                ref size,
            } => {
                let mut html = ctx.html().div();
                let mut classes = ArrayVec::<[&str; 2]>::new();

                classes.push("image-container");

                if let Some(align) = direction {
                    match (align, float) {
                        (Alignment::Left, true) => classes.push("floatleft"),
                        (Alignment::Right, true) => classes.push("floatright"),
                        (Alignment::Left, false) => classes.push("alignleft"),
                        (Alignment::Right, false) => classes.push("alignright"),
                        (Alignment::Center, _) => classes.push("aligncenter"),
                        (Alignment::Justify, _) => panic!("Justify alignment in image"),
                    }
                }

                html.attr("class", classes.as_slice());
                html.contents(|ctx| {
                    fmt_image(ctx, filename, alt, width, height, style, class, size)
                })?;
            }
            &Italics { ref words } => {
                ctx.html().i().inner(&words)?;
            }
            &Math { expr } => {
                // TODO
                return Err(Error::StaticMsg(
                    "Rendering for inline mathematical expressions is not implemented yet",
                ));
            }
            &Module {
                name,
                ref arguments,
                contents,
            } => module::render(name, ctx, arguments, contents)?,
            &Monospace { ref words } => {
                ctx.html().tt().inner(&words)?;
            }
            &Note { ref paragraphs } => {
                ctx.html()
                    .div()
                    .attr("class", &["wiki-note"])
                    .inner(&paragraphs)?;
            }
            &PageInfo { field } => {
                let info = ctx.info();
                let text = match field {
                    PageInfoField::Title => info.title,
                    PageInfoField::AltTitle => info.alt_title.unwrap_or(info.title),
                    PageInfoField::Header => info.header.unwrap_or(DEFAULT_HEADER),
                    PageInfoField::SubHeader => info.subheader.unwrap_or(DEFAULT_SUBHEADER),
                };

                ctx.push_escaped(text);
            }
            &Raw { contents } => ctx.html().text(contents),
            &Size {
                size,
                ref paragraphs,
            } => {
                ctx.html()
                    .span()
                    .attr("style", &["size: ", size])
                    .inner(&paragraphs)?;
            }
            &Span {
                ref id,
                ref class,
                ref style,
                ref paragraphs,
            } => {
                let mut html = ctx.html().span();

                if let Some(id) = id {
                    html.attr("id", &[id]);
                }

                if let Some(class) = class {
                    html.attr("class", &[class]);
                }

                if let Some(style) = style {
                    html.attr("style", &[style]);
                }

                html.inner(&paragraphs)?;
            }
            &Strikethrough { ref words } => {
                ctx.html().strike().inner(&words)?;
            }
            &Subscript { ref words } => {
                ctx.html().sub().inner(&words)?;
            }
            &Superscript { ref words } => {
                ctx.html().sup().inner(&words)?;
            }
            &TabList { ref tabs } => {
                // TODO
                return Err(Error::StaticMsg(
                    "Rendering for tab lists is not implemented",
                ));
            }
            &Text { contents } => ctx.push_escaped(contents),
            &Underline { ref words } => {
                ctx.html().u().inner(&words)?;
            }
            &User {
                username,
                show_picture,
            } => {
                let handle = ctx.handle();
                let user = handle.get_user_by_name(username)?;

                match user {
                    Some(user) => fmt_user(ctx, &user, show_picture)?,
                    None => write!(ctx, "invalid username: {}", username)?,
                }
            }
        }

        Ok(())
    }
}

fn fmt_image(
    ctx: &mut HtmlContext,
    filename: &str,
    alt: &Option<Cow<str>>,
    width: &Option<Cow<str>>,
    height: &Option<Cow<str>>,
    style: &Option<Cow<str>>,
    class: &Option<Cow<str>>,
    size: &Option<Cow<str>>,
) -> Result<()> {
    // TODO adjust source for CDNs and other links
    ctx.html().a().attr("href", &[filename]).contents(|ctx| {
        let mut html = ctx.html().img();
        html.attr("src", &[filename]);

        if let Some(alt) = alt {
            let alt = alt.as_ref();
            html.attr("alt", &[alt]);
        }

        // TODO title

        if let Some(width) = width {
            let width = width.as_ref();
            html.attr("width", &[width]);
        }

        if let Some(height) = height {
            let height = height.as_ref();
            html.attr("height", &[height]);
        }

        if let Some(style) = style {
            let style = style.as_ref();
            html.attr("style", &[style]);
        }

        if let Some(class) = class {
            let class = class.as_ref();
            html.attr("class", &[class]);
        }

        if let Some(size) = size {
            return Err(Error::Msg(format!(
                "Size arguments in images are not supported (passed '{}')",
                size
            )));
        }

        Ok(())
    })?;

    Ok(())
}

fn fmt_user(ctx: &mut HtmlContext, user: &data::User, show_picture: bool) -> Result<()> {
    // TODO change HTML formatting
    let mut html = ctx.html().a();
    html.attr_fmt("href", |ctx| {
        write!(
            ctx,
            "http://www.wikidot.com/user:info/{}",
            percent_encode_url(&user.name)
        )?;
        Ok(())
    })?;

    if show_picture {
        html.contents(|ctx| {
            ctx.html()
                .img()
                .attr("class", &["small"])
                .attr_fmt("src", |ctx| {
                    write!(ctx, "https://example.com/avatars/{}", user.id)?;
                    Ok(())
                })?
                .attr_fmt("alt", |ctx| {
                    write!(ctx, "{}", percent_encode_url(&user.name))?;
                    Ok(())
                })?;

            Ok(())
        })?;
    }

    Ok(())
}
