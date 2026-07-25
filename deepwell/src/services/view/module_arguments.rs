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

        PageModuleArguments { tag }
    }

    /// Whether the path addressed any module at all.
    pub fn is_empty(&self) -> bool {
        self.tag.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
