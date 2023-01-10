/*
 * utils/category.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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

/// Splits a normalized slug into the category and page portions.
///
/// This finds the last `:` in the full slug and returns everything
/// up to that as the category slug.
///
/// Normalized slugs do not have an explicit `_default`, so they
/// should lack a `:` entirely.
pub fn split_category(slug: &str) -> (Option<&str>, &str) {
    match slug.rfind(':') {
        None => (None, slug),
        Some(idx) => {
            let (category, page) = slug.split_at(idx);
            (Some(category), &page[1..])
        }
    }
}

/// Splits a normalized slug into the category and page portions.
///
/// If the category would be `None`, then an`explicit `_default`
/// is returned.
#[inline]
pub fn split_category_name(slug: &str) -> (&str, &str) {
    let (category, slug) = split_category(slug);
    (category.unwrap_or("_default"), slug)
}

/// Retrieves the category portion of a slug, if it exists.
#[inline]
#[allow(dead_code)] // TEMP
pub fn get_category(slug: &str) -> Option<&str> {
    split_category(slug).0
}

/// Retrieves the category name for a slug.
#[inline]
pub fn get_category_name(slug: &str) -> &str {
    split_category_name(slug).0
}

/// Trims off the `_default:` category if present.
pub fn trim_default(slug: &str) -> &str {
    // We cannot simply use str::strip_prefix() here,
    // since if the category *starts* with "_default"
    // but is not solely "_default" (for instance,
    // the category string "_default:blah", as in
    // "_default:blah:page-name") then this will
    // mangle the category name.

    match split_category_name(slug) {
        ("_default", page_slug) => page_slug,
        (_, _) => slug,
    }
}

#[test]
fn test_split_category() {
    macro_rules! check {
        ($input:expr, $category:expr, $page:expr $(,)?) => {
            assert_eq!(
                split_category($input),
                ($category, $page),
                "Actual split category doesn't match expected",
            )
        };
    }

    check!("apple", None, "apple");
    check!("foo-bar", None, "foo-bar");
    check!("component:wide-modal", Some("component"), "wide-modal");
    check!(
        "archived:component:wide-modal",
        Some("archived:component"),
        "wide-modal",
    );
    check!("_default:start", Some("_default"), "start");
    check!("_default:_template", Some("_default"), "_template");
}

#[test]
fn test_split_category_name() {
    macro_rules! check {
        ($input:expr, $category:expr, $page:expr $(,)?) => {
            assert_eq!(
                split_category_name($input),
                ($category, $page),
                "Actual split category with name doesn't match expected",
            )
        };
    }

    check!("apple", "_default", "apple");
    check!("foo-bar", "_default", "foo-bar");
    check!("component:wide-modal", "component", "wide-modal");
    check!(
        "archived:component:wide-modal",
        "archived:component",
        "wide-modal",
    );
    check!("_default:start", "_default", "start");
    check!("_default:_template", "_default", "_template");
}

#[test]
fn test_get_category() {
    macro_rules! check {
        ($input:expr, $expected:expr $(,)?) => {
            assert_eq!(
                get_category($input),
                $expected,
                "Actual parsed category doesn't match expected",
            )
        };
    }

    check!("apple", None);
    check!("foo-bar", None);
    check!("component:wide-modal", Some("component"));
    check!("archived:component:wide-modal", Some("archived:component"));
    check!("_default:start", Some("_default"));
    check!("_default:_template", Some("_default"));
}

#[test]
fn test_get_category_name() {
    macro_rules! check {
        ($input:expr, $expected:expr $(,)?) => {
            assert_eq!(
                get_category_name($input),
                $expected,
                "Actual parsed category doesn't match expected",
            )
        };
    }

    check!("apple", "_default");
    check!("foo-bar", "_default");
    check!("component:wide-modal", "component");
    check!("archived:component:wide-modal", "archived:component");
    check!("_default:start", "_default");
    check!("_default:_template", "_default");
}

#[test]
fn test_trim_default() {
    macro_rules! check {
        ($input:expr, $expected:expr $(,)?) => {
            assert_eq!(
                trim_default($input),
                $expected,
                "Actual trimmed slug doesn't match expected",
            )
        };
    }

    check!("apple", "apple");
    check!("foo-bar", "foo-bar");
    check!("component:wide-modal", "component:wide-modal");
    check!(
        "archived:component:wide-modal",
        "archived:component:wide-modal",
    );
    check!("_default:start", "start");
    check!("_default:foo-bar", "foo-bar");
    check!("_default:_template", "_template");
    check!("archived:_default:start", "archived:_default:start");
    check!("_default:archived:start", "_default:archived:start");
}
