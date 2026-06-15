/*
 * render/html/element/text.rs
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

pub fn render_wikitext_raw(ctx: &mut HtmlContext, text: &str) {
    debug!("Escaping raw string '{text}'");

    match ctx.layout() {
        Layout::Wikidot => {
            ctx.html()
                .span()
                .attr(attr!("style" => "white-space: pre-wrap;"))
                .contents(text);
        }
        Layout::Wikijump => {
            ctx.html()
                .span()
                .attr(attr!("class" => "wj-raw"))
                .contents(text);
        }
    }
}

pub fn render_email(ctx: &mut HtmlContext, email: &str) {
    debug!("Rendering email address '{email}'");

    match ctx.layout() {
        Layout::Wikidot => {
            ctx.html()
                .span()
                .attr(attr!(
                    "class" => "wiki-email",
                    "style" => "visibility: visible;",
                ))
                .inner(|ctx| {
                    ctx.html()
                        .a()
                        .attr(attr!("href" => "mailto:" email))
                        .contents(email);
                });
        }
        Layout::Wikijump => {
            // Since our usecase doesn't typically have emails as real,
            // but rather as fictional elements, we're just rendering as text.

            ctx.html()
                .span()
                .attr(attr!("class" => "wj-email"))
                .contents(email);
        }
    }
}

pub fn render_code(ctx: &mut HtmlContext, language: Option<&str>, contents: &str) {
    debug!(
        "Rendering code block (language {})",
        language.unwrap_or("<none>"),
    );
    let index = ctx.next_code_snippet_index();
    ctx.handle().post_code(index, contents);

    let class = {
        let mut class = format!("wj-code wj-language-{}", language.unwrap_or("none"));
        class.make_ascii_lowercase();
        class
    };

    ctx.html()
        .element("wj-code")
        .attr(attr!("class" => &class))
        .inner(|ctx| {
            // Panel for holding additional features
            ctx.html()
                .div()
                .attr(attr!(
                    "class" => "wj-code-panel",
                ))
                .inner(|ctx| {
                    let button_title = ctx
                        .handle()
                        .get_message(ctx.language(), "button-copy-clipboard");

                    // Copy to clipboard button
                    ctx.html()
                        .element("wj-code-copy")
                        .attr(attr!(
                            "type" => "button",
                            "class" => "wj-code-copy",
                            "title" => button_title,
                        ))
                        .inner(|ctx| {
                            ctx.html().sprite("wj-clipboard");
                            // Hidden normally, shown when clicked
                            ctx.html().sprite("wj-clipboard-success");
                        });

                    // Span showing name of language
                    ctx.html()
                        .span()
                        .attr(attr!(
                            "class" => "wj-code-language",
                        ))
                        .contents(language.unwrap_or(""));
                });

            // Code block containing highlighted contents
            ctx.html().pre().inner(|ctx| {
                ctx.html().code().contents(contents);
            });
        });
}
