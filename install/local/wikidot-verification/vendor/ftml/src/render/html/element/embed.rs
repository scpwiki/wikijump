/*
 * render/html/element/embed.rs
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
use crate::tree::Embed;

pub fn render_embed(ctx: &mut HtmlContext, embed: &Embed) {
    debug!(
        "Rendering embed (variant '{}', url '{}')",
        embed.name(),
        embed.direct_url(),
    );

    ctx.html()
        .div()
        .attr(attr!(
            "class" => "wj-embed",
        ))
        .inner(|ctx| match embed {
            Embed::Youtube { video_id } => {
                let url = format!("https://www.youtube.com/embed/{video_id}");

                ctx.html().iframe().attr(attr!(
                    "src" => &url,
                    "frameborder" => "0",
                    "allow" => "accelerometer; autoplay; "
                               "clipboard-write; encrypted-media; "
                               "gyroscope; picture-in-picture",
                    "allowfullscreen",
                ));
            }

            Embed::Vimeo { video_id } => {
                let url = format!("https://player.vimeo.com/video/{video_id}");

                ctx.html().iframe().attr(attr!(
                    "src" => &url,
                    "frameborder" => "0",
                    "allow" => "autoplay; fullscreen; picture-inpicture",
                    "allowfullscreen",
                ));
            }

            Embed::GithubGist { username, hash } => {
                let url = format!("https://gist.github.com/{username}/{hash}.js");

                ctx.html().script().attr(attr!("src" => &url));
            }

            Embed::GitlabSnippet { snippet_id } => {
                let url = format!("https://gitlab.com/-/snippets/{snippet_id}.js");

                ctx.html().script().attr(attr!("src" => &url));
            }
        });
}
