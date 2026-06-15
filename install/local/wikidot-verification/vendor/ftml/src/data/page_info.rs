/*
 * data/page_info.rs
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

use super::ScoreValue;
use std::borrow::Cow;

/// Metadata information on the article being rendered.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PageInfo<'a> {
    /// The slug for this page.
    ///
    /// That is, the page component of the URL.
    /// The component portion is already removed.
    pub page: Cow<'a, str>,

    /// The component this page is in, if any.
    ///
    /// If `None`, then the page is within the `_default` category.
    pub category: Option<Cow<'a, str>>,

    /// The slug the site that page is being written for.
    ///
    /// That is, the part of the URL in the form `{slug}.wikijump.com`.
    pub site: Cow<'a, str>,

    /// The title of this page.
    ///
    /// For SCPs this is "SCP-XXXX".
    pub title: Cow<'a, str>,

    /// The alternate title of this page.
    ///
    /// For SCPs this is its series listing title.
    /// If this is None then the main title is used instead.
    pub alt_title: Option<Cow<'a, str>>,

    /// The score this page currently has.
    pub score: ScoreValue,

    /// The current set of tags this page has.
    pub tags: Vec<Cow<'a, str>>,

    /// The language that this page is being rendered for.
    pub language: Cow<'a, str>,
}

impl PageInfo<'_> {
    /// Generate a dummy PageInfo instance for tests.
    #[cfg(test)]
    pub fn dummy() -> Self {
        PageInfo {
            page: cow!("some-page"),
            category: None,
            site: cow!("sandbox"),
            title: cow!("A page for the age"),
            alt_title: None,
            score: ScoreValue::Float(69.0),
            tags: vec![cow!("tale"), cow!("_cc")],
            language: cow!("default"),
        }
    }
}
