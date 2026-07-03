/*
 * services/settings/structs.rs
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

/// Describes a navigation page slug.
///
/// This can either be `Enabled(_)`, containing the page slug to use (if it exists),
/// or `Disabled`, which means this navigation element should *not* be rendered
/// for this category.
///
/// # Invariants
/// * `Enabled(_)` never contains an empty string.
#[derive(Debug)]
pub enum NavigationPage {
    Enabled(String),
    Disabled,
}

impl From<String> for NavigationPage {
    fn from(page_slug: String) -> NavigationPage {
        if page_slug.is_empty() {
            NavigationPage::Disabled
        } else {
            NavigationPage::Enabled(page_slug)
        }
    }
}

/// Describes the navigation pages to be used for a category.
#[derive(Debug)]
pub struct NavigationPageSlugs {
    pub top_bar_page: NavigationPage,
    pub side_bar_page: NavigationPage,
}

/// Contains the page wikitexts for the navigation pages for a category.
#[derive(Debug)]
pub struct NavigationPageWikitext {
    pub top_bar_page_wikitext: Option<String>,
    pub side_bar_page_wikitext: Option<String>,
}

/// Contains the page rendered HTML for the navigation pages for a category.
#[derive(Debug)]
pub struct NavigationPageHtml {
    pub compiled_top_bar_html: Option<String>,
    pub compiled_side_bar_html: Option<String>,
}

/// Contains effective forum settings for a site/category pair.
#[allow(dead_code)] // TODO
#[derive(Debug, Copy, Clone)]
pub struct ForumStructureSettings {
    pub max_nest_level: i16,
    pub per_page_discussion: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_page_from_string_distinguishes_disabled_from_enabled() {
        match NavigationPage::from(String::new()) {
            NavigationPage::Disabled => {}
            NavigationPage::Enabled(value) => panic!("empty slug enabled as {value}"),
        }

        match NavigationPage::from(String::from("_default")) {
            NavigationPage::Enabled(value) => assert_eq!(value, "_default"),
            NavigationPage::Disabled => panic!("non-empty slug was disabled"),
        }
    }
}
