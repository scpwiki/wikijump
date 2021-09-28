/*
 * render/html/embed.rs
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

use unicase::UniCase;

lazy_static! {
    pub static ref DEFAULT_EMBED_SOURCES: HashMap<UniCase<&'static str>, EmbedSource<'static>> = {
        hashmap! {
            UniCase::ascii("youtube") => EmbedSource {
                required: &["video"],
                optional: &[
                    ("width", "1280"),
                    ("height", "720"),
                    ("title", "YouTube video player"),
                ],
                template: r#"<iframe width="%%width%%" height="%%height%%" src="https://www.youtube.com/embed/%%video%%" title="%%title%%" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>"#,
            },
            UniCase::ascii("vimeo") => EmbedSource {
                required: &["video"],
                optional: &[
                    ("width", "640"),
                    ("height", "360"),
                ],
                template: r#"<iframe src="https://player.vimeo.com/video/%%video%%" width="%%width%%" height="%%height%%" frameborder="0" allow="autoplay; fullscreen; picture-in-picture" allowfullscreen></iframe>"#,
            },
        }
    };
}
