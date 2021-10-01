/*
 * render/html/element/embed.rs
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
use crate::tree::Embed;

pub fn render_embed(log: &Logger, ctx: &mut HtmlContext, embed: &Embed) {
    info!(
        log,
        "Rendering embed";
        "variant" => embed.name(),
        "url" => embed.direct_url(),
    );

    ctx.html()
        .div()
        .attr(attr!(
            "is" => "wj-embed",
            "class" => "wj-embed",
        ))
        .contents(|ctx| match embed {
            Embed::Youtube {
                video_id,
                width,
                height,
            } => {
                let url = format!("https://www.youtube.com/embed/{}", video_id);
                let width = str!(width.unwrap_or(1280));
                let height = str!(height.unwrap_or(720));

                ctx.html().iframe().attr(attr!(
                    "src" => &url,
                    "width" => &width,
                    "height" => &height,
                    "frameborder" => "0",
                    "allow" => "accelerometer; autoplay; "
                               "clipboard-write; encrypted-media; "
                               "gyroscope; picture-in-picture",
                    "allowfullscreen",
                ));
            }

            Embed::Vimeo {
                video_id,
                width,
                height,
            } => {
                let url = format!("https://player.vimeo.com/video/{}", video_id);
                let width = str!(width.unwrap_or(640));
                let height = str!(height.unwrap_or(360));

                ctx.html().iframe().attr(attr!(
                    "src" => &url,
                    "width" => &width,
                    "height" => &height,
                    "frameborder" => "0",
                    "allow" => "autoplay; fullscreen; picture-inpicture",
                    "allowfullscreen",
                ));
            }

            Embed::GithubGist { username, hash } => {
                let url = format!("https://gist.github.com/{}/{}.js", username, hash);

                ctx.html().script().attr(attr!("src" => &url));
            }

            Embed::GitlabSnippet { snippet_id } => {
                let url = format!("https://gitlab.com/-/snippets/{}.js", snippet_id);

                ctx.html().script().attr(attr!("src" => &url));
            }
        });
}
