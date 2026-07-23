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

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PageRatingPermission {
    Registered,
    Members,
}

impl PageRatingPermission {
    pub const fn as_storage(self) -> &'static str {
        match self {
            Self::Registered => "registered",
            Self::Members => "members",
        }
    }

    pub fn from_storage(value: &str) -> Option<Self> {
        match value {
            "registered" => Some(Self::Registered),
            "members" => Some(Self::Members),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PageRatingVisibility {
    Visible,
    Anonymous,
}

impl PageRatingVisibility {
    pub const fn as_storage(self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::Anonymous => "anonymous",
        }
    }

    pub fn from_storage(value: &str) -> Option<Self> {
        match value {
            "visible" => Some(Self::Visible),
            "anonymous" => Some(Self::Anonymous),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PageRatingType {
    Plus,
    PlusMinus,
    Stars,
}

impl PageRatingType {
    pub const fn as_storage(self) -> &'static str {
        match self {
            Self::Plus => "plus",
            Self::PlusMinus => "plus_minus",
            Self::Stars => "stars",
        }
    }

    pub const fn vote_store_key(self) -> &'static str {
        match self {
            Self::Plus | Self::PlusMinus => "points",
            Self::Stars => "stars",
        }
    }

    pub fn from_storage(value: &str) -> Option<Self> {
        match value {
            "plus" => Some(Self::Plus),
            "plus_minus" => Some(Self::PlusMinus),
            "stars" => Some(Self::Stars),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub struct PageRatingSettings {
    pub enabled: bool,
    pub permission: PageRatingPermission,
    pub visibility: PageRatingVisibility,
    pub rating_type: PageRatingType,
}

impl Default for PageRatingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            permission: PageRatingPermission::Registered,
            visibility: PageRatingVisibility::Visible,
            rating_type: PageRatingType::PlusMinus,
        }
    }
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

    #[test]
    fn page_rating_storage_values_and_defaults_match_wikidot_contract() {
        let defaults = PageRatingSettings::default();
        assert!(defaults.enabled);
        assert_eq!(defaults.permission, PageRatingPermission::Registered);
        assert_eq!(defaults.visibility, PageRatingVisibility::Visible);
        assert_eq!(defaults.rating_type, PageRatingType::PlusMinus);

        for (stored, value) in [
            ("registered", PageRatingPermission::Registered),
            ("members", PageRatingPermission::Members),
        ] {
            assert_eq!(PageRatingPermission::from_storage(stored), Some(value));
            assert_eq!(value.as_storage(), stored);
        }
        for (stored, value) in [
            ("visible", PageRatingVisibility::Visible),
            ("anonymous", PageRatingVisibility::Anonymous),
        ] {
            assert_eq!(PageRatingVisibility::from_storage(stored), Some(value));
            assert_eq!(value.as_storage(), stored);
        }
        for (stored, value) in [
            ("plus", PageRatingType::Plus),
            ("plus_minus", PageRatingType::PlusMinus),
            ("stars", PageRatingType::Stars),
        ] {
            assert_eq!(PageRatingType::from_storage(stored), Some(value));
            assert_eq!(value.as_storage(), stored);
        }
    }
}
