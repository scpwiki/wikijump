/*
 * tree/embed.rs
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

use super::clone::string_to_owned;
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "embed", content = "data")]
pub enum Embed<'t> {
    #[serde(rename_all = "kebab-case")]
    Youtube { video_id: Cow<'t, str> },

    #[serde(rename_all = "kebab-case")]
    Vimeo { video_id: Cow<'t, str> },

    GithubGist {
        username: Cow<'t, str>,
        hash: Cow<'t, str>,
    },

    #[serde(rename_all = "kebab-case")]
    GitlabSnippet { snippet_id: Cow<'t, str> },
}

impl Embed<'_> {
    pub fn name(&self) -> &'static str {
        match self {
            Embed::Youtube { .. } => "YouTube",
            Embed::Vimeo { .. } => "Vimeo",
            Embed::GithubGist { .. } => "GithubGist",
            Embed::GitlabSnippet { .. } => "GitlabSnippet",
        }
    }

    pub fn direct_url(&self) -> String {
        match self {
            Embed::Youtube { video_id } => format!("https://youtu.be/{video_id}"),
            Embed::Vimeo { video_id } => format!("https://vimeo.com/{video_id}"),
            Embed::GithubGist { username, hash } => {
                format!("https://gist.github.com/{username}/{hash}")
            }
            Embed::GitlabSnippet { snippet_id } => {
                format!("https://gitlab.com/-/snippets/{snippet_id}")
            }
        }
    }

    pub fn to_owned(&self) -> Embed<'static> {
        match self {
            Embed::Youtube { video_id } => Embed::Youtube {
                video_id: string_to_owned(video_id),
            },

            Embed::Vimeo { video_id } => Embed::Vimeo {
                video_id: string_to_owned(video_id),
            },

            Embed::GithubGist { username, hash } => Embed::GithubGist {
                username: string_to_owned(username),
                hash: string_to_owned(hash),
            },

            Embed::GitlabSnippet { snippet_id } => Embed::GitlabSnippet {
                snippet_id: string_to_owned(snippet_id),
            },
        }
    }
}
