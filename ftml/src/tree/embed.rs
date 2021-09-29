/*
 * tree/embed.rs
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

use super::clone::string_to_owned;
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "type", content = "data")]
pub enum Embed<'t> {
    #[serde(rename_all = "kebab-case")]
    YouTube {
        video_id: Cow<'t, str>,
        width: Option<u32>,
        height: Option<u32>,
    },

    #[serde(rename_all = "kebab-case")]
    Vimeo {
        video_id: Cow<'t, str>,
        width: Option<u32>,
        height: Option<u32>,
    },

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
            Embed::YouTube { .. } => "YouTube",
            Embed::Vimeo { .. } => "Vimeo",
            Embed::GithubGist { .. } => "GithubGist",
            Embed::GitlabSnippet { .. } => "GitlabSnippet",
        }
    }

    pub fn to_owned(&self) -> Embed<'static> {
        match self {
            Embed::YouTube {
                video_id,
                width,
                height,
            } => Embed::YouTube {
                video_id: string_to_owned(video_id),
                width: *width,
                height: *height,
            },

            Embed::Vimeo {
                video_id,
                width,
                height,
            } => Embed::Vimeo {
                video_id: string_to_owned(video_id),
                width: *width,
                height: *height,
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
