/*
 * tree/link.rs
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
use crate::data::PageRef;
use crate::settings::WikitextSettings;
use crate::url::is_url;
use std::borrow::Cow;
use strum_macros::EnumIter;

#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum LinkLocation<'a> {
    /// This link points to a particular page on a wiki.
    Page(PageRef),

    /// This link is to a specific URL.
    Url(Cow<'a, str>),
}

impl<'a> LinkLocation<'a> {
    /// Like `parse()`, but also handles interwiki links.
    pub fn parse_with_interwiki(
        link: Cow<'a, str>,
        settings: &WikitextSettings,
    ) -> Option<(Self, LinkType)> {
        // Handle interwiki (starts with "!", like "!wp:Apple")
        match link.as_ref().strip_prefix('!') {
            // Not interwiki, parse as normal
            None => {
                let interwiki = Self::parse(link);
                let ltype = interwiki.link_type();
                Some((interwiki, ltype))
            }

            // Try to interpret as interwiki
            Some(link) => settings
                .interwiki
                .build(link)
                .map(|url| (LinkLocation::Url(Cow::Owned(url)), LinkType::Interwiki)),
        }
    }

    pub fn parse(link: Cow<'a, str>) -> Self {
        // Check for direct URLs or anchor links
        // TODO: parse local links into LinkLocation::Page
        // Known bug: single "/" parsed into Url instead of Page
        let link_str = &link;
        if is_url(link_str) || link_str.starts_with('#') || link_str.starts_with("/") {
            return LinkLocation::Url(link);
        }

        match PageRef::parse(link_str) {
            Err(_) => LinkLocation::Url(link),
            Ok(page_ref) => LinkLocation::Page(page_ref),
        }
    }

    pub fn to_owned(&self) -> LinkLocation<'static> {
        match self {
            LinkLocation::Page(page) => LinkLocation::Page(page.to_owned()),
            LinkLocation::Url(url) => LinkLocation::Url(string_to_owned(url)),
        }
    }

    pub fn link_type(&self) -> LinkType {
        match self {
            LinkLocation::Page(_) => LinkType::Page,
            LinkLocation::Url(_) => LinkType::Direct,
        }
    }
}

#[test]
fn test_link_location() {
    // Use of a helper function coerces None to be the right kind of Option<T>
    #[inline]
    fn convert_opt(s: Option<&str>) -> Option<String> {
        s.map(String::from)
    }

    macro_rules! test {
        // LinkLocation::Page
        ($input:expr => $site:expr, $page:expr, $extra:expr $(,)?) => {{
            let expected = LinkLocation::Page(PageRef {
                site: convert_opt($site),
                page: str!($page),
                extra: convert_opt($extra),
            });
            test!($input; expected);
        }};

        // LinkLocation::Url
        ($input:expr => $url:expr $(,)?) => {
            let url = cow!($url);
            let expected = LinkLocation::Url(url);
            test!($input; expected);
        };

        // Specified LinkLocation
        ($input:expr; $expected:expr $(,)?) => {{
            let actual = LinkLocation::parse(cow!($input));
            assert_eq!(
                actual,
                $expected,
                "Actual link location result doesn't match expected",
            );
        }};
    }

    test!("" => "");
    test!("#" => "#");
    test!("#anchor" => "#anchor");

    test!("page" => None, "page", None);
    test!("page/edit" => None, "page", Some("/edit"));
    test!("page#toc0" => None, "page", Some("#toc0"));
    test!("page/comments#main" => None, "page", Some("/comments#main"));

    test!("/page" => "/page");
    test!("/page/edit" => "/page/edit");
    test!("/page#toc0" => "/page#toc0");

    test!("component:theme" => None, "component:theme", None);
    test!(":scp-wiki:scp-1000" => Some("scp-wiki"), "scp-1000", None);
    test!(
        ":scp-wiki:scp-1000#page-options-bottom" =>
            Some("scp-wiki"), "scp-1000", Some("#page-options-bottom"),
    );
    test!(
        ":scp-wiki:component:theme" =>
            Some("scp-wiki"), "component:theme", None,
    );
    test!(
        ":scp-wiki:component:theme/edit/true" =>
            Some("scp-wiki"), "component:theme", Some("/edit/true"),
    );

    test!("http://blog.wikidot.com/" => "http://blog.wikidot.com/");
    test!("https://example.com" => "https://example.com");
    test!("mailto:test@example.net" => "mailto:test@example.net");

    test!("::page" => "::page");
    test!("::component:theme" => "::component:theme");
    test!("multiple:category:page" => None, "multiple-category:page", None);
}

#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LinkLabel<'a> {
    /// Custom text link label.
    ///
    /// Can be set to any arbitrary value of the input text's choosing.
    Text(Cow<'a, str>),

    /// Page slug-based link label.
    ///
    /// This is set when the link is also the label.
    /// The link is pre-normalization but post-category stripping.
    ///
    /// For instance:
    /// * `[[[SCP-001]]]`
    /// * `[[[Ethics Committee Orientation]]]`
    /// * `[[[system: Recent Pages]]]`
    Slug(Cow<'a, str>),

    /// URL-mirroring link label.
    ///
    /// This is where the label is just the same as the URL.
    Url,

    /// Article title-based link label.
    ///
    /// The label for this link is whatever the page's title is.
    Page,
}

impl LinkLabel<'_> {
    pub fn to_owned(&self) -> LinkLabel<'static> {
        match self {
            LinkLabel::Text(text) => LinkLabel::Text(string_to_owned(text)),
            LinkLabel::Slug(text) => LinkLabel::Slug(string_to_owned(text)),
            LinkLabel::Url => LinkLabel::Url,
            LinkLabel::Page => LinkLabel::Page,
        }
    }
}

#[derive(EnumIter, Serialize, Deserialize, Debug, Hash, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LinkType {
    /// This URL was specified directly.
    ///
    /// For instance, as a raw URL, or a single-bracket link.
    Direct,

    /// This URL was specified by specifying a particular Wikijump page.
    ///
    /// This variant comes from triple-bracket links.
    Page,

    /// This URL was generated via interwiki substitution.
    Interwiki,

    /// This URL points to an anchor elsewhere on this page.
    Anchor,

    /// This URL points to entries on a page in a table of contents.
    TableOfContents,
}

impl LinkType {
    pub fn name(self) -> &'static str {
        match self {
            LinkType::Direct => "direct",
            LinkType::Page => "page",
            LinkType::Interwiki => "interwiki",
            LinkType::Anchor => "anchor",
            LinkType::TableOfContents => "table-of-contents",
        }
    }
}

impl<'a> TryFrom<&'a str> for LinkType {
    type Error = &'a str;

    fn try_from(value: &'a str) -> Result<LinkType, &'a str> {
        match value {
            "direct" => Ok(LinkType::Direct),
            "page" => Ok(LinkType::Page),
            "interwiki" => Ok(LinkType::Interwiki),
            "anchor" => Ok(LinkType::Anchor),
            "table-of-contents" => Ok(LinkType::TableOfContents),
            _ => Err(value),
        }
    }
}

/// Ensure `LinkType::name()` produces the same output as serde.
#[test]
fn link_type_name_serde() {
    use strum::IntoEnumIterator;

    for variant in LinkType::iter() {
        let output = serde_json::to_string(&variant).expect("Unable to serialize JSON");
        let serde_name: String =
            serde_json::from_str(&output).expect("Unable to deserialize JSON");

        assert_eq!(
            &serde_name,
            variant.name(),
            "Serde name does not match variant name",
        );

        let converted: LinkType = serde_name
            .as_str()
            .try_into()
            .expect("Could not convert item");

        assert_eq!(converted, variant, "Converted item does not match variant");
    }
}
