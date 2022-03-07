/*
 * tree/image.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2022 Wikijump Team
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
use crate::url::is_url;
use std::borrow::Cow;
use strum_macros::IntoStaticStr;

#[derive(Serialize, Deserialize, IntoStaticStr, Debug, Hash, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "type", content = "data")]
pub enum ImageSource<'a> {
    /// Image is sourced from an arbitrary URL.
    Url(Cow<'a, str>),

    /// Image is attached the current page.
    File1 { file: Cow<'a, str> },

    /// Image is attached to another page on the site.
    File2 {
        page: Cow<'a, str>,
        file: Cow<'a, str>,
    },

    /// Image is attached to another page on another site.
    File3 {
        site: Cow<'a, str>,
        page: Cow<'a, str>,
        file: Cow<'a, str>,
    },
}

impl<'t> ImageSource<'t> {
    pub fn parse(source: &'t str) -> Option<ImageSource<'t>> {
        if is_url(source) {
            return Some(ImageSource::Url(cow!(source)));
        }

        // Strip leading / if present
        let source = source.strip_prefix('/').unwrap_or(source);

        // Get parts for path
        let parts: Vec<&str> = source.split('/').collect();

        // Depending on the number of parts, determine the file variant
        let source = match parts.len() {
            1 => ImageSource::File1 {
                file: cow!(parts[0]),
            },
            2 => ImageSource::File2 {
                page: cow!(parts[0]),
                file: cow!(parts[1]),
            },
            3 => ImageSource::File3 {
                site: cow!(parts[0]),
                page: cow!(parts[1]),
                file: cow!(parts[2]),
            },
            _ => return None,
        };

        Some(source)
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        self.into()
    }

    pub fn to_owned(&self) -> ImageSource<'static> {
        match self {
            ImageSource::Url(url) => ImageSource::Url(string_to_owned(url)),
            ImageSource::File1 { file } => ImageSource::File1 {
                file: string_to_owned(file),
            },
            ImageSource::File2 { page, file } => ImageSource::File2 {
                page: string_to_owned(page),
                file: string_to_owned(file),
            },
            ImageSource::File3 { site, page, file } => ImageSource::File3 {
                site: string_to_owned(site),
                page: string_to_owned(page),
                file: string_to_owned(file),
            },
        }
    }
}
