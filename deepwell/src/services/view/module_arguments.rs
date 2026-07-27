/*
 * services/view/module_arguments.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
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

//! Wikidot URL path arguments that a module on the viewed page reads.
//!
//! These are the same `/name/value` pairs [`PageOptions`] parses, split out
//! because they address the page's modules rather than the view itself. Only
//! names with a live capture appear here; an unrecognized name is discarded,
//! which is what live Wikidot does with `/bogusarg/xyz`.
//!
//! [`PageOptions`]: super::options::PageOptions

use super::options::PAGE_ARGUMENTS_SCHEMA;
use unicase::UniCase;
use wikidot_path::PageArguments;

/// URL path arguments addressed to the viewed page's modules.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageModuleArguments {
    /// `/tag/<value>`, read by `PagesByTag`.
    ///
    /// `Some("")` is distinct from `None`: live renders an empty tag heading
    /// for `/tag` and `/tag/`, and renders nothing at all when the argument is
    /// absent.
    pub tag: Option<String>,

    /// `/p/<n>`, the 1-based page number a paginated `ListPages` renders.
    ///
    /// Only a positive integer counts. Live ignores `/p/0` and `/p/abc` rather
    /// than erroring, so those parse as absent and the module renders its
    /// first page.
    pub page: Option<u32>,

    /// `/category/<value>`, read by a `ListPages` `category="@URL"` selector.
    ///
    /// As with [`tag`](Self::tag), `Some("")` is kept distinct from `None` so
    /// the renderer decides what an empty segment means.
    pub category: Option<String>,

    /// `/offset/<n>`, read by a ListPages `offset="@URL|fallback"` selector.
    ///
    /// Invalid and negative values are absent, so the selector uses its
    /// authored fallback.
    pub offset: Option<u32>,
}

impl PageModuleArguments {
    pub fn parse(extra: &str) -> Self {
        let arguments = PageArguments::parse(extra, PAGE_ARGUMENTS_SCHEMA).0;

        // The raw segment is used rather than the parsed `ArgumentValue`
        // because a tag can legitimately be `true`, `t`, `f`, or a number,
        // which the value parser would turn into a boolean or an integer.
        let tag = arguments
            .get(&UniCase::unicode("tag"))
            .map(|(_, raw)| (*raw).to_owned());

        let page = arguments
            .get(&UniCase::unicode("p"))
            .and_then(|(_, raw)| raw.parse::<u32>().ok())
            .filter(|page| *page > 0);

        let category = arguments
            .get(&UniCase::unicode("category"))
            .map(|(_, raw)| (*raw).to_owned());

        let offset = arguments
            .get(&UniCase::unicode("offset"))
            .and_then(|(_, raw)| raw.parse::<u32>().ok());

        PageModuleArguments {
            tag,
            page,
            category,
            offset,
        }
    }

    /// Whether the path addressed any module at all.
    pub fn is_empty(&self) -> bool {
        self.tag.is_none()
            && self.page.is_none()
            && self.category.is_none()
            && self.offset.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_page_number_is_read_as_a_positive_integer() {
        assert_eq!(PageModuleArguments::parse("/p/3").page, Some(3));
        assert_eq!(PageModuleArguments::parse("/p/1").page, Some(1));
    }

    #[test]
    fn a_page_number_that_is_not_a_positive_integer_is_absent() {
        for extra in ["/p/0", "/p/abc", "/p/-2", "/p/", "/p/2.5"] {
            assert_eq!(
                PageModuleArguments::parse(extra).page,
                None,
                "{extra} should not yield a page number",
            );
        }
    }

    #[test]
    fn a_category_argument_is_read_verbatim() {
        assert_eq!(
            PageModuleArguments::parse("/category/wjcatzone")
                .category
                .as_deref(),
            Some("wjcatzone"),
        );
        assert_eq!(
            PageModuleArguments::parse("/category").category.as_deref(),
            Some(""),
        );
        assert_eq!(PageModuleArguments::parse("/tag/x").category, None);
    }

    #[test]
    fn an_offset_is_read_as_a_nonnegative_integer() {
        assert_eq!(PageModuleArguments::parse("/offset/0").offset, Some(0));
        assert_eq!(PageModuleArguments::parse("/offset/25").offset, Some(25));
    }

    #[test]
    fn an_invalid_offset_is_absent() {
        for extra in [
            "/offset/not-an-integer",
            "/offset/-1",
            "/offset/",
            "/offset/2.5",
        ] {
            assert_eq!(
                PageModuleArguments::parse(extra).offset,
                None,
                "{extra} should not yield an offset",
            );
        }
    }

    #[test]
    fn a_page_number_and_a_tag_can_share_one_path() {
        let arguments = PageModuleArguments::parse("/tag/alpha/p/2");
        assert_eq!(arguments.tag.as_deref(), Some("alpha"));
        assert_eq!(arguments.page, Some(2));
    }

    #[test]
    fn no_arguments_address_no_module() {
        assert!(PageModuleArguments::parse("").is_empty());
        assert!(PageModuleArguments::parse("/").is_empty());
    }

    #[test]
    fn tag_argument_is_read_verbatim() {
        assert_eq!(
            PageModuleArguments::parse("/tag/golem-of-prague")
                .tag
                .as_deref(),
            Some("golem-of-prague"),
        );
    }

    #[test]
    fn tag_values_that_look_like_other_types_survive() {
        // `ArgumentValue` would render these as booleans and an integer.
        for value in ["true", "false", "t", "f", "2024"] {
            assert_eq!(
                PageModuleArguments::parse(&format!("/tag/{value}"))
                    .tag
                    .as_deref(),
                Some(value),
                "tag '{value}' must survive as written",
            );
        }
    }

    #[test]
    fn a_name_without_a_value_yields_an_empty_tag() {
        // Live renders `<em></em>` and an empty list for both of these, which
        // is distinct from omitting the argument entirely.
        assert_eq!(PageModuleArguments::parse("/tag").tag.as_deref(), Some(""));
        assert_eq!(PageModuleArguments::parse("/tag/").tag.as_deref(), Some(""));
    }

    #[test]
    fn pair_order_does_not_matter() {
        let tag_first = PageModuleArguments::parse("/tag/alpha/p/2");
        let page_first = PageModuleArguments::parse("/p/2/tag/alpha");

        assert_eq!(tag_first, page_first);
        assert_eq!(tag_first.tag.as_deref(), Some("alpha"));
    }

    #[test]
    fn a_repeated_name_takes_the_last_occurrence() {
        assert_eq!(
            PageModuleArguments::parse("/tag/alpha/tag/beta")
                .tag
                .as_deref(),
            Some("beta"),
        );
    }

    #[test]
    fn an_unrecognized_name_is_discarded() {
        assert!(PageModuleArguments::parse("/bogusarg/xyz").is_empty());
    }

    #[test]
    fn a_solo_view_option_does_not_swallow_the_following_tag() {
        assert_eq!(
            PageModuleArguments::parse("/norender/tag/alpha")
                .tag
                .as_deref(),
            Some("alpha"),
        );
    }
}
