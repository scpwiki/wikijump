/*
 * data/page_ref.rs
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

use ref_map::*;
use std::borrow::Cow;
use std::fmt::{self, Display};

/// Represents a reference to a page on the wiki, as used by includes.
///
/// It tracks whether it refers to a page on this wiki, or some other,
/// and what the names of these are.
///
/// The Wikidot syntax here allows for two cases:
/// * `:wiki-name:page` (off-site)
/// * `page` (on-site)
///
/// Additionally "`page`" here may also contain colons, such as `component:some-thing`.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct PageRef<'t> {
    site: Option<Cow<'t, str>>,
    page: Cow<'t, str>,
}

impl<'t> PageRef<'t> {
    #[inline]
    pub fn new(site: Option<Cow<'t, str>>, page: Cow<'t, str>) -> Self {
        PageRef { site, page }
    }

    #[inline]
    pub fn page_and_site<S1, S2>(site: S1, page: S2) -> Self
    where
        S1: Into<Cow<'t, str>>,
        S2: Into<Cow<'t, str>>,
    {
        let site = site.into();
        let page = page.into();

        PageRef::new(Some(site), page)
    }

    #[inline]
    pub fn page_only<S>(page: S) -> Self
    where
        S: Into<Cow<'t, str>>,
    {
        let page = page.into();

        PageRef::new(None, page)
    }

    #[inline]
    pub fn site(&self) -> Option<&str> {
        self.site.ref_map(|s| s.as_ref())
    }

    #[inline]
    pub fn page(&self) -> &str {
        self.page.as_ref()
    }

    pub fn parse(s: &'t str) -> Option<PageRef<'t>> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }

        let result = match s.find(':') {
            // Off-site page, e.g. ":scp-wiki:something"
            Some(0) => {
                // Find the second colon
                let idx = match s[1..].find(':') {
                    Some(idx) => idx + 1,
                    None => return None,
                };

                // Get site and page slices
                let site = &s[1..idx];
                let page = &s[idx + 1..];

                PageRef::page_and_site(site, page)
            }

            // On-site page, e.g. "component:thing"
            Some(_) => PageRef::page_only(s),

            // On-site page, with no category, e.g. "page"
            None => PageRef::page_only(s),
        };

        Some(result)
    }

    pub fn to_owned(&self) -> PageRef<'static> {
        macro_rules! owned {
            ($value:expr) => {
                Cow::Owned($value.as_ref().to_owned())
            };
        }

        let site = self.site.ref_map(|value| owned!(value));
        let page = owned!(&self.page);

        PageRef { site, page }
    }
}

impl Display for PageRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(site) = self.site() {
            write!(f, ":{}:", &site)?;
        }

        write!(f, "{}", &self.page)
    }
}

#[test]
fn page_ref() {
    macro_rules! test {
        ($input:expr $(,)?) => {
            test!($input => None)
        };

        ($input:expr, $expected:expr $(,)?) => {
            test!($input => Some($expected))
        };

        ($input:expr => $expected:expr) => {{
            let actual = PageRef::parse($input);
            println!("Input: {:?}", $input);
            println!("Output: {:?}", actual);
            println!();

            assert_eq!(actual, $expected, "Actual parse results don't match expected");
        }};
    }

    test!("");
    test!(":page");
    test!("page", PageRef::page_only("page"));
    test!("component:page", PageRef::page_only("component:page"));
    test!(
        "deleted:secret:fragment:page",
        PageRef::page_only("deleted:secret:fragment:page"),
    );
    test!(":scp-wiki:page", PageRef::page_and_site("scp-wiki", "page"));
    test!(
        ":scp-wiki:component:page",
        PageRef::page_and_site("scp-wiki", "component:page"),
    );
    test!(
        ":scp-wiki:deleted:secret:fragment:page",
        PageRef::page_and_site("scp-wiki", "deleted:secret:fragment:page"),
    );
}
