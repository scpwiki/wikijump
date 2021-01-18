/*
 * include/object.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Ammon Smith
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
use std::collections::HashMap;
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

    pub fn parse(s: &'t str) -> Result<PageRef<'t>, ()> {
        let s = s.trim();
        if s.is_empty() {
            return Err(());
        }

        let result = match s.find(':') {
            // Off-site page, e.g. ":scp-wiki:something"
            Some(0) => {
                // Find the second colon
                let idx = match s[1..].find(':') {
                    Some(idx) => idx + 1,
                    None => return Err(()),
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

        Ok(result)
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

/// Represents an include block.
///
/// It contains the page being included, as well as the arguments
/// to be passed to it when doing the substitution.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct IncludeRef<'t> {
    page: PageRef<'t>,
    variables: HashMap<Cow<'t, str>, Cow<'t, str>>,
}

impl<'t> IncludeRef<'t> {
    #[inline]
    pub fn page_with_args(
        page: PageRef<'t>,
        variables: HashMap<Cow<'t, str>, Cow<'t, str>>,
    ) -> Self {
        IncludeRef { page, variables }
    }

    #[inline]
    pub fn page_only(page: PageRef<'t>) -> Self {
        Self::page_with_args(page, HashMap::new())
    }
}
