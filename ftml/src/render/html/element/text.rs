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
    debug!(log, "Escaping raw string"; "text" => text);

    ctx.html()
        .span()
        .attr("class", &["raw"])
        .attr("style", &["white-space: pre-wrap;"]) // TODO add this to the "raw" class
        .inner(log, &text);
}

pub fn render_email(log: &Logger, ctx: &mut HtmlContext, email: &str) {
    debug!(log, "Rendering email address"; "email" => email);

    // Since our usecase doesn't typically have emails as real,
    // but rather as fictional elements, we're just rendering as text.

    ctx.html()
        .span()
        .attr("class", &["email"])
        .attr("style", &["word-break: keep-all;"]) // TODO add this to the "email" class
        .inner(log, &email);
}

pub fn render_code(
    log: &Logger,
    ctx: &mut HtmlContext,
    language: Option<&str>,
    contents: &str,
) {
    debug!(
        log,
        "Rendering code block";
        "language" => language.unwrap_or("<none>"),
        "contents" => contents,
    );

    let index = ctx.next_code_snippet_index();
    ctx.handle().post_code(log, index, contents);

    let class = {
        let mut class = format!("code language-{}", language.unwrap_or("none"));
        class.make_ascii_lowercase();
        class
    };

    // TODO: syntax highlighting based on 'language'

    ctx.html() //
        .pre()
        .attr("class", &[&class])
        .contents(|ctx| {
            ctx.html().code().inner(log, &contents);
        });
}
