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
        let mut link_str = link.as_ref();

        // Check for direct URLs or anchor links
        if is_url(link_str) || link_str.starts_with('#') {
            return LinkLocation::Url(link);
        }

        // Check for local links starting with '/'
        if link_str.starts_with('/') {
            link_str = &link_str[1..];
        }

        match PageRef::parse(link_str) {
            Err(_) => LinkLocation::Url(link),
            Ok(page_ref) => LinkLocation::Page(page_ref.to_owned()),
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
        let string;

        serializer.emit_str(
            key,
            match self {
                LinkLocation::Url(url) => &url,
                LinkLocation::Page(page) => {
                    string = str!(page);
                    &string
                }
            },
        )
    }
}

#[test]
fn test_link_location() {
    macro_rules! check {
        ($input:expr => $site:expr, $page:expr) => {{
            let site = $site.map(|site| cow!(site));
            let page = cow!($page);
            let expected = LinkLocation::Page(PageRef { site, page });

            check!($input; expected);
        }};

        ($input:expr => $url:expr) => {
            let url = cow!($url);
            let expected = LinkLocation::Url(url);

            check!($input; expected);
        };

        ($input:expr; $expected:expr) => {{
            let actual = LinkLocation::parse(cow!($input));

            assert_eq!(
                actual,
                $expected,
                "Actual link location result doesn't match expected",
            );
        }};
    }

    check!("" => "");
    check!("#" => "#");
    check!("#anchor" => "#anchor");

    check!("page" => None, "page");
    check!("/page" => None, "page");
    check!("component:theme" => None, "component:theme");
    check!(":scp-wiki:scp-1000" => Some("scp-wiki"), "scp-1000");
    check!(":scp-wiki:component:theme" => Some("scp-wiki"), "component:theme");

    check!("http://blog.wikidot.com/" => "http://blog.wikidot.com/");
    check!("https://example.com" => "https://example.com");
    check!("mailto:test@example.net" => "mailto:test@example.net");

    check!("::page" => "::page");
    check!("::component:theme" => "::component:theme");
    check!("page:multiple:category" => None, "page:multiple:category");
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
