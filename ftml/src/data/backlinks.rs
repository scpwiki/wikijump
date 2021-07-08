/*
 * render/backlinks.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Backlinks<'a> {
    pub included_pages: Vec<Link<'a>>,
    pub internal_links: Vec<Link<'a>>,
    pub external_links: Vec<Link<'a>>,
}

impl<'a> Backlinks<'a> {
    #[inline]
    pub fn new() -> Self {
        Backlinks::default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Link<'a> {
    /// The string storing the link target.
    pub link: Cow<'a, str>,

    /// Whether this link exists or not.
    ///
    /// If `true`, then the item this link targets exists.
    /// If `false`, then it is missing or "wanted".
    pub exists: bool,
}
