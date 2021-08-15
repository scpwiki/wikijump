/*
 * tree/link.rs
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

use super::clone::{option_string_to_owned, string_to_owned};
use crate::data::PageRef;
use crate::url::is_url;
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum LinkLocation<'a> {
    /// This link points to a particular page on a wiki.
    Page(PageRef<'a>),

    /// This link is to a specific URL.
    Url(Cow<'a, str>),
}

impl<'a> LinkLocation<'a> {
    pub fn parse(link: Cow<'a, str>) -> Self {
        let link_str = link.as_ref();

        if is_url(link_str) || link_str.starts_with('#') {
            return LinkLocation::Url(link);
        }

        match PageRef::parse(link_str) {
            Ok(page_ref) => LinkLocation::Page(page_ref),
            Err(_) => LinkLocation::Url(link),
        }
    }

    pub fn url(&self, domain: &str) -> Cow<str> {
        match self {
            LinkLocation::Url(url) => cow_borrow!(url),
            LinkLocation::Page(page_ref) => {
                let page = page_ref.page();

                match page_ref.site() {
                    Some(site) => {
                        Cow::Owned(format!("https://{}.{}/{}", site, domain, page))
                    }
                    None => cow_borrow!(page),
                }
            }
        }
    }

    pub fn to_owned(&self) -> LinkLocation<'static> {
        match self {
            LinkLocation::Page(page) => LinkLocation::Page(page.to_owned()),
            LinkLocation::Url(url) => LinkLocation::Url(string_to_owned(url)),
        }
    }
}

#[cfg(feature = "log")]
impl slog::Value for LinkLocation<'_> {
    fn serialize(
        &self,
        _: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(
            key,
            match self {
                LinkLocation::Page(page) => &str!(page),
                LinkLocation::Url(url) => &url,
            },
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LinkLabel<'a> {
    /// Custom text link label.
    ///
    /// Can be set to any arbitrary value of the input text's choosing.
    Text(Cow<'a, str>),

    /// URL-mirroring link label.
    ///
    /// If `None`, then the label for this link is the same as the URL.
    /// If `Some(_)`, then the label is a subslice of the URL it targets.
    Url(Option<Cow<'a, str>>),

    /// Article title-based link label.
    ///
    /// The label for this link is whatever the page's title is.
    Page,
}

impl LinkLabel<'_> {
    pub fn to_owned(&self) -> LinkLabel<'static> {
        match self {
            LinkLabel::Text(text) => LinkLabel::Text(string_to_owned(text)),
            LinkLabel::Url(url) => LinkLabel::Url(option_string_to_owned(url)),
            LinkLabel::Page => LinkLabel::Page,
        }
    }
}

#[test]
fn test_link_location() {
    todo!();
}
