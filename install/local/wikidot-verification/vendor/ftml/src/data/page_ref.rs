/*
 * data/page_ref.rs
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

use std::fmt::{self, Display};
use wikidot_normalize::normalize;

/// Represents a reference to a page on the wiki, as used by include notation.
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
pub struct PageRef {
    pub site: Option<String>,
    pub page: String,
    pub extra: Option<String>,
}

impl PageRef {
    /// Separates a non-normalized page slug (potentially with extra URL parts).
    fn split_page(page: &str) -> (&str, Option<&str>) {
        match page.find(['#', '/']) {
            None => (page, None),
            Some(index) => {
                let (page, extra) = page.split_at(index);
                (page, Some(extra))
            }
        }
    }

    /// Creates a [`PageRef`] with an optional site.
    pub fn new<S1, S2>(site: Option<S1>, page: S2) -> Self
    where
        S1: Into<String>,
        S2: AsRef<str>,
    {
        match site {
            Some(site) => Self::page_and_site(site, page),
            None => Self::page_only(page),
        }
    }

    /// Creates a [`PageRef`] with the given page and site.
    #[inline]
    pub fn page_and_site<S1, S2>(site: S1, page: S2) -> Self
    where
        S1: Into<String>,
        S2: AsRef<str>,
    {
        let (page, extra) = Self::split_page(page.as_ref());
        let mut site = site.into();
        let mut page = str!(page);
        let extra = extra.map(String::from);
        normalize(&mut site);
        normalize(&mut page);

        PageRef {
            site: Some(site),
            page,
            extra,
        }
    }

    /// Creates a [`PageRef`] with the given page and no site.
    #[inline]
    pub fn page_only<S>(page: S) -> Self
    where
        S: AsRef<str>,
    {
        let (page, extra) = Self::split_page(page.as_ref());
        let mut page = str!(page);
        let extra = extra.map(String::from);
        normalize(&mut page);
        PageRef {
            site: None,
            page,
            extra,
        }
    }

    #[inline]
    pub fn site(&self) -> Option<&str> {
        self.site.as_deref()
    }

    #[inline]
    pub fn page(&self) -> &str {
        &self.page
    }

    #[inline]
    pub fn extra(&self) -> Option<&str> {
        self.extra.as_deref()
    }

    #[inline]
    pub fn fields(&self) -> (Option<&str>, &str, Option<&str>) {
        (self.site(), self.page(), self.extra())
    }

    /// Like `fields()`, but uses the current site value to avoid returning `Option`.
    pub fn fields_or<'a>(&'a self, current_site: &'a str) -> (&'a str, &'a str, &'a str) {
        (
            self.site().unwrap_or(current_site),
            self.page(),
            self.extra().unwrap_or(""),
        )
    }

    pub fn parse(s: &str) -> Result<PageRef, PageRefParseError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(PageRefParseError);
        }

        let result = match s.find(':') {
            // Off-site page, e.g. ":scp-wiki:something"
            Some(0) => {
                // Find the second colon
                let idx = match s[1..].find(':') {
                    // Empty site name, e.g. "::something"
                    // or no second colon, e.g. ":something"
                    Some(0) | None => return Err(PageRefParseError),

                    // Slice off the rest
                    Some(idx) => idx + 1,
                };

                // Get site and page slices
                let site = s[1..idx].trim();
                let page = s[idx + 1..].trim();
                PageRef::page_and_site(site, page)
            }

            // On-site page, e.g. "component:thing"
            Some(_) => PageRef::page_only(s),

            // On-site page, with no category, e.g. "page"
            None => PageRef::page_only(s),
        };

        Ok(result)
    }
}

impl Display for PageRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(site) = self.site() {
            write!(f, ":{}:", &site)?;
        }

        write!(f, "{}", &self.page)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PageRefParseError;

// Tests

#[test]
fn split_page() {
    macro_rules! test {
        ($input:expr => $page:expr, $extra:expr $(,)?) => {
            assert_eq!(
                PageRef::split_page($input),
                ($page, $extra),
                "Split page portion does not have correct page and extra parts",
            )
        };

        // Test case with 'extra' part
        ($input:expr, $page:expr, $extra:expr $(,)?) => {
            test!($input => $page, Some($extra))
        };

        // Test case for no 'extra' part
        ($input:expr) => {
            test!($input => $input, None)
        };
    }

    // This function only does splitting, no normalization

    test!("scp-001");
    test!("scp-001/edit", "scp-001", "/edit");
    test!("scp-001/edit/true", "scp-001", "/edit/true");
    test!("Ethics Committee Orientation");
    test!(
        "Ethics Committee Orientation/edit",
        "Ethics Committee Orientation",
        "/edit",
    );
    test!(
        "Ethics Committee Orientation/edit/true",
        "Ethics Committee Orientation",
        "/edit/true",
    );
    test!("main#toc4", "main", "#toc4");
    test!("SCP-500/edit#page-options", "SCP-500", "/edit#page-options");
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
            let expected = $expected.ok_or(PageRefParseError);

            println!("Input: {:?}", $input);
            println!("Output: {:?}", actual);
            println!();

            assert_eq!(actual, expected, "Actual parse results don't match expected");
        }};
    }

    test!("");
    test!(":page");
    test!("::page");
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

#[cfg(test)]
mod prop {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(4096))]

        #[test]
        fn page_ref_page_prop(s in r".+") {
            let _ = PageRef::parse(&s);
        }

        #[test]
        fn page_ref_both_prop(s in r".+:.+") {
            let _ = PageRef::parse(&s);
        }

        #[test]
        fn page_ref_extra_prop(s in r".+:.+[#/].+") {
            let _ = PageRef::parse(&s);
        }
    }
}
