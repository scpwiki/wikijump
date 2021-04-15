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

use super::super::escape::escape_char;
use super::prelude::*;

pub fn render_wikitext_raw(log: &slog::Logger, ctx: &mut HtmlContext, text: &str) {
    debug!(log, "Escaping raw string"; "text" => text);

    for ch in text.chars() {
        match (ch, escape_char(ch)) {
            // Turn spaces into non-breaking spaces
            (' ', _) => ctx.push_raw_str("&nbsp;"),

            // Escape the character
            (_, Some(escaped)) => ctx.push_raw_str(escaped),

            // Character doesn't need escaping
            (_, None) => ctx.push_raw(ch),
        }
    }
}

pub fn render_email(log: &slog::Logger, ctx: &mut HtmlContext, email: &str) {
    debug!(log, "Rendering email address"; "email" => email);

    // Since our usecase doesn't typically have emails as real,
    // but rather as fictional elements, we're just rendering as text.

    ctx.html()
        .span()
        .attr("class", &["email"])
        .attr("style", &["word-break: keep-all;"])
        .inner(log, &email);
}

pub fn render_code(
    log: &slog::Logger,
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

    // TODO: syntax highlighting based on 'language'

    ctx.html()
        .div()
        .attr("class", &["code"])
        .inner(log, &contents);
}
