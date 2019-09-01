/*
 * render/info.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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

/// Metadata information on the article being rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageInfo<'a> {
    /// The title of this page.
    ///
    /// For SCPs this is "SCP-XXXX".
    pub title: &'a str,

    /// The alternate title of this page.
    ///
    /// For SCPs this is its series listing title.
    /// If this is None then the main title is used instead.
    pub alt_title: Option<&'a str>,

    /// The header of this page, if it's setting one.
    ///
    /// For regular pages this is "SCP Foundation".
    /// Previously this value was overriden using custom CSS.
    pub header: Option<&'a str>,

    /// The sub-header of this page, if it's setting one.
    ///
    /// For regular pages this is "Secure, Contain, Protect".
    /// Previously this value was overriden using custom CSS.
    pub subheader: Option<&'a str>,

    /// The current rating the page has.
    pub rating: i32,

    /// The current set of tags this page has.
    pub tags: &'a [&'a str],
}

/// Information about a site user. Fetched from the backend.
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub name: String,
    pub id: u64,
    pub avatar: String,
}
