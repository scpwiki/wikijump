/*
 * services/render/url_arguments.rs
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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

//! Which wikitext depends on the arguments in a request's URL path.
//!
//! Deepwell bakes a page's HTML at revision-save time, so a request carrying
//! path arguments is only answered correctly if the view re-renders. This
//! module decides when that is necessary.

use super::child_pages::CHILD_PAGES_MODULE_REGEX;
use super::next_previous_page::NEXT_PREVIOUS_PAGE_MODULE_OPEN_REGEX;
use super::pages::PAGES_MODULE_REGEX;
use super::pages_by_tag::PAGES_BY_TAG_MODULE_REGEX;
use regex::Regex;
use std::borrow::Cow;
use std::sync::LazyLock;

/// A ListPages module opening whose head names `@URL` somewhere.
///
/// Only the head is examined, and only for the marker itself: whether the
/// marker sits in a `tags` selector or somewhere the renderer ignores is
/// settled later. Matching too eagerly costs one extra render; matching too
/// narrowly would serve the stored HTML and drop the argument entirely.
static LIST_PAGES_URL_SELECTOR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\[\[\s*module\s+listpages\b[^\]]*@url").unwrap());

/// One raw URL path argument addressed to a page module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UrlArgumentPair {
    pub name: String,
    pub value: Option<String>,
}

/// The Wikidot URL path arguments a render is answering.
///
/// Empty for every render that is not serving a page view, including the one
/// that produces a revision's stored HTML.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UrlArguments<'a> {
    /// `/tag/<value>`, read by `PagesByTag` and by a `tags="@URL"` selector.
    pub tag: Option<&'a str>,

    /// `/p/<n>`, the 1-based page a paginated `ListPages` renders.
    pub page: Option<u32>,

    /// `/category/<value>`, read by a `category="@URL"` selector.
    pub category: Option<&'a str>,

    /// `/offset/<n>`, read by a ListPages `offset="@URL|fallback"` selector.
    pub offset: Option<u32>,

    /// Ordered raw path arguments, kept for ListPages pager links.
    pub path_arguments: &'a [UrlArgumentPair],
}

impl<'a> UrlArguments<'a> {
    pub(in crate::services::render) fn value_for_list_pages_argument(
        self,
        prefix: Option<&str>,
        argument_name: &str,
    ) -> Option<&'a str> {
        let key = list_pages_argument_key(prefix, argument_name);
        let path_value = self
            .path_arguments
            .iter()
            .rfind(|argument| argument.name.eq_ignore_ascii_case(key.as_ref()))
            .and_then(|argument| argument.value.as_deref())
            .filter(|value| !value.is_empty());
        if path_value.is_some()
            || prefix
                .map(str::trim)
                .is_some_and(|prefix| !prefix.is_empty())
        {
            return path_value;
        }

        match argument_name.to_ascii_lowercase().as_str() {
            "tag" | "tags" => self.tag.filter(|value| !value.is_empty()),
            "category" | "categories" => self.category.filter(|value| !value.is_empty()),
            _ => None,
        }
    }

    pub(in crate::services::render) fn page_for_prefix(
        self,
        prefix: Option<&str>,
    ) -> Option<u32> {
        let key = list_pages_page_argument_key(prefix);
        let page = self
            .path_arguments
            .iter()
            .filter(|argument| argument.name.eq_ignore_ascii_case(key.as_ref()))
            .filter_map(|argument| argument.value.as_deref())
            .filter_map(|value| value.parse::<u32>().ok())
            .rfind(|page| *page > 0);
        page.or_else(|| (key == "p").then_some(self.page).flatten())
    }
}

pub(in crate::services::render) fn list_pages_page_argument_key(
    prefix: Option<&str>,
) -> Cow<'_, str> {
    list_pages_argument_key(prefix, "p")
}

fn list_pages_argument_key<'a>(
    prefix: Option<&str>,
    argument_name: &'a str,
) -> Cow<'a, str> {
    match prefix.map(str::trim).filter(|prefix| !prefix.is_empty()) {
        Some(prefix) => Cow::Owned(format!("{prefix}_{argument_name}")),
        None => Cow::Borrowed(argument_name),
    }
}

/// A ListPages module opening that may answer `/p/<n>`.
///
/// ListPages defaults to 20 rows per page, so an explicit `perPage` argument is
/// not required for a request path to affect the rendered result.
static LIST_PAGES_MODULE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\[\[\s*module\s+listpages\b").unwrap());

/// Whether this wikitext holds a module whose output depends on the request's
/// URL path arguments.
///
/// The page view uses this to decide whether a request carrying arguments
/// needs a render of its own instead of the revision's stored HTML. It looks
/// at the page's own source only: a module reached through `[[include]]`
/// renders as it does without arguments, which is the same result Wikijump
/// produced before arguments were routed at all.
pub fn wikitext_reads_url_arguments(wikitext: &str) -> bool {
    wikitext_has_bare_pages_module(wikitext)
        || PAGES_BY_TAG_MODULE_REGEX.is_match(wikitext)
        || LIST_PAGES_URL_SELECTOR_REGEX.is_match(wikitext)
        || NEXT_PREVIOUS_PAGE_MODULE_OPEN_REGEX.is_match(wikitext)
        || LIST_PAGES_MODULE_REGEX.is_match(wikitext)
}

/// Whether a page view must render from source even without URL arguments.
///
/// `Pages` is a live site index. Its first page changes when pages are created,
/// renamed, deleted, or become visible, so stored revision HTML cannot answer
/// even the bare request.
pub fn wikitext_requires_runtime_render(wikitext: &str) -> bool {
    wikitext_has_bare_pages_module(wikitext)
        || CHILD_PAGES_MODULE_REGEX.is_match(wikitext)
        || NEXT_PREVIOUS_PAGE_MODULE_OPEN_REGEX.is_match(wikitext)
}

fn wikitext_has_bare_pages_module(wikitext: &str) -> bool {
    PAGES_MODULE_REGEX.captures_iter(wikitext).any(|captures| {
        captures
            .name("head")
            .is_none_or(|head| head.as_str().trim().is_empty())
    })
}

#[cfg(test)]
mod tests {
    use super::{
        UrlArgumentPair, UrlArguments, wikitext_reads_url_arguments,
        wikitext_requires_runtime_render,
    };

    #[test]
    fn a_pages_by_tag_module_reads_url_arguments() {
        assert!(wikitext_reads_url_arguments("[[module PagesByTag]]"));
    }

    #[test]
    fn a_pages_module_reads_url_arguments() {
        assert!(wikitext_reads_url_arguments("[[module Pages]]"));
    }

    #[test]
    fn a_pages_module_always_requires_runtime_rendering() {
        assert!(wikitext_requires_runtime_render("[[module Pages]]"));
        assert!(wikitext_requires_runtime_render("[[module ChildPages]]"));
        assert!(wikitext_requires_runtime_render(
            r#"[[module NextPage by="title"]]%%linked_title%%[[/module]]"#
        ));
        assert!(wikitext_reads_url_arguments(
            r#"[[module PreviousPage tags="@URL"]]%%linked_title%%[[/module]]"#
        ));
        assert!(!wikitext_requires_runtime_render(
            "[[module ListPages category=\"news\"]]%%title%%[[/module]]"
        ));
        assert!(!wikitext_requires_runtime_render(
            "[[module Pages limit=\"5\"]]"
        ));
    }

    #[test]
    fn a_list_pages_url_selector_reads_url_arguments() {
        assert!(wikitext_reads_url_arguments(
            r#"[[module ListPages tags="@URL" limit="20"]]%%title%%[[/module]]"#
        ));
        assert!(wikitext_reads_url_arguments(
            r#"[[module listpages tags="@url|_"]]%%title%%[[/module]]"#
        ));
    }

    #[test]
    fn a_paginated_list_pages_module_reads_url_arguments() {
        assert!(wikitext_reads_url_arguments(
            r#"[[module ListPages tags="alpha" perPage="20"]]%%title%%[[/module]]"#
        ));
        assert!(wikitext_reads_url_arguments(
            r#"[[module listpages per_page="5"]]%%title%%[[/module]]"#
        ));
    }

    #[test]
    fn a_default_list_pages_module_reads_url_arguments() {
        assert!(wikitext_reads_url_arguments(
            r#"[[module ListPages tags="alpha"]]%%title%%[[/module]]"#
        ));
    }

    #[test]
    fn plain_wikitext_does_not() {
        assert!(!wikitext_reads_url_arguments(
            "Ordinary text mentioning @URL and ListPages separately."
        ));
    }

    #[test]
    fn page_selection_uses_last_positive_matching_prefix() {
        let path_arguments = vec![
            UrlArgumentPair {
                name: "p".to_owned(),
                value: Some("2".to_owned()),
            },
            UrlArgumentPair {
                name: "p".to_owned(),
                value: Some("3".to_owned()),
            },
            UrlArgumentPair {
                name: "a_p".to_owned(),
                value: Some("4".to_owned()),
            },
            UrlArgumentPair {
                name: "b_p".to_owned(),
                value: Some("0".to_owned()),
            },
        ];
        let url = UrlArguments {
            path_arguments: &path_arguments,
            ..UrlArguments::default()
        };

        assert_eq!(url.page_for_prefix(None), Some(3));
        assert_eq!(url.page_for_prefix(Some("a")), Some(4));
        assert_eq!(url.page_for_prefix(Some("b")), None);
    }

    #[test]
    fn list_pages_arguments_use_the_last_matching_prefixed_value() {
        let path_arguments = vec![
            UrlArgumentPair {
                name: "limit".to_owned(),
                value: Some("9".to_owned()),
            },
            UrlArgumentPair {
                name: "page2_limit".to_owned(),
                value: Some("1".to_owned()),
            },
            UrlArgumentPair {
                name: "PAGE2_LIMIT".to_owned(),
                value: Some("2".to_owned()),
            },
        ];
        let url = UrlArguments {
            path_arguments: &path_arguments,
            ..UrlArguments::default()
        };

        assert_eq!(url.value_for_list_pages_argument(None, "limit"), Some("9"),);
        assert_eq!(
            url.value_for_list_pages_argument(Some("page2"), "limit"),
            Some("2"),
        );
        assert_eq!(
            url.value_for_list_pages_argument(Some("page3"), "limit"),
            None,
        );
    }
}
