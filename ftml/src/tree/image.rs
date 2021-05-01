/*
 * tree/image.rs
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

#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ImageSource<'a> {
    /// Image is sourced from an arbitrary URL.
    Url(Cow<'a, str>),

    /// Image is attached the current page.
    File(Cow<'a, str>),

    /// Image is attached to another page on the site.
    OtherFile {
        page: Cow<'a, str>,
        file: Cow<'a, str>,
    },

    /// Image is attached to another page on another site.
    RemoteFile {
        site: Cow<'a, str>,
        page: Cow<'a, str>,
        file: Cow<'a, str>,
    },
}

impl ImageSource<'_> {
    pub fn to_owned(&self) -> ImageSource<'static> {
        match self {
            ImageSource::Url(url) => ImageSource::Url(string_to_owned(url)),
            ImageSource::File(file) => ImageSource::File(string_to_owned(file)),
            ImageSource::OtherFile { page, file } => ImageSource::OtherFile {
                page: string_to_owned(page),
                file: string_to_owned(file),
            },
            ImageSource::RemoteFile { site, page, file } => ImageSource::RemoteFile {
                site: string_to_owned(site),
                page: string_to_owned(page),
                file: string_to_owned(file),
            },
        }
    }
}
