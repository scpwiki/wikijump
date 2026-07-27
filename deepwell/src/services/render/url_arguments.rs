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

use super::pages::PAGES_MODULE_REGEX;
use super::pages_by_tag::PAGES_BY_TAG_MODULE_REGEX;
use regex::Regex;
use std::sync::LazyLock;

/// A ListPages module opening whose head names `@URL` somewhere.
///
/// Only the head is examined, and only for the marker itself: whether the
/// marker sits in a `tags` selector or somewhere the renderer ignores is
/// settled later. Matching too eagerly costs one extra render; matching too
/// narrowly would serve the stored HTML and drop the argument entirely.
static LIST_PAGES_URL_SELECTOR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\[\[\s*module\s+listpages\b[^\]]*@url").unwrap());

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
}

/// A ListPages module opening that paginates, and so answers `/p/<n>`.
///
/// A module without `perPage` renders one fixed list no matter what page the
/// URL asks for, so it is left out rather than re-rendered for nothing.
static LIST_PAGES_PAGINATED_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[\s*module\s+listpages\b[^\]]*per_?page").unwrap()
});

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
        || LIST_PAGES_PAGINATED_REGEX.is_match(wikitext)
}

/// Whether a page view must render from source even without URL arguments.
///
/// `Pages` is a live site index. Its first page changes when pages are created,
/// renamed, deleted, or become visible, so stored revision HTML cannot answer
/// even the bare request.
pub fn wikitext_requires_runtime_render(wikitext: &str) -> bool {
    wikitext_has_bare_pages_module(wikitext)
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
    use super::{wikitext_reads_url_arguments, wikitext_requires_runtime_render};

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
    fn a_static_list_pages_module_does_not() {
        assert!(!wikitext_reads_url_arguments(
            r#"[[module ListPages tags="alpha"]]%%title%%[[/module]]"#
        ));
    }

    #[test]
    fn plain_wikitext_does_not() {
        assert!(!wikitext_reads_url_arguments(
            "Ordinary text mentioning @URL and ListPages separately."
        ));
    }
}
