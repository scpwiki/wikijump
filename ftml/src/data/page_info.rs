/*
 * data/page_info.rs
 *
 * ftml - Library to parse Wikidot code
 * Copyright (C) 2019-2020 Ammon Smith
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

use std::borrow::Cow;

/// Metadata information on the article being rendered.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PageInfo<'a> {
    /// The slug for this page.
    ///
    /// That is, the page component of the URL.
    pub slug: Cow<'a, str>,

    /// The title of this page.
    ///
    /// For SCPs this is "SCP-XXXX".
    pub title: Cow<'a, str>,

    /// The alternate title of this page.
    ///
    /// For SCPs this is its series listing title.
    /// If this is None then the main title is used instead.
    pub alt_title: Option<Cow<'a, str>>,

    /// The header of this page, if it's setting one.
    ///
    /// For regular pages this is "SCP Foundation".
    /// Previously this value was overriden using custom CSS.
    pub header: Option<Cow<'a, str>>,

    /// The sub-header of this page, if it's setting one.
    ///
    /// For regular pages this is "Secure, Contain, Protect".
    /// Previously this value was overriden using custom CSS.
    pub subheader: Option<Cow<'a, str>>,

    /// The current rating the page has.
    pub rating: f32,

    /// The current set of tags this page has.
    pub tags: Vec<Cow<'a, str>>,
}
