/*
 * render/html/element/text.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

pub fn render_wikitext_raw(log: &Logger, ctx: &mut HtmlContext, text: &str) {
    info!(log, "Escaping raw string"; "text" => text);

    ctx.html()
        .span()
        .attr(attr!(
            "is" => "wj-raw",
            "class" => "wj-raw",
        ))
        .inner(log, text);
}

pub fn render_email(log: &Logger, ctx: &mut HtmlContext, email: &str) {
    info!(log, "Rendering email address"; "email" => email);

    // Since our usecase doesn't typically have emails as real,
    // but rather as fictional elements, we're just rendering as text.

    ctx.html()
        .span()
        .attr(attr!(
            "is" => "wj-email",
            "class" => "wj-email",
        ))
        .inner(log, email);
}

pub fn render_code(
    log: &Logger,
    ctx: &mut HtmlContext,
    language: Option<&str>,
    contents: &str,
) {
    info!(
        log,
        "Rendering code block";
        "language" => language.unwrap_or("<none>"),
        "contents" => contents,
    );

    let index = ctx.next_code_snippet_index();
    ctx.handle().post_code(log, index, contents);

    let class = {
        let mut class = format!("wj-code wj-language-{}", language.unwrap_or("none"));
        class.make_ascii_lowercase();
        class
    };

    ctx.html() //
        .div()
        .attr(attr!(
            "is" => "wj-code",
            "class" => &class,
        ))
        .contents(|ctx| {
            // Panel for holding additional features
            ctx.html()
                .div()
                .attr(attr!(
                    "class" => "wj-code-panel",
                ))
                .contents(|ctx| {
                    let button_title = ctx.handle().get_message(
                        log,
                        ctx.language(),
                        "button-copy-clipboard",
                    );

                    // Copy to clipboard button
                    ctx.html().button().attr(attr!(
                        "is" => "wj-code-copy",
                        "type" => "button",
                        "class" => "wj-code-copy",
                        "title" => button_title,
                    ));

                    // Span showing name of language
                    ctx.html()
                        .span()
                        .attr(attr!(
                            "class" => "wj-code-language",
                        ))
                        .inner(log, language.unwrap_or(""));
                });

            // Code block containing highlighted contents
            ctx.html().pre().contents(|ctx| {
                ctx.html().code().inner(log, contents);
            });
        });
}
